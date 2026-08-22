//! `pulse.compare` — a continuous **value** field → a discrete PULSE, with
//! Schmitt hysteresis (Motion Nodes M2, the value domain — doc 12/13/14). This
//! is the value domain's bridge BACK to the pulse domain: the genuine dual of
//! `pulse.sample_hold` (which turns `value + pulse → value`). With it a value
//! graph can feed the pulse graph, closing the continuous↔discrete round-trip —
//! a smooth `value.lfo`/`value.math` field crossing a threshold becomes a clock.
//!
//! **Not a duplicate of `pulse.threshold`.** That node reads a *transform
//! channel* of the instance stream (`INST_VEC2` — X/Y/Rot/Size, the input of the
//! "clock hack" doc 09 killed); this reads the *value* field on the `v` column
//! (doc 12). Same Schmitt core, different input domain — the reason both exist.
//!
//! **Schmitt hysteresis (the whole point):** a single threshold fires on every
//! wiggle across it — noise alone produces a burst of spurious pulses. Two
//! thresholds — `rise` > `fall` — give a bistable memory: once armed, the signal
//! must fall below the separate `fall` level before it can re-arm (Wikipedia
//! Schmitt trigger; the names mirror TouchDesigner `threshup`/`threshdown` and
//! Pd `threshold~` trigger/rest). That latched `armed` state is a per-instance
//! recurrence over the tick, so it rides the `pre` self-loop on the `state` port,
//! exactly like `pulse.threshold`/`pulse.counter`. `Effect::Pure` — the tick
//! enters the fingerprint through the consumed `pre` edge.
//!
//! **Direction** (`edge`): Rise (arm crossing, the default), Fall (disarm
//! crossing), or Both — Max `edge~`'s two outlets, the TD "Trigger On" selector.
//!
//! **Unary over the field:** the pulse it emits mirrors the value field's length
//! `N` (each instance compares its own value), so a length-N `value.math` field
//! fires a length-N pulse — the downstream `motion.strobe`/`pulse.counter` can
//! ratchet each instance on its own crossing. Transcendental-free (HR-5):
//! comparisons only.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type this node reads — the continuous per-instance scalar field on
/// the `v` column (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this
/// crate stays a leaf drop-crate — the shared vocabulary is the port, not a
/// shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// The pulse type it emits — a discrete per-instance event. `Event` clock: it
/// will not connect to a `Frame` port by a plain edge (the membrane), so
/// downstream can only be a pulse consumer.
pub const PULSE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Event);

/// The value stream's column (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The canonical column of a pulse stream: `1.0` on the tick it fires.
const PULSE_COL: &str = "pulse";
/// The latched bistable state carried on the `pre` self-loop (`1.0` = armed). A
/// sibling column of the pulse stream, distinct from `pulse.threshold`'s `armed`.
const ARMED_COL: &str = "cmp_armed";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("pulse.compare"),
    name: "pulse.compare",
    inputs: &[
        PortSpec {
            name: "value",
            ty: VALUE,
        },
        // Feedback: last tick's pulse output carries the latched `cmp_armed`
        // state. Named `state` so the editor plumbs its `pre` self-loop on drop.
        PortSpec {
            name: "state",
            ty: PULSE,
        },
        // ⚠️ **Apendada** (índice 2, no FIM): a REFERÊNCIA por-elemento. Toda
        // aresta já autorada aponta para 0 ou 1 e continua a apontar para o mesmo
        // sítio; desligada, o nó lê os params `rise`/`fall` como sempre leu.
        //
        // ⚠️ **Ela existe porque a rota óbvia COLAPSA em silêncio.** Um `value.*`
        // ligado ao param `rise` por fio (doc 58) passa pelo `driven_value`, que é
        // `xs.first()` — a linha **0** aplicada a toda a gente. Contra um sinal
        // uniforme isso é a resposta certa; contra um CAMPO é um bug que não dá
        // erro, não dá aviso, e desenha um limiar plausível.
        PortSpec {
            name: "reference",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: PULSE,
    }],
    // Pure: the tick enters the fingerprint through the consumed `pre` edge.
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "rise",
            default: 0.5,
        },
        // Below `rise` by default → a real hysteresis band. `fall > rise` is
        // clamped to `rise` at eval (a band cannot be inverted).
        ParamSpec {
            name: "fall",
            default: 0.3,
        },
        // 0 Rise · 1 Fall · 2 Both — which arm transition fires the pulse.
        ParamSpec {
            name: "edge",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Direction selector for [`step_one`].
#[derive(Copy, Clone, PartialEq, Eq)]
enum EdgeDir {
    Rise,
    Fall,
    Both,
}

impl EdgeDir {
    fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => EdgeDir::Fall,
            2 => EdgeDir::Both,
            _ => EdgeDir::Rise,
        }
    }
    fn fires(self, rose: bool, fell: bool) -> bool {
        match self {
            EdgeDir::Rise => rose,
            EdgeDir::Fall => fell,
            EdgeDir::Both => rose || fell,
        }
    }
}

