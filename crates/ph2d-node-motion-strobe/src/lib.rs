//! `motion.strobe` — a PULSE fires a decaying flash on the stream it passes
//! through (Motion Nodes M2; decision doc `06_pulse_*`, §4).
//!
//! This is the first CONSUMER of the pulse type, and the wave's visible payoff:
//! every pulse "lights up" the element (a size boost + a colour flash) that then
//! **decays geometrically** over the next ticks — attack-instant, decay-exp, the
//! minimal ADSR. It is the TouchDesigner Trigger CHOP's *"audio-style ADSR
//! envelope"* and Unreal's Notify-State *begin/tick/end*, reduced to the shape a
//! motion element actually needs.
//!
//! **The envelope IS the value of the `pre` self-loop.** A per-instance `glow`
//! (0..1) rides the `state` port: a pulse sets it to 1.0, and each tick with no
//! pulse multiplies it by `decay`. Applied ONCE per tick to the carried glow (so
//! a glow `n` ticks old has decayed exactly `n` times), never recomputed from an
//! age — the same geometric discipline as `motion.trail`. The lit look is
//! applied to the FRESH upstream `in` each tick (size/tint), so the boost never
//! compounds into the geometry; only `glow` persists.
//!
//! Positional per-instance (v1), matching `pulse.threshold`: `in`/`pulse`/`state`
//! pair by row order. The focus rig has a stable count.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The pulse type (mirror of `ph2d_node_pulse_threshold::PULSE`; kept local so
/// this crate stays a leaf drop-crate — the shared vocabulary is the port
/// `(Instances, Scalar, Event)`, not a shared symbol).
const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The pulse stream's fire column (`1.0` on a fired tick).
const PULSE_COL: &str = "pulse";
/// The per-instance envelope carried on the `pre` self-loop.
const GLOW_COL: &str = "glow";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.strobe"),
    name: "motion.strobe",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "pulse",
            ty: PULSE,
        },
        // The envelope feedback; `state` → editor-plumbed `pre` self-loop.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Per-tick envelope decay. 0.85 ≈ a ~0.2 s flash at 60 Hz.
        ParamSpec {
            name: "decay",
            default: 0.85,
        },
        // Peak size multiplier at full glow: size *= 1 + size_boost·glow.
        ParamSpec {
            name: "size_boost",
            default: 0.8,
        },
        // Flash colour + how much of it to mix at full glow (0 = size-only).
        ParamSpec {
            name: "flash_r",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_g",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_b",
            default: 1.0,
        },
        ParamSpec {
            name: "flash_amount",
            default: 0.9,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct Params {
    decay: f32,
    size_boost: f32,
    flash: [f32; 3],
    flash_amount: f32,
}

/// The envelope value for one instance this tick: a pulse re-arms it to 1.0,
/// otherwise it decays geometrically from last tick's value.
fn glow_of(pulse: f32, prev_glow: f32, decay: f32) -> f32 {
    if pulse > 0.5 { 1.0 } else { prev_glow * decay }
}

fn step(input: &Stream, pulse: &Stream, state: &Stream, p: &Params) -> Stream {
    let n = input.count();
    let pulses = scalar_col(pulse, PULSE_COL, n, 0.0);
    let prev_glow = scalar_col(state, GLOW_COL, n, 0.0);

    let glow: Vec<f32> = (0..n)
        .map(|i| glow_of(pulses[i], prev_glow[i], p.decay).clamp(0.0, 1.0))
        .collect();

    // Apply the lit look to the FRESH upstream geometry (never the state), so the
    // boost cannot compound; copy every other column through untouched.
    let mut size = vec2_col(input, "size", n, [1.0, 1.0]);
    let mut tint = vec4_col(input, "tint", n, [1.0, 1.0, 1.0, 1.0]);
    for i in 0..n {
        let g = glow[i];
        let k = 1.0 + p.size_boost * g;
        size[i] = [size[i][0] * k, size[i][1] * k];
        // Lerp RGB toward the flash colour by amount·glow; alpha untouched (a
        // flash brightens, it does not change opacity).
        let a = p.flash_amount * g;
        for (channel, &target) in tint[i].iter_mut().zip(p.flash.iter()) {
            *channel += (target - *channel) * a;
        }
    }

    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != "size" && name != "tint" {
            out.set(name.clone(), col.clone());
        }
    }
    out.set("size", Column::Vec2(size));
    out.set("tint", Column::Vec4(tint));
    out.set(GLOW_COL, Column::Scalar(glow));
    out
}

fn scalar_col(s: &Stream, name: &str, n: usize, id: f32) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}
fn vec2_col(s: &Stream, name: &str, n: usize, id: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}
fn vec4_col(s: &Stream, name: &str, n: usize, id: [f32; 4]) -> Vec<[f32; 4]> {
    let mut v = match s.get(name) {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, id);
    v
}

struct MotionStrobe;

