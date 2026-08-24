#![forbid(unsafe_code)]
//! `motion.lattice` — a **hexagonal (triangular) lattice** distribution: `rows×cols`
//! points on the densest regular packing, every point equidistant from its six
//! neighbours (Motion Nodes M3, distributions — doc 01 §3 / doc 23). The crystalline
//! counterpart to `motion.grid` (square) and the ordered opposite of the blue-noise
//! `motion.scatter`. Honeycombs, bubble rafts, close-packed circles.
//!
//! **Algorithm — the triangular lattice, the 2D densest circle packing.** Even rows
//! sit on a square pitch `spacing`; odd rows are shifted half a cell and the row
//! pitch is `spacing·√3/2`, so every nearest-neighbour distance equals `spacing`
//! exactly (equilateral triangles / regular hexagons). A `jitter` value input melts
//! the lattice toward white noise: each point is displaced by a hashed offset scaled
//! by `jitter` (world units), so a `value.lfo` makes the honeycomb shimmer and reform.
//!
//! A **Source** node (no stream input, mints `P`). Stateless (Jarzynski/Olano): the
//! jitter is a pure hash of `(seed, index)`, so the layout reproduces bit-for-bit.
//! `Effect::Pure` (no clock — animation arrives through the `jitter` input).
//! Transcendental-free (HR-5): the `√3/2` row pitch is a constant, the jitter is the
//! splitmix hash; no calls.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
// ⚠️ **`Region`, e não `Domain`** — o `Domain` do `port` diz em que PLANO a porta vive.
use ph2d_motion_region::carve;

mod hash;
use hash::hash3;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `jitter` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// `√3/2` — the equilateral row pitch (a constant, not a call; HR-5).
const ROW_PITCH: f32 = 0.866_025_4;
/// Grid side clamp (cost is O(rows·cols)).
const MAX_SIDE: i64 = 400;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.lattice"),
    name: "motion.lattice",
    inputs: &[
        // Displacement scale toward white noise (animatable). Optional: unconnected
        // reads as 0 → a perfect lattice.
        PortSpec {
            name: "jitter",
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
        ParamSpec {
            name: "rows",
            default: 6.0,
        },
        ParamSpec {
            name: "cols",
            default: 7.0,
        },
        ParamSpec {
            name: "spacing",
            default: 0.7,
        },
        // **A FORMA** (doc 89, folha 01) — `Rect` é a colmeia de sempre, ao bit. A
        // rede triangular não se dobra para caber num círculo, então a forma RECORTA:
        // a contagem cai, e é suposto cair. Ver `ph2d_motion_region`.
        ParamSpec {
            name: ph2d_motion_region::SHAPE,
            default: 0.0,
        },
        ParamSpec {
            name: ph2d_motion_region::INNER,
            default: 0.5,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// O `jitter` **do ponto `i`** — desligado (vazio) → `0.0`; um valor → todos; N → o dele.
///
/// ⚠️ **Era `v.first()`, a mesma forma que desligava o `motion.spline_wrap` em silêncio**
/// (doc 90 §5). A porta é um CAMPO, e o elemento 0 de uma rampa é `0.0` ⇒ ligar-lhe o gesto
/// óbvio dava jitter **zero à treliça inteira**. Aqui o índice já existia: o gerador computa
/// `i = r * cols + c` para semear o hash, e é o mesmo `i`.
///
/// ⚠️ A lei é a do irmão (`0 → neutro · 1 → broadcast · N → por-elemento`), e o `1 → broadcast`
/// mantém byte-idêntica toda cena que já existe.
fn jitter_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 0.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(0.0),
    }
}

/// Lay out the hexagonal lattice (centred on the origin), each point displaced by a
/// hashed offset scaled by `jitter`.
fn lattice(rows: usize, cols: usize, spacing: f32, seed: u32, jitter: &[f32]) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(rows * cols);
    // Half-extents for centring: odd rows reach half a cell further in x.
    let half_w = ((cols as f32 - 1.0) * spacing + spacing * 0.5) * 0.5;
    let half_h = (rows as f32 - 1.0) * spacing * ROW_PITCH * 0.5;
    for r in 0..rows {
        let row_shift = if r % 2 == 1 { spacing * 0.5 } else { 0.0 };
        for c in 0..cols {
            let i = (r * cols + c) as u32;
            let j = jitter_at(jitter, i as usize);
            let jx = (hash3(seed, i, 0) - 0.5) * 2.0 * j;
            let jy = (hash3(seed, i, 1) - 0.5) * 2.0 * j;
            out.push([
                c as f32 * spacing + row_shift - half_w + jx,
                r as f32 * spacing * ROW_PITCH - half_h + jy,
            ]);
        }
    }
    out
}

