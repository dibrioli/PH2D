//! **O mapa mental no desenho** (W6): os ramos — a curva MEDIDA do Miro, derivada da relação
//! pai→filho (M13) — e o selo de cada ramo recolhido com o número escondido (M10); por cima, o «+»
//! do nó seleccionado e o realce do nó onde o arrasto largaria o que leva.

use std::collections::BTreeSet;

use ph2d_board_edit::{Metrics, Overlay};
use ph2d_board_model::{Board, DEFAULT_PAPER, MIND_BRANCH_WIDTH, MIND_FONT, Rgba, RichText};
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

/// Os ramos à vista, debaixo das formas.
pub(crate) fn paint_branches(scene: &mut VectorScene, board: &Board, v: Affine, clip: Rect) {
    if !board.doc.has_mind_links() {
        return;
    }
    let zoom = board.camera.zoom;
    for (_, c) in board.doc.mind_links() {
        let Some([p0, p1, p2, p3]) = ph2d_board_model::mind_branch(&board.doc, c) else {
            continue;
        };
        let Some(m) = board.doc.get(c).and_then(|e| e.mind()) else {
            continue;
        };
        let mut path = BezPath::new();
        path.move_to(point(p0));
        path.curve_to(point(p1), point(p2), point(p3));
        let path = v * path;
        let width = MIND_BRANCH_WIDTH * map_font(board, c) / MIND_FONT * zoom;
        if path
            .bounding_box()
            .inflate(width, width)
            .intersect(clip)
            .area()
            <= 0.0
        {
            continue;
        }
        scene.inner_mut().stroke(
            &Stroke::new(width),
            Affine::IDENTITY,
            &Brush::Solid(doc_color(m.color)),
            None,
            &path,
        );
    }
}

/// O selo de cada ramo recolhido à vista: a cor do ramo, o número de escondidos a branco.
pub(crate) fn paint_badges(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    board: &Board,
    v: Affine,
) {
    if !board.doc.has_mind_links() {
        return;
    }
    let parents: BTreeSet<_> = board.doc.mind_links().map(|(p, _)| p).collect();
    for p in parents {
        let Some(el) = board.doc.get(p) else {
            continue;
        };
        let Some(m) = el.mind().filter(|m| m.collapsed && !el.is_hidden()) else {
            continue;
        };
        let n = ph2d_board_model::hidden_count(&board.doc, p);
        let Some([x, y, w, h]) =
            ph2d_board_model::badge_rect(&board.doc, p, ph2d_board_edit::digits(n))
        else {
            continue;
        };
        let body = RoundedRect::new(x, y, x + w, y + h, h * BADGE_ROUND).to_path(0.1);
        scene.fill_path(&body, &Brush::Solid(doc_color(m.color)), v);
        let text = RichText::plain(&n.to_string());
        let size = (h * BADGE_TEXT) as f32;
        let layout = cache.text.get_rich(
            ts,
            (board.id.0, p.0 | BADGE_SALT),
            &text,
            size,
            f32::MAX,
            false,
        );
        let (tw, th) = (f64::from(layout.width()), f64::from(layout.height()));
        let at = v * Affine::translate((x + (w - tw) / 2.0, y + (h - th) / 2.0));
        ph2d_board_layout::paint(scene, layout, at, doc_color(Rgba(DEFAULT_PAPER)));
    }
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
        scene.inner_mut().stroke(
            &Stroke::new(width),
            Affine::IDENTITY,
            &Brush::Solid(accent),
            None,
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
