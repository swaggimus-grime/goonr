struct Uniforms {
    wg_size_x: i32,
    wg_size_y: i32,
    wg_size_z: i32,
}

@group(0) @binding(0) var<storage, read> thread_counts: array<i32>;
@group(0) @binding(1) var<storage, read_write> wg_count: array<i32>;

@group(0) @binding(2) var<storage, read> uniforms: Uniforms;

fn ceil_div(a: i32, b: i32) -> i32 {
    return (a + b - 1) / b;
}

@compute
@workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    if global_id.x > 0 {
        return;
    }

    var cx = 1;
    if arrayLength(&thread_counts) >= 1u {
        cx = thread_counts[0];
    }

    var cy = 1;
    if arrayLength(&thread_counts) >= 2u {
        cy = thread_counts[1];
    }

    var cz = 1;
    if arrayLength(&thread_counts) >= 3u {
        cz = thread_counts[2];
    }

    wg_count[0] = ceil_div(cx, uniforms.wg_size_x);
    wg_count[1] = ceil_div(cy, uniforms.wg_size_y);
    wg_count[2] = ceil_div(cz, uniforms.wg_size_z);
}