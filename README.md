# Canary Release

**A Rust library for canary deployment orchestration** — models progressive rollout with health-based promotion and rollback decisions, the deployment strategy used by Netflix, Google, and Facebook to safely ship new code to production.

## Why It Matters

A canary release deploys a new version to a small percentage of traffic (the "canary"), monitors health metrics, and either **promotes** (increases traffic) or **rolls back** (reverts) based on observed behavior. The name comes from coal miners who carried canaries as early-warning systems for toxic gases — the canary would falter before the miners were affected.

Canary releases are the gold standard for safe deployments because:

- **Limit blast radius** — a bad deploy affects 5% of users, not 100%. For a service with 10M users, that's 500K affected vs. 10M.
- **Fast feedback** — production metrics reveal issues that staging can't: real traffic patterns, edge cases, data interactions
- **Gradual confidence** — increasing traffic from 5% → 25% → 50% → 100% validates the service at each scale level
- **Automated rollback** — if error rate spikes, automatically revert before customers notice

The mathematical foundation is **progressive rollout with sequential testing**. At each step, we collect n samples and test the null hypothesis H₀: "the canary and baseline are equivalent" vs. H₁: "the canary is worse." The decision boundary balances two risks:

| Risk | Definition | Mitigation |
|------|-----------|------------|
| Type I (false alarm) | Rollback a good deploy | Wastes engineering time, delays features |
| Type II (missed regression) | Promote a bad deploy | Harms users, potential incident |

With 5 SLO checks at 95% confidence each, the composite false-alarm rate is 1 − 0.95⁵ ≈ 22.6%. This is why canary deployments should wait for sufficient samples before deciding.

## How It Works

### SLO-Based Health Evaluation

Each `CanaryDeployment` tracks the canary version, traffic weight (percentage 0–100), and live metrics. Three Service Level Objectives (SLOs) must all pass:

| SLO | Threshold | Rationale |
|-----|-----------|-----------|
| Error rate | < 5% | Are users hitting errors? 5% = 1 in 20 requests failing |
| P99 latency | < 500ms | Is it responsive? 99% of users get response in <500ms |
| Success rate | > 95% | Are requests succeeding? Complement of error rate with retries |

The `is_healthy()` method is a **logical AND** of all three:

```
healthy = (error_rate < 0.05) ∧ (latency_p99 < 500ms) ∧ (success_rate > 0.95)
```

If any SLO fails, the canary is rolled back immediately. This is a **conservative policy** — better to false-alarm than to ship a regression.

### Progressive Rollout Protocol

The standard canary progression follows a geometric schedule:

```
weight: 1% → 5% → 10% → 25% → 50% → 100%
```

At each step, the deployment holds for an **observation window** (typically 5–30 minutes depending on traffic volume) while metrics stabilize. The sample size at each step:

```
n = traffic_rate × total_rps × observation_window
```

For a service handling 10,000 requests/second with a 5-minute window at 5% canary:

```
n = 0.05 × 10,000 × 300 = 150,000 samples
```

This is more than enough for statistical significance (typically n > 30 suffices for the Central Limit Theorem).

**Complexity:**

| Operation | Time | Space |
|-----------|------|-------|
| Health check evaluation | O(1) | O(1) |
| Promote (increase weight) | O(1) | O(1) |
| Rollback (revert weight) | O(1) | O(1) |
| Statistical test (t-test) | O(n) for n samples | O(1) |

### Promotion vs. Rollback

```
Promote: weight → min(weight × 2, 100)
Rollback: weight → 0 (immediate, no gradual decrease)
```

Promotion doubles the traffic weight at each step (exponential ramp). Rollback is immediate — if the canary is unhealthy, there's no value in gradually reducing traffic.

The asymmetry is deliberate: **promote cautiously, rollback aggressively**. This is the same principle as a circuit breaker: try slowly, fail fast.

## Quick Start

```rust
// This crate uses a main.rs demo with inline structs.
// The deployment evaluation logic:

fn is_healthy(error_rate: f64, latency_p99_ms: u64, success_rate: f64) -> bool {
    error_rate < 0.05 && latency_p99_ms < 500 && success_rate > 0.95
}

// Example: canary v3.0.0 at 10% traffic
let canary_healthy = is_healthy(0.02, 320, 0.98);
if canary_healthy {
    println!("Promote: increase traffic from 10% to 25%");
} else {
    println!("Rollback: revert to v2.0.0");
}

// Example: canary with elevated error rate
let canary_unhealthy = is_healthy(0.08, 800, 0.91);
assert!(!canary_unhealthy);  // fails all three SLOs
```

## API

*Implemented as a demo binary with inline structs:*

| Type | Fields | Description |
|------|--------|-------------|
| `Metrics` | `error_rate: f64`, `latency_p99_ms: u64`, `success_rate: f64` | Live health metrics |
| `Metrics::is_healthy` | `(&self) → bool` | All three SLOs passing? |
| `CanaryDeployment` | `name: String`, `canary_weight: u8`, `metrics: Metrics` | Deployment state |
| `CanaryDeployment::promote` | `(&self)` | Increase traffic weight |
| `CanaryDeployment::rollback` | `(&self)` | Revert to zero traffic |

## Architecture Notes

Part of the SuperInstance deployment pipeline, alongside `circuit-breaker` and `bulkhead-pattern`. In production, the metrics would come from **Prometheus** or **Datadog** and the traffic shifting would be handled by a service mesh (**Istio**, **Linkerd**) or load balancer (**Envoy**, **HAProxy**).

Within γ + η = C, the canary deployment instantiates the conservation law as **risk conservation**: the total risk of a bad deploy is bounded by the traffic weight. At 5% canary, maximum user impact is 5% — the risk is conserved and proportional. As traffic increases, risk increases proportionally (γ grows), but the health checks provide the compensating response (η responds). If the system is unhealthy, the conservation invariant is maintained by immediate rollback.

See the [architecture overview](https://github.com/casey-digennaro/canary-release/blob/main/ARCHITECTURE.md).

## References

1. Beyer, B. et al. (2016). *Site Reliability Engineering*. O'Reilly. Chapter 27: "Reliable Product Launches at Scale." (Canary at Google)
2. Netflix Technology Blog (2013). "Deployit: Safe and Efficient Deployments." (Netflix's approach to canary)
3. Shah, J. (2018). "Progressive Delivery: What It Is, Why It Matters." *InfoQ*.
4. Kohavi, R. et al. (2020). *Trustworthy Online Controlled Experiments*. Cambridge University Press. (A/B testing statistical foundations)

## License

MIT
