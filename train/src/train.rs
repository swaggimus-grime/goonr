use crate::{
    adam_scaled::{AdamScaled, AdamScaledConfig, AdamState},
    config::TrainConfig,
    msg::{RefineStats, TrainStepStats},
    multinomial::multinomial_sample,
    quat_vec::quaternion_vec_multiply,
    ssim::Ssim,
    stats::RefineRecord,
};

use dataset::scene::SceneBatch;
use render::sh::sh_coeffs_for_degree;
use render::{
    MainBackend,
    gaussian_splats::{Splats, inverse_sigmoid},
};
use render_bwd::burn_glue::SplatForwardDiff;
use burn::{
    backend::{
        Autodiff,
        wgpu::{WgpuDevice, WgpuRuntime},
    },
    lr_scheduler::{
        LrScheduler,
        exponential::{ExponentialLrScheduler, ExponentialLrSchedulerConfig},
    },
    module::ParamId,
    optim::{GradientsParams, Optimizer, adaptor::OptimizerAdaptor, record::AdaptorRecord},
    prelude::Backend,
    tensor::{
        Bool, Distribution, Tensor, TensorData, TensorPrimitive, activation::sigmoid,
        backend::AutodiffBackend, s,
    },
};

use burn_cubecl::cubecl::Runtime;
use glam::Vec3;
use hashbrown::{HashMap, HashSet};
use rand::Rng;
use std::f64::consts::SQRT_2;
use tracing::trace_span;

const MIN_OPACITY: f32 = 0.99 / 255.0;

type OptimizerType =
OptimizerAdaptor<AdamScaled, Splats<Autodiff<MainBackend>>, Autodiff<MainBackend>>;

pub struct SplatTrainer {
    config: TrainConfig,
    sched_mean: ExponentialLrScheduler,
    sched_scale: ExponentialLrScheduler,
    ssim: Ssim<Autodiff<MainBackend>>,
    refine_record: Option<RefineRecord<MainBackend>>,
    optim: Option<OptimizerType>,
}

fn inv_sigmoid<B: Backend>(x: Tensor<B, 1>) -> Tensor<B, 1> {
    (x.clone() / (1.0f32 - x)).log()
}

fn create_default_optimizer() -> OptimizerType {
    AdamScaledConfig::new().with_epsilon(1e-15).init()
}

impl SplatTrainer {
    pub fn new(config: &TrainConfig, device: &WgpuDevice) -> Self {
        const SSIM_WINDOW_SIZE: usize = 11; // Could be configurable but meh, rather keep consistent.
        let ssim = Ssim::new(SSIM_WINDOW_SIZE, 3, device);

        let decay = (config.lr_mean_end / config.lr_mean).powf(1.0 / config.total_steps as f64);
        let lr_mean = ExponentialLrSchedulerConfig::new(config.lr_mean, decay);

        let decay = (config.lr_scale_end / config.lr_scale).powf(1.0 / config.total_steps as f64);
        let lr_scale = ExponentialLrSchedulerConfig::new(config.lr_scale, decay);

        Self {
            config: config.clone(),
            sched_mean: lr_mean.init().expect("Mean lr schedule must be valid."),
            sched_scale: lr_scale.init().expect("Scale lr schedule must be valid."),
            optim: None,
            refine_record: None,
            ssim,
        }
    }

