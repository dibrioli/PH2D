//! ⭐ **O minimapa** (`M`, o do Miro: *«navigation controls in the bottom-right … the minimap»*) —
//! TODO o conteúdo do quadro em pequeno e o rectângulo da vista; carregar ou arrastar nele move a
//! vista para ali.
//!
//! ⚠️ **O custo é dos PIXELS, nunca dos elementos:** o conteúdo rasteriza-se numa grelha de células
//! de `Spacing::Xxs` (cada uma com a cor do elemento de cima) só quando o documento muda; cada
//! imagem pinta as células (≤ as do rectângulo) e a vista. Medido (07/10): 100 mil rectângulos
//! desenhados um a um custam ~6 ms por imagem — o minimapa não pode pagar isso a cada quadro.

use super::HeroScreen;
use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_board_model::{Board, Rgba};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::{Affine, Brush, Color, Stroke, VectorScene};

/// O tamanho do minimapa em passos de `Spacing::Xl4`: 4 × 3.
const SIZE_STEPS: (f32, f32) = (4.0, 3.0); // LITERAL-PX-OK: proporção do minimapa

/// O rectângulo do minimapa: no canto de baixo à direita, por cima dos botões da navegação.
#[must_use]
pub fn minimap_rect(area: Rect) -> Rect {
    let step = Spacing::Xl4.px();
    let (w, h) = (step * SIZE_STEPS.0, step * SIZE_STEPS.1);
    let buttons = super::button_rects(area);
    let top = buttons.first().map_or(area.y + area.h, |(_, r)| r.y);
    Rect::new(
        area.x + area.w - Spacing::Md.px() - w,
        top - Spacing::Md.px() - h,
        w,
        h,
    )
}

/// O mundo `extent` encaixado (com a proporção dele) no rectângulo `r` do ecrã.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Map {
    extent: [f64; 4],
    scale: f64,
    origin: [f64; 2],
}

impl Map {
    fn new(extent: [f64; 4], r: Rect) -> Self {
        let (ew, eh) = ((extent[2] - extent[0]).max(1.0), (extent[3] - extent[1]).max(1.0));
        let scale = (f64::from(r.w) / ew).min(f64::from(r.h) / eh);
        let origin = [
            f64::from(r.x) + (f64::from(r.w) - ew * scale) / 2.0,
            f64::from(r.y) + (f64::from(r.h) - eh * scale) / 2.0,
        ];
        Self {
            extent,
            scale,
            origin,
        }
    }

    fn to_screen(self, p: [f64; 2]) -> [f64; 2] {
        [
            self.origin[0] + (p[0] - self.extent[0]) * self.scale,
            self.origin[1] + (p[1] - self.extent[1]) * self.scale,
        ]
    }

    fn to_world(self, p: [f64; 2]) -> [f64; 2] {
        [
            self.extent[0] + (p[0] - self.origin[0]) / self.scale,
            self.extent[1] + (p[1] - self.origin[1]) / self.scale,
        ]
    }
}

/// As células já rasterizadas, válidas para uma revisão do documento e um rectângulo.
pub(super) struct Cache {
    key: (u64, [u32; 4]),
    map: Map,
    cells: VectorScene,
}

fn doc_color(Rgba([r, g, b, a]): Rgba) -> Color {
    Color::from_rgba8(r, g, b, a) // LITERAL-COLOR-OK: cor do DOCUMENTO
}

/// A cor de um elemento no minimapa: o preenchimento, senão o traço.
fn mark(el: &ph2d_board_model::Element) -> Option<Rgba> {
    let st = el.style();
    st.fill.or(st.stroke)
}

