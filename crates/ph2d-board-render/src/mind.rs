//! **O mapa mental no desenho** (W6): os ramos — a curva MEDIDA do Miro, derivada da relação
//! pai→filho (M13) — e o selo de cada ramo recolhido com o número escondido (M10); por cima, o «+»
//! do nó seleccionado e o realce do nó onde o arrasto largaria o que leva.

use ph2d_board_edit::digits;
use ph2d_board_edit::{Metrics, Overlay};
use ph2d_board_model::{Board, Element, MIND_BRANCH_WIDTH, MIND_FONT, RichText};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Point, Rect, RoundedRect, Shape as _, Stroke, VectorScene,
};

use crate::{RenderCache, doc_color, point, token};

/// A letra do número no selo, em fracção da altura dele.
const BADGE_TEXT: f64 = 0.7;
/// O raio dos cantos do selo, em fracção da altura dele.
const BADGE_ROUND: f64 = 0.25;
/// O sal da chave do texto do selo na cache (o do nó usa o id cru).
const BADGE_SALT: u64 = 1 << 62;

/// A letra do mapa de `id` (a da raiz: o Miro escala o mapa todo por ela).
fn map_font(board: &Board, id: ph2d_board_model::ElementId) -> f64 {
    board
        .doc
        .mind_root(id)
        .and_then(|r| board.doc.get(r))
        .map_or(MIND_FONT, |r| r.style().font_size)
}

/// O ramo que chega a `el`, pintado logo ANTES dele — debaixo dele e no recorte do frame onde ele
/// vive (pintado à parte, atravessava a borda do frame — auditoria W6).
pub(crate) fn paint_branch(
    scene: &mut VectorScene,
    board: &Board,
    el: &Element,
    v: Affine,
    clip: Rect,
) {
    let Some([p0, p1, p2, p3]) = ph2d_board_model::mind_branch(&board.doc, el.id) else {
        return;
    };
    let Some(m) = el.mind() else {
        return;
    };
    let mut path = BezPath::new();
    path.move_to(point(p0));
    path.curve_to(point(p1), point(p2), point(p3));
    let path = v * path;
    let width = MIND_BRANCH_WIDTH * map_font(board, el.id) / MIND_FONT * board.camera.zoom;
    let on_screen = path
        .bounding_box()
        .inflate(width, width)
        .intersect(clip)
        .area()
        > 0.0;
    if on_screen {
        scene.stroke_path(
            &Stroke::new(width),
            Affine::IDENTITY,
            &Brush::Solid(doc_color(m.color)),
            &path,
        );
    }
}

/// O selo de `el` se é um ramo recolhido: a cor do ramo, o número de escondidos na tinta que se lê
/// nela (o branco lia-se a 1,4:1 no verde-água — auditoria W6). Pintado logo DEPOIS do nó, no
/// mesmo recorte.
pub(crate) fn paint_badge(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    board: &Board,
    el: &Element,
    v: Affine,
) {
    let Some(m) = el.mind().filter(|m| m.collapsed && !el.is_hidden()) else {
        return;
    };
    let n = ph2d_board_model::hidden_count(&board.doc, el.id);
    let Some([x, y, w, h]) = ph2d_board_model::badge_rect(&board.doc, el.id, digits(n)) else {
        return;
    };
    if n == 0 {
        return;
    }
    let body = RoundedRect::new(x, y, x + w, y + h, h * BADGE_ROUND).to_path(0.1);
    scene.fill_path(&body, &Brush::Solid(doc_color(m.color)), v);
    let text = RichText::plain(&n.to_string());
    let size = (h * BADGE_TEXT) as f32;
    let key = (board.id.0, el.id.0 | BADGE_SALT);
    let layout = cache.text.get_rich(ts, key, &text, size, f32::MAX, false);
    let (tw, th) = (f64::from(layout.width()), f64::from(layout.height()));
    let at = v * Affine::translate((x + (w - tw) / 2.0, y + (h - th) / 2.0));
    ph2d_board_layout::paint(
        scene,
        layout,
        text.as_str(),
        at,
        doc_color(m.color.readable_ink()),
    );
}

/// Por cima: o realce do nó onde o arrasto largaria (como pai) e o «+» do nó seleccionado.
pub(crate) fn paint_overlay(
    scene: &mut VectorScene,
    overlay: &Overlay,
    v: Affine,
    metrics: &Metrics,
    theme: Theme,
) {
    let accent = token(ColorToken::Accent, theme);
    let thin = f64::from(StrokeToken::Thin.px());
    let thick = f64::from(StrokeToken::Thick.px());
    let stroke = |scene: &mut VectorScene, path: &BezPath, width: f64| {
        scene.stroke_path(
            &Stroke::new(width),
            Affine::IDENTITY,
            &Brush::Solid(accent),
            path,
        );
    };
    if let Some([x, y, w, h]) = overlay.mind_drop {
        let r = Rect::from_points(v * Point::new(x, y), v * Point::new(x + w, y + h));
        let pad = metrics.handle / 2.0;
        let path = RoundedRect::from_rect(r.inflate(pad, pad), pad).to_path(0.1);
        stroke(scene, &path, thick);
    }
    let paper = Brush::Solid(token(ColorToken::Bg1, theme));
    for (at, dir) in &overlay.mind_plus {
        let c = v * point(*at);
        let c = Point::new(c.x + dir[0] * metrics.dot, c.y + dir[1] * metrics.dot);
        // Do tamanho do alcance do clique (`Editor::mind_plus_at`): o que se vê é o que se agarra.
        let knob = ph2d_vector::Circle::new(c, metrics.handle).to_path(0.1);
        scene.fill_path(&knob, &paper, Affine::IDENTITY);
        stroke(scene, &knob, thin);
        let arm = metrics.handle / 2.0;
        let mut plus = BezPath::new();
        plus.move_to(Point::new(c.x - arm, c.y));
        plus.line_to(Point::new(c.x + arm, c.y));
        plus.move_to(Point::new(c.x, c.y - arm));
        plus.line_to(Point::new(c.x, c.y + arm));
        stroke(scene, &plus, thin);
    }
}
