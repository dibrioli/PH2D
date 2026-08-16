#![forbid(unsafe_code)]
//! `motion.verlet-rope` — a **sequential** rope/chain simulation: `count` points
//! linked by fixed-length segments, integrated with position-based Verlet and
//! relaxed against distance constraints, hanging from a pinned anchor that can be
//! ANIMATED (Motion Nodes M4, simulation — doc 01 §3 / doc 21). Slide the anchor
//! and the rope whips and swings behind it with real follow-through; let it hang
//! and gravity pulls it into a catenary. The canonical "cloth strand / whip /
//! pendulum chain" of every motion pack.
//!
//! **Algorithm — Jakobsen's *Advanced Character Physics* (2001), the gold
//! standard.** Verlet stores each point's *current* and *previous* position; the
//! step is `x' = x + (x − x_prev)·(1−damp) + a·dt²` (velocity is implicit in the
//! position pair, so there is no separate velocity state to diverge), then a few
//! **relaxation passes** pull each segment back to its rest length. It is
//! unconditionally stable (unlike explicit spring meshes, which explode when
//! stiff) and dead simple. Position-based constraints are exactly why cloth/rope
//! sims use Verlet rather than force springs.
//!
//! ## Topology (the `pre` self-loop)
//!
//! A sequential node like `motion.spring`/`motion.integrate`: its state (the point
//! positions + their previous positions) rides its own output through the `state`
//! feedback port, which the editor auto-plumbs as the `pre` self-loop on add
//! (`out --pre--> state`). At tick 0 the `pre` reads Empty → the node **seeds** a
//! straight strand from the anchor; from tick 1 on it steps. `dt` derives from the
//! state's own `sim_t` column (playhead at last eval), clamped to `[0, MAX_DT]` —
//! no cross-crate timestep, and a backwards playhead jump (loop wrap) freezes for
//! one tick instead of exploding, exactly like `motion.integrate`.
//!
//! ## The anchor is animatable (value inputs)
//!
//! `anchor_x`/`anchor_y` are **value** inputs (doc 12): wire a `value.lfo` and the
//! pinned head slides, whipping the whole chain. Unconnected, the anchor reads as
//! the origin `(0, 0)`. `pin_tail` optionally also pins the far end to a fixed
//! point (a washing-line / suspension bridge) instead of leaving it free (a whip).
//!
//! Transcendental-free (HR-5): Verlet integration + constraint relaxation are
//! arithmetic and one IEEE `sqrt` per segment (the segment length), like
//! `motion.spring`. Deterministic → `Effect::Temporal` (reads the playhead), replays
//! bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};

mod params_ui;
use params_ui::{PARAM_HARD_MAX, PARAM_HINTS, PARAM_UNITS};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `anchor_*` inputs — the per-instance scalar field on the
/// `v` column (mirror of `motion.look_at::VALUE`; kept local, leaf crate).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Ceiling on a single integration step (see `motion.integrate`): in steady state
/// `dt` is the fixed timestep; this only guards a pathological playhead jump.
const MAX_DT: f32 = 0.1;
/// Below this a segment length is treated as zero (skip the normalise).
const EPS: f32 = 1e-6;
/// With `pin_tail`, the far end is fixed at this fraction of the rope length from
/// the head — leaving 25% slack so the span sags into a catenary instead of
/// pulling taut into a straight line.
const PINNED_SPAN: f32 = 0.75;

