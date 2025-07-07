use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use anyhow::Context;
use async_fn_stream::{StreamEmitter, TryStreamEmitter};
use async_trait::async_trait;
use burn::module::AutodiffModule;
use burn::prelude::Backend;
use burn_cubecl::cubecl::Runtime;
use burn_wgpu::{WgpuDevice, WgpuRuntime};
use rand::SeedableRng;
use tokio_stream::StreamExt;
use dataset::{Dataset, LoadConfig, SceneLoader};
use render::gaussian_splats::{RandomSplatsConfig, Splats};
use render::MainBackend;
use scene_source::Source;
use train::config::TrainConfig;
use train::eval::eval_stats;
use train::train::SplatTrainer;
use crate::config::PipelineConfig;
use crate::eval_export::eval_save_to_disk;
use crate::message::PipelineMessage;
use crate::pipeline_stream::*;
use crate::PipelineError;

pub struct TrainStream {
    load_config: LoadConfig,
    pipeline_config: PipelineConfig,
    train_config: TrainConfig,
    source: Source,
    device: WgpuDevice,
}

impl TrainStream {
    pub fn new(source: Source, device: WgpuDevice) -> Self {
        Self {
            load_config: LoadConfig::new(),
            pipeline_config: PipelineConfig::new(),
            train_config: TrainConfig::new(),
            source,
            device
        }
    }
}

#[async_trait]
impl PipelineStream for TrainStream {
    async fn run(&mut self, emitter: TryStreamEmitter<PipelineMessage, anyhow::Error>) -> anyhow::Result<()> {
        let (mut splat_stream, dataset) = dataset::load_dataset(
            self.source.clone(), self.load_config.clone(), &self.device).await?;
        
        let mut initial_splats = None;

        let estimated_up = dataset.estimate_up();
        
        while let Some(message) = splat_stream.next().await {
            let message = message?;
            let msg = PipelineMessage::ViewSplats {
                // If the metadata has an up axis prefer that, otherwise estimate
                // the up direction.
                up_axis: message.meta.up_axis.or(Some(estimated_up)),
                splats: Box::new(message.splats.clone()),
                frame: 0,
                total_frames: 0,
            };
            emitter.emit(msg).await;
            initial_splats = Some(message.splats);
        }

        let pipeline_config = &self.pipeline_config;
        log::info!("Using seed {}", pipeline_config.seed);
        <MainBackend as Backend>::seed(pipeline_config.seed);
        let mut rng = rand::rngs::StdRng::from_seed([pipeline_config.seed as u8; 32]);
        
        let splats = if let Some(splats) = initial_splats {
            splats
        } else {
            log::info!("Starting with random splat config.");

            // By default, spawn the splats in bounds.
            let bounds = dataset.train.bounds();
            let bounds_extent = bounds.extent.length();
            // Arbitrarily assume area of interest is 0.2 - 0.75 of scene bounds.
            // Somewhat specific to the blender scenes
            let adjusted_bounds = dataset
                .train
                .adjusted_bounds(bounds_extent * 0.25, bounds_extent);
            let config = RandomSplatsConfig::new();

            Splats::from_random_config(&config, adjusted_bounds, &mut rng, &self.device)
        };

        let splats = splats.with_sh_degree(self.train_config.sh_degree);
        let mut splats = splats.into_autodiff();

        let mut eval_scene = dataset.eval;
        let scene_extent = dataset.train.estimate_extent().unwrap_or(1.0);

        let mut train_duration = Duration::from_secs(0);
        let mut dataloader = SceneLoader::new(&dataset.train, 42, &self.device);
        let mut trainer = SplatTrainer::new(&self.train_config, &self.device);

        log::info!("Start training loop.");
        for iter in self.pipeline_config.start_iter..self.train_config.total_steps {
            log::info!("Training iteration {} of {}", iter + 1, self.train_config.total_steps);

            let step_time = Instant::now();

            let batch = dataloader.next_batch().await;
            let (new_splats, stats) = trainer.step(scene_extent, iter, &batch, splats);
            splats = new_splats;
            let (new_splats, refine) = trainer.refine_if_needed(iter, splats).await;
            splats = new_splats;

            #[allow(unused)]
            let export_path = Path::new(&self.pipeline_config.export_path).to_owned();
            
            // We just finished iter 'iter', now starting iter + 1.
            let iter = iter + 1;
            let is_last_step = iter == self.train_config.total_steps;

            // Check if we want to evaluate _next iteration_. Small detail, but this ensures we evaluate
            // before doing a refine.
            if iter % self.pipeline_config.eval_every == 0 || is_last_step {
                if let Some(eval_scene) = eval_scene.as_mut() {
                    let mut psnr = 0.0;
                    let mut ssim = 0.0;
                    let mut count = 0;

                    log::info!("Running evaluation for iteration {iter}");

                    for (i, view) in eval_scene.views.iter().enumerate() {
                        let sample = eval_stats(splats.valid(), view, &self.device)
                            .await
                            .context("Failed to run eval for sample.")?;

                        count += 1;
                        psnr += sample.psnr.clone().into_scalar_async().await;
                        ssim += sample.ssim.clone().into_scalar_async().await;

                        if self.pipeline_config.eval_save_to_disk {
                            let img_name = Path::new(&view.image.path)
                                .file_stem()
                                .expect("No file name for eval view.")
                                .to_string_lossy();
                            let path = Path::new(&export_path)
                                .join(format!("eval_{iter}"))
                                .join(format!("{img_name}.png"));
                            eval_save_to_disk(&sample, &path).await?;
                        }
                    }

                    psnr /= count as f32;
                    ssim /= count as f32;

                    let message = PipelineMessage::EvalResult {
                        iter,
                        avg_psnr: psnr,
                        avg_ssim: ssim,
                    };

                    emitter.emit(message).await;
                }
            }

            let client = WgpuRuntime::client(&self.device);

            // Add up time from this step.
            train_duration += step_time.elapsed();

            // Emit some messages. Important to not count these in the training time (as this might pause).
            if let Some(stats) = refine {
                emitter
                    .emit(PipelineMessage::RefineStep {
                        stats: Box::new(stats),
                        cur_splat_count: splats.num_splats(),
                        iter,
                    })
                    .await;
            }

            // How frequently to update the UI after a training step.
            const UPDATE_EVERY: u32 = 5;
            if iter % UPDATE_EVERY == 0 || is_last_step {
                let message = PipelineMessage::TrainStep {
                    splats: Box::new(splats.valid()),
                    stats: Box::new(stats),
                    iter,
                    total_elapsed: train_duration,
                };
                emitter.emit(message).await;
            }
        }

        emitter.emit(PipelineMessage::Finished).await;
        Ok(())
    }
}
