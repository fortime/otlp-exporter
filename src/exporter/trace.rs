use futures::{future::BoxFuture, FutureExt};
use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};

#[cfg(feature = "grpc")]
use crate::config::GrpcImpl;

use crate::{
    config::{Config, Protocol},
    error::OtlpExporterError,
};

#[derive(Debug)]
pub enum TraceExporter {
    #[cfg(feature = "tonic")]
    Tonic(TonicTraceExporter),
    #[cfg(feature = "grpcio")]
    Grpcio(GrpcioTraceExporter),
    #[cfg(feature = "http")]
    Http(HttpTraceExporter),
}

impl TryFrom<Config> for TraceExporter {
    type Error = OtlpExporterError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let exporter = match config.protocol() {
            #[cfg(feature = "grpc")]
            Protocol::Grpc => match config.grpc_impl() {
                #[cfg(feature = "tonic")]
                GrpcImpl::Tonic(_) => TonicTraceExporter::try_new(config)?.into(),
                #[cfg(feature = "grpcio")]
                GrpcImpl::Grpcio(_) => GrpcioTraceExporter::try_new(config)?.into(),
            },
            #[cfg(feature = "http")]
            Protocol::HttpProtobuf => HttpTraceExporter::try_new_in_protobuf(config)?.into(),
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => HttpTraceExporter::try_new_in_json(config)?.into(),
        };
        Ok(exporter)
    }
}

#[cfg(feature = "tonic")]
mod tonic {
    use opentelemetry_proto::tonic::collector::trace::v1::{
        trace_service_client::TraceServiceClient, ExportTraceServiceRequest,
    };
    use opentelemetry_sdk::export::trace::SpanData;
    use tonic::{metadata::MetadataMap, transport::Channel, Request};

    use crate::{config::Config, error::OtlpExporterResult};

    use super::TraceExporter;

    #[derive(Debug)]
    pub struct TonicTraceExporter {
        config: Config,
        metadata_map: MetadataMap,
        client: TraceServiceClient<Channel>,
    }

    impl TonicTraceExporter {
        pub(super) fn try_new(config: Config) -> OtlpExporterResult<Self> {
            // TODO client.accept_compression().send_compression()
            Ok(Self {
                metadata_map: crate::exporter::tonic::gen_metadata_map(config.headers())?,
                client: TraceServiceClient::new(Channel::try_from(&config)?),
                config,
            })
        }

        pub fn client(&self) -> &TraceServiceClient<Channel> {
            &self.client
        }

        pub fn config(&self) -> &Config {
            &self.config
        }

        pub fn gen_request(&self, batch: Vec<SpanData>) -> Request<ExportTraceServiceRequest> {
            let mut request = Request::new(ExportTraceServiceRequest {
                resource_spans: batch.into_iter().map(Into::into).collect(),
            });
            *request.metadata_mut() = self.metadata_map.clone();
            request
        }
    }

    impl From<TonicTraceExporter> for TraceExporter {
        fn from(exporter: TonicTraceExporter) -> Self {
            TraceExporter::Tonic(exporter)
        }
    }
}
#[cfg(feature = "tonic")]
pub use self::tonic::TonicTraceExporter;

#[cfg(feature = "grpcio")]
mod grpcio {
    use std::fmt;

    use grpcio::{Channel, Metadata};
    use opentelemetry_proto::grpcio::{
        trace_service::ExportTraceServiceRequest, trace_service_grpc::TraceServiceClient,
    };
    use opentelemetry_sdk::export::trace::SpanData;

    use crate::{config::Config, error::OtlpExporterResult};

    use super::TraceExporter;

    pub struct GrpcioTraceExporter {
        config: Config,
        client: TraceServiceClient,
        metadata: Metadata,
    }

    impl GrpcioTraceExporter {
        pub(super) fn try_new(config: Config) -> OtlpExporterResult<Self> {
            Ok(Self {
                client: TraceServiceClient::new(Channel::try_from(&config)?),
                metadata: crate::exporter::grpcio::gen_metadata(config.headers())?,
                config,
            })
        }

        pub fn client(&self) -> &TraceServiceClient {
            &self.client
        }

        pub fn config(&self) -> &Config {
            &self.config
        }

        pub fn metadata(&self) -> &Metadata {
            &self.metadata
        }

        pub fn gen_request(&self, batch: Vec<SpanData>) -> ExportTraceServiceRequest {
            ExportTraceServiceRequest {
                resource_spans: protobuf::RepeatedField::from_vec(
                    batch.into_iter().map(Into::into).collect(),
                ),
                ..Default::default()
            }
        }
    }

