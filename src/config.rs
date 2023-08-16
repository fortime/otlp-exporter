//! OTLP exporter configurations.
//!
//! Refer: `<https://github.com/open-telemetry/opentelemetry-specification/blob/v1.21.0/specification/protocol/exporter.md>`

/// Env key: OTEL_EXPORTER_OTLP_ENDPOINT
pub const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
/// Env key: OTEL_EXPORTER_OTLP_TIMEOUT
pub const OTEL_EXPORTER_OTLP_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT";
/// Env key: OTEL_EXPORTER_OTLP_INSECURE
pub const OTEL_EXPORTER_OTLP_INSECURE: &str = "OTEL_EXPORTER_OTLP_INSECURE";
/// Env key: OTEL_EXPORTER_OTLP_CERTIFICATE
pub const OTEL_EXPORTER_OTLP_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_CERTIFICATE";
/// Env key: OTEL_EXPORTER_OTLP_CLIENT_KEY
pub const OTEL_EXPORTER_OTLP_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_CLIENT_KEY";
/// Env key: OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE
pub const OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE";
/// Env key: OTEL_EXPORTER_OTLP_HEADERS
pub const OTEL_EXPORTER_OTLP_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS";
/// Env key: OTEL_EXPORTER_OTLP_PROTOCOL
pub const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL";
/// TODO compression

/// Default timeout is 10s.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
/// OTLP default http endpoint
#[cfg(feature = "http")]
pub const DEFAULT_HTTP_ENDPOINT: &str = "http://localhost:4318";
/// OTLP default grpc endpoint
#[cfg(feature = "grpc")]
pub const DEFAULT_GRPC_ENDPOINT: &str = "localhost:4317";

#[cfg(feature = "traces")]
mod trace_envs {
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_ENDPOINT
    pub const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_TIMEOUT
    pub const OTEL_EXPORTER_OTLP_TRACES_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_INSECURE
    pub const OTEL_EXPORTER_OTLP_TRACES_INSECURE: &str = "OTEL_EXPORTER_OTLP_TRACES_INSECURE";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY
    pub const OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_HEADERS
    pub const OTEL_EXPORTER_OTLP_TRACES_HEADERS: &str = "OTEL_EXPORTER_OTLP_TRACES_HEADERS";
    /// Env key: OTEL_EXPORTER_OTLP_TRACES_PROTOCOL
    pub const OTEL_EXPORTER_OTLP_TRACES_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_TRACES_PROTOCOL";
}
use std::{
    collections::HashMap,
    ffi::OsString,
    fmt::{self, Display},
    fs,
    time::Duration,
};

use http::{uri::Scheme, Uri};
#[cfg(feature = "traces")]
pub use trace_envs::*;

#[cfg(feature = "metrics")]
mod metric_envs {
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_ENDPOINT
    pub const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_TIMEOUT
    pub const OTEL_EXPORTER_OTLP_METRICS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_INSECURE
    pub const OTEL_EXPORTER_OTLP_METRICS_INSECURE: &str = "OTEL_EXPORTER_OTLP_METRICS_INSECURE";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY
    pub const OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_HEADERS
    pub const OTEL_EXPORTER_OTLP_METRICS_HEADERS: &str = "OTEL_EXPORTER_OTLP_METRICS_HEADERS";
    /// Env key: OTEL_EXPORTER_OTLP_METRICS_PROTOCOL
    pub const OTEL_EXPORTER_OTLP_METRICS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_METRICS_PROTOCOL";
}
#[cfg(feature = "metrics")]
pub use metric_envs::*;

