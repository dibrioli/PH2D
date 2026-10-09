#![forbid(unsafe_code)]
//! `motion.stagger` — a Motion **behaviour**: writes a per-index ramp `min→max`
//! (shaped by an easing curve) into a chosen channel of the stream, **added** to
//! the existing value and scaled per-instance by the multiplicative `falloff`
//! column (§1.2; absent → `1.0`). The ramp uses the instance's position in the
//! stream (`i / (n-1)`), so the first instance takes ≈`min·falloff` and the last
//! ≈`max·falloff` (`reverse` flips the direction). Index-based, not time-based →
//! `Pure`. Every other column passes through unchanged (count preserved).
//!
//! Params (read via `ctx.param`):
//! - `channel` (1): target channel — `0` X, `1` Y, `2` Rotation, `3` Size.
//!   Position is world units, Rotation is **degrees** (the `rot` column's unit),
//!   Size a scale delta on the unit identity `[1,1]`.
//! - `min` (-1), `max` (1): the ramp endpoints (channel-native units).
//! - `ease_curve` (0): curve family — `0` Linear, `1` Quad, `2` Cubic, `3` Quart,
//!   `4` Quint, `5` Circ, `6` Back, `7` Bounce. All polynomial except Circ,
//!   which uses IEEE `sqrt` — correctly rounded, so still deterministic (HR-5;
//!   no transcendentals anywhere).
//! - `ease_dir` (0): the family's direction — `0` In, `1` Out, `2` In-Out.
//! - `reverse` (0/1): mirror the ramp (last instance takes `min`).
//!
//! `delta_i = (min + ease(raw_i)·(max−min)) · falloff_i`, added to the channel.

use ph2d_node_registry::{
    NodeRegistry, ParamChannelRange, ParamUnit, ParamUnitDecl, RegistryError,
};
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod ease;
mod kernel;
pub mod order;
use channel::{apply_channel_delta, falloff_at};
use ease::ease;
use ph2d_curve::Curve;

/// A chave do text param que carrega a forma da ease `Custom` (uma string `ph2d-curve`,
/// autorada pelo editor `ParamWidget::Curve`). **NÃO** é um `ParamSpec` — uma curva não é
/// um número. Ver [`ease::EASE_CUSTOM`].
pub const CURVE_KEY: &str = "curve";

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.stagger"),
    name: "motion.stagger",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Index-based (no playhead, no state) → freely cacheable.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0,
        },
        ParamSpec {
            name: "min",
            default: -1.0,
        },
        ParamSpec {
            name: "max",
            default: 1.0,
        },
        // Easing = a curve family shaped by a direction (see `ease`).
        ParamSpec {
            name: "ease_curve",
            default: 0.0,
        },
        ParamSpec {
            name: "ease_dir",
            default: 0.0,
        },
        // O *Offset* da Cavalry: desliza a rampa ao longo do índice, ciclicamente.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "reverse",
            default: 0.0,
        },
        // ⭐⭐⭐ **A ORDEM** (ciclo 2, W2 — ver [`order`]). Apendada no fim, e o default é
        // `Index`: todo grafo já autorado lê exactamente a onda que lia.
        ParamSpec {
            name: "order",
            default: 0.0,
        },
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // ⭐ CICLO 17 (doc 126) — apendados no fim; os defaults deixam todo grafo autorado ao bit.
        // A FORMA das famílias que a leem (`ph2d_ease::Shape`).
        ParamSpec {
            name: "overshoot",
            default: ph2d_ease::Shape::DEFAULT.overshoot,
        },
        ParamSpec {
            name: "period",
            default: ph2d_ease::Shape::DEFAULT.period,
        },
        ParamSpec {
            name: "steps",
            default: ph2d_ease::Shape::DEFAULT.steps,
        },
        ParamSpec {
            name: "damping",
            default: ph2d_ease::Shape::DEFAULT.damping,
        },
        ParamSpec {
            name: "bez_x1",
            default: ph2d_ease::Shape::DEFAULT.bezier[0],
        },
        ParamSpec {
            name: "bez_y1",
            default: ph2d_ease::Shape::DEFAULT.bezier[1],
        },
        ParamSpec {
            name: "bez_x2",
            default: ph2d_ease::Shape::DEFAULT.bezier[2],
        },
        ParamSpec {
            name: "bez_y2",
            default: ph2d_ease::Shape::DEFAULT.bezier[3],
        },
        // A ORDEM no ESPAÇO (`Grid` · `From Point`).
        ParamSpec {
            name: "grid_cols",
            default: 10.0,
        },
        ParamSpec {
            name: "grid_from",
            default: 1.0,
        },
        ParamSpec {
            name: "grid_cell",
            default: 0.0,
        },
        ParamSpec {
            name: "axis",
            default: 0.0,
        },
        ParamSpec {
            name: "origin_x",
            default: 0.0,
        },
        ParamSpec {
            name: "origin_y",
            default: 0.0,
        },
        ParamSpec {
            name: "reach",
            default: 5.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// A forma das famílias (`Back` · `Elastic` · `Steps` · `Spring` · `Bezier`), lida dos params.
fn shape_of(ctx: &EvalCtx<'_>) -> ph2d_ease::Shape {
    ph2d_ease::Shape {
        overshoot: ctx.param("overshoot"),
        period: ctx.param("period"),
        steps: ctx.param("steps"),
        damping: ctx.param("damping"),
        bezier: [
            ctx.param("bez_x1"),
            ctx.param("bez_y1"),
            ctx.param("bez_x2"),
            ctx.param("bez_y2"),
        ],
    }
}

/// A ordem inteira, lida dos params — os que escolhem ramo arredondados como o WGSL os arredonda.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "índices de enum e contagens"
)]
fn order_of(ctx: &EvalCtx<'_>) -> order::Order {
    order::Order {
        kind: ctx.param("order").round() as i32,
        seed: ctx.param("seed").max(0.0).round() as u32,
        cols: ctx.param("grid_cols").round().max(1.0) as u32,
        from: ctx.param("grid_from").round() as i32,
        cell: ctx.param("grid_cell").round().max(0.0) as u32,
        axis: ctx.param("axis").round() as i32,
        origin: [ctx.param("origin_x"), ctx.param("origin_y")],
        reach: ctx.param("reach"),
    }
}

