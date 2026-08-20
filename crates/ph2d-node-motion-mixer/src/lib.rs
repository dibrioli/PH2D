#![forbid(unsafe_code)]
//! `motion.mixer` — **blend several instance streams element-wise**: the Houdini
//! "Attribute Interpolate" / "Sequence Blend" (Motion Nodes M1, streams — doc 01 §1.7 /
//! doc 30). Where `motion.combine` stacks streams end to end, this fuses them per
//! element: element `i` of the output is the average / sum / lerp of element `i` across
//! the inputs. **Blend** two layouts and a `value.lfo` morphs one into the other (a grid
//! into a ring); **Avg** blends up to four at once.
//!
//! **Algorithm — element-wise reduction over the common columns.** The count is the
//! **minimum** across the contributing inputs (the extra tail of a longer input is
//! dropped — the Sequence-Blend convention). Every column present in **all** contributing
//! inputs is reduced: **Avg** = mean, **Add** = sum (both over all non-empty inputs);
//! **Blend** = `lerp(in0, in1, blend)` with the `blend` value input (unconnected → 0.5).
//! Transcendental-free (HR-5): component arithmetic. `Effect::Pure`.
//!
//! ⚠️ **The `blend` is a FIELD, one weight per element** (doc 12's broadcast rule,
//! the same one `motion.drive` and `motion.morph` read): absent → the midpoint,
//! length-1 HELD across the stream, length-N per-element. It used to be
//! `v.first()` — a length-N field handed **element zero's number to everybody**, so
//! the one thing a per-element blend could express was an accident, and every
//! reference disagreed (Blender's *Mix* `Factor` is a field; C4D gives each field
//! layer its own Mask; our own `motion.morph` was already per-element).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `blend` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Mix modes (the `mode` param). Avg is [`MODE_AVG`] — o braço default da redução, e
/// nomeado desde a wave dos pesos porque o `ParamGate` deles precisa do número.
const MODE_ADD: i64 = 1;
/// Blend mode: `lerp(in0, in1, blend)`.
const MODE_BLEND: i64 = 2;
/// What an UNCONNECTED `blend` input means: the midpoint, which is the number
/// the node has always used and the one an artist reads off the word "Blend".
const DEFAULT_BLEND: f32 = 0.5;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mixer"),
    name: "motion.mixer",
    inputs: &[
        PortSpec {
            name: "in0",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in1",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in2",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in3",
            ty: INST_VEC2,
        },
        // Blend weight for the Blend mode (animatable). Optional: unconnected → 0.5.
        PortSpec {
            name: "blend",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Avg · 1 Add · 2 Blend (in0→in1 by the blend input).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // **O PESO DE CADA ENTRADA** — ver [`WEIGHTS`]. Apendados, todos `1` ⇒ literal.
        ParamSpec {
            name: "weight_0",
            default: 1.0,
        },
        ParamSpec {
            name: "weight_1",
            default: 1.0,
        },
        ParamSpec {
            name: "weight_2",
            default: 1.0,
        },
        ParamSpec {
            name: "weight_3",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **O PESO POR ENTRADA** (doc 89 folha 08 — MiniCavalry `mixer`: `wa/wb/wc/wd`; C4D dá a
/// cada camada de field a sua Strength).
///
/// A célula media o custo do que já existia: encadear `Blend` aos pares reproduz qualquer
/// combinação convexa, mas **os pesos COMPÕEM** — o peso de `c` é `w2` e o de `a` é
/// `(1−w1)(1−w2)`, então o artista que quer 0,2 / 0,3 / 0,5 tem de resolver um sistema, e
/// paga um nó por entrada.
///
/// ⚠️ **Um por PORTA, não um por contribuinte.** Este nó descarta as entradas vazias antes de
/// reduzir, então a 3ª entrada LIGADA pode ser a porta `in3`; se o peso seguisse a posição na
/// lista de contribuintes, desligar um fio **remexeria os pesos dos outros três**. O peso
/// viaja com a porta desde o `snapshot`.
///
/// ⚠️ **`Avg` normaliza, `Add` não** — e é isso que mantém os dois modos a serem o que já
/// eram: a média ponderada é `Σ wᵢ·cᵢ / Σ wᵢ` (com todos a `1`, `Σ w` é a contagem, ao bit) e
/// a soma ponderada é `Σ wᵢ·cᵢ`. Normalizar a soma seria transformá-la numa média.
///
/// ⚠️ **`Σ w = 0` não é uma divisão por zero: é a resposta ZERO.** Com todo peso a zero a
/// média ponderada é indefinida em matemática, e um `0/0` daria `NaN` — uma cena que
/// desaparece sem explicação. O que se emite é o numerador (que é zero), que se lê como
/// *"você desligou todas as entradas"* — visível e explicável.
///
/// ⚠️ **Gateados fora do `Blend`**, onde o peso já tem dono: ali a resposta a *"quanto de cada
/// um?"* é o campo `blend`, por elemento. Pintar quatro sliders ao lado dele seriam duas
/// portas para a mesma pergunta, e a segunda ganharia em silêncio.
const WEIGHTS: [&str; 4] = ["weight_0", "weight_1", "weight_2", "weight_3"];

/// Avg is mode `0` — nomeado porque o [`ParamGate`] dos pesos precisa do número.
const MODE_AVG: i64 = 0;

/// A cloned snapshot of one input.
struct Snap {
    count: usize,
    cols: Vec<(String, Column)>,
}

impl Snap {
    fn column(&self, name: &str) -> Option<&Column> {
        self.cols.iter().find(|(n, _)| n == name).map(|(_, c)| c)
    }
}

fn snapshot(s: &Stream) -> Snap {
    Snap {
        count: s.count(),
        cols: s.columns().map(|(n, c)| (n.clone(), c.clone())).collect(),
    }
}

/// A column truncated to the first `n` rows.
fn trunc(c: &Column, n: usize) -> Column {
    match c {
        Column::Scalar(v) => Column::Scalar(v[..n].to_vec()),
        Column::Vec2(v) => Column::Vec2(v[..n].to_vec()),
        Column::Vec3(v) => Column::Vec3(v[..n].to_vec()),
        Column::Vec4(v) => Column::Vec4(v[..n].to_vec()),
    }
}

/// Component-wise `a + b·k` (same variant, same length).
fn add_scaled(a: &Column, b: &Column, k: f32) -> Column {
    macro_rules! z {
        ($va:expr, $vb:expr, $w:literal) => {{
            $va.iter()
                .zip($vb.iter())
                .map(|(x, y)| {
                    let mut r = *x;
                    for c in 0..$w {
                        r[c] += y[c] * k;
                    }
                    r
                })
                .collect()
        }};
    }
    match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => {
            Column::Scalar(x.iter().zip(y).map(|(a, b)| a + b * k).collect())
        }
        (Column::Vec2(x), Column::Vec2(y)) => Column::Vec2(z!(x, y, 2)),
        (Column::Vec3(x), Column::Vec3(y)) => Column::Vec3(z!(x, y, 3)),
        (Column::Vec4(x), Column::Vec4(y)) => Column::Vec4(z!(x, y, 4)),
        _ => a.clone(),
    }
}

/// Component-wise scale.
fn scale(c: &Column, k: f32) -> Column {
    macro_rules! s {
        ($v:expr, $w:literal) => {{
            $v.iter()
                .map(|x| {
                    let mut r = *x;
                    for c in 0..$w {
                        r[c] *= k;
                    }
                    r
                })
                .collect()
        }};
    }
    match c {
        Column::Scalar(v) => Column::Scalar(v.iter().map(|x| x * k).collect()),
        Column::Vec2(v) => Column::Vec2(s!(v, 2)),
        Column::Vec3(v) => Column::Vec3(s!(v, 3)),
        Column::Vec4(v) => Column::Vec4(s!(v, 4)),
    }
}

/// The `blend` for element `i` — **the one broadcast rule** (doc 12), the same one
/// `motion.drive` and `motion.morph` already read: **unconnected (empty) → the
/// midpoint**, length-1 is HELD across every instance, length-N is per-element.
///
/// ⚠️ This is the P0 of doc 89 folha 08, and what it replaced was `v.first()` — a
/// length-N field handed **element zero's number to the whole stream**, so the
/// only thing a per-element blend could express was an accident. Blender's *Mix*
/// makes `Factor` a field (the diamond socket), C4D gives every field layer its
/// own Mask, and **our own `motion.morph` was already per-element** — the mixer
/// was the one place in the family where the answer collapsed to one scalar.
///
/// ⚠️ **Not clamped, deliberately.** `motion.morph` clamps to `[0, 1]` and this
/// does not, and the two are right for different reasons: morph interpolates a
/// SHAPE toward another and promises `1` is `b`, while a mixer lerp past `1` is an
/// overshoot **that has a picture** — a layout thrown past the target one, which is
/// a thing an artist asks for. Clamping here would be a silent behaviour change on
/// top of the fix, so the range stays exactly what it was.
fn blend_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => DEFAULT_BLEND,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(DEFAULT_BLEND),
    }
}

/// `a·(1−t) + b·t` per lane, with `t` read per element.
///
/// ⚠️ The two-term form is not stylistic: at `t = 1` the first term is `a·0.0`,
/// which IEEE-754 makes exactly zero for any finite `a`, and the second is `b·1.0`
/// — so `blend = 1` is `in1` **to the bit**, which is what the node's own doc
/// promises. `a + (b − a)·t` lands *near* `b` and is not the same number.
fn lerp_col(a: &Column, b: &Column, blend: &[f32], n: usize) -> Column {
    macro_rules! z {
        ($va:expr, $vb:expr, $w:literal, $ctor:path) => {{
            $ctor(
                (0..n)
                    .map(|i| {
                        let t = blend_at(blend, i);
                        let (x, y) = ($va[i], $vb[i]);
                        let mut r = x;
                        for c in 0..$w {
                            r[c] = x[c] * (1.0 - t) + y[c] * t;
                        }
                        r
                    })
                    .collect(),
            )
        }};
    }
    match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => Column::Scalar(
            (0..n)
                .map(|i| {
                    let t = blend_at(blend, i);
                    x[i] * (1.0 - t) + y[i] * t
                })
                .collect(),
        ),
        (Column::Vec2(x), Column::Vec2(y)) => z!(x, y, 2, Column::Vec2),
        (Column::Vec3(x), Column::Vec3(y)) => z!(x, y, 3, Column::Vec3),
        (Column::Vec4(x), Column::Vec4(y)) => z!(x, y, 4, Column::Vec4),
        // Variants disagree: summing a Vec2 into a Vec4 means nothing, so the
        // first input wins — the same arm `add_scaled` already takes.
        _ => a.clone(),
    }
}

