use serde::{Deserialize, Serialize};
use std::time::Duration;

#[tauri::command]
async fn start_pipeline(app: tauri::AppHandle, zip_path: String) -> Result<(), String> {
  // Spawn the pipeline on a background task so the command returns quickly.
  tokio::spawn(async move {
    // Initialize a default WGPU device for Burn/render.
    let device = render::burn_init_setup().await;
    let source = scene_source::Source::Zip { path: zip_path };

    // Default configs
    let load_config = dataset::config::LoadConfig::new();
    let pipeline_config = pipeline::config::PipelineConfig::new();
    let train_config = train::config::TrainConfig::new();

    // Create a stream we can consume messages from while run() emits to it.
    let stream = async_fn_stream::try_fn_stream(|emitter| async move {
      // Ignore final result; errors will be propagated through the stream if returned.
      let _ = pipeline::train_stream::run(
        source,
        load_config,
        pipeline_config,
        train_config,
        device,
        emitter,
      )
      .await;
      Ok(()) as anyhow::Result<()>
    });

    use tokio_stream::StreamExt;
    tokio::pin!(stream);

    while let Some(item) = stream.next().await {
      match item {
        Ok(msg) => {
          // Map pipeline messages to serializable events and emit to the frontend.
          match msg {
            pipeline::message::PipelineMessage::ViewSplats { up_axis, splats, frame, total_frames } => {
              let payload = ViewSplatsEvent {
                up_axis: up_axis.map(|v| [v.x, v.y, v.z]),
                frame,
                total_frames,
                splat_count: splats.num_splats(),
              };
              let _ = app.emit("pipeline://view_splats", payload);
            }
            pipeline::message::PipelineMessage::TrainStep { splats, stats: _stats, iter, total_elapsed } => {
              let payload = TrainStepEvent {
                iter,
                total_elapsed_secs: total_elapsed.as_secs_f32(),
                splat_count: splats.num_splats(),
              };
              let _ = app.emit("pipeline://train_step", payload);
            }
            pipeline::message::PipelineMessage::RefineStep { stats: _stats, cur_splat_count, iter } => {
              let payload = RefineStepEvent { iter, cur_splat_count };
              let _ = app.emit("pipeline://refine_step", payload);
            }
            pipeline::message::PipelineMessage::EvalResult { iter, avg_psnr, avg_ssim } => {
              let payload = EvalResultEvent { iter, avg_psnr, avg_ssim };
              let _ = app.emit("pipeline://eval_result", payload);
            }
            pipeline::message::PipelineMessage::Finished => {
              let _ = app.emit("pipeline://finished", FinishedEvent {});
            }
            pipeline::message::PipelineMessage::NewSource | pipeline::message::PipelineMessage::StartLoading { .. } => {
              // Currently unused in desktop bridge.
            }
          }
        }
        Err(err) => {
          let _ = app.emit("pipeline://error", ErrorEvent { message: format!("{err:?}") });
        }
      }
    }
  });

  Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewSplatsEvent {
  up_axis: Option<[f32; 3]>,
  frame: u32,
  total_frames: u32,
  splat_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrainStepEvent {
  iter: u32,
  total_elapsed_secs: f32,
  splat_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefineStepEvent {
  iter: u32,
  cur_splat_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvalResultEvent {
  iter: u32,
  avg_psnr: f32,
  avg_ssim: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FinishedEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorEvent { message: String }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![start_pipeline])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
