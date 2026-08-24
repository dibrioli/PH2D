#![forbid(unsafe_code)]
//! `motion.spring` — a damped spring that makes one transform channel *chase*
//! its animated upstream target with lag, overshoot and settle (follow-through).
//! The reference's warning is the usage manual: **it only acts on targets that
//! CHANGE** — wire it after an oscillator/stagger/mouse-ish driver, or it is
//! invisible.
//!
//! A **sequential** node: its state (`spring_value`, `spring_vel` per instance)
//! rides its own output as columns through the `pre` self-loop
//! (`out --pre--> state`, auto-wired on add — the input named `state` with the
//! output's type is the sequential-node convention). At tick 0 the state seeds
//! at the target (no snap); each tick it advances one fixed step.
//!
//! Algorithm (MiniCavalryV2 `spring.js`, clean-room — plan §1.7 mandates the
//! literal port): semi-implicit Euler with an **adaptive sub-step** chosen from
//! the stability limit `sub_dt² · tension < 0.05` — stiff springs stay stable
//! by taking more, smaller steps instead of exploding:
//!
//! ```text
//! accel = −friction·vel − tension·(value − target)
//! vel   += accel·sub_dt ; value += vel·sub_dt      (× steps)
//! ```
//!
//! `dt` derives from the state's own `sim_t` column (playhead at last eval),
//! clamped to `[0, MAX_DT]` — no cross-crate timestep constant, deterministic
//! (HR-5: arithmetic + IEEE sqrt only). The multiplicative `falloff` column
//! blends the OUTPUT between the raw target (0) and the full spring (1); the
//! raw state keeps evolving regardless, exactly like the reference.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream, par_build};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod channel;
mod kernel;
use channel::{channel_get, channel_set, falloff_at, ids_of, inv_mass_at};
use std::collections::BTreeMap;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Ceiling on a single integration step (see `motion.integrate`).
const MAX_DT: f32 = 0.1;
/// Stability bound for the adaptive sub-step: `sub_dt² · tension < STABLE`.
const STABLE: f32 = 0.05;
/// Hard cap on sub-steps per tick — at the UI's max tension (60) and MAX_DT the
/// adaptive count is 4, so 64 only guards absurd hand-authored overrides.
const MAX_STEPS: usize = 64;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spring"),
    name: "motion.spring",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The feedback port — auto-wired `out --pre--> state` on add.
        PortSpec {
            name: "state",
            ty: INST_VEC2,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Temporal: `eval` reads `ctx.playhead()` (it stamps `sim_t` and derives
    // `dt` from it), and only a Temporal manifest folds the playhead into the
    // memo fingerprint. The consumed `pre` edge already forces a re-cook per
    // tick, which masks this during forward playback — but a same-tick re-cook
    // at a moved playhead (checkpoint/restore scrub, M2.N2) would return a
    // stale trajectory under `Pure`. Convention: reads playhead ⇒ Temporal
    // (`motion.oscillator`, `pulse.beat`).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "channel",
            default: 1.0, // Y — the reference's default
        },
        ParamSpec {
            name: "tension",
            default: 8.0,
        },
        ParamSpec {
            name: "friction",
            default: 1.5,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct MotionSpring;

impl NodeOp for MotionSpring {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let channel = ctx.param("channel").round() as i32;
        let tension = ctx.param("tension").max(0.1);
        let friction = ctx.param("friction").max(0.05);
        let playhead = ctx.playhead() as f32;
        let out = {
            let input = ctx.input(0);
            let state = ctx.input(1);
            step(input, state, channel, tension, friction, playhead)
        };
        ctx.emit(out);
    }
}

/// One spring step (or a seed) as a pure function — the whole node.
/// **UM NÓ PARA A POSIÇÃO INTEIRA** (doc 89 folha 03 — MOPs *Spring Modifier* faz T/R/S de
/// uma vez; a célula: *"1 canal por nó ⇒ **4 nós** para molejar um transform"*).
///
/// ⚠️ **A célula media a composição e ela FUNCIONA** — encadear quatro molas é legal, cada uma
/// tem o `pre` dela e o clobber de `spring_value` não as cruza. O que entra aqui é o caso
/// COMUM, não a generalização: uma mola de posição precisa dos dois eixos, e dois nós para
/// *"esta coisa persegue aquela"* é o dobro do grafo para um gesto só.
///
/// ⚠️ **É o precedente do `Position XY` do `motion.wiggle`**, e a mesma palavra — um artista
/// que aprendeu o canal num nó não o re-aprende no outro.
///
/// ⚠️ **O estado é APPEND-ONLY:** o eixo Y guarda-se em `spring_value_y`/`spring_vel_y`,
/// colunas que os quatro canais escalares **nunca escrevem**. Um grafo já autorado continua a
/// ler e escrever exactamente as colunas que lia, e o `pairing` continua a perguntar pelo
/// `spring_value` — que existe nos cinco casos.
///
/// ⛔ **E NÃO é o T/R/S inteiro da referência**: rotação e tamanho têm unidades próprias e
/// quereriam a sua própria tensão (uma mola que persegue graus com a rigidez de metros não é
/// a mesma mola), então juntá-los pediria um jogo de knobs por canal — que é o nó de quatro
/// cabeças que a cerca do `motion.scale` (§4-C3) já ensinou a não construir.
const CHANNEL_POSITION_XY: i32 = 4;

fn step(
    input: &Stream,
    state: &Stream,
    channel: i32,
    tension: f32,
    friction: f32,
    playhead: f32,
) -> Stream {
    if channel == CHANNEL_POSITION_XY {
        // Os dois eixos, cada um com o seu estado. O X escreve as colunas de sempre.
        let x = solve(
            input,
            state,
            0,
            "spring_value",
            "spring_vel",
            tension,
            friction,
            playhead,
        );
        let y = solve(
            input,
            state,
            1,
            "spring_value_y",
            "spring_vel_y",
            tension,
            friction,
            playhead,
        );
        let mut out = channel_set(input, 0, &x.0);
        out = channel_set(&out, 1, &y.0);
        out.set("spring_value", Column::Scalar(x.1));
        out.set("spring_vel", Column::Scalar(x.2));
        out.set("spring_value_y", Column::Scalar(y.1));
        out.set("spring_vel_y", Column::Scalar(y.2));
        out.set("sim_t", Column::Scalar(vec![playhead; input.count()]));
        return out;
    }
    let (blended, value, vel) = solve(
        input,
        state,
        channel,
        "spring_value",
        "spring_vel",
        tension,
        friction,
        playhead,
    );
    let mut out = channel_set(input, channel, &blended);
    out.set("spring_value", Column::Scalar(value));
    out.set("spring_vel", Column::Scalar(vel));
    out.set("sim_t", Column::Scalar(vec![playhead; input.count()]));
    out
}

/// **Uma mola sobre UM canal escalar** — devolve `(saída misturada, valor, velocidade)`.
///
/// ⚠️ Os nomes das colunas de estado são ARGUMENTOS, e é isso que deixa o `Position XY` correr
/// a mesma lei duas vezes sem uma segunda cópia dela.
#[expect(
    clippy::too_many_arguments,
    reason = "cada argumento é um param do MANIFEST ou o nome de uma coluna de estado; um struct só para contar menos poria uma segunda declaração da mesma lista ao lado do manifesto"
)]
fn solve(
    input: &Stream,
    state: &Stream,
    channel: i32,
    value_col: &str,
    vel_col: &str,
    tension: f32,
    friction: f32,
    playhead: f32,
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let n = input.count();
    // Pure per-instance map → parallel above the threshold
    // (bit-identical, no reduction). GPU/M5 Fase 0.
    let targets: Vec<f32> = par_build(n, |i| channel_get(input, channel, i));

    // Every element starts seeded AT its target (no snap); the ones the state
    // knows then step. Identity is the `id` column when present (a particle
    // keeps its spring across the set's churn), else positional.
    let mut value = targets.clone();
    let mut vel = vec![0.0f32; n];

    if let Some(prev) = pairing(input, state, n) {
        let sn = state.count();
        let read = |name: &str| -> Vec<f32> {
            let mut v = match state.get(name) {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            };
            v.resize(sn, 0.0);
            v
        };
        let (s_value, s_vel) = (read(value_col), read(vel_col));
        let t_prev = match state.get("sim_t") {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(playhead),
            _ => playhead,
        };
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        // Adaptive sub-step from the stability limit (reference parity).
        let ideal = (STABLE / tension).sqrt();
        let steps = if dt > 0.0 {
            ((dt / ideal).ceil() as usize).clamp(1, MAX_STEPS)
        } else {
            1
        };
        let sub_dt = dt / steps as f32;
        for (i, slot) in prev.iter().enumerate() {
            let Some(j) = *slot else { continue }; // fresh id: stays at its target
            let (mut v, mut x) = (s_vel[j], s_value[j]);
            // NaN/∞ guard (reference parity): a diverged instance recovers.
            if !(x.is_finite() && v.is_finite()) {
                x = targets[i];
                v = 0.0;
            }
            for _ in 0..steps {
                let accel = -friction * v - tension * (x - targets[i]);
                v += accel * sub_dt;
                x += v * sub_dt;
            }
            value[i] = x;
            vel[i] = v;
        }
    }

    // Output channel: the spring blended toward the raw target by the falloff
    // field, times the pin weight (falloff 0 or a pinned element → the spring is
    // transparent there; a free element with no pin multiplies by an exact 1.0,
    // so every pre-pin graph keeps its bit-identical trajectory).
    // Pure per-instance map → parallel above the threshold
    // (bit-identical, no reduction). GPU/M5 Fase 0.
    let blended: Vec<f32> = par_build(n, |i| {
        let fs = falloff_at(input, i) * inv_mass_at(input, i);
        if fs >= 1.0 {
            value[i]
        } else {
            targets[i] + (value[i] - targets[i]) * fs.max(0.0)
        }
    });
    (blended, value, vel)
}

