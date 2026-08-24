//! `motion.scatter` — a **blue-noise** point distribution: `count` points spread
//! evenly-but-randomly over a rectangle, no two too close (Motion Nodes M3,
//! distributions — doc 01 §3 / doc 19). This is the "even random" layout — the
//! opposite of the orderly `motion.fibonacci` spiral and of plain white-noise
//! `random` (which clumps and gaps). Blue noise is what scattering, stippling and
//! sampling all want: the human eye reads it as "natural, unstructured" precisely
//! because it has NO clumps.
//!
//! **Algorithm — Mitchell's best-candidate (1991), not Bridson Poisson-disk.** The
//! gold-standard blue-noise generators are Bridson's dart-throwing (fills a domain
//! to a min-radius — the count is *implicit*) and Mitchell's best-candidate (place
//! exactly `N`, each the farthest of `k` random darts from the points so far — the
//! count is *exact*). A node has a `count` param, so exact-count fits far better,
//! and best-candidate is simpler and allocation-bounded. Its blue-noise quality is
//! excellent for scattering (each point maximises its distance to its neighbours).
//!
//! A **Source** node (no input, mints `P`). Stateless (Jarzynski/Olano): every
//! candidate is a hash of `(seed, index, candidate, lane)`, so the layout is a pure
//! function of the params — `Effect::Pure`, scrub-stable, no clock. Transcendental-
//! free (HR-5): hashing, squared distances (`√` only for the final comparison is
//! avoidable — we compare squared distances), min/max.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::hash3;
// ⚠️ **`Region`, e não `Domain`** — a palavra `Domain` já é da casa: no
// `ph2d_nodegraph::port` ela diz em que PLANO de dados a porta vive (instâncias,
// vetor, campo). Reusá-la para «a região do plano» daria dois sentidos ao mesmo
// nome dentro deste arquivo, que importa os dois.
use ph2d_motion_region::Region;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Random darts thrown per placed point; the farthest from the existing set wins.
/// Mitchell's rule of thumb (`k ≈ current_count + 1`) is bounded here to a small
/// constant — the blue-noise quality is already excellent and the cost stays
/// `O(N²·K)` linear in K.
const CANDIDATES: u32 = 12;

/// A chave do param **da densidade graduada** — quanto a densidade cai do coração
/// da região até à fronteira dela.
pub const DENSITY_FALLOFF: &str = "density_falloff";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.scatter"),
    name: "motion.scatter",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Number of points. Clamped ≥0 and capped like the grid.
        ParamSpec {
            name: "count",
            default: 180.0,
        },
        // The rectangle to fill (world units), centred on the origin.
        ParamSpec {
            name: "width",
            default: 4.0,
        },
        ParamSpec {
            name: "height",
            default: 4.0,
        },
        // Integer seed (rounded at eval) — re-rolls the whole layout.
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
        // **A REGIÃO** (doc 89, folha 01) — `Rect` é o retângulo de sempre, ao bit.
        ParamSpec {
            name: ph2d_motion_region::SHAPE,
            default: 0.0,
        },
        ParamSpec {
            name: ph2d_motion_region::INNER,
            default: 0.5,
        },
        // **A DENSIDADE GRADUADA** — `0` é uniforme, e uniforme é o de hoje.
        ParamSpec {
            name: DENSITY_FALLOFF,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The squared distance from `p` to the nearest of `placed` (`f32::MAX` if empty).
fn nearest_sq(p: [f32; 2], placed: &[[f32; 2]]) -> f32 {
    placed
        .iter()
        .map(|q| {
            let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
            dx * dx + dy * dy
        })
        .fold(f32::MAX, f32::min)
}

/// Lay out `count` blue-noise points in `region` (centred on the origin) by
/// best-candidate: for each point, throw `CANDIDATES` hashed darts and keep the one
/// whose nearest existing neighbour is farthest away.
///
/// ## ⭐ A densidade entra na PONTUAÇÃO, e por isso a contagem não se mexe
///
/// O critério de Mitchell é *"o candidato mais longe do que já lá está"*. Com uma
/// densidade graduada ele passa a ser *"o mais longe **medido em espaçamentos
/// locais**"* — e como o espaçamento alvo vai com `1/√densidade`, a pontuação é
/// `nearest² × densidade`. Um candidato numa zona rala só ganha se estiver `√5 ≈
/// 2,2×` mais afastado, o que dá exactamente `1/5` da densidade de área.
///
/// ⚠️ **É isto que a cadeia `field.remap(probability) → motion.cull` NÃO faz.** Ela
/// sorteia quem morre: pedir 400 pontos com metade da densidade devolve ~200, e a
/// contagem deixa de ser o que o artista digitou. Aqui `count` continua exacto e o
/// que muda é **onde** os pontos se acomodam — que é o que a referência (Blender
/// *Distribute Points on Faces*, `Density Max` × campo) entrega.
fn scatter(count: usize, region: &Region, falloff: f32, seed: u32) -> Vec<[f32; 2]> {
    let mut placed: Vec<[f32; 2]> = Vec::with_capacity(count);
    let graded = falloff > 0.0;
    for i in 0..count {
        let mut best = [0.0, 0.0];
        let mut best_score = -1.0_f32;
        for k in 0..CANDIDATES {
            let key = i as u32 * CANDIDATES + k;
            // Two decorrelated lanes → a point drawn uniformly over the region.
            let p = region.sample(hash3(seed, key, 0), hash3(seed, key, 1));
            let d = nearest_sq(p, &placed);
            // ⚠️ Sem gradação a pontuação É a distância — sem uma multiplicação por
            // `1,0` no caminho, que num `f32` não é a identidade para todo valor.
            let score = if graded {
                d * region.density(p, falloff)
            } else {
                d
            };
            if score > best_score {
                best_score = score;
                best = p;
            }
        }
        placed.push(best);
    }
    placed
}

struct MotionScatter;

impl NodeOp for MotionScatter {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS);
        let region = Region::of(
            ctx.param(ph2d_motion_region::SHAPE),
            ctx.param("width"),
            ctx.param("height"),
            ctx.param(ph2d_motion_region::INNER),
        );
        let seed = ctx.param("seed").max(0.0).round() as u32;
        let positions = scatter(count, &region, ctx.param(DENSITY_FALLOFF), seed);
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionScatter))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Scatter",
            // Source: a generator that mints the stream (like Grid / Fibonacci).
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

