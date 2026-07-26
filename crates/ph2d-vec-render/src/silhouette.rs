//! **A SILHUETA em segmentos** — o que o campo de distância dos FX raster precisa saber sobre a
//! forma, em espaço de texel do scratch.
//!
//! Irmão de [`super::draw_path_isolated`] **de propósito**: as duas respondem à mesma pergunta (*o
//! que esta forma desenha, isolada, nesta escala?*) e resolvem o transform pela mesma via. Duas
//! portas divergiriam, e a divergência aqui é muda: o campo descreveria uma forma e o raster
//! desenharia outra.
//!
//! ⚠️ **Por que a geometria, se já há o raster.** A rampa de anti-aliasing ocupa 1,0–1,41 texel e o
//! estêncil que a mede ocupa 2, então a estimativa da fronteira pela COBERTURA sempre inclui
//! amostras saturadas, e o recorte é função da FASE do texel na escada de rasterização. Medido: até
//! 0,68 px de erro de profundidade, e — numa aresta a 4,6°, onde a escada tem passo de 12,4 texels
//! e um 3×3 lê a aresta como horizontal — erro de direção igual ao **ângulo inteiro**. O feather lê
//! esse campo pela distância e o bevel pela direção; renderizado, o primeiro sai com dentes e o
//! segundo com um pente. Com o pé exato: bevel 59,25 → 2,46 níveis de ondulação, feather 5,56 →
//! 0,82 (o controle liso dá 2,40 / 0,00).

use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms};
use ph2d_vector::{Affine, PathEl, Point};

use crate::{LiveGeometry, build_fill_bezpath, path_to_screen};

/// Erro máximo do achatamento, em texels. Bem abaixo do que a wave persegue (o campo mira ~0,05
/// texel), então o achatamento não é o termo dominante.
const FLATTEN_TOL: f64 = 0.02;

/// Teto de segmentos. ⚠️ **É de CUSTO, e o número é MEDIDO** (`fx_stack_segment_cost`, na
/// `ph2d-render`): o passe de campo é `O(texels × segmentos)` e, numa textura de 512² — a maior que
/// a pilha aloca numa forma de tela cheia —, custa **0,72 ms sem geometria · 0,51 ms com 64
/// segmentos · 3,94 ms com 4096**.
///
/// ⚠️ **O caminho exato é mais BARATO que o do raster no caso comum**, e não por acaso: havendo
/// silhueta, os passes de semente e de salto do JFA não são despachados — o finalize já computa o
/// pé por texel. Trocou-se um número de passes proporcional à banda por um laço proporcional à
/// complexidade da forma.
///
/// O teto é onde um passe ainda cabe num frame de 60 fps com folga para o resto da pilha;
/// estourá-lo devolve vazio, e o campo cai no caminho do raster — pior, mas nunca trava.
pub const MAX_SEGMENTS: usize = 4096;

/// Os segmentos da silhueta de `id`, já no espaço de texel do scratch (`offset * camera`).
///
/// Vazio significa *"não sei responder com exatidão"*, e o chamador cai no caminho do raster. Hoje
/// isso acontece em dois casos, os dois deliberados:
///
/// - **forma com TRAÇO** — a silhueta desenhada é `preenchimento ∪ contorno-do-traço`, e a borda
///   INTERNA do traço não é fronteira de silhueta nenhuma: semeá-la poria a fronteira no meio da
///   forma. A união exata é trabalho da booleana, e entra quando houver quem a peça.
/// - **estouro do teto** de segmentos.
#[must_use]
pub fn silhouette_segments(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    camera: Affine,
    offset: Affine,
) -> Vec<[f32; 4]> {
    let mut out = Vec::new();
    if let Some(items) = live.get(&id) {
        for item in items {
            if !push_path(&mut out, item, offset * camera) {
                return Vec::new();
            }
        }
    } else if let Some(path) = scene.paths().iter().find(|p| p.id == id)
        && !push_path(&mut out, path, offset * path_to_screen(xforms, id, camera))
    {
        return Vec::new();
    }
    if out.is_empty() || out.len() > MAX_SEGMENTS {
        return Vec::new();
    }
    out
}