/// For each of the `n` input elements, the row of `state` holding its previous
/// spring — `None` for a freshly-seen element (it starts at its target).
/// `None` overall when there is no state yet, or when a count change on an
/// id-less stream says the set was rebuilt. Mirrors `motion.integrate::pairing`.
fn pairing(input: &Stream, state: &Stream, n: usize) -> Option<Vec<Option<usize>>> {
    state.get("spring_value")?; // no state yet (tick 0, `pre` = Empty)
    let sn = state.count();
    match (ids_of(input, n), ids_of(state, sn)) {
        (Some(in_ids), Some(state_ids)) => {
            let index: BTreeMap<u32, usize> = state_ids
                .iter()
                .enumerate()
                .map(|(j, id)| (*id, j))
                .collect();
            Some(in_ids.iter().map(|id| index.get(id).copied()).collect())
        }
        _ if sn == n => Some((0..n).map(Some).collect()),
        _ => None,
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSpring))?;
    // ADR-0155: a solver reads `inv_mass` — so a `motion.pin_constraint` upstream
    // is live under a spring too.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("inv_mass")],
    );
    reg.register_gpu_kernel(MANIFEST.id, kernel::GPU_KERNEL);
    // ADR-0130: per-element: chases a target, never reorders/rewrites id.
    reg.register_dense_window(MANIFEST.id);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Spring",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1) — ranges mirror the reference's sliders.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "channel",
        label: "Channel",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            // ⚠️ **Apendado no FIM** — ver [`CHANNEL_POSITION_XY`]: os quatro índices que já
            // existiam ficam onde estavam, e toda cena guardada continua a nomear o mesmo canal.
            labels: &["X", "Y", "Rotation", "Size", "Position XY"],
        },
    },
    ParamUiHint {
        param: "tension",
        label: "Tension",
        min: 0.5,
        max: 60.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "friction",
        label: "Friction",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
];