/// O buraco só existe no anel — nas outras duas formas ele seria um controle vivo
/// que não muda nada, que é a doença que um gate cura.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[ph2d_node_registry::ParamGate {
    param: ph2d_motion_region::INNER,
    when: ph2d_motion_region::SHAPE,
    values: &[ph2d_motion_region::SHAPE_RING],
}];

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};
/// **O teto DURO de `count` — e aqui ele é um limite de RECURSO, não um freio ergonômico** (doc 88
/// A1 · §0). O blue noise por *best-candidate* (Mitchell) mede a distância de cada candidato a
/// TODOS os pontos já postos (`nearest_sq`), logo o custo é **O(count² × CANDIDATES)** — inerente
/// ao algoritmo, não um defeito. O cook mediu, pela porta do produto:
///
/// | instâncias | cook |
/// |---|---|
/// | 2.000 | 5,190 ms |
/// | **3.000** | **11,443 ms** |
/// | 4.000 | 20,661 ms ← passa o quadro |
/// | 6.000 | 44,512 ms |
///
/// ⚠️ **O quadro de 60 fps quebra entre 3.000 e 4.000**, então o teto é **3.000** — e ele fica
/// deliberadamente PERTO do soft de 2.000. Um teto redondo e generoso aqui (os 1.000.000 que os
/// nós LINEARES desta mesma wave receberam) deixaria o artista digitar um número que **congela o
/// app por minutos**: a 100.000 este cook custou **12,3 segundos**, e a 400.000, **208**.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: "count",
    max: 3_000.0,
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 2000.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "width",
        label: "Width",
        min: 0.1,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "height",
        label: "Height",
        min: 0.1,
        max: 20.0,
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
    ParamUiHint {
        param: DENSITY_FALLOFF,
        label: "Density Falloff",
        min: 0.0,
        max: 1.0,
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
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "width",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "height",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
mod region_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// O retângulo de sempre — o que estes gates mediam antes de a região existir.
    pub(super) fn rect(w: f32, h: f32) -> Region {
        Region::of(0.0, w, h, 0.0)
    }

    fn nearest_neighbour_sq(pts: &[[f32; 2]]) -> f32 {
        // The smallest neighbour distance in the whole set — the blue-noise floor.
        let mut min = f32::MAX;
        for (i, a) in pts.iter().enumerate() {
            for b in &pts[i + 1..] {
                let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
                min = min.min(dx * dx + dy * dy);
            }
        }
        min
    }

    /// Every point lands INSIDE the requested rectangle (centred on the origin).
    #[test]
    fn all_points_lie_in_the_rectangle() {
        let pts = scatter(200, &rect(4.0, 3.0), 0.0, 1);
        assert_eq!(pts.len(), 200);
        for p in &pts {
            assert!(
                p[0].abs() <= 2.0 + 1e-4 && p[1].abs() <= 1.5 + 1e-4,
                "outside: {p:?}"
            );
        }
    }

    /// The blue-noise property, FALSIFIED against plain white noise: best-candidate
    /// keeps every pair well apart, so the smallest neighbour distance is far larger
    /// than uniform random darts (which clump). We compare the min neighbour gap of
    /// the scatter to that of the raw first-candidate (white-noise) draws.
    #[test]
    fn it_is_blue_noise_not_white_noise() {
        let (w, h, n, seed) = (4.0, 4.0, 200, 3);
        let blue = scatter(n, &rect(w, h), 0.0, seed);
        // White noise: just the first dart per index (no best-candidate choice).
        let white: Vec<[f32; 2]> = (0..n)
            .map(|i| {
                let key = i as u32 * CANDIDATES;
                [
                    (hash3(seed, key, 0) - 0.5) * w,
                    (hash3(seed, key, 1) - 0.5) * h,
                ]
            })
            .collect();
        let blue_gap = nearest_neighbour_sq(&blue);
        let white_gap = nearest_neighbour_sq(&white);
        assert!(
            blue_gap > white_gap * 4.0,
            "blue noise spaces points far better than white (blue {blue_gap} vs white {white_gap})"
        );
    }

    /// Deterministic (scrub-stable): the same params redraw the identical layout,
    /// and a different seed re-rolls it.
    #[test]
    fn same_seed_reproduces_and_a_new_seed_re_rolls() {
        assert_eq!(
            scatter(50, &rect(4.0, 4.0), 0.0, 1),
            scatter(50, &rect(4.0, 4.0), 0.0, 1),
            "reproducible"
        );
        assert_ne!(
            scatter(50, &rect(4.0, 4.0), 0.0, 1),
            scatter(50, &rect(4.0, 4.0), 0.0, 2),
            "seed re-rolls"
        );
    }

    /// Cooks through the registry and emits the `P` column at the requested count.
    #[test]
    fn registers_and_cooks_the_position_stream() {
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionScatter as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let s = g.add_node("motion.scatter");
        g.set_param(s, "count", 16.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, s, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 16, "16 points emitted"),
            _ => panic!("P"),
        }
    }
}
