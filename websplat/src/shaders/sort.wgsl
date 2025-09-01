struct VisibleSplat {
    pos: vec4<f32>,
    index: u32,
    depth: f32,
    _pad: vec2<u32>,
};

struct GpuSplat {
    pos: vec3<f32>,
    _pad0: f32,
    log_scales: vec3<f32>,
    _pad1: f32,
    rotation: vec4<f32>,
    color: vec3<f32>,
    opacity: f32,
};

@group(0) @binding(0)
var<storage, read_write> visible: array<VisibleSplat>;
@group(0) @binding(1)
var<uniform> params: vec2<u32>; // k, j

fn swap(i: u32, j: u32) {
    let tmp = visible[i];
    visible[i] = visible[j];
    visible[j] = tmp;
}

@compute @workgroup_size(64)
fn bitonic_step(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let N = arrayLength(&visible);
    if (i >= N) { return; }

    let k = params.x;
    let j = params.y;
    let ixj = i ^ j;

    if (ixj > i) {
        let ascending = (i & k) == 0u;
        let vi = visible[i];
        let vj = visible[ixj];

        if (ascending) {
            if (vi.depth > vj.depth) { swap(i, ixj); }
        } else {
            if (vi.depth < vj.depth) { swap(i, ixj); }
        }
    }
}

// ==== map_to_full: read sorted visible -> write full GpuSplat ordered ====
fn zero_splat() -> GpuSplat {
    return GpuSplat(
        vec3<f32>(0.0,0.0,0.0),
        0.0,
        vec3<f32>(0.0,0.0,0.0),
        0.0,
        vec4<f32>(0.0,0.0,0.0,1.0),
        vec3<f32>(0.0,0.0,0.0),
        0.0
    );
}

@group(0) @binding(0)
var<storage, read> visible_read: array<VisibleSplat>;
@group(0) @binding(1)
var<storage, read> full_splats: array<GpuSplat>;
@group(0) @binding(2)
var<storage, read_write> sorted_splats: array<GpuSplat>;

@compute @workgroup_size(64)
fn map_to_full(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let N = arrayLength(&visible_read);
    if (i >= N) { return; }

    let vs = visible_read[i];
    if (vs.index == 0xffffffffu) {
        sorted_splats[i] = zero_splat();
    } else {
        sorted_splats[i] = full_splats[vs.index];
    }
}