struct MotionLattice;

impl NodeOp for MotionLattice {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let side = |name: &str| (ctx.param(name).round() as i64).clamp(1, MAX_SIDE) as usize;
        let rows = side("rows");
        let cols = side("cols");
        let spacing = ctx.param("spacing").max(1e-3);
        let seed = ctx.param("seed").max(0.0).round() as u32;
        let jitter = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let positions = lattice(rows, cols, spacing, seed, &jitter);
        // ⚠️ **A extensão é a que a `lattice` de facto ocupa** — a largura leva a
        // meia célula extra que o desencontro das fileiras ímpares acrescenta, e a
        // altura vai no passo `ROW_PITCH`. Uma caixa «rows × cols × spacing» daria um
        // círculo que não encosta na colmeia.
        let positions = carve(
            positions,
            &ph2d_motion_region::Region::of(
                ctx.param(ph2d_motion_region::SHAPE),
                (cols as f32 - 1.0) * spacing + spacing * 0.5,
                (rows as f32 - 1.0) * spacing * ROW_PITCH,
                ctx.param(ph2d_motion_region::INNER),
            ),
        );
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionLattice))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Lattice",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `rows`/`cols` — é o CLAMP DO KERNEL, e a distinção é o achado** (doc 88 A1).
///
/// ⚠️ **Um teto digitável acima do que o kernel honra é uma caixa que MENTE:** o `eval` clampa
/// cada lado em [`MAX_SIDE`], então uma caixa que aceitasse 5.000 mostraria 5.000 e o produto
/// entregaria 400 — pior que um teto baixo, porque aceita e não avisa. O hard max é o clamp.
///
/// E o clamp **não é de custo**: medido pela porta do produto (`measure_the_count_ceiling`,
/// `cols = 1`), a treliça no próprio teto de 400 por lado custa **0,001 ms** — quatro ordens de
/// grandeza abaixo de um quadro. Subi-lo é mudar o KERNEL, com a medição do produto `rows × cols`
/// ao lado; não é coisa que um teto de UI possa fazer sozinho.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "rows",
        max: MAX_SIDE as f32,
    },
    ParamHardMax {
        param: "cols",
        max: MAX_SIDE as f32,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 1.0,
        max: 60.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "Cols",
        min: 1.0,
        max: 60.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 0.1,
        max: 4.0,
        step: 0.05,
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
        param: ph2d_motion_region::SHAPE,
        label: "Shape",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: ph2d_motion_region::SHAPE_LABELS,
        },
    },
    ParamUiHint {
        param: ph2d_motion_region::INNER,
        label: "Hole",
        min: 0.0,
        max: 0.98,
        step: 0.01,
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
    param: "spacing",
    unit: ParamUnit::Length,
}];

/// O buraco só existe no anel.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[ph2d_node_registry::ParamGate {
    param: ph2d_motion_region::INNER,
    when: ph2d_motion_region::SHAPE,
    values: &[ph2d_motion_region::SHAPE_RING],
}];

#[cfg(test)]
mod region_tests;

#[cfg(test)]
mod tests {
    use super::*;

    /// The nearest-neighbour distance in the set.
    fn nearest_neighbour(pts: &[[f32; 2]]) -> f32 {
        let mut min = f32::MAX;
        for (i, a) in pts.iter().enumerate() {
            for b in &pts[i + 1..] {
                let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
                min = min.min((dx * dx + dy * dy).sqrt());
            }
        }
        min
    }

    /// The lattice IS hexagonal: with no jitter, the nearest-neighbour distance equals
    /// `spacing` for every point (equilateral packing). FALSIFIED for a square grid,
    /// whose diagonal neighbours would be `spacing·√2` — but the row pitch `√3/2` keeps
    /// the offset rows exactly `spacing` away.
    #[test]
    fn it_is_a_hexagonal_packing() {
        let pts = lattice(5, 6, 0.7, 1, &[]);
        assert_eq!(pts.len(), 30);
        let nn = nearest_neighbour(&pts);
        assert!(
            (nn - 0.7).abs() < 1e-4,
            "every nearest neighbour is one spacing away (hex): {nn}"
        );
    }

    /// Odd rows are shifted half a cell — the defining hex offset. Row 0 col 0 and
    /// row 1 col 0 differ by half a spacing in x (not zero, as a square grid would).
    #[test]
    fn odd_rows_are_half_shifted() {
        let cols = 6;
        let pts = lattice(2, cols, 1.0, 1, &[]);
        let dx = pts[cols][0] - pts[0][0]; // (row1,col0) − (row0,col0)
        assert!((dx - 0.5).abs() < 1e-5, "odd row shifted half a cell: {dx}");
    }

