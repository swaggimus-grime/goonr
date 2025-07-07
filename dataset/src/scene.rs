use std::sync::Arc;
use ::image::DynamicImage;
use burn::prelude::{Backend, Tensor, TensorData};
use glam::{vec3, Affine3A, Vec3};
pub(crate) use crate::scene::image::ImageFile;

mod image;
pub mod splat;
mod loader;

pub use loader::SceneLoader;
use render::bounding_box::BoundingBox;
use render::camera::Camera;

#[derive(Clone)]
pub struct SceneView {
    pub image: ImageFile,
    pub camera: Camera,
}

#[derive(Clone)]
pub struct Scene {
    pub views: Arc<Vec<SceneView>>,
}

impl Scene {
    pub fn new(views: Vec<SceneView>) -> Self {
        Self {
            views: Arc::new(views),
        }
    }

    // Returns the extent of the cameras in the scene.
    pub fn bounds(&self) -> BoundingBox {
        self.adjusted_bounds(0.0, 0.0)
    }

    // Returns the extent of the cameras in the scene, taking into account
    // the near and far plane of the cameras.
    pub fn adjusted_bounds(&self, cam_near: f32, cam_far: f32) -> BoundingBox {
        let (min, max) = self.views.iter().fold(
            (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
            |(min, max), view| {
                let cam = &view.camera;
                let pos1 = cam.position + cam.rotation * Vec3::Z * cam_near;
                let pos2 = cam.position + cam.rotation * Vec3::Z * cam_far;
                (min.min(pos1).min(pos2), max.max(pos1).max(pos2))
            },
        );
        BoundingBox::from_min_max(min, max)
    }

    pub fn get_nearest_view(&self, reference: Affine3A) -> Option<usize> {
        self.views
            .iter()
            .enumerate() // This will give us (index, view) pairs
            .min_by(|(_, a), (_, b)| {
                let score_a = camera_distance_penalty(a.camera.local_to_world(), reference);
                let score_b = camera_distance_penalty(b.camera.local_to_world(), reference);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, _)| index) // We return the index instead of the camera
    }

    pub fn estimate_extent(&self) -> Option<f32> {
        if self.views.len() < 5 {
            None
        } else {
            // TODO: This is really sensitive to outliers.
            let bounds = self.bounds();
            let smallest = find_two_smallest(bounds.extent * 2.0);
            Some(smallest.0.hypot(smallest.1))
        }
    }
}

fn camera_distance_penalty(cam_local_to_world: Affine3A, reference: Affine3A) -> f32 {
    let mut penalty = 0.0;
    for off_x in [-1.0, 0.0, 1.0] {
        for off_y in [-1.0, 0.0, 1.0] {
            let offset = vec3(off_x, off_y, 1.0);
            let cam_pos = cam_local_to_world.transform_point3(offset);
            let ref_pos = reference.transform_point3(offset);
            penalty += (cam_pos - ref_pos).length();
        }
    }
    penalty
}

fn find_two_smallest(v: Vec3) -> (f32, f32) {
    let mut arr = v.to_array();
    arr.sort_by(|a, b| a.partial_cmp(b).expect("NaN"));
    (arr[0], arr[1])
}

#[derive(Clone, Debug)]
pub struct SceneBatch<B: Backend> {
    pub img_tensor: Tensor<B, 3>,
    pub alpha_is_mask: bool,
    pub camera: Camera,
}

impl<B: Backend> SceneBatch<B> {
    pub fn has_alpha(&self) -> bool {
        self.img_tensor.shape().dims[2] == 4
    }
}

// Converts an image to a train sample. The tensor will be a floating point image with a [0, 1] image.
//
// This assume the input image has un-premultiplied alpha, whereas the output has pre-multiplied alpha.
pub fn view_to_sample_image(image: DynamicImage, alpha_is_mask: bool) -> DynamicImage {
    if image.color().has_alpha() && !alpha_is_mask {
        let mut rgba_bytes = image.to_rgba8();

        // Assume image has un-multiplied alpha and convert it to pre-multiplied.
        // Perform multiplication in byte space before converting to float.
        for pixel in rgba_bytes.chunks_exact_mut(4) {
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            pixel[0] = ((r as u16 * a as u16 + 127) / 255) as u8;
            pixel[1] = ((g as u16 * a as u16 + 127) / 255) as u8;
            pixel[2] = ((b as u16 * a as u16 + 127) / 255) as u8;
            pixel[3] = a;
        }
        DynamicImage::ImageRgba8(rgba_bytes)
    } else {
        image
    }
}

pub fn sample_to_tensor<B: Backend>(sample: &DynamicImage, device: &B::Device) -> Tensor<B, 3> {
    let (w, h) = (sample.width(), sample.height());
    let data = if sample.color().has_alpha() {
        TensorData::new(sample.to_rgba32f().into_vec(), [h as usize, w as usize, 4])
    } else {
        TensorData::new(sample.to_rgb32f().into_vec(), [h as usize, w as usize, 3])
    };
    Tensor::from_data(data, device)
}