    pub fn step(
        &mut self,
        scene_extent: f32,
        iter: u32,
        batch: &SceneBatch<Autodiff<MainBackend>>,
        splats: Splats<Autodiff<MainBackend>>,
    ) -> (Splats<Autodiff<MainBackend>>, TrainStepStats<MainBackend>) {
        let mut splats = splats;

        let [img_h, img_w, _] = batch.img_tensor.dims();
        let camera = &batch.camera;

        let current_opacity = splats.opacities();
        let (pred_image, aux, refine_weight_holder) = {
            let background = if batch.has_alpha() {
                // For transparent items, do _not_ use a random background color. This could work
                // if we blend the background color with the training view, but makes more sense to just use a black background color.
                Vec3::ZERO
            } else {
                // Generate a uniform background color
                Vec3::new(
                    rand::rng().random(),
                    rand::rng().random(),
                    rand::rng().random(),
                )
            };

            let diff_out = <Autodiff<MainBackend> as SplatForwardDiff<_>>::render_splats(
                camera,
                glam::uvec2(img_w as u32, img_h as u32),
                splats.means.val().into_primitive().tensor(),
                splats.log_scales.val().into_primitive().tensor(),
                splats.rotation.val().into_primitive().tensor(),
                splats.sh_coeffs.val().into_primitive().tensor(),
                current_opacity.clone().into_primitive().tensor(),
                background,
            );
            let img = Tensor::from_primitive(TensorPrimitive::Float(diff_out.img));

            #[cfg(feature = "debug-validation")]
            diff_out.aux.debug_assert_valid();

            (img, diff_out.aux, diff_out.refine_weight_holder)
        };

        let train_t = (iter as f32 / self.config.total_steps as f32).clamp(0.0, 1.0);

        let _span = trace_span!("Calculate losses", sync_burn = true).entered();

        let pred_rgb = pred_image.clone().slice(s![.., .., 0..3]);
        let gt_rgb = batch.img_tensor.clone().slice(s![.., .., 0..3]);

        let l1_rgb = (pred_rgb.clone() - gt_rgb).abs();

        let total_err = if self.config.ssim_weight > 0.0 {
            let gt_rgb = batch.img_tensor.clone().slice(s![.., .., 0..3]);
            let ssim_err = self.ssim.ssim(pred_rgb, gt_rgb);
            l1_rgb * (1.0 - self.config.ssim_weight) - (ssim_err * self.config.ssim_weight)
        } else {
            l1_rgb
        };

        let loss = if batch.has_alpha() {
            let alpha_input = batch.img_tensor.clone().slice(s![.., .., 3..4]);

            if batch.alpha_is_mask {
                (total_err * alpha_input).mean()
            } else {
                let pred_alpha = pred_image.clone().slice(s![.., .., 3..4]);
                total_err.mean()
                    + (alpha_input - pred_alpha).abs().mean() * self.config.match_alpha_weight
            }
        } else {
            total_err.mean()
        };

        let opac_loss_weight = self.config.opac_loss_weight;
        let visible: Tensor<_, 1> =
            Tensor::from_primitive(TensorPrimitive::Float(aux.visible.clone()));

        let loss = if opac_loss_weight > 0.0 {
            // Invisible splats still have a tiny bit of loss. Otherwise,
            // they would never die off.
            let visible = visible.clone() + 1e-3;
            loss + (current_opacity.clone() * visible).sum() * (opac_loss_weight * (1.0 - train_t))
        } else {
            loss
        };

        let mut grads = trace_span!("Backward pass", sync_burn = true).in_scope(|| loss.backward());

        let (lr_mean, lr_rotation, lr_scale, lr_coeffs, lr_opac) = (
            self.sched_mean.step() * scene_extent as f64,
            self.config.lr_rotation,
            // Scale is relative to the scene scale, but the exp() activation function
            // means "offsetting" all values also solves the learning rate scaling.
            self.sched_scale.step(),
            self.config.lr_coeffs_dc,
            self.config.lr_opac,
        );

        let optimizer = self.optim.get_or_insert_with(|| {
            let sh_degree = splats.sh_degree();
            let device = splats.device();

            let coeff_count = sh_coeffs_for_degree(sh_degree) as i32;
            let sh_size = coeff_count;
            let mut sh_lr_scales = vec![1.0];
            for _ in 1..sh_size {
                sh_lr_scales.push(1.0 / self.config.lr_coeffs_sh_scale);
            }
            let sh_lr_scales = Tensor::<_, 1>::from_floats(sh_lr_scales.as_slice(), &device)
                .reshape([1, coeff_count, 1]);

            create_default_optimizer().load_record(HashMap::from([(
                splats.sh_coeffs.id,
                AdaptorRecord::from_state(AdamState {
                    momentum: None,
                    scaling: Some(sh_lr_scales),
                }),
            )]))
        });

        splats = trace_span!("Optimizer step", sync_burn = true).in_scope(|| {
            splats = trace_span!("SH Coeffs step", sync_burn = true).in_scope(|| {
                let grad_coeff =
                    GradientsParams::from_params(&mut grads, &splats, &[splats.sh_coeffs.id]);
                optimizer.step(lr_coeffs, splats, grad_coeff)
            });
            splats = trace_span!("Rotation step", sync_burn = true).in_scope(|| {
                let grad_rot =
                    GradientsParams::from_params(&mut grads, &splats, &[splats.rotation.id]);
                optimizer.step(lr_rotation, splats, grad_rot)
            });
            splats = trace_span!("Scale step", sync_burn = true).in_scope(|| {
                let grad_scale =
                    GradientsParams::from_params(&mut grads, &splats, &[splats.log_scales.id]);
                optimizer.step(lr_scale, splats, grad_scale)
            });
            splats = trace_span!("Mean step", sync_burn = true).in_scope(|| {
                let grad_means =
                    GradientsParams::from_params(&mut grads, &splats, &[splats.means.id]);
                optimizer.step(lr_mean, splats, grad_means)
            });
            splats = trace_span!("Opacity step", sync_burn = true).in_scope(|| {
                let grad_opac =
                    GradientsParams::from_params(&mut grads, &splats, &[splats.raw_opacity.id]);
                optimizer.step(lr_opac, splats, grad_opac)
            });
            splats
        });

        let _housekeep = trace_span!("Housekeeping", sync_burn = true);
        // Get the xy gradient norm from the dummy tensor.
        let refine_weight = refine_weight_holder
            .grad_remove(&mut grads)
            .expect("XY gradients need to be calculated.");

        let device = splats.device();
        let num_splats = splats.num_splats();
        let record = self
            .refine_record
            .get_or_insert_with(|| RefineRecord::new(num_splats, &device));

        record.gather_stats(
            refine_weight,
            glam::uvec2(img_w as u32, img_h as u32),
            aux.global_from_compact_gid.clone(),
            aux.num_visible().into_primitive(),
        );
        drop(_housekeep);

        let mean_noise_weight_scale = self.config.mean_noise_weight * (1.0 - train_t);

        if mean_noise_weight_scale > 0.0 {
            let device = splats.device();
            // Add random noise. Only do this in the growth phase, otherwise
            // let the splats settle in without noise, not much point in exploring regions anymore.
            // trace_span!("Noise means").in_scope(|| {
            let one = Tensor::ones([1], &device);
            let noise_weight = (one - current_opacity.inner())
                .powi_scalar(100)
                .clamp(0.0, 1.0);
            let noise_weight = noise_weight * visible.inner(); // Only noise visible gaussians.
            let noise_weight = noise_weight.unsqueeze_dim(1);

            let samples = quaternion_vec_multiply(
                splats.rotations_normed().inner(),
                Tensor::random(
                    [splats.num_splats() as usize, 3],
                    Distribution::Normal(0.0, 1.0),
                    &device,
                ) * splats.scales().inner(),
            );

            let noise_weight = noise_weight * (lr_mean as f32 * mean_noise_weight_scale);
            splats.means = splats
                .means
                .map(|m| Tensor::from_inner(m.inner() + samples * noise_weight).require_grad());
        }

        let stats = TrainStepStats {
            pred_image: pred_image.inner(),
            num_visible: aux.num_visible().inner(),
            num_intersections: aux.num_intersections().inner(),
            loss: loss.inner(),
            lr_mean,
            lr_rotation,
            lr_scale,
            lr_coeffs,
            lr_opac,
        };

        (splats, stats)
    }

