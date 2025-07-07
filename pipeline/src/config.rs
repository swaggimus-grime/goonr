use burn::prelude::Config;

#[derive(Config, Debug)]
pub struct PipelineConfig {
    /// Random seed.
    #[config(default = 42)]
    pub seed: u64,

    /// Iteration to resume from
    #[config(default = 0)]
    pub start_iter: u32,

    /// Eval every this many steps.
    #[config(default = 1000)]
    pub eval_every: u32,
    
    /// Save the rendered eval images to disk. Uses export-path for the file location.
    #[config(default = false)]
    pub eval_save_to_disk: bool,

    /// Export every this many steps.
    #[config(default = 5000)]
    pub export_every: u32,
    
    /// Location to put exported files. By default uses the cwd.
    ///
    /// This path can be set to be relative to the CWD.
    #[config(default = "String::from('.')")]
    pub export_path: String,
    
    /// Filename of exported ply file
    #[config(default = "String::from(\"export_{iter}.ply\")")]
    pub export_name: String,
}