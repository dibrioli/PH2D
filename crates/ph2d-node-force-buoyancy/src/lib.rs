#![forbid(unsafe_code)]
//! `force.buoyancy` — **Archimedes, and a sea to float on.** A Motion force (marked
//! `Temporal`: the wave reads the playhead) that adds its contribution into the transient
//! `accel` column × the multiplicative `falloff` field; `motion.integrate` / `sim.step`
//! turns the accumulated acceleration into motion (Motion Nodes M2 forces — doc 01 §3 /
//! doc 60).
//!
//! ## The model
//!
//! The reference is the 2D one every game engine converged on — Unity's
//! `BuoyancyEffector2D` (a *surface level*, a *density*, a *linear drag*) — with the
//! surface promoted from a level to a **travelling wave**, because a flat sea does not
//! bob and bobbing is the entire reason to reach for this node.
//!
//! ```text
//! surface(x, t) = level + amplitude · sin( (x − speed·t) / wavelength )   [phase in cycles]
//! submersion    = clamp( (surface − y) / depth , 0 , 1 )        0 = dry, 1 = fully under
//! a  =  density · submersion · n            n = the surface NORMAL at x
//!    −  drag    · submersion · velocity
//! ```
//!
//! Three things fall out of that, and each is a gate below:
//!
//! - **It floats.** The buoyant force grows with how deep the thing is, so with gravity
//!   pulling `g` and this pushing `density` the object settles at the depth where they
//!   cancel — `submersion = g/density`. It is not a floor you sit on: push it under and
//!   it springs back, drop it from high and it plunges and rises. (`density > g` or it
//!   sinks — as it should. The demo's water is `12` against a gravity of `4`.)
//! - **It bobs sideways too.** The buoyant force is normal to the *surface*, not straight
//!   up (pressure is normal to the isobar), so on the flank of a wave it has a horizontal
//!   component pointing **downhill** — a float drifts into the trough and rides the swell
//!   instead of pumping up and down on the spot. The slope comes free: it is the `cos`
//!   the same parabolic-sine pair already computes.
//! - **Water is thick.** `drag`, applied only to the submerged fraction, is what stops the
//!   float oscillating forever; it is why a thing that hits the water *slows* rather than
//!   bouncing, and it is the same `−k·v` the standalone `force.drag` applies (this one is
//!   gated by submersion, so a thing in the air is untouched by it).
//!
//! A horizontal current is *not* a param here: that is `force.wind` with `angle = 0`, the
//! same argument by which there is no separate gravity node.
//!
//! HR-5: the wave is the corrected parabolic sine (sibling `trig.rs`), the normal is a
//! `sqrt` — no transcendentals.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod accum;
mod trig;
use accum::{add_accel, falloff_at, vec2_at};
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Zero draft would divide by zero: an object of no depth is submerged infinitely fast.
const MIN_DEPTH: f32 = 1e-3;
/// Zero wavelength is a wave of infinite frequency — spatial aliasing, and a division by
/// zero in the slope.
const MIN_WAVELENGTH: f32 = 1e-3;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.buoyancy"),
    name: "force.buoyancy",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // The wave reads the playhead. (Convention: reads playhead ⇒ Temporal — only a
    // Temporal manifest folds the playhead into the memo fingerprint, so a same-tick
    // re-cook at a moved playhead returns the sea where it now is, not where it was.)
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // The still-water line (world Y): where the sea sits with no wave.
        ParamSpec {
            name: "level",
            default: 0.0,
        },
        // The buoyant acceleration at FULL submersion. Beat gravity and it floats; lose
        // to gravity and it sinks (a stone is not a bug).
        ParamSpec {
            name: "density",
            default: 12.0,
        },
        // The object's draft: how far under the surface it must be to be fully submerged.
        // This is what `size` would be if the substrate had one true notion of it.
        ParamSpec {
            name: "depth",
            default: 0.3,
        },
        // Viscous damping, applied to the submerged fraction only.
        ParamSpec {
            name: "drag",
            default: 2.0,
        },
        // The swell. Amplitude 0 = a flat sea (and the node is then a pure float line).
        ParamSpec {
            name: "wave_amplitude",
            default: 0.12,
        },
        ParamSpec {
            name: "wave_length",
            default: 2.5,
        },
        // World units per second the crests travel in +X (negative = the other way).
        ParamSpec {
            name: "wave_speed",
            default: 0.4,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct ForceBuoyancy;

impl NodeOp for ForceBuoyancy {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let level = ctx.param("level");
        let density = ctx.param("density").max(0.0);
        let depth = ctx.param("depth").max(MIN_DEPTH);
        let drag = ctx.param("drag").max(0.0);
        let amp = ctx.param("wave_amplitude");
        let lambda = ctx.param("wave_length").max(MIN_WAVELENGTH);
        let speed = ctx.param("wave_speed");
        let t = ctx.playhead() as f32;

        let out = {
            let input = ctx.input(0);
            let contrib: Vec<[f32; 2]> = (0..input.count())
                .map(|i| {
                    let p = vec2_at(input, "P", i, [0.0, 0.0]);
                    let vel = vec2_at(input, "vel", i, [0.0, 0.0]);

                    // The sea at this instance's x, right now.
                    let phase = (p[0] - speed * t) / lambda;
                    let (cos, sin) = cos_sin_cycles(phase);
                    let surface = level + amp * sin;
                    // d/dx of `amp·sin(2π·(x − vt)/λ)` — the surface slope under it.
                    let slope = amp * (std::f32::consts::TAU / lambda) * cos;

                    // How much of it is under water: 0 dry, 1 fully submerged.
                    let sub = ((surface - p[1]) / depth).clamp(0.0, 1.0);
                    let w = sub * falloff_at(input, i);

                    // Buoyancy is normal to the surface: n = normalize(−slope, 1). On the
                    // flank of a wave that tilts the push downhill, into the trough.
                    let inv_len = 1.0 / (slope * slope + 1.0).sqrt();
                    [
                        (density * -slope * inv_len - drag * vel[0]) * w,
                        (density * inv_len - drag * vel[1]) * w,
                    ]
                })
                .collect();
            add_accel(input, &contrib)
        };
        ctx.emit(out);
    }
}