    /// The lattice is centred on the origin (mean position ≈ 0).
    #[test]
    fn it_is_centred_on_the_origin() {
        let pts = lattice(7, 7, 0.6, 1, &[]);
        let mean = pts
            .iter()
            .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
        let n = pts.len() as f32;
        assert!((mean[0] / n).abs() < 0.05 && (mean[1] / n).abs() < 0.05);
    }

    /// `jitter` melts the lattice: a positive jitter breaks the exact packing (the
    /// nearest-neighbour distance drops below `spacing`), and it is deterministic.
    #[test]
    fn jitter_melts_the_lattice_deterministically() {
        let ordered = lattice(6, 6, 0.7, 3, &[]);
        let melted = lattice(6, 6, 0.7, 3, &[0.3]);
        assert!(
            nearest_neighbour(&melted) < nearest_neighbour(&ordered),
            "jitter clumps some points closer than the perfect packing"
        );
        assert_eq!(melted, lattice(6, 6, 0.7, 3, &[0.3]), "reproducible");
        assert_ne!(
            melted,
            lattice(6, 6, 0.7, 4, &[0.3]),
            "seed re-rolls the jitter"
        );
    }

    /// **O `jitter` É UM CAMPO** — cada ponto derrete pelo SEU valor, não pelo do ponto 0.
    ///
    /// ⚠️ O defeito que este gate tranca não dava erro nenhum (doc 90 §5): a porta é do domínio
    /// `Instances`, e ligar-lhe o gesto óbvio (uma rampa) entregava à treliça inteira o elemento
    /// `0`, que numa rampa é `0.0` ⇒ **jitter zero em todo lado**, com a porta ligada.
    ///
    /// O oráculo é a comparação PONTO A PONTO contra as duas corridas uniformes: os pontos com
    /// `0` têm de estar onde a treliça perfeita os põe, e os com `0,3` onde a derretida os põe.
    #[test]
    fn the_jitter_is_a_field_not_the_first_element() {
        let (rows, cols) = (4usize, 4usize);
        let n = rows * cols;
        let ordered = lattice(rows, cols, 0.7, 3, &[]);
        let melted = lattice(rows, cols, 0.7, 3, &[0.3]);
        // CONTROLE: a fixture tem de conter o fenómeno — as duas corridas discordam.
        assert_ne!(ordered, melted, "controle: 0,3 tem de derreter a treliça");

        // Metade quieta, metade derretida — pelo ÍNDICE do ponto.
        let half: Vec<f32> = (0..n).map(|i| if i < n / 2 { 0.0 } else { 0.3 }).collect();
        let mixed = lattice(rows, cols, 0.7, 3, &half);
        for i in 0..n / 2 {
            assert_eq!(
                mixed[i], ordered[i],
                "ponto {i}: jitter 0 ⇒ treliça perfeita"
            );
        }
        for i in n / 2..n {
            assert_eq!(mixed[i], melted[i], "ponto {i}: jitter 0,3 ⇒ derretido");
        }
    }

    /// Cooks through the registry and emits the `P` column, with the `jitter` input
    /// unconnected (→ a perfect lattice).
    #[test]
    fn registers_and_cooks() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionLattice as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let n = g.add_node("motion.lattice");
        g.set_param(n, "rows", 4.0);
        g.set_param(n, "cols", 5.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 20),
            _ => panic!("P"),
        }
    }
}

#[cfg(test)]
mod hard_max_gates {
    use super::{MAX_SIDE, PARAM_HARD_MAX};

    /// **O teto DIGITÁVEL não pode passar do que o KERNEL honra.**
    ///
    /// O slider dual separa *a faixa confortável* do *onde o disfuncional começa*, e a segunda
    /// pergunta tem um respondente natural quando o `eval` clampa: o clamp. Se a caixa aceitar
    /// 5.000 e o `eval` entregar 400, o artista digita, o número FICA na tela e a treliça não
    /// muda — um controle que **aceita e mente**, que é pior que um teto baixo, porque um teto
    /// baixo pelo menos recusa à vista.
    #[test]
    fn the_typed_ceiling_stops_where_the_kernel_clamps() {
        for param in ["rows", "cols"] {
            let limit = PARAM_HARD_MAX
                .iter()
                .find(|h| h.param == param)
                .unwrap_or_else(|| panic!("{param} tem teto duro"));
            assert_eq!(
                limit.max, MAX_SIDE as f32,
                "o teto digitável de {param} tem de ser o clamp do kernel, nem mais nem menos"
            );
        }
    }
}
