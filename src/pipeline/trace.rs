use std::mem;

use opentelemetry_api::global;
use opentelemetry_sdk::{
    runtime::RuntimeChannel,
    trace::{
        BatchMessage, Builder as TracerProviderBuilder, Config as TracerConfig, Tracer,
        TracerProvider,
    },
};

use crate::{
    config::{ConfigBuilder, DataType, Protocol},
    error::OtlpExporterResult,
    exporter::trace::TraceExporter,
    Pipeline,
};

pub struct TracePipeline {
    config_builder: ConfigBuilder,
    tracer_config: Option<TracerConfig>,
}

impl TracePipeline {
    pub fn with_env(mut self) -> Self {
        self.config_builder = self.config_builder.with_env(Some(DataType::Trace));
        self
    }

    pub fn with_config_builder(mut self, config_builder: ConfigBuilder) -> Self {
        self.config_builder = config_builder;
        self
    }

    pub fn with_config_builder_customizer(
        mut self,
        customizer: impl FnOnce(ConfigBuilder) -> ConfigBuilder,
    ) -> Self {
        let config_builder = mem::take(&mut self.config_builder);
        self.config_builder = customizer(config_builder);
        self
    }

    pub fn with_tracer_config(mut self, tracer_config: TracerConfig) -> Self {
        self.tracer_config = Some(tracer_config);
        self
    }

    fn install(
        self,
        builder_creator: impl FnOnce(
            Protocol,
            TraceExporter,
        ) -> OtlpExporterResult<TracerProviderBuilder>,
    ) -> OtlpExporterResult<Tracer> {
        let Self {
            config_builder,
            tracer_config,
        } = self;
        let config = config_builder.build()?;
        let mut builder = builder_creator(config.protocol(), TryFrom::try_from(config)?)?;
        if let Some(tracer_config) = tracer_config {
            builder = builder.with_config(tracer_config);
        }
        let provider = builder.build();
        let tracer = opentelemetry_api::trace::TracerProvider::versioned_tracer(
            &provider,
            env!("CARGO_PKG_NAME"),
            Some(env!("CARGO_PKG_VERSION")),
            None::<&'static str>,
            None,
        );
        global::set_tracer_provider(provider);
        Ok(tracer)
    }

    /// build the tracer
    pub fn install_simple(self) -> OtlpExporterResult<Tracer> {
        self.install(|protocol, exporter| {
            match protocol {
                #[cfg(feature = "http")]
                Protocol::HttpProtobuf => {
                    let _ = exporter;
                    Err(crate::error::OtlpExporterError::Unsupported(format!("install_simple can't be worked with http/protobuf, use install_batch with tokio instead")))
                },
                #[cfg(feature = "http-json")]
                Protocol::HttpJson => {
                    let _ = exporter;
                    Err(crate::error::OtlpExporterError::Unsupported(format!("install_simple can't be worked with http/json, use install_batch with tokio instead")))
                },
                #[cfg(feature = "grpc")]
                _ => Ok(TracerProvider::builder().with_simple_exporter(exporter)),
            }
        })
    }

    pub fn install_batch<R: RuntimeChannel<BatchMessage>>(
        self,
        runtime: R,
    ) -> OtlpExporterResult<Tracer> {
        self.install(move |_, exporter| {
            Ok(TracerProvider::builder().with_batch_exporter(exporter, runtime))
        })
    }
}

impl Pipeline {
    pub fn trace(self) -> TracePipeline {
        TracePipeline {
            config_builder: Default::default(),
            tracer_config: None,
        }
    }
}
