//! `motion.time_remap` — rewrite the clock of everything wired ABOVE this node
//! (Motion Nodes M2.N1, plan §1.5).
//!
//! Slow motion on the impact, a looping background rig, a frozen still while the
//! physics below keeps running, an explosion played backwards. The reference
//! catalogue's timeRemap: *"Modifica o `time` passado à sub-árvore UPSTREAM (não
//! a si mesmo)"* — and that is exactly what this node cannot do by itself.
//!
//! **The node is a passthrough. The remap happens in the puller.** A node only
//! ever sees its own resolved inputs (`EvalCtx`, the FBP black box of ADR-0031);
//! nothing inside `eval` can change the playhead its upstream was pulled at. So
//! the params here are *read by the domain layer* (`ph2d_eval_motion::time_scopes`)
//! into a [`ph2d_nodegraph::time::TimeMap`], and the cook applies it while
//! descending into this node's inputs ([`ph2d_nodegraph::cook::Cook::cook_scoped`]).
//! `eval` then just forwards the (already correctly-timed) stream.
//!
//! Consequences the artist can see:
//! - **Freeze is free**: a constant `t'` means the whole subtree hits the memo
//!   every frame — one cook for the life of the still.
//! - **Downstream keeps its own clock**: physics under a frozen rig still runs.
//! - **A sequential node (spring / integrate) may not sit upstream**: its state
//!   is a recurrence over the outer tick, so a rewritten clock has no meaning.
//!   The cook refuses it (`CookError::SequentialInTimeScope`) instead of
//!   producing a plausible wrong trajectory.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamGate, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use ph2d_nodegraph::time::{TimeMap, TimeMode, identity_curve_lut};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Option order of the `mode` enum param — mirrors [`TimeMode`]'s declaration
/// order (`TimeMode::from_index`) and the panel's segmented selector labels.
/// Changing one without the others silently re-labels every saved document.
pub const MODE_LABELS: &[&str] = &["Scale", "Loop", "Ping Pong", "Freeze", "Reverse", "Curve"];

/// A chave do text param que carrega a FORMA da curva (uma string do
/// `ph2d-curve`, autorada pelo editor arrastável `ParamWidget::Curve`).
///
/// ⚠️ **NÃO é um `ParamSpec`** — o manifesto é f32-only por contrato congelado
/// (ADR-0039), e uma curva não é um número; o canal de texto do `Graph` é o
/// padrão canônico (o assento da fórmula do `motion.expression`, da tabela do
/// `value.pattern` e do próprio `value.curve`).
pub const CURVE_KEY: &str = "curve";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.time_remap"),
    name: "motion.time_remap",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Pure: this node reads no clock of its own — it is a passthrough. Its
    // output changes only when the upstream (cooked at `t'`) changes, which the
    // cook already tracks through the input revision.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "mode",
            default: 0.0, // Scale — the identity, so a dropped node changes nothing
        },
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "duration",
            default: 2.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Read this node's params into the [`TimeMap`] the cook applies to its
/// upstream. `param` is the caller's param resolver (override, else the
/// manifest default) — the domain layer owns that lookup, since the substrate
/// hands params out only inside `eval`.
///
/// Kept here, next to the `ParamSpec`s, so the param names live in exactly one
/// place: a rename that misses this function is a compile-time-silent, but the
/// node's own golden test below catches it.
#[must_use]
pub fn time_map_from<'t>(
    param: impl Fn(&str) -> f32,
    text: impl Fn(&str) -> Option<&'t str>,
) -> TimeMap {
    TimeMap {
        mode: TimeMode::from_index(param("mode")),
        scale: f64::from(param("scale")),
        offset: f64::from(param("offset")),
        duration: f64::from(param("duration")),
        curve: curve_lut(text(CURVE_KEY)),
    }
}

/// Amostra a forma autorada na tabela que o substrato transporta — a metade do
/// NÓ do canal, exactamente como o `fill` de um `LutSpec`.
///
/// ⚠️ **String ausente ou malformada é a IDENTIDADE** (a lei do `value.curve`:
/// *uma curva não-desenhada é um passthrough*), e a identidade da tabela é a
/// rampa, não zeros — zeros no modo `Curve` congelariam a sub-árvore no `offset`,
/// que é outro modo.
#[must_use]
pub fn curve_lut(text: Option<&str>) -> [f32; ph2d_nodegraph::time::TIME_CURVE_SAMPLES] {
    let Some(curve) = text.and_then(ph2d_curve::parse) else {
        return identity_curve_lut();
    };
    let n = ph2d_nodegraph::time::TIME_CURVE_SAMPLES;
    let last = (n - 1) as f32;
    let mut out = [0.0f32; ph2d_nodegraph::time::TIME_CURVE_SAMPLES];
    for (k, o) in out.iter_mut().enumerate() {
        *o = curve.eval(k as f32 / last);
    }
    out
}