#[cfg(feature = "logs")]
mod log_envs {
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_ENDPOINT
    pub const OTEL_EXPORTER_OTLP_LOGS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_LOGS_ENDPOINT";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_TIMEOUT
    pub const OTEL_EXPORTER_OTLP_LOGS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_LOGS_TIMEOUT";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_INSECURE
    pub const OTEL_EXPORTER_OTLP_LOGS_INSECURE: &str = "OTEL_EXPORTER_OTLP_LOGS_INSECURE";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE: &str = "OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY
    pub const OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY: &str = "OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE
    pub const OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE: &str =
        "OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_HEADERS
    pub const OTEL_EXPORTER_OTLP_LOGS_HEADERS: &str = "OTEL_EXPORTER_OTLP_LOGS_HEADERS";
    /// Env key: OTEL_EXPORTER_OTLP_LOGS_PROTOCOL
    pub const OTEL_EXPORTER_OTLP_LOGS_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_LOGS_PROTOCOL";
}
#[cfg(feature = "logs")]
pub use log_envs::*;

#[cfg(feature = "grpc")]
mod grpc {
    use std::fmt::{self, Display};

    #[cfg(feature = "grpcio")]
    pub mod grpcio {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct GrpcioConfig {
            cq_count: usize,
        }

        impl Default for GrpcioConfig {
            fn default() -> Self {
                Self {
                    // set to 2 like opentelmetry-otlp
                    cq_count: 2,
                }
            }
        }

        impl GrpcioConfig {
            pub fn cq_count(&self) -> usize {
                self.cq_count
            }

            pub fn with_cq_count(mut self, cq_count: usize) -> Self {
                self.cq_count = cq_count;
                self
            }
        }
    }

    #[cfg(feature = "tonic")]
    pub mod tonic {
        #[derive(Clone, Debug, Default, Eq, PartialEq)]
        pub struct TonicConfig {
            _holder: (),
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum GrpcImpl {
        #[cfg(feature = "tonic")]
        Tonic(tonic::TonicConfig),
        #[cfg(feature = "grpcio")]
        Grpcio(grpcio::GrpcioConfig),
    }

    impl Display for GrpcImpl {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                #[cfg(feature = "tonic")]
                GrpcImpl::Tonic(_) => f.write_str("tonic"),
                #[cfg(feature = "grpcio")]
                GrpcImpl::Grpcio(_) => f.write_str("grpcio"),
            }
        }
    }

    impl Default for GrpcImpl {
        fn default() -> Self {
            #[cfg(feature = "tonic")]
            {
                GrpcImpl::Tonic(Default::default())
            }

            #[cfg(all(not(feature = "tonic"), feature = "grpcio"))]
            {
                GrpcImpl::Grpcio(Default::default())
            }
        }
    }
}
#[cfg(feature = "grpc")]
pub use grpc::GrpcImpl;

use crate::error::{OtlpExporterError, OtlpExporterResult};

macro_rules! set_from_env_with_default {
    ($env_name:ident, $fn:expr, $default:expr $(,)?) => {
        match std::env::var_os($env_name) {
            Some(v) => match $fn(v) {
                Some(v) => v,
                None => $default,
            },
            None => $default,
        }
    };
}

macro_rules! set_from_env {
    ($ident:expr, $env_name:ident, $fn:expr $(,)?) => {
        if let Some(v) = std::env::var_os($env_name) {
            if let Some(v) = $fn(v) {
                $ident = v;
            }
        }
    };
}

/// http/protobuf is the default protocol when http is enabled.
#[cfg(feature = "http")]
fn default_protocol() -> Protocol {
    Protocol::HttpProtobuf
}

/// Use grpc if http is disabled and one of grpcio and tonic is enabled. If all of http, grpcio
/// and tonic are disabled, it won't compile.
#[cfg(all(feature = "grpc", not(feature = "http")))]
fn default_protocol() -> Protocol {
    Protocol::Grpc
}

fn default_endpoint(protocol: Protocol) -> &'static str {
    match protocol {
        #[cfg(feature = "grpc")]
        Protocol::Grpc => DEFAULT_GRPC_ENDPOINT,
        #[cfg(feature = "http")]
        Protocol::HttpProtobuf => DEFAULT_HTTP_ENDPOINT,
        #[cfg(feature = "http-json")]
        Protocol::HttpJson => DEFAULT_HTTP_ENDPOINT,
    }
}

