# Canary Release

**A Rust library for canary deployment orchestration** — models progressive rollout with health-based promotion and rollback decisions, the deployment strategy used by Netflix, Google, and Facebook.

## Why It Matters

A canary release deploys a new version to a small percentage of traffic (the "canary"), monitors health metrics, and either promotes (increases traffic) or rolls back (reverts) based on observed behavior. The name comes from coal miners who carried canaries as early-warning systems for toxic gases.

Canary releases are the gold standard for safe deployments because:

- **Limit blast radius** — a bad deploy affects 5% of users, not 100%
- **Fast feedback** — production metrics reveal issues that staging can't (real traffic patterns, edge cases)
- **Gradual confidence** — increasing traffic from 5% → 25% → 50% → 100% validates at scale
- **Automated rollback** — if error rate spikes, automatically revert before customers notice

The health check evaluates three SLOs (Service Level Objectives):
- **Error rate** < 5% (are users hitting errors?)
- **P99 latency** < 500ms (is it responsive?)
- **Success rate** > 95% (are requests succeeding?)

## How It Works

**Metrics model**: Each `CanaryDeployment` tracks the canary version, traffic weight (percentage 0–100), and live metrics (error rate, P99 latency, success rate).

**Health evaluation**: The `is_healthy()` method applies the three SLO thresholds. If all three pass, the canary is promoted (traffic weight increased). If any fail, it's rolled back immediately.

**Promotion/rollback**: `promote()` increases the canary weight toward 100%; `rollback()` reverts to 0%. In a real system, this would trigger API calls to the load balancer (e.g., Envoy, HAProxy) to adjust routing weights.

## Quick Start

```rust
// This crate uses a main.rs demo with inline structs.
// The deployment evaluation logic:

fn is_healthy(error_rate: f64, latency_p99_ms: u64, success_rate: f64) -> bool {
    error_rate < 0.05 && latency_p99_ms < 500 && success_rate > 0.95
}

// Example: canary v3.0.0 at 10% traffic
let healthy = is_healthy(0.02, 320, 0.98);
if healthy {
    println!("Promote: increase traffic weight");
} else {
    println!("Rollback: revert to previous version");
}
```

## API

*Implemented as a demo binary with inline structs:*
- **`Metrics`** — error_rate, latency_p99_ms, success_rate; `is_healthy()`
- **`CanaryDeployment`** — name, canary_weight, metrics; `promote()`, `rollback()`

## Architecture Notes

Part of the SuperInstance deployment pipeline, alongside `circuit-breaker` and `bulkhead-pattern`. In production, the metrics would come from Prometheus/Datadog and the traffic shifting from a service mesh (Istio/Linkerd). See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
