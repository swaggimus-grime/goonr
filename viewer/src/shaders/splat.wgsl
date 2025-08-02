struct VertexInput {
    @location(0) corner: vec2<f32>,           // From quad [-1, 1]^2
    @location(1) mean: vec3<f32>,
    @location(2) rotation: vec3<f32>,
    @location(3) log_scale: vec3<f32>,
    @location(4) opacity: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) opacity: f32,
};

struct CameraUniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

fn perspective_matrix() -> mat4x4<f32> {
    return camera.view_proj;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    // We'll treat rotation as Euler angles for now (future: 3x3 matrix or quaternion)
    // We'll use log_scale as actual scale = exp(log_scale)
    let scale = exp(input.log_scale);

    // For simplicity, ignore rotation for now and apply scale directly
    let offset = vec3<f32>(
        input.corner.x * scale.x,
        input.corner.y * scale.y,
        0.0
    );

    // Translate quad to center at mean
    let world_pos = input.mean + offset;

    // Simple perspective projection (assuming z in [0, ∞])
    let projection = perspective_matrix(); // We'll define this below

    var out: VertexOutput;
    out.position = projection * vec4<f32>(world_pos, 1.0);
    out.opacity = input.opacity;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, input.opacity); // white splats with transparency
}