fn default_headers() -> HashMap<String, Vec<String>> {
    let mut headers = HashMap::new();
    headers.insert(
        "User-Agent".to_string(),
        vec![format!(
            "{}-rust/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )],
    );
    headers
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Protocol {
    #[cfg(feature = "grpc")]
    Grpc,
    #[cfg(feature = "http")]
    HttpProtobuf,
    #[cfg(feature = "http-json")]
    HttpJson,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "grpc")]
            Protocol::Grpc => f.write_str("grpc"),
            #[cfg(feature = "http")]
            Protocol::HttpProtobuf => f.write_str("http/protobuf"),
            #[cfg(feature = "http-json")]
            Protocol::HttpJson => f.write_str("http/json"),
        }
    }
}

/// The type of data for the OTLP exporter.
#[derive(Debug)]
pub enum DataType {
    /// Trace data.
    #[cfg(feature = "traces")]
    Trace,

    /// Metric data.
    #[cfg(feature = "metrics")]
    Metric,

    /// Log data.
    #[cfg(feature = "logs")]
    Log,
}

/// Configuration builder for the OTLP exporter.
#[derive(Clone, Debug)]
pub struct ConfigBuilder {
    endpoint: String,

    /// Whether to enable client transport security for the exporter's gRPC connection. This option
    /// only applies to OTLP/gRPC when an endpoint is provided without the http or https scheme -
    /// OTLP/HTTP always uses the scheme provided for the endpoint.
    insecure: bool,

    /// Path of the trusted certificate(PEM format) to use when verifying a server's TLS credentials.
    certificate_file: Option<OsString>,

    /// Path of the client's private key(PEM format) to use in mTLS communication.
    client_key_file: Option<OsString>,

    /// Path of the client's certificate/chain to use in mTLS communication.
    client_certificate_file: Option<OsString>,

    /// Key-value pairs to be used as headers associated with gRPC or HTTP requests.
    headers: HashMap<String, Vec<String>>,

    /// Maximum time the OTLP exporter will wait for each batch export.
    timeout: Duration,

    /// The transport protocol.
    protocol: Protocol,

    /// The implementation of grpc.
    #[cfg(feature = "grpc")]
    grpc_impl: GrpcImpl,

    /// Domain in the certificate.
    certificate_domain: Option<String>,
}