    pub async fn refine_if_needed(
        &mut self,
        iter: u32,
        splats: Splats<Autodiff<MainBackend>>,
    ) -> (Splats<Autodiff<MainBackend>>, Option<RefineStats>) {
        if iter == 0 || iter % self.config.refine_every != 0 {
            return (splats, None);
        }

        let device = splats.means.device();
        let client = WgpuRuntime::client(&device);
        client.memory_cleanup();

        // If not refining, update splat to step with gradients applied.
        // Prune dead splats. This ALWAYS happen even if we're not "refining" anymore.
        let mut record = self
            .optim
            .take()
            .expect("Can only refine after optimizer is initialized")
            .to_record();
        let refiner = self
            .refine_record
            .take()
            .expect("Can only refine if refine stats are initialized");
        let alpha_mask = splats
            .raw_opacity
            .val()
            .inner()
            .lower_elem(inverse_sigmoid(MIN_OPACITY));

        let (mut splats, refiner, pruned_count) =
            prune_points(splats, &mut record, refiner, alpha_mask).await;
        let mut add_indices = HashSet::new();

        // Replace dead gaussians if we're still refining.
        if pruned_count > 0 {
            // Sample from random opacities.
            let resampled_weights = splats.opacities().inner();
            let resampled_weights = resampled_weights
                .into_data_async()
                .await
                .into_vec::<f32>()
                .expect("Failed to read weights");
            let resampled_inds = multinomial_sample(&resampled_weights, pruned_count);
            add_indices.extend(resampled_inds);
        }

        if iter < self.config.growth_stop_iter {
            let above_threshold = refiner
                .refine_weight_norm
                .clone()
                .greater_elem(self.config.growth_grad_threshold)
                .int();
            let threshold_count = above_threshold.clone().sum().into_scalar_async().await as u32;

            let grow_count =
                (threshold_count as f32 * self.config.growth_select_fraction).round() as u32;

            let sample_high_grad = grow_count.saturating_sub(pruned_count);

            // Only grow to the max nr. of splats.
            let cur_splats = splats.num_splats() + add_indices.len() as u32;
            let grow_count = sample_high_grad.min(self.config.max_splats - cur_splats);

            // If still growing, sample from indices which are over the threshold.
            if grow_count > 0 {
                let weights = above_threshold.float() * refiner.refine_weight_norm;
                let weights = weights
                    .into_data_async()
                    .await
                    .into_vec::<f32>()
                    .expect("Failed to read weights");
                let growth_inds = multinomial_sample(&weights, grow_count);
                add_indices.extend(growth_inds);
            }
        }

        let refine_count = add_indices.len();

        if refine_count > 0 {
            let refine_inds = Tensor::from_data(
                TensorData::new(add_indices.into_iter().collect(), [refine_count]),
                &device,
            );

            let cur_means = splats.means.val().inner().select(0, refine_inds.clone());
            let cur_rots = splats
                .rotations_normed()
                .inner()
                .select(0, refine_inds.clone());
            let cur_log_scale = splats
                .log_scales
                .val()
                .inner()
                .select(0, refine_inds.clone());
            let cur_coeff = splats
                .sh_coeffs
                .val()
                .inner()
                .select(0, refine_inds.clone());
            let cur_raw_opac = splats
                .raw_opacity
                .val()
                .inner()
                .select(0, refine_inds.clone());

            // The amount to offset the scale and opacity should maybe depend on how far away we have sampled these gaussians,
            // but a fixed amount seems to work ok. The only note is that divide by _less_ than SQRT(2) seems to exponentially
            // blow up, as more 'mass' is added each refine.
            let scale_div = Tensor::ones_like(&cur_log_scale) * SQRT_2.ln();

            let one = Tensor::ones([1], &device);
            let cur_opac = sigmoid(cur_raw_opac.clone());
            let new_opac = one.clone() - (one - cur_opac).sqrt();
            let new_raw_opac = inv_sigmoid(new_opac.clamp(1e-24, 1.0 - 1e-24));

            // Scatter needs [N, 3] indices for means and scales.
            let refine_inds_2d = refine_inds.clone().unsqueeze_dim(1).repeat_dim(1, 3);

            let samples = quaternion_vec_multiply(
                cur_rots.clone(),
                Tensor::random([refine_count, 3], Distribution::Normal(0.0, 0.5), &device)
                    * cur_log_scale.clone().exp(),
            );

            // Shrink & offset existing splats.
            splats.means = splats.means.map(|m| {
                let new_means = m
                    .inner()
                    .scatter(0, refine_inds_2d.clone(), -samples.clone());
                Tensor::from_inner(new_means).require_grad()
            });
            splats.log_scales = splats.log_scales.map(|s| {
                let new_scales = s
                    .inner()
                    .scatter(0, refine_inds_2d.clone(), -scale_div.clone());
                Tensor::from_inner(new_scales).require_grad()
            });
            splats.raw_opacity = splats.raw_opacity.map(|m| {
                let difference = new_raw_opac.clone() - cur_raw_opac.clone();
                let new_opacities = m.inner().scatter(0, refine_inds.clone(), difference);
                Tensor::from_inner(new_opacities).require_grad()
            });

            // Concatenate new splats.
            let sh_dim = splats.sh_coeffs.dims()[1];
            splats = map_splats_and_opt(
                splats,
                &mut record,
                |x| Tensor::cat(vec![x, cur_means + samples], 0),
                |x| Tensor::cat(vec![x, cur_rots], 0),
                |x| Tensor::cat(vec![x, cur_log_scale - scale_div], 0),
                |x| Tensor::cat(vec![x, cur_coeff], 0),
                |x| Tensor::cat(vec![x, new_raw_opac], 0),
                |x| Tensor::cat(vec![x, Tensor::zeros([refine_count, 3], &device)], 0),
                |x| Tensor::cat(vec![x, Tensor::zeros([refine_count, 4], &device)], 0),
                |x| Tensor::cat(vec![x, Tensor::zeros([refine_count, 3], &device)], 0),
                |x| {
                    Tensor::cat(
                        vec![x, Tensor::zeros([refine_count, sh_dim, 3], &device)],
                        0,
                    )
                },
                |x| Tensor::cat(vec![x, Tensor::zeros([refine_count], &device)], 0),
            );
        }

        self.optim = Some(create_default_optimizer().load_record(record));

        client.memory_cleanup();

        (
            splats,
            Some(RefineStats {
                num_added: refine_count as u32,
                num_pruned: pruned_count,
            }),
        )
    }
}