/// The GPU kernel (ADR-0126): **side metadata**, not a lowering — the manifest above is
/// untouched and `eval` stays the canonical answer. This one exists because a force with no
/// kernel is not "one node slower": a single uncovered node **inside the loop** leaves a
/// boundary, and the boundary makes `plan` refuse the whole simulation (the two-sims rule).
/// Five forces on the GPU are worth nothing in the graph that drops a buoy in the water.
///
/// The wave is the sibling `trig.rs` ported literally — the same corrected parabolic sine
/// `force.wind` already carries, so the sea has one shape on both sides (HR-5). `sqrt` and
/// `/` are *not* bit-exact by the Vulkan guarantee (3 / 2.5 ULP), which is why this is
/// gated at the ε the sim parity already budgets, and why `force.attractor` — sqrt and a
/// divide in the same breath — is the precedent that says the budget holds.
///
/// **The param clamps are part of the model, not hygiene** — `eval` reads `depth`/`wave_length`
/// through `.max(1e-3)` and `density`/`drag` through `.max(0.0)`, and a kernel taking the raw
/// uniform would answer a different node. They are not equally *observable*, and the gates say
/// which is which rather than implying they all are:
///
/// - `wave_length` is gated (`a_sea_of_no_wavelength_matches_the_cpu`): drop the clamp and
///   `phase = x/0 = inf`, `frac(inf) = NaN`, and the whole field NaNs while the CPU sails on.
/// - `depth` **cannot be caught by any fixture**, and that is worth knowing rather than
///   papering over: the downstream `clamp(sub, 0, 1)` already tames the ±inf, so the two paths
///   differ only for an instance sitting *exactly* on the waterline — where the CPU reads
///   `0/1e-3 = 0` (dry, does not move) and an unclamped GPU reads `0/0 = NaN`, which
///   `motion.integrate`'s own finiteness guard rejects (so it also does not move). The
///   integrator's guard converges them. It stays because it mirrors `eval`, which is the
///   contract — not because a red gate is holding it here.
/// - `density`/`drag` `.max(0.0)` is likewise below the resolution of a parity gate (a
///   negative density is a well-defined, if odd, sea on both sides).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let by_p = read_P(i);\n\
        let by_vel = read_vel(i);\n\
        let by_lambda = max(params.wave_length, 1e-3);\n\
        let by_cs = by_cos_sin_cycles(\n\
        \x20   (by_p.x - params.wave_speed * params.playhead) / by_lambda);\n\
        let by_surface = params.level + params.wave_amplitude * by_cs.y;\n\
        // d/dx of `amp·sin(2π·(x − vt)/λ)` — the surface slope under it.\n\
        let by_slope = params.wave_amplitude * (6.2831855 / by_lambda) * by_cs.x;\n\
        let by_sub = clamp((by_surface - by_p.y) / max(params.depth, 1e-3), 0.0, 1.0);\n\
        let by_w = by_sub * read_falloff(i);\n\
        // Buoyancy is normal to the surface: n = normalize(-slope, 1).\n\
        let by_inv_len = 1.0 / sqrt(by_slope * by_slope + 1.0);\n\
        let by_dens = max(params.density, 0.0);\n\
        let by_drag = max(params.drag, 0.0);\n\
        write_accel(i, read_accel(i) + vec2<f32>(\n\
        \x20   (by_dens * -by_slope * by_inv_len - by_drag * by_vel.x) * by_w,\n\
        \x20   (by_dens * by_inv_len - by_drag * by_vel.y) * by_w));\n",
    wgsl_lib: "\
        fn by_sin_cycles(phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn by_cos_sin_cycles(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(by_sin_cycles(phase + 0.25), by_sin_cycles(phase));\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "accel",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        // The drag term reads velocity. A field with no `vel` is a still sea acting on
        // still things: identity 0 makes `−drag·v` vanish, exactly as `vec2_at`'s default
        // does on the CPU.
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: &[
        "level",
        "density",
        "depth",
        "drag",
        "wave_amplitude",
        "wave_length",
        "wave_speed",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ForceBuoyancy))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // ADR-0130: per-element force: accumulates accel, identity preserved.
    reg.register_dense_window(MANIFEST.id);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Buoyancy",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

