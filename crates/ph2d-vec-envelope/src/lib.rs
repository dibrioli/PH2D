//! Deformação **não-afim** de geometria Bézier — a espinha do envelope / puppet warp (ADR-0129).
//!
//! # A armadilha, e por que esta crate existe
//!
//! Só transformações **afins** comutam com a avaliação de Bézier (elas preservam combinações
//! convexas). Logo isto **está errado**:
//!
//! ```ignore
//! for v in verts { v.anchor = warp(v.anchor); }   // NÃO
//! ```
//!
//! A curva resultante **não é a imagem** da curva original. E erra de um jeito traiçoeiro: sob um
//! mapa suave ela acerta a verdade em `t=0` e `t=1` **exatamente**, e no interior **nunca**. Ou
//! seja: as alças pousam onde o artista espera, e só a curva *entre* elas mente. É por isso que
//! este bug shipa em produto sério (Blender, Rive, Skia) e é por isso que o gate desta crate é
//! **invariância à subdivisão**, não inspeção a olho.
//!
//! O argumento que fecha a questão **não é a precisão, é o DOMÍNIO**: a alça de uma cúbica vive
//! *fora* do casco convexo da curva, e uma gaiola é um domínio *limitado* — então o caminho ingênuo
//! amostra o campo num ponto que pode nem estar na gaiola, onde o mapa é indefinido. Precisão se
//! calibra; domínio não.
//!
//! # O que fazemos
//!
//! `amostrar W(C(t)) + J_W·C'(t)` → [`kurbo::fit_to_bezpath`]. Nada de composição simbólica (medido:
//! é o *mesmo* problema de aproximação, e o grau 18 não tem onde morar) e nada de Bézier racional
//! (exigiria peso por ponto de controle, que o [`VecVertex`] não tem). Ver ADR-0129 §2.

use kurbo::{BezPath, CubicBez, ParamCurve, ParamCurveDeriv, PathEl, Point, Vec2};
use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};

mod coons;
mod fit;
mod gesture;
mod mls;
mod preset;
mod quad;

pub use coons::{CageEdges, CoonsWarp, cage_folds, rest_edges};
pub use fit::WarpedCubic;
pub use gesture::{
    CageMove, MESH_HANDLE_COUNT, edge_handle_index, move_corner_convex, move_handle,
    nearest_corner, nearest_handle,
};
pub use mls::{MlsWarp, Pin, pins_fold};
pub use preset::{AMP, EdgeBows, UNIT_CAGE, preset_cage};
pub use quad::QuadWarp;

/// Um mapa `R2 -> R2` **arbitrário** (afim ou não), com a sua diferencial.
///
/// É o único ponto de extensão da crate: todo gesto de envelope (preset, quad/perspectiva, 4
/// curvas de lado, pinos) é um `Warp` diferente plugado na **mesma** espinha. Foi essa unificação
/// que o ADR-0129 §4 decidiu — envelope e puppet não são features distintas, são dois gestos sobre
/// um pipeline.
pub trait Warp {
    /// A imagem de `p`.
    fn map(&self, p: [f64; 2]) -> [f64; 2];

    /// A jacobiana em `p`, em ordem de linha: `[[dx/du, dx/dv], [dy/du, dy/dv]]`.
    ///
    /// É o que dá a derivada da curva deformada pela regra da cadeia — e é por isso que o `Warp`
    /// a exige em vez de estimá-la por diferença finita: para todo mapa que o ADR-0129 adota
    /// (homografia, Coons, presets, MLS) ela é **fechada**, e uma diferença finita trocaria uma
    /// derivada exata por um epsilon inventado.
    ///
    /// # Contrato: ela tem de ser a derivada REAL de [`Warp::map`]
    ///
    /// Não é pedantismo, e o sintoma de quebrá-lo é **surpreendente**: uma jacobiana inconsistente
    /// com o `map` não produz uma imagem errada — ela faz o `fit_to_bezpath` **não convergir**. O
    /// fitter subdivide para reconciliar um ponto e uma tangente que não pertencem à mesma curva, e
    /// nunca consegue. Medido: removendo a regra da cadeia daqui, os gates que exercitam
    /// [`warp_path`] **travam**; os que não a exercitam passam intactos.
    ///
    /// Ou seja: um `Warp` novo com jacobiana errada falha **alto**, não em silêncio. Isso é bom — e
    /// é bom saber de antemão, para não caçar o bug no lugar errado.
    fn jacobian(&self, p: [f64; 2]) -> [[f64; 2]; 2];
}

