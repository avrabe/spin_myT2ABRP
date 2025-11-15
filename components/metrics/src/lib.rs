// Prometheus-compatible metrics for monitoring
//
// Provides basic request metrics, cache statistics, and error tracking
// in Prometheus text format for integration with monitoring systems.

#[allow(warnings)]
mod bindings;

use bindings::exports::toyota::metrics::collector::Guest;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

struct Component;

/// Global metrics collector
struct Metrics {
    // Request counters
    total_requests: AtomicU64,
    total_errors: AtomicU64,

    // Per-endpoint counters (protected by mutex due to HashMap)
    endpoint_requests: Mutex<HashMap<String, u64>>,
    endpoint_errors: Mutex<HashMap<String, u64>>,

    // Cache statistics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,

    // Authentication metrics
    login_attempts: AtomicU64,
    login_failures: AtomicU64,
    active_sessions: AtomicU64,

    // Rate limiting
    rate_limit_hits: AtomicU64,

    // Circuit breaker
    circuit_breaker_opens: AtomicU64,

    // Retry logic
    retry_attempts: AtomicU64,
    retry_successes: AtomicU64,
    retry_exhausted: AtomicU64,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            total_requests: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            endpoint_requests: Mutex::new(HashMap::new()),
            endpoint_errors: Mutex::new(HashMap::new()),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            login_attempts: AtomicU64::new(0),
            login_failures: AtomicU64::new(0),
            active_sessions: AtomicU64::new(0),
            rate_limit_hits: AtomicU64::new(0),
            circuit_breaker_opens: AtomicU64::new(0),
            retry_attempts: AtomicU64::new(0),
            retry_successes: AtomicU64::new(0),
            retry_exhausted: AtomicU64::new(0),
        }
    }

    fn record_request(&self, endpoint: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut map) = self.endpoint_requests.lock() {
            *map.entry(endpoint.to_string()).or_insert(0) += 1;
        }
    }

    fn record_error(&self, endpoint: &str) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut map) = self.endpoint_errors.lock() {
            *map.entry(endpoint.to_string()).or_insert(0) += 1;
        }
    }

    fn to_prometheus_format(&self) -> String {
        let mut output = String::new();

        // Total requests
        output.push_str("# HELP myt2abrp_requests_total Total number of HTTP requests\n");
        output.push_str("# TYPE myt2abrp_requests_total counter\n");
        output.push_str(&format!(
            "myt2abrp_requests_total {}\n\n",
            self.total_requests.load(Ordering::Relaxed)
        ));

        // Total errors
        output.push_str("# HELP myt2abrp_errors_total Total number of errors\n");
        output.push_str("# TYPE myt2abrp_errors_total counter\n");
        output.push_str(&format!(
            "myt2abrp_errors_total {}\n\n",
            self.total_errors.load(Ordering::Relaxed)
        ));

        // Per-endpoint requests
        output.push_str("# HELP myt2abrp_endpoint_requests_total Requests per endpoint\n");
        output.push_str("# TYPE myt2abrp_endpoint_requests_total counter\n");
        if let Ok(map) = self.endpoint_requests.lock() {
            for (endpoint, count) in map.iter() {
                output.push_str(&format!(
                    "myt2abrp_endpoint_requests_total{{endpoint=\"{}\"}} {}\n",
                    endpoint, count
                ));
            }
        }
        output.push('\n');

        // Cache hits/misses
        output.push_str("# HELP myt2abrp_cache_hits_total Total cache hits\n");
        output.push_str("# TYPE myt2abrp_cache_hits_total counter\n");
        output.push_str(&format!(
            "myt2abrp_cache_hits_total {}\n\n",
            self.cache_hits.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP myt2abrp_cache_misses_total Total cache misses\n");
        output.push_str("# TYPE myt2abrp_cache_misses_total counter\n");
        output.push_str(&format!(
            "myt2abrp_cache_misses_total {}\n\n",
            self.cache_misses.load(Ordering::Relaxed)
        ));

        // Authentication
        output.push_str("# HELP myt2abrp_login_attempts_total Total login attempts\n");
        output.push_str("# TYPE myt2abrp_login_attempts_total counter\n");
        output.push_str(&format!(
            "myt2abrp_login_attempts_total {}\n\n",
            self.login_attempts.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP myt2abrp_active_sessions Current active sessions\n");
        output.push_str("# TYPE myt2abrp_active_sessions gauge\n");
        output.push_str(&format!(
            "myt2abrp_active_sessions {}\n\n",
            self.active_sessions.load(Ordering::Relaxed)
        ));

        // Circuit breaker
        output.push_str("# HELP myt2abrp_circuit_breaker_opens_total Circuit breaker open events\n");
        output.push_str("# TYPE myt2abrp_circuit_breaker_opens_total counter\n");
        output.push_str(&format!(
            "myt2abrp_circuit_breaker_opens_total {}\n\n",
            self.circuit_breaker_opens.load(Ordering::Relaxed)
        ));

        // Retry metrics
        output.push_str("# HELP myt2abrp_retry_attempts_total Total retry attempts\n");
        output.push_str("# TYPE myt2abrp_retry_attempts_total counter\n");
        output.push_str(&format!(
            "myt2abrp_retry_attempts_total {}\n\n",
            self.retry_attempts.load(Ordering::Relaxed)
        ));

        output
    }
}

// Global singleton
static METRICS: OnceLock<Metrics> = OnceLock::new();

fn get_metrics() -> &'static Metrics {
    METRICS.get_or_init(Metrics::new)
}

impl Guest for Component {
    fn record_request(endpoint: String) {
        get_metrics().record_request(&endpoint);
    }

    fn record_error(endpoint: String) {
        get_metrics().record_error(&endpoint);
    }

    fn record_cache_hit() {
        get_metrics().cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_cache_miss() {
        get_metrics().cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    fn record_login_attempt() {
        get_metrics().login_attempts.fetch_add(1, Ordering::Relaxed);
    }

    fn record_login_failure() {
        get_metrics().login_failures.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_active_sessions() {
        get_metrics().active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    fn decrement_active_sessions() {
        get_metrics().active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    fn record_rate_limit_hit() {
        get_metrics().rate_limit_hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_circuit_breaker_open() {
        get_metrics().circuit_breaker_opens.fetch_add(1, Ordering::Relaxed);
    }

    fn record_retry_attempt() {
        get_metrics().retry_attempts.fetch_add(1, Ordering::Relaxed);
    }

    fn record_retry_success() {
        get_metrics().retry_successes.fetch_add(1, Ordering::Relaxed);
    }

    fn record_retry_exhausted() {
        get_metrics().retry_exhausted.fetch_add(1, Ordering::Relaxed);
    }

    fn export_prometheus() -> String {
        get_metrics().to_prometheus_format()
    }
}

bindings::export!(Component with_types_in bindings);
