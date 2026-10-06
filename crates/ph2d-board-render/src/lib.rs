//! **Desenhar um quadro** (MiroClone) numa área do ecrã: o fundo, a grelha de pontos, as formas
//! vivas em ordem de z (contorno, estilo, rotação, texto dentro) e, por cima, o que o editor diz
//! (moldura e pegas, selecção por arrasto, guias, cursor do texto).
//!
//! ⚠️ Reconstrói a cena a cada quadro, recortando ao ecrã; o texto moldado vem da [`TextCache`].
//! As réguas (`tests/it/measure_*`) dizem quando isso deixa de chegar.

use ph2d_board_edit::{Frame, Handle, Metrics, Overlay};
use ph2d_board_geom::{outline, text_origin, text_rect, to_world};
use std::collections::HashMap;

use ph2d_board_geom::Outline;
use ph2d_board_layout::TextCache;
use ph2d_board_model::{Area, Board, Camera, Dash, Element, Rgba, Shape, ShapeType};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Cap, Color, Point, Rect, Shape as _, Stroke, VectorScene, VelloBlend,
};

/// Abaixo deste tamanho no ecrã (px) a letra não se lê: cada linha vira um traço (o «greeking»
/// do nível de detalhe). ⛔ Medido 06/10: com o tecto a 2 px, 10 mil formas com uma palavra cada,
/// todas à vista, pediam 14,9 ms de placa por quadro a desenhar letras de 2 px que ninguém lê.
const READ_PX: f64 = 6.0;
/// Abaixo disto nem o traço se vê.
const GREEK_MIN_PX: f64 = 1.0;
/// Opacidade do traço que substitui uma linha de texto.
const GREEK_ALPHA: f32 = 0.35;
/// Abaixo deste lado no ecrã (px) uma forma é um ponto de cor: só a caixa cheia, sem contorno nem
/// curva (o «nível de detalhe» de quem se afastou muito). ⛔ Medido 06/10: a 2 px, 100 mil formas de
/// 2,5 px desenhadas inteiras custavam 26,6 ms de CPU por quadro.
const DOT_PX: f64 = 4.0;

/// Quantos quadros um contorno guardado sobrevive sem ser desenhado.
const KEEP_FRAMES: u64 = 120;

/// O contorno de uma forma só depende disto.
type OutlineKey = (ShapeType, u64, u64, bool);

/// ⭐ **O que o desenho guarda entre quadros** — o texto moldado e o contorno de cada forma. ⛔
/// Medido 06/10: refazer o contorno a cada quadro levou 10 mil rectângulos de 0,45 para 3,1 ms.
#[derive(Default)]
pub struct RenderCache {
    pub text: TextCache,
    outlines: HashMap<(u64, u64), (OutlineKey, Outline, u64)>,
    frame: u64,
}

impl RenderCache {
    fn outline(&mut self, owner: (u64, u64), shape: &Shape, w: f64, h: f64) -> &Outline {
        let key = (shape.kind, w.to_bits(), h.to_bits(), shape.style.round);
        let frame = self.frame;
        let e = self
            .outlines
            .entry(owner)
            .or_insert_with(|| (key, outline(shape, w, h), frame));
        if e.0 != key {
            *e = (key, outline(shape, w, h), frame);
        }
        e.2 = frame;
        &e.1
    }

    fn end_frame(&mut self) {
        self.text.end_frame();
        self.frame += 1;
        if self.frame.is_multiple_of(KEEP_FRAMES) {
            let cut = self.frame.saturating_sub(KEEP_FRAMES);
            self.outlines.retain(|_, e| e.2 >= cut);
        }
    }
}

/// Mundo do quadro → ecrã.
#[must_use]
pub fn view(camera: &Camera, area: Area) -> Affine {
    let [x, y, w, h] = area;
    Affine::translate((x + w / 2.0, y + h / 2.0))
        * Affine::scale(camera.zoom)
        * Affine::translate((-camera.center_x, -camera.center_y))
}

/// Pinta `board` em `area` (`[x, y, w, h]` em px de ecrã).
pub fn paint(
    board: &Board,
    area: Area,
    scene: &mut VectorScene,
    theme: Theme,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
) {
    let [x, y, w, h] = area;
    if !(w > 0.0 && h > 0.0) {
        return;
    }
    let clip = Rect::new(x, y, x + w, y + h);
    scene.push_clip(&clip);
    scene.fill_rect(clip, token(ColorToken::Bg1, theme));
    let dots = dot_grid(&board.camera, area);
    scene.fill_path(
        &dots,
        &Brush::Solid(token(ColorToken::GridLine, theme)),
        Affine::IDENTITY,
    );
    let v = view(&board.camera, area);
    let zoom = board.camera.zoom;
    for el in board.doc.live_in_z_order() {
        let [ax0, ay0, ax1, ay1] = el.aabb();
        let a = board.camera.to_screen(area, [ax0, ay0]);
        let b = board.camera.to_screen(area, [ax1, ay1]);
        let r = Rect::new(a[0], a[1], b[0], b[1]);
        if r.intersect(clip).area() <= 0.0 {
            continue;
        }
        let Some(shape) = el.shape() else {
            continue;
        };
        let st = &shape.style;
        let plain = el.angle == 0.0 && st.opacity == 100 && st.stroke.is_none() && !st.round;
        if r.width().max(r.height()) < DOT_PX
            || (plain && shape.kind == ShapeType::Rectangle && shape.text.is_empty())
        {
            // Um ponto de cor, ou o rectângulo cheio sem mais nada: o caminho rápido da W0.
            if let Some(c) = st.fill.or(st.stroke) {
                scene.fill_rect(r, doc_color(c));
            }
            continue;
        }
        paint_shape(scene, el, shape, v, zoom, ts, cache, board.id.0);
    }
    scene.pop_layer();
    cache.end_frame();
}

