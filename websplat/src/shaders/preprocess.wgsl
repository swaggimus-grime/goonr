//preprocess.wgsl

struct GPUSplat {
    pos: vec3<f32>,
    _pad0: f32,
    log_scales: vec3<f32>,
    _pad1: f32,
    rotation: vec4<f32>,
    color: vec3<f32>,
    opacity: f32,
};

struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<storage, read> input_splats: array<GPUSplat>;

@group(0) @binding(1)
var<uniform> camera: Camera;

struct VisibleSplat {
    pos: vec4<f32>,
    index: u32,
    depth: f32,
    _pad: vec2<u32>,
};

@group(0) @binding(2)
var<storage, read_write> visible_splats: array<VisibleSplat>;

// DrawIndexedIndirect args = [index_count, instance_count, first_index, base_vertex, first_instance]
@group(0) @binding(3)
var<storage, read_write> draw_args: array<atomic<u32>, 5>;

// --- Shared workgroup state ---
var<workgroup> wg_count: atomic<u32>;
var<workgroup> local_indices: array<u32, 256>;

// --- Reset kernel: run once per frame before culling ---
@compute @workgroup_size(1)
fn reset_main() {
    atomicStore(&draw_args[0], 6u);  // index_count = 6
    atomicStore(&draw_args[1], 0u);  // instance_count = 0
    atomicStore(&draw_args[2], 0u);
    atomicStore(&draw_args[3], 0u);
    atomicStore(&draw_args[4], 0u);
}

// --- Main culling + append visible splats ---
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>,
        @builtin(local_invocation_id) local_id: vec3<u32>) {

    let idx = global_id.x;
    let local_idx = local_id.x;

    // Reset per-workgroup counter
    if (local_idx == 0u) {
        atomicStore(&wg_count, 0u);
    }
    workgroupBarrier();

    var is_visible: bool = false;
    var clip_pos: vec4<f32>;

    if (idx < arrayLength(&input_splats)) {
        let splat = input_splats[idx];
        clip_pos = camera.view_proj * vec4<f32>(splat.pos, 1.0);
        let ndc = clip_pos.xyz / clip_pos.w;

        if (ndc.x >= -1.0 && ndc.x <= 1.0 &&
            ndc.y >= -1.0 && ndc.y <= 1.0 &&
            ndc.z >= 0.0  && ndc.z <= 1.0) {
            is_visible = true;
        }
    }

    if (is_visible) {
        let local_offset = atomicAdd(&wg_count, 1u);
        if (local_offset < 256u) {
            local_indices[local_offset] = idx;
        }
    }

    // Uniform barrier
    workgroupBarrier();

    if (local_idx == 0u) {
        let count = atomicLoad(&wg_count);
        let base = atomicAdd(&draw_args[1], count);

        for (var i = 0u; i < count; i = i + 1u) {
            let global_idx = base + i;
            if (global_idx < arrayLength(&visible_splats)) {
                let splat_idx = local_indices[i];
                let splat = input_splats[splat_idx];
                let cpos = camera.view_proj * vec4<f32>(splat.pos, 1.0);
                let ndc = cpos.xyz / cpos.w;

                visible_splats[global_idx].pos = cpos;
                visible_splats[global_idx].index = splat_idx;
                visible_splats[global_idx].depth = ndc.z;
                visible_splats[global_idx]._pad = vec2<u32>(0u, 0u);
            }
        }
    }
}
