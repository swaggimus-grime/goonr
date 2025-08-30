use bytemuck::{Pod, Zeroable};
use web_cmn::splats::RawSplats;

#[inline]
fn sigmoid(x: f32) -> f32 { 1.0 / (1.0 + (-x).exp()) }

/// GPU-side splat layout (match WGSL exactly)
/// Layout: vec4 (position.xyz, unused), vec4 (scales.x, scales.y, scales.z, unused),
///         vec4 (rotation.xyzw), vec4 (opacity, color.rgb)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GpuSplat {
    pub position: [f32; 3],
    pub _pos_pad: f32,       // pad to 16
    pub scales: [f32; 3],
    pub _scales_pad: f32,    // pad to 16
    pub rotation: [f32; 4],  // quaternion
    pub color: [f32; 3],     // rgb
    pub opacity: f32,
}

impl GpuSplat {
    /// Convert RawSplats -> GPU splats, using SH DC term as color.
    pub fn vec_from_raw(raw: &RawSplats) -> Vec<GpuSplat> {
        let n = raw.raw_opacity.len();
        let mut out = Vec::with_capacity(n);

        // SH layout: [num, channels, harmonics]
        let channels = raw.sh_coeffs_dims[1];
        let harmonics = raw.sh_coeffs_dims[2];
        let per_splat_coeffs = channels.saturating_mul(harmonics);

        // SH l=0 constant
        const K0: f32 = 0.282_094_791_8_f32;

        for i in 0..n {
            let position = [
                raw.means[i * 3],
                raw.means[i * 3 + 1],
                raw.means[i * 3 + 2],
            ];

            // scales: use all three log_scales -> exp()
            let sx = raw.log_scales[i * 3].exp();
            let sy = raw.log_scales[i * 3 + 1].exp();
            let sz = raw.log_scales[i * 3 + 2].exp();

            let rotation = [
                raw.rotation[i * 4],
                raw.rotation[i * 4 + 1],
                raw.rotation[i * 4 + 2],
                raw.rotation[i * 4 + 3],
            ];

            // opacity (sigmoid)
            let opacity = sigmoid(raw.raw_opacity[i]);

            // color from SH DC (if present)
            let color = if per_splat_coeffs >= (channels * harmonics) && channels >= 3 && harmonics >= 1 {
                let base = i * per_splat_coeffs;
                let stride = harmonics.max(1);
                let r_dc = raw.sh_coeffs[base + 0 * stride + 0] * K0;
                let g_dc = raw.sh_coeffs[base + 1 * stride + 0] * K0;
                let b_dc = raw.sh_coeffs[base + 2 * stride + 0] * K0;
                [sigmoid(r_dc), sigmoid(g_dc), sigmoid(b_dc)]
            } else {
                [1.0, 1.0, 1.0]
            };

            out.push(GpuSplat {
                position,
                _pos_pad: 0.0,
                scales: [sx, sy, sz],
                _scales_pad: 0.0,
                rotation,
                color,
                opacity,
            });
        }

        out
    }
}
