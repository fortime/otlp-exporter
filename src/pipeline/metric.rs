use std::mem;

use crate::{
    config::{ConfigBuilder, DataType},
    Pipeline,
};

pub struct MetricPipeline {
    config_builder: ConfigBuilder,
}

impl MetricPipeline {
    pub fn with_env(mut self) -> MetricPipeline {
        self.config_builder = self.config_builder.with_env(Some(DataType::Metric));
        self
    }

    pub fn with_config_builder(mut self, config_builder: ConfigBuilder) -> MetricPipeline {
        self.config_builder = config_builder;
        self
    }

    pub fn with_config_builder_customizer(
        mut self,
        customizer: impl FnOnce(ConfigBuilder) -> ConfigBuilder,
    ) -> MetricPipeline {
        let config_builder = mem::take(&mut self.config_builder);
        self.config_builder = customizer(config_builder);
        self
    }
}

impl Pipeline {
    pub fn metric(self) -> MetricPipeline {
        MetricPipeline {
            config_builder: Default::default(),
        }
    }
}