impl ConfigBuilder {
    pub fn with_env(mut self, data_type: Option<DataType>) -> Self {
        macro_rules! set_all_from_env {
            ($c:ident, $timeout_key:ident, $insecure_key:ident, $certificate_key:ident, $client_key_key:ident, $client_certificate_key:ident, $headers_key:ident $(,)?) => {
                set_from_env!($c.timeout, $timeout_key, parse_duration);
                set_from_env!($c.insecure, $insecure_key, parse_bool);
                set_from_env!($c.certificate_file, $certificate_key, |v| Some(Some(v)));
                set_from_env!($c.client_key_file, $client_key_key, |v| Some(Some(v)));
                set_from_env!($c.client_certificate_file, $client_certificate_key, |v| {
                    Some(Some(v))
                });
                set_from_env!($c.headers, $headers_key, parse_headers);
            };
        }

        fn parse_protocol(p: OsString) -> Option<Protocol> {
            match p.to_str()? {
                #[cfg(feature = "http")]
                "http/protobuf" => Some(Protocol::HttpProtobuf),
                #[cfg(feature = "http-json")]
                "http/json" => Some(Protocol::HttpJson),
                #[cfg(feature = "grpc")]
                "grpc" => Some(Protocol::Grpc),
                _ => None,
            }
        }

        fn parse_duration(t: OsString) -> Option<Duration> {
            let mut t = t.to_str()?;
            // support format like: 10s
            if t.ends_with(|c| c == 's' || c == 'S') {
                t = &t[0..t.len() - 1];
            }
            t.parse().ok().map(Duration::from_secs)
        }

        fn parse_bool(b: OsString) -> Option<bool> {
            b.to_str()?.parse().ok()
        }

        fn parse_headers(h: OsString) -> Option<HashMap<String, Vec<String>>> {
            // Parse headers according to:
            // https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#specifying-headers-via-environment-variables
            let items = h.to_str()?.split(',');
            let mut res = default_headers();
            for item in items {
                if let Some((key, value)) = item.split_once('=') {
                    let key = key.trim();
                    if key.is_empty() {
                        continue;
                    }
                    res.entry(key.to_string())
                        .or_default()
                        .push(value.trim().to_string());
                }
            }
            Some(res)
        }

        fn gen_endpoint(
            cur_protocol: Protocol,
            default_protocol: Protocol,
            path: &str,
            endpoint: &str,
        ) -> String {
            #[cfg(feature = "grpc")]
            #[allow(irrefutable_let_patterns)]
            if let Protocol::Grpc = cur_protocol {
                // there is no need to append path
                if cur_protocol == default_protocol {
                    return endpoint.to_owned();
                } else {
                    return default_endpoint(cur_protocol).to_owned();
                }
            }
            if cur_protocol == default_protocol {
                format!("{}{}", endpoint.trim_end_matches('/'), path)
            } else {
                format!(
                    "{}{}",
                    default_endpoint(cur_protocol).trim_end_matches('/'),
                    path
                )
            }
        }

        let protocol = set_from_env_with_default!(
            OTEL_EXPORTER_OTLP_PROTOCOL,
            parse_protocol,
            default_protocol(),
        );
        self.protocol = protocol;

        self.endpoint = set_from_env_with_default!(
            OTEL_EXPORTER_OTLP_ENDPOINT,
            |e: OsString| e.to_str().map(ToString::to_string),
            default_endpoint(self.protocol).to_string(),
        );

        set_all_from_env!(
            self,
            OTEL_EXPORTER_OTLP_TIMEOUT,
            OTEL_EXPORTER_OTLP_INSECURE,
            OTEL_EXPORTER_OTLP_CERTIFICATE,
            OTEL_EXPORTER_OTLP_CLIENT_KEY,
            OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE,
            OTEL_EXPORTER_OTLP_HEADERS
        );

        match data_type {
            #[cfg(feature = "traces")]
            Some(DataType::Trace) => {
                let trace_protocol = set_from_env_with_default!(
                    OTEL_EXPORTER_OTLP_TRACES_PROTOCOL,
                    parse_protocol,
                    protocol
                );
                self.protocol = trace_protocol;

                self.endpoint = set_from_env_with_default!(
                    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
                    |e: OsString| e.to_str().map(ToString::to_string),
                    gen_endpoint(trace_protocol, protocol, "/v1/traces", &self.endpoint)
                );

                set_all_from_env!(
                    self,
                    OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
                    OTEL_EXPORTER_OTLP_TRACES_INSECURE,
                    OTEL_EXPORTER_OTLP_TRACES_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_TRACES_CLIENT_KEY,
                    OTEL_EXPORTER_OTLP_TRACES_CLIENT_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_TRACES_HEADERS
                );
            },
            #[cfg(feature = "metrics")]
            Some(DataType::Metric) => {
                let metric_protocol = set_from_env_with_default!(
                    OTEL_EXPORTER_OTLP_METRICS_PROTOCOL,
                    parse_protocol,
                    protocol
                );
                self.protocol = metric_protocol;

                self.endpoint = set_from_env_with_default!(
                    OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
                    |e: OsString| e.to_str().map(ToString::to_string),
                    gen_endpoint(metric_protocol, protocol, "/v1/metrics", &self.endpoint)
                );

                set_all_from_env!(
                    self,
                    OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
                    OTEL_EXPORTER_OTLP_METRICS_INSECURE,
                    OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY,
                    OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_METRICS_HEADERS
                );
            },
            #[cfg(feature = "logs")]
            Some(DataType::Log) => {
                let log_protocol = set_from_env_with_default!(
                    OTEL_EXPORTER_OTLP_LOGS_PROTOCOL,
                    parse_protocol,
                    protocol
                );
                self.protocol = log_protocol;

                self.endpoint = set_from_env_with_default!(
                    OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
                    |e: OsString| e.to_str().map(ToString::to_string),
                    gen_endpoint(log_protocol, protocol, "/v1/logs", &self.endpoint)
                );

                set_all_from_env!(
                    self,
                    OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
                    OTEL_EXPORTER_OTLP_LOGS_INSECURE,
                    OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY,
                    OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE,
                    OTEL_EXPORTER_OTLP_LOGS_HEADERS
                );
            },
            #[cfg(not(any(feature = "traces", feature = "metrics", feature = "logs")))]
            Some(_) => unreachable!("DataType should be an empty enum when all of `traces`, `metrics`, `logs` are disabled"),
            None => {}
        }
        self
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn with_insecure(mut self, insecure: bool) -> Self {
        self.insecure = insecure;
        self
    }

    pub fn with_certificate_file(mut self, certificate_file: OsString) -> Self {
        self.certificate_file = Some(certificate_file);
        self
    }

    pub fn with_client_key_file(mut self, client_key_file: OsString) -> Self {
        self.client_key_file = Some(client_key_file);
        self
    }

    pub fn with_client_certificate_file(mut self, client_certificate_file: OsString) -> Self {
        self.client_certificate_file = Some(client_certificate_file);
        self
    }

    pub fn add_header<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let name = name.into();
        self.headers.entry(name).or_default().push(value.into());
        self
    }

