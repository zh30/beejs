//! Distributed tracing for Service Mesh
//! Provides simplified tracing implementation for distributed tracing

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global span ID counter
static SPAN_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static TRACE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_span_id() -> String {
    format!("span-{:016x}", SPAN_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
}

fn generate_trace_id() -> String {
    format!("trace-{:016x}", TRACE_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Distributed tracer
pub struct DistributedTracer {
    /// Service name
    service_name: String,

    /// Span history for analysis
    span_history: Vec<SpanRecord>,
}

impl DistributedTracer {
    /// Create a new distributed tracer
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            span_history: Vec::new(),
        }
    }

    /// Start a new span
    pub fn start_span(
        &mut self,
        operation_name: &str,
        trace_context: Option<TraceContext>,
    ) -> SpanHandle {
        let trace_id: _ = trace_context
            .as_ref()
            .map(|c| c.trace_id.clone())
            .unwrap_or_else(generate_trace_id);

        let span_id: _ = generate_span_id();
        let parent_span_id: _ = trace_context.map(|c| c.span_id);

        let record: _ = SpanRecord {
            span_id: span_id.clone(),
            trace_id: trace_id.clone(),
            parent_span_id,
            operation_name: operation_name.to_string(),
            service_name: self.service_name.clone(),
            start_time: std::time::Instant::now(),
            end_time: None,
            status: SpanStatus::Running,
            events: Vec::new(),
            attributes: HashMap::new(),
        };

        self.span_history.push(record);

        SpanHandle {
            span_id,
            trace_id,
            operation_name: operation_name.to_string(),
        }
    }

    /// Start a child span
    pub fn start_child_span(
        &mut self,
        parent: &SpanHandle,
        operation_name: &str,
    ) -> SpanHandle {
        let context: _ = TraceContext {
            trace_id: parent.trace_id.clone(),
            span_id: parent.span_id.clone(),
        };
        self.start_span(operation_name, Some(context))
    }

    /// End a span
    pub fn end_span(&mut self, span: SpanHandle) {
        if let Some(record) = self.span_history.iter_mut().find(|r| r.span_id == span.span_id) {
            record.end_time = Some(std::time::Instant::now());
            record.status = SpanStatus::Completed;
        }
    }

    /// Add event to span
    pub fn add_event(&mut self, span: &SpanHandle, event_name: &str, attributes: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>>>>) {
        if let Some(record) = self.span_history.iter_mut().find(|r| r.span_id == span.span_id) {
            record.events.push(SpanEvent {
                name: event_name.to_string(),
                timestamp: std::time::Instant::now(),
                attributes,
            });
        }
    }

    /// Add error to span
    pub fn add_error(&mut self, span: &SpanHandle, _error: &str) {
        if let Some(record) = self.span_history.iter_mut().find(|r| r.span_id == span.span_id) {
            record.status = SpanStatus::Error;
        }
    }

    /// Set span attribute
    pub fn set_attribute(&mut self, span: &SpanHandle, key: &str, value: &str) {
        if let Some(record) = self.span_history.iter_mut().find(|r| r.span_id == span.span_id) {
            record.attributes.insert(key.to_string(), value.to_string());
        }
    }

    /// Get span history
    pub fn get_span_history(&self) -> &[SpanRecord] {
        &self.span_history
    }

    /// Get trace by ID
    pub fn get_trace(&self, trace_id: &str) -> Option<Vec<&SpanRecord>> {
        let spans: Vec<&SpanRecord> = self.span_history
            .iter()
            .filter(|s| s.trace_id == trace_id)
            .collect();

        if spans.is_empty() {
            None
        } else {
            Some(spans)
        }
    }

    /// Analyze span performance
    pub fn analyze_performance(&self, trace_id: &str) -> Option<PerformanceAnalysis> {
        let trace_spans: _ = self.get_trace(trace_id)?;

        let mut total_duration = std::time::Duration::from_secs(0);
        let mut max_duration = std::time::Duration::from_secs(0);
        let mut slowest_span: Option<&SpanRecord> = None;

        for span in &trace_spans {
            if let Some(end_time) = span.end_time {
                let duration: _ = end_time.duration_since(span.start_time);
                total_duration += duration;

                if duration > max_duration {
                    max_duration = duration;
                    slowest_span = Some(span);
                }
            }
        }

        Some(PerformanceAnalysis {
            trace_id: trace_id.to_string(),
            total_spans: trace_spans.len(),
            total_duration,
            slowest_span: slowest_span.cloned(),
            average_duration: if !trace_spans.is_empty() {
                total_duration / trace_spans.len() as u32
            } else {
                std::time::Duration::from_secs(0)
            },
        })
    }
}

/// Span handle wrapper
#[derive(Debug, Clone)]
pub struct SpanHandle {
    span_id: String,
    trace_id: String,
    operation_name: String,
}

impl SpanHandle {
    /// Get span ID
    pub fn span_id(&self) -> &str {
        &self.span_id
    }

    /// Get trace ID
    pub fn trace_id(&self) -> &str {
        &self.trace_id
    }

    /// Get operation name
    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }
}