fn map_splats_and_opt(
    mut splats: Splats<Autodiff<MainBackend>>,
    record: &mut HashMap<ParamId, AdaptorRecord<AdamScaled, Autodiff<MainBackend>>>,
    map_mean: impl FnOnce(Tensor<MainBackend, 2>) -> Tensor<MainBackend, 2>,
    map_rotation: impl FnOnce(Tensor<MainBackend, 2>) -> Tensor<MainBackend, 2>,
    map_scale: impl FnOnce(Tensor<MainBackend, 2>) -> Tensor<MainBackend, 2>,
    map_coeffs: impl FnOnce(Tensor<MainBackend, 3>) -> Tensor<MainBackend, 3>,
    map_opac: impl FnOnce(Tensor<MainBackend, 1>) -> Tensor<MainBackend, 1>,

    map_opt_mean: impl Fn(Tensor<MainBackend, 2>) -> Tensor<MainBackend, 2>,
    map_opt_rotation: impl Fn(Tensor<MainBackend, 2>) -> Tensor<MainBackend, 2>,
    map_opt_scale: impl Fn(Tensor<MainBackend, 2>) -> Tensor<MainBackend, 2>,
    map_opt_coeffs: impl Fn(Tensor<MainBackend, 3>) -> Tensor<MainBackend, 3>,
    map_opt_opac: impl Fn(Tensor<MainBackend, 1>) -> Tensor<MainBackend, 1>,
) -> Splats<Autodiff<MainBackend>> {
    splats.means = splats
        .means
        .map(|x| Tensor::from_inner(map_mean(x.inner())).require_grad());
    map_opt(splats.means.id, record, &map_opt_mean);

    splats.rotation = splats
        .rotation
        .map(|x| Tensor::from_inner(map_rotation(x.inner())).require_grad());
    map_opt(splats.rotation.id, record, &map_opt_rotation);

    splats.log_scales = splats
        .log_scales
        .map(|x| Tensor::from_inner(map_scale(x.inner())).require_grad());
    map_opt(splats.log_scales.id, record, &map_opt_scale);

    splats.sh_coeffs = splats
        .sh_coeffs
        .map(|x| Tensor::from_inner(map_coeffs(x.inner())).require_grad());
    map_opt(splats.sh_coeffs.id, record, &map_opt_coeffs);

    splats.raw_opacity = splats
        .raw_opacity
        .map(|x| Tensor::from_inner(map_opac(x.inner())).require_grad());
    map_opt(splats.raw_opacity.id, record, &map_opt_opac);

    splats
}