    pub fn replace_header<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        let name = name.into();
        let values = self.headers.entry(name).or_default();
        values.clear();
        values.push(value.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    #[cfg(feature = "grpc")]
    pub fn with_grpc_impl(mut self, grpc_impl: GrpcImpl) -> Self {
        self.grpc_impl = grpc_impl;
        self
    }

    pub fn with_certificate_domain(mut self, domain: impl Into<String>) -> Self {
        self.certificate_domain = Some(domain.into());
        self
    }

    pub fn build(self) -> OtlpExporterResult<Config> {
        self.try_into()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        let protocol = default_protocol();
        Self {
            endpoint: default_endpoint(protocol).to_string(),
            insecure: false,
            certificate_file: None,
            client_key_file: None,
            client_certificate_file: None,
            headers: default_headers(),
            timeout: DEFAULT_TIMEOUT,
            protocol,
            #[cfg(feature = "grpc")]
            grpc_impl: Default::default(),
            certificate_domain: None,
        }
    }
}

/// Configuration for the OTLP exporter.
#[derive(Clone, Debug)]
pub struct Config {
    endpoint: Uri,
    builder: ConfigBuilder,
}

impl Config {
    pub fn endpoint(&self) -> &Uri {
        &self.endpoint
    }

    pub fn insecure(&self) -> bool {
        self.endpoint.scheme_str() != Some("https")
    }

    /// Read ca certificate if `certificate_file` is not none.
    pub fn read_certificate(&self) -> OtlpExporterResult<Option<String>> {
        let res = match self.builder.certificate_file.as_ref() {
            Some(path) => Some(fs::read_to_string(path)?),
            None => None,
        };
        Ok(res)
    }

    /// Read client key if `client_key_file` is not none.
    pub fn read_client_key(&self) -> OtlpExporterResult<Option<String>> {
        let res = match self.builder.client_key_file.as_ref() {
            Some(path) => Some(fs::read_to_string(path)?),
            None => None,
        };
        Ok(res)
    }

    /// Read client certificate if `client_certificate_file` is not none.
    pub fn read_client_certificate(&self) -> OtlpExporterResult<Option<String>> {
        let res = match self.builder.client_certificate_file.as_ref() {
            Some(path) => Some(fs::read_to_string(path)?),
            None => None,
        };
        Ok(res)
    }

    pub fn headers(&self) -> &HashMap<String, Vec<String>> {
        &self.builder.headers
    }

    pub fn timeout(&self) -> Duration {
        self.builder.timeout
    }

