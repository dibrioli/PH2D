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
        // **A DENSIDADE GRADUADA** — `0` é uniforme, e uniforme é o de hoje.
        ParamSpec {
            name: DENSITY_FALLOFF,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The squared distance from `p` to the nearest of `placed` (`f32::MAX` if empty).
/// ⚠️ **O ORÁCULO, e só isso** — a varredura linear que a [`NearGrid`] substituiu em
/// 2026-09-05. Fica viva **em teste** porque é ela que prova a bit-identidade da grelha
/// (`the_grid_returns_exactly_what_the_scan_returned`); apagá-la deixaria a troca sem oráculo,
/// e o gate a comparar a grelha consigo própria.
#[cfg(test)]
fn nearest_sq(p: [f32; 2], placed: &[[f32; 2]]) -> f32 {
    placed
        .iter()
        .map(|q| {
            let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
            dx * dx + dy * dy
        })
        .fold(f32::MAX, f32::min)
}

/// ⭐⭐⭐ **A GRELHA DE VIZINHANÇA — o que tira este nó de `O(n²)`.**
///
/// ⛔⛔ **Medido em 2026-09-05: `motion.scatter` custava `153 ms` para 10 000 pontos** — nove
/// quadros a 60 fps, para o nó que um artista põe primeiro. A causa é o critério de Mitchell na
/// forma ingénua: por cada um dos `count` pontos lançam-se [`CANDIDATES`] dardos, e cada dardo
/// varria **todos os pontos já colocados** ⇒ `10 000 × 12 × 5 000 ≈ 600 milhões` de distâncias.
///
/// A grelha indexa os pontos colocados por célula do tamanho do espaçamento alvo (~1 ponto por
/// célula), e a consulta cresce em ANÉIS à volta da célula do candidato, parando quando o anel
/// seguinte já não pode conter nada mais perto que o melhor até agora.
///
/// ⭐⭐ **A saída é BIT-IDÊNTICA, e não por promessa:** [`Self::nearest_sq`] devolve o **mínimo
/// do mesmo conjunto** que a varredura devolvia, e um mínimo não depende da ordem de visita
/// (`f32::min` é comutativo fora de `NaN`). A paragem é conservadora — só corta quando nenhum
/// ponto por visitar pode estar mais perto —, e o gate
/// `the_grid_returns_exactly_what_the_scan_returned` mede-o ponto a ponto.
struct NearGrid {
    /// Lado de uma célula. Nunca zero: uma região degenerada cai numa célula só.
    cell: f32,
    cols: usize,
    rows: usize,
    /// O canto inferior-esquerdo da caixa da região.
    ox: f32,
    oy: f32,
    /// Índices em `placed`, por célula.
    cells: Vec<Vec<u32>>,
}

impl NearGrid {
    /// Uma grelha dimensionada para ~**um ponto por célula** — o que faz a consulta ser `O(1)`
    /// amortizado. ⚠️ O número de células é limitado a `4 × count` para uma região muito
    /// alongada não pedir memória a mais do que os pontos que vai guardar.
    fn new(region: &Region, count: usize) -> Self {
        let [hw, hh] = region.half_extents();
        let (w, h) = (
            (2.0 * hw).max(f32::MIN_POSITIVE),
            (2.0 * hh).max(f32::MIN_POSITIVE),
        );
        let alvo = (w * h / (count.max(1) as f32))
            .max(f32::MIN_POSITIVE)
            .sqrt();
        let mut cols = ((w / alvo).ceil() as usize).max(1);
        let mut rows = ((h / alvo).ceil() as usize).max(1);
        let tecto = count.max(1).saturating_mul(4);
        while cols.saturating_mul(rows) > tecto && (cols > 1 || rows > 1) {
            cols = (cols / 2).max(1);
            rows = (rows / 2).max(1);
        }
        Self {
            cell: (w / cols as f32)
                .max(h / rows as f32)
                .max(f32::MIN_POSITIVE),
            cols,
            rows,
            ox: -hw,
            oy: -hh,
            cells: vec![Vec::new(); cols * rows],
        }
    }

    /// A célula de `p`, presa à grelha (um candidato nasce dentro da região, logo dentro da
    /// caixa; o clamp é a rede contra o bordo exacto e contra um `f32` não-finito).
    fn cell_of(&self, p: [f32; 2]) -> (usize, usize) {
        let cx = ((p[0] - self.ox) / self.cell) as isize;
        let cy = ((p[1] - self.oy) / self.cell) as isize;
        (
            cx.clamp(0, self.cols as isize - 1) as usize,
            cy.clamp(0, self.rows as isize - 1) as usize,
        )
    }

    fn insert(&mut self, i: u32, p: [f32; 2]) {
        let (cx, cy) = self.cell_of(p);
        self.cells[cy * self.cols + cx].push(i);
    }

    /// O quadrado da distância ao ponto colocado mais próximo — **o mesmo `f32`** que a
    /// varredura devolvia. `f32::MAX` quando ainda não há nenhum.
    fn nearest_sq(&self, p: [f32; 2], placed: &[[f32; 2]]) -> f32 {
        let (cx, cy) = self.cell_of(p);
        let mut best = f32::MAX;
        let max_anel = self.cols.max(self.rows);
        for r in 0..=max_anel {
            // ⚠️ **A paragem é CONSERVADORA:** `p` pode estar em qualquer ponto da sua célula,
            // então um ponto no anel `r` está a pelo menos `(r-1)·cell`. Cortar em `r·cell`
            // perderia vizinhos e a saída deixaria de bater com a varredura.
            if r > 1 {
                let piso = (r - 1) as f32 * self.cell;
                if piso * piso > best {
                    break;
                }
            }
            let (x0, x1) = (cx.saturating_sub(r), (cx + r).min(self.cols - 1));
            let (y0, y1) = (cy.saturating_sub(r), (cy + r).min(self.rows - 1));
            // ⚠️ **Só o CONTORNO, percorrido como contorno** — o miolo já foi visitado num anel
            // anterior. A 1.ª versão varria o RECTÂNGULO e saltava o miolo com um `if`, o que
            // faz cada anel custar `O(r²)` em vez de `O(r)`; com a grelha quase vazia (os
            // primeiros pontos) o `r` chega ao lado da grelha e isso sozinho era metade do
            // relógio.
            let visita = |cel: &Vec<u32>, best: &mut f32| {
                for &i in cel {
                    let q = placed[i as usize];
                    let (dx, dy) = (p[0] - q[0], p[1] - q[1]);
                    *best = best.min(dx * dx + dy * dy);
                }
            };
            if r == 0 {
                visita(&self.cells[cy * self.cols + cx], &mut best);
            } else {
                // As duas fileiras horizontais (topo e fundo), inteiras.
                for x in x0..=x1 {
                    if cy >= r {
                        visita(&self.cells[y0 * self.cols + x], &mut best);
                    }
                    if cy + r < self.rows {
                        visita(&self.cells[y1 * self.cols + x], &mut best);
                    }
                }
                // E as duas colunas verticais, sem repetir os cantos.
                let (iy0, iy1) = (
                    if cy >= r { y0 + 1 } else { y0 },
                    if cy + r < self.rows {
                        y1.saturating_sub(1)
                    } else {
                        y1
                    },
                );
                for y in iy0..=iy1.max(iy0) {
                    if y > y1 {
                        break;
                    }
                    if cx >= r {
                        visita(&self.cells[y * self.cols + x0], &mut best);
                    }
                    if cx + r < self.cols {
                        visita(&self.cells[y * self.cols + x1], &mut best);
                    }
                }
            }
            // ⚠️ **E pára quando o anel já saiu da grelha inteira** — senão, com a grelha quase
            // vazia, a busca percorre-a toda por cada um dos primeiros pontos.
            if x0 == 0 && y0 == 0 && x1 == self.cols - 1 && y1 == self.rows - 1 {
                break;
            }
        }
        best
    }
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
    let mut grelha = NearGrid::new(region, count);
    let graded = falloff > 0.0;
    for i in 0..count {
        let mut best = [0.0, 0.0];
        let mut best_score = -1.0_f32;
        for k in 0..CANDIDATES {
            let key = i as u32 * CANDIDATES + k;
            // Two decorrelated lanes → a point drawn uniformly over the region.
            let p = region.sample(hash3(seed, key, 0), hash3(seed, key, 1));
            let d = grelha.nearest_sq(p, &placed);
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
        grelha.insert(placed.len() as u32, best);
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
        let region = Region::rect(ctx.param("width"), ctx.param("height"));
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
            display_key: "node.motion.scatter.name",
            // Source: a generator that mints the stream (like Grid / Fibonacci).
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

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
        label: "node.motion.scatter.param.count",
        min: 1.0,
        max: 2000.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "width",
        label: "node.motion.scatter.param.width",
        min: 0.1,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "height",
        label: "node.motion.scatter.param.height",
        min: 0.1,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "node.motion.scatter.param.seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: DENSITY_FALLOFF,
        label: "node.motion.scatter.param.density_falloff",
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
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// O retângulo — desde 2026-09-19 a única região que existe (ordem do dono: *«tire de
    /// todos»*), e a fachada fica porque estes gates a nomeiam dezenas de vezes.
    pub(super) fn rect(w: f32, h: f32) -> Region {
        Region::rect(w, h)
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

#[cfg(test)]
mod grid_tests {
    use super::{CANDIDATES, NearGrid, hash3, nearest_sq, scatter};
    use ph2d_motion_region::Region;

    /// ⭐⭐⭐ **A GRELHA DEVOLVE EXACTAMENTE O QUE A VARREDURA DEVOLVIA** — ponto a ponto, bit a
    /// bit.
    ///
    /// ⚠️ **Ele varria as TRÊS formas e passou a varrer três EXTENSÕES** (2026-09-19): o `Shape`
    /// saiu por ordem do dono, e a grandeza que de facto discrimina a grelha de vizinhança é a
    /// forma da CAIXA — uma caixa larga e baixa põe mais pontos por célula numa direcção do que
    /// na outra, que é onde um raio de busca errado se vê. *Varrer um param que já não existe
    /// seria varrer o mesmo caso três vezes.*
    ///
    /// ⚠️ É este gate que autoriza a troca: o critério de Mitchell escolhe pelo `>` sobre a
    /// pontuação, então **um único `f32` diferente muda o ponto escolhido** e daí em diante a
    /// nuvem inteira. FALSIFICADO por cortar a busca em `r·cell` em vez de `(r-1)·cell`
    /// (perde-se um vizinho na célula ao lado e a distância vem maior).
    #[test]
    fn the_grid_returns_exactly_what_the_scan_returned() {
        for (w, h) in [(400.0_f32, 260.0_f32), (400.0, 40.0), (60.0, 400.0)] {
            let region = Region::rect(w, h);
            let mut placed: Vec<[f32; 2]> = Vec::new();
            let mut grelha = NearGrid::new(&region, 600);
            for i in 0..600u32 {
                let p = region.sample(hash3(7, i, 0), hash3(7, i, 1));
                let pela_grelha = grelha.nearest_sq(p, &placed);
                let pela_varredura = nearest_sq(p, &placed);
                assert_eq!(
                    pela_grelha.to_bits(),
                    pela_varredura.to_bits(),
                    "caixa {w}x{h}, ponto {i}: a grelha deu {pela_grelha} e a varredura {pela_varredura}"
                );
                grelha.insert(placed.len() as u32, p);
                placed.push(p);
            }
        }
    }

    /// **E a NUVEM inteira é a mesma** — o gate de cima mede a consulta, este mede o produto.
    /// FALSIFICADO por qualquer diferença na ordem de inserção ou no critério de paragem.
    #[test]
    fn the_cloud_is_bit_identical_to_the_quadratic_one() {
        for (w, h, count, falloff) in [
            (300.0_f32, 200.0_f32, 400usize, 0.0_f32),
            (300.0, 40.0, 250, 0.7),
            (40.0, 300.0, 300, 0.0),
        ] {
            let region = Region::rect(w, h);
            let novo = scatter(count, &region, falloff, 3);
            // O algoritmo de referência, escrito aqui à letra do que existia antes da grelha.
            let mut placed: Vec<[f32; 2]> = Vec::with_capacity(count);
            let graded = falloff > 0.0;
            for i in 0..count {
                let mut best = [0.0, 0.0];
                let mut best_score = -1.0_f32;
                for k in 0..CANDIDATES {
                    let key = i as u32 * CANDIDATES + k;
                    let p = region.sample(hash3(3, key, 0), hash3(3, key, 1));
                    let d = nearest_sq(p, &placed);
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
            assert_eq!(novo.len(), placed.len());
            for (i, (a, b)) in novo.iter().zip(&placed).enumerate() {
                assert_eq!(
                    (a[0].to_bits(), a[1].to_bits()),
                    (b[0].to_bits(), b[1].to_bits()),
                    "caixa {w}x{h}, ponto {i}: {a:?} contra {b:?}"
                );
            }
        }
    }

    /// **A MEDIÇÃO da cura** — `#[ignore]`, para o número do doc ser reproduzível com um comando.
    ///
    /// `cargo test -p ph2d-node-motion-scatter --release -- --ignored --nocapture measure_scatter_cost`
    #[test]
    #[ignore = "medicao"]
    fn measure_scatter_cost() {
        let region = Region::rect(800.0, 600.0);
        eprintln!(
            "  load: {}",
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
        eprintln!("  {:>8} │ {:>10} │ {:>12}", "pontos", "ms", "us/ponto");
        for n in [500usize, 1_000, 5_000, 10_000, 50_000] {
            let mut melhor = f64::MAX;
            for _ in 0..3 {
                let t = std::time::Instant::now();
                let v = scatter(n, &region, 0.0, 1);
                melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
                assert_eq!(v.len(), n);
            }
            eprintln!(
                "  {n:>8} │ {melhor:>10.2} │ {:>12.3}",
                melhor * 1000.0 / n as f64
            );
        }
    }
}
