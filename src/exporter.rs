use std::{collections::HashMap, str::FromStr};

use ::http::{HeaderMap, HeaderName, HeaderValue};

use crate::error::{OtlpExporterError, OtlpExporterResult};

#[cfg(feature = "logs")]
pub mod log;
#[cfg(feature = "metrics")]
pub mod metric;
#[cfg(feature = "traces")]
pub mod trace;

fn gen_header_map(headers: &HashMap<String, Vec<String>>) -> OtlpExporterResult<HeaderMap> {
    let mut header_map = HeaderMap::with_capacity(headers.len());
    for (key, values) in headers {
        for value in values {
            header_map.append(
                HeaderName::from_str(key.as_str()).map_err(|e| {
                    OtlpExporterError::ConfigError(format!(
                        "invalid header name: {key}, error: {e}"
                    ))
                })?,
                HeaderValue::from_str(value.as_str()).map_err(|e| {
                    OtlpExporterError::ConfigError(format!(
                        "invalid header value: {value}, error: {e}"
                    ))
                })?,
            );
        }
    }
    Ok(header_map)
}

#[cfg(feature = "tonic")]
pub(crate) mod tonic {
    use std::{collections::HashMap, time::Duration};

    #[cfg(feature = "traces")]
    use opentelemetry_api::trace::TraceError;
    use tonic::{metadata::MetadataMap, transport::Channel, Status};

    #[cfg(feature = "tls")]
    use tonic::transport::{Certificate, ClientTlsConfig, Identity};

    use crate::{
        config::{Config, GrpcImpl, Protocol},
        error::{OtlpExporterError, OtlpExporterResult},
    };

    use super::gen_header_map;

    impl<'a> TryFrom<&'a Config> for Channel {
        type Error = OtlpExporterError;

        fn try_from(config: &'a Config) -> Result<Self, Self::Error> {
            if config.protocol() != Protocol::Grpc {
                return Err(OtlpExporterError::ConfigError(format!(
                    "protocol is {}, not {}",
                    config.protocol(),
                    Protocol::Grpc,
                )));
            }
            match config.grpc_impl() {
                GrpcImpl::Tonic(_) => {}
                #[cfg(feature = "grpcio")]
                _ => {
                    return Err(OtlpExporterError::ConfigError(format!(
                        "grpc impl is not set to {}, it is {}",
                        GrpcImpl::Tonic(Default::default()),
                        config.grpc_impl(),
                    )));
                }
            }

            let mut channel_builder = Channel::builder(config.endpoint().clone())
                .connect_timeout(config.timeout())
                .timeout(config.timeout());

            #[cfg(feature = "tls")]
            if !config.insecure() {
                let mut tls_config = ClientTlsConfig::new();
                if let Some(ca) = config.read_certificate()? {
                    tls_config = tls_config.ca_certificate(Certificate::from_pem(ca));
                }
                if let (Some(key), Some(cert)) =
                    (config.read_client_key()?, config.read_client_certificate()?)
                {
                    tls_config = tls_config.identity(Identity::from_pem(cert, key));
                }
                if let Some(domain) = config.certificate_domain() {
                    tls_config = tls_config.domain_name(domain.to_owned());
                }
                channel_builder = channel_builder.tls_config(tls_config).map_err(|e| {
                    OtlpExporterError::ConfigError(format!("invalid tonic config: {:?}", e))
                })?;
            };

            Ok(channel_builder.connect_lazy())
        }
    }

    pub fn gen_metadata_map(
        headers: &HashMap<String, Vec<String>>,
    ) -> OtlpExporterResult<MetadataMap> {
        Ok(MetadataMap::from_headers(gen_header_map(headers)?))
    }

    #[cfg(feature = "traces")]
    pub fn gen_trace_error(status: Status, timeout: Duration) -> Result<(), TraceError> {
        match status.code() {
            tonic::Code::Ok => Ok(()),
            tonic::Code::Cancelled => Err(TraceError::ExportTimedOut(timeout)),
            _ => Err(TraceError::from(OtlpExporterError::TonicError(status))),
        }
    }
}

#[cfg(feature = "grpcio")]
mod grpcio {
    use std::{collections::HashMap, sync::Arc};

    use grpcio::{
        Channel, ChannelBuilder, Environment, Metadata, MetadataBuilder,
    };

