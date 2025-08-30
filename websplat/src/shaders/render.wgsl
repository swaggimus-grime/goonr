struct GPUSplat {
    pos: vec3<f32>,
    _pod_pad: f32,
    scales: vec3<f32>,
    _scales_pad: f32,
    rotation: vec4<f32>, // quaternion
    color: vec3<f32>,
    opacity: f32,
};

@group(0) @binding(0)
var<storage, read> splats: array<GPUSplat>;

struct Camera {
    view_proj: mat4x4<f32>
};

@group(0) @binding(1)
var<uniform> camera: Camera;

// Quad vertex input
struct VSIn {
    @location(0) quad_pos: vec2<f32>,
    @builtin(instance_index) idx: u32
};

struct VSOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

// Quaternion rotate function
fn rotate_quat(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    let u = q.xyz;
    let s = q.w;

    return 2.0 * dot(u, v) * u
         + (s*s - dot(u, u)) * v
         + 2.0 * s * cross(u, v);
}

@vertex
fn vs_main(input: VSIn) -> VSOut {
    let splat = splats[input.idx];

    // Quad offset in 3D
    var offset = vec3<f32>(input.quad_pos.x * splat.scales.x,
                           input.quad_pos.y * splat.scales.y,
                           0.0);

    // Apply rotation
    offset = rotate_quat(offset, splat.rotation);

    var out: VSOut;
    out.clip_pos = camera.view_proj * vec4<f32>(splat.pos + offset, 1.0);
    out.color = splat.color * splat.opacity;
    return out;
}

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
