//! `ph2d-field-profile` — **o desenho do editor vetorial virando perfil de sólido** ([ADR-0161]).
//!
//! Uma crate, uma pergunta: *como é que um `VecPath` se torna um [`Profile`]?* É aqui que o fluxo do
//! MoI renasce sobre a caneta que a casa já tem — desenha-se o contorno no editor de vetores e
//! extruda-se ou revoluciona-se.
//!
//! # ⭐ O arredondamento de quina do perfil vem de graça, e é de propósito
//!
//! O cozimento parte de [`VecPath::cooked`], que é a geometria **já com os Live Corners aplicados**
//! ([ADR-0121]). Logo o *corner widget* do editor vetorial é o arredondamento das arestas verticais
//! da extrusão — o módulo 3D não tem, e não deve ter, uma segunda resposta para "arredondar a quina
//! de um contorno". *Uma quina, um dono.*
//!
//! A pilha de Live Path Effects ([ADR-0132]) entra pelo mesmo caminho: `cooked()` já a correu.
//!
//! # ⚠️ O que é COZIDO aqui não é reversível, e é por isso que a tolerância viaja junto
//!
//! O que sai é uma polilinha. A curva original fica no documento vetorial, que continua a ser a
//! **fonte**; o [`Profile`] é o **cozido**, e leva dentro de si a tolerância com que foi feito, para
//! que "este perfil está bom?" tenha resposta sem adivinhação.
//!
//! # O eixo Y
//!
//! ⚠️ **A conversão não vira nem espelha nada**: o `(x, y)` do path é o `(x, y)` do perfil, e há
//! gate a fixá-lo. Se o plano de desenho de uma ferramenta tiver o Y para baixo, quem espelha é a
//! **ferramenta**, na hora de escolher o plano — não esta função, que não sabe de que plano o
//! desenho veio.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md
//! [ADR-0121]: ../../../docs/architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md
//! [ADR-0132]: ../../../docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md

use kurbo::{BezPath, PathEl, Point};
use ph2d_field::{FillRule, Profile, ProfileError};
use ph2d_vec_scene::{VecPath, VecVertex};

/// Tolerância de achatamento como **fração da maior dimensão do perfil**, usada por
/// [`cook_path_auto`].
///
/// ⚠️ **Este número é MEDIDO, não escolhido** (`CLAUDE.md §0`). O que ele controla é o número de
/// arestas, e o número de arestas é literalmente o custo do traçado — cada aresta são ~26 nós de
/// árvore avaliados **por pixel**. A tabela, a 640×480 com 32 threads
/// (`ph2d-field-render::measure_profile_trace_cost`):
///
/// | arestas | traçado paralelo |
/// |---|---|
/// | 32 | 16,0 ms |
/// | 64 | 24,1 ms |
/// | 128 | 43,6 ms |
/// | 256 | 80,6 ms |
///
/// O baseline do módulo (a junção de três cilindros da W0) custa 25 ms no mesmo quadro, então **~64
/// arestas é o orçamento** que mantém um perfil no mesmo preço de uma peça de primitivas. Para um
/// contorno redondo, o número de arestas que uma tolerância `ε` produz é `≈ 2,22·√(R/ε)`; com
/// `ε = 10⁻³·D` (D = a maior dimensão, R ≈ D/2) isso dá **≈ 50 arestas**. É daí que sai o `1e-3`.
///
/// ⚠️ **É uma FRAÇÃO e não um absoluto** porque a mesma peça desenhada em milímetros ou em metros
/// tem de sair com a mesma qualidade — uma tolerância absoluta faria a unidade do documento decidir
/// a suavidade da forma.
pub const TOLERANCE_RATIO: f64 = 1e-3;

/// Por que um path não pôde virar perfil.
#[derive(Clone, Debug, PartialEq)]
pub enum CookError {
    /// Um contorno **aberto**. Um perfil delimita área, e um contorno aberto não delimita nada.
    ///
    /// ⚠️ Recusa em vez de ignorar: saltar o contorno aberto em silêncio daria um sólido que é *quase*
    /// o desenho, e a diferença só apareceria como um buraco que não fechou.
    OpenContour { contour: u32 },
    /// O path não tem contorno nenhum com pontos.
    Empty,
    /// A polilinha saiu, e o documento a recusou. Ver [`ProfileError`].
    Rejected(ProfileError),
}