impl NodeOp for MotionStrobe {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let p = Params {
            decay: ctx.param("decay"),
            size_boost: ctx.param("size_boost"),
            flash: [
                ctx.param("flash_r"),
                ctx.param("flash_g"),
                ctx.param("flash_b"),
            ],
            flash_amount: ctx.param("flash_amount"),
        };
        let out = step(ctx.input(0), ctx.input(1), ctx.input(2), &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionStrobe))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Strobe",
            // FX magenta: a stylistic flash effect.
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "decay",
        label: "Decay",
        min: 0.0,
        max: 0.99,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "size_boost",
        label: "Size Boost",
        min: 0.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // The flash colour authored as one swatch → OKLCH picker (the canonical
    // colour UI), driving the three linear channels. `flash_amount` stays a
    // plain slider (it is an intensity, not a colour channel).
    ParamUiHint {
        param: "flash_r",
        label: "Flash",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Color {
            channels: ["flash_r", "flash_g", "flash_b", "flash_amount"],
        },
    },
    ParamUiHint {
        param: "flash_amount",
        label: "Flash Amount",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn dot() -> Stream {
        Stream::new(1)
            .with("P", Column::Vec2(vec![[0.0, 0.0]]))
            .with("size", Column::Vec2(vec![[1.0, 1.0]]))
            .with("tint", Column::Vec4(vec![[0.2, 0.2, 0.2, 1.0]]))
    }
    fn fire(v: f32) -> Stream {
        Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
    }
    fn params() -> Params {
        Params {
            decay: 0.5,
            size_boost: 1.0,
            flash: [1.0, 1.0, 1.0],
            flash_amount: 1.0,
        }
    }
    fn glow(s: &Stream) -> f32 {
        match s.get(GLOW_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }
    fn size_x(s: &Stream) -> f32 {
        match s.get("size").unwrap() {
            Column::Vec2(v) => v[0][0],
            _ => panic!(),
        }
    }

    /// A pulse lights the element to full glow, then the envelope decays
    /// geometrically (×decay per tick) — size and flash follow it down. The
    /// upstream geometry is fresh each tick, so the boost never compounds.
    #[test]
    fn a_pulse_lights_then_the_envelope_decays_geometrically() {
        let p = params();
        // Tick 0: fire → glow 1.0, size ×(1+1·1)=2.0.
        let s = step(&dot(), &fire(1.0), &Stream::new(1), &p);
        assert_eq!(glow(&s), 1.0);
        assert_eq!(size_x(&s), 2.0);
        // Tick 1: no fire → glow ×0.5 = 0.5, size ×1.5 (from the FRESH unit size).
        let s = step(&dot(), &fire(0.0), &s, &p);
        assert_eq!(glow(&s), 0.5);
        assert_eq!(size_x(&s), 1.5);
        // Tick 2: glow 0.25, size ×1.25.
        let s = step(&dot(), &fire(0.0), &s, &p);
        assert_eq!(glow(&s), 0.25);
        assert_eq!(size_x(&s), 1.25);
    }

    /// FALSIFICATION of the "apply to fresh upstream, not to state" rule: the
    /// size boost must not COMPOUND. After a pulse and one decay tick, size is
    /// 1.5 (unit × 1.5), NOT 2.0 × 1.5 = 3.0 (which is what re-boosting the
    /// already-boosted state would give).
    #[test]
    fn the_size_boost_does_not_compound_across_ticks() {
        let p = params();
        let s = step(&dot(), &fire(1.0), &Stream::new(1), &p); // size 2.0
        let s = step(&dot(), &fire(0.0), &s, &p);
        assert_eq!(
            size_x(&s),
            1.5,
            "boost applies to fresh geometry, not to 2.0"
        );
    }

    /// At full glow the tint reaches the flash colour (amount 1.0); with no glow
    /// it is the untouched upstream tint. Alpha is never touched.
    #[test]
    fn the_flash_lerps_rgb_toward_the_flash_colour_leaving_alpha_alone() {
        let p = params();
        let lit = step(&dot(), &fire(1.0), &Stream::new(1), &p);
        match lit.get("tint").unwrap() {
            Column::Vec4(v) => {
                assert_eq!(v[0], [1.0, 1.0, 1.0, 1.0], "full flash = white, alpha kept")
            }
            _ => panic!(),
        }
        // Idle (no pulse, glow 0) → the upstream tint, verbatim.
        let dark = step(&dot(), &fire(0.0), &Stream::new(1), &p);
        match dark.get("tint").unwrap() {
            Column::Vec4(v) => assert_eq!(v[0], [0.2, 0.2, 0.2, 1.0]),
            _ => panic!(),
        }
    }

    /// A re-pulse mid-decay re-arms to full glow (the envelope retriggers, like a
    /// bang restarting a `line~`). Not additive — it resets, it does not stack.
    #[test]
    fn a_re_pulse_retriggers_the_envelope_to_full() {
        let p = params();
        let s = step(&dot(), &fire(1.0), &Stream::new(1), &p);
        let s = step(&dot(), &fire(0.0), &s, &p); // glow 0.5
        let s = step(&dot(), &fire(1.0), &s, &p); // re-fire
        assert_eq!(glow(&s), 1.0, "retrigger resets to full, not 0.5+something");
    }

    /// A bare positional stream (no size/tint) still gets those columns created
    /// at their identities and modulated — the strobe is not a no-op on a
    /// generator that only emits P.
    #[test]
    fn a_bare_stream_gains_size_and_tint_at_their_identities() {
        let p = params();
        let bare = Stream::new(1).with("P", Column::Vec2(vec![[5.0, 6.0]]));
        let lit = step(&bare, &fire(1.0), &Stream::new(1), &p);
        assert_eq!(size_x(&lit), 2.0, "unit identity ×2");
        match lit.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [5.0, 6.0], "P passes through"),
            _ => panic!(),
        }
    }
}
