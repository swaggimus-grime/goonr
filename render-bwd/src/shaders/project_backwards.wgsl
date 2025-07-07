#import helpers;
#import grads;

@group(0) @binding(0) var<storage, read> uniforms: helpers::RenderUniforms;

@group(0) @binding(1) var<storage, read> means: array<helpers::PackedVec3>;
@group(0) @binding(2) var<storage, read> log_scales: array<helpers::PackedVec3>;
@group(0) @binding(3) var<storage, read> quats: array<vec4f>;

@group(0) @binding(4) var<storage, read> global_from_compact_gid: array<i32>;

@group(0) @binding(5) var<storage, read> v_grads: array<f32>;

@group(0) @binding(6) var<storage, read_write> v_means: array<helpers::PackedVec3>;
@group(0) @binding(7) var<storage, read_write> v_scales: array<helpers::PackedVec3>;
@group(0) @binding(8) var<storage, read_write> v_quats: array<vec4f>;

fn normalize_vjp(quat: vec4f) -> mat4x4f {
    let quat_sqr = quat * quat;
    let quat_len_sqr = dot(quat, quat);
    let quat_len = sqrt(quat_len_sqr);

    let cross_complex = -quat.xyz * quat.yzx;
    let cross_scalar = -quat.xyz * quat.w;

    return mat4x4f(
        vec4f(quat_len_sqr - quat_sqr.x, cross_complex.x, cross_complex.z, cross_scalar.x),
        vec4f(cross_complex.x, quat_len_sqr - quat_sqr.y, cross_complex.y, cross_scalar.y),
        vec4f(cross_complex.z, cross_complex.y, quat_len_sqr - quat_sqr.z, cross_scalar.z),
        vec4f(cross_scalar.x, cross_scalar.y, cross_scalar.z, quat_len_sqr - quat_sqr.w),
    ) * (1.0 / (quat_len * quat_len_sqr));
}

fn quat_to_mat_vjp(quat: vec4f, v_R: mat3x3f) -> vec4f {
    let w = quat.x;
    let x = quat.y;
    let y = quat.z;
    let z = quat.w;

    return vec4f(
        // w element stored in x field
        (
            x * (v_R[1][2] - v_R[2][1]) + y * (v_R[2][0] - v_R[0][2]) +
            z * (v_R[0][1] - v_R[1][0])
        ),
        // x element in y field
        (
            -2.f * x * (v_R[1][1] + v_R[2][2]) + y * (v_R[0][1] + v_R[1][0]) +
            z * (v_R[0][2] + v_R[2][0]) + w * (v_R[1][2] - v_R[2][1])
        ),
        // y element in z field
        (
            x * (v_R[0][1] + v_R[1][0]) - 2.f * y * (v_R[0][0] + v_R[2][2]) +
            z * (v_R[1][2] + v_R[2][1]) + w * (v_R[2][0] - v_R[0][2])
        ),
        // z element in w field
        (
            x * (v_R[0][2] + v_R[2][0]) + y * (v_R[1][2] + v_R[2][1]) -
            2.f * z * (v_R[0][0] + v_R[1][1]) + w * (v_R[0][1] - v_R[1][0])
        )
    ) * 2.0;
}

fn inverse_vjp(Minv: mat2x2f, v_Minv: mat2x2f) -> mat2x2f {
    // P = M^-1
    // df/dM = -P * df/dP * P
    return mat2x2f(-Minv[0], -Minv[1]) * v_Minv * Minv;
}

fn outer_product(a: vec3<f32>, b: vec3<f32>) -> mat3x3<f32> {
    return mat3x3f(
        a.x * b.x, a.x * b.y, a.x * b.z,
        a.y * b.x, a.y * b.y, a.y * b.z,
        a.z * b.x, a.z * b.y, a.z * b.z
    );
}

fn persp_proj_vjp(
    J: mat3x2f,
    // fwd inputs
    mean3d: vec3f,
    cov3d: mat3x3f,
    focal: vec2f,
    pixel_center: vec2f,
    img_size: vec2u,
    // grad outputs
    v_cov2d: mat2x2f,
    v_mean2d: vec2f,
) -> vec3f {
    let x = mean3d.x;
    let y = mean3d.y;
    let z = mean3d.z;

    let rz = 1.0 / mean3d.z;
    let rz2 = rz * rz;

    // df/dx = fx * rz * df/dpixx
    // df/dy = fy * rz * df/dpixy
    // df/dz = - fx * mean.x * rz2 * df/dpixx - fy * mean.y * rz2 * df/dpixy
    var v_mean3d = vec3f(
        focal.x * rz * v_mean2d[0],
        focal.y * rz * v_mean2d[1],
        -(focal.x * x * v_mean2d[0] + focal.y * y * v_mean2d[1]) * rz2
    );

    // df/dx = -fx * rz2 * df/dJ_02
    // df/dy = -fy * rz2 * df/dJ_12
    // df/dz = -fx * rz2 * df/dJ_00 - fy * rz2 * df/dJ_11
    //         + 2 * fx * tx * rz3 * df/dJ_02 + 2 * fy * ty * rz3
    let rz3 = rz2 * rz;
    let v_J = v_cov2d * J * transpose(cov3d) + transpose(v_cov2d) * J * cov3d;

    let tan_fov = 0.5 * vec2f(img_size.xy) / focal;

    let lims_pos = (vec2f(img_size.xy) - pixel_center) / focal + 0.3f * tan_fov;
    let lims_neg = pixel_center / focal + 0.3f * tan_fov;
    // Get ndc coords +- clipped to the frustum.
    let t = mean3d.z * clamp(mean3d.xy * rz, -lims_neg, lims_pos);

    let lim_x_pos = lims_pos.x;
    let lim_x_neg = lims_neg.x;
    let lim_y_pos = lims_pos.y;
    let lim_y_neg = lims_neg.y;
    let tx = t.x;
    let ty = t.y;

    // fov clipping
    if (x * rz <= lim_x_pos && x * rz >= -lim_x_neg) {
        v_mean3d.x += -focal.x * rz2 * v_J[2][0];
    } else {
        v_mean3d.z += -focal.x * rz3 * v_J[2][0] * tx;
    }
    if (y * rz <= lim_y_pos && y * rz >= -lim_y_neg) {
        v_mean3d.y += -focal.y * rz2 * v_J[2][1];
    } else {
        v_mean3d.z += -focal.y * rz3 * v_J[2][1] * ty;
    }
    v_mean3d.z += -focal.x * rz2 * v_J[0][0] - focal.y * rz2 * v_J[1][1] +
                  2.f * focal.x * tx * rz3 * v_J[2][0] +
                  2.f * focal.y * ty * rz3 * v_J[2][1];

    // add contribution from v_depths
    // Disabled as there is no depth supervision currently.
    // v_mean3d.z += v_depths[0];

    return v_mean3d;
}