#[allow(clippy::too_many_arguments)]
fn paint_shape(
    scene: &mut VectorScene,
    el: &Element,
    shape: &Shape,
    v: Affine,
    zoom: f64,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    board: u64,
) {
    let st = &shape.style;
    let t = v * to_world(el);
    let faded = st.opacity < 100;
    if faded {
        let [x0, y0, x1, y1] = el.aabb();
        let pad = st.stroke_width;
        let p0 = v * Point::new(x0 - pad, y0 - pad);
        let p1 = v * Point::new(x1 + pad, y1 + pad);
        scene.push_object_layer(
            &Rect::from_points(p0, p1),
            VelloBlend::default(),
            f32::from(st.opacity) / 100.0,
        );
    }
    let o = cache.outline((board, el.id.0), shape, el.w, el.h);
    if let Some(c) = st.fill {
        scene.fill_path(&o.fill, &Brush::Solid(doc_color(c)), t);
    }
    if let Some(c) = st.stroke.filter(|_| st.stroke_width > 0.0) {
        let stroke = style_stroke(st.stroke_width, st.dash);
        let brush = Brush::Solid(doc_color(c));
        scene.inner_mut().stroke(&stroke, t, &brush, None, &o.fill);
        if !o.lines.is_empty() {
            scene.inner_mut().stroke(&stroke, t, &brush, None, &o.lines);
        }
    }
    let font_px = st.font_size * zoom;
    if !shape.text.is_empty() && font_px >= GREEK_MIN_PX {
        let [_, _, tw, _] = text_rect(shape.kind, el.w, el.h);
        let layout = cache.text.get(
            ts,
            (board, el.id.0),
            &shape.text,
            st.font_size as f32,
            tw as f32,
        );
        let [ox, oy] = text_origin(shape.kind, el.w, el.h, f64::from(layout.height()));
        let at = t * Affine::translate((ox, oy));
        if font_px >= READ_PX {
            ph2d_board_layout::paint(scene, layout, at, doc_color(st.text_color));
        } else {
            let bars = ph2d_board_layout::line_bars(layout);
            let Rgba([r, g, b, _]) = st.text_color;
            let c = Color::from_rgba8(r, g, b, 255).with_alpha(GREEK_ALPHA);
            scene.fill_path(&bars, &Brush::Solid(c), at);
        }
    }
    if faded {
        scene.pop_layer();
    }
}

/// O traço de uma forma: contínuo, tracejado ou pontilhado (com pontas redondas, para o ponto ser
/// um ponto). Comprimentos em unidades do mundo, proporcionais à espessura.
fn style_stroke(width: f64, dash: Dash) -> Stroke {
    let s = Stroke::new(width);
    match dash {
        Dash::Solid => s,
        Dash::Dashed => s.with_dashes(0.0, [width * 4.0, width * 4.0]),
        Dash::Dotted => s.with_caps(Cap::Round).with_dashes(0.0, [0.0, width * 3.0]),
    }
}

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
        scene.inner_mut().stroke(
            &Stroke::new(width),
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            path,
        );
    };
    if let Some(t) = &overlay.text
        && let Some(el) = board.doc.get(t.element)
    {
        let to_screen = v * to_world(el) * Affine::translate((t.origin[0], t.origin[1]));
        let soft = Brush::Solid(token(ColorToken::AccentSoft, theme));
        for r in &t.selection {
            scene.fill_path(
                &Rect::new(r[0], r[1], r[2], r[3]).to_path(0.1),
                &soft,
                to_screen,
            );
        }
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
        scene.fill_path(
            &rect,
            &Brush::Solid(token(ColorToken::AccentSoft, theme)),
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
    scene.pop_layer();
}

fn handle_top() -> Handle {
    Handle::Resize(ph2d_board_edit::Dir::N)
}

fn point(p: [f64; 2]) -> Point {
    Point::new(p[0], p[1])
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

/// O passo da grelha no MUNDO para esta vista: parte do passo de base e dobra/divide por 2 até o
/// espaçamento no ecrã ficar em `[mínimo, 2·mínimo)` — a mesma densidade de pontos em qualquer zoom.
#[must_use]
pub fn grid_step_world(camera: &Camera) -> f64 {
    let min_px = f64::from(Spacing::Xl.px());
    let mut step = f64::from(Spacing::Xl2.px());
    while step * camera.zoom < min_px {
        step *= 2.0;
    }
    while step * camera.zoom >= 2.0 * min_px {
        step /= 2.0;
    }
    step
}

/// Os pontos da grelha visíveis em `area`, como UM caminho (uma chamada de preenchimento).
#[must_use]
pub fn dot_grid(camera: &Camera, area: Area) -> BezPath {
    let [x, y, w, h] = area;
    let step = grid_step_world(camera);
    let half = f64::from(Spacing::Xxs.px()) / 2.0;
    let lo = camera.to_world(area, [x, y]);
    let hi = camera.to_world(area, [x + w, y + h]);
    let mut path = BezPath::new();
    let mut gx = (lo[0] / step).ceil() * step;
    while gx <= hi[0] {
        let mut gy = (lo[1] / step).ceil() * step;
        while gy <= hi[1] {
            let [sx, sy] = camera.to_screen(area, [gx, gy]);
            let dot = Rect::new(sx - half, sy - half, sx + half, sy + half);
            path.extend(dot.path_elements(0.1));
            gy += step;
        }
        gx += step;
    }
    path
}

fn token(t: ColorToken, theme: Theme) -> Color {
    let c = t.resolve(theme);
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

fn doc_color(Rgba([r, g, b, a]): Rgba) -> Color {
    Color::from_rgba8(r, g, b, a)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
