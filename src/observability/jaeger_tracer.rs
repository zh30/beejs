//! Jaeger distributed tracing for Beejs runtime
//!
//! This module provides distributed tracing capabilities using Jaeger.
//! It allows tracking the execution of scripts, network operations,
//! and other runtime activities across service boundaries.

use anyhow::{Context, Result};
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::trace::{self, Tracer};
use opentelemetry::trace::{Span, SpanKind, Status, Tracer as TracerTrait};
use opentelemetry::{global, KeyValue};
use opentelemetry_jaeger::JaegerTraceRuntime;
use opentelemetry_sdk::trace::Tracer as SdkTracer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, instrument};

/// Jaeger tracer for distributed tracing
pub struct JaegerTracer {
    /// OpenTelemetry tracer
    tracer: Tracer,
    /// Trace context propagator
    propagator: TraceContextPropagator,
    /// Agent address for sending spans
    agent_addr: SocketAddr,
}

impl JaegerTracer {
    /// Create a new Jaeger tracer
    pub fn new(agent_addr: SocketAddr) -> Result<Self> {
        info!("Initializing Jaeger tracer with agent at {}", agent_addr);

        // Initialize the Jaeger exporter
        let agent_endpoint = format!("{}:{}", agent_addr.ip(), agent_addr.port());

        let tracer = opentelemetry_jaeger::new_pipeline()
            .with_agent_endpoint(agent_endpoint)
            .with_service_name("beejs-runtime")
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .context("Failed to install Jaeger tracer")?;

        // Initialize global propagator
        global::set_text_propagator(TraceContextPropagator::new());

        info!("Jaeger tracer initialized successfully");

        Ok(Self {
            tracer,
            propagator: TraceContextPropagator::new(),
            agent_addr,
        })
    }

    /// Create a new span for an operation
    pub fn create_span(&self, operation_name: &str) -> JaegerSpan {
        let span = self.tracer
            .span_builder(operation_name)
            .with_kind(SpanKind::Internal)
            .start(&self.tracer);

        debug!("Created Jaeger span for operation: {}", operation_name);

        JaegerSpan {
            span,
            tracer: &self.tracer,
        }
    }

    /// Create a span with parent context
    pub fn create_child_span(&self, operation_name: &str, parent: &JaegerSpan) -> JaegerSpan {
        let span = self.tracer
            .span_builder(operation_name)
            .with_parent(parent.span.span_context().clone())
            .with_kind(SpanKind::Internal)
            .start(&self.tracer);

        debug!("Created child Jaeger span for operation: {}", operation_name);

        JaegerSpan {
            span,
            tracer: &self.tracer,
        }
    }

    /// Extract trace context from carrier
    pub fn extract_from_carrier(&self, carrier: &HashMap<String, String>) -> opentelemetry::trace::SpanContext {
        // Simple implementation for HashMap carrier
        let mut extractor = opentelemetry::propagation::TextMapExtractor::new(carrier);
        self.propagator.extract(&mut extractor)
    }

    /// Inject trace context to carrier
    pub fn inject_to_carrier(&self, carrier: &mut HashMap<String, String>) {
        let mut injector = opentelemetry::propagation::TextMapInjector::new(carrier);
        self.propagator.inject(&opentelemetry::trace::TraceContext::new_with_span_context(
            &opentelemetry::trace::SpanContext::current(),
        ), &mut injector);
    }

    /// Get the tracer
    pub fn tracer(&self) -> &Tracer {
        &self.tracer
    }

    /// Record an error in the current span
    pub fn record_error(&self, error: &dyn std::error::Error) {
        error!("Recording error in Jaeger span: {}", error);
        global::tracer("beejs-runtime").record_error(error);
    }
}

/// A Jaeger span wrapper
pub struct JaegerSpan<'a> {
    /// The underlying OpenTelemetry span
    span: Span,
    /// Reference to the tracer
    tracer: &'a Tracer,
}