/// Column names present in every contributing snapshot, in the first input's order.
fn common_columns(snaps: &[&Snap]) -> Vec<String> {
    let Some(first) = snaps.first() else {
        return Vec::new();
    };
    first
        .cols
        .iter()
        .map(|(n, _)| n.clone())
        .filter(|n| snaps.iter().all(|s| s.column(n).is_some()))
        .collect()
}

/// Reduce the contributing inputs into one stream. `blend` is only used in Blend mode;
/// `weights` is aligned to `contributing` and carries **each snapshot's own port weight**
/// ([`WEIGHTS`]).
fn mix(mode: i64, contributing: &[&Snap], blend: &[f32], weights: &[f32]) -> Stream {
    if contributing.is_empty() {
        return Stream::new(0);
    }
    let count = contributing.iter().map(|s| s.count).min().unwrap_or(0);
    let mut out = Stream::new(count);
    if count == 0 {
        return out;
    }
    let total_w: f32 = weights.iter().sum();
    for name in common_columns(contributing) {
        let cols: Vec<Column> = contributing
            .iter()
            .map(|s| trunc(s.column(&name).unwrap(), count))
            .collect();
        // A soma PONDERADA, que os dois modos de redução partilham. ⚠️ Com todos os pesos a
        // `1` ela é a soma de antes **ao bit**: `x·1.0` é `x` em IEEE-754 para todo `x`
        // finito, e o primeiro termo passa pelo mesmo `scale` que os outros.
        let weighted_sum = || {
            cols.iter()
                .zip(weights)
                .skip(1)
                .fold(scale(&cols[0], weights[0]), |acc, (c, w)| {
                    add_scaled(&acc, c, *w)
                })
        };
        let mixed = match mode {
            MODE_BLEND if cols.len() >= 2 => lerp_col(&cols[0], &cols[1], blend, count),
            MODE_ADD => weighted_sum(),
            _ => {
                // Avg (and Blend with a single input): the WEIGHTED mean over the inputs.
                // ⚠️ `Σ w = 0` emite o numerador (zero) em vez de `0/0 = NaN` — ver
                // [`WEIGHTS`].
                let sum = weighted_sum();
                if total_w == 0.0 {
                    sum
                } else {
                    scale(&sum, 1.0 / total_w)
                }
            }
        };
        out.set(name, mixed);
    }
    out
}

