//! **Desenhar um quadro** (MiroClone) numa área do ecrã: o fundo, a grelha de pontos, as formas
//! e as setas vivas em ordem de z (contorno, estilo, rotação, texto dentro; rota, pontas, rótulo) e,
//! por cima, o que o editor diz (moldura e pegas, selecção por arrasto, guias, cursor do texto, as
//! pontas da seta seleccionada, a forma onde a seta se vai prender, os pontos azuis).
//!
//! ⚠️ Reconstrói a cena a cada quadro, recortando ao ecrã; o texto moldado vem da [`TextCache`].
//! As réguas (`tests/it/measure_*`) dizem quando isso deixa de chegar.

use ph2d_board_edit::{Frame, Handle, Metrics, Overlay};
use ph2d_board_geom::{outline, text_origin, text_rect, to_world};
use std::collections::BTreeMap;

use ph2d_board_geom::Outline;
use ph2d_board_layout::TextCache;
use ph2d_board_model::{Area, Board, Camera, Connector, Dash, Element, Rgba, Shape, ShapeType};
use ph2d_board_route::{LABEL_WRAP, RouteCache, Routed, label_origin};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Cap, Color, Join, Point, Rect, Shape as _, Stroke, VectorScene,
    VelloBlend,
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
    outlines: BTreeMap<(u64, u64), (OutlineKey, Outline, u64)>,
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

/// Pinta `board` em `area` (`[x, y, w, h]` em px de ecrã). `routes` = as rotas das setas de
/// `board.doc`, já em dia (`ph2d_board_edit::Editor::routes`).
#[allow(clippy::too_many_arguments)]
pub fn paint(
    board: &Board,
    area: Area,
    scene: &mut VectorScene,
    theme: Theme,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    routes: &RouteCache,
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
        if let Some(c) = el.connector() {
            if let Some(r) = routes.get(el.id) {
                let [bx0, by0, bx1, by1] = r.bbox;
                // A ponta de seta sai até 4 comprimentos de traço da rota; o rótulo, meia quebra.
                let head = c.style.stroke_width * ph2d_board_route::HEAD_SCALE * 4.0;
                let pad = if c.label.is_empty() {
                    head
                } else {
                    head.max(LABEL_WRAP / 2.0)
                };
                let a = board.camera.to_screen(area, [bx0 - pad, by0 - pad]);
                let b = board.camera.to_screen(area, [bx1 + pad, by1 + pad]);
                if Rect::new(a[0], a[1], b[0], b[1]).intersect(clip).area() > 0.0 {
                    let ink = Brush::Solid(token(ColorToken::Bg1, theme));
                    let at = (board.id.0, el.id.0);
                    paint_connector(scene, c, r, v, zoom, ts, cache, at, &ink);
                }
            }
            continue;
        }
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

/// ⭐ **Uma seta**: a linha (no traço do estilo, com pontas e dobras redondas — as do Excalidraw,
/// `stroke-linecap="round"`), as pontas de seta (cheias pintam, vazadas traçam a linha contínua) e
/// o rótulo a meio da rota, sobre um recorte do fundo (a linha não lhe passa por baixo).
#[allow(clippy::too_many_arguments)]
fn paint_connector(
    scene: &mut VectorScene,
    c: &Connector,
    r: &Routed,
    v: Affine,
    zoom: f64,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    owner: (u64, u64),
    paper: &Brush,
) {
    let st = &c.style;
    let faded = st.opacity < 100;
    if faded {
        let [x0, y0, x1, y1] = r.bbox;
        let pad = st.stroke_width * ph2d_board_route::HEAD_SCALE * 4.0;
        scene.push_object_layer(
            &Rect::from_points(
                v * Point::new(x0 - pad, y0 - pad),
                v * Point::new(x1 + pad, y1 + pad),
            ),
            VelloBlend::default(),
            f32::from(st.opacity) / 100.0,
        );
    }
    let [x0, y0, x1, y1] = r.bbox;
    if (x1 - x0).max(y1 - y0) * zoom < DOT_PX {
        // Uma seta de poucos px: o traço de ponta a ponta, sem curva nem pontas (o nível de detalhe
        // das formas, `DOT_PX`).
        if let Some(color) = st.stroke {
            let [a, b] = r.ends();
            let mut p = BezPath::new();
            p.move_to(point(a));
            p.line_to(point(b));
            let line = Stroke::new(st.stroke_width);
            scene
                .inner_mut()
                .stroke(&line, v, &Brush::Solid(doc_color(color)), None, &p);
        }
        if faded {
            scene.pop_layer();
        }
        return;
    }
    if let Some(color) = st.stroke.filter(|_| st.stroke_width > 0.0) {
        let d = ph2d_board_route::drawn(r, c.heads, st.stroke_width);
        let brush = Brush::Solid(doc_color(color));
        let line = style_stroke(st.stroke_width, st.dash)
            .with_caps(Cap::Round)
            .with_join(Join::Round);
        scene.inner_mut().stroke(&line, v, &brush, None, &d.line);
        let solid = Stroke::new(st.stroke_width)
            .with_caps(Cap::Round)
            .with_join(Join::Round);
        for (head, filled) in &d.heads {
            if *filled {
                scene.fill_path(head, &brush, v);
            } else {
                scene.inner_mut().stroke(&solid, v, &brush, None, head);
            }
        }
    }
    let font_px = st.font_size * zoom;
    if !c.label.is_empty() && font_px >= GREEK_MIN_PX {
        let layout = cache
            .text
            .get(ts, owner, &c.label, st.font_size as f32, LABEL_WRAP as f32);
        let [ox, oy] = label_origin(r.mid, f64::from(layout.height()));
        let at = v * Affine::translate((ox, oy));
        let bars = ph2d_board_layout::line_bars(layout);
        let knock = bars
            .bounding_box()
            .inflate(st.font_size * LABEL_PAD, st.font_size * LABEL_PAD);
        scene.fill_path(&knock.to_path(0.1), paper, at);
        if font_px >= READ_PX {
            ph2d_board_layout::paint(scene, layout, at, doc_color(st.text_color));
        } else {
            let Rgba([r8, g8, b8, _]) = st.text_color;
            let c = Color::from_rgba8(r8, g8, b8, 255).with_alpha(GREEK_ALPHA);
            scene.fill_path(&bars, &Brush::Solid(c), at);
        }
    }
    if faded {
        scene.pop_layer();
    }
}

/// A folga do recorte do rótulo à volta do texto, em fracção da letra.
const LABEL_PAD: f64 = 0.25;

/// O traço de uma forma: contínuo, tracejado ou pontilhado (com pontas redondas, para o ponto ser
/// um ponto). Comprimentos em unidades do mundo, proporcionais à espessura.
#[must_use]
pub fn style_stroke(width: f64, dash: Dash) -> Stroke {
    let s = Stroke::new(width);
    match dash {
        Dash::Solid => s,
        Dash::Dashed => s.with_dashes(0.0, [width * 4.0, width * 4.0]),
        // ⛔ Um traço de comprimento ZERO com ponta redonda não se desenha (foto de 06/10: o
        // pontilhado sumia): o ponto é um traço de meia espessura, e a ponta redonda arredonda-o.
        Dash::Dotted => s
            .with_caps(Cap::Round)
            .with_dashes(0.0, [width * 0.5, width * 2.5]),
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