    pub fn protocol(&self) -> Protocol {
        self.builder.protocol
    }

    #[cfg(feature = "grpc")]
    pub fn grpc_impl(&self) -> &GrpcImpl {
        &self.builder.grpc_impl
    }

    pub fn certificate_domain(&self) -> Option<&str> {
        self.builder.certificate_domain.as_deref()
    }
}

/// Check if values in ConfigBuilder are valid
impl TryFrom<ConfigBuilder> for Config {
    type Error = OtlpExporterError;

    fn try_from(builder: ConfigBuilder) -> Result<Self, Self::Error> {
        let mut endpoint_parts = Uri::try_from(&builder.endpoint)
            .map_err(|e| {
                OtlpExporterError::ConfigError(format!(
                    "endpoint[{}] is not a valid uri: {}",
                    builder.endpoint, e
                ))
            })?
            .into_parts();

        let scheme = endpoint_parts.scheme.as_ref().map(Scheme::as_str);
        if scheme != Some("http") && scheme != Some("https") {
            if builder.insecure {
                endpoint_parts.scheme = Some(Scheme::HTTP);
            } else {
                endpoint_parts.scheme = Some(Scheme::HTTPS);
            }
        }
        if endpoint_parts.path_and_query.is_none() {
            // It is insane, parts without path will failed to construct a uri
            endpoint_parts.path_and_query = "/".try_into().ok();
        }

        Ok(Self {
            endpoint: endpoint_parts.try_into().map_err(|e| {
                OtlpExporterError::UnknownError(format!(
                    "internal error! endpoint_parts should be valid: {}",
                    e
                ))
            })?,
            builder,
        })
    }
}

impl From<Config> for ConfigBuilder {
    fn from(value: Config) -> Self {
        value.builder
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_headers, Config, ConfigBuilder, DataType, OTEL_EXPORTER_OTLP_ENDPOINT,
        OTEL_EXPORTER_OTLP_HEADERS, OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
        OTEL_EXPORTER_OTLP_LOGS_HEADERS, OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
        OTEL_EXPORTER_OTLP_METRICS_HEADERS, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
        OTEL_EXPORTER_OTLP_TRACES_HEADERS,
    };
    #[cfg(feature = "grpc")]
    use super::{
        OTEL_EXPORTER_OTLP_INSECURE, OTEL_EXPORTER_OTLP_LOGS_INSECURE,
        OTEL_EXPORTER_OTLP_METRICS_INSECURE, OTEL_EXPORTER_OTLP_PROTOCOL,
        OTEL_EXPORTER_OTLP_TRACES_INSECURE,
    };

    fn build_config_with_env(data_type: Option<DataType>) -> Config {
        ConfigBuilder::default()
            .with_env(data_type)
            .try_into()
            .unwrap()
    }

