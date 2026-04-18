//! Observability setup: tracing + OpenTelemetry.
//!
//! When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, traces are sent via OTLP/gRPC.
//! Otherwise, structured JSON logs go to stderr with `RUST_LOG` filtering.

use opentelemetry::trace::TracerProvider;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// Initialize tracing. Call once at startup before any spans are created.
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("thymos=info,tower_http=info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true);

    // Try to set up OTLP exporter if endpoint is configured.
    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("OTLP exporter");

        let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(opentelemetry_sdk::Resource::builder()
                .with_service_name("thymos-server")
                .build())
            .build();

        let tracer = provider.tracer("thymos");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();

        eprintln!("telemetry: OTLP traces enabled");
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();

        eprintln!("telemetry: JSON logs to stderr (set OTEL_EXPORTER_OTLP_ENDPOINT for OTLP)");
    }
}