@compute
@workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3u) {
    let compact_gid = i32(gid.x);
    if compact_gid >= uniforms.num_visible {
        return;
    }

    let viewmat = uniforms.viewmat;
    let focal = uniforms.focal;
    let img_size = uniforms.img_size;
    let pixel_center = uniforms.pixel_center;

    let global_gid = global_from_compact_gid[compact_gid];
    let mean = helpers::as_vec(means[global_gid]);
    let scale = exp(helpers::as_vec(log_scales[global_gid]));
    let quat_unorm = quats[global_gid];
    // Safe to normalize, quats with norm 0 are invisible.
    let quat = normalize(quat_unorm);

    let v_mean2d = vec2f(v_grads[compact_gid * 9 + 0], v_grads[compact_gid * 9 + 1]);
    let v_conics = vec3f(v_grads[compact_gid * 9 + 2], v_grads[compact_gid * 9 + 3], v_grads[compact_gid * 9 + 4]);

    let R = mat3x3f(viewmat[0].xyz, viewmat[1].xyz, viewmat[2].xyz);
    let mean_c = R * mean + viewmat[3].xyz;
    let rz = 1.0 / mean_c.z;
    let rz2 = rz * rz;

    let rotmat = helpers::quat_to_mat(quat);
    let S = helpers::scale_to_mat(scale);
    let M = rotmat * S;

    let covar = M * transpose(M);
    let cov2d = helpers::calc_cov2d(covar, mean_c, focal, img_size, pixel_center, viewmat);
    let covar2d_inv = helpers::inverse(cov2d);

    let v_covar2d_inv = mat2x2f(vec2f(v_conics.x, v_conics.y * 0.5f), vec2f(v_conics.y * 0.5f, v_conics.z));

    let v_covar2d = inverse_vjp(covar2d_inv, v_covar2d_inv);

    // covar_world_to_cam
    let covar_c = R * covar * transpose(R);

    // persp_proj_vjp
    let J = helpers::calc_cam_J(mean_c, focal, img_size, pixel_center);
    let v_mean_c = persp_proj_vjp(J, mean_c, covar_c, focal, pixel_center, img_size, v_covar2d, v_mean2d);
    // cov = J * V * Jt; G = df/dcov = v_cov
    // -> df/dV = Jt * G * J
    // -> df/dJ = G * J * Vt + Gt * J * V
    let v_covar_c = transpose(J) * v_covar2d * J;

    // df/dx = -fx * rz2 * df/dJ_02
    // df/dy = -fy * rz2 * df/dJ_12
    // df/dz = -fx * rz2 * df/dJ_00 - fy * rz2 * df/dJ_11
    //         + 2 * fx * tx * rz3 * df/dJ_02 + 2 * fy * ty * rz3
    // for D = W * X, G = df/dD
    // df/dW = G * XT, df/dX = WT * G

    // TODO: Camera gradient is not done yet.
    // var v_R = outer_product(v_mean_c, mean);
    let v_mean = transpose(R) * v_mean_c;

    // covar_world_to_cam_vjp
    // TODO: Camera gradient is not done yet.
    // v_R += v_covar_c * R * transpose(covar) +
    //        transpose(v_covar_c) * R * covar;

    let v_covar = transpose(R) * v_covar_c * R;

    // quat_scale_to_covar_vjp
    // TODO: Merge with cov calculation.

    // https://math.stackexchange.com/a/3850121
    // for D = W * X, G = df/dD
    // df/dW = G * XT, df/dX = WT * G
    // so
    // for D = M * Mt,
    // df/dM = df/dM + df/dMt = G * M + (Mt * G)t = G * M + Gt * M
    let v_M = (v_covar + transpose(v_covar)) * M;

    let v_scale = vec3f(
        dot(rotmat[0], v_M[0]),
        dot(rotmat[1], v_M[1]),
        dot(rotmat[2], v_M[2]),
    );
    let v_scale_exp = v_scale * scale;

    // grad for (quat, scale) from covar
    let v_quat = normalize_vjp(quat_unorm) * quat_to_mat_vjp(quat, v_M * S);

    v_means[global_gid] = helpers::as_packed(v_mean);
    v_scales[global_gid] = helpers::as_packed(v_scale_exp);
    v_quats[global_gid] = v_quat;
}
