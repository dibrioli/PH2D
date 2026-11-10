//! O texto da lista, medido antes de pintar: cada um entra no censo com «coube?».

use ph2d_editor_core::paint::{
    folga_do_texto, paint_text_elided, paint_text_elided_weighted, paint_text_title_elided,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::Color;
use ph2d_vector::VectorScene;

use crate::census::Census;

/// «Não coube?» com a MESMA comparação do pintor (`text_elide::coube`: `largura <= orçamento`, sem folga): uma folga
/// aqui deixava passar o corte de um ulp que o pintor faz (o «F1» da C4 saía «…» e o censo dizia que coube).
fn cortado(w: f32, budget: f32) -> bool {
    w > budget
}

/// Pinta `s` numa linha dentro de `r` (centrado na vertical), regista-o no censo, e devolve a largura usada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn line(
    census: &mut Census,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    px: f32,
    color: Color,
) -> f32 {
    let w = ts.prefix_width(s, px);
    census.texts.push((r, s.to_owned(), cortado(w, r.w)));
    paint_text_elided(
        ts,
        scene,
        s,
        r.x,
        r.y + folga_do_texto(r.h, px),
        px,
        r.w,
        color,
    );
    w.min(r.w)
}

/// O peso do corpo da A e da B (maquetes 01 e 03): texto, chips, pés e dicas em Regular; o SemiBold fica para o
/// destaque e os nomes das acções ([`title`]).
pub(crate) const BODY: FontWeight = FontWeight::NORMAL;

/// A largura de `s` no peso do corpo ([`body`]).
pub(crate) fn body_width(ts: &mut TextSystem, s: &str, px: f32) -> f32 {
    ts.prefix_width_weighted(s, px, BODY)
}

/// [`line`] no peso do corpo (Regular): mede, corta e pinta com o MESMO peso, e entra no censo `regular`.
pub(crate) fn body(
    census: &mut Census,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    px: f32,
    color: Color,
) -> f32 {
    let w = body_width(ts, s, px);
    census.texts.push((r, s.to_owned(), cortado(w, r.w)));
    census.regular.push(s.to_owned());
    let at = [r.x, r.y + folga_do_texto(r.h, px)];
    paint_text_elided_weighted(ts, scene, s, at, px, r.w, (color, BODY));
    w.min(r.w)
}

/// `s` partido pelas palavras: a 1.ª linha tem `first` (o que sobra ao lado de um destaque), as outras `w`. Uma
/// 1.ª linha onde nem a primeira palavra cabe fica vazia (o resto começa por baixo), nunca cortada.
pub(crate) fn wrap_after(ts: &mut TextSystem, s: &str, px: f32, first: f32, w: f32) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut budget = first;
    for word in s.split(' ').filter(|w| !w.is_empty()) {
        let next = if cur.is_empty() {
            word.to_owned()
        } else {
            format!("{cur} {word}")
        };
        if ts.prefix_width(&next, px) <= budget {
            cur = next;
        } else {
            out.push(std::mem::replace(&mut cur, word.to_owned()));
            budget = w;
        }
    }
    out.push(cur);
    out
}

/// A largura de `s` em SemiBold (o peso de [`title`]).
pub(crate) fn title_width(ts: &mut TextSystem, s: &str, px: f32) -> f32 {
    ts.prefix_width_weighted(s, px, FontWeight::SEMI_BOLD)
}

/// [`line`] em SemiBold (o nome da acção e o destaque do pé, maquete A): mede e pinta com o MESMO peso.
#[allow(clippy::too_many_arguments)]
pub(crate) fn title(
    census: &mut Census,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    px: f32,
    color: Color,
) -> f32 {
    let w = title_width(ts, s, px);
    census.texts.push((r, s.to_owned(), cortado(w, r.w)));
    census.semibold.push(s.to_owned());
    paint_text_title_elided(
        ts,
        scene,
        s,
        r.x,
        r.y + folga_do_texto(r.h, px),
        px,
        r.w,
        color,
    );
    w.min(r.w)
}