impl<'a> JaegerSpan<'a> {
    /// Add an attribute to the span
    pub fn set_attribute(&self, key: &str, value: &str) -> &Self {
        self.span.add_event(
            "annotation",
            vec![KeyValue::new(key.to_string(), value.to_string())],
        );
        self
    }

    /// Add multiple attributes to the span
    pub fn set_attributes(&self, attributes: HashMap<String, String>) -> &Self {
        for (key, value) in attributes {
            self.span.add_event(
                "annotation",
                vec![KeyValue::new(key, value)],
            );
        }
        self
    }

    /// Add an event to the span
    pub fn add_event(&self, event_name: &str, attributes: HashMap<String, String>) -> &Self {
        let mut event_attributes = Vec::new();
        for (key, value) in attributes {
            event_attributes.push(KeyValue::new(key, value));
        }

        self.span.add_event(event_name.to_string(), event_attributes);
        self
    }

    /// Set the span status
    pub fn set_status(&self, status: Status) -> &Self {
        self.span.set_status(status);
        self
    }

    /// Mark the span as successful
    pub fn success(&self) -> &Self {
        self.set_status(Status::Ok);
        self
    }

    /// Mark the span as failed with an error
    pub fn error(&self, error_message: &str) -> &Self {
        self.set_status(Status::error(error_message.to_string()));
        self
    }

    /// Get the span context
    pub fn span_context(&self) -> opentelemetry::trace::SpanContext {
        self.span.span_context().clone()
    }

    /// Get trace ID as a string
    pub fn trace_id_string(&self) -> String {
        format!("{:032x}", self.span.span_context().trace_id())
    }

    /// Get span ID as a string
    pub fn span_id_string(&self) -> String {
        format!("{:016x}", self.span.span_context().span_id())
    }
}

impl<'a> Drop for JaegerSpan<'a> {
    fn drop(&mut self) {
        self.span.end();
    }
}

/// Script execution tracer
pub struct ScriptTracer<'a> {
    span: JaegerSpan<'a>,
    start_time: SystemTime,
}

impl<'a> ScriptTracer<'a> {
    /// Create a new script tracer
    pub fn new(tracer: &'a JaegerTracer, script_name: &str) -> Self {
        let span = tracer.create_span(&format!("script:{}", script_name));

        // Add script metadata
        span.set_attribute("script.name", script_name);
        span.set_attribute("service.name", "beejs-runtime");

        Self {
            span,
            start_time: SystemTime::now(),
        }
    }

    /// Get the underlying span
    pub fn span(&self) -> &JaegerSpan {
        &self.span
    }

    /// Record script execution time
    pub fn record_execution_time(&self, duration: Duration) {
        self.span.set_attribute("script.duration_ms", &format!("{}", duration.as_millis()));
    }

    /// Record memory usage
    pub fn record_memory_usage(&self, bytes: usize) {
        self.span.set_attribute("script.memory_bytes", &format!("{}", bytes));
    }

    /// Record script success
    pub fn success(&self) {
        self.span.success();
        self.span.set_attribute("script.status", "success");
    }

    /// Record script failure
    pub fn error(&self, error_message: &str) {
        self.span.error(error_message);
        self.span.set_attribute("script.status", "error");
        self.span.set_attribute("script.error", error_message);
    }

    /// Get execution time
    pub fn execution_time(&self) -> Duration {
        self.start_time.duration_since(UNIX_EPOCH).unwrap_or_default()
    }
}

/// Network I/O tracer
pub struct NetworkTracer<'a> {
    span: JaegerSpan<'a>,
    operation: String,
}

impl<'a> NetworkTracer<'a> {
    /// Create a new network tracer
    pub fn new(tracer: &'a JaegerTracer, operation: &str, target: &str) -> Self {
        let span = tracer.create_span(&format!("network:{}", operation));

        span.set_attribute("network.operation", operation);
        span.set_attribute("network.target", target);

        Self {
            span,
            operation: operation.to_string(),
        }
    }

    /// Get the underlying span
    pub fn span(&self) -> &JaegerSpan {
        &self.span
    }