    #[cfg(feature = "tls")]
    use grpcio::ChannelCredentialsBuilder;

    use crate::{
        config::{Config, GrpcImpl, Protocol},
        error::{OtlpExporterError, OtlpExporterResult},
    };

    impl<'a> TryFrom<&'a Config> for Channel {
        type Error = OtlpExporterError;

        fn try_from(config: &'a Config) -> Result<Self, Self::Error> {
            if config.protocol() != Protocol::Grpc {
                return Err(OtlpExporterError::ConfigError(format!(
                    "protocol is {}, not {}",
                    config.protocol(),
                    Protocol::Grpc,
                )));
            }
            let mut channel_builder = match config.grpc_impl() {
                GrpcImpl::Grpcio(c) => {
                    ChannelBuilder::new(Arc::new(Environment::new(c.cq_count())))
                }
                #[cfg(feature = "tonic")]
                _ => {
                    return Err(OtlpExporterError::ConfigError(format!(
                        "grpc impl is not set to {}, it is {}",
                        GrpcImpl::Grpcio(Default::default()),
                        config.grpc_impl(),
                    )))
                }
            };

            #[cfg(feature = "tls")]
            if !config.insecure() {
                let mut channel_credentials_builder = ChannelCredentialsBuilder::new();
                if let Some(ca) = config.read_certificate()? {
                    channel_credentials_builder =
                        channel_credentials_builder.root_cert(ca.into_bytes());
                }
                if let (Some(key), Some(cert)) =
                    (config.read_client_key()?, config.read_client_certificate()?)
                {
                    channel_credentials_builder =
                        channel_credentials_builder.cert(cert.into_bytes(), key.into_bytes());
                }
                channel_builder =
                    channel_builder.set_credentials(channel_credentials_builder.build());
            }

            Ok(channel_builder.connect(&config.endpoint().to_string()))
        }
    }

    pub fn gen_metadata(headers: &HashMap<String, Vec<String>>) -> OtlpExporterResult<Metadata> {
        let mut builder = MetadataBuilder::new();

        for (key, values) in headers {
            for value in values {
                builder.add_str(&*key, &*value).map_err(|e| {
                    OtlpExporterError::ConfigError(format!(
                        "invalid header key/value: {key}/{value}, error: {e}"
                    ))
                })?;
            }
        }

        Ok(builder.build())
    }
}

#[cfg(feature = "http")]
mod http {
    use reqwest::Client;

    #[cfg(feature = "tls")]
    use reqwest::{Certificate, Identity};

    use crate::{
        config::{Config, Protocol},
        error::OtlpExporterError,
        exporter::gen_header_map,
    };

    impl<'a> TryFrom<&'a Config> for Client {
        type Error = OtlpExporterError;

        fn try_from(config: &'a Config) -> Result<Self, Self::Error> {
            match config.protocol() {
                Protocol::HttpJson | Protocol::HttpProtobuf => {}
                #[cfg(feature = "grpc")]
                _ => {
                    return Err(OtlpExporterError::ConfigError(format!(
                        "protocol is {}, not {}/{}",
                        config.protocol(),
                        Protocol::HttpProtobuf,
                        Protocol::HttpJson,
                    )));
                }
            }

            let mut builder = Client::builder()
                .connect_timeout(config.timeout())
                .timeout(config.timeout())
                .default_headers(gen_header_map(config.headers())?);

            #[cfg(feature = "tls")]
            if !config.insecure() {
                if let Some(ca) = config.read_certificate()? {
                    builder = builder.add_root_certificate(
                        Certificate::from_pem(ca.as_bytes()).map_err(|e| {
                            OtlpExporterError::ConfigError(format!(
                                "invalid ca certificate, error: {e}"
                            ))
                        })?,
                    );
                }
                if let (Some(key), Some(cert)) =
                    (config.read_client_key()?, config.read_client_certificate()?)
                {
                    builder = builder.identity(
                        Identity::from_pem((key + &cert).as_bytes()).map_err(|e| {
                            OtlpExporterError::ConfigError(format!(
                                "invalid client key or client certificate, error: {e}"
                            ))
                        })?,
                    );
                }
            }

            Ok(builder.build()?)
        }
    }

    #[derive(Debug)]
    pub enum Encoder {
        Protobuf,
        #[cfg(feature = "http-json")]
        Json,
    }
}