/// Trace context
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: String,

    /// Span ID
    pub span_id: String,
}

/// Span record for history
#[derive(Debug, Clone)]
pub struct SpanRecord {
    /// Span ID
    pub span_id: String,

    /// Trace ID
    pub trace_id: String,

    /// Parent span ID
    pub parent_span_id: Option<String>,

    /// Operation name
    pub operation_name: String,

    /// Service name
    pub service_name: String,

    /// Start time
    pub start_time: std::time::Instant,

    /// End time
    pub end_time: Option<std::time::Instant>,

    /// Span status
    pub status: SpanStatus,

    /// Events
    pub events: Vec<SpanEvent>,

    /// Attributes
    pub attributes: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>>>>,
}

/// Span status
#[derive(Debug, Clone, PartialEq)]
pub enum SpanStatus {
    Running,
    Completed,
    Error,
}

/// Span event
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Event name
    pub name: String,

    /// Event timestamp
    pub timestamp: std::time::Instant,

    /// Event attributes
    pub attributes: HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String, std::collections::HashMap<String, String, std::collections::HashMap<String, String, String, String, String, String, std::collections::HashMap<String, String, String, String>>>>>>>,
}

/// Performance analysis
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// Trace ID
    pub trace_id: String,

    /// Total number of spans
    pub total_spans: usize,

    /// Total duration
    pub total_duration: std::time::Duration,

    /// Slowest span
    pub slowest_span: Option<SpanRecord>,

    /// Average duration
    pub average_duration: std::time::Duration,
}

impl PerformanceAnalysis {
    /// Get slowest span duration
    pub fn slowest_duration(&self) -> Option<std::time::Duration> {
        self.slowest_span.as_ref().and_then(|s| {
            s.end_time.map(|end| end.duration_since(s.start_time))
        })
    }
}

/// Metrics collector for service mesh
pub struct MetricsCollector {
    /// Service name
    #[allow(dead_code)]
    service_name: String,

    /// Request metrics
    request_metrics: RequestMetrics,

    /// Latency metrics
    latency_metrics: LatencyMetrics,

    /// Error metrics
    error_metrics: ErrorMetrics,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(service_name: String) -> Self {
        Self {
            service_name: service_name.clone(),
            request_metrics: RequestMetrics::new(),
            latency_metrics: LatencyMetrics::new(),
            error_metrics: ErrorMetrics::new(),
        }
    }

    /// Record request
    pub fn record_request(
        &mut self,
        method: &str,
        path: &str,
        status_code: u16,
        latency_ms: u64,
    ) {
        self.request_metrics.record(method, path, status_code);
        self.latency_metrics.record(latency_ms);

        if status_code >= 400 {
            self.error_metrics.record(method, path, status_code);
        }
    }

    /// Get request metrics
    pub fn get_request_metrics(&self) -> &RequestMetrics {
        &self.request_metrics
    }

    /// Get latency metrics
    pub fn get_latency_metrics(&self) -> &LatencyMetrics {
        &self.latency_metrics
    }

    /// Get error metrics
    pub fn get_error_metrics(&self) -> &ErrorMetrics {
        &self.error_metrics
    }

    /// Generate metrics report
    pub fn generate_report(&self) -> MetricsReport {
        MetricsReport {
            service_name: self.service_name.clone(),
            total_requests: self.request_metrics.total_requests,
            success_rate: self.request_metrics.success_rate(),
            average_latency_ms: self.latency_metrics.average(),
            p95_latency_ms: self.latency_metrics.p95(),
            p99_latency_ms: self.latency_metrics.p99(),
            error_rate: self.error_metrics.rate(),
            top_error_endpoints: self.error_metrics.top_endpoints(5),
        }
    }
}

/// Request metrics
#[derive(Debug)]
pub struct RequestMetrics {
    /// Total requests
    pub total_requests: u64,

    /// Requests by method
    requests_by_method: HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64>>>>>>>,

    /// Requests by path
    requests_by_path: HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64>>>>>>>,

    /// Requests by status code
    requests_by_status: HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64, std::collections::HashMap<u16, u64, std::collections::HashMap<u16, u64, u16, u64, u16, u64, std::collections::HashMap<u16, u64, u16, u64>>>>>>>,
}

