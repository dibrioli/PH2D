#![forbid(unsafe_code)]
//! `motion.collide` — **push apart**: relax a layout so no two instances overlap, the
//! Cinema 4D "Push Apart Effector" / a circle-packing separation (Motion Nodes M3,
//! distributions — doc 01 §3 / doc 26). Distinct from `motion.voronoi` (which spreads
//! points to *uniform density* via Lloyd/CVT): this enforces a *hard radius* — every
//! instance is a disc of radius `radius`, and overlapping pairs are pushed off each
//! other until they merely touch.
//!
//! **Algorithm — the Position Based Dynamics non-penetration contact constraint**
//! (Müller et al., *Position Based Dynamics*, 2007; the relaxation is Jakobsen,
//! *Advanced Character Physics*, 2001). For each pair closer than `2·radius`, the
//! constraint gradient is the contact normal (the unit vector between them) and the
//! correction is half the penetration each, moved apart along that normal — so the pair
//! ends up touching with their midpoint preserved. A **pure relaxation of the input each
//! cook** (no state, like the Voronoi's Lloyd), so a `radius` value input that breathes
//! makes the packing expand and contract — deterministic and replay-safe (HR-5:
//! arithmetic + `sqrt`, no trig). `Effect::Pure`.
//!
//! ## The sweep is AVERAGED JACOBI, not Gauss–Seidel (ADR-0140 Fase 5)
//!
//! Each `iterations` sweep reads ONE snapshot of the positions, accumulates every
//! contact's requested correction per disc, and then applies the **average** of what
//! that disc's contacts asked for (mass splitting — Macklin & Müller, *Unified Particle
//! Physics*, 2014, which is what FleX ships). Averaging is what makes Jacobi stable:
//! summing raw would launch a disc with many contacts across the scene, because every
//! neighbour independently asks for the full push.
//!
//! It replaced an in-place Gauss–Seidel sweep, and the reason is **correctness before
//! speed**: Gauss–Seidel mutates `q[i]`/`q[j]` inside the pair loop, so each pair sees
//! the corrections of pairs already visited — which makes the result depend on the
//! **index order of the stream**. Measured on a crowded cloud of 256 discs, the same
//! SET of points merely listed in a different order packed up to **6.11 world units**
//! apart (1018 % of a disc diameter); the artist neither controls nor sees that order.
//! Averaged Jacobi is order-independent (measured: **0.0**), and on the shipped default
//! of 8 iterations it also packs BETTER (min gap 0.270 vs 0.050 of the required
//! `2·radius`; Gauss–Seidel only overtakes past ~32 iterations on a pathological cloud).
//! It is additionally the scheme a GPU can run at all — every thread reads the same
//! snapshot — which is what lets the spatial-grid port exist.
//!
//! O(n²·iterations) here; the device path (spatial hash on the GPU) is ADR-0140.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod gpu;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `spread` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";
/// The inverse-mass column (PBD's `w = 1/m`) that `motion.pin_constraint` writes:
/// `1` = free (the default when absent — every pre-pin packing is unchanged),
/// `0` = pinned. A string convention shared by the module's solvers, spelled
/// locally by each reader (like `P` / `falloff`) rather than coupling the crates.
const INV_MASS_COL: &str = "inv_mass";

