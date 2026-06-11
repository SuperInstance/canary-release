use std::collections::HashMap;

#[derive(Debug)]
struct Metrics {
    error_rate: f64,
    latency_p99_ms: u64,
    success_rate: f64,
}

impl Metrics {
    fn is_healthy(&self) -> bool {
        self.error_rate < 0.05 && self.latency_p99_ms < 500 && self.success_rate > 0.95
    }
}

struct CanaryDeployment {
    name: String,
    canary_weight: u8,
    metrics: Metrics,
}

impl CanaryDeployment {
    fn promote(&self) { println!("Promoting canary '{}' to full rollout", self.name); }
    fn rollback(&self) { println!("Rolling back canary '{}'", self.name); }
}

fn main() {
    let canary = CanaryDeployment {
        name: "v3.0.0".into(),
        canary_weight: 10,
        metrics: Metrics { error_rate: 0.02, latency_p99_ms: 320, success_rate: 0.98 },
    };
    println!("Canary weight: {}%", canary.canary_weight);
    if canary.metrics.is_healthy() { canary.promote(); } else { canary.rollback(); }
}
