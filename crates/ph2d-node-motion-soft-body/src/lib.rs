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
use ph2d_nodegraph::node::{
    LADO_MAX_DE_GRELHA, LoweringKind, MAX_INSTANCIAS_POR_NO, NodeManifest, NodeOp, NodeTypeId,
    ParamSpec, PortSpec,
};
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

/// ⭐ **O corpo DECLARA ao colisor quem está junto** (doc 124 §0.2): o REPOUSO de cada partícula
/// (o `rest` do Houdini) e o CORPO a que pertence (`node_key + 1`; `0` = nenhum, que é o zero-fill
/// de um `motion.combine`). O `motion.collide` não separa um par do mesmo corpo que o repouso já
/// põe a menos de `r_i + r_j` — o filtro de auto-colisão do FleX.
const REST_COL: &str = "rest";
const BODY_COL: &str = "body";
/// A posição que o corpo EMITIU e o passo que a produziu: no tique seguinte, `P − sb_pos` é o
/// quanto um nó do laço de estado o empurrou, e isso entra na velocidade ([`fold_correction`]).
const EMITTED_COL: &str = "sb_pos";
const STEP_DT_COL: &str = "sb_dt";

/// Ceiling on a single step (see `motion.integrate`): guards a playhead jump.
const MAX_DT: f32 = 0.1;
/// Below this a magnitude is treated as zero (skip the normalise / division).
const EPS: f32 = 1e-6;
/// **O lado da malha — o da CASA** (ordem do dono, 2026-10-08: *«o nó de corpo mole deve ter
/// limite de 32 768»*): cada lado vai a [`LADO_MAX_DE_GRELHA`] e a malha a `181² = 32 761`,
/// o molde do `motion.grid` (`d60a8398c`). DERIVADO, nunca um literal.
///
/// ⚠️ O tecto de antes era `512` (`262 144` partículas) e era de um RECURSO (o sub-orçamento de
/// física de `2,0 ms`); a escada medida (`512²` = `1,43 ms` uma moldura · `5,3–5,9 ms` com
/// `clusters`) vive no [doc 124 §1](../../../docs/Motion%20Nodes/124_ciclo_15_o_tecto_dos_criadores_e_o_colisor_do_corpo_mole.md).
/// Hoje o tecto é de PRODUTO e o recurso folga muito abaixo dele.
const MAX_SIDE: i64 = LADO_MAX_DE_GRELHA as i64;
const _: () = assert!(
    (MAX_SIDE * MAX_SIDE) as usize <= MAX_INSTANCIAS_POR_NO,
    "a malha no tecto cabe no tecto de instancias por no"
);

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
        // ⭐ **Sub-passos — lidos pelo SEQUENCIADOR, não pelo `eval`** (doc 124 §2): um nó que
        // oferece um param chamado `ph2d_nodegraph::cook::SUBSTEPS_PARAM` declara *«o meu interior
        // sub-tica»* e o pump re-coze o laço de estado inteiro `n` vezes por quadro — o colisor
        // no laço incluído, que é o que um laço dentro deste `eval` não podia dar. `1` = o passo
        // único de sempre, ao bit. Medido na fixtura do Bug #15: a `4` o colisor segura o
        // diâmetro (`0,555` contra `0,560`) e o corpo NÃO incha — o número do oráculo.
        ParamSpec {
            name: "substeps",
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

/// ⭐ **O CONTACTO ENTRA NA VELOCIDADE** — a do PBD, `v = (x_final − x_antes)/dt`, com o
/// `x_final` a incluir o que um nó do laço de estado fez à posição (o `motion.collide`). Sem isto
/// a velocidade do passo seguinte era a de ANTES do contacto e o corpo voltava a entrar no vizinho
/// a cada tique (Bug #15). `Δ = 0` não toca a velocidade: sem nada no laço, o corpo é o de sempre
/// AO BIT.
fn fold_correction(
    pos: &[[f32; 2]],
    vel: Vec<[f32; 2]>,
    state: &Stream,
    keep: f32,
) -> Vec<[f32; 2]> {
    let emitted = vec2_col(state, EMITTED_COL);
    let dt = scalar_col(state, STEP_DT_COL)
        .first()
        .copied()
        .unwrap_or(0.0);
    if emitted.len() != pos.len() || dt.is_nan() || dt <= EPS {
        return vel;
    }
    let k = keep / dt;
    vel.into_iter()
        .zip(pos.iter().zip(&emitted))
        .map(|(mut v, (x, e))| {
            for a in 0..2 {
                let d = x[a] - e[a];
                if d != 0.0 && d.is_finite() {
                    v[a] += d * k;
                }
            }
            v
        })
        .collect()
}

/// The whole node as a pure function: seed on the first tick / a shape change,
/// else step. Emits `P` + the `sb_vel`/`sim_t` state, the `sb_pos`/`sb_dt` the next tick
/// reads its correction from, and the `rest`/`body` the collider reads.
fn simulate(
    anchor: [f32; 2],
    state: &Stream,
    shape_in: &[[f32; 2]],
    playhead: f32,
    p: &Params,
    body: f32,
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

    let (pos, vel, emitted, dt) = if s_pos.len() == n && s_vel.len() == n {
        let t_prev = scalar_col(state, "sim_t")
            .first()
            .copied()
            .unwrap_or(playhead);
        let dt = (playhead - t_prev).clamp(0.0, MAX_DT);
        if dt < EPS {
            // Loop-wrap / same tick → hold. ⚠️ O `sb_pos`/`sb_dt` de ANTES viajam intactos: um
            // empurrão recebido num tique parado ainda entra na velocidade do próximo passo.
            let carried = vec2_col(state, EMITTED_COL);
            let emitted = if carried.len() == n {
                carried
            } else {
                s_pos.clone()
            };
            let dt_prev = scalar_col(state, STEP_DT_COL)
                .first()
                .copied()
                .unwrap_or(0.0);
            (s_pos, s_vel, emitted, dt_prev)
        } else {
            let accel = accel_col(state, n);
            let w = inv_mass_col(state, n);
            let fall = falloff_col(state, n);
            let s_vel = fold_correction(&s_pos, s_vel, state, 1.0 - p.damping);
            let (pos, vel) = step(
                &s_pos,
                &s_vel,
                &accel,
                &w,
                fall.as_deref(),
                anchor,
                &layout,
                dt,
                p,
            );
            let emitted = pos.clone();
            (pos, vel, emitted, dt)
        }
    } else {
        // Seed the rest mesh at the anchor, at rest (zero velocity).
        let seed: Vec<[f32; 2]> = rest
            .iter()
            .map(|q| [q[0] + anchor[0], q[1] + anchor[1]])
            .collect();
        (seed.clone(), vec![[0.0, 0.0]; n], seed, 0.0)
    };

    Stream::new(pos.len())
        .with("P", Column::Vec2(pos))
        .with("sb_vel", Column::Vec2(vel))
        .with("sim_t", Column::Scalar(vec![playhead; n]))
        .with(EMITTED_COL, Column::Vec2(emitted))
        .with(STEP_DT_COL, Column::Scalar(vec![dt; n]))
        .with(REST_COL, Column::Vec2(rest.clone()))
        .with(BODY_COL, Column::Scalar(vec![body; n]))
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
        // A identidade do corpo é a do NÓ (estável, nunca reusada), `+ 1` para o `0` ficar
        // «nenhum corpo». ⚠️ Exacta num `f32` até `2^24` nós num grafo.
        #[expect(clippy::cast_precision_loss, reason = "a chave de um no' e' pequena")]
        let body = (ctx.node_key() as f32) + 1.0;
        let out = simulate(anchor, state, &shape_in, playhead, &p, body);
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
            display_key: "node.motion.soft_body.name",
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

/// Os gates do tecto da malha — separados do `lib.rs` pelo tecto de LOC.
#[cfg(test)]
#[path = "cap_gates.rs"]
mod cap_gates;

#[cfg(test)]
#[path = "probes.rs"]
mod probes;