/// Deforma um path inteiro (todos os contornos) por `warp`, refitando a `accuracy`.
///
/// `accuracy` é distância de **Fréchet** (a garantia do `fit_to_bezpath`, que a própria doc do
/// kurbo qualifica como *"not a rigorous guarantee, as the error metric is computed
/// approximately"*). É a mesma grandeza que o Fidelity do Illustrator vende como porcentagem — só
/// que lá é **inserção por contagem**, uniforme, e aqui é fit adaptativo.
///
/// **Um mapa afim volta byte-idêntico?** Não: volta *geometricamente* idêntico, mas o fit pode
/// reparametrizar — quem quer afim exato aplica o afim, não o envelope. O gate
/// `an_affine_warp_agrees_with_the_naive_path` fixa isso como piso: sob afim, este motor e o
/// caminho ingênuo (que ali está CERTO) coincidem, e o resíduo medido é do instrumento.
#[must_use]
pub fn warp_path(path: &VecPath, warp: &impl Warp, accuracy: f64) -> VecPath {
    let mut out = path.clone();
    if let Some(v) = warp_contour(&path.verts, path.closed, warp, accuracy) {
        out.verts = v;
    }
    for (i, s) in path.subpaths.iter().enumerate() {
        if let Some(v) = warp_contour(&s.verts, s.closed, warp, accuracy) {
            out.subpaths[i].verts = v;
        }
    }
    out
}

/// Deforma **um** contorno. `None` quando não há segmento nenhum (contorno degenerado) — o chamador
/// mantém a fonte, em vez de trocá-la por uma lista vazia.
///
/// # Granularidade: por SEGMENTO, e é decisão, não atalho
///
/// Cada segmento-fonte é fitado por si, então **toda âncora autorada sobrevive** e o fit só
/// *acrescenta* onde a curvatura do mapa exige. É o oposto do Illustrator, cuja patente descreve o
/// Fidelity como *"a variable frequency of insertion"* de pontos **uniformemente** — inserindo onde
/// não precisa e por isso cuspindo pontos no Expand.
///
/// Fitar o trecho inteiro **entre quinas** (deixando o fitter escolher onde pôr âncora) reduz mais
/// a contagem, mas descarta âncoras que o artista pôs de propósito. É otimização de uma fatia
/// futura, e ela tem de ser medida contra este piso, não assumida melhor.
#[must_use]
fn warp_contour(
    verts: &[VecVertex],
    closed: bool,
    warp: &impl Warp,
    accuracy: f64,
) -> Option<Vec<VecVertex>> {
    let n = verts.len();
    let seg_count = segment_count(n, closed)?;

    // Cada segmento vira 1+ cúbicas. Acumulamos os pontos de controle em ordem e remontamos os
    // vértices no fim: um vértice do PH2D é (âncora, in, out), e o `out` de um vértice e o `in` do
    // seguinte pertencem a segmentos diferentes — remontar durante o laço obrigaria a olhar para
    // trás a cada passo.
    let mut pts: Vec<[Point; 3]> = Vec::new(); // (p1, p2, p3) por cúbica emitida
    let start = warp_point(warp, verts[0].anchor);

    for s in 0..seg_count {
        let src = segment(verts, s, n);
        let fitted = fit::fit_warped(&src, warp, accuracy);
        push_cubics(&fitted, &mut pts);
    }

    Some(rebuild(&pts, start, closed))
}

/// Quantos segmentos tem um contorno de `n` vértices. `None` = nenhum (não há o que deformar).
fn segment_count(n: usize, closed: bool) -> Option<usize> {
    let c = if closed { n } else { n.checked_sub(1)? };
    (c > 0).then_some(c)
}

/// O segmento `s` como cúbica de kurbo. Fechado: o último liga de volta ao primeiro.
fn segment(verts: &[VecVertex], s: usize, n: usize) -> CubicBez {
    let a = &verts[s];
    let b = &verts[(s + 1) % n];
    CubicBez::new(
        pt(a.anchor),
        pt(a.out_handle),
        pt(b.in_handle),
        pt(b.anchor),
    )
}

fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

fn warp_point(warp: &impl Warp, p: [f64; 2]) -> Point {
    let q = warp.map(p);
    Point::new(q[0], q[1])
}

