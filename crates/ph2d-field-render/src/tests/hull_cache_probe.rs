//! ⏳ **§83.9 — A CACHE CONTRA O CASCO, e não contra a caixa** — a sonda de CONTAGEM que decide se a
//! wave existe (2026-09-13).
//!
//! A W82 guarda cada fita construída para a **caixa** inflada, e o caminho sem cache especializa
//! contra o **casco** do tubo (W59), que é mais apertado. O `docs/3DModeling/06` §83.9 deixou nomeado
//! o que isso deixa na mesa — *«`1,11×` de `74,6 → 67,2` arestas»* — e a saída: um teste em **dois
//! níveis** (a caixa rejeita, o casco confirma).
//!
//! ⭐ A sonda mede as DUAS metades com a mesma política de cache, lado a lado:
//! - **acerto** e **compilações por quadro** num arrasto;
//! - **arestas da fita SERVIDA** por região, contra as do caminho sem cache. ⚠️ O preço de uma
//!   amostra segue as arestas quase à letra (§82.12: `88,3 / 74,6 = 1,18×` arestas ⇒ `1,18×` no
//!   custo), então esta coluna **é** a razão de custo da marcha.
//!
//! # O que as corridas de 13/09 já disseram
//!
//! 1. ⛔⛔ **A caixa serve `1,9×`–`2,4×` as arestas do caminho sem cache** — não os `1,18×`–`1,31×` do
//!    §83.8, que foram medidos com o ladrilho a `64`. A `24` px o tubo é fino, e a caixa de um tubo
//!    fino e oblíquo é quase toda vazio. ⇒ *a nota punha `1,11×` na mesa; a mesa tem até `~2×`.*
//! 2. ⛔ **O casco escalado por `f` perde os acertos** (`f = 1,25`: `48`–`70 %`, e as compilações
//!    TRIPLICAM; `f = 2,0`: volta a ser a caixa). O mecanismo: a folga de um casco escalado é
//!    proporcional à **largura** do tubo, e o que a câmera move num arrasto é uma **distância** —
//!    raio × ângulo — que não sabe a largura de tubo nenhum.
//! 3. ⛔ **A ORDEM não era o suspeito.** O [`crate::TapeCache::get`] devolve a primeira fita que
//!    contém a região, e o vector vai da mais velha para a mais nova; medido, servir a mais velha, a
//!    mais nova ou a de menor volume muda as arestas servidas em **menos de `5 %`**. ⇒ o peso é da
//!    caixa, e não de qual caixa.
//! 4. ⭐⭐⭐ **O casco crescido por uma DISTÂNCIA domina a caixa** ([`Grow::Pad`]), nas 12 células
//!    (2 tamanhos × 2 peças × 3 velocidades): a `δ = 0,08` serve as MESMAS arestas com **3× a 6×
//!    menos compilações**; a `δ = 0,04` corta **`~25 %` das arestas**, e a `640×360` compila MENOS
//!    que a caixa ao mesmo tempo. ⇒ a pergunta passa a ser **de onde o `δ` sai** — ver
//!    [`measure_where_the_pad_should_come_from`].
//!
//! ⚠️ **Tudo contagens** — vale com a máquina sob carga. O relógio só decide depois, a `load < 5`.
//!
//! ⚠️ O teste de contenção aqui é **o que a especialização consome**: numa folha na pose identidade o
//! `(u, v)` do perfil é o `(x, y)` do mundo, e o casco é o [`ph2d_field_eval::probe_hull_uv`] que o
//! `compile_at` usa. Um produto com poses tem de o repetir **por folha**.
//!
//! ⚠️ **O despejo é simplificado**: guarda-se o que foi pedido nos últimos [`FRAMES_KEPT`] quadros,
//! que é a população que o tecto derivado do produto (`regiões × 3`) aguenta, e a ordem do vector é a
//! de inserção (a mais velha à frente), como no produto. As variantes correm todas com a mesma regra,
//! e o que se lê é a diferença entre elas.
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --lib -- --exact \
//!     tests::hull_cache_probe::measure_what_a_hull_cache_would_buy --ignored --nocapture
//! cargo test -p ph2d-field-render --profile ci-test --lib -- --exact \
//!     tests::hull_cache_probe::measure_where_the_pad_should_come_from --ignored --nocapture
//! ```

