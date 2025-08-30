use burn::prelude::Backend;
use render::gaussian_splats::Splats;
use web_cmn::splats::RawSplats;

pub fn splats_from_module<B: Backend>(splats: &Splats<B>) -> RawSplats {
    let means = splats.means.val().into_data().to_vec().unwrap();
    let rotation_data = splats.rotations_normed().into_data().to_vec().unwrap(); // Use normalized rotations
    let n_splats = splats.means.dims()[0];

    // Reorder rotation from [w, x, y, z] to [x, y, z, w] for each splat
    let mut rotation = Vec::with_capacity(rotation_data.len());
    for i in 0..n_splats {
        let base = i * 4;
        let w = rotation_data[base];
        let x = rotation_data[base + 1];
        let y = rotation_data[base + 2];
        let z = rotation_data[base + 3];
        rotation.extend_from_slice(&[x, y, z, w]); // Reorder to [x, y, z, w]
    }
    let log_scales = splats.log_scales.val().into_data().to_vec().unwrap();
    let raw_opacity = splats.raw_opacity.val().into_data().to_vec().unwrap();

    let sh_coeffs_tensor = splats.sh_coeffs.val();
    let sh_coeffs_data = sh_coeffs_tensor.clone().into_data();
    let sh_coeffs = sh_coeffs_data.to_vec().unwrap();
    let sh_coeffs_dims = sh_coeffs_data.shape;

    RawSplats {
        means,
        rotation,
        log_scales,
        raw_opacity,
        sh_coeffs,
        sh_coeffs_dims: [sh_coeffs_dims[0], sh_coeffs_dims[1], sh_coeffs_dims[2]],
    }
}