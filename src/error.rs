use std::io;

use opentelemetry_api::ExportError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OtlpExporterError {
    #[error("invalid config: {0}")]
    ConfigError(String),
    #[error("std io error: {0:?}")]
    StdIoError(#[from] io::Error),
    #[cfg(feature = "tonic")]
    #[error("tonic error: {0}")]
    TonicError(tonic::Status),
    #[cfg(feature = "grpcio")]
    #[error("grpcio error: {0}")]
    GrpcioError(#[from] grpcio::Error),
    #[cfg(feature = "http")]
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("unknown error: {0}")]
    UnknownError(String),
}

pub type OtlpExporterResult<T> = Result<T, OtlpExporterError>;

impl ExportError for OtlpExporterError {
    fn exporter_name(&self) -> &'static str {
        "otlp-exporter"
    }
}
