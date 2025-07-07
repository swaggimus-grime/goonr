use miette::IntoDiagnostic;

fn main() -> miette::Result<()> {
    wgsl::build_modules(
        &[
            "src/shaders/rasterize_backwards.wgsl",
            "src/shaders/gather_grads.wgsl",
            "src/shaders/project_backwards.wgsl",
        ],
        &["../render/src/shaders/helpers.wgsl"],
        "src/shaders/mod.rs",
    )
    .into_diagnostic()
}