/// **OS TETOS DIGITÁVEIS, MEDIDOS** (doc 88 B2 · doc 89 folha 03 linha 69 · CLAUDE.md §0).
///
/// Este nó não tinha nenhum, e a const [`MAX_STEPS`] já nomeava o vão em prosa (*"64 só guarda
/// overrides absurdos escritos à mão"*) sem dizer onde o absurdo começa. A sonda
/// `measure_spring_ceiling` varre os dois params pela porta do produto (o `Cook` do registry),
/// nos DOIS relógios, e o veredito tem três regimes: **sadia · EXPLODE · SALTA**.
///
/// ⚠️ **O relógio do PIOR caso é que decide, e não é o de 60 fps.** O `eval` clampa `dt` em
/// `MAX_DT = 0,1`, então um quadro perdido ou um salto de régua entrega legitimamente `0,1` ao
/// integrador. Um teto que só vale a 60 fps é um teto que depende da MÁQUINA — a mola do artista
/// explodiria na primeira engasgada, e nada na tela diria por quê.
///
/// **`friction` — o teto é 20, e ele COINCIDE com o slider.** O termo de amortecimento é
/// explícito, logo estável enquanto `friction · sub_dt < 2`; no pior caso `sub_dt = MAX_DT`
/// (tensão baixa ⇒ um sub-passo só) ⇒ **`2 / 0,1 = 20`**. Medido a `MAX_DT`, com o valor de
/// fronteira já explodindo:
///
/// | tension | sub-passos | teto previsto | último sadio | primeiro a explodir |
/// |---|---|---|---|---|
/// | 0,1 | 1 | 20 | **20** | 21 |
/// | 8 (default) | 2 | 40 | 21 | **40** |
/// | 60 (slider) | 4 | 80 | 40 | **80** |
/// | 20.480 | 64 | 1.280 | 200 | **1.280** |
///
/// ⚠️ **Isto é o achado, e ele é sobre o slider:** o `20,0` do `ParamUiHint` **não era um número
/// de gosto** — é `2 / MAX_DT`, o limite de estabilidade do amortecimento explícito no pior
/// passo que o kernel admite. O slider já estava SENTADO no teto, e ninguém sabia. ⚠️ E o preço
/// fica NOMEADO: entre 20 e ~120 a mola funciona a 60 fps e explode no primeiro `dt` grande —
/// *o valor certo seria função de OUTRO knob* (`20 · sub-passos`, ou seja da tensão), e a lei
/// desta casa é que isso é bug de desenho ([[feedback_ergonomics_verdict_is_a_design_bug]]).
/// Escolher o pior caso é o único número que **nunca mente**.
///
/// **`tension` — 1.600.000**, quatro ordens de grandeza acima do slider, e o teto existe pelo
/// que acontece ACIMA dele, não abaixo. Medido com o `friction` no teto DELE (20) e a `MAX_DT`:
///
/// | tension | pico \|Y\| | veredito |
/// |---|---|---|
/// | 800.000 | 100,204 | sadia |
/// | **1.600.000** | 461,368 | **sadia (o último)** |
/// | 1.638.400 | 7,6e28 | EXPLODE |
/// | 2.000.000 | 1,1e28 | EXPLODE |
/// | 4.000.000 | 100,000 | **SALTA** |
///
/// ⚠️ **A linha de 4 M é a razão de o oráculo ter três braços e não dois.** Ela reporta pico
/// exactamente `100,000` — *sadia*, para quem só olha a magnitude —, e não é: numa tensão
/// absurda o passo estoura DENTRO do primeiro sub-passo, o guard de NaN do `eval` repõe a
/// posição no alvo e a velocidade em zero, e isso se repete a cada tique. A mola fica **pregada
/// no alvo, finita e imóvel** — um controle que parece não fazer nada. É exactamente o que o
/// artista vê hoje, sem teto nenhum, ao digitar um número grande. O discriminante é o TEMPO:
/// uma mola de verdade leva tiques para chegar; uma pregada pelo guard já está lá no tique
/// seguinte ao degrau.
///
/// ⚠️ **E a saturação do passo adaptativo NÃO é o teto** — a derivação óbvia diz que `steps`
/// satura em 64 a partir de `tension = 20.480` (`MAX_DT · √(T/STABLE) > 64`) e que dali o
/// `sub_dt` para de encolher; medido, **nada quebra ali**. A razão é que `STABLE = 0,05` é
/// **80× mais conservador** que o limite real deste integrador: o passo é semi-implícito
/// (`v` antes de `x`), cuja fronteira é `sub_dt² · tension < 4`. O teto verdadeiro é
/// `4 / (MAX_DT / MAX_STEPS)² = 4 · 640² = 1.638.400` — que é **o número que a medição achou**,
/// ao dígito. Derivação e relógio como duas testemunhas; o teto fica no último SADIO.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "tension",
        max: 1_600_000.0,
    },
    ParamHardMax {
        param: "friction",
        max: 20.0,
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "xy_tests.rs"]
mod xy_tests;