/// Collect the time scopes a motion graph declares: one [`TimeMap`] per
/// `motion.time_remap` node, ready for
/// [`ph2d_nodegraph::cook::Cook::cook_scoped`].
///
/// The substrate deliberately knows no node types (scopes are keyed by
/// `NodeId`), so *someone* must translate "this node type means remap" into a
/// map. That someone is this crate — the one that owns both the type name and
/// the param names. Identity maps are skipped, so a freshly-dropped node adds
/// no scope lane at all.
///
/// Cheap enough to rebuild every frame: one pass over the node list, and the
/// map is empty for the overwhelmingly common graph with no remapper.
#[must_use]
pub fn time_scopes(
    graph: &ph2d_nodegraph::graph::Graph,
    ops: &dyn ph2d_nodegraph::cook::OpResolver,
) -> ph2d_nodegraph::cook::TimeScopes {
    let mut scopes = ph2d_nodegraph::cook::TimeScopes::new();
    for inst in graph.nodes() {
        if inst.type_name != MANIFEST.name {
            continue;
        }
        let Some(manifest) = ops.resolve(inst.type_id()).map(NodeOp::manifest) else {
            continue;
        };
        let overrides = graph.node_param_overrides(inst.id);
        let texts = graph.node_text_param_overrides(inst.id);
        let map = time_map_from(
            |name| {
                overrides
                    .and_then(|o| o.get(name).copied())
                    .or_else(|| manifest.param_default(name))
                    .unwrap_or(0.0)
            },
            |name| texts.and_then(|t| t.get(name)).map(String::as_str),
        );
        if !map.is_identity() {
            scopes.insert(inst.id, map);
        }
    }
    scopes
}

/// Does `remapper`'s upstream subtree contain a **sequential** node (one fed by
/// a `pre` edge — a spring, an integrate)? Such a node integrates a recurrence
/// over the outer tick and cannot run on a rewritten clock, so the cook refuses
/// it ([`ph2d_nodegraph::cook::CookError::SequentialInTimeScope`]).
///
/// The editor calls this **before committing a wire**, to refuse the edit with
/// an explanation instead of letting the scene silently stop cooking.
#[must_use]
pub fn scopes_a_sequential_node(
    graph: &ph2d_nodegraph::graph::Graph,
    remapper: ph2d_nodegraph::graph::NodeId,
) -> bool {
    let is_sequential = |n| graph.edges().iter().any(|e| e.delayed && e.to.0 == n);
    let mut stack = vec![remapper];
    let mut seen = std::collections::BTreeSet::new();
    while let Some(node) = stack.pop() {
        if !seen.insert(node) {
            continue;
        }
        if node != remapper && is_sequential(node) {
            return true;
        }
        // Walk the FORWARD upstream only: a `pre` edge is not a cook-time
        // dependency (it reads last tick's snapshot), so it never carries the
        // remapped clock into its source.
        for e in graph.edges() {
            if !e.delayed && e.to.0 == node {
                stack.push(e.from.0);
            }
        }
    }
    false
}

struct MotionTimeRemap;

impl NodeOp for MotionTimeRemap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // The stream already arrived sampled at `t'` (the cook remapped the
        // clock on the way up). Nothing left to do but pass it along.
        let passthrough = ctx.input(0).clone();
        ctx.emit(passthrough);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTimeRemap))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Time Remap",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// A linha da CURVA pertence ao modo que a lê, e a mais nenhum — o precedente do
/// `column` do `motion.drive`. Um editor de curva pintado sob `Loop` seria um
/// controle que não move um quadro.
static PARAM_GATES: &[ParamGate] = &[ParamGate {
    param: CURVE_KEY,
    when: "mode",
    values: &[5], // Curve
}];

