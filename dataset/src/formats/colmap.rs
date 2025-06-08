mod point;
mod image;
mod camera;
mod input;
mod parse;

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::sync::Arc;
use async_fn_stream::try_fn_stream;
use burn::backend::wgpu::WgpuDevice;
use glam::Vec3;
use log::info;
use render::sh::rgb_to_sh;
use render::splat::Splats;
use crate::config::LoadConfig;
use crate::Dataset;
use crate::filesystem::Filesystem;
use crate::error::{DatasetError, FormatError, Result};
use crate::formats::colmap::camera::Camera;
use crate::formats::colmap::image::Image;
use crate::formats::colmap::input::{InputFile, InputType};
use crate::formats::colmap::parse::ImagesParser;
use crate::formats::DataStream;
use crate::scene::{ImageFile, SceneView};
use crate::scene::splat::{ParseMetadata, SplatMessage};

pub async fn load(fs: Arc<Filesystem>, config: LoadConfig, device: &WgpuDevice) -> Result<(DataStream<SplatMessage>, Dataset)> {
    let (cam_path, img_path, is_bin) = if let Some(path) = fs.file_ending_in("cameras.bin") {
        let path = path.parent().expect("unreachable");
        (path.join("cameras.bin"), path.join("images.bin"), true)
    } else if let Some(path) = fs.file_ending_in("cameras.txt") {
        let path = path.parent().expect("unreachable");
        (path.join("cameras.txt"), path.join("images.txt"), false)
    } else {
        return Err(DatasetError::from(FormatError::Io(String::from("Camera file could be found"))));
    };
    
    let cam_model_data = InputFile::new(cam_path, InputType::Cameras, is_bin).parse().await.unwrap().as_cameras().unwrap();
    let img_infos = InputFile::new(img_path, InputType::Images, is_bin).parse().await.unwrap().as_images().unwrap();

    let mut img_info_list = img_infos.into_iter().collect::<Vec<_>>();
    img_info_list.sort_by_key(|key_img| key_img.1.name.clone());
    
    let (train_views, eval_views) = create_views(fs.clone(), &cam_model_data, &img_info_list, &config).await?;

    let load_args = config.clone();
    let fs = fs.clone();
    let device = device.clone();
    let init_stream = try_fn_stream(|emitter| async move {
        let points_path = fs.file_ending_in("points3d.txt")
            .or_else(|| fs.file_ending_in("points3d.bin"));

        let Some(points_path) = points_path else {
            return Ok(());
        };

        let is_binary = matches!(
            points_path.extension().and_then(|p| p.to_str()),
            Some("bin")
        );

        // Extract COLMAP sfm points.
        let points_data = InputFile::new(points_path, InputType::Images, is_binary).parse().await.unwrap().as_points();

        // Ignore empty points data.
        if let Some(points_data) = points_data {
            if !points_data.is_empty() {
                log::info!("Starting from colmap points {}", points_data.len());

                // The ply importer handles subsampling normally. Here just
                // do it manually, maybe nice to unify at some point.
                let step = load_args.subsample_points.unwrap_or(1) as usize;

                let positions: Vec<Vec3> =
                    points_data.values().step_by(step).map(|p| p.xyz).collect();
                let colors: Vec<f32> = points_data
                    .values()
                    .step_by(step)
                    .flat_map(|p| {
                        let sh = rgb_to_sh(glam::vec3(
                            p.rgb[0] as f32 / 255.0,
                            p.rgb[1] as f32 / 255.0,
                            p.rgb[2] as f32 / 255.0,
                        ));
                        [sh.x, sh.y, sh.z]
                    })
                    .collect();

                let init_splat =
                    Splats::from_raw(&positions, None, None, Some(&colors), None, &device);
                emitter
                    .emit(SplatMessage {
                        meta: ParseMetadata {
                            up_axis: None,
                            total_splats: init_splat.num_splats(),
                            frame_count: 1,
                            current_frame: 0,
                        },
                        splats: init_splat,
                    })
                    .await;
            }
        }

        Ok(())
    });

    Ok((
        Box::pin(init_stream),
        Dataset::from_views(train_views, eval_views),
    ))
}

async fn create_views(fs: Arc<Filesystem>, cam_model_data: &HashMap<i32, Camera>, img_info_list: &Vec<(i32, Image)>, config: &LoadConfig) -> Result<(Vec<SceneView>, Vec<SceneView>)> {
    let mut train_views = vec![];
    let mut eval_views = vec![];

    for (i, (_img_id, img_info)) in img_info_list
        .into_iter()
        .take(config.max_frames.unwrap_or(usize::MAX))
        .step_by(config.subsample_frames.unwrap_or(1) as usize)
        .enumerate()
    {
        let cam_data = cam_model_data[&img_info.camera_id].clone();

        // Create a future to handle loading the image.
        let focal = cam_data.focal();

        let fovx = render::camera::focal_to_fov(focal.0, cam_data.width as u32);
        let fovy = render::camera::focal_to_fov(focal.1, cam_data.height as u32);

        let center = cam_data.principal_point();
        let center_uv = center / glam::vec2(cam_data.width as f32, cam_data.height as f32);

        // Convert w2c to c2w.
        let world_to_cam = glam::Affine3A::from_rotation_translation(img_info.quat, img_info.tvec);
        let cam_to_world = world_to_cam.inverse();
        let (_, quat, translation) = cam_to_world.to_scale_rotation_translation();

        let camera = render::Camera::new(translation, quat, fovx, fovy, center_uv);

        let img_file = if let Some(img_path) = fs.file_ending_in(&img_info.name) {
            Ok(ImageFile::new(img_path.as_path(), config.max_resolution))
        } else {
            Err(DatasetError::from(FormatError::Io(format!("Image file {} not found", &img_info.name))))
        }?.or_else(|err| {Err(DatasetError::from(FormatError::InvalidImage(err)))});

        let view = SceneView {
            camera,
            image: img_file?
        };

        if let Some(eval_period) = config.eval_split_every {
            if i % eval_period == 0 {
                eval_views.push(view);
            } else {
                train_views.push(view);
            }
        } else {
            train_views.push(view);
        }
    }
    
    Ok((train_views, eval_views))
}