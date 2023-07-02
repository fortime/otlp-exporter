#[cfg(feature = "logs")]
pub mod log;
#[cfg(feature = "metrics")]
pub mod metric;
#[cfg(feature = "traces")]
pub mod trace;

pub struct Pipeline;

/// Create a pipeline builder.
pub fn new_pipeline() -> Pipeline {
    Pipeline
}