/// The per-instance stagger delta before it's added to the channel: the eased
/// ramp `min→max` at position `raw ∈ [0,1]`, scaled by `falloff`. Easing is the
/// `curve` family shaped by `dir` (In / Out / In-Out) — see [`ease`].
#[expect(
    clippy::too_many_arguments,
    reason = "a conta é uma linha; os números são do nó"
)]
fn stagger_delta(
    min: f32,
    max: f32,
    curve: i32,
    dir: i32,
    raw: f32,
    falloff: f32,
    custom: Option<&Curve>,
    shape: &ph2d_ease::Shape,
) -> f32 {
    (min + ease(curve, dir, raw, custom, shape) * (max - min)) * falloff
}

struct MotionStagger;

impl NodeOp for MotionStagger {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let min = ctx.param("min");
        let max = ctx.param("max");
        let curve = ctx.param("ease_curve").round() as i32;
        let dir = ctx.param("ease_dir").round() as i32;
        let reverse = ctx.param("reverse") >= 0.5;
        let offset = ctx.param("offset");
        // ⭐ A ORDEM em que a onda corre pela fila (ciclo 2 · ciclo 17 — ver [`order`]) e a
        // FORMA da família (ciclo 17).
        let ordem = order_of(ctx);
        let shape = shape_of(ctx);
        // A forma DESENHADA (`ease_curve = Custom`). Parseada UMA vez, fora do laço: ela
        // é uniforme na fileira inteira, e um parse por elemento seria uma string por
        // instância.
        let custom = ctx.text_param(CURVE_KEY).and_then(ph2d_curve::parse);
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Só o `From Point` lê a posição; as outras ordens nem a pedem.
            let posicoes: &[[f32; 2]] = match input.get("P") {
                Some(Column::Vec2(v)) if ordem.kind == order::ORDER_FROM_POINT => v,
                _ => &[],
            };
            let deltas: Vec<f32> = (0..n)
                .map(|i| {
                    // A posição na onda, `0..1` — pela porta única, a mesma que o WGSL usa.
                    let p = posicoes.get(i).copied().unwrap_or([0.0, 0.0]);
                    #[expect(clippy::cast_possible_truncation, reason = "a contagem cabe num u32")]
                    let raw = order::raw_at(&ordem, i as u32, n as u32, p);
                    let raw = if reverse { 1.0 - raw } else { raw };
                    // ⚠️ **O `frac` só corre com o knob armado, e a razão é a
                    // PONTA da rampa:** o último elemento senta exactamente em
                    // `1.0`, e `frac(1.0)` é `0.0` — aplicado sempre, o neutro
                    // mandaria a última peça para o começo da rampa.
                    let raw = if offset != 0.0 {
                        let s = raw + offset;
                        s - s.floor()
                    } else {
                        raw
                    };
                    stagger_delta(
                        min,
                        max,
                        curve,
                        dir,
                        raw,
                        falloff_at(input, i),
                        custom.as_ref(),
                        &shape,
                    )
                })
                .collect();
            apply_channel_delta(input, channel, &deltas)
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionStagger))?;
    reg.register_gpu_kernel(MANIFEST.id, kernel::GPU_KERNEL);
    // A1-gpu: a tabela da ease `Custom`, para o `sg_ease` a ler no device.
    reg.register_luts(MANIFEST.id, kernel::LUTS);
    // M1.R1 — UI metadata. Behaviours modify transform channels → Transform
    // (blue) for now; a dedicated Behaviour category (cyan) is a follow-up.
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_key: "node.motion.stagger.name",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_channel_range(MANIFEST.id, PARAM_CHANNEL_RANGE);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

mod ui;
#[cfg(test)]
use ph2d_node_registry::ParamWidget;
use ui::{PARAM_CHANNEL_RANGE, PARAM_GATES, PARAM_HINTS, PARAM_UNITS};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "offset_tests.rs"]
mod offset_tests;

#[cfg(test)]
#[path = "custom_ease_tests.rs"]
mod custom_ease_tests;
