use ball_tree::BallTree;
use burn::module::{Module, Param, ParamId};
use burn::prelude::Backend;
use burn::tensor::{Tensor, TensorData};
use glam::{Quat, Vec3};

#[derive(Module, Debug)]
pub struct Splats<B: Backend> {
    pub means: Param<Tensor<B, 2>>,
    pub rotation: Param<Tensor<B, 2>>,
    pub log_scales: Param<Tensor<B, 2>>,
    pub sh_coeffs: Param<Tensor<B, 3>>,
    pub raw_opacity: Param<Tensor<B, 1>>,
}

impl<B: Backend> Splats<B> {
    pub fn from_raw(
        means: &[Vec3],
        rotations: Option<&[Quat]>,
        log_scales: Option<&[Vec3]>,
        sh_coeffs: Option<&[f32]>,
        raw_opacities: Option<&[f32]>,
        device: &B::Device,
    ) -> Self {
        let n_splats = means.len();

        let means_tensor: Vec<f32> = means.iter().flat_map(|v| [v.x, v.y, v.z]).collect();
        let means_tensor = Tensor::from_data(TensorData::new(means_tensor, [n_splats, 3]), device);

        let rotations = if let Some(rotations) = rotations {
            // Rasterizer expects quaternions in scalar form.
            let rotations: Vec<f32> = rotations
                .iter()
                .flat_map(|v| [v.w, v.x, v.y, v.z])
                .collect();
            Tensor::from_data(TensorData::new(rotations, [n_splats, 4]), device)
        } else {
            norm_vec(Tensor::random(
                [n_splats, 4],
                burn::tensor::Distribution::Normal(0.0, 1.0),
                device,
            ))
        };

        let log_scales = if let Some(log_scales) = log_scales {
            let log_scales: Vec<f32> = log_scales.iter().flat_map(|v| [v.x, v.y, v.z]).collect();
            Tensor::from_data(TensorData::new(log_scales, [n_splats, 3]), device)
        } else {
            let tree_pos: Vec<[f64; 3]> = means
                .iter()
                .map(|v| [v.x as f64, v.y as f64, v.z as f64])
                .collect();

            let empty = vec![(); tree_pos.len()];
            let tree = BallTree::new(tree_pos.clone(), empty);

            let extents: Vec<_> = tree_pos
                .iter()
                .map(|p| {
                    // Get average of 4 nearest distances.
                    0.5 * tree.query().nn(p).skip(1).take(2).map(|x| x.1).sum::<f64>() / 2.0
                })
                .map(|p| p.max(1e-12))
                .map(|p| p.ln() as f32)
                .collect();

            Tensor::<B, 1>::from_floats(extents.as_slice(), device)
                .reshape([n_splats, 1])
                .repeat_dim(1, 3)
        };

        let sh_coeffs = if let Some(sh_coeffs) = sh_coeffs {
            let n_coeffs = sh_coeffs.len() / n_splats;
            Tensor::from_data(
                TensorData::new(sh_coeffs.to_vec(), [n_splats, n_coeffs / 3, 3]),
                device,
            )
        } else {
            Tensor::<_, 1>::from_floats([0.5, 0.5, 0.5], device)
                .unsqueeze::<3>()
                .repeat_dim(0, n_splats)
        };

        let raw_opacities = if let Some(raw_opacities) = raw_opacities {
            Tensor::from_data(TensorData::new(raw_opacities.to_vec(), [n_splats]), device)
                .require_grad()
        } else {
            Tensor::random(
                [n_splats],
                burn::tensor::Distribution::Uniform(
                    inverse_sigmoid(0.1) as f64,
                    inverse_sigmoid(0.25) as f64,
                ),
                device,
            )
        };

        Self::from_tensor_data(
            means_tensor,
            rotations,
            log_scales,
            sh_coeffs,
            raw_opacities,
        )
    }

    pub fn from_tensor_data(
        means: Tensor<B, 2>,
        rotation: Tensor<B, 2>,
        log_scales: Tensor<B, 2>,
        sh_coeffs: Tensor<B, 3>,
        raw_opacity: Tensor<B, 1>,
    ) -> Self {
        assert_eq!(means.dims()[1], 3, "Means must be 3D");
        assert_eq!(rotation.dims()[1], 4, "Rotation must be 4D");
        assert_eq!(log_scales.dims()[1], 3, "Scales must be 3D");

        Self {
            means: Param::initialized(ParamId::new(), means.detach().require_grad()),
            sh_coeffs: Param::initialized(ParamId::new(), sh_coeffs.detach().require_grad()),
            rotation: Param::initialized(ParamId::new(), rotation.detach().require_grad()),
            raw_opacity: Param::initialized(ParamId::new(), raw_opacity.detach().require_grad()),
            log_scales: Param::initialized(ParamId::new(), log_scales.detach().require_grad()),
        }
    }

    pub fn num_splats(&self) -> u32 {
        self.means.dims()[0] as u32
    }
}


fn norm_vec<B: Backend>(vec: Tensor<B, 2>) -> Tensor<B, 2> {
    let magnitudes =
        Tensor::clamp_min(Tensor::sum_dim(vec.clone().powi_scalar(2), 1).sqrt(), 1e-32);
    vec / magnitudes
}

pub fn inverse_sigmoid(x: f32) -> f32 {
    (x / (1.0 - x)).ln()
}