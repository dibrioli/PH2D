//! GPU-resident cook routing (GPU/M5 Fase 1 F1.1 + F1.2, ADR-0122).
//!
//! The per-frame decision — does this document's chain cook on the GPU, and if
//! so fully or from a CPU boundary — is a **pure function** of the plan and a
//! few flags, extracted here so it is unit-testable without a device (the bridge
//! tests are headless; the ε-parity of the actual dispatch is gated in the
//! motor, `ph2d-gpu-cook`'s `gpu_cpu_parity`). The bridge's `dispatch` reads the
//! route and drives the pump / `GpuCook` accordingly.

use ph2d_nodegraph::graph::NodeId;

/// Which cook path this frame takes.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum GpuRoute {
    /// The CPU pump renders the sinks (no GPU, or the GPU can't claim useful work).
    Cpu,
    /// The whole chain is kernel-covered: cook it 100% on the GPU, no CPU pump,
    /// no readback (F1.1).
    FullyGpu,
    /// A CPU prefix cooks up to this boundary node; the GPU runs the suffix (F1.2).
    Hybrid(NodeId),
}

/// Choose the cook route from the plan and this frame's flags — the one place
/// the "fully vs hybrid vs CPU" policy lives.
///
/// - GPU is opt-in (`gpu_enabled`, `PH2D_GPU_COOK=1`) and only for a **single**
///   sink with **no time scopes** — multi-sink and `motion.time_remap` recuse to
///   the CPU whole (F1.1's scope; F2+ territory).
/// - Fully-GPU when the plan claims the whole chain (`boundary` is `None`).
/// - Hybrid when the plan leaves a CPU boundary **and** the GPU suffix has at
///   least one dispatching stage — a boundary whose only GPU stage is the
///   pass-through `output` would upload the sink stream just to lower it (no
///   compute win), so that recuses to the CPU.
pub(super) fn gpu_route(
    gpu_enabled: bool,
    n_sinks: usize,
    scopes_empty: bool,
    boundary: Option<NodeId>,
    dispatching_stages: usize,
) -> GpuRoute {
    if !gpu_enabled || n_sinks != 1 || !scopes_empty {
        return GpuRoute::Cpu;
    }
    match boundary {
        None => GpuRoute::FullyGpu,
        Some(node) if dispatching_stages >= 1 => GpuRoute::Hybrid(node),
        Some(_) => GpuRoute::Cpu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A stand-in boundary node id (the routing never dereferences it).
    fn node() -> NodeId {
        NodeId(7)
    }

    #[test]
    fn disabled_or_multi_sink_or_scoped_is_always_cpu() {
        // Every gate is independent: flip one and the GPU is refused even when a
        // fully-claimed plan is on offer.
        assert_eq!(gpu_route(false, 1, true, None, 3), GpuRoute::Cpu);
        assert_eq!(gpu_route(true, 2, true, None, 3), GpuRoute::Cpu);
        assert_eq!(gpu_route(true, 0, true, None, 3), GpuRoute::Cpu);
        assert_eq!(gpu_route(true, 1, false, None, 3), GpuRoute::Cpu);
    }

    #[test]
    fn fully_claimed_plan_runs_fully_on_the_gpu() {
        assert_eq!(gpu_route(true, 1, true, None, 3), GpuRoute::FullyGpu);
    }

    #[test]
    fn a_boundary_with_gpu_work_is_hybrid() {
        assert_eq!(
            gpu_route(true, 1, true, Some(node()), 2),
            GpuRoute::Hybrid(node())
        );
        // One dispatching stage is enough.
        assert_eq!(
            gpu_route(true, 1, true, Some(node()), 1),
            GpuRoute::Hybrid(node())
        );
    }

    #[test]
    fn a_boundary_with_no_dispatching_suffix_recuses_to_cpu() {
        // A lone pass-through `output` above the boundary is no compute win —
        // uploading the sink stream just to lower it — so it stays on the CPU.
        assert_eq!(gpu_route(true, 1, true, Some(node()), 0), GpuRoute::Cpu);
    }
}
