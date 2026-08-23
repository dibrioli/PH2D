#![forbid(unsafe_code)]
//! `motion.soft_body` — a **shape-matching** deformable body: a `rows×cols` mesh of
//! particles that squashes, stretches and JIGGLES back to its rest shape, like jelly
//! (Motion Nodes M4, simulation — doc 01 §3 / doc 22). The continuum-media
//! counterpart to the discrete `motion.verlet_rope`/`motion.boids`: a 2D body that
//! deforms as a whole and always recovers its form.
//!
//! **Algorithm.** The *shape-matching constraint* is Müller et al., *Meshless
//! Deformations Based on Shape Matching* (SIGGRAPH 2005): find the single best-fit
//! frame `(M, c)` of the rest shape to the deformed cloud, whose *goal* for each
//! particle is `gᵢ = M·qᵢ + c` (`qᵢ = xᵢ⁰ − c₀`). `M` is the **rigid** rotation `R`
//! (the polar factor of `A_pq = Σ pᵢ qᵢᵀ`), optionally blended with the paper's
//! area-preserved **linear** map `A = A_pq A_qq⁻¹` by `stretch` (`M = β·A + (1−β)·R`)
//! for squash & stretch. The frame math lives in the `shape` sibling module.
//!
//! The *integration* is the **Position-Based Dynamics** reformulation of shape
//! matching (Müller et al. 2007; Matthias Müller's "Ten Minute Physics"), NOT the
//! 2005 paper's velocity-blend `v += α(g−x)/h`: predict under gravity + inertia,
//! project each particle a fraction `stiffness` toward its goal (computed from the
//! *predicted* cloud), then read velocity back as `v = (x_new − x_old)/dt`. Same
//! author lineage, and the modern-standard, unconditionally-stable scheme (the goal
//! is always a valid pose — nothing to explode; no per-edge constraint list).
//!
//! Masses are uniform here (`wᵢ = mᵢ = 1` — exact for this even grid, so the paper's
//! mass-weighted centroid/`A_pq` reduce to the plain sums). The 2D polar factor has
//! a closed form with NO trig, so `R` costs one `sqrt` (HR-5).
//!
//! ## Topology (the `pre` self-loop)
//!
//! Sequential like the other sims: each particle's `P` and `sb_vel` ride the `state`
//! feedback port, auto-plumbed as the `pre` self-loop on add. Tick 0 (`pre` = Empty)
//! **seeds** the rest mesh at the anchor; from tick 1 it steps. `dt` derives from
//! the state's own `sim_t`, clamped to `[0, MAX_DT]` (a loop-wrap freezes one tick).
//!
//! ## The anchor pins the top edge (value inputs)
//!
//! `anchor_x`/`anchor_y` are **value** inputs: with `pin` on, the top edge is held
//! rigid at the anchor (a hanging jelly flag), and sliding the anchor with a
//! `value.lfo` wobbles the whole body. Unconnected → the origin. `pin` off lets the
//! body fall freely.
//!
//! ## O corpo não é obrigado a ser um rectângulo (a porta `shape`)
//!
//! A malha `rows × cols` é o corpo **por omissão**, não a definição dele: ligar
//! um stream à porta `shape` faz a nuvem que chega ali ser a forma de repouso —
//! um disco, uma letra, um caminho vectorial amostrado, a saída de um
//! `field.shape`. É o que a referência faz (Cavalry põe um Forge Soft Body em
//! qualquer forma; o Vellum em qualquer geometria).
//!
//! O que a grelha respondia — *quem é pino*, *qual é o contorno*, *como o corpo
//! se divide em regiões* — passou a ser um facto da forma de repouso, e vive no
//! [`layout`]. ⚠️ **A malha autorada continua a dar os MESMOS bits**: ela é o seu
//! próprio fornecedor daquelas três respostas, e cada uma devolve a sequência de
//! índices que este ficheiro percorria à mão.
//!
//! Transcendental-free (HR-5): prediction, the 2×2 polar decomposition (`sqrt` only)
//! and the goal pull are arithmetic. Deterministic → `Effect::Temporal`, replays
//! bit-for-bit.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod params_ui;
use params_ui::{PARAM_GROUPS, PARAM_HARD_MAX, PARAM_HINTS, PARAM_UNITS};
mod cluster;
mod columns;
mod layout;
mod shape;
use cluster::cluster_goals_weighted;
use columns::{accel_col, falloff_col, inv_mass_col, scalar_col, value_head, vec2_col};
use layout::BodyLayout;
#[cfg(test)]
use shape::{boundary_area, rest_shape};
use shape::{pressure_scale, ring_area, shape_goals_weighted, weighted_rest_centroid};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `anchor_*` inputs (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";
/// A massa INVERSA por partícula (`motion.pin_constraint`): `1` = livre, `0` =
/// pinada. Convenção de string partilhada pelos solvers do módulo, soletrada
/// LOCALMENTE por cada leitor (como `P` / `accel`) em vez de acoplar as crates.
const INV_MASS_COL: &str = "inv_mass";
/// O peso por partícula (a espinha MOPs). Convenção de string partilhada,
/// soletrada LOCALMENTE por cada leitor — como `P` / `accel` / `inv_mass`.
const FALLOFF_COL: &str = "falloff";

/// Ceiling on a single step (see `motion.integrate`): guards a playhead jump.
const MAX_DT: f32 = 0.1;
/// Below this a magnitude is treated as zero (skip the normalise / division).
const EPS: f32 = 1e-6;
/// Grid dimensions are clamped to this many cells per side — **262 144
/// particles**, and the resource is the **HR-4 soft-physics sub-budget of
/// 2,0 ms/tick**. MEASURED (`cost_probe`, single-threaded, one tick of the
/// full `step`): 512² = **1,426 ms = 71% do orçamento** · 724² = 3,296 ms =
/// 165% (estoura). The mesh cost is `O(rows·cols)` and nothing here is
/// quadratic — shape matching reduces the whole cloud to 8 scalars.
///
/// ⚠️ **Era 40** (1600 partículas), and that number named no resource: it cost
/// **0,005 ms**, 0,25% of the budget — 164× below what the same code delivers
/// (§0.0: measure before you limit). A jelly at 40×40 has the resolution of a
/// checkerboard; this is the cap the algorithm actually earns.
///
/// ⚠️ At the cap the binding constraint is no longer the SIM — it is the
/// RENDER: 262 144 quads is exactly where the zone scene measured the frame
/// drop (2026-07-20). Reaching this number is a deliberate act, not a default.
///
/// ⚠️ **And this number was measured against ONE shape match, which `clusters`
/// stops being.** Overlapping regions cover the mesh about four times over, so
/// the match is paid roughly that many times: MEASURED
/// (`what_clusters_buy_and_what_they_cost`, one tick) 512² costs **1,77 ms with a
/// single frame and 5,3–5,9 ms clustered** — 3,4×, or 275-297% of the sub-budget
/// this cap was chosen to fit. Clustered, the budget holds to about **300 a
/// side** (256² measures 1,22-1,36 ms = 61-68%).
///
/// The cap is NOT lowered when clusters are on, and that is deliberate: a
/// resolution ceiling that moved with another knob would take a mesh the artist
/// already authored away from them, and the right value would be a function of
/// the neighbour — the shape this codebase calls an ergonomics bug. The two
/// numbers multiply, they are both on the panel, and this is where the product
/// is written down.
const MAX_SIDE: i64 = 512;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.soft_body"),
    name: "motion.soft_body",
    inputs: &[
        // The pin anchor, as two value fields (animatable). Optional: unconnected → 0.
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
        // ⚠️ **A FORMA de repouso, quando o artista tem uma.** Ligada, o corpo
        // deixa de ser o rectângulo `rows × cols` e passa a ser a nuvem que
        // chega aqui — Cavalry põe um Forge Soft Body em qualquer forma, o
        // Vellum em qualquer geometria. Desligada, `rows`/`cols`/`spacing`
        // respondem como sempre responderam, ao bit.
        //
        // ⚠️ **É a ÚLTIMA porta de propósito.** Acrescentar no fim mantém
        // `anchor_x`/`anchor_y`/`state` nos índices 0/1/2, que é o que faz os
        // grafos já autorados (e o auto-fio `out --pre--> state`) continuarem a
        // apontar para onde apontavam.
        PortSpec {
            name: "shape",
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
            name: "rows",
            default: 4.0,
        },
        ParamSpec {
            name: "cols",
            default: 4.0,
        },
        ParamSpec {
            name: "spacing",
            default: 0.7,
        },
        ParamSpec {
            name: "gravity",
            default: 9.0,
        },
        // Shape recovery ∈ [0,1]: 1 = rigid snap-back, low = slow gooey wobble.
        ParamSpec {
            name: "stiffness",
            default: 0.4,
        },
        // Linear-deformation blend β ∈ [0,1] (Müller 2005): 0 = rigid, higher =
        // more squash & stretch (area-preserved). Default 0 → pure rigid.
        ParamSpec {
            name: "stretch",
            default: 0.0,
        },
        ParamSpec {
            name: "damping",
            default: 0.03,
        },
        // 1 = pin the top row to the anchor (a hanging jelly); 0 = free fall.
        ParamSpec {
            name: "pin",
            default: 1.0,
        },
        // Internal pressure: the WEIGHT of the volume defence. 0 = off (and
        // byte-identical to the body that shipped), 1 = the goal targets exactly
        // the rest area. See `Params::pressure`.
        ParamSpec {
            name: "pressure",
            default: 0.0,
        },
        // Overlapping shape-match regions along the body's longer side (Müller
        // 2005 §4.3). 1 = the single global frame that shipped. See
        // `Params::clusters`.
        ParamSpec {
            name: "clusters",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Resolved parameters (arithmetic-ready).
struct Params {
    rows: usize,
    cols: usize,
    spacing: f32,
    gravity: f32,
    stiffness: f32,
    /// The Müller 2005 linear-deformation blend `β` ∈ [0,1]: 0 = pure rigid (holds
    /// shape), higher = more squash & stretch (area-preserved).
    beta: f32,
    damping: f32,
    /// **The gas inside the jelly** — how hard the body defends the AREA its rest
    /// shape encloses (Cavalry Forge *pressure*; Houdini Vellum *Balloon*).
    ///
    /// ⚠️ This is not reachable by any other knob, and that was MEASURED before it
    /// was built (`pressure_probe`): the goal is either the RIGID rest shape or
    /// the paper's AREA-PRESERVED linear map, so both already carry the rest area
    /// — but the cloud only travels `stiffness` of the way there each step, and
    /// the volume is lost in that lag. A body squeezed by a `force.attractor`
    /// settles at **90,8 %** of its rest area and stays there for ever; a shaken
    /// one loses **15 %**; and turning `stretch` all the way up changes that to
    /// **90,7 %**, which is to say nothing at all.
    ///
    /// `0` is off, and off is byte-identical: the shoelace never runs and the goal
    /// is the expression that shipped.
    pressure: f32,
    /// **How many overlapping regions the body is matched in** (Müller et al. 2005
    /// §4.3), counted along its longer side.
    ///
    /// ⚠️ `1` is not merely the neutral, it is the ONLY pose vocabulary the node
    /// had: one best-fit frame can translate, rotate and (with `stretch`) shear
    /// the rest shape uniformly, and none of those bends anything. MEASURED
    /// (`how_much_can_a_long_body_bend_today`): a 32×4 body's spine deviates from
    /// a straight line by **0,0000** of its own length — at every stiffness, and
    /// with the linear mode fully on. The plate is not a stiffness setting, it is
    /// the shape of the model.
    ///
    /// Composing several `motion.soft_body` nodes does not reach this: they would
    /// be independent bodies, and a cluster exists precisely because neighbouring
    /// regions SHARE particles whose averaged goal drags the two frames into
    /// agreement. Without a shared particle there is no seam to carry a curve.
    clusters: usize,
    pin: bool,
}

impl Params {
    /// Whether particle `i` is pinned — the TOP EDGE of the rest shape, when
    /// `pin`.
    ///
    /// ⚠️ **Deixou de ser `i < cols` e isso é a mesma resposta com outra
    /// pergunta:** a linha de topo de uma grelha É o conjunto de `y` máximo, e a
    /// [`BodyLayout`] devolve exactamente `0..cols` para toda malha autorada. O
    /// índice era um atalho que só a grelha podia percorrer; o facto — *a
    /// aresta de cima* — é o que uma nuvem qualquer também tem.
    fn is_pinned(&self, layout: &BodyLayout, i: usize) -> bool {
        self.pin && layout.is_pinned(i)
    }
}

/// The pinned target for the top-row particle at grid column `c`: the anchor plus
/// that particle's rest offset, so the top edge stays a rigid bar sliding with the
/// anchor.
fn pin_target(anchor: [f32; 2], rest: &[[f32; 2]], i: usize) -> [f32; 2] {
    [anchor[0] + rest[i][0], anchor[1] + rest[i][1]]
}

/// One shape-matching step as a pure function. `pos`/`vel` are this tick's entry
/// state; returns the new `(pos, vel)`.
#[allow(clippy::too_many_arguments)]
fn step(
    pos: &[[f32; 2]],
    vel: &[[f32; 2]],
    accel: &[[f32; 2]],
    inv_mass: &[f32],
    falloff: Option<&[f32]>,
    anchor: [f32; 2],
    layout: &BodyLayout,
    dt: f32,
    p: &Params,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let rest = &layout.rest;
    let n = pos.len();
    let dt2 = dt * dt;
    // Predict under inertia + gravity; pinned particles jump to their pin target.
    let mut pred = vec![[0.0f32; 2]; n];
    for i in 0..n {
        let w = inv_mass.get(i).copied().unwrap_or(1.0);
        // ⚠️ **UMA lei para as duas espécies de pino, e a correção é de 2026-08-16.**
        // O intrínseco (a linha de topo, o param `pin`) sempre foi clampado ao alvo
        // `âncora + repouso`; o genérico — a massa infinita que um
        // `motion.pin_constraint` na cadeia de estado escreve — segurava **onde
        // estava**, e eu escrevi isso como decisão (*"não há alvo nenhum a que o
        // prender"*). **Medido, era um beco sem saída:** arrastar a âncora movia a
        // linha intrínseca **3,0000** e a genérica **0,0000**, com o corpo inteiro
        // preso em 0,0020 — a bandeira num mastro que se move era inexprimível, e
        // mexer no `spacing` deixava a linha pinada na largura antiga (o report do
        // Enio no smoke da cena `=50`).
        //
        // A lei que fica é a da referência (Blender Cloth segue a malha original ·
        // Vellum `pintoanimation` segue a geometria animada · nCloth *constrain to
        // transform*): **uma partícula de massa infinita segue a pose que o nó sabe
        // PRESCREVER para ela.** Este nó sabe — é a malha de repouso ancorada, e é
        // função de `anchor`/`rows`/`cols`/`spacing`, todos vivos. A corda e o bando
        // **não** sabem (uma corda tem restrições de distância, não uma forma de
        // repouso posicional), e é por isso que ali `pos[i]` continua a ser a única
        // resposta — a mesma lei, com o nó a decidir se tem prescrição.
        //
        // ⚠️ O preço, nomeado: pinar uma partícula de um corpo JÁ ASSENTADO a
        // clampa ao repouso em vez de a congelar onde ela está. É exactamente o que
        // o pino intrínseco sempre cobrou, e quem quer uma partícula meramente mais
        // pesada tem o `strength < 1` (que não entra neste ramo).
        if p.is_pinned(layout, i) || w <= 0.0 {
            pred[i] = pin_target(anchor, rest, i);
        } else {
            // The external `accel` lands beside the built-in gravity: both are
            // accelerations, and a prediction takes one as `a·dt²`. A pinned
            // particle is unmoved by it ON PURPOSE — a pin is a constraint, and
            // a force that could drag the pin would make the pin a suggestion.
            let a = accel.get(i).copied().unwrap_or([0.0, 0.0]);
            // A massa inversa escala a ACELERAÇÃO e não a inércia (`vel · dt`), o
            // espelho exacto do que a corda faz: `a = F/m` com o `gravity` lido como
            // força por massa de referência. Peso `1` é byte-idêntico.
            pred[i] = [
                pos[i][0] + vel[i][0] * dt + a[0] * dt2 * w,
                pos[i][1] + vel[i][1] * dt - p.gravity * dt * dt * w + a[1] * dt2 * w,
            ];
        }
    }
    // Match the rest shape and pull each particle a fraction toward its goal.
    //
    // The pressure term rides INTO the goal rather than beside it: a soft body
    // that defends its volume is one whose goal is bigger than its rest shape
    // while it is squashed, and everything downstream (the `stiffness` pull, the
    // pin, the velocity read-back, the NaN guard) then applies to it unchanged.
    // A separate outward push after the projection would be a second author of
    // the same positions, and the two would have to agree about the centroid.
    //
    // ⚠️ The `> 0.0` guard is COST and robustness, not correctness, and a mutation
    // says so: forcing the term to run at zero gain leaves every gate green,
    // because `0 · finite` is exactly `0` and `x · 1.0` is exactly `x` in
    // IEEE-754 — the answer is already the answer that shipped. What the guard
    // buys is the boundary shoelace nobody asked for, and a neutral that stays
    // structural if a future edit gives `pressure_scale` a term the gain does not
    // multiply. Documented instead of gated: a ~1% cost at the node's own cap is
    // not a number a ratio test can resolve honestly.
    let scale = if p.pressure > 0.0 {
        pressure_scale(
            &pred,
            layout.ring(),
            ring_area(rest, layout.ring()),
            p.pressure,
            p.stiffness,
        )
    } else {
        1.0
    };
    // ⚠️ **O peso entra no AJUSTE e no PUXÃO, e é UMA ideia com duas
    // consequências:** o peso diz *quanto esta partícula PERTENCE ao corpo*, e
    // pertencer é (a) definir a forma e (b) ser puxado de volta a ela. Peso zero
    // ⇒ a partícula não define o quadro nem é puxada por ele: uma partícula
    // LIVRE. Escrever só a metade (b) deixaria uma partícula que se solta ainda
    // a arrastar o ajuste do corpo inteiro atrás dela.
    let goals = if p.clusters > 1 {
        cluster_goals_weighted(
            &pred,
            rest,
            &layout.buckets(p.clusters),
            p.beta,
            scale,
            falloff,
        )
    } else {
        let c0 = weighted_rest_centroid(rest, falloff);
        shape_goals_weighted(&pred, rest, p.beta, scale, falloff, c0)
    };
    let mut out_pos = vec![[0.0f32; 2]; n];
    let mut out_vel = vec![[0.0f32; 2]; n];
    let keep = 1.0 - p.damping;
    for i in 0..n {
        // O puxão para o goal é distribuído pela massa inversa, exactamente como
        // o `motion.integrate` escala a velocidade por ela: peso `1` é o corpo de
        // hoje **ao bit** (`x · 1.0` é exacto em IEEE-754) e peso `0` deixa a
        // partícula onde a predição a pôs — que, para os DOIS pinos, é o alvo
        // `âncora + repouso`.
        //
        // ⚠️ O ramo é `pull <= 0.0` e não `is_pinned`, e não é higiene: com peso
        // zero a outra expressão vale `x + (g − x)·0`, que é `x` para todo `g`
        // FINITO e **NaN** para um `g` infinito. O ramo é o que impede um goal
        // degenerado de envenenar uma partícula que, por definição, nada move.
        let pull = p.stiffness
            * inv_mass.get(i).copied().unwrap_or(1.0)
            * falloff.map_or(1.0, |f| f.get(i).copied().unwrap_or(1.0));
        let mut np = if p.is_pinned(layout, i) || pull <= 0.0 {
            pred[i] // pinned particles stay exactly on the pin
        } else {
            [
                pred[i][0] + (goals[i][0] - pred[i][0]) * pull,
                pred[i][1] + (goals[i][1] - pred[i][1]) * pull,
            ]
        };
        // Velocity from the position change (this is what jiggles).
        let mut nv = [
            (np[0] - pos[i][0]) / dt * keep,
            (np[1] - pos[i][1]) / dt * keep,
        ];
        if !(np[0].is_finite() && np[1].is_finite() && nv[0].is_finite() && nv[1].is_finite()) {
            np = pin_target(anchor, rest, i); // NaN guard: recover on the rest frame
            nv = [0.0, 0.0];
        }
        out_pos[i] = np;
        out_vel[i] = nv;
    }
    (out_pos, out_vel)
}

/// The whole node as a pure function: seed on the first tick / a shape change,
/// else step. Emits `P` + the `sb_vel`/`sim_t` state.
fn simulate(
    anchor: [f32; 2],
    state: &Stream,
    shape_in: &[[f32; 2]],
    playhead: f32,
    p: &Params,
) -> Stream {
    // ⚠️ **A porta GANHA quando tem alguma coisa.** Vazia (desligada, ou um
    // montante que ainda não cozinhou) cai na malha autorada — e cair para o
    // rectângulo é a resposta certa: um corpo que desaparecesse porque o fio de
    // cima ainda não produziu nada seria um corpo que pisca.
    let layout = if shape_in.len() >= 3 {
        BodyLayout::from_cloud(shape_in)
    } else {
        BodyLayout::from_grid(p.rows, p.cols, p.spacing)
    };
    let rest = &layout.rest;
    let n = layout.len();
    let s_pos = vec2_col(state, "P");
    let s_vel = vec2_col(state, "sb_vel");

    let (pos, vel) = if s_pos.len() == n && s_vel.len() == n {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        if dt < EPS {
            (s_pos, s_vel) // loop-wrap / same tick → hold
        } else {
            let accel = accel_col(state, n);
            let w = inv_mass_col(state, n);
            let fall = falloff_col(state, n);
            step(
                &s_pos,
                &s_vel,
                &accel,
                &w,
                fall.as_deref(),
                anchor,
                &layout,
                dt,
                p,
            )
        }
    } else {
        // Seed the rest mesh at the anchor, at rest (zero velocity).
        let seed: Vec<[f32; 2]> = rest
            .iter()
            .map(|q| [q[0] + anchor[0], q[1] + anchor[1]])
            .collect();
        (seed, vec![[0.0, 0.0]; n])
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("sb_vel", Column::Vec2(vel))
        .with("sim_t", Column::Scalar(vec![playhead; n]))
}

struct MotionSoftBody;

impl NodeOp for MotionSoftBody {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let side = |name: &str| (ctx.param(name).round() as i64).clamp(2, MAX_SIDE) as usize;
        let p = Params {
            rows: side("rows"),
            cols: side("cols"),
            spacing: ctx.param("spacing").max(1e-3),
            gravity: ctx.param("gravity"),
            stiffness: ctx.param("stiffness").clamp(0.0, 1.0),
            beta: ctx.param("stretch").clamp(0.0, 1.0),
            damping: ctx.param("damping").clamp(0.0, 0.99),
            pressure: ctx.param("pressure").max(0.0),
            clusters: (ctx.param("clusters").round() as i64).clamp(1, MAX_SIDE) as usize,
            pin: ctx.param("pin") >= 0.5,
        };
        let playhead = ctx.playhead() as f32;
        let anchor = [
            value_head(&scalar_col(ctx.input(0), VALUE_COL)),
            value_head(&scalar_col(ctx.input(1), VALUE_COL)),
        ];
        let state = ctx.input(2);
        let shape_in = vec2_col(ctx.input(3), "P");
        let out = simulate(anchor, state, &shape_in, playhead, &p);
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSoftBody))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Soft Body",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    // ADR-0155: this body CONSUMES `accel`, so a `force.*` in its state chain is
    // live instead of inert — and the diagnose stops offering to splice a
    // `motion.integrate`, which is `Temporal` and would stamp `sim_t = playhead`
    // on the way past, handing this node `dt = 0` and FREEZING it.
    reg.register_couplings(
        MANIFEST.id,
        &[
            ph2d_node_registry::Coupling::Consumes("accel"),
            ph2d_node_registry::Coupling::Consumes("inv_mass"),
            ph2d_node_registry::Coupling::Consumes("falloff"),
        ],
    );
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

/// Os gates da porta `shape` — separados do `tests.rs` pelo tecto de LOC.
#[cfg(test)]
#[path = "port_tests.rs"]
mod port_tests;

#[cfg(test)]
mod cap_gates {
    use super::shape::{rest_shape, shape_goals};
    use super::{MAX_SIDE, PARAM_HARD_MAX, PARAM_HINTS};

    /// **A CAPACIDADE tem de ser alcançável, e o SLIDER tem de ser arrastável** — duas
    /// perguntas, e este gate afirmava as duas com um número só.
    ///
    /// A preocupação original está inteira e continua gateada: *um controle que para em 40
    /// sobre um clamp de 512 tem o terço de cima morto, e nada mais notaria*. O que mudou é
    /// **de que lado do par soft/hard essa igualdade mora**: com o slider preso ao clamp, um
    /// track de ~154 px movia **3,3 linhas por pixel** sobre um default de 4 — o `MAX_SIDE`
    /// era alcançável e o DEFAULT não (doc 88 §11). Agora o teto DIGITÁVEL é o clamp (nada
    /// se perdeu) e o curso do dedo é uma faixa de autoria estritamente abaixo dele.
    #[test]
    fn the_mesh_sliders_reach_exactly_the_clamp() {
        for param in ["rows", "cols"] {
            let hint = PARAM_HINTS
                .iter()
                .find(|h| h.param == param)
                .unwrap_or_else(|| panic!("{param} has a hint"));
            let hard = PARAM_HARD_MAX
                .iter()
                .find(|h| h.param == param)
                .unwrap_or_else(|| panic!("{param} has a hard max"));
            assert_eq!(
                hard.max, MAX_SIDE as f32,
                "o teto digitavel de {param} tem de alcancar o clamp, e parar nele"
            );
            assert!(
                hint.max < hard.max,
                "o slider de {param} e a FAIXA DE AUTORIA, nao o teto: \
                 soft {} deveria ficar abaixo do hard {}",
                hint.max,
                hard.max
            );
        }
    }

    /// **The cap's whole justification is that the cost is LINEAR** — shape
    /// matching reduces the cloud to 8 scalars, so there is no `N²` anywhere.
    /// Asserted as a RATIO, never wall-clock: `ci-test` builds at `opt-level=1`
    /// and a millisecond bar would measure the PROFILE (the ADR-0124 lesson).
    /// Someone who made this quadratic would take the cap from 71% of the HR-4
    /// budget to 45× it, and every unit test would stay green.
    #[test]
    fn the_shape_match_is_linear_in_the_mesh() {
        let cost = |side: usize| {
            let rest = rest_shape(side, side, 1.0);
            let pred: Vec<[f32; 2]> = rest.iter().map(|p| [p[0] * 1.1, p[1] * 0.9]).collect();
            let t0 = std::time::Instant::now();
            for _ in 0..5 {
                std::hint::black_box(shape_goals(&pred, &rest, 0.3, 1.0));
            }
            t0.elapsed().as_secs_f64()
        };
        let small = cost(128);
        let big = cost(256); // 4x the particles
        let ratio = big / small.max(1e-9);
        assert!(
            ratio <= 8.0,
            "4x the mesh must cost ~4x, not {ratio:.1}x — the cap assumes linear"
        );
    }
}

#[cfg(test)]
#[path = "probes.rs"]
mod probes;