/// One tick of the Schmitt trigger, per instance. Returns the `(pulse, armed)`
/// pair for row `i`, given the value and last tick's armed state. Identical core
/// to `pulse.threshold::step_one` — the only difference is the input domain
/// (a value field vs a transform channel).
fn step_one(v: f32, prev_armed: bool, rise: f32, fall: f32, edge: EdgeDir) -> (f32, f32) {
    // Hysteresis: once armed, only `fall` disarms; once disarmed, only `rise`
    // arms. `fall` is clamped ≤ `rise` so the band can never invert.
    let fall = fall.min(rise);
    let armed_now = if prev_armed { v > fall } else { v >= rise };
    let rose = armed_now && !prev_armed;
    let fell = !armed_now && prev_armed;
    let pulse = if edge.fires(rose, fell) { 1.0 } else { 0.0 };
    (pulse, if armed_now { 1.0 } else { 0.0 })
}

fn scalar_col(s: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

/// **A referência DESTA linha** — o campo da porta se ela estiver ligada, senão o
/// param.
///
/// ⚠️ **A referência move o PAR, não só uma das bordas.** O artista autorou uma
/// LARGURA de histerese (`rise − fall`), e ela é a propriedade do gatilho: deslizar
/// só o `rise` estreitaria ou inverteria a banda à medida que a referência se move,
/// e o nó passaria a chocalhar exactamente onde a histerese existe para o impedir.
fn thresholds_at(refs: &[f32], i: usize, rise: f32, fall: f32) -> (f32, f32) {
    match refs.get(i) {
        Some(r) => (*r, r - (rise - fall)),
        None => (rise, fall),
    }
}

fn step(
    value: &Stream,
    state: &Stream,
    reference: &Stream,
    rise: f32,
    fall: f32,
    edge: EdgeDir,
) -> Stream {
    let n = value.count();
    let vals = scalar_col(value, VALUE_COL, n);
    let prev_armed = scalar_col(state, ARMED_COL, n);
    // ⚠️ Porta desligada ⇒ stream VAZIO ⇒ `refs` vazio ⇒ `thresholds_at` devolve os
    // params. Um `scalar_col` aqui daria zeros de comprimento `n`, que é um limiar
    // em zero — o neutro errado, e silencioso.
    let refs: Vec<f32> = match reference.get(VALUE_COL) {
        Some(Column::Scalar(v)) if !v.is_empty() => {
            let mut v = v.clone();
            // A regra `1→N` da casa: um campo de comprimento 1 é SEGURADO.
            if v.len() == 1 {
                v.resize(n, v[0]);
            } else {
                v.resize(n, 0.0);
            }
            v
        }
        _ => Vec::new(),
    };
    let mut pulses = Vec::with_capacity(n);
    let mut armed = Vec::with_capacity(n);
    for i in 0..n {
        let was = prev_armed[i] > 0.5;
        let (r, f) = thresholds_at(&refs, i, rise, fall);
        let (p, a) = step_one(vals[i], was, r, f, edge);
        pulses.push(p);
        armed.push(a);
    }
    Stream::new(n)
        .with(PULSE_COL, Column::Scalar(pulses))
        .with(ARMED_COL, Column::Scalar(armed))
}

struct PulseCompare;

impl NodeOp for PulseCompare {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let rise = ctx.param("rise");
        let fall = ctx.param("fall");
        let edge = EdgeDir::from_param(ctx.param("edge"));
        let out = step(ctx.input(0), ctx.input(1), ctx.input(2), rise, fall, edge);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(PulseCompare))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Compare",
            // Utility grey: a value→pulse adapter, not a visible transform.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "rise",
        max: 10.0,
    },
    ParamHardMax {
        param: "fall",
        max: 10.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rise",
        label: "Rise",
        min: -10.0,
        max: 5.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "fall",
        label: "Fall",
        min: -10.0,
        max: 3.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "edge",
        label: "Edge",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Rise", "Fall", "Both"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    fn value(v: f32) -> Stream {
        Stream::new(1).with(VALUE_COL, Column::Scalar(vec![v]))
    }
    fn fired(s: &Stream) -> f32 {
        match s.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => v[0],
            _ => panic!(),
        }
    }

    /// Feed a value ramp up then down through the Schmitt trigger, carrying the
    /// output back as `state` (what the `pre` self-loop does). It fires ONCE on
    /// the way up (crossing `rise`) and, with hysteresis, only re-arms after the
    /// value drops below the separate, lower `fall`.
    #[test]
    fn it_fires_once_on_the_rising_edge_and_holds_through_the_hysteresis_band() {
        let (rise, fall) = (0.6, 0.3);
        // Climbs past `rise`, dips INTO the band (0.3..0.6) without falling below
        // `fall`, climbs again, then finally drops below `fall`.
        let signal = [0.0, 0.5, 0.7, 0.9, 0.4, 0.8, 0.2, 0.7];
        let mut state = Stream::new(1);
        let mut out = Vec::new();
        for &v in &signal {
            state = step(
                &value(v),
                &state,
                &Stream::new(0),
                rise,
                fall,
                EdgeDir::Rise,
            );
            out.push(fired(&state));
        }
        // Rises at index 2 (0.5→0.7 crosses 0.6). Index 4 (0.4) stays in the band
        // → still armed → NO refire at index 5. Index 6 (0.2 < fall) disarms →
        // index 7 (0.7) re-fires. Exactly two pulses; the band swallowed the dip.
        assert_eq!(out, vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    }

    /// FALSIFICATION of the hysteresis: with a SINGLE threshold (fall == rise) the
    /// same in-band dip re-arms and the signal chatters — the extra pulse the
    /// two-threshold design exists to suppress.
    #[test]
    fn a_single_threshold_chatters_where_the_schmitt_stays_quiet() {
        let signal = [0.0, 0.7, 0.4, 0.8]; // dip to 0.4 is below a single 0.6...
        let run = |rise: f32, fall: f32| {
            let mut state = Stream::new(1);
            let mut count = 0;
            for &v in &signal {
                state = step(
                    &value(v),
                    &state,
                    &Stream::new(0),
                    rise,
                    fall,
                    EdgeDir::Rise,
                );
                count += (fired(&state) > 0.5) as i32;
            }
            count
        };
        assert_eq!(run(0.6, 0.6), 2, "single threshold chatters");
        assert_eq!(run(0.6, 0.3), 1, "hysteresis suppresses the chatter");
    }

    /// A sustained-high value fires exactly ONE pulse (the Rive "true for a short
    /// time"), not a train — the rising edge, not the level.
    #[test]
    fn a_sustained_high_value_fires_exactly_one_pulse() {
        let mut state = Stream::new(1);
        let mut total = 0.0;
        for _ in 0..10 {
            state = step(
                &value(1.0),
                &state,
                &Stream::new(0),
                0.5,
                0.3,
                EdgeDir::Rise,
            );
            total += fired(&state);
        }
        assert_eq!(total, 1.0, "one edge, not one-per-tick-held-high");
    }

    /// `Fall` fires on the disarm crossing; `Both` fires on arm AND disarm; `Rise`
    /// on arm only — Max `edge~`'s two outlets as one selector.
    #[test]
    fn the_edge_selector_picks_the_arm_the_disarm_or_both() {
        let signal = [0.0, 0.8, 0.1]; // arm at 0.8, disarm at 0.1.
        let run = |edge: EdgeDir| {
            let mut state = Stream::new(1);
            let mut out = Vec::new();
            for &v in &signal {
                state = step(&value(v), &state, &Stream::new(0), 0.5, 0.3, edge);
                out.push(fired(&state));
            }
            out
        };
        assert_eq!(run(EdgeDir::Rise), vec![0.0, 1.0, 0.0], "arm only");
        assert_eq!(run(EdgeDir::Fall), vec![0.0, 0.0, 1.0], "disarm only");
        assert_eq!(run(EdgeDir::Both), vec![0.0, 1.0, 1.0], "both crossings");
    }

    /// An inverted band (`fall > rise`) is clamped, never honoured — it
    /// degenerates to a single threshold at `rise`, never a negative-width band.
    #[test]
    fn an_inverted_band_is_clamped_not_honoured() {
        let (p, a) = step_one(0.7, false, 0.5, 0.9, EdgeDir::Rise);
        assert_eq!(
            (p, a),
            (1.0, 1.0),
            "still arms at rise; the bad fall is ignored"
        );
    }

    /// Unary over the FIELD: a length-N value field fires a length-N pulse, each
    /// instance on its own crossing. Two dots either side of the threshold →
    /// only the one above fires. A broadcast collapse would fire all or none.
    #[test]
    fn it_compares_each_instance_of_the_field_independently() {
        let two = Stream::new(2).with(VALUE_COL, Column::Scalar(vec![0.9, 0.1]));
        let s = step(
            &two,
            &Stream::new(2),
            &Stream::new(0),
            0.5,
            0.3,
            EdgeDir::Rise,
        );
        match s.get(PULSE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![1.0, 0.0], "only the dot above rise fires"),
            _ => panic!(),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    /// **A PORTA DESLIGADA É O NÓ QUE SEMPRE SHIPOU — bit-a-bit.**
    #[test]
    fn an_unwired_reference_is_the_node_that_shipped() {
        let empty = Stream::new(0);
        for v in [0.0f32, 0.2, 0.5, 0.9] {
            let a = step(&value(v), &Stream::new(1), &empty, 0.5, 0.3, EdgeDir::Rise);
            let b = step(&value(v), &Stream::new(1), &empty, 0.5, 0.3, EdgeDir::Rise);
            assert_eq!(fired(&a), fired(&b), "v={v}");
        }
        // E o limiar é o do PARAM: 0,49 não arma, 0,5 arma.
        let below = step(
            &value(0.49),
            &Stream::new(1),
            &empty,
            0.5,
            0.3,
            EdgeDir::Rise,
        );
        let at = step(
            &value(0.5),
            &Stream::new(1),
            &empty,
            0.5,
            0.3,
            EdgeDir::Rise,
        );
        assert_eq!(fired(&below), 0.0);
        assert_eq!(fired(&at), 1.0);
    }

    /// **A REFERÊNCIA É POR-ELEMENTO — e é isso que o fio no PARAM não sabe fazer.**
    ///
    /// ⚠️ **A metade que importa é o CONTROLE.** Um `value.*` ligado ao param `rise`
    /// por fio passa pelo `driven_value`, que é `xs.first()`: a linha **0** aplicada
    /// a toda a gente. Contra um sinal uniforme isso é a resposta certa; contra um
    /// campo é um limiar plausível e errado, sem erro e sem aviso. A fixture aqui
    /// dá a TODAS as linhas o mesmo valor e a cada uma uma referência diferente —
    /// uma implementação que lesse a linha 0 armaria as três juntas.
    #[test]
    fn the_reference_is_per_element_which_a_driven_param_cannot_be() {
        let vals = Stream::new(3).with(VALUE_COL, Column::Scalar(vec![0.5, 0.5, 0.5]));
        // Referências 0,2 / 0,5 / 0,8 ⇒ armam as duas primeiras e não a terceira.
        let refs = Stream::new(3).with(VALUE_COL, Column::Scalar(vec![0.2, 0.5, 0.8]));
        let out = step(&vals, &Stream::new(3), &refs, 0.5, 0.3, EdgeDir::Rise);
        let Some(Column::Scalar(p)) = out.get(PULSE_COL) else {
            panic!("pulse")
        };
        assert_eq!(p, &vec![1.0, 1.0, 0.0], "cada linha no seu limiar");
        // ⚠️ O CONTROLE: com a referência COLAPSADA na linha 0 (o que o fio no param
        // faz), as três dariam o mesmo — e é isso que este gate impede de voltar.
        let collapsed = Stream::new(3).with(VALUE_COL, Column::Scalar(vec![0.2; 3]));
        let flat = step(&vals, &Stream::new(3), &collapsed, 0.5, 0.3, EdgeDir::Rise);
        let Some(Column::Scalar(q)) = flat.get(PULSE_COL) else {
            panic!("pulse")
        };
        assert_eq!(q, &vec![1.0, 1.0, 1.0], "colapsada, as tres armam juntas");
    }

    /// **UM CAMPO DE COMPRIMENTO 1 É SEGURADO** — a regra `1→N` da casa, e não um
    /// erro nem um truncamento.
    #[test]
    fn a_length_one_reference_is_held_across_the_field() {
        let vals = Stream::new(3).with(VALUE_COL, Column::Scalar(vec![0.1, 0.5, 0.9]));
        let one = Stream::new(1).with(VALUE_COL, Column::Scalar(vec![0.4]));
        let out = step(&vals, &Stream::new(3), &one, 0.5, 0.3, EdgeDir::Rise);
        let Some(Column::Scalar(p)) = out.get(PULSE_COL) else {
            panic!("pulse")
        };
        assert_eq!(p, &vec![0.0, 1.0, 1.0], "o limiar 0,4 vale para as tres");
    }

    /// **A REFERÊNCIA DESLIZA O PAR, NÃO SÓ UMA BORDA** — a largura da histerese que
    /// o artista autorou é uma propriedade do gatilho, e ela sobrevive ao deslize.
    ///
    /// ⚠️ Sem esta lei, uma referência a subir estreitaria a banda até a inverter, e
    /// o nó passaria a chocalhar exactamente onde a histerese existe para o impedir.
    #[test]
    fn the_reference_slides_the_pair_and_keeps_the_hysteresis_width() {
        let (rise, fall) = (0.5f32, 0.3f32);
        for r in [0.0f32, 0.4, 2.5] {
            let (rr, ff) = thresholds_at(&[r], 0, rise, fall);
            assert!(
                (rr - r).abs() < 1e-6,
                "a borda de subida vai para a referencia"
            );
            assert!(
                ((rr - ff) - (rise - fall)).abs() < 1e-6,
                "a largura mudou: {} contra {}",
                rr - ff,
                rise - fall
            );
        }
    }
}