/// **Coze um path do editor vetorial num perfil**, com a tolerância dada em unidades do documento.
///
/// # Errors
/// Ver [`CookError`].
pub fn cook_path(path: &VecPath, tolerance: f64) -> Result<Profile, CookError> {
    // A geometria COZIDA: Live Corners e a pilha de efeitos já correram. É a forma que está na tela,
    // e é ela que tem de virar sólido.
    let path = &*path.cooked();

    let mut contours: Vec<Vec<[f32; 2]>> = Vec::with_capacity(path.contour_count());
    for c in 0..path.contour_count() {
        let Some((verts, closed)) = path.contour(c) else {
            continue;
        };
        if verts.is_empty() {
            continue;
        }
        if !closed {
            return Err(CookError::OpenContour { contour: c as u32 });
        }
        if verts.len() < 2 {
            // Um contorno "fechado" de um ponto só não tem aresta nenhuma — deixa-se cair, e o
            // `Profile` recusa depois se não sobrar nada.
            continue;
        }
        contours.push(flatten_contour(verts, tolerance));
    }
    if contours.is_empty() {
        return Err(CookError::Empty);
    }
    Profile::new(contours, fill_rule(path.fill_rule), tolerance as f32).map_err(CookError::Rejected)
}

/// O mesmo, com a tolerância **derivada do tamanho do desenho** — `TOLERANCE_RATIO` da maior
/// dimensão da caixa envolvente.
///
/// É a porta normal: uma tolerância absoluta obriga quem chama a saber a escala do documento, e
/// errar nisso é ou um perfil facetado ou um traçado dez vezes mais caro.
///
/// # Errors
/// Ver [`CookError`].
pub fn cook_path_auto(path: &VecPath) -> Result<Profile, CookError> {
    let cooked = path.cooked();
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for c in 0..cooked.contour_count() {
        let Some((verts, _)) = cooked.contour(c) else {
            continue;
        };
        for v in verts {
            // ⚠️ Os HANDLES entram na conta, e não só as âncoras: uma curva pode sair da caixa das
            // âncoras, e uma tolerância derivada de uma caixa pequena demais faz o achatamento
            // trabalhar de mais exatamente onde a curva é mais larga.
            for p in [v.anchor, v.in_handle, v.out_handle] {
                for k in 0..2 {
                    min[k] = min[k].min(p[k]);
                    max[k] = max[k].max(p[k]);
                }
            }
        }
    }
    let span = (max[0] - min[0]).max(max[1] - min[1]);
    // Um desenho sem extensão não tem escala de onde tirar tolerância; o `Profile` recusa-o logo a
    // seguir, e um número positivo qualquer chega lá.
    let tolerance = if span.is_finite() && span > 0.0 {
        span * TOLERANCE_RATIO
    } else {
        TOLERANCE_RATIO
    };
    cook_path(&cooked, tolerance)
}

fn fill_rule(r: ph2d_vec_scene::FillRule) -> FillRule {
    match r {
        ph2d_vec_scene::FillRule::NonZero => FillRule::NonZero,
        ph2d_vec_scene::FillRule::EvenOdd => FillRule::EvenOdd,
    }
}

/// Um contorno fechado de vértices cúbicos → polilinha.
fn flatten_contour(verts: &[VecVertex], tolerance: f64) -> Vec<[f32; 2]> {
    let mut bez = BezPath::new();
    bez.move_to(pt(verts[0].anchor));
    for i in 0..verts.len() {
        let a = &verts[i];
        let b = &verts[(i + 1) % verts.len()];
        // ⚠️ O segmento de FECHO (último → primeiro) entra pelo `%` — é o mesmo laço, e não um caso
        // à parte depois dele. Um caso à parte é onde este tipo de conversão costuma perder a
        // aresta que fecha a figura.
        if a.out_handle == a.anchor && b.in_handle == b.anchor {
            // Reta exata: não passa pelo achatamento. Não é otimização de gosto — mandar uma cúbica
            // degenerada ao flattener é pedir-lhe uma decisão sobre uma curva que não existe.
            bez.line_to(pt(b.anchor));
        } else {
            bez.curve_to(pt(a.out_handle), pt(b.in_handle), pt(b.anchor));
        }
    }
    bez.close_path();

    let mut out: Vec<[f32; 2]> = Vec::new();
    kurbo::flatten(bez, tolerance, |el| match el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => out.push([p.x as f32, p.y as f32]),
        // `flatten` só emite retas; o fecho é implícito no `Profile` (que não repete o 1º ponto).
        _ => {}
    });
    out
}

fn pt(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
}

#[cfg(test)]
mod tests;
