use async_trait::async_trait;
use opentelemetry_api::metrics::MetricsError;
use opentelemetry_sdk::metrics::{
    data::{ResourceMetrics, Temporality},
    reader::{AggregationSelector, TemporalitySelector},
    Aggregation, InstrumentKind,
};

#[cfg(feature = "_grpc")]
use crate::config::GrpcImpl;
use crate::{
    config::{Config, Protocol},
    error::{OtlpExporterError, OtlpExporterResult},
};

#[derive(Debug)]
enum InnerMetricExporter {
    #[cfg(feature = "tonic")]
    Tonic(TonicMetricExporter),
    #[cfg(feature = "grpcio")]
    Grpcio(GrpcioMetricExporter),
    #[cfg(feature = "http")]
    Http(HttpMetricExporter),
}

#[derive(Debug)]
pub struct MetricExporter<AS, TS> {
    aggregator_selector: AS,
    temporality_selector: TS,
    inner: InnerMetricExporter,
}

impl<AS, TS> MetricExporter<AS, TS> {
    fn try_new(
        config: Config,
        aggregator_selector: AS,
        temporality_selector: TS,
    ) -> OtlpExporterResult<Self> {
        let inner = match config.protocol() {
            #[cfg(feature = "_grpc")]
            Protocol::Grpc => match config.grpc_impl() {
                #[cfg(feature = "tonic")]
                GrpcImpl::Tonic(_) => TonicMetricExporter::try_new(config)?.into(),
                #[cfg(feature = "grpcio")]
                GrpcImpl::Grpcio(_) => GrpcioMetricExporter::try_new(config)?.into(),
            },
            #[cfg(feature = "http")]
            Protocol::HttpProtobuf => HttpMetricExporter::try_new_in_protobuf(config)?.into(),
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => HttpMetricExporter::try_new_in_json(config)?.into(),
        };
        Ok(Self {
            inner,
            aggregator_selector,
            temporality_selector,
        })
    }
}

#[cfg(feature = "tonic")]
mod tonic {
    use opentelemetry_api::metrics::MetricsError;
    use opentelemetry_proto::tonic::collector::metrics::v1::{
        metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest,
    };
    use opentelemetry_sdk::metrics::data::ResourceMetrics;
    use tonic::{metadata::MetadataMap, transport::Channel, Request, Status};

    use crate::{
        config::Config,
        error::{OtlpExporterError, OtlpExporterResult},
    };

    use super::InnerMetricExporter;

    #[derive(Debug)]
    pub struct TonicMetricExporter {
        metadata_map: MetadataMap,
        client: MetricsServiceClient<Channel>,
    }

    impl TonicMetricExporter {
        pub(super) fn try_new(config: Config) -> OtlpExporterResult<Self> {
            Ok(Self {
                metadata_map: crate::exporter::tonic::gen_metadata_map(config.headers())?,
                client: MetricsServiceClient::new(Channel::try_from(&config)?),
            })
        }

        pub fn client(&self) -> &MetricsServiceClient<Channel> {
            &self.client
        }

        pub fn gen_request(
            &self,
            metrics: &mut ResourceMetrics,
        ) -> Request<ExportMetricsServiceRequest> {
            let mut request = Request::new(ExportMetricsServiceRequest {
                resource_metrics: vec![crate::converter::metric::tonic::convert_metrics(metrics)]
            });
            *request.metadata_mut() = self.metadata_map.clone();
            request
        }
    }

    impl From<TonicMetricExporter> for InnerMetricExporter {
        fn from(exporter: TonicMetricExporter) -> Self {
            InnerMetricExporter::Tonic(exporter)
        }
    }

    pub fn gen_metric_error(status: Status) -> Result<(), MetricsError> {
        match status.code() {
            tonic::Code::Ok => Ok(()),
            _ => Err(MetricsError::from(OtlpExporterError::TonicError(status))),
        }
    }
}
#[cfg(feature = "tonic")]
pub use self::tonic::TonicMetricExporter;

#[cfg(feature = "grpcio")]
mod grpcio {
    use std::fmt;

    use grpcio::{Channel, Metadata};
    use opentelemetry_proto::grpcio::{
        metrics_service::ExportMetricsServiceRequest, metrics_service_grpc::MetricsServiceClient,
    };

    use crate::{config::Config, error::OtlpExporterResult};

    use super::InnerMetricExporter;

    pub struct GrpcioMetricExporter {
        config: Config,
        client: MetricsServiceClient,
        metadata: Metadata,
    }

    impl GrpcioMetricExporter {
        pub(super) fn try_new(config: Config) -> OtlpExporterResult<Self> {
            Ok(Self {
                client: MetricsServiceClient::new(Channel::try_from(&config)?),
                metadata: crate::exporter::grpcio::gen_metadata(config.headers())?,
                config,
            })
        }

        pub fn client(&self) -> &MetricsServiceClient {
            &self.client
        }

        pub fn config(&self) -> &Config {
            &self.config
        }

        pub fn metadata(&self) -> &Metadata {
            &self.metadata
        }
    }

