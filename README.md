An exporter exports trace, metric and log data in the OTLP format.

# Examples

* For `grpc`, we can use `install_simple` simply. It uses `future_executors`.
```
use opentelemetry_api::{trace::Tracer, global, KeyValue};
use opentelemetry_sdk::Resource;

#[tokio::main]
pub async fn main() {
    let tracer = match otlp_exporter::new_pipeline()
        .trace()
        .with_env()
        .with_tracer_config(
            opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "otlp-exporter-example",
            )])),
        )
        .install_simple()
    {
        Ok(tracer) => tracer,
        Err(e) => {
            println!("error: {e}");
            return;
        }
    };

    tracer.in_span("otlp-exporter trace example", |_cx| {});

    global::shutdown_tracer_provider();
}
```

* For `http/protocol` and `http/json`, it depends on `reqwest` which depends on `tokio`. So, we must use `install_batch` with `Tokio`.
```
use opentelemetry_api::{trace::Tracer, global, KeyValue};
use opentelemetry_sdk::{runtime::Tokio, Resource};

#[tokio::main]
pub async fn main() {
    let tracer = match otlp_exporter::new_pipeline()
        .trace()
        .with_env()
        .with_tracer_config(
            opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "otlp-exporter-example",
            )])),
        )
        .install_batch(Tokio)
    {
        Ok(tracer) => tracer,
        Err(e) => {
            println!("error: {e}");
            return;
        }
    };

    tracer.in_span("otlp-exporter trace example", |_cx| {});

    global::shutdown_tracer_provider();
}
```