    /// Record bytes sent
    pub fn record_bytes_sent(&self, bytes: usize) {
        self.span.set_attribute("network.bytes_sent", &format!("{}", bytes));
    }

    /// Record bytes received
    pub fn record_bytes_received(&self, bytes: usize) {
        self.span.set_attribute("network.bytes_received", &format!("{}", bytes));
    }

    /// Record latency
    pub fn record_latency(&self, duration: Duration) {
        self.span.set_attribute("network.latency_ms", &format!("{}", duration.as_millis()));
    }

    /// Record success
    pub fn success(&self) {
        self.span.success();
        self.span.set_attribute("network.status", "success");
    }

    /// Record error
    pub fn error(&self, error_message: &str) {
        self.span.error(error_message);
        self.span.set_attribute("network.status", "error");
        self.span.set_attribute("network.error", error_message);
    }
}

/// JIT compilation tracer
pub struct JITTracer<'a> {
    span: JaegerSpan<'a>,
}

impl<'a> JITTracer<'a> {
    /// Create a new JIT tracer
    pub fn new(tracer: &'a JaegerTracer, function_name: &str) -> Self {
        let span = tracer.create_span(&format!("jit:{}", function_name));

        span.set_attribute("jit.function", function_name);

        Self { span }
    }

    /// Get the underlying span
    pub fn span(&self) -> &JaegerSpan {
        &self.span
    }

    /// Record compilation time
    pub fn record_compilation_time(&self, duration: Duration) {
        self.span.set_attribute("jit.compilation_time_ms", &format!("{}", duration.as_millis()));
    }

    /// Record optimization level
    pub fn record_optimization_level(&self, level: u32) {
        self.span.set_attribute("jit.optimization_level", &format!("{}", level));
    }

    /// Record success
    pub fn success(&self) {
        self.span.success();
        self.span.set_attribute("jit.status", "success");
    }

    /// Record error
    pub fn error(&self, error_message: &str) {
        self.span.error(error_message);
        self.span.set_attribute("jit.status", "error");
        self.span.set_attribute("jit.error", error_message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jaeger_tracer_creation() {
        let addr = "127.0.0.1:6831".parse().unwrap();
        let tracer = JaegerTracer::new(addr);
        assert!(tracer.is_ok());
    }

    #[test]
    fn test_jaeger_span_creation() {
        let addr = "127.0.0.1:6831".parse().unwrap();
        let tracer = JaegerTracer::new(addr).unwrap();

        let span = tracer.create_span("test_operation");
        assert_eq!(span.span_context().trace_id(), opentelemetry::trace::TraceId::INVALID);
    }

    #[test]
    fn test_script_tracer() {
        let addr = "127.0.0.1:6831".parse().unwrap();
        let tracer = JaegerTracer::new(addr).unwrap();

        let script_tracer = ScriptTracer::new(&tracer, "test.js");
        script_tracer.success();

        assert!(script_tracer.span().span_context().trace_id() != opentelemetry::trace::TraceId::INVALID);
    }

    #[test]
    fn test_network_tracer() {
        let addr = "127.0.0.1:6831".parse().unwrap();
        let tracer = JaegerTracer::new(addr).unwrap();

        let network_tracer = NetworkTracer::new(&tracer, "http_get", "example.com");
        network_tracer.record_bytes_sent(1024);
        network_tracer.record_bytes_received(2048);
        network_tracer.success();

        assert!(network_tracer.span().span_context().trace_id() != opentelemetry::trace::TraceId::INVALID);
    }

    #[test]
    fn test_jit_tracer() {
        let addr = "127.0.0.1:6831".parse().unwrap();
        let tracer = JaegerTracer::new(addr).unwrap();

        let jit_tracer = JITTracer::new(&tracer, "test_function");
        jit_tracer.record_compilation_time(Duration::from_millis(100));
        jit_tracer.record_optimization_level(3);
        jit_tracer.success();

        assert!(jit_tracer.span().span_context().trace_id() != opentelemetry::trace::TraceId::INVALID);
    }
}