    impl fmt::Debug for GrpcioTraceExporter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("GrpcioTraceExporter")
                .field("config", &self.config)
                .field("client", &"...")
                .finish()
        }
    }

    impl From<GrpcioTraceExporter> for TraceExporter {
        fn from(exporter: GrpcioTraceExporter) -> Self {
            TraceExporter::Grpcio(exporter)
        }
    }
}
#[cfg(feature = "grpcio")]
pub use self::grpcio::GrpcioTraceExporter;

#[cfg(feature = "http")]
mod http {
    use http::header::CONTENT_TYPE;
    use opentelemetry_proto::grpcio::trace_service::ExportTraceServiceRequest;
    use opentelemetry_sdk::export::trace::SpanData;
    use protobuf::Message;
    use reqwest::{Client, RequestBuilder};

    use crate::{
        config::Config,
        error::{OtlpExporterError, OtlpExporterResult},
    };

    use super::TraceExporter;

    #[derive(Debug)]
    pub struct HttpTraceExporter {
        config: Config,
        encoder: crate::exporter::http::Encoder,
        client: Client,
    }

    impl HttpTraceExporter {
        pub(super) fn try_new_in_protobuf(config: Config) -> OtlpExporterResult<Self> {
            Ok(Self {
                client: Client::try_from(&config)?,
                encoder: crate::exporter::http::Encoder::Protobuf,
                config,
            })
        }

        #[cfg(feature = "http-json")]
        pub(super) fn try_new_in_json(config: Config) -> OtlpExporterResult<Self> {
            Ok(Self {
                client: Client::try_from(&config)?,
                encoder: crate::exporter::http::Encoder::Json,
                config,
            })
        }

        pub fn gen_request_builder(
            &self,
            batch: Vec<SpanData>,
        ) -> OtlpExporterResult<RequestBuilder> {
            let payload = ExportTraceServiceRequest {
                resource_spans: protobuf::RepeatedField::from_vec(
                    batch.into_iter().map(Into::into).collect(),
                ),
                ..Default::default()
            };

            let mut request_builder = self.client.post(self.config.endpoint().to_string());

            match self.encoder {
                crate::exporter::http::Encoder::Protobuf => {
                    request_builder = request_builder
                        .header(CONTENT_TYPE, "application/x-protobuf")
                        .body(payload.write_to_bytes().map_err(|e| {
                            OtlpExporterError::UnknownError(format!(
                                "failed to serialize trace request to protobuf, error: {e}"
                            ))
                        })?);
                }
                #[cfg(feature = "http-json")]
                crate::exporter::http::Encoder::Json => {
                    request_builder = request_builder
                        .header(CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&payload).map_err(|e| {
                            OtlpExporterError::UnknownError(format!(
                                "failed to serialize trace request to json, error: {e}"
                            ))
                        })?);
                }
            }

            Ok(request_builder)
        }
    }

    impl From<HttpTraceExporter> for TraceExporter {
        fn from(exporter: HttpTraceExporter) -> Self {
            TraceExporter::Http(exporter)
        }
    }
}
#[cfg(feature = "http")]
pub use self::http::HttpTraceExporter;

impl SpanExporter for TraceExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        match self {
            #[cfg(feature = "tonic")]
            TraceExporter::Tonic(exporter) => {
                let request = exporter.gen_request(batch);
                let mut client = exporter.client().clone();
                let timeout = exporter.config().timeout();
                async move {
                    if let Err(status) = client.export(request).await {
                        return crate::exporter::tonic::gen_trace_error(status, timeout);
                    }
                    Ok(())
                }
                .boxed()
            }
            #[cfg(feature = "grpcio")]
            TraceExporter::Grpcio(exporter) => {
                let request = exporter.gen_request(batch);
                let client = exporter.client().clone();
                let call_option = ::grpcio::CallOption::default()
                    .timeout(exporter.config().timeout())
                    .headers(exporter.metadata().clone());
                async move {
                    let response = client
                        .export_async_opt(&request, call_option)
                        .map_err(OtlpExporterError::from)?;
                    response.await.map_err(OtlpExporterError::from)?;
                    Ok(())
                }
                .boxed()
            }
            #[cfg(feature = "http")]
            TraceExporter::Http(exporter) => {
                let request_builder = exporter.gen_request_builder(batch);
                async move {
                    request_builder?
                        .send()
                        .await
                        .map_err(OtlpExporterError::from)?;
                    Ok(())
                }
                .boxed()
            }
        }
    }
}
