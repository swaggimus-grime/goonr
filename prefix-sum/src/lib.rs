mod shaders;

use kernel::calc_cube_count;
use kernel::create_tensor;
use kernel::kernel_source_gen;
use burn::tensor::DType;
use burn_cubecl::cubecl::server::Bindings;
use burn_wgpu::WgpuRuntime;
use shaders::prefix_sum_add_scanned_sums;
use shaders::prefix_sum_scan;
use shaders::prefix_sum_scan_sums;

kernel_source_gen!(PrefixSumScan {}, prefix_sum_scan);
kernel_source_gen!(PrefixSumScanSums {}, prefix_sum_scan_sums);
kernel_source_gen!(PrefixSumAddScannedSums {}, prefix_sum_add_scanned_sums);

use burn_wgpu::CubeTensor;

pub fn prefix_sum(input: CubeTensor<WgpuRuntime>) -> CubeTensor<WgpuRuntime> {
    let threads_per_group = shaders::prefix_sum_helpers::THREADS_PER_GROUP as usize;
    let num = input.shape.dims[0];
    let client = &input.client;
    let outputs = create_tensor(input.shape.dims::<1>(), &input.device, client, DType::I32);

    // SAFETY: Kernel has to contain no OOB indexing, bounded loops.
    unsafe {
        client.execute_unchecked(
            PrefixSumScan::task(),
            calc_cube_count([num as u32], PrefixSumScan::WORKGROUP_SIZE),
            Bindings::new().with_buffers(vec![
                input.handle.binding(),
                outputs.handle.clone().binding(),
            ]),
        );
    }

    if num <= threads_per_group {
        return outputs;
    }

    let mut group_buffer = vec![];
    let mut work_size = vec![];
    let mut work_sz = num;
    while work_sz > threads_per_group {
        work_sz = work_sz.div_ceil(threads_per_group);
        group_buffer.push(create_tensor::<1, WgpuRuntime>(
            [work_sz],
            &input.device,
            client,
            DType::I32,
        ));
        work_size.push(work_sz);
    }

    // SAFETY: Kernel has to contain no OOB indexing, bounded loops.
    unsafe {
        client.execute_unchecked(
            PrefixSumScanSums::task(),
            calc_cube_count([work_size[0] as u32], PrefixSumScanSums::WORKGROUP_SIZE),
            Bindings::new().with_buffers(vec![
                outputs.handle.clone().binding(),
                group_buffer[0].handle.clone().binding(),
            ]),
        );
    }

    for l in 0..(group_buffer.len() - 1) {
        // SAFETY: Kernel has to contain no OOB indexing, bounded loops.
        unsafe {
            client.execute_unchecked(
                PrefixSumScanSums::task(),
                calc_cube_count([work_size[l + 1] as u32], PrefixSumScanSums::WORKGROUP_SIZE),
                Bindings::new().with_buffers(vec![
                    group_buffer[l].handle.clone().binding(),
                    group_buffer[l + 1].handle.clone().binding(),
                ]),
            );
        }
    }

    for l in (1..group_buffer.len()).rev() {
        let work_sz = work_size[l - 1];

        // SAFETY: Kernel has to contain no OOB indexing, bounded loops.
        unsafe {
            client.execute_unchecked(
                PrefixSumAddScannedSums::task(),
                calc_cube_count([work_sz as u32], PrefixSumAddScannedSums::WORKGROUP_SIZE),
                Bindings::new().with_buffers(vec![
                    group_buffer[l].handle.clone().binding(),
                    group_buffer[l - 1].handle.clone().binding(),
                ]),
            );
        }
    }

    // SAFETY: Kernel has to contain no OOB indexing, bounded loops.
    unsafe {
        client.execute_unchecked(
            PrefixSumAddScannedSums::task(),
            calc_cube_count(
                [(work_size[0] * threads_per_group) as u32],
                PrefixSumAddScannedSums::WORKGROUP_SIZE,
            ),
            Bindings::new().with_buffers(vec![
                group_buffer[0].handle.clone().binding(),
                outputs.handle.clone().binding(),
            ]),
        );
    }

    outputs
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use crate::prefix_sum;
    use burn::tensor::{Int, Tensor};
    use burn_wgpu::{CubeBackend, WgpuRuntime};

    type Backend = CubeBackend<WgpuRuntime, f32, i32, u32>;

    #[test]
    fn test_sum_tiny() {
        let device = Default::default();
        let keys = Tensor::<Backend, 1, Int>::from_data([1, 1, 1, 1], &device).into_primitive();
        let summed = prefix_sum(keys);
        let summed = Tensor::<Backend, 1, Int>::from_primitive(summed).to_data();
        let summed = summed.as_slice::<i32>().expect("Wrong type");
        assert_eq!(summed.len(), 4);
        assert_eq!(summed, [1, 2, 3, 4]);
    }

    #[test]
    fn test_512_multiple() {
        const ITERS: usize = 1024;
        let mut data = vec![];
        for i in 0..ITERS {
            data.push(90 + i as i32);
        }
        let device = Default::default();
        let keys = Tensor::<Backend, 1, Int>::from_data(data.as_slice(), &device).into_primitive();
        let summed = prefix_sum(keys);
        let summed = Tensor::<Backend, 1, Int>::from_primitive(summed).to_data();
        let prefix_sum_ref: Vec<_> = data
            .into_iter()
            .scan(0, |x, y| {
                *x += y;
                Some(*x)
            })
            .collect();
        for (summed, reff) in summed
            .as_slice::<i32>()
            .expect("Wrong type")
            .iter()
            .zip(prefix_sum_ref)
        {
            assert_eq!(*summed, reff);
        }
    }

    #[test]
    fn test_sum() {
        const ITERS: usize = 512 * 16 + 123;
        let mut data = vec![];
        for i in 0..ITERS {
            data.push(2 + i as i32);
            data.push(0);
            data.push(32);
            data.push(512);
            data.push(30965);
        }

        let device = Default::default();
        let keys = Tensor::<Backend, 1, Int>::from_data(data.as_slice(), &device).into_primitive();
        let summed = prefix_sum(keys);
        let summed = Tensor::<Backend, 1, Int>::from_primitive(summed).to_data();

        let prefix_sum_ref: Vec<_> = data
            .into_iter()
            .scan(0, |x, y| {
                *x += y;
                Some(*x)
            })
            .collect();

        for (summed, reff) in summed
            .as_slice::<i32>()
            .expect("Wrong type")
            .iter()
            .zip(prefix_sum_ref)
        {
            assert_eq!(*summed, reff);
        }
    }
}
