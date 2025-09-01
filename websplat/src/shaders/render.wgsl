struct GPUSplat {
    pos: vec3<f32>,
    _pad0: f32,
    log_scales: vec3<f32>,
    _pad1: f32,
    rotation: vec4<f32>,
    color: vec3<f32>,
    opacity: f32,
};

@group(0) @binding(0)
var<storage, read> splats: array<GPUSplat>;

struct Camera { view_proj: mat4x4<f32> }
@group(0) @binding(1)
var<uniform> camera: Camera;

struct VSIn {
    @location(0) quad_pos: vec2<f32>,
    @builtin(instance_index) idx: u32,
};

struct VSOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) local_uv: vec2<f32>,
    @location(2) scales: vec2<f32>,
    @location(3) opacity: f32
};

fn rotate_quat(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    let u = q.xyz;
    let s = q.w;
    return 2.0 * dot(u,v)*u + (s*s - dot(u,u))*v + 2.0 * s * cross(u,v);
}

@vertex
fn vs_main(input: VSIn) -> VSOut {
    let splat = splats[input.idx];
    let scales = splat.log_scales;

    // Offset quad vertex by scale, rotate, and add position
    var offset = vec3<f32>(input.quad_pos.x*scales.x,
                           input.quad_pos.y*scales.y,
                           0.0);
    offset = rotate_quat(offset, splat.rotation);

    var out: VSOut;
    out.clip_pos = camera.view_proj * vec4<f32>(splat.pos + offset, 1.0);

    // Pass color and opacity
    out.color = splat.color;
    out.opacity = splat.opacity;

    // Normalize quad coordinates to [-1,1] for consistent Gaussian
    out.local_uv = input.quad_pos * 2.0;

    // Pass XY scales for Gaussian calculation
    out.scales = scales.xy;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // Compute squared distance normalized by scale
    let dist2 = (in.local_uv.x*in.local_uv.x)/(in.scales.x*in.scales.x)
              + (in.local_uv.y*in.local_uv.y)/(in.scales.y*in.scales.y);

    // Gaussian weight
    let weight = exp(-2.0 * dist2); // stronger falloff

    // Final alpha
    let alpha = in.opacity * weight;

    return vec4<f32>(in.color, alpha);
}