    #[test]
    fn test_endpoint_from_env() {
        let expected_endpoint = "https://test_endpoint_from_env:4317".to_string();
        let expected_endpoint_with_slash = "https://test_endpoint_from_env:4317/".to_string();
        temp_env::with_vars(
            vec![
                (OTEL_EXPORTER_OTLP_ENDPOINT, Some(&expected_endpoint)),
                #[cfg(feature = "traces")]
                (OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, None),
                #[cfg(feature = "metrics")]
                (OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, None),
                #[cfg(feature = "logs")]
                (OTEL_EXPORTER_OTLP_LOGS_ENDPOINT, None),
            ],
            || {
                // endpoint without specifying the data type
                assert_eq!(
                    build_config_with_env(None).endpoint().to_string(),
                    expected_endpoint_with_slash
                );
                // endpoint for traces
                #[cfg(feature = "traces")]
                {
                    assert_eq!(
                        build_config_with_env(Some(DataType::Trace))
                            .endpoint()
                            .to_string(),
                        format!("{}/v1/traces", expected_endpoint),
                    );
                    std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, &expected_endpoint);
                    assert_eq!(
                        build_config_with_env(Some(DataType::Trace))
                            .endpoint()
                            .to_string(),
                        expected_endpoint_with_slash,
                    );
                }

                // endpoint for metrics
                #[cfg(feature = "metrics")]
                {
                    assert_eq!(
                        build_config_with_env(Some(DataType::Metric))
                            .endpoint()
                            .to_string(),
                        format!("{}/v1/metrics", expected_endpoint),
                    );
                    std::env::set_var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT, &expected_endpoint);
                    assert_eq!(
                        build_config_with_env(Some(DataType::Metric))
                            .endpoint()
                            .to_string(),
                        expected_endpoint_with_slash
                    );
                }

                // endpoint for logs
                #[cfg(feature = "logs")]
                {
                    assert_eq!(
                        build_config_with_env(Some(DataType::Log))
                            .endpoint()
                            .to_string(),
                        format!("{}/v1/logs", expected_endpoint),
                    );
                    std::env::set_var(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT, &expected_endpoint);
                    assert_eq!(
                        build_config_with_env(Some(DataType::Log))
                            .endpoint()
                            .to_string(),
                        expected_endpoint_with_slash
                    );
                }

                // test if trailing '/' will be trimmed.
                std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, &expected_endpoint_with_slash);
                #[cfg(feature = "traces")]
                {
                    std::env::remove_var(OTEL_EXPORTER_OTLP_TRACES_ENDPOINT);
                    assert_eq!(
                        build_config_with_env(Some(DataType::Trace))
                            .endpoint()
                            .to_string(),
                        format!("{}/v1/traces", expected_endpoint),
                    );
                }

                #[cfg(feature = "metrics")]
                {
                    std::env::remove_var(OTEL_EXPORTER_OTLP_METRICS_ENDPOINT);
                    assert_eq!(
                        build_config_with_env(Some(DataType::Metric))
                            .endpoint()
                            .to_string(),
                        format!("{}/v1/metrics", expected_endpoint),
                    );
                }