/// `false` = esta forma não tem resposta exata barata (ver o doc de [`silhouette_segments`]).
fn push_path(out: &mut Vec<[f32; 4]>, path: &VecPath, transform: Affine) -> bool {
    if path.stroke.is_some() || path.fill.is_none() {
        return false;
    }
    // A MESMA porta que o desenho usa para o preenchimento — os contornos ABERTOS ficam de fora lá
    // e têm de ficar aqui (eles não têm interior, então não são fronteira de silhueta).
    flatten_into(out, &build_fill_bezpath(path), transform);
    true
}

/// Achata o `BezPath` já TRANSFORMADO em segmentos, fechando cada subpath.
fn flatten_into(out: &mut Vec<[f32; 4]>, bp: &ph2d_vector::BezPath, transform: Affine) {
    let mut start = Point::ZERO;
    let mut cur = Point::ZERO;
    let mut push = |a: Point, b: Point, out: &mut Vec<[f32; 4]>| {
        if a != b {
            out.push([a.x as f32, a.y as f32, b.x as f32, b.y as f32]);
        }
    };
    for el in bp.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                let p = transform * p;
                start = p;
                cur = p;
            }
            PathEl::LineTo(p) => {
                let p = transform * p;
                push(cur, p, out);
                cur = p;
            }
            PathEl::QuadTo(c, p) => {
                let (c, p) = (transform * c, transform * p);
                cur = quad(out, cur, c, p, &mut push);
            }
            PathEl::CurveTo(c1, c2, p) => {
                let (c1, c2, p) = (transform * c1, transform * c2, transform * p);
                cur = cubic(out, cur, c1, c2, p, &mut push);
            }
            PathEl::ClosePath => {
                push(cur, start, out);
                cur = start;
            }
        }
    }
    // Um subpath de preenchimento não fechado explicitamente ainda é uma região FECHADA para o
    // rasterizador — a silhueta tem de concordar com o que é pintado.
    push(cur, start, out);
}

/// Quantos pedaços uma curva precisa para ficar dentro de [`FLATTEN_TOL`]. O desvio de uma curva
/// para a corda cai com `n²`, e o polígono de controle limita o desvio inicial.
fn steps(hull: f64) -> usize {
    ((hull / (8.0 * FLATTEN_TOL)).sqrt().ceil() as usize).clamp(1, 256)
}

fn quad<F>(out: &mut Vec<[f32; 4]>, p0: Point, c: Point, p1: Point, push: &mut F) -> Point
where
    F: FnMut(Point, Point, &mut Vec<[f32; 4]>),
{
    let n = steps((c - p0).hypot() + (p1 - c).hypot());
    let mut prev = p0;
    for i in 1..=n {
        let t = i as f64 / n as f64;
        let u = 1.0 - t;
        let p = ((p0.to_vec2() * u + c.to_vec2() * t) * u
            + (c.to_vec2() * u + p1.to_vec2() * t) * t)
            .to_point();
        push(prev, p, out);
        prev = p;
    }
    prev
}

fn cubic<F>(
    out: &mut Vec<[f32; 4]>,
    p0: Point,
    c1: Point,
    c2: Point,
    p1: Point,
    push: &mut F,
) -> Point
where
    F: FnMut(Point, Point, &mut Vec<[f32; 4]>),
{
    let n = steps((c1 - p0).hypot() + (c2 - c1).hypot() + (p1 - c2).hypot());
    let mut prev = p0;
    for i in 1..=n {
        let t = i as f64 / n as f64;
        let u = 1.0 - t;
        let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        let p =
            (p0.to_vec2() * a + c1.to_vec2() * b + c2.to_vec2() * c + p1.to_vec2() * d).to_point();
        push(prev, p, out);
        prev = p;
    }
    prev
}
