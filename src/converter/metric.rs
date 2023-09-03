#[cfg(any(feature = "tonic", feature = "http"))]
pub mod tonic {
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    use opentelemetry_sdk::metrics::data::ResourceMetrics;

    pub fn convert(metrics: &mut ResourceMetrics) -> ExportMetricsServiceRequest {
        todo!()
    }
}