    impl fmt::Debug for GrpcioMetricExporter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("GrpcioMetricExporter")
                .field("config", &self.config)
                .field("client", &"...")
                .finish()
        }
    }

    impl From<GrpcioMetricExporter> for InnerMetricExporter {
        fn from(exporter: GrpcioMetricExporter) -> Self {
            InnerMetricExporter::Grpcio(exporter)
        }
    }
}
#[cfg(feature = "grpcio")]
pub use self::grpcio::GrpcioMetricExporter;

#[cfg(feature = "http")]
mod http {
    use http::header::CONTENT_TYPE;
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    use opentelemetry_sdk::metrics::data::ResourceMetrics;
    use prost::Message;
    use reqwest::{Client, RequestBuilder};

    use crate::{config::Config, error::OtlpExporterResult};

    use super::InnerMetricExporter;

    #[derive(Debug)]
    pub struct HttpMetricExporter {
        config: Config,
        encoder: crate::exporter::http::Encoder,
        client: Client,
    }

    impl HttpMetricExporter {
        pub(super) fn try_new_in_protobuf(config: Config) -> OtlpExporterResult<Self> {
            Ok(Self {
                client: Client::try_from(&config)?,
                encoder: crate::exporter::http::Encoder::Protobuf,
                config,
            })
        }

        #[cfg(feature = "http-json")]
        pub(super) fn try_new_in_json(config: Config) -> OtlpExporterResult<Self> {
            /*
            let _ = Self {
                client: Client::try_from(&config)?,
                encoder: crate::exporter::http::Encoder::Json,
                config,
            };
            */
            unimplemented!("it needs time to find out how to serialize to json, refer: https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding")
        }

        pub fn gen_request_builder(
            &self,
            metrics: &mut ResourceMetrics,
        ) -> OtlpExporterResult<RequestBuilder> {
            let payload = ExportMetricsServiceRequest {
                resource_metrics: vec![crate::converter::metric::tonic::convert_metrics(metrics)]
            };

            let mut request_builder = self.client.post(self.config.endpoint().to_string());

            match self.encoder {
                crate::exporter::http::Encoder::Protobuf => {
                    request_builder = request_builder
                        .header(CONTENT_TYPE, "application/x-protobuf")
                        .body(payload.encode_to_vec());
                }
                #[cfg(feature = "http-json")]
                crate::exporter::http::Encoder::Json => {
                    todo!("it needs time to find out how to serialize to json, refer: https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding");
                    /*
                    request_builder = request_builder
                        .header(CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&payload).map_err(|e| {
                            OtlpExporterError::UnknownError(format!(
                                "failed to serialize trace request to json, error: {e}"
                            ))
                        })?);
                    */
                }
            }

            Ok(request_builder)
        }
    }

    impl From<HttpMetricExporter> for InnerMetricExporter {
        fn from(exporter: HttpMetricExporter) -> Self {
            InnerMetricExporter::Http(exporter)
        }
    }
}
#[cfg(feature = "http")]
pub use self::http::HttpMetricExporter;

impl<AS, TS> AggregationSelector for MetricExporter<AS, TS>
where
    AS: AggregationSelector,
    TS: Send + Sync,
{
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        self.aggregator_selector.aggregation(kind)
    }
}

impl<AS, TS> TemporalitySelector for MetricExporter<AS, TS>
where
    AS: Send + Sync,
    TS: TemporalitySelector,
{
    fn temporality(&self, kind: InstrumentKind) -> Temporality {
        self.temporality_selector.temporality(kind)
    }
}

#[async_trait]
impl<AS, TS> opentelemetry_sdk::metrics::exporter::PushMetricsExporter for MetricExporter<AS, TS>
where
    AS: AggregationSelector + 'static,
    TS: TemporalitySelector + 'static,
{
    async fn export(&self, metrics: &mut ResourceMetrics) -> Result<(), MetricsError> {
        match &self.inner {
            #[cfg(feature = "tonic")]
            InnerMetricExporter::Tonic(exporter) => {
                let request = exporter.gen_request(metrics);
                let mut client = exporter.client().clone();
                if let Err(status) = client.export(request).await {
                    return tonic::gen_metric_error(status);
                }
                Ok(())
            }
            #[cfg(feature = "grpcio")]
            InnerMetricExporter::Grpcio(exporter) => {
                todo!()
                /*
                let request = exporter.gen_request(batch);
                let client = exporter.client().clone();
                let call_option = ::grpcio::CallOption::default()
                    .timeout(exporter.config().timeout())
                    .headers(exporter.metadata().clone());
                let response = client
                    .export_async_opt(&request, call_option)
                    .map_err(OtlpExporterError::from)?;
                response.await.map_err(OtlpExporterError::from)?;
                Ok(())
                */
            }
            #[cfg(feature = "http")]
            InnerMetricExporter::Http(exporter) => {
                exporter
                    .gen_request_builder(metrics)?
                    .send()
                    .await
                    .map_err(OtlpExporterError::from)?;
                Ok(())
            }
        }
    }

    async fn force_flush(&self) -> Result<(), MetricsError> {
        Ok(())
    }

    fn shutdown(&self) -> Result<(), MetricsError> {
        Ok(())
    }
}
