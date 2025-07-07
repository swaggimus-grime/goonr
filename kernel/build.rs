use miette::IntoDiagnostic;

fn main() -> miette::Result<()> {
    wgsl::build_modules(&["src/shaders/wg.wgsl"], &[], "src/shaders/mod.rs").into_diagnostic()
}