/// Below this a pair is treated as coincident (the normal is undefined).
const EPS: f32 = 1e-9;
/// A hard cap on the relaxation sweeps.
const MAX_ITERATIONS: i64 = 64;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.collide"),
    name: "motion.collide",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // A multiplier on `radius` (animatable): unconnected reads as 1. A `value.lfo`
        // makes the packing breathe.
        PortSpec {
            name: "spread",
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
        // The disc radius: pairs closer than 2·radius are pushed apart.
        ParamSpec {
            name: "radius",
            default: 0.3,
        },
        // Averaged-Jacobi sweeps over all pairs (more = tighter packing).
        ParamSpec {
            name: "iterations",
            default: 8.0,
        },
        // Relaxation factor per sweep (1 = full correction; <1 softens/settles).
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The `spread` multiplier: unconnected (empty) → 1.0; else the first element.
///
/// ⚠️ **É GLOBAL de propósito, e o device lê a MESMA linha** (`read_spread_v(0u)`
/// no WGSL, não `read_spread_v(i)`). A porta é de VALOR — logo, por tipo, um campo
/// por-instância —, mas o que este nó quer dela é a *respiração* animável: um
/// multiplicador só para todos os discos.
///
/// A simetria não é zelo: `ColumnAccess::ReadBroadcast` só faz broadcast quando a
/// porta traz **um** valor, e com N ela devolve `in[i]`. Enquanto o WGSL lia por
/// elemento, alimentar a porta com um campo variável dava DOIS desenhos — medido,
/// a CPU devolvia a grade intocada (o `vals[0]` era 0 ⇒ raio 0 ⇒ identidade) e o
/// device empurrava: `0,165` de divergência de posição, com todos os gates verdes
/// porque as fixtures alimentavam comprimento 1.
///
/// **O raio POR ELEMENTO é capacidade real e desejada** (o `pscale` do Houdini POP
/// Interact) e **não é isto**: a lei honesta é `r_i + r_j` — simétrica, e
/// byte-idêntica à de hoje sob spread uniforme —, que na grade do device exige
/// limitar o alcance pelo raio MÁXIMO, ou seja um `Max` reduce sobre a coluna.
/// Nenhum kernel do repo combina `register_grid` com `reduces()` hoje, então é
/// wave própria (plano 89 §10.2, W1-B), não uma linha.
fn spread_amount(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(1.0)
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Push apart the discs so no two are closer than `2·radius`, sweeping every pair
/// `iterations` times. Returns the relaxed positions. A pure function — the whole
/// node.
///
/// `w` is the per-element inverse mass (PBD's `w = 1/m`, written by
/// `motion.pin_constraint`; all-`1` when no pin is wired). The contact correction
/// is split between the pair **in proportion to their `w`s**, which is the
/// constraint-projection rule of Müller et al. 2007 — with two free elements each
/// takes half (the midpoint of the pair is preserved, bit-for-bit as before the
/// pin existed), and against a pinned element (`w = 0`, infinite mass) the free one
/// takes the whole penetration and the pin does not budge. That is what makes a
/// pinned disc an OBSTACLE the others pack around.
fn push_apart(
    p: &[[f32; 2]],
    w: &[f32],
    radius: f32,
    iterations: usize,
    strength: f32,
) -> Vec<[f32; 2]> {
    let n = p.len();
    let mut q = p.to_vec();
    let min_dist = 2.0 * radius;
    if n < 2 || min_dist <= 0.0 || strength <= 0.0 {
        return q;
    }
    let min_d2 = min_dist * min_dist;
    // Averaged-Jacobi scratch, allocated ONCE: the summed correction each disc is
    // asked for this sweep, and how many contacts asked.
    let mut delta = vec![[0.0f32; 2]; n];
    let mut contacts = vec![0u32; n];
    for _ in 0..iterations {
        delta.fill([0.0, 0.0]);
        contacts.fill(0);
        // ── gather pass: every pair reads the SAME snapshot `q` ──
        for i in 0..n {
            for j in (i + 1)..n {
                // Two immovable discs (or two infinitely heavy ones) have no
                // correction to share — the constraint simply cannot be met.
                let sum_w = w[i] + w[j];
                if sum_w <= 0.0 {
                    continue;
                }
                let dx = q[j][0] - q[i][0];
                let dy = q[j][1] - q[i][1];
                let d2 = dx * dx + dy * dy;
                if d2 >= min_d2 {
                    continue;
                }
                let (nx, ny, penetration) = if d2 > EPS {
                    let d = d2.sqrt();
                    (dx / d, dy / d, min_dist - d)
                } else {
                    // Coincident: split along a deterministic axis (x for even i+j,
                    // y otherwise) so the relaxation stays replay-stable (HR-5, no rng).
                    // ⚠️ This is the ONE order-dependent corner left, and it is
                    // irreducible: two EXACTLY coincident points carry nothing
                    // intrinsic to break the symmetry with, so the index is the only
                    // handle there is. Measure-zero — any jitter escapes it.
                    if (i + j) % 2 == 0 {
                        (1.0, 0.0, min_dist)
                    } else {
                        (0.0, 1.0, min_dist)
                    }
                };
                // Each disc is ASKED to move its SHARE of the penetration:
                // w_i / (w_i + w_j). Both free = half each, so the pair's midpoint is
                // preserved exactly as before.
                let push_i = penetration * (w[i] / sum_w) * strength;
                let push_j = penetration * (w[j] / sum_w) * strength;
                delta[i][0] -= nx * push_i;
                delta[i][1] -= ny * push_i;
                delta[j][0] += nx * push_j;
                delta[j][1] += ny * push_j;
                contacts[i] += 1;
                contacts[j] += 1;
            }
        }
        // ── apply pass: each disc takes the AVERAGE of what its contacts asked ──
        // Averaging (mass splitting, Macklin & Müller 2014) is what makes Jacobi
        // stable: summing raw would launch a disc with many contacts across the
        // scene, because every neighbour independently asks for the full push.
        for i in 0..n {
            let c = contacts[i];
            if c > 0 {
                let inv = 1.0 / c as f32;
                q[i][0] += delta[i][0] * inv;
                q[i][1] += delta[i][1] * inv;
            }
        }
    }
    q
}

/// The inverse-mass column (`motion.pin_constraint`), widened to `n` and made
/// safe: absent reads as free (`1`), and a negative or non-finite weight from a
/// hand-edited document reads as pinned (`0`) rather than INVERTING the push.
fn inv_mass(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(INV_MASS_COL) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

struct MotionCollide;

impl NodeOp for MotionCollide {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let base_radius = ctx.param("radius");
        let iterations = (ctx.param("iterations").round() as i64).clamp(0, MAX_ITERATIONS) as usize;
        let strength = ctx.param("strength");
        let spread = spread_amount(&scalar_col(ctx.input(1), VALUE_COL));
        let radius = base_radius * spread;
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let w = inv_mass(input, n);
        let out_p = push_apart(&p, &w, radius, iterations, strength);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "P" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("P", Column::Vec2(out_p));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionCollide))?;
    // ADR-0155: the push-apart weights by `inv_mass` — another solver a pin can feed.
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("inv_mass")],
    );
    // GPU/M5 (ADR-0140 Fase 5): the push-apart on the device via the spatial grid,
    // swept `iterations` times. Only expressible because the reference became
    // averaged Jacobi — an in-place Gauss-Seidel sweep is sequential by definition.
    reg.register_gpu_kernel(MANIFEST.id, gpu::GPU_KERNEL);
    reg.register_grid(MANIFEST.id, gpu::GRID);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Collide",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 5.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "iterations",
        label: "Iterations",
        min: 0.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// **OS TETOS DIGITÁVEIS, MEDIDOS** (doc 88 B2 · doc 89 folha 03 linha 63 · CLAUDE.md §0).
