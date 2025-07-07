use burn::prelude::Config;

#[derive(Config, Debug)]
pub struct TrainConfig {
    #[config(default = 3)]
    pub sh_degree: u32,

    /// Total number of steps to train for.
    #[config(default = 1000)]
    pub total_steps: u32,

    /// Max nr. of splats. This is an upper bound, but the actual final number of splats might be lower than this.
    #[config(default = 10000000)]
    pub max_splats: u32,

    /// Start learning rate for the mean parameters.
    #[config(default = 4e-5)]
    pub lr_mean: f64,

    /// Start learning rate for the mean parameters.
    #[config(default = 4e-7)]
    pub lr_mean_end: f64,

    /// How much noise to add to the mean parameters of low opacity gaussians.
    #[config(default = 1e4)]
    pub mean_noise_weight: f32,

    /// Learning rate for the base SH (RGB) coefficients.
    #[config(default = 3e-3)]
    pub lr_coeffs_dc: f64,

    /// How much to divide the learning rate by for higher SH orders.
    #[config(default = 20.0)]
    pub lr_coeffs_sh_scale: f32,

    /// Learning rate for the opacity parameter.
    #[config(default = 3e-2)]
    pub lr_opac: f64,

    /// Learning rate for the scale parameters.
    #[config(default = 1e-2)]
    pub lr_scale: f64,

    /// Learning rate for the scale parameters.
    #[config(default = 6e-3)]
    pub lr_scale_end: f64,

    /// Learning rate for the rotation parameters.
    #[config(default = 1e-3)]
    pub lr_rotation: f64,

    /// Frequency of 'refinement' where gaussians are replaced and densified. This should
    /// roughly be the number of images it takes to properly "cover" your scene.
    #[config(default = 150)]
    pub refine_every: u32,

    /// Threshold to control splat growth. Lower means faster growth.
    #[config(default = 0.00085)]
    pub growth_grad_threshold: f32,

    /// What fraction of splats that are deemed as needing to grow do actually grow.
    /// Increase this to make splats grow more aggressively.
    #[config(default = 0.1)]
    pub growth_select_fraction: f32,

    /// Period after which splat growth stops.
    #[config(default = 12500)]
    pub growth_stop_iter: u32,

    /// Weight of SSIM loss (compared to l1 loss)
    #[config(default = 0.2)]
    pub ssim_weight: f32,

    /// Weight of the opacity loss.
    #[config(default = 1e-8)]
    pub opac_loss_weight: f32,

    /// Weight of l1 loss on alpha if input view has transparency.
    #[config(default = 0.1)]
    pub match_alpha_weight: f32,
}