/// A massa INVERSA por ponto (`motion.pin_constraint`): `1` = livre, `0` = pinado.
/// Convenção de string partilhada pelos solvers do módulo, soletrada LOCALMENTE
/// por cada leitor (como `P` / `accel`) em vez de acoplar as crates.
const INV_MASS_COL: &str = "inv_mass";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.verlet_rope"),
    name: "motion.verlet_rope",
    inputs: &[
        // The pinned head, as two value fields (so it can be animated). Optional:
        // unconnected reads as 0 → the origin.
        PortSpec {
            name: "anchor_x",
            ty: VALUE,
        },
        PortSpec {
            name: "anchor_y",
            ty: VALUE,
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
    // Temporal: `eval` reads `ctx.playhead()` (stamps `sim_t`, derives `dt`).
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        // Number of points in the chain. Clamped ≥2 and capped like the grid.
        ParamSpec {
            name: "count",
            default: 24.0,
        },
        // Total rest length (world units); segment rest = length / (count−1).
        ParamSpec {
            name: "length",
            default: 6.0,
        },
        // Downward gravity magnitude (accel, world units/s²).
        ParamSpec {
            name: "gravity",
            default: 9.0,
        },
        // Constraint relaxation passes — higher = stiffer/less stretchy.
        ParamSpec {
            name: "iterations",
            default: 24.0,
        },
        // Verlet velocity damping per step ∈ [0,1) — settles the swing.
        ParamSpec {
            name: "damping",
            default: 0.02,
        },
        // 0 = free tail (a whip); 1 = the far end is pinned too (a bridge/line).
        ParamSpec {
            name: "pin_tail",
            default: 0.0,
        },
        // A RIGIDEZ À FLEXÃO (Vellum *Bend Stiffness*) — **0 = off**, o default,
        // e o caminho de hoje é pulado por completo, não multiplicado por zero.
        ParamSpec {
            name: "bend",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The value coordinate for element 0 (the anchor): **unconnected (empty) → 0.0**;
/// otherwise the first element (broadcast).
fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn vec2_col(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The transient `accel` the state carries, at exactly `n` — **absent is zeros**.
///
/// ## What this one column buys
///
/// A `force.*` node wired into this rope's `state` chain (`rope.out --pre-->
/// force.wind --> rope.state`) accumulates world-units/s² here, and reading it
/// hands the rope the WHOLE force family at once: gravity with a DIRECTION,
/// wind, curl, an attractor, a vortex, drag. None of that is a kernel this crate
/// has to grow — it is one column, read once (doc 89 §2.1).
///
/// ⚠️ **Consumed, never emitted.** The emitted stream carries `P`/`rope_prev`/
/// `sim_t` and nothing else, so every tick starts from zero acceleration — the
/// same discipline `motion.integrate` states in its own docs. A rope that
/// forwarded `accel` would integrate last tick's wind forever.
///
/// ⚠️ **Zeros are the IDENTITY, so a rope no force reaches is byte-identical to
/// the one that shipped**: `x + 0.0 * dt²` is `x`. That is a property of the
/// arithmetic, not a fast path to keep in step with a slow one.
/// A massa inversa por ponto, alargada a `n` e tornada segura — o espelho exacto
/// do leitor do `motion.collide`: **ausente lê como livre (`1`)**, e um peso
/// negativo ou não-finito de um documento editado à mão lê como **pinado (`0`)**
/// em vez de INVERTER a correção.
///
/// ## O que esta coluna compra, e por que é a MESMA porta do `accel`
///
/// Um `motion.pin_constraint` na cadeia de estado desta corda
/// (`rope.out --pre--> pin --> rope.state`) prega um ÍNDICE ARBITRÁRIO — a
/// capacidade que a folha 03 pedia (linha 51) e que o doc do pino declarava
/// inalcançável (*"um pino a montante não tem fio por onde os alcançar"*). O fio
/// é a cadeia de estado, e ela já era o fio pelo qual o `accel` entra.
///
/// ⚠️ **Consumida, nunca emitida** — a mesma disciplina do `accel`, e aqui a razão
/// é MEDÍVEL: o `motion.pin_constraint` MULTIPLICA no que já está no stream, então
/// uma corda que reemitisse `inv_mass` faria um pino parcial de `0,5` decair
/// `0,5 → 0,25 → 0,125` a cada tique — o *produto sobre a lista* que este módulo
/// já pagou noutro lugar. Emitida uma vez por tique pelo pino, lida uma vez.
fn inv_mass_col(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(INV_MASS_COL) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

/// A fração da correção que cabe a cada ponta de uma restrição — a fórmula PBD
/// `w_i / (w_i + w_j)`, a MESMA que o `motion.collide` usa no seu empurrão.
///
/// ⚠️ **Ela REDUZ LITERALMENTE à tabela de quatro braços que shipava** quando os
/// pesos são `{0, 1}`: `0/1 = 0`, `1/1 = 1` e `1/2 = 0,5` são todos EXACTOS em
/// IEEE-754, e o par degenerado `(0, 0)` cai no guard. É por isso que uma corda
/// que nenhum pino alcança é byte-idêntica — por ARITMÉTICA, não por promessa.
fn share(wa: f32, wb: f32) -> (f32, f32) {
    let sum = wa + wb;
    if sum <= 0.0 {
        return (0.0, 0.0);
    }
    (wa / sum, wb / sum)
}

fn accel_col(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("accel") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

/// The simulation parameters resolved at eval (all arithmetic-ready).
struct Params {
    count: usize,
    seg_rest: f32,
    gravity: f32,
    iterations: usize,
    damping: f32,
    pin_tail: bool,
    /// **A diferença entre uma CORDA e um CABO** (Houdini Vellum *Bend
    /// Stiffness*, ao lado do *Stretch* que a relaxação de distância já é).
    ///
    /// Hoje só existe a restrição `i↔i+1`, que fixa o COMPRIMENTO e não diz nada
    /// sobre o ÂNGULO — então a corda dobra 180° sobre si mesma sem custo, que é
    /// o que o artista vê na primeira cena. A restrição de flexão é a irmã
    /// `i↔i+2`, cujo repouso é a configuração RETA (`2 · seg_rest`): puxar o
    /// vão de dois segmentos de volta ao comprimento reto é exatamente resistir
    /// à dobra, sem um segundo modelo e sem trigonometria (HR-5).
    ///
    /// `0..1` é a fração da correção aplicada por passe — `1` com iterações
    /// suficientes converge para uma BARRA; valores baixos dão o cabo que cede.
    bend: f32,
}

/// Seed a straight, horizontal strand of `count` points from `anchor`, pinned at
/// index 0 (previous == current → at rest). The first gravity step then swings it.
fn seed(anchor: [f32; 2], p: &Params) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let pos: Vec<[f32; 2]> = (0..p.count)
        .map(|i| [anchor[0] + i as f32 * p.seg_rest, anchor[1]])
        .collect();
    let prev = pos.clone();
    (pos, prev)
}

/// One Verlet step + constraint relaxation as a pure function.
///
/// `pos`/`prev` are this tick's entry state (last tick's positions). Returns the
/// new `(pos, prev)`: `prev_out` = the entry `pos` (Verlet's memory), `pos_out` =
/// the integrated + relaxed positions. Pinned points (head, and tail if
/// `pin_tail`) are clamped to their fixed targets every pass.
///
/// ## Duas espécies de pino, e a diferença é o ALVO
///
/// O peso efectivo é `0` para os pinos INTRÍNSECOS (a cabeça, e a cauda com
/// `pin_tail`) e a massa inversa lida do stream para todos os outros. Os dois são
/// massa infinita e tomam `0` de toda correção; o que os separa é para ONDE
/// olham — o intrínseco é clampado a um alvo **animado** (a âncora que uma
/// `value.lfo` varre) e o genérico segura **onde está**, que é o que *pinar um
/// índice* significa quando não há alvo nenhum a que o prender.
#[allow(clippy::too_many_arguments)]
fn step(
    mut pos: Vec<[f32; 2]>,
    prev: &[[f32; 2]],
    accel: &[[f32; 2]],
    inv_mass: &[f32],
    anchor: [f32; 2],
    tail_pin: [f32; 2],
    dt: f32,
    p: &Params,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let n = pos.len();
    // O peso efectivo: os pinos intrínsecos são massa infinita, o resto é o que a
    // cadeia de estado trouxe (ausente ⇒ `1`, livre).
    let w: Vec<f32> = (0..n)
        .map(|i| {
            if is_pinned(i, n, p) {
                0.0
            } else {
                inv_mass.get(i).copied().unwrap_or(1.0)
            }
        })
        .collect();
    // Verlet's memory for the NEXT tick is this tick's entry positions.
    let prev_out = pos.clone();
    let keep = 1.0 - p.damping;
    // ⚠️ `ga` keeps its ORIGINAL association `(gravity·dt)·dt` — IEEE-754
    // multiplication is not associative, and re-grouping it as `gravity·dt²`
    // moves the ulp on every rope that ever hung. `dt2` is for the term that is
    // NEW here, where there is no earlier grouping to preserve.
    let dt2 = dt * dt;
    let ga = p.gravity * dt * dt; // a·dt² (downward, so subtract from y)

    // Integrate every point; the pins are overwritten right after.
    //
    // ⚠️ Massa infinita não integra — e para os pinos INTRÍNSECOS isto é um no-op
    // byte-idêntico, porque o `pin()` logo abaixo sobrescrevia o que a integração
    // deles produzia. Para um pino GENÉRICO é a linha inteira: sem alvo a que o
    // clampar, *não se mover* É o pino, e `prev_out` já guardou a posição de
    // entrada ⇒ a velocidade dele fica zero e ele segura para sempre.
    for i in 0..n {
        if w[i] <= 0.0 {
            continue;
        }
        let (c, pv) = (pos[i], prev[i]);
        // The external `accel` enters exactly where the built-in gravity does —
        // both are accelerations, and Verlet takes an acceleration as `a·dt²`.
        let a = accel.get(i).copied().unwrap_or([0.0, 0.0]);
        // ⚠️ **A massa inversa escala a ACELERAÇÃO e NÃO a inércia**, e a assimetria
        // é o modelo: `a = F/m` com a gravidade e o `accel` lidos como forças por
        // massa de referência (a leitura que o `motion.integrate` já faz e que o doc
        // do pino promete — *"deixar um elemento mais pesado meramente RESISTIR"*),
        // enquanto `(c − pv)` é MOMENTO, e escalá-lo faria um ponto pesado perder
        // velocidade, que é o contrário de pesado.
        //
        // ⚠️ **Sem isto um pino parcial é INVISÍVEL numa corda em repouso**, e está
        // medido: só a partilha de correções muda, e no equilíbrio as restrições já
        // estão satisfeitas ⇒ `strength = 0,5` desenhava EXACTAMENTE o mesmo que
        // `0,0` (y = −3,0688 nos dois). Peso `1` é byte-idêntico (`x · 1.0` é exacto).
        let mut np = [
            c[0] + (c[0] - pv[0]) * keep + a[0] * dt2 * w[i],
            c[1] + (c[1] - pv[1]) * keep - ga * w[i] + a[1] * dt2 * w[i],
        ];
        // NaN/∞ guard (reference parity): a diverged point recovers at the anchor.
        if !(np[0].is_finite() && np[1].is_finite()) {
            np = anchor;
        }
        pos[i] = np;
    }
    pin(&mut pos, anchor, tail_pin, p);

    // Relaxation: pull each segment back to its rest length. A pinned endpoint
    // holds; a free one takes the full correction, else each takes half.
    for _ in 0..p.iterations {
        for i in 0..n.saturating_sub(1) {
            let (a, b) = (pos[i], pos[i + 1]);
            let d = [b[0] - a[0], b[1] - a[1]];
            let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if dist < EPS {
                continue;
            }
            let diff = (dist - p.seg_rest) / dist;
            let (wa, wb) = share(w[i], w[i + 1]);
            pos[i] = [a[0] + d[0] * diff * wa, a[1] + d[1] * diff * wa];
            pos[i + 1] = [b[0] - d[0] * diff * wb, b[1] - d[1] * diff * wb];
        }
        // A restrição de FLEXÃO — a irmã `i↔i+2` da de distância, no MESMO passe
        // de relaxação (Gauss-Seidel, como o Vellum resolve as suas). O repouso é
        // a configuração RETA, então encurtar o vão de dois segmentos custa.
        //
        // ⚠️ **O early-out é o que torna `bend = 0` byte-idêntico**, e não a
        // multiplicação por zero: `x + (-0.0)` devolve `x`, mas `(-0.0) + 0.0`
        // devolve `+0.0` — um padrão de bits diferente. Uma corda cujo ponto
        // pousa exatamente em `-0.0` (o topo pinado na origem é o caso comum)
        // teria mudado de bits sem ninguém pedir nada.
        if p.bend > 0.0 {
            let bend_rest = p.seg_rest + p.seg_rest;
            for i in 0..n.saturating_sub(2) {
                let (a, b) = (pos[i], pos[i + 2]);
                let d = [b[0] - a[0], b[1] - a[1]];
                let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
                if dist < EPS {
                    continue;
                }
                let diff = (dist - bend_rest) / dist * p.bend;
                let (wa, wb) = share(w[i], w[i + 2]);
                pos[i] = [a[0] + d[0] * diff * wa, a[1] + d[1] * diff * wa];
                pos[i + 2] = [b[0] - d[0] * diff * wb, b[1] - d[1] * diff * wb];
            }
        }
        pin(&mut pos, anchor, tail_pin, p);
    }
    (pos, prev_out)
}

/// Whether point `i` of `n` is pinned (head always; tail when `pin_tail`).
fn is_pinned(i: usize, n: usize, p: &Params) -> bool {
    i == 0 || (p.pin_tail && i + 1 == n)
}

/// Clamp the pinned points to their fixed targets.
fn pin(pos: &mut [[f32; 2]], anchor: [f32; 2], tail_pin: [f32; 2], p: &Params) {
    if let Some(head) = pos.first_mut() {
        *head = anchor;
    }
    if p.pin_tail
        && let Some(tail) = pos.last_mut()
    {
        *tail = tail_pin;
    }
}

/// The whole node as a pure function: seed on the first tick / a count change,
/// else step. Returns the emitted stream (`P` + the `rope_prev`/`sim_t` state).
fn simulate(anchor: [f32; 2], state: &Stream, playhead: f32, p: &Params) -> Stream {
    let s_pos = vec2_col(state, "P");
    let s_prev = vec2_col(state, "rope_prev");
    // The tail's fixed point sits at PINNED_SPAN of the rope length along +x from
    // the head, so the extra length sags. Derived (not stored) so it never drifts.
    let tail_pin = [
        anchor[0] + PINNED_SPAN * (p.count as f32 - 1.0) * p.seg_rest,
        anchor[1],
    ];

    let (pos, prev) = if s_pos.len() == p.count && s_prev.len() == p.count {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        let accel = accel_col(state, p.count);
        let w = inv_mass_col(state, p.count);
        step(s_pos, &s_prev, &accel, &w, anchor, tail_pin, dt, p)
    } else {
        // Tick 0 (Empty state) or a count change → re-seed the strand.
        seed(anchor, p)
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("rope_prev", Column::Vec2(prev))
        .with("sim_t", Column::Scalar(vec![playhead; p.count]))
}

struct MotionVerletRope;

impl NodeOp for MotionVerletRope {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS).max(2);
        let length = ctx.param("length").max(0.0);
        let p = Params {
            count,
            seg_rest: length / (count as f32 - 1.0),
            gravity: ctx.param("gravity"),
            iterations: (ctx.param("iterations").round() as i64).clamp(1, 128) as usize,
            damping: ctx.param("damping").clamp(0.0, 0.99),
            pin_tail: ctx.param("pin_tail") >= 0.5,
            bend: ctx.param("bend").clamp(0.0, 1.0),
        };
        let playhead = ctx.playhead() as f32;
        let anchor = [
            value_head(&scalar_col(ctx.input(0), VALUE_COL)),
            value_head(&scalar_col(ctx.input(1), VALUE_COL)),
        ];
        let state = ctx.input(2);
        let out = simulate(anchor, state, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionVerletRope))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Verlet Rope",
            // Source: it mints its own point stream (like Grid / Scatter), then
            // simulates it.
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // ADR-0155: this rope CONSUMES `accel`, so a `force.*` wired into its state
    // chain is live rather than inert — and the diagnose sees a consumer where
    // it would otherwise offer to splice a `motion.integrate`. That offer would
    // be poison here: an integrator is `Temporal`, it stamps `sim_t = playhead`
    // on the way past, and this node derives `dt` from the state's own `sim_t`
    // — so the "cure" would hand the rope `dt = 0` and FREEZE it.
    // E CONSOME `inv_mass`, o que faz um `motion.pin_constraint` na cadeia de
    // estado prender um índice arbitrário (folha 03, linha 51 / 77).
    reg.register_couplings(
        MANIFEST.id,
        &[
            ph2d_node_registry::Coupling::Consumes("accel"),
            ph2d_node_registry::Coupling::Consumes("inv_mass"),
        ],
    );
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
