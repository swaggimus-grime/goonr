use burn::serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum CameraModel {
    SimplePinhole,
    Pinhole,
    SimpleRadial,
    Radial,
    OpenCV,
    OpenCvFishEye,
    FullOpenCV,
    Fov,
    SimpleRadialFisheye,
    RadialFisheye,
    ThinPrismFisheye,
}

impl CameraModel {
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::SimplePinhole),
            1 => Some(Self::Pinhole),
            2 => Some(Self::SimpleRadial),
            3 => Some(Self::Radial),
            4 => Some(Self::OpenCV),
            5 => Some(Self::OpenCvFishEye),
            6 => Some(Self::FullOpenCV),
            7 => Some(Self::Fov),
            8 => Some(Self::SimpleRadialFisheye),
            9 => Some(Self::RadialFisheye),
            10 => Some(Self::ThinPrismFisheye),
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "SIMPLE_PINHOLE" => Some(Self::SimplePinhole),
            "PINHOLE" => Some(Self::Pinhole),
            "SIMPLE_RADIAL" => Some(Self::SimpleRadial),
            "RADIAL" => Some(Self::Radial),
            "OPENCV" => Some(Self::OpenCV),
            "OPENCV_FISHEYE" => Some(Self::OpenCvFishEye),
            "FULL_OPENCV" => Some(Self::FullOpenCV),
            "FOV" => Some(Self::Fov),
            "SIMPLE_RADIAL_FISHEYE" => Some(Self::SimpleRadialFisheye),
            "RADIAL_FISHEYE" => Some(Self::RadialFisheye),
            "THIN_PRISM_FISHEYE" => Some(Self::ThinPrismFisheye),
            _ => None,
        }
    }

    pub fn num_params(&self) -> usize {
        match self {
            Self::SimplePinhole => 3,
            Self::Pinhole => 4,
            Self::SimpleRadial => 4,
            Self::Radial => 5,
            Self::OpenCV => 8,
            Self::OpenCvFishEye => 8,
            Self::FullOpenCV => 12,
            Self::Fov => 5,
            Self::SimpleRadialFisheye => 4,
            Self::RadialFisheye => 5,
            Self::ThinPrismFisheye => 12,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Camera {
    pub id: i32,
    pub model: CameraModel,
    pub width: u64,
    pub height: u64,
    pub params: Vec<f64>,
}

impl Camera {
    pub fn focal(&self) -> (f64, f64) {
        let x = self.params[0];
        let y = self.params[match self.model {
            CameraModel::SimplePinhole => 0,
            CameraModel::Pinhole => 1,
            CameraModel::SimpleRadial => 0,
            CameraModel::Radial => 0,
            CameraModel::OpenCV => 1,
            CameraModel::OpenCvFishEye => 1,
            CameraModel::FullOpenCV => 1,
            CameraModel::Fov => 1,
            CameraModel::SimpleRadialFisheye => 0,
            CameraModel::RadialFisheye => 0,
            CameraModel::ThinPrismFisheye => 1,
        }];
        (x, y)
    }

    pub fn principal_point(&self) -> glam::Vec2 {
        let x = self.params[match self.model {
            CameraModel::SimplePinhole => 1,
            CameraModel::Pinhole => 2,
            CameraModel::SimpleRadial => 1,
            CameraModel::Radial => 1,
            CameraModel::OpenCV => 2,
            CameraModel::OpenCvFishEye => 2,
            CameraModel::FullOpenCV => 2,
            CameraModel::Fov => 2,
            CameraModel::SimpleRadialFisheye => 1,
            CameraModel::RadialFisheye => 1,
            CameraModel::ThinPrismFisheye => 2,
        }] as f32;
        let y = self.params[match self.model {
            CameraModel::SimplePinhole => 2,
            CameraModel::Pinhole => 3,
            CameraModel::SimpleRadial => 2,
            CameraModel::Radial => 2,
            CameraModel::OpenCV => 3,
            CameraModel::OpenCvFishEye => 3,
            CameraModel::FullOpenCV => 3,
            CameraModel::Fov => 3,
            CameraModel::SimpleRadialFisheye => 2,
            CameraModel::RadialFisheye => 2,
            CameraModel::ThinPrismFisheye => 3,
        }] as f32;
        glam::vec2(x, y)
    }
}