/// Param UI hints (M1.P1).
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "level",
        label: "Level",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "density",
        label: "Density",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "depth",
        label: "Depth",
        min: 0.01,
        max: 3.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "drag",
        label: "Drag",
        min: 0.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "wave_amplitude",
        label: "Wave Amplitude",
        min: 0.0,
        max: 2.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "wave_length",
        label: "Wave Length",
        min: 0.05,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "wave_speed",
        label: "Wave Speed",
        min: -5.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// A one-instance source at `(x, y)` with velocity `vel` — and, when asked, a half
    /// `falloff` so the field's gating is visible too.
    struct Src {
        p: [f32; 2],
        vel: [f32; 2],
        falloff: Option<f32>,
    }
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.buoyancy.test.src"),
        name: "force.buoyancy.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let mut s = Stream::new(1)
                .with("P", Column::Vec2(vec![self.p]))
                .with("vel", Column::Vec2(vec![self.vel]));
            if let Some(f) = self.falloff {
                s = s.with("falloff", Column::Scalar(vec![f]));
            }
            ctx.emit(s);
        }
    }
    struct Ops(Src);
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            if ty == MANIFEST.id {
                Some(&ForceBuoyancy as &dyn NodeOp)
            } else if ty == SRC_MAN.id {
                Some(&self.0 as &dyn NodeOp)
            } else {
                None
            }
        }
    }

    /// The acceleration this force contributes to one instance.
    fn accel(src: Src, params: &[(&str, f32)], t: f64) -> [f32; 2] {
        let mut g = Graph::new();
        let s = g.add_node("force.buoyancy.test.src");
        let b = g.add_node("force.buoyancy");
        g.connect(Edge {
            from: (s, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
        for (k, v) in params {
            g.set_param(b, *k, *v);
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops(src), b, t).unwrap();
        match out[0].as_stream().get("accel") {
            Some(Column::Vec2(v)) => v[0],
            _ => panic!("a force writes accel"),
        }
    }

    /// A still sea: no wave, so the only thing acting is the vertical push.
    const FLAT: &[(&str, f32)] = &[("wave_amplitude", 0.0), ("level", 0.0), ("drag", 0.0)];

    fn at(y: f32) -> Src {
        Src {
            p: [0.0, y],
            vel: [0.0, 0.0],
            falloff: None,
        }
    }

    /// **Above the water the node does nothing.** The gate that keeps a force honest: it
    /// acts where its field is, and nowhere else. (A missing `clamp(0,1)` makes the
    /// submersion negative up here and the sea would *suck things down* from the sky.)
    #[test]
    fn a_thing_in_the_air_is_untouched() {
        assert_eq!(accel(at(1.0), FLAT, 0.0), [0.0, 0.0]);
        // …including a fast-moving one: the drag is gated by submersion, not applied to
        // everything that passes overhead.
        let flying = Src {
            p: [0.0, 1.0],
            vel: [5.0, -9.0],
            falloff: None,
        };
        assert_eq!(accel(flying, &[("drag", 8.0)], 0.0), [0.0, 0.0]);
    }

    /// Fully under, the push is the density, straight up.
    #[test]
    fn a_submerged_thing_is_pushed_up_by_its_density() {
        let a = accel(at(-1.0), &[("wave_amplitude", 0.0), ("density", 12.0)], 0.0);
        assert!((a[1] - 12.0).abs() < 1e-4, "expected +12 up, got {a:?}");
        assert!(a[0].abs() < 1e-6, "a flat sea pushes straight up");
    }

    /// **The submersion RAMPS** — this is the difference between floating and standing on
    /// a floor. Half a draft under the surface, half the force; a node that binarised the
    /// test (`under ? density : 0`) passes the two gates above and fails this one.
    #[test]
    fn the_push_grows_with_how_deep_it_sits() {
        let params = &[("wave_amplitude", 0.0), ("density", 12.0), ("depth", 0.4)];
        let quarter = accel(at(-0.1), params, 0.0)[1];
        let half = accel(at(-0.2), params, 0.0)[1];
        let full = accel(at(-0.4), params, 0.0)[1];
        let deeper = accel(at(-4.0), params, 0.0)[1];
        assert!((quarter - 3.0).abs() < 1e-4, "a quarter under: {quarter}");
        assert!((half - 6.0).abs() < 1e-4, "half under: {half}");
        assert!((full - 12.0).abs() < 1e-4, "fully under: {full}");
        assert!(
            (deeper - 12.0).abs() < 1e-4,
            "and it does not keep growing below that: {deeper}"
        );
    }

    /// **It FLOATS** — the product claim, not a component one. With gravity `g` and
    /// density `d` the thing settles where the two cancel: `submersion = g/d`, i.e. a
    /// draft-fraction `g/d` under the surface. Assert the net acceleration there is zero.
    #[test]
    fn it_settles_where_buoyancy_cancels_gravity() {
        const G: f32 = 4.0;
        const D: f32 = 12.0;
        const DEPTH: f32 = 0.3;
        // g/d = 1/3 of the draft below the surface.
        let y = -DEPTH * (G / D);
        let a = accel(
            at(y),
            &[
                ("wave_amplitude", 0.0),
                ("density", D),
                ("depth", DEPTH),
                ("drag", 0.0),
            ],
            0.0,
        );
        assert!(
            (a[1] - G).abs() < 1e-3,
            "at the waterline the lift should exactly answer gravity ({G}), got {}",
            a[1]
        );
        // Push it under and it comes back up harder; lift it and it falls back.
        assert!(accel(at(y - 0.05), FLAT_D, 0.0)[1] > accel(at(y), FLAT_D, 0.0)[1]);
        assert!(accel(at(y + 0.05), FLAT_D, 0.0)[1] < accel(at(y), FLAT_D, 0.0)[1]);
    }
    const FLAT_D: &[(&str, f32)] = &[
        ("wave_amplitude", 0.0),
        ("density", 12.0),
        ("depth", 0.3),
        ("drag", 0.0),
    ];

    /// Water is thick: the drag opposes the velocity, and only under water.
    #[test]
    fn drag_brakes_the_submerged() {
        let moving = Src {
            p: [0.0, -1.0],
            vel: [2.0, -3.0],
            falloff: None,
        };
        let a = accel(
            moving,
            &[("wave_amplitude", 0.0), ("density", 0.0), ("drag", 2.0)],
            0.0,
        );
        assert!((a[0] - -4.0).abs() < 1e-4, "−k·v in x: {a:?}");
        assert!((a[1] - 6.0).abs() < 1e-4, "−k·v in y: {a:?}");
    }

    /// **The wave travels.** A crest at `x` now is at `x + speed·Δt` later — so the force
    /// on a float at `x` now equals the force on a float at `x + speed·Δt` then. This is
    /// the identity that pins BOTH the sign of `wave_speed` and the sign of the phase,
    /// which no static snapshot of the surface can.
    #[test]
    fn the_swell_moves_downstream() {
        let p = &[
            ("wave_amplitude", 0.3),
            ("wave_length", 2.0),
            ("wave_speed", 0.5),
        ];
        let here_now = accel(
            Src {
                p: [0.6, -0.2],
                vel: [0.0, 0.0],
                falloff: None,
            },
            p,
            0.0,
        );
        // 2 s later the same water is 1.0 world unit downstream.
        let there_later = accel(
            Src {
                p: [1.6, -0.2],
                vel: [0.0, 0.0],
                falloff: None,
            },
            p,
            2.0,
        );
        assert!(
            (here_now[0] - there_later[0]).abs() < 1e-3
                && (here_now[1] - there_later[1]).abs() < 1e-3,
            "the wave should have carried this exact water downstream: {here_now:?} vs \
             {there_later:?}"
        );
    }

    /// **The push tilts downhill.** On the flank of a wave the buoyant force is normal to
    /// the surface, so it has a horizontal component pointing toward the trough — which is
    /// what makes a float ride a swell instead of pumping on the spot. (Straight-up
    /// buoyancy passes every gate above and fails this one.)
    ///
    /// A frozen wave (speed 0) of wavelength 4, so the geometry is nameable: the surface
    /// **climbs** from the zero-crossing at `x=0` to the crest at `x=1`, is **flat** on the
    /// crest, and **falls** from there to the trough at `x=3`. The float leans away from
    /// the climb on the way up and away from the fall on the way down — always into the
    /// trough. (The mirror flank is at `x=2`, NOT at `x=−2`: a sine is odd, so its slope
    /// is *symmetric* about the origin, and both sides of `x=0` climb the same way. My
    /// first version of this gate asserted the opposite and this test caught me, not the
    /// code.)
    #[test]
    fn on_a_slope_the_float_is_pushed_toward_the_trough() {
        let p = &[
            ("wave_amplitude", 0.5),
            ("wave_length", 4.0),
            ("wave_speed", 0.0),
            ("drag", 0.0),
        ];
        let under = |x: f32| Src {
            p: [x, -1.0],
            vel: [0.0, 0.0],
            falloff: None,
        };
        let climbing = accel(under(0.5), p, 0.0);
        assert!(
            climbing[0] < -0.1,
            "the surface climbs to the right here, so the push leans left: {climbing:?}"
        );
        let falling = accel(under(2.0), p, 0.0);
        assert!(
            falling[0] > 0.1,
            "and on the far flank, where it falls, the push leans right: {falling:?}"
        );
        // On the crest and in the trough the surface is flat: no lean either way.
        assert!(
            accel(under(1.0), p, 0.0)[0].abs() < 0.05,
            "the crest is flat"
        );
        assert!(
            accel(under(3.0), p, 0.0)[0].abs() < 0.05,
            "the trough is flat"
        );
    }

    /// The multiplicative `falloff` field gates it like every other force (plan §1.6): a
    /// half falloff is half a sea.
    #[test]
    fn the_falloff_field_scales_the_force() {
        let half = Src {
            p: [0.0, -1.0],
            vel: [0.0, 0.0],
            falloff: Some(0.5),
        };
        let a = accel(half, &[("wave_amplitude", 0.0), ("density", 12.0)], 0.0);
        assert!(
            (a[1] - 6.0).abs() < 1e-4,
            "half the field, half the lift: {a:?}"
        );
    }
}