                #[cfg(feature = "logs")]
                {
                    std::env::remove_var(OTEL_EXPORTER_OTLP_LOGS_ENDPOINT);
                    assert_eq!(
                        build_config_with_env(Some(DataType::Log))
                            .endpoint()
                            .to_string(),
                        format!("{}/v1/logs", expected_endpoint),
                    );
                }
            },
        );
    }

    #[test]
    fn test_headers_from_env() {
        let mut otlp_headers = default_headers();
        otlp_headers.insert("Accept".to_owned(), vec!["text/plain".to_owned()]);
        let mut otlp_traces_headers = default_headers();
        otlp_traces_headers.insert("Accept".to_owned(), vec!["application/json".to_owned()]);
        let mut otlp_metrics_headers = default_headers();
        otlp_metrics_headers.insert("Accept".to_owned(), vec!["application/xml".to_owned()]);
        let mut otlp_logs_headers = default_headers();
        otlp_logs_headers.insert("Accept".to_owned(), vec!["application/html".to_owned()]);
        temp_env::with_vars(
            vec![
                (OTEL_EXPORTER_OTLP_HEADERS, None::<String>),
                #[cfg(feature = "traces")]
                (OTEL_EXPORTER_OTLP_TRACES_HEADERS, None),
                #[cfg(feature = "metrics")]
                (OTEL_EXPORTER_OTLP_METRICS_HEADERS, None),
                #[cfg(feature = "logs")]
                (OTEL_EXPORTER_OTLP_LOGS_HEADERS, None),
            ],
            || {
                assert_eq!(build_config_with_env(None).headers(), &default_headers());

                #[cfg(feature = "traces")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Trace)).headers(),
                    &default_headers()
                );
                #[cfg(feature = "metrics")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Metric)).headers(),
                    &default_headers()
                );
                #[cfg(feature = "logs")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Log)).headers(),
                    &default_headers()
                );

                std::env::set_var(OTEL_EXPORTER_OTLP_HEADERS, "Accept=text/plain");
                assert_eq!(build_config_with_env(None).headers(), &otlp_headers);

                #[cfg(feature = "traces")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Trace)).headers(),
                    &otlp_headers
                );
                #[cfg(feature = "metrics")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Metric)).headers(),
                    &otlp_headers
                );
                #[cfg(feature = "logs")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Log)).headers(),
                    &otlp_headers
                );

                #[cfg(feature = "traces")]
                std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_HEADERS, "Accept=application/json");
                #[cfg(feature = "metrics")]
                std::env::set_var(OTEL_EXPORTER_OTLP_METRICS_HEADERS, "Accept=application/xml");
                #[cfg(feature = "logs")]
                std::env::set_var(OTEL_EXPORTER_OTLP_LOGS_HEADERS, "Accept=application/html");
                assert_eq!(build_config_with_env(None).headers(), &otlp_headers);

                #[cfg(feature = "traces")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Trace)).headers(),
                    &otlp_traces_headers
                );
                #[cfg(feature = "metrics")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Metric)).headers(),
                    &otlp_metrics_headers
                );
                #[cfg(feature = "logs")]
                assert_eq!(
                    build_config_with_env(Some(DataType::Log)).headers(),
                    &otlp_logs_headers
                );
            },
        );
    }

    #[cfg(feature = "grpc")]
    #[test]
    fn test_insecure_from_env() {
        temp_env::with_vars(
            vec![
                // insecure is used only when there is no https:// or http:// in endpoint
                (
                    OTEL_EXPORTER_OTLP_ENDPOINT,
                    Some("localhost:4317".to_owned()),
                ),
                (OTEL_EXPORTER_OTLP_PROTOCOL, Some("grpc".to_owned())),
                (OTEL_EXPORTER_OTLP_INSECURE, None::<String>),
                #[cfg(feature = "traces")]
                (OTEL_EXPORTER_OTLP_TRACES_INSECURE, None),
                #[cfg(feature = "metrics")]
                (OTEL_EXPORTER_OTLP_METRICS_INSECURE, None),
                #[cfg(feature = "logs")]
                (OTEL_EXPORTER_OTLP_LOGS_INSECURE, None),
            ],
            || {
                assert!(!build_config_with_env(None).insecure());

                #[cfg(feature = "traces")]
                assert!(!build_config_with_env(Some(DataType::Trace)).insecure(),);
                #[cfg(feature = "metrics")]
                assert!(!build_config_with_env(Some(DataType::Metric)).insecure(),);
                #[cfg(feature = "logs")]
                assert!(!build_config_with_env(Some(DataType::Log)).insecure());

                std::env::set_var(OTEL_EXPORTER_OTLP_INSECURE, "true");
                assert!(build_config_with_env(None).insecure());

                #[cfg(feature = "traces")]
                assert!(build_config_with_env(Some(DataType::Trace)).insecure(),);
                #[cfg(feature = "metrics")]
                assert!(build_config_with_env(Some(DataType::Metric)).insecure(),);
                #[cfg(feature = "logs")]
                assert!(build_config_with_env(Some(DataType::Log)).insecure());

                std::env::set_var(OTEL_EXPORTER_OTLP_INSECURE, "false");
                #[cfg(feature = "traces")]
                std::env::set_var(OTEL_EXPORTER_OTLP_TRACES_INSECURE, "true");
                #[cfg(feature = "metrics")]
                std::env::set_var(OTEL_EXPORTER_OTLP_METRICS_INSECURE, "true");
                #[cfg(feature = "logs")]
                std::env::set_var(OTEL_EXPORTER_OTLP_LOGS_INSECURE, "true");
                assert!(!build_config_with_env(None).insecure());

                #[cfg(feature = "traces")]
                assert!(build_config_with_env(Some(DataType::Trace)).insecure(),);
                #[cfg(feature = "metrics")]
                assert!(build_config_with_env(Some(DataType::Metric)).insecure(),);
                #[cfg(feature = "logs")]
                assert!(build_config_with_env(Some(DataType::Log)).insecure());
            },
        );
    }
}