struct MotionMixer;

impl NodeOp for MotionMixer {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i64;
        // ⚠️ The WHOLE column, not `v.first()`: the field is a per-element answer
        // and reading one row of it was the P0 of doc 89 folha 08.
        let blend: Vec<f32> = match ctx.input(4).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        // ⚠️ O peso é lido junto com a porta e viaja com ela pelo filtro — ver [`WEIGHTS`]:
        // uma entrada vazia sai da lista, e um peso indexado pela POSIÇÃO na lista passaria
        // a valer para outra porta.
        let ws: Vec<f32> = WEIGHTS.iter().map(|w| ctx.param(w)).collect();
        // Snapshot the four stream inputs, one at a time.
        let snaps: Vec<(Snap, f32)> = (0..4u16)
            .map(|k| (snapshot(ctx.input(k as usize)), ws[k as usize]))
            .filter(|(s, _)| s.count > 0)
            .collect();
        // Blend uses only the first two inputs; Avg/Add use all non-empty.
        let taken = if mode == MODE_BLEND { 2 } else { snaps.len() };
        let contributing: Vec<&Snap> = snaps.iter().take(taken).map(|(s, _)| s).collect();
        let weights: Vec<f32> = snaps.iter().take(taken).map(|(_, w)| *w).collect();
        ctx.emit(mix(mode, &contributing, &blend, &weights));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMixer))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Mixer",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