impl RequestMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            requests_by_method: HashMap::new(),
            requests_by_path: HashMap::new(),
            requests_by_status: HashMap::new(),
        }
    }

    fn record(&mut self, method: &str, path: &str, status_code: u16) {
        self.total_requests += 1;
        *self.requests_by_method.entry(method.to_string()).or_insert(0) += 1;
        *self.requests_by_path.entry(path.to_string()).or_insert(0) += 1;
        *self.requests_by_status.entry(status_code).or_insert(0) += 1;
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }

        let success_count: u64 = self.requests_by_status
            .iter()
            .filter(|(status, _)| **status < 400)
            .map(|(_, count)| *count)
            .sum();

        (success_count as f64 / self.total_requests as f64) * 100.0
    }
}

/// Latency metrics
#[derive(Debug)]
pub struct LatencyMetrics {
    /// Latency samples
    latencies: Vec<u64>,
}

impl LatencyMetrics {
    fn new() -> Self {
        Self {
            latencies: Vec::new(),
        }
    }

    fn record(&mut self, latency_ms: u64) {
        self.latencies.push(latency_ms);
    }

    fn average(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let sum: u64 = self.latencies.iter().sum();
        sum as f64 / self.latencies.len() as f64
    }

    fn p95(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut sorted = self.latencies.clone();
        sorted.sort();
        let index: _ = (sorted.len() as f64 * 0.95) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(0) as f64
    }

    fn p99(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }

        let mut sorted = self.latencies.clone();
        sorted.sort();
        let index: _ = (sorted.len() as f64 * 0.99) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(0) as f64
    }
}

/// Error metrics
#[derive(Debug)]
pub struct ErrorMetrics {
    /// Error count
    error_count: u64,

    /// Errors by endpoint
    errors_by_endpoint: HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64, std::collections::HashMap<String, u64, std::collections::HashMap<String, u64, String, u64, String, u64, std::collections::HashMap<String, u64, String, u64>>>>>>>,
}

impl ErrorMetrics {
    fn new() -> Self {
        Self {
            error_count: 0,
            errors_by_endpoint: HashMap::new(),
        }
    }

    fn record(&mut self, method: &str, path: &str, status_code: u16) {
        if status_code >= 400 {
            self.error_count += 1;
            let endpoint: _ = format!("{} {}", method, path);
            *self.errors_by_endpoint.entry(endpoint).or_insert(0) += 1;
        }
    }

    fn rate(&self) -> f64 {
        self.error_count as f64
    }

    fn top_endpoints(&self, limit: usize) -> Vec<(String, u64)> {
        let mut errors: Vec<_> = self.errors_by_endpoint.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));
        errors.into_iter().take(limit).map(|(k, v)| (k.clone(), *v)).collect()
    }
}

/// Metrics report
#[derive(Debug, Clone)]
pub struct MetricsReport {
    /// Service name
    pub service_name: String,

    /// Total requests
    pub total_requests: u64,

    /// Success rate percentage
    pub success_rate: f64,

    /// Average latency in milliseconds
    pub average_latency_ms: f64,

    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,

    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,

    /// Error rate
    pub error_rate: f64,

    /// Top error endpoints
    pub top_error_endpoints: Vec<(String, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[test]
    fn test_span_record() {
        let record: _ = SpanRecord {
            span_id: "span-123".to_string(),
            trace_id: "trace-456".to_string(),
            parent_span_id: None,
            operation_name: "test-operation".to_string(),
            service_name: "test-service".to_string(),
            start_time: std::time::Instant::now(),
            end_time: None,
            status: SpanStatus::Running,
            events: Vec::new(),
            attributes: HashMap::new(),
        };

        assert_eq!(record.span_id, "span-123");
        assert_eq!(record.trace_id, "trace-456");
        assert_eq!(record.status, SpanStatus::Running);
    }

    #[test]
    fn test_distributed_tracer() {
        let mut tracer = DistributedTracer::new("test-service".to_string());

        let span: _ = tracer.start_span("test-operation", None);
        assert!(!span.span_id().is_empty());
        assert!(!span.trace_id().is_empty());

        tracer.end_span(span);
        assert!(!tracer.get_span_history().is_empty());
    }

    #[test]
    fn test_metrics_collection() {
        let mut collector = MetricsCollector::new("test-service".to_string());

        collector.record_request("GET", "/api/users", 200, 50);
        collector.record_request("POST", "/api/users", 201, 75);
        collector.record_request("GET", "/api/users/1", 404, 25);

        let report: _ = collector.generate_report();

        assert_eq!(report.service_name, "test-service");
        assert_eq!(report.total_requests, 3);
        assert!(report.success_rate > 0.0);
        assert!(report.average_latency_ms > 0.0);
    }

    #[test]
    fn test_latency_percentiles() {
        let mut metrics = LatencyMetrics::new();

        metrics.record(10);
        metrics.record(20);
        metrics.record(30);
        metrics.record(40);
        metrics.record(50);

        assert!(metrics.average() > 0.0);
        assert!(metrics.p95() >= metrics.average());
        assert!(metrics.p99() >= metrics.p95());
    }
}
