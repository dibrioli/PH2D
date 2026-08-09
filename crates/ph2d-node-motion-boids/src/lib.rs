#![forbid(unsafe_code)]
//! `motion.boids` — **emergent flocking**: `count` agents that steer by the three
//! classic local rules and coalesce into a living flock (Motion Nodes M4,
//! simulation — doc 01 §3 / doc 21). No path is authored; the swirling, splitting,
//! re-forming shoal is what *emerges* from every agent obeying its neighbours. The
//! counterpart to the deterministic `motion.verlet_rope`: same sequential substrate
//! (`pre` state), opposite character — self-organising rather than constrained.
//!
//! **Algorithm — Reynolds' boids (SIGGRAPH 1987), the gold standard.** Each agent
//! looks only at flock-mates within a perception `radius` and sums three steering
//! urges:
//! - **Separation** — steer away from crowding neighbours (inverse-square repulsion,
//!   so it bites only when they get close).
//! - **Alignment** — steer toward the average heading of neighbours.
//! - **Cohesion** — steer toward the average position (centroid) of neighbours.
//!
//! Plus a **seek** pull toward a target point (the value inputs; unconnected → the
//! origin). Seek is a linear spring, so it doubles as the flock's tether: it grows
//! with distance and keeps the swarm bounded around its home (no fly-away, no
//! separate boundary hack). A `value.lfo` on the target herds the whole flock.
//!
//! Neighbour search is the honest `O(N²)` all-pairs scan — exact and simple for the
//! bounded counts a node produces. A spatial hash (the standard scale trick) is a
//! pure-perf follow-up (doc 21 §5); it would not change a single emitted position.
//!
//! ## Topology (the `pre` self-loop)
//!
//! Sequential like `motion.spring`/`motion.integrate`/`motion.verlet_rope`: each
//! agent's `P` and `vel` ride the `state` feedback port, auto-plumbed as the `pre`
//! self-loop on add (`out --pre--> state`). Tick 0 (`pre` = Empty) **seeds** a
//! hashed cloud of positions + velocities (stateless, reproducible — Jarzynski/
//! Olano); from tick 1 it steps. `dt` derives from the state's own `sim_t`, clamped
//! to `[0, MAX_DT]` (a loop-wrap freezes one tick, never explodes) — like
//! `motion.integrate`.
//!
//! Transcendental-free (HR-5): steering is add/multiply and one IEEE `sqrt` per
//! normalise (heading / speed clamp); the seed is the splitmix hash. Deterministic
//! → `Effect::Temporal`, replays bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod gpu;
mod hash;
use hash::hash3;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `target_*` inputs (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Ceiling on a single step (see `motion.integrate`): guards a playhead jump.
const MAX_DT: f32 = 0.1;
/// Half-extent of the seed cloud (world units), centred on the target/home.
const SEED_SPREAD: f32 = 3.0;
/// The count at which the √N spread (`spread` on) reproduces the fixed
/// [`SEED_SPREAD`] — chosen so the density (agents per perception cell) it holds
/// is a lively murmuration (~22 neighbours per agent at the default radius). It
/// is the ONLY thing the reference sets: at `spread` off it is unused, and at any
/// count it is the density, never the size.
const SEED_DENSITY_REF: f32 = 64.0;
/// A boid never drifts below this fraction of `max_speed` — keeps the flock
/// gliding instead of stalling to a dead point.
const MIN_SPEED_FRAC: f32 = 0.2;
/// Below this a vector length is treated as zero (skip the normalise).
const EPS: f32 = 1e-6;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.boids"),
    name: "motion.boids",
    inputs: &[
        // The seek target / home, as two value fields (animatable). Optional:
        // unconnected reads as 0 → the origin.
        PortSpec {
            name: "target_x",
            ty: VALUE,
        },
        PortSpec {
            name: "target_y",
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
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "count",
            default: 48.0,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
        // Perception radius: only neighbours within it steer an agent.
        ParamSpec {
            name: "radius",
            default: 2.0,
        },
        ParamSpec {
            name: "separation",
            default: 1.6,
        },
        ParamSpec {
            name: "alignment",
            default: 1.0,
        },
        ParamSpec {
            name: "cohesion",
            default: 0.9,
        },
        // Pull toward the target/home (a linear spring → also the tether/bound).
        ParamSpec {
            name: "seek",
            default: 1.0,
        },
        ParamSpec {
            name: "max_speed",
            default: 4.0,
        },
        // Reynolds' OTHER clamp — the steering budget (**0 = off**, the default,
        // so every flock authored before this param is byte-identical).
        ParamSpec {
            name: "max_force",
            default: 0.0,
        },
        // √N seed spread (0 = fixed, the default; 1 = density-bounded). Off is
        // byte-identical at EVERY count — it only decides whether a bigger flock
        // is a denser ball or a wider murmuration (the latter keeps the spatial
        // grid `O(N)`, which is how the count reaches the millions the GPU can do).
        ParamSpec {
            name: "spread",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Resolved simulation weights (arithmetic-ready).
struct Params {
    count: usize,
    seed: u32,
    radius_sq: f32,
    separation: f32,
    alignment: f32,
    cohesion: f32,
    seek: f32,
    max_speed: f32,
    /// **O orçamento de DIREÇÃO** — o segundo clamp de Reynolds (*Steering
    /// Behaviors For Autonomous Characters*, GDC 1999). `max_speed` limita quão
    /// RÁPIDO um agente vai; este limita quão FORTE ele consegue virar, e o
    /// modelo canônico tem os dois: sem ele um bando com `cohesion` alta faz
    /// curvas de ângulo reto, porque a aceleração de steering não tem teto.
    ///
    /// ⚠️ **`0` é DESLIGADO, e o encoding não custa capacidade:** um orçamento
    /// de steering exatamente zero é o agente que não vira nada, e isso já é
    /// alcançável zerando os quatro pesos (`separation`/`alignment`/`cohesion`/
    /// `seek`) — então o zero está livre para carregar *sem teto*. A
    /// alternativa (default `∞`) poria `inf` na caixa numérica do artista.
    max_force: f32,
    /// √N seed spread: when on, the seed cloud grows with the count so the
    /// density stays bounded and the spatial grid stays `O(N)` — the difference
    /// between a dense ball and a murmuration at a million agents. Off (the
    /// default) is the fixed [`SEED_SPREAD`], byte-identical at every count.
    spread: bool,
}

impl Params {
    /// The seed cloud's half-extent: fixed, or grown with √N (bounded density).
    /// At [`SEED_DENSITY_REF`] agents the two agree, so `spread` never changes
    /// the reference-count flock — it only holds the density as the count moves.
    fn seed_half_extent(&self) -> f32 {
        if self.spread {
            SEED_SPREAD * (self.count as f32 / SEED_DENSITY_REF).sqrt()
        } else {
            SEED_SPREAD
        }
    }
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
/// A `force.*` wired into this flock's `state` chain (`boids.out --pre-->
/// force.curl --> boids.state`) accumulates world-units/s² here, and this one
/// read hands the flock the whole force family: curl (which IS Reynolds'
/// *wander*), wind, an attractor, a vortex, drag (doc 89 §2.1). It joins the
/// three flocking urges as a fourth term, so `max_speed` still bounds it.
///
/// ⚠️ **Consumed, never emitted** (the stream carries `P`/`vel`/`sim_t`), so
/// every tick starts from zero acceleration; and zeros are the IDENTITY, so a
/// flock no force reaches is byte-identical to the one that shipped.
fn accel_col(s: &Stream, n: usize) -> Vec<[f32; 2]> {
    match s.get("accel") {
        Some(Column::Vec2(v)) if v.len() == n => v.clone(),
        _ => vec![[0.0, 0.0]; n],
    }
}

/// The value coordinate for the (single) target: **unconnected → 0.0** (origin).
fn value_head(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(0.0)
}

/// `(unit, len)` of `v`; `([0,0], 0)` for a ~zero vector (no NaN).
fn norm(v: [f32; 2]) -> ([f32; 2], f32) {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len < EPS {
        ([0.0, 0.0], 0.0)
    } else {
        ([v[0] / len, v[1] / len], len)
    }
}

/// Seed a hashed cloud of positions + velocities around `home` (tick 0 / a count
/// change). Stateless: a pure function of `(seed, index)`, so it reproduces.
fn seed(home: [f32; 2], p: &Params) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let mut pos = Vec::with_capacity(p.count);
    let mut vel = Vec::with_capacity(p.count);
    let half = p.seed_half_extent();
    for i in 0..p.count as u32 {
        pos.push([
            home[0] + (hash3(p.seed, i, 0) - 0.5) * 2.0 * half,
            home[1] + (hash3(p.seed, i, 1) - 0.5) * 2.0 * half,
        ]);
        // Initial velocities spread across ±max_speed/2 so the flock starts alive.
        vel.push([
            (hash3(p.seed, i, 2) - 0.5) * p.max_speed,
            (hash3(p.seed, i, 3) - 0.5) * p.max_speed,
        ]);
    }
    (pos, vel)
}

/// One flocking step as a pure function. `pos`/`vel` are this tick's entry state.
fn step(
    pos: &[[f32; 2]],
    vel: &[[f32; 2]],
    ext_accel: &[[f32; 2]],
    target: [f32; 2],
    dt: f32,
    p: &Params,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let n = pos.len();
    let mut out_pos = vec![[0.0f32; 2]; n];
    let mut out_vel = vec![[0.0f32; 2]; n];
    let min_speed = p.max_speed * MIN_SPEED_FRAC;

    for i in 0..n {
        let (pi, vi) = (pos[i], vel[i]);
        // Accumulate the three Reynolds urges over neighbours within the radius.
        let mut sep = [0.0f32; 2];
        let mut align = [0.0f32; 2];
        let mut centroid = [0.0f32; 2];
        let mut neighbours = 0u32;
        for j in 0..n {
            if j == i {
                continue;
            }
            let d = [pi[0] - pos[j][0], pi[1] - pos[j][1]];
            let dist_sq = d[0] * d[0] + d[1] * d[1];
            if dist_sq > p.radius_sq || dist_sq < EPS {
                continue;
            }
            // Separation: inverse-square repulsion (bites hardest when closest).
            let inv = 1.0 / dist_sq;
            sep[0] += d[0] * inv;
            sep[1] += d[1] * inv;
            // Alignment / cohesion accumulate neighbour heading / position.
            align[0] += vel[j][0];
            align[1] += vel[j][1];
            centroid[0] += pos[j][0];
            centroid[1] += pos[j][1];
            neighbours += 1;
        }

        let mut accel = [0.0f32; 2];
        if neighbours > 0 {
            let inv_n = 1.0 / neighbours as f32;
            // Alignment: steer toward the average heading.
            accel[0] += (align[0] * inv_n - vi[0]) * p.alignment;
            accel[1] += (align[1] * inv_n - vi[1]) * p.alignment;
            // Cohesion: steer toward the centroid.
            accel[0] += (centroid[0] * inv_n - pi[0]) * p.cohesion;
            accel[1] += (centroid[1] * inv_n - pi[1]) * p.cohesion;
            // Separation: push away (already an inverse-square sum).
            accel[0] += sep[0] * p.separation;
            accel[1] += sep[1] * p.separation;
        }
        // Seek/home: a linear spring toward the target — herds AND bounds.
        accel[0] += (target[0] - pi[0]) * p.seek;
        accel[1] += (target[1] - pi[1]) * p.seek;
        // Reynolds' STEERING BUDGET (GDC 1999) — truncate what the agent's own
        // muscle may ask for, before anything the WORLD does to it. `0` is off.
        //
        // ⚠️ The external accel is deliberately OUTSIDE this: `max_force` is how
        // hard a boid can TURN, not how much wind it can ignore. Clamping the
        // two together would let a steering budget cancel a gale — a flock that
        // resists arbitrary force is not a flock, and it would quietly undo the
        // whole point of a `force.*` reaching the state chain.
        if p.max_force > 0.0 {
            let (unit, mag) = norm(accel);
            if mag > p.max_force {
                accel = [unit[0] * p.max_force, unit[1] * p.max_force];
            }
        }
        // The EXTERNAL urge, from a `force.*` in this flock's state chain: a
        // fourth term beside the three Reynolds ones, added BEFORE the speed
        // clamp on purpose — `max_speed` still bounds the flock, which is what
        // makes `force.curl` read as Reynolds' *wander* instead of a shove.
        let ext = ext_accel.get(i).copied().unwrap_or([0.0, 0.0]);
        accel[0] += ext[0];
        accel[1] += ext[1];

        // Integrate, then clamp the speed into [min, max] so the flock glides.
        let mut nv = [vi[0] + accel[0] * dt, vi[1] + accel[1] * dt];
        let (unit, speed) = norm(nv);
        if speed > p.max_speed {
            nv = [unit[0] * p.max_speed, unit[1] * p.max_speed];
        } else if speed > EPS && speed < min_speed {
            nv = [unit[0] * min_speed, unit[1] * min_speed];
        }
        let mut np = [pi[0] + nv[0] * dt, pi[1] + nv[1] * dt];
        // NaN/∞ guard (reference parity): a diverged agent recovers at the target.
        if !(np[0].is_finite() && np[1].is_finite() && nv[0].is_finite() && nv[1].is_finite()) {
            np = target;
            nv = [0.0, 0.0];
        }
        out_pos[i] = np;
        out_vel[i] = nv;
    }
    (out_pos, out_vel)
}

/// The whole node as a pure function: seed on the first tick / a count change,
/// else step. Emits `P` + the `vel`/`sim_t` state.
fn simulate(target: [f32; 2], state: &Stream, playhead: f32, p: &Params) -> Stream {
    let s_pos = vec2_col(state, "P");
    let s_vel = vec2_col(state, "vel");

    let (pos, vel) = if s_pos.len() == p.count && s_vel.len() == p.count {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        let accel = accel_col(state, p.count);
        step(&s_pos, &s_vel, &accel, target, dt, p)
    } else {
        seed(target, p)
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("vel", Column::Vec2(vel))
        .with("sim_t", Column::Scalar(vec![playhead; p.count]))
}

struct MotionBoids;

impl NodeOp for MotionBoids {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS).max(1);
        let radius = ctx.param("radius").max(0.0);
        let p = Params {
            count,
            seed: ctx.param("seed").max(0.0).round() as u32,
            radius_sq: radius * radius,
            separation: ctx.param("separation").max(0.0),
            alignment: ctx.param("alignment").max(0.0),
            cohesion: ctx.param("cohesion").max(0.0),
            seek: ctx.param("seek").max(0.0),
            max_speed: ctx.param("max_speed").max(0.01),
            max_force: ctx.param("max_force").max(0.0),
            spread: ctx.param("spread") > 0.5,
        };
        let playhead = ctx.playhead() as f32;
        let target = [
            value_head(&scalar_col(ctx.input(0), VALUE_COL)),
            value_head(&scalar_col(ctx.input(1), VALUE_COL)),
        ];
        let state = ctx.input(2);
        let out = simulate(target, state, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionBoids))?;
    // GPU/M5 (ADR-0140): the flock on the device via the spatial grid.
    reg.register_gpu_kernel(MANIFEST.id, gpu::GPU_KERNEL);
    reg.register_grid(MANIFEST.id, gpu::GRID);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Boids",
            // Source: it mints its own agent stream, then simulates it.
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    // ADR-0155: this flock CONSUMES `accel`, so a `force.*` in its state chain
    // is live instead of inert — and the diagnose stops offering to splice a
    // `motion.integrate`, which is `Temporal` and would stamp `sim_t = playhead`
    // on the way past, handing this node `dt = 0` and FREEZING the flock.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("accel")],
    );
    Ok(())
}

use ph2d_node_registry::{ParamGroup, ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `count` — MEDIDO NO DEVICE** (doc 88 A1 · doc 89 W1 · §0), enquanto o slider
/// fica nos 500 que cobrem a autoria confortável por arrasto.
///
/// ⚠️ **Ele já foi `2.000`, e esse número era o da CPU** — o caso do §0.0 na letra: o teto saía do
/// `measure_the_count_ceiling`, que cozinha pelo `Cook` do registry (o caminho de REFERÊNCIA,
/// `O(N²)` all-pairs), enquanto este nó tem `register_grid` + [`gpu::GPU_KERNEL`] e a GPU shipa
/// LIGADA por default. Nos MESMOS 2.000 agentes: **CPU 10,392 ms · device 0,476 ms** — 21,8×.
/// O caminho lento só precisa **computar a mesma resposta**; quem manda no teto é o dispositivo.
///
/// **A escala do device é CONDICIONAL ao [`Params::spread`]**, então as duas tabelas mandam juntas
/// (`gpu_boids_scale::where_the_flock_leaves_the_frame_budget`, um quadro de 60 fps = 16,7 ms):
///
/// | agentes | packed (`spread` OFF, o DEFAULT) | spread ON |
/// |---|---|---|
/// | 2.000 | 0,476 ms · 3% | — |
/// | 8.000 | 2,237 ms · 13% | — |
/// | 16.384 | 5,314 ms · 32% | — |
/// | 32.768 | **13,509 ms · 81%** | — |
/// | 65.536 | 56,633 ms · 340% | 0,624 ms · 4% |
/// | 262.144 | — | 2,083 ms · 12% |
/// | **1.048.576** | — | **14,283 ms · 86%** |
///
/// Packed, a semeadura é uma caixa fixa de ~6×6 (`SEED_SPREAD`), o enxame inteiro cai em meia
/// dúzia de células e a varredura 3×3 visita ~todo mundo ⇒ **a grade não acelera nada e o device
/// também é `O(N²)`**, saindo do quadro entre 32.768 e 65.536. Com `spread`, a densidade fica
/// limitada, a varredura é `O(k)` e o tique é `O(N)`.
///
/// **O teto quota a linha de baixo** — 2²⁰ é o que o `PH2D_GPU_COOK_DEMO=7` de fato SHIPA (três
/// rodadas de smoke do Enio, medidas até o equilíbrio em 160 s), e o teto antigo tornava o número
/// do próprio demo **indigitável**: o documento continha um valor que a caixa de texto recusava.
///
/// ⚠️ **E o preço da outra coluna fica NOMEADO, não escondido:** digitar um milhão com `spread`
/// desligado pede ~256× a célula de 65 k — minutos por tique. O kernel **honra** o pedido (a
/// resposta é a certa, só é lenta), o que o mantém do lado certo da lei *um teto digitável não
/// pode passar do que o kernel honra* — é a parada ERGONÔMICA do `rate` do `motion.emitter`, e o
/// interruptor que a resolve (`spread`) está na mesma seção do painel. O que `count` **não** pode
/// exprimir é um limite sobre o PRODUTO `count × densidade`, exatamente como o `rate` do emitter
/// não exprime `rate × playhead`.
///
/// ⚠️ **O slider fica em 500 por RESOLUÇÃO, não por custo** (o device faz 8.000 em 13% de um
/// quadro): 1..500 dá ~2,5 agentes por pixel de arrasto contra um default de 48, e uma pista até
/// 8.000 tornaria a contagem comum imprecisa de autorar. Quem quer a escala DIGITA — o par
/// slider/chip do doc 88 B2.
static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "count",
    max: 1_048_576.0,
}];

/// As SEÇÕES deste nó (doc 88 B3). Nove sliders numa lista plana escondem que eles respondem
/// a três perguntas diferentes — e as três pesos clássicos do boids (separação, alinhamento,
/// coesão) são literalmente UMA.
///
/// ⚠️ O `radius` está em **Flocking** porque é o raio de PERCEPÇÃO: é ele que decide quem é
/// vizinho, e sem vizinho os três pesos não têm sobre o que agir.
///
/// ⚠️ Fica SOLTO só o `count` — o número que o artista muda o tempo todo.
static PARAM_GROUPS: &[ParamGroup] = &[
    // Quem é vizinho, e o que fazer com ele.
    ParamGroup::new("radius", "Flocking"),
    ParamGroup::new("separation", "Flocking"),
    ParamGroup::new("alignment", "Flocking"),
    ParamGroup::new("cohesion", "Flocking"),
    // Para onde o bando é levado, e quão rápido pode ir.
    ParamGroup::new("seek", "Steering"),
    ParamGroup::new("max_speed", "Steering"),
    ParamGroup::new("max_force", "Steering"),
    // Como a nuvem inicial nasce.
    ParamGroup::new("seed", "Spawn"),
    ParamGroup::new("spread", "Spawn"),
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 500.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.1,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "separation",
        label: "Separation",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "alignment",
        label: "Alignment",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cohesion",
        label: "Cohesion",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seek",
        label: "Seek",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max_speed",
        label: "Max Speed",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // ⚠️ A pista **começa em 0 porque 0 é o DESLIGADO**, não porque um orçamento
    // de zero seja útil — o piso de um slider aqui esconderia o neutro. O topo
    // acompanha o `max_speed`: as duas grandezas vivem na mesma escala de mundo
    // (uma por segundo, a outra por segundo²), e um teto muito acima dele seria
    // pista onde o clamp já não morde.
    ParamUiHint {
        param: "max_force",
        label: "Max Force",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // 0 fixed (default), 1 the √N density-bounded spread that lets the count climb into the
    // millions the GPU path can flock. **A `Toggle`, because the eval reads it as a `bool`**
    // (`ctx.param("spread") > 0.5` → the `spread: bool` field): a slider whose step equals its
    // whole range has exactly two positions, and painting it as a continuous drag tells the
    // artist there is something in between.
    ParamUiHint {
        param: "spread",
        label: "Spread √N",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