/// ⚠️ **A faixa é `0..2` e o `1` fica no MEIO** — o literal é o centro do curso, e a metade de
/// cima é o que separa uma média ponderada de uma média: dar peso `2` a uma entrada é dizer
/// *"esta conta o dobro"*, e sem isso o knob só saberia apagar.
macro_rules! weight_hint {
    ($p:expr, $l:expr) => {
        ParamUiHint {
            param: $p,
            label: $l,
            min: 0.0,
            max: 2.0,
            step: 0.01,
            widget: ParamWidget::Slider,
        }
    };
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Avg", "Add", "Blend"],
        },
    },
    weight_hint!(WEIGHTS[0], "Weight 0"),
    weight_hint!(WEIGHTS[1], "Weight 1"),
    weight_hint!(WEIGHTS[2], "Weight 2"),
    weight_hint!(WEIGHTS[3], "Weight 3"),
];

/// Os pesos só aparecem onde são lidos — ver [`WEIGHTS`]: no `Blend` quem responde
/// *"quanto de cada um?"* é o campo `blend`, e um segundo controle para a mesma pergunta
/// ganharia em silêncio.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: WEIGHTS[0],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
    ParamGate {
        param: WEIGHTS[1],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
    ParamGate {
        param: WEIGHTS[2],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
    ParamGate {
        param: WEIGHTS[3],
        when: "mode",
        values: &[MODE_AVG as i32, MODE_ADD as i32],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Avg mode (the production default arm), named here for the tests' readability.
    const MODE_AVG: i64 = 0;

    fn snap_p(p: Vec<[f32; 2]>) -> Snap {
        Snap {
            count: p.len(),
            cols: vec![("P".to_string(), Column::Vec2(p))],
        }
    }

    fn p_of(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    /// Avg is the midpoint of the inputs: two points averaged land halfway between.
    #[test]
    fn avg_is_the_midpoint() {
        let a = snap_p(vec![[0.0, 0.0], [2.0, 0.0]]);
        let b = snap_p(vec![[4.0, 0.0], [2.0, 4.0]]);
        let out = mix(MODE_AVG, &[&a, &b], &[0.5], &[1.0, 1.0]);
        assert_eq!(p_of(&out), vec![[2.0, 0.0], [2.0, 2.0]]);
    }

    /// Add sums the inputs component-wise.
    #[test]
    fn add_sums_the_inputs() {
        let a = snap_p(vec![[1.0, 1.0]]);
        let b = snap_p(vec![[2.0, 3.0]]);
        let out = mix(MODE_ADD, &[&a, &b], &[0.5], &[1.0, 1.0]);
        assert_eq!(p_of(&out), vec![[3.0, 4.0]]);
    }

    /// Blend lerps in0→in1: weight 0 is in0, 1 is in1, 0.25 is a quarter across.
    /// FALSIFIED by an averaging that ignores the weight.
    #[test]
    fn blend_lerps_in0_to_in1() {
        let a = snap_p(vec![[0.0, 0.0]]);
        let b = snap_p(vec![[4.0, 8.0]]);
        assert_eq!(
            p_of(&mix(MODE_BLEND, &[&a, &b], &[0.0], &[1.0, 1.0])),
            vec![[0.0, 0.0]]
        );
        assert_eq!(
            p_of(&mix(MODE_BLEND, &[&a, &b], &[1.0], &[1.0, 1.0])),
            vec![[4.0, 8.0]]
        );
        assert_eq!(
            p_of(&mix(MODE_BLEND, &[&a, &b], &[0.25], &[1.0, 1.0])),
            vec![[1.0, 2.0]]
        );
    }

    /// Mismatched counts blend the common prefix (the minimum count).
    #[test]
    fn count_is_the_minimum() {
        let a = snap_p(vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]);
        let b = snap_p(vec![[2.0, 2.0]]);
        let out = mix(MODE_AVG, &[&a, &b], &[0.5], &[1.0, 1.0]);
        assert_eq!(out.count(), 1, "truncated to the shorter input");
    }

    /// Deterministic + cooks through the registry: two sources blend to their midpoint at
    /// blend 0.5.
    #[test]
    fn registers_and_mixes_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        const fn src(id: &'static str) -> NodeManifest {
            NodeManifest {
                id: NodeTypeId::of(id),
                name: id,
                inputs: &[],
                outputs: &[PortSpec {
                    name: "out",
                    ty: INST_VEC2,
                }],
                effect: Effect::Pure,
                clock: Clock::Frame,
                params: &[] as &[ParamSpec],
                lowerings: &[LoweringKind::Cpu],
            }
        }
        static SA: NodeManifest = src("motion.mixer.test.a");
        static SB: NodeManifest = src("motion.mixer.test.b");
        struct A;
        impl NodeOp for A {
            fn manifest(&self) -> &'static NodeManifest {
                &SA
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
            }
        }
        struct B;
        impl NodeOp for B {
            fn manifest(&self) -> &'static NodeManifest {
                &SB
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[4.0, 0.0], [4.0, 0.0]])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SA.id => Some(&A),
                    t if t == SB.id => Some(&B),
                    t if t == MANIFEST.id => Some(&MotionMixer),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let a = g.add_node("motion.mixer.test.a");
        let b = g.add_node("motion.mixer.test.b");
        let m = g.add_node("motion.mixer");
        g.connect(Edge {
            from: (a, 0),
            to: (m, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (b, 0),
            to: (m, 1),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [2.0, 0.0], "midpoint of the two sources"),
            _ => panic!("P"),
        }
    }
}

#[cfg(test)]
#[path = "blend_field_tests.rs"]
mod blend_field_tests;

#[cfg(test)]
#[path = "weights_tests.rs"]
mod weights_tests;