/// Param rows: a named mode selector (never a number the artist must decode),
/// then the three scalars the modes read. Seconds, matching the playhead.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 5.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: MODE_LABELS,
        },
    },
    // A FORMA — um TEXT param (`CURVE_KEY`), o editor arrastável do A1. Não
    // desenhada = identidade, e aí o modo `Curve` é o `Loop` (a janela repete),
    // a menos do arredondamento da ida-e-volta pela tabela.
    ParamUiHint {
        param: CURVE_KEY,
        label: "Curve",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
    },
    ParamUiHint {
        param: "scale",
        label: "Speed",
        min: -4.0,
        max: 4.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "duration",
        label: "Duration",
        min: 0.1,
        max: 10.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "duration",
    unit: ParamUnit::Seconds,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::attr::{Column, Stream};

    /// The mode selector's labels and `TimeMode`'s indices are one vocabulary
    /// split across two crates: a reorder on either side silently re-labels
    /// every saved document (a "Loop" that becomes a "Ping Pong" on reload).
    #[test]
    fn the_mode_labels_match_the_time_mode_indices() {
        let modes = [
            TimeMode::Scale,
            TimeMode::Loop,
            TimeMode::PingPong,
            TimeMode::Freeze,
            TimeMode::Reverse,
            TimeMode::Curve,
        ];
        assert_eq!(MODE_LABELS.len(), modes.len());
        for (i, mode) in modes.into_iter().enumerate() {
            assert_eq!(
                TimeMode::from_index(i as f32),
                mode,
                "label {:?} decodes to the wrong mode",
                MODE_LABELS[i]
            );
        }
    }

    /// The manifest's defaults must build the IDENTITY map: dropping the node
    /// onto a live chain has to change nothing until the artist picks a mode.
    /// (A default `duration` of 0 would also be a latent divide-by-zero.)
    #[test]
    fn the_default_params_build_the_identity_map() {
        let map = time_map_from(
            |name| MANIFEST.param_default(name).expect("declared param"),
            |_| None,
        );
        assert!(map.is_identity(), "a fresh Time Remap warps time: {map:?}");
        assert_eq!(map.apply(1.25), 1.25);
        assert!(map.duration > 0.0);
    }

    /// **A FORMA autorada chega ao mapa** — a metade que só o NÓ pode responder.
    ///
    /// ⚠️ O gate da lei mora no substrato (`ph2d-nodegraph::time`) e monta o
    /// `TimeMap` **à mão**, logo é **CEGO à fiação**: se o `time_map_from`
    /// esquecer o text param, a curva do artista nunca sai do documento e os
    /// gates da lei seguem VERDES sobre um relógio que ninguém dobrou.
    #[test]
    fn the_authored_shape_reaches_the_map_the_cook_applies() {
        let params = |name: &str| match name {
            "mode" => 5.0, // Curve
            "duration" => 2.0,
            "scale" => 1.0,
            "offset" => 0.0,
            _ => panic!("undeclared param {name}"),
        };
        // Um ease de dois pontos com tangentes chatas, na serializacao do editor.
        let drawn = ph2d_curve::serialize(&ph2d_curve::Curve {
            points: vec![
                ph2d_curve::Point {
                    x: 0.0,
                    y: 0.0,
                    interp: ph2d_curve::Interp::Smooth,
                },
                ph2d_curve::Point {
                    x: 1.0,
                    y: 1.0,
                    interp: ph2d_curve::Interp::Smooth,
                },
            ],
        });
        let bent = time_map_from(params, |k| (k == CURVE_KEY).then_some(drawn.as_str()));
        let flat = time_map_from(params, |_| None);

        assert_eq!(bent.mode, TimeMode::Curve);
        // No MEIO da janela um ease pousa no meio; o que muda e' o CAMINHO ate' la'.
        // A um quarto, o ease ja' esta' visivelmente ATRAS da reta.
        let (b, f) = (bent.apply(0.5), flat.apply(0.5));
        assert!(
            (b - f).abs() > 0.05,
            "a curva autorada tem de mover o relogio: {b} contra {f}"
        );
        // E o CONTROLE: sem texto, o mapa é a rampa — e a rampa lida numa janela que
        // REPETE é o `Loop`, não o `Scale`. ⚠️ A varredura passa da janela de
        // propósito (0 .. 6 s sobre uma de 2 s): parar em `t = duration` é o que
        // fazia a suíte inteira ficar verde sobre um relógio que expirava.
        let looped = TimeMap {
            mode: TimeMode::Loop,
            scale: 1.0,
            offset: 0.0,
            duration: 2.0,
            ..TimeMap::default()
        };
        for k in 0..=60 {
            let t = k as f64 * 0.1;
            let (a, b) = (flat.apply(t), looped.apply(t));
            assert!(
                (a - b).abs() < 1e-6,
                "sem forma autorada o modo Curve e' o Loop, e em {t} da {a} contra {b}"
            );
        }
    }

    /// **A linha da curva é gateada no MODO que a lê, e o índice é o do enum.**
    ///
    /// Duas grafias do mesmo vocabulário (o valor do `ParamGate` e o
    /// `TimeMode::index()`) moram em crates diferentes; se derivarem, o editor de
    /// curva aparece sob `Reverse` e some sob `Curve`, sem ninguém reclamar.
    #[test]
    fn the_curve_row_is_gated_on_the_mode_that_reads_it() {
        let gate = PARAM_GATES
            .iter()
            .find(|g| g.param == CURVE_KEY)
            .expect("a curva e' gateada");
        assert_eq!(gate.when, "mode");
        assert_eq!(gate.values, &[i32::from(TimeMode::Curve.index())]);
    }

    /// **Uma string malformada é a identidade, nunca um relógio inventado.**
    #[test]
    fn a_malformed_curve_is_the_identity() {
        let lut = curve_lut(Some("isto nao e' uma curva"));
        assert_eq!(lut, ph2d_nodegraph::time::identity_curve_lut());
    }

    #[test]
    fn params_decode_into_the_map_the_cook_applies() {
        let map = time_map_from(
            |name| match name {
                "mode" => 1.0, // Loop
                "scale" => 2.0,
                "offset" => 0.5,
                "duration" => 3.0,
                _ => panic!("undeclared param {name}"),
            },
            |_| None,
        );
        assert_eq!(map.mode, TimeMode::Loop);
        // t=2 → scaled 4 → 4 mod 3 = 1 → +0.5.
        assert_eq!(map.apply(2.0), 1.5);
    }

    /// The node is a passthrough: it forwards its input verbatim (the cook did
    /// the remapping upstream). Anything else would double-count the warp.
    #[test]
    fn eval_forwards_the_stream_verbatim() {
        struct Reg;
        impl ph2d_nodegraph::cook::OpResolver for Reg {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionTimeRemap)
            }
        }
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::{Edge, Graph};

        // A tiny source so the remap has something to forward.
        static SRC_MAN: NodeManifest = NodeManifest {
            id: NodeTypeId::of("test.src"),
            name: "test.src",
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
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC_MAN
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[3.0, 4.0]])));
            }
        }
        struct Ops;
        impl ph2d_nodegraph::cook::OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == MANIFEST.id => Some(&MotionTimeRemap),
                    t if t == SRC_MAN.id => Some(&Src),
                    _ => None,
                }
            }
        }
        let _ = Reg;

        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let remap = g.add_node("motion.time_remap");
        g.connect(Edge {
            from: (src, 0),
            to: (remap, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, remap, 0.0).unwrap();
        match out[0].as_stream().get("P") {
            Some(Column::Vec2(p)) => assert_eq!(p, &vec![[3.0, 4.0]]),
            other => panic!("expected the input stream verbatim, got {other:?}"),
        }
    }

    /// The SAFETY refusal the editor leans on (audit 2026-07-10 flagged it as
    /// untested): a sequential node (one fed by a `pre` edge — spring,
    /// integrate) in the remapper's FORWARD upstream must be detected, so the
    /// wire is refused instead of the scene going dark at cook time
    /// (`SequentialInTimeScope`).
    #[test]
    fn detects_a_sequential_node_in_the_forward_upstream_only() {
        use ph2d_nodegraph::graph::{Edge, Graph};
        let fwd = |from, to| Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        };

        // src → spring(⟲ pre self-loop on port 1) → remap: upstream IS
        // sequential → refused.
        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let spring = g.add_node("test.spring");
        let remap = g.add_node("motion.time_remap");
        g.connect(fwd(src, spring)).unwrap();
        g.connect(Edge {
            from: (spring, 0),
            to: (spring, 1),
            delayed: true,
        })
        .unwrap();
        g.connect(fwd(spring, remap)).unwrap();
        assert!(
            scopes_a_sequential_node(&g, remap),
            "a pre-fed node upstream must be refused"
        );

        // src → remap → spring(⟲): the sequential node is DOWNSTREAM — the
        // remapper never rewrites its clock → allowed.
        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let remap = g.add_node("motion.time_remap");
        let spring = g.add_node("test.spring");
        g.connect(fwd(src, remap)).unwrap();
        g.connect(fwd(remap, spring)).unwrap();
        g.connect(Edge {
            from: (spring, 0),
            to: (spring, 1),
            delayed: true,
        })
        .unwrap();
        assert!(
            !scopes_a_sequential_node(&g, remap),
            "downstream sequential nodes are not scoped"
        );

        // spring(⟲) --pre--> remap: reachable only across a `pre` edge — not a
        // cook-time dependency, the remapped clock never crosses it → allowed.
        let mut g = Graph::new();
        let spring = g.add_node("test.spring");
        let remap = g.add_node("motion.time_remap");
        g.connect(Edge {
            from: (spring, 0),
            to: (spring, 1),
            delayed: true,
        })
        .unwrap();
        g.connect(Edge {
            from: (spring, 0),
            to: (remap, 0),
            delayed: true,
        })
        .unwrap();
        assert!(
            !scopes_a_sequential_node(&g, remap),
            "a pre edge does not carry the remapped clock upstream"
        );

        // A purely combinational upstream chain → allowed.
        let mut g = Graph::new();
        let src = g.add_node("test.src");
        let mid = g.add_node("test.mid");
        let remap = g.add_node("motion.time_remap");
        g.connect(fwd(src, mid)).unwrap();
        g.connect(fwd(mid, remap)).unwrap();
        assert!(!scopes_a_sequential_node(&g, remap));
    }
}
