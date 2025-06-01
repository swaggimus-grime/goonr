use serde::Serialize;

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
    pub(crate) fn from_id(id: i32) -> Option<Self> {
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

    pub(crate) fn from_name(name: &str) -> Option<Self> {
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

    pub(crate) fn num_params(&self) -> usize {
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