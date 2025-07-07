use burn::prelude::Backend;
use render::gaussian_splats::Splats;
use web_cmn::splats::RawSplats;

pub fn splats_from_module<B: Backend>(splats: &Splats<B>) -> RawSplats {
    let means = splats.means.val().into_data().to_vec().unwrap();
    let rotation = splats.rotation.val().into_data().to_vec().unwrap();
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