///
/// Sonda: `measure_collide_ceiling`. Os três params deste nó quebram por mecanismos
/// **diferentes**, e um deles não quebra — o que também é um resultado.
///
/// **`iterations` = 64, e o teto É o clamp.** O `eval` já faz
/// `.clamp(0, MAX_ITERATIONS)`, então hoje a caixa de texto **aceita e mente**: a cicatriz do
/// `lattice` 400 e do `kaleidoscope` 256. Medido — folga mínima da nuvem apertada:
///
/// | iterations | 8 | 32 | **64** | 65 | 200 | 100.000 |
/// |---|---|---|---|---|---|---|
/// | folga | 0,2900 | 0,4346 | **0,5183** | 0,5183 | 0,5183 | 0,5183 |
///
/// ⚠️ As três últimas colunas não são *parecidas* com a de 64: são **byte a byte** a de 64.
/// Um número que o artista digita e o kernel joga fora é um controle que mente em silêncio.
///
/// **`strength` = 3,0 — e aqui a faixa confortável tem MESMO folga**, ao contrário dos irmãos
/// acima (`iterations`) e do `friction` do `motion.spring`, onde o slider já sentava no teto. O
/// slider para em `1,0` (*"1 = correção inteira"*), e a sobre-relaxação acima disso **compra
/// empacotamento de verdade**: a 64 varreduras a folga vai de `0,5183` para `0,5995`, **+16%
/// mais apertado pelo mesmo custo**. Medido:
///
/// | strength | 1,0 | 2,0 | 3,0 | 3,4 | 3,6 | **3,8** | 4,5 | 6,0 | 16,0 |
/// |---|---|---|---|---|---|---|---|---|---|
/// | folga | 0,5183 | 0,5883 | 0,5995 | 0,6004 | 0,6000 | 0,6052 | 0,6008 | 0,6039 | 0,6922 |
/// | raio/semeadura | 2,38 | 2,44 | 2,47 | 2,49 | 2,47 | **3,54** | 2,73 | **6,17** | **12,28** |
///
/// Duas coisas acontecem, e o teto fica onde as duas ainda são boas: **o ganho ESTAGNA em 3,0**
/// (3,2 e 3,4 acrescentam 0,0002 e 0,0009 — nada) e a partir de **3,8 a nuvem começa a ser
/// ATIRADA** (a extensão salta da banda estável 2,4-2,5 para 3,54, depois 6,17 e 12,28).
///
/// ⚠️ **E a derivação óbvia estava ERRADA — o par isolado a refutou.** O raciocínio natural é
/// *"sobre-relaxação acima de 2 faz o par atravessar um pelo outro e oscilar"*, o limite
/// clássico do SOR. Medido em DOIS discos sozinhos, a folga é monotônica em `strength`
/// (0,6000 · 0,9500 · 3,0500 em 1,0 · 2,0 · 8,0): **não há oscilação nenhuma**, porque a
/// restrição é de **um lado só** — o laço só empurra quando há penetração, então depois do
/// overshoot não sobra sobreposição e a varredura seguinte não faz nada. O que degrada é o
/// caso DENSO, onde cada disco recebe correção de vários vizinhos no mesmo sweep.
///
/// ⚠️ **`radius` NÃO GANHA TETO, e isso é uma medição, não um esquecimento.** Este nó só sabe
/// afastar discos, então dobrar o raio dobra tudo — a fração adimensional `folga / 2·raio` é
/// **0,3317 constante de `r = 1` a `r = 1e15`**, quinze ordens de grandeza sem uma casa
/// decimal se mover. Não existe ponto em que o kernel deixe de honrar o número, e escrever um
/// teto aqui seria o palpite que o §0 proíbe. O custo tem um cliff no DEVICE (a célula da
/// grade **é** o raio — `GridSpec { cell_param: "radius" }` —, então um raio grande demais
/// para a nuvem colapsa toda gente numa célula e a varredura 3×3 vira `O(N²)`, o mecanismo que
/// o `spread` do `motion.boids` documenta), e esse número quer a sonda de device, não esta.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "iterations",
        max: MAX_ITERATIONS as f32,
    },
    ParamHardMax {
        param: "strength",
        max: 3.0,
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
    param: "radius",
    unit: ParamUnit::Length,
}];

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
