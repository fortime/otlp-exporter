**_THIS IS A PERSONAL PROJECT. IT IS STILL IN DEVELOPMENT. USE ON YOUR OWN RISK._**

An exporter exports trace, metric and log data in the OTLP format.

# Support Matrix

## Protocol

| protocol         | trace    | metric   | log      |
| ---------------- | -------- | -------- | -------- |
| grpc(tonic)      | &check;  | &#x2610; | &#x2610; |
| grpc(grpcio)[^1] | &check;  | &#x2610; | &#x2610; |
| http/protobuf    | &check;  | &#x2610; | &#x2610; |
| http/json        | blocking | &#x2610; | &#x2610; |

## TLS

| dep     | std      | provided ca | client key |
| ------- | -------- | ----------- | ---------- |
| tonic   | not test | not test    | not test   |
| grpcio  | not test | not test    | not test   |
| reqwest | not test | not test    | not test   |

# Examples

- For `grpc`, we can use `install_simple` simply. It uses `future_executors`.

```rust
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

- For `http/protocol` and `http/json`, it depends on `reqwest` which depends on `tokio`. So, we must use `install_batch` with `Tokio`.

```rust
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

[^1]: As of 2023-08-16, grpc 0.12.1 can't be compiled with gcc 13, you can patch it with its git repo.