use super::{ngon_probe, star_probe};
use crate::{Orbit, Screen};
use ph2d_field::{FieldDoc, FillRule, NodeId, Primitive, Profile, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_eval::profile_index::ProfileIndex;

/// Uma caixa de mundo.
type Aabb = ([f32; 3], [f32; 3]);

/// Quantos quadros de regiões a cache simulada guarda — a mesma contagem do produto
/// (`tape_cache::FRAMES_KEPT`).
const FRAMES_KEPT: usize = 3;

/// Quadros medidos por arrasto; o quadro `0` enche a cache do zero e não entra na conta.
const FRAMES: usize = 12;

/// Uma região de um quadro, exactamente como o `tiles::tiled_trace` a pede.
struct Query {
    lo: [f32; 3],
    hi: [f32; 3],
    pts: Vec<[f32; 3]>,
    seed: u64,
}

/// Uma fita guardada: a região para que foi construída e as arestas que ela guarda.
struct Entry {
    lo: [f32; 3],
    hi: [f32; 3],
    /// O casco que a especialização usou — **vazio** quando ela cortou pela caixa.
    hull: Vec<[f32; 2]>,
    kept: usize,
    seen: usize,
}

impl Entry {
    fn volume(&self) -> f32 {
        (0..3).map(|k| self.hi[k] - self.lo[k]).product()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Policy {
    /// O que shipa: a fita é construída para a caixa crescida e servida por contenção de caixa.
    Box,
    /// A candidata: a fita é construída para o casco crescido, e servida se a caixa E o casco contêm.
    Hull,
}

/// Como a região cresce antes de a fita ser compilada.
#[derive(Clone, Copy)]
enum Grow {
    /// O que shipa: escala `f` em torno do centro, deslocada pela fase ([`crate::tape_cache::PHASE`]).
    Scale(f32),
    /// Uma distância `δ` de mundo em cada lado, com a MESMA dispersão de fase (em fracção de `δ`).
    Pad(f32),
}

/// Qual das fitas que contêm a região é servida.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pick {
    /// O que shipa: a primeira do vector, que é a mais velha.
    Oldest,
    /// A última do vector — a mais nova.
    Newest,
    /// A de menor VOLUME de caixa.
    Tightest,
}

#[derive(Default)]
struct Tally {
    uses: usize,
    hits: usize,
    compiles: usize,
    /// Σ das arestas da fita que serviu cada região.
    served: usize,
    /// Σ das arestas que o caminho SEM cache usaria na mesma região.
    honest: usize,
}

/// Uma peça de perfil na pose identidade, com o índice do contorno e a caixa da marcha.
struct Piece {
    idx: ProfileIndex,
    bbox: Aabb,
}

impl Piece {
    /// A meia-diagonal da caixa da peça — o maior braço com que uma órbita em torno da origem move um
    /// ponto dela.
    fn reach(&self) -> f32 {
        (0..3)
            .map(|k| {
                let h = 0.5 * (self.bbox.1[k] - self.bbox.0[k]);
                h * h
            })
            .sum::<f32>()
            .sqrt()
    }
}

fn piece(ring: Vec<[f32; 2]>, half_height: f32) -> Piece {
    let profile = Profile::new(vec![ring], FillRule::NonZero, 1e-4).expect("perfil");
    let idx = ProfileIndex::build(&profile);
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Extrude {
                profile,
                half_height,
                round: 0.1 * half_height,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("extrusão");
    // ⚠️ A MESMA caixa que o traçado usa — ver `trace_inner_tiles`.
    let bbox = ph2d_field_eval::bounds::bounding_ball(&doc, &Registry::new())
        .map(ph2d_field_eval::bounds_clip::march_clip)
        .expect("caixa");
    Piece { idx, bbox }
}

const fn uv(a: [f32; 3]) -> [f32; 2] {
    [a[0], a[1]]
}

fn box_uv_corners(lo: [f32; 3], hi: [f32; 3]) -> Vec<[f32; 2]> {
    vec![[lo[0], lo[1]], [hi[0], lo[1]], [hi[0], hi[1]], [lo[0], hi[1]]]
}

/// O casco que a especialização usaria para esta caixa e estes cantos, e as arestas que ela guarda
/// — com a MESMA queda para a caixa que o `specialised_profile` faz quando o casco degenera.
fn hull_and_kept(
    idx: &ProfileIndex,
    lo: [f32; 3],
    hi: [f32; 3],
    pts: &[[f32; 3]],
) -> (Vec<[f32; 2]>, usize) {
    let hull = ph2d_field_eval::probe_hull_uv(pts, uv(lo), uv(hi));
    if hull.len() >= 3 {
        let k = idx.probe_cull_hull(&hull);
        (hull, k)
    } else {
        (Vec::new(), idx.probe_cull(uv(lo), uv(hi)))
    }
}

/// As regiões de um quadro, na ordem (ladrilho, fatia), com a semente de fase do produto.
fn regions(bbox: Aabb, cam: &Orbit, plane: Screen, margin: f32) -> Vec<Query> {
    let (w, h) = (plane.width() as usize, plane.height() as usize);
    let (tile, slabs) = (crate::tile_for_test(), crate::slabs_for_test());
    let mut out = Vec::new();
    for ty in 0..h.div_ceil(tile) {
        for tx in 0..w.div_ceil(tile) {
            let (x0, y0) = (tx * tile, ty * tile);
            let (x1, y1) = ((x0 + tile).min(w), (y0 + tile).min(h));
            let Some((t_lo, t_hi)) = crate::tiles::tile_t_range(cam, plane, (x0, y0), (x1, y1), bbox)
            else {
                continue;
            };
            let bounds = crate::tiles::slab_bounds(t_lo, t_hi, slabs);
            for k in 0..bounds.len() - 1 {
                let Some(r) = crate::tiles::slab_region(
                    cam,
                    plane,
                    (x0, y0),
                    (x1, y1),
                    bbox,
                    margin,
                    &bounds,
                    k,
                ) else {
                    continue;
                };
                out.push(Query {
                    lo: r.lo,
                    hi: r.hi,
                    pts: r.pts,
                    // ⚠️ A MESMA semente que o `tiles::tiled_trace` usa — a fase tem de ser a do
                    // produto, senão a sonda mede outra dispersão de coortes.
                    seed: ((x0 as u64) << 40) ^ ((y0 as u64) << 20) ^ (k as u64),
                });
            }
        }
    }
    out
}

/// Os cantos levados pelo MESMO mapa que levou a caixa — escala `f` e deslocamento de fase, eixo a
/// eixo. ⚠️ É um mapa afim, então o casco dos cantos levados é o casco levado, e a folga que o
/// `hull_uv` lê da caixa cresce pelo mesmo `f`.
fn carried(pts: &[[f32; 3]], from: Aabb, to: Aabb) -> Vec<[f32; 3]> {
    pts.iter()
        .map(|p| {
            let mut q = [0.0f32; 3];
            for k in 0..3 {
                let span = from.1[k] - from.0[k];
                q[k] = if span > 0.0 {
                    to.0[k] + (p[k] - from.0[k]) * (to.1[k] - to.0[k]) / span
                } else {
                    p[k]
                };
            }
            q
        })
        .collect()
}

/// A região crescida para a fita que vai para a cache, e os cantos que o casco dela usa.
fn grown(q: &Query, g: Grow) -> (Aabb, Vec<[f32; 3]>) {
    match g {
        Grow::Scale(f) => {
            let b = crate::tape_cache::inflate_phased(q.lo, q.hi, f, q.seed, crate::tape_cache::PHASE);
            let pts = carried(&q.pts, (q.lo, q.hi), b);
            (b, pts)
        }
        Grow::Pad(d) => {
            // ⚠️ A fase é a do produto, lida da própria função: numa caixa unitária com `f = 2` o
            // deslocamento do centro É o `u ∈ [−amp, amp]` de cada eixo.
            let unit = crate::tape_cache::inflate_phased(
                [-1.0; 3],
                [1.0; 3],
                2.0,
                q.seed,
                crate::tape_cache::PHASE,
            );
            let mut b = (q.lo, q.hi);
            for k in 0..3 {
                let u = 0.5 * (unit.0[k] + unit.1[k]);
                b.0[k] = q.lo[k] - d + d * u;
                b.1[k] = q.hi[k] + d + d * u;
            }
            // Os cantos NÃO se mexem: o casco herda a folga `δ` pela caixa (ver `hull_uv`).
            (b, q.pts.clone())
        }
    }
}

fn box_inside(lo: [f32; 3], hi: [f32; 3], e: &Entry) -> bool {
    (0..3).all(|k| lo[k] >= e.lo[k] && hi[k] <= e.hi[k])
}

fn hull_inside(inner: &[[f32; 2]], outer: &[[f32; 2]]) -> bool {
    inner
        .iter()
        .all(|p| ph2d_field_eval::probe_in_hull(*p, outer))
}

/// Uma variante a medir.
#[derive(Clone, Copy)]
struct Variant {
    policy: Policy,
    grow: Grow,
    pick: Pick,
}

impl Variant {
    fn label(self) -> String {
        let pol = if self.policy == Policy::Box {
            "caixa"
        } else {
            "casco"
        };
        let g = match self.grow {
            Grow::Scale(f) => format!("f {f:.2}"),
            Grow::Pad(d) => format!("δ {d:.3}"),
        };
        let p = match self.pick {
            Pick::Oldest => "velha",
            Pick::Newest => "nova",
            Pick::Tightest => "apertada",
        };
        format!("{pol:5} {g:7} {p:8}")
    }
}

/// Um arrasto de [`FRAMES`] quadros a `graus` por quadro, com a lente a `half_extent`.
fn drag(p: &Piece, (w, h): (u32, u32), half_extent: f32, graus: f32, v: Variant) -> Tally {
    let plane = Screen::new(w, h, half_extent);
    let margin = crate::Sharpness::for_frame(half_extent, w.min(h) as usize).normal;
    let mut cache: Vec<Entry> = Vec::new();
    let mut t = Tally::default();
    for i in 0..=FRAMES {
        let cam = Orbit {
            rotation: Orbit::from_yaw_pitch(0.72 + i as f32 * graus.to_radians(), 0.52).rotation,
            half_extent,
            ..Orbit::default()
        };
        let counting = i > 0;
        for q in regions(p.bbox, &cam, plane, margin) {
            let (qhull, honest) = hull_and_kept(&p.idx, q.lo, q.hi, &q.pts);
            let probe = if qhull.is_empty() {
                box_uv_corners(q.lo, q.hi)
            } else {
                qhull
            };
            let fits = |e: &Entry| {
                box_inside(q.lo, q.hi, e)
                    && (v.policy == Policy::Box || e.hull.is_empty() || hull_inside(&probe, &e.hull))
            };
            let pos = match v.pick {
                Pick::Oldest => cache.iter().position(fits),
                Pick::Newest => cache.iter().rposition(fits),
                Pick::Tightest => cache
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| fits(e))
                    .min_by(|a, b| a.1.volume().total_cmp(&b.1.volume()))
                    .map(|(i, _)| i),
            };
            let served = if let Some(e) = pos.map(|k| &mut cache[k]) {
                e.seen = i;
                if counting {
                    t.hits += 1;
                }
                e.kept
            } else {
                let ((lo, hi), pts) = grown(&q, v.grow);
                let (hull, kept) = match v.policy {
                    Policy::Box => (Vec::new(), p.idx.probe_cull(uv(lo), uv(hi))),
                    Policy::Hull => hull_and_kept(&p.idx, lo, hi, &pts),
                };
                cache.push(Entry {
                    lo,
                    hi,
                    hull,
                    kept,
                    seen: i,
                });
                if counting {
                    t.compiles += 1;
                }
                kept
            };
            if counting {
                t.uses += 1;
                t.served += served;
                t.honest += honest;
            }
        }
        cache.retain(|e| e.seen + FRAMES_KEPT > i);
    }
    t
}

fn row(t: &Tally) -> String {
    let u = t.uses as f64;
    format!(
        "{:5.1}% | {:14.1} | {:16.1} | {:10.3}x",
        100.0 * t.hits as f64 / u,
        t.compiles as f64 / FRAMES as f64,
        t.served as f64 / u,
        t.served as f64 / t.honest.max(1) as f64,
    )
}

/// ⏳ **O QUE UMA CACHE CONTRA O CASCO COMPRARIA** — ver o doc do módulo.
#[test]
#[ignore]
fn measure_what_a_hull_cache_would_buy() {
    let v = |policy, grow, pick| Variant { policy, grow, pick };
    let pieces = [
        ("círculo 168", piece(ngon_probe(168, 0.6), 0.4)),
        ("estrela 168", piece(star_probe(168, 0.22, 0.6), 0.4)),
    ];
    let variants = [
        // O que shipa, e a refutação da ORDEM.
        v(Policy::Box, Grow::Scale(crate::INFLATE), Pick::Oldest),
        v(Policy::Box, Grow::Scale(crate::INFLATE), Pick::Newest),
        v(Policy::Box, Grow::Scale(crate::INFLATE), Pick::Tightest),
        // O casco escalado — a família que perde os acertos.
        v(Policy::Hull, Grow::Scale(1.25), Pick::Oldest),
        v(Policy::Hull, Grow::Scale(1.5), Pick::Oldest),
        // A família do crescimento ABSOLUTO.
        v(Policy::Box, Grow::Pad(0.02), Pick::Oldest),
        v(Policy::Box, Grow::Pad(0.04), Pick::Oldest),
        v(Policy::Hull, Grow::Pad(0.005), Pick::Oldest),
        v(Policy::Hull, Grow::Pad(0.01), Pick::Oldest),
        v(Policy::Hull, Grow::Pad(0.02), Pick::Oldest),
        v(Policy::Hull, Grow::Pad(0.04), Pick::Oldest),
        v(Policy::Hull, Grow::Pad(0.08), Pick::Oldest),
    ];
    let half = Orbit::default().half_extent;
    println!(
        "câmera por omissão: half_extent {half:.3} · caixa da peça {:?}",
        pieces[0].1.bbox
    );
    println!(
        "tamanho | peça        | graus | variante               | acerto | compila/quadro | arestas servidas | ÷ sem cache"
    );
    for size in [(426u32, 240u32), (640, 360)] {
        for (name, p) in &pieces {
            for graus in [1.0f32, 2.0, 4.0] {
                for var in variants {
                    let t = drag(p, size, half, graus, var);
                    assert!(t.uses > 100, "{name}: o arrasto quase não pediu regiões ({})", t.uses);
                    println!(
                        "{:>3}x{:<3} | {name:11} | {graus:5.0} | {} | {}",
                        size.0,
                        size.1,
                        var.label(),
                        row(&t)
                    );
                }
            }
        }
    }
}

/// ⏳ **DE ONDE O `δ` DEVE SAIR** — a varredura que escolhe a normalização, antes de haver constante.
///
/// ⚠️ **Duas contas puxam o `δ` para sítios diferentes, e cada uma tem a sua escala:**
/// - **o acerto** segue o movimento de um arrasto, que é `raio × ângulo` — um comprimento de MUNDO que
///   não sabe o zoom (a órbita roda um ângulo por pixel de mão, qualquer que seja a lente);
/// - **as arestas** seguem `δ` contra a largura do tubo, que é o ladrilho em MUNDO — e essa escala com
///   o zoom (`2 · half_extent · TILE / altura`).
///
/// ⇒ a varredura muda as duas escalas **separadamente** — o zoom (`half_extent`) e o tamanho da peça
/// — e exprime o `δ` em fracção do [`Piece::reach`], imprimindo ao lado quanto ele vale em ladrilhos.
/// *Se o melhor ponto ficar na mesma fracção do alcance em todas as linhas, é daí que o `δ` sai; se
/// ficar no mesmo número de ladrilhos, é de lá.*
#[test]
#[ignore]
fn measure_where_the_pad_should_come_from() {
    let (w, h) = (426u32, 240u32);
    let tile = crate::tile_for_test() as f32;
    println!(
        "zoom | escala | alcance | graus | variante               | δ/ladrilho | acerto | compila/quadro | arestas servidas | ÷ sem cache"
    );
    for half_extent in [0.4f32, 0.8, 1.6] {
        for escala in [0.5f32, 1.0, 2.0] {
            let p = piece(ngon_probe(168, 0.6 * f64::from(escala)), 0.4 * escala);
            let reach = p.reach();
            // O ladrilho em MUNDO no plano do alvo: `2 · half_extent` cobre a altura do quadro.
            let tile_world = 2.0 * half_extent * tile / h as f32;
            for graus in [2.0f32, 4.0] {
                let mut variants = vec![Variant {
                    policy: Policy::Box,
                    grow: Grow::Scale(crate::INFLATE),
                    pick: Pick::Oldest,
                }];
                for frac in [0.02f32, 0.04, 0.06, 0.08, 0.12] {
                    variants.push(Variant {
                        policy: Policy::Hull,
                        grow: Grow::Pad(frac * reach),
                        pick: Pick::Oldest,
                    });
                }
                for var in variants {
                    let t = drag(&p, (w, h), half_extent, graus, var);
                    if t.uses < 100 {
                        println!(
                            "{half_extent:4.1} | {escala:6.1} | {reach:7.3} | {graus:5.0} | {} | (a peça quase não aparece: {} regiões)",
                            var.label(),
                            t.uses
                        );
                        continue;
                    }
                    let per_tile = match var.grow {
                        Grow::Pad(d) => d / tile_world,
                        Grow::Scale(_) => f32::NAN,
                    };
                    println!(
                        "{half_extent:4.1} | {escala:6.1} | {reach:7.3} | {graus:5.0} | {} | {per_tile:10.2} | {}",
                        var.label(),
                        row(&t)
                    );
                }
            }
        }
    }
}
