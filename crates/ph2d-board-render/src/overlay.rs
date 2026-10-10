//! **O que o editor desenha por cima do quadro** (a moldura e as pegas, os contornos dos
//! seleccionados, o laço, as guias, o texto em edição) — fora do documento, por isso fora do export.
//! Saiu do `lib.rs` pelo tecto de LOC (W8).

use ph2d_board_edit::{Frame, Handle, Metrics, Overlay};
use ph2d_board_geom::to_world;
use ph2d_board_model::{Area, Board};
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Rect, Shape as _, Stroke, VectorScene};

use crate::{mind, point, sketch, token, view};

/// ⭐ **O que o editor desenha por cima** — moldura e pegas, contornos dos seleccionados, selecção
/// por arrasto, guias e o texto em edição (selecção + cursor). Espessuras e pegas são de ECRÃ.
pub fn paint_overlay(
    board: &Board,
    area: Area,
    scene: &mut VectorScene,
    theme: Theme,
    overlay: &Overlay,
    metrics: &Metrics,
) {
    let [x, y, w, h] = area;
    let clip = Rect::new(x, y, x + w, y + h);
    scene.push_clip(&clip);
    let v = view(&board.camera, area);
    let px = 1.0 / board.camera.zoom;
    let accent = token(ColorToken::Accent, theme);
    let thin = f64::from(StrokeToken::Thin.px());
    let line = |scene: &mut VectorScene, path: &BezPath, color: Color, width: f64| {
        scene.stroke_path(
            &Stroke::new(width),
            Affine::IDENTITY,
            &Brush::Solid(color),
            path,
        );
    };
    if let Some(t) = &overlay.text
        && let Some(el) = board.doc.get(t.element)
    {
        let to_screen = v * to_world(el) * Affine::translate((t.origin[0], t.origin[1]));
        // A selecção do texto pinta-se no quadro, DEBAIXO das letras (`paint`); aqui só o cursor.
        if let Some(c) = t.caret {
            // O cursor tem a espessura de UM px de ecrã a qualquer zoom.
            let mid = (c[0] + c[2]) / 2.0;
            let mut p = BezPath::new();
            p.move_to(to_screen * Point::new(mid, c[1]));
            p.line_to(to_screen * Point::new(mid, c[3]));
            line(scene, &p, token(ColorToken::Text1, theme), thin);
        }
    }
    for f in &overlay.boxes {
        line(scene, &frame_path(f, &v), accent, thin);
    }
    if let Some(r) = overlay.marquee {
        let p0 = v * Point::new(r[0], r[1]);
        let p1 = v * Point::new(r[2], r[3]);
        let rect = Rect::from_points(p0, p1).to_path(0.1);
        // TRANSLÚCIDA por contrato (`GraphMarquee`): a faixa passa por cima do que selecciona, e a
        // `AccentSoft` opaca tapava as formas e o texto (smoke da W3).
        scene.fill_path(
            &rect,
            &Brush::Solid(token(ColorToken::GraphMarquee, theme)),
            Affine::IDENTITY,
        );
        line(scene, &rect, accent, thin);
    }
    let guide = token(ColorToken::Danger, theme);
    for g in &overlay.guides {
        let mut p = BezPath::new();
        p.move_to(v * Point::new(g.a[0], g.a[1]));
        p.line_to(v * Point::new(g.b[0], g.b[1]));
        line(scene, &p, guide, thin);
    }
    if let Some(t) = &overlay.target
        && let Some(o) = board
            .doc
            .get(t.element)
            .and_then(ph2d_board_geom::world_outline)
    {
        let thick = f64::from(StrokeToken::Thick.px());
        line(scene, &(v * o.fill), accent, thick);
        if let Some(p) = t.fixed {
            let dot = ph2d_vector::Circle::new(v * point(p), metrics.handle / 2.0).to_path(0.1);
            scene.fill_path(&dot, &Brush::Solid(accent), Affine::IDENTITY);
        }
    }
    let paper = Brush::Solid(token(ColorToken::Bg1, theme));
    for w in &overlay.wires {
        let path = v * ph2d_vec_render::build_bezpath(&w.path);
        line(scene, &path, accent, thin);
        // As pontas e os pontos de ajuste: círculos ocos (arrastam-se) — o idioma do Miro.
        // Do tamanho do alcance do clique (`Editor::wire_handle_at`): o que se vê é o que se agarra.
        for e in w.ends.iter().chain(&w.points) {
            let c = ph2d_vector::Circle::new(v * point(*e), metrics.handle).to_path(0.1);
            scene.fill_path(&c, &paper, Affine::IDENTITY);
            line(scene, &c, accent, thin);
        }
        // O meio de cada trecho: a bolinha cheia que, arrastada, cria um ponto ali.
        for m in &w.mids {
            let c = ph2d_vector::Circle::new(v * point(*m), metrics.handle / 2.0).to_path(0.1);
            scene.fill_path(&c, &Brush::Solid(accent), Affine::IDENTITY);
        }
    }
    for (at, dir) in &overlay.dots {
        let c = v * point(*at);
        let p = Point::new(c.x + dir[0] * metrics.dot, c.y + dir[1] * metrics.dot);
        // Do tamanho do alcance do clique (`Editor::dot_at`): o que se vê é o que se agarra.
        let dot = ph2d_vector::Circle::new(p, metrics.handle).to_path(0.1);
        scene.fill_path(&dot, &Brush::Solid(accent), Affine::IDENTITY);
        line(scene, &dot, token(ColorToken::Bg1, theme), thin);
    }
    mind::paint_overlay(scene, overlay, v, metrics, theme);
    if let Some(f) = &overlay.frame {
        line(scene, &frame_path(f, &v), accent, thin);
        let top = v * point(f.handle(handle_top(), px, metrics));
        let knob = v * point(f.handle(Handle::Rotate, px, metrics));
        let mut stem = BezPath::new();
        stem.move_to(top);
        stem.line_to(knob);
        line(scene, &stem, accent, thin);
        let half = metrics.handle / 2.0;
        let paper = Brush::Solid(token(ColorToken::Bg1, theme));
        let knob_path = ph2d_vector::Circle::new(knob, half).to_path(0.1);
        scene.fill_path(&knob_path, &paper, Affine::IDENTITY);
        line(scene, &knob_path, accent, thin);
        for d in ph2d_board_edit::Dir::ALL {
            let c = v * point(f.handle(Handle::Resize(d), px, metrics));
            let sq = Rect::new(c.x - half, c.y - half, c.x + half, c.y + half).to_path(0.1);
            scene.fill_path(&sq, &paper, Affine::IDENTITY);
            line(scene, &sq, accent, thin);
        }
    }
    if let Some(g) = overlay.grid {
        // A pega de ARRUMAR EM GRELHA: um botão redondo com os quatro pontos, fora do canto superior
        // direito — onde `Editor::grid_handle` a agarra (do tamanho do alcance do clique).
        let off = ph2d_board_edit::grid_handle_offset(metrics);
        let c = v * point(g);
        let c = Point::new(c.x + off, c.y - off);
        let knob = ph2d_vector::Circle::new(c, metrics.handle).to_path(0.1);
        scene.fill_path(&knob, &paper, Affine::IDENTITY);
        line(scene, &knob, accent, thin);
        let q = metrics.handle / 3.0;
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let dot = ph2d_vector::Circle::new(Point::new(c.x + sx * q, c.y + sy * q), q / 2.0);
            scene.fill_path(&dot.to_path(0.1), &Brush::Solid(accent), Affine::IDENTITY);
        }
    }
    sketch::paint_laser(scene, &overlay.laser, v, theme, f64::from(Spacing::Xs.px()));
    if let Some(c) = overlay.eraser {
        let ring = ph2d_vector::Circle::new(v * point(c), metrics.eraser).to_path(0.1);
        line(scene, &ring, token(ColorToken::Text2, theme), thin);
    }
    scene.pop_layer();
}

fn handle_top() -> Handle {
    Handle::Resize(ph2d_board_edit::Dir::N)
}

/// O contorno de uma moldura rodada, já no ecrã.
fn frame_path(f: &Frame, v: &Affine) -> BezPath {
    let (hw, hh) = (f.w / 2.0, f.h / 2.0);
    let mut p = BezPath::new();
    for (i, l) in [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]]
        .iter()
        .enumerate()
    {
        let q = *v * point(f.point(*l));
        if i == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    p.close_path();
    p
}
