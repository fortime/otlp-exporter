use std::mem;

use opentelemetry_api::global;
use opentelemetry_sdk::trace::{
    Builder as TracerProviderBuilder, Config as TracerConfig, TraceRuntime, Tracer, TracerProvider,
};

use crate::{
    config::{ConfigBuilder, DataType},
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
        let config_builder = mem::replace(&mut self.config_builder, Default::default());
        self.config_builder = customizer(config_builder);
        self
    }

    pub fn with_tracer_config(mut self, tracer_config: TracerConfig) -> Self {
        self.tracer_config = Some(tracer_config);
        self
    }

    fn install(
        self,
        builder_creator: impl FnOnce(TraceExporter) -> TracerProviderBuilder,
    ) -> OtlpExporterResult<Tracer> {
        let Self {
            config_builder,
            tracer_config,
        } = self;
        let config = config_builder.build()?;
        let mut builder = builder_creator(TryFrom::try_from(config)?);
        if let Some(tracer_config) = tracer_config {
            builder = builder.with_config(tracer_config);
        }
        let provider = builder.build();
        let tracer = opentelemetry_api::trace::TracerProvider::versioned_tracer(
            &provider,
            env!("CARGO_PKG_NAME"),
            Some(env!("CARGO_PKG_VERSION")),
            None,
        );
        global::set_tracer_provider(provider);
        Ok(tracer)
    }

    pub fn install_simple(self) -> OtlpExporterResult<Tracer> {
        self.install(|exporter| TracerProvider::builder().with_simple_exporter(exporter))
    }

    pub fn install_batch<R: TraceRuntime>(self, runtime: R) -> OtlpExporterResult<Tracer> {
        self.install(move |exporter| {
            TracerProvider::builder().with_batch_exporter(exporter, runtime)
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