/// Extrai as cúbicas de um `BezPath` fitado. O `fit_to_bezpath` emite `MoveTo` + `CurveTo`s; uma
/// `LineTo` aparece quando o trecho é reto o bastante (o `try_fit_line` interno do kurbo) — e nós a
/// **elevamos a cúbica na convenção (⅓, ⅔)**, jamais para `(P0,P0,P3,P3)`.
///
/// Isso não é gosto: a `ph2d-vec-boolean::to_bez` **testa** a forma degenerada para emitir `line_to`,
/// e o `corner_live::sub_cubic` já pagou o preço de a de Casteljau envenenar essa convenção
/// (BUGS #9). A forma (⅓, ⅔) é afim em `t` e não mente para ninguém.
fn push_cubics(path: &BezPath, pts: &mut Vec<[Point; 3]>) {
    let mut cur = Point::ZERO;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => cur = p,
            PathEl::LineTo(p) => {
                let d = p - cur;
                pts.push([cur + d / 3.0, cur + d * (2.0 / 3.0), p]);
                cur = p;
            }
            PathEl::CurveTo(p1, p2, p3) => {
                pts.push([p1, p2, p3]);
                cur = p3;
            }
            PathEl::QuadTo(p1, p2) => {
                // O fit não emite quadráticas, mas o tipo permite — elevar é exato, então
                // aceitamos em vez de entrar em pânico ou descartar geometria em silêncio.
                let (c1, c2) = quad_to_cubic(cur, p1, p2);
                pts.push([c1, c2, p2]);
                cur = p2;
            }
            PathEl::ClosePath => {}
        }
    }
}

/// Elevação exata de quadrática para cúbica.
fn quad_to_cubic(p0: Point, p1: Point, p2: Point) -> (Point, Point) {
    (p0 + (p1 - p0) * (2.0 / 3.0), p2 + (p1 - p2) * (2.0 / 3.0))
}

/// Remonta os vértices a partir das cúbicas acumuladas.
///
/// Num contorno **fechado**, a última cúbica termina na âncora inicial: o `in_handle` dela pertence
/// ao vértice 0, e emitir um vértice a mais duplicaria a âncora de partida.
fn rebuild(pts: &[[Point; 3]], start: Point, closed: bool) -> Vec<VecVertex> {
    let mut out: Vec<VecVertex> = Vec::with_capacity(pts.len() + 1);
    let mut anchor = start;
    let mut pending_in: Option<Point> = None;

    for (i, [p1, p2, p3]) in pts.iter().enumerate() {
        let last = i + 1 == pts.len();
        out.push(VecVertex {
            anchor: [anchor.x, anchor.y],
            in_handle: pending_in.map_or([anchor.x, anchor.y], |p| [p.x, p.y]),
            out_handle: [p1.x, p1.y],
            // O fit não conhece a intenção do artista, e um join derivado não é autoria: marcar
            // `Smooth` afirmaria colinearidade que o fit não garante. `Corner` é o tipo honesto —
            // ele não RESTRINGE nada, só descreve handles independentes.
            kind: VertexKind::Corner,
            // A quina foi COZIDA na deformação; um raio sobrevivente re-arredondaria o já
            // arredondado. Mesma regra da booleana (ADR-0121 §4).
            corner_radius: 0.0,
        });
        pending_in = Some(*p2);
        anchor = *p3;
        if last && !closed {
            out.push(VecVertex {
                anchor: [anchor.x, anchor.y],
                in_handle: [p2.x, p2.y],
                out_handle: [anchor.x, anchor.y],
                kind: VertexKind::Corner,
                corner_radius: 0.0,
            });
        }
    }

    if closed && let (Some(p2), Some(first)) = (pending_in, out.first_mut()) {
        first.in_handle = [p2.x, p2.y];
    }
    out
}

/// A imagem de um ponto e da derivada da curva nele, pela regra da cadeia: `J_W(C(t)) · C'(t)`.
///
/// É o coração da crate e cabe em duas linhas — o resto é encanamento.
pub(crate) fn map_pt_deriv(warp: &impl Warp, c: &CubicBez, t: f64) -> (Point, Vec2) {
    let p = c.eval(t);
    let d = c.deriv().eval(t).to_vec2();
    let j = warp.jacobian([p.x, p.y]);
    let q = warp.map([p.x, p.y]);
    (
        Point::new(q[0], q[1]),
        Vec2::new(j[0][0] * d.x + j[0][1] * d.y, j[1][0] * d.x + j[1][1] * d.y),
    )
}