fn map_opt<B: AutodiffBackend, const D: usize>(
    param_id: ParamId,
    record: &mut HashMap<ParamId, AdaptorRecord<AdamScaled, B>>,
    map_opt: &impl Fn(Tensor<B::InnerBackend, D>) -> Tensor<B::InnerBackend, D>,
) {
    let mut state: AdamState<_, D> = record
        .remove(&param_id)
        .expect("failed to get optimizer record")
        .into_state();

    state.momentum = state.momentum.map(|mut moment| {
        moment.moment_1 = map_opt(moment.moment_1);
        moment.moment_2 = map_opt(moment.moment_2);
        moment
    });

    record.insert(param_id, AdaptorRecord::from_state(state));
}

// Prunes points based on the given mask.
//
// Args:
//   mask: bool[n]. If True, prune this Gaussian.
async fn prune_points(
    mut splats: Splats<Autodiff<MainBackend>>,
    record: &mut HashMap<ParamId, AdaptorRecord<AdamScaled, Autodiff<MainBackend>>>,
    mut refiner: RefineRecord<MainBackend>,
    prune: Tensor<MainBackend, 1, Bool>,
) -> (
    Splats<Autodiff<MainBackend>>,
    RefineRecord<MainBackend>,
    u32,
) {
    assert_eq!(
        prune.dims()[0] as u32,
        splats.num_splats(),
        "Prune mask must have same number of elements as splats"
    );

    let prune_count = prune.dims()[0];
    if prune_count == 0 {
        return (splats, refiner, 0);
    }

    let valid_inds = prune.bool_not().argwhere_async().await;

    if valid_inds.dims()[0] == 0 {
        log::warn!("Trying to create empty splat!");
        return (splats, refiner, 0);
    }

    let start_splats = splats.num_splats();
    let new_points = valid_inds.dims()[0] as u32;
    if new_points < start_splats {
        let valid_inds = valid_inds.squeeze(1);
        splats = map_splats_and_opt(
            splats,
            record,
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
            |x| x.select(0, valid_inds.clone()),
        );
        refiner = refiner.keep(valid_inds);
    }
    (splats, refiner, start_splats - new_points)
}