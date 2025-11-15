// Prometheus-compatible metrics for monitoring
//
// Provides basic request metrics, cache statistics, and error tracking
// in Prometheus text format for integration with monitoring systems.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

/// Global metrics collector
pub struct Metrics {
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
        }
    }

    // Request tracking
    pub fn record_request(&self, endpoint: &str) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        if let Ok(mut map) = self.endpoint_requests.lock() {
            *map.entry(endpoint.to_string()).or_insert(0) += 1;
        }
    }

    pub fn record_error(&self, endpoint: &str) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);

        if let Ok(mut map) = self.endpoint_errors.lock() {
            *map.entry(endpoint.to_string()).or_insert(0) += 1;
        }
    }

    // Cache tracking
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    // Authentication tracking
    pub fn record_login_attempt(&self) {
        self.login_attempts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_login_failure(&self) {
        self.login_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_active_sessions(&self) {
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_active_sessions(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    // Rate limiting
    pub fn record_rate_limit_hit(&self) {
        self.rate_limit_hits.fetch_add(1, Ordering::Relaxed);
    }

    // Circuit breaker
    pub fn record_circuit_breaker_open(&self) {
        self.circuit_breaker_opens.fetch_add(1, Ordering::Relaxed);
    }

    // Generate Prometheus-format metrics
    pub fn to_prometheus_format(&self) -> String {
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

        // Per-endpoint errors
        output.push_str("# HELP myt2abrp_endpoint_errors_total Errors per endpoint\n");
        output.push_str("# TYPE myt2abrp_endpoint_errors_total counter\n");
        if let Ok(map) = self.endpoint_errors.lock() {
            for (endpoint, count) in map.iter() {
                output.push_str(&format!(
                    "myt2abrp_endpoint_errors_total{{endpoint=\"{}\"}} {}\n",
                    endpoint, count
                ));
            }
        }
        output.push('\n');

        // Cache statistics
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total_cache_requests = hits + misses;
        let hit_rate = if total_cache_requests > 0 {
            (hits as f64 / total_cache_requests as f64) * 100.0
        } else {
            0.0
        };

        output.push_str("# HELP myt2abrp_cache_hits_total Cache hits\n");
        output.push_str("# TYPE myt2abrp_cache_hits_total counter\n");
        output.push_str(&format!("myt2abrp_cache_hits_total {}\n\n", hits));

        output.push_str("# HELP myt2abrp_cache_misses_total Cache misses\n");
        output.push_str("# TYPE myt2abrp_cache_misses_total counter\n");
        output.push_str(&format!("myt2abrp_cache_misses_total {}\n\n", misses));

        output.push_str("# HELP myt2abrp_cache_hit_rate_percent Cache hit rate percentage\n");
        output.push_str("# TYPE myt2abrp_cache_hit_rate_percent gauge\n");
        output.push_str(&format!("myt2abrp_cache_hit_rate_percent {:.2}\n\n", hit_rate));

        // Authentication metrics
        output.push_str("# HELP myt2abrp_login_attempts_total Total login attempts\n");
        output.push_str("# TYPE myt2abrp_login_attempts_total counter\n");
        output.push_str(&format!(
            "myt2abrp_login_attempts_total {}\n\n",
            self.login_attempts.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP myt2abrp_login_failures_total Failed login attempts\n");
        output.push_str("# TYPE myt2abrp_login_failures_total counter\n");
        output.push_str(&format!(
            "myt2abrp_login_failures_total {}\n\n",
            self.login_failures.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP myt2abrp_active_sessions Currently active sessions\n");
        output.push_str("# TYPE myt2abrp_active_sessions gauge\n");
        output.push_str(&format!(
            "myt2abrp_active_sessions {}\n\n",
            self.active_sessions.load(Ordering::Relaxed)
        ));

        // Rate limiting
        output.push_str("# HELP myt2abrp_rate_limit_hits_total Requests rejected by rate limiting\n");
        output.push_str("# TYPE myt2abrp_rate_limit_hits_total counter\n");
        output.push_str(&format!(
            "myt2abrp_rate_limit_hits_total {}\n\n",
            self.rate_limit_hits.load(Ordering::Relaxed)
        ));

        // Circuit breaker
        output.push_str("# HELP myt2abrp_circuit_breaker_opens_total Times circuit breaker opened\n");
        output.push_str("# TYPE myt2abrp_circuit_breaker_opens_total counter\n");
        output.push_str(&format!(
            "myt2abrp_circuit_breaker_opens_total {}\n\n",
            self.circuit_breaker_opens.load(Ordering::Relaxed)
        ));

        // Error rate
        let total_reqs = self.total_requests.load(Ordering::Relaxed);
        let total_errs = self.total_errors.load(Ordering::Relaxed);
        let error_rate = if total_reqs > 0 {
            (total_errs as f64 / total_reqs as f64) * 100.0
        } else {
            0.0
        };

        output.push_str("# HELP myt2abrp_error_rate_percent Overall error rate percentage\n");
        output.push_str("# TYPE myt2abrp_error_rate_percent gauge\n");
        output.push_str(&format!("myt2abrp_error_rate_percent {:.2}\n", error_rate));

        output
    }
}

// Global metrics instance using OnceLock for lazy initialization
static METRICS_INSTANCE: OnceLock<Metrics> = OnceLock::new();

/// Get the global metrics instance
pub fn metrics() -> &'static Metrics {
    METRICS_INSTANCE.get_or_init(|| Metrics::new())
}

// Public alias for easier access
pub static METRICS: MetricsProxy = MetricsProxy;

/// Proxy struct to provide easy access to metrics
pub struct MetricsProxy;

impl std::ops::Deref for MetricsProxy {
    type Target = Metrics;

    fn deref(&self) -> &'static Metrics {
        metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = Metrics::new();

        metrics.record_request("/test");
        metrics.record_request("/test");
        metrics.record_error("/test");

        assert_eq!(metrics.total_requests.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.total_errors.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = Metrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        assert_eq!(metrics.cache_hits.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.cache_misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_prometheus_format() {
        let metrics = Metrics::new();

        metrics.record_request("/test");
        metrics.record_cache_hit();

        let output = metrics.to_prometheus_format();

        assert!(output.contains("myt2abrp_requests_total 1"));
        assert!(output.contains("myt2abrp_cache_hits_total 1"));
        assert!(output.contains("# TYPE"));
        assert!(output.contains("# HELP"));
    }
}
