#![doc = include_str!("../README.md")]

pub mod config;
mod converter;
pub mod error;
mod exporter;
mod pipeline;

#[cfg(feature = "metrics")]
pub use pipeline::metric::MetricPipeline;
#[cfg(feature = "traces")]
pub use pipeline::trace::TracePipeline;
pub use pipeline::{new_pipeline, Pipeline};