fn build(board: &Board, r: Rect, theme: Theme) -> Option<Cache> {
    let extent = board.doc.extent()?;
    let map = Map::new(extent, r);
    let cell = f64::from(Spacing::Xxs.px());
    let (cols, rows) = ((f64::from(r.w) / cell) as usize, (f64::from(r.h) / cell) as usize);
    let mut grid: Vec<Option<Color>> = vec![None; cols * rows];
    let frame_edge = resolve(ColorToken::Border, theme);
    let frames = board.doc.frames_in_draw_order();
    let others = board.doc.live_in_z_order();
    let all = frames.into_iter().chain(others.into_iter().filter(|e| e.as_frame().is_none()));
    for el in all.filter(|e| e.connector().is_none()) {
        let Some(c) = mark(el).map(doc_color).or(el.as_frame().map(|_| frame_edge)) else {
            continue;
        };
        let [x0, y0, x1, y1] = el.aabb();
        let a = map.to_screen([x0, y0]);
        let b = map.to_screen([x1, y1]);
        let cx = |v: f64| (((v - f64::from(r.x)) / cell).floor().max(0.0) as usize).min(cols - 1);
        let cy = |v: f64| (((v - f64::from(r.y)) / cell).floor().max(0.0) as usize).min(rows - 1);
        for row in cy(a[1])..=cy(b[1]) {
            for col in cx(a[0])..=cx(b[0]) {
                grid[row * cols + col] = Some(c);
            }
        }
    }
    let mut cells = VectorScene::new();
    for (i, c) in grid.iter().enumerate() {
        if let Some(c) = c {
            let (col, row) = ((i % cols) as f64, (i / cols) as f64);
            let x = f64::from(r.x) + col * cell;
            let y = f64::from(r.y) + row * cell;
            cells.fill_rect(ph2d_vector::Rect::new(x, y, x + cell, y + cell), *c);
        }
    }
    Some(Cache {
        key: (board.doc.rev(), rect_bits(r)),
        map,
        cells,
    })
}

fn rect_bits(r: Rect) -> [u32; 4] {
    [r.x.to_bits(), r.y.to_bits(), r.w.to_bits(), r.h.to_bits()]
}

pub(super) fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, theme: Theme) {
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    if !live.nav.minimap_open {
        return;
    }
    let r = minimap_rect(area);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
    let key = (board.doc.rev(), rect_bits(r));
    if live.nav.minimap_cache.as_ref().is_none_or(|c| c.key != key) {
        live.nav.minimap_cache = build(board, r, theme);
    }
    let Some(cache) = live.nav.minimap_cache.as_ref() else {
        return;
    };
    scene.push_clip(&crate::paint::rect_to_vello(r));
    scene.inner_mut().append(cache.cells.inner(), None);
    // A vista: o rectângulo do mundo à vista, na escala do minimapa.
    let a = super::super::board_view::area_of(area);
    let v0 = cache.map.to_screen(board.camera.to_world(a, [a[0], a[1]]));
    let v1 = cache.map.to_screen(board.camera.to_world(a, [a[0] + a[2], a[1] + a[3]]));
    let view = ph2d_vector::Rect::new(v0[0], v0[1], v1[0], v1[1]);
    let line = Stroke::new(f64::from(StrokeToken::Default.px()));
    let accent = Brush::Solid(resolve(ColorToken::Accent, theme));
    scene
        .inner_mut()
        .stroke(&line, Affine::IDENTITY, &accent, None, &view);
    scene.pop_layer();
}

/// Põe o centro da vista no ponto do minimapa sob `(x, y)`.
fn center_on(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let area = hero.last_canvas;
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let Some(map) = live.nav.minimap_cache.as_ref().map(|c| c.map) else {
        return false;
    };
    let w = map.to_world([f64::from(x), f64::from(y)]);
    let _ = area;
    board.camera.center_x = w[0];
    board.camera.center_y = w[1];
    true
}

pub(super) fn down(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    if !hero.documents.live.nav.minimap_open {
        return false;
    }
    let r = minimap_rect(hero.last_canvas);
    if !(x >= r.x && y >= r.y && x < r.x + r.w && y < r.y + r.h) {
        return false;
    }
    hero.documents.live.nav.minimap_drag = center_on(hero, x, y);
    true
}

pub(super) fn drag(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    if !hero.documents.live.nav.minimap_drag {
        return false;
    }
    center_on(hero, x, y);
    true
}
