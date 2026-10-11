//! Os OUTROS sobre o quadro da sala (módulo filho de [`super`]): o cursor de cada pessoa (já
//! interpolado pela família) com o nome ao lado, e a selecção dela em contorno tracejado (c20 — o do
//! Excalidraw), tudo na cor da pessoa. Pinta-se logo depois do quadro: os painéis ficam por cima.

use super::super::super::super::HeroScreen;
use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Fill, Stroke, VectorScene};

/// O ponteiro em unidades da sua altura: a ponta em `(0, 0)`, o desenho clássico de seta de rato.
const ARROW: [(f64, f64); 7] = [
    (0.0, 0.0),
    (0.0, 1.0),
    (0.27, 0.76), // LITERAL-PX-OK: forma da seta, em fracções da altura
    (0.45, 1.1),  // LITERAL-PX-OK: forma da seta, em fracções da altura
    (0.58, 1.04), // LITERAL-PX-OK: forma da seta, em fracções da altura
    (0.41, 0.7),  // LITERAL-PX-OK: forma da seta, em fracções da altura
    (0.72, 0.7),  // LITERAL-PX-OK: forma da seta, em fracções da altura
];

/// ⭐ Pinta os outros da sala, se o quadro activo é o da sala.
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let lv = &hero.documents.live_view;
    let Some(board) = hero.documents.active_board() else {
        return;
    };
    if lv.board != Some(board.id) || lv.others.is_empty() {
        return;
    }
    let a = super::super::super::super::board_view::area_of(area);
    scene.push_clip(&crate::paint::rect_to_vello(area));
    // A selecção primeiro: os cursores passam por cima dela.
    let dash = f64::from(Spacing::Xs.px());
    let width = f64::from(StrokeToken::Default.px());
    // W12 — anónimo: a tinta neutra do texto, a mesma para todos (a cor diria quem é).
    let tint = |o: &crate::documents::live::LiveOther| {
        if o.anonymous {
            ColorToken::Text2
        } else {
            ColorToken::person(o.color)
        }
    };
    for o in &lv.others {
        let ink = Brush::Solid(resolve(tint(o), theme));
        let stroke = Stroke::new(width).with_dashes(0.0, [dash * 2.0, dash]);
        for id in &o.selection {
            let Some(el) = board.doc.get(*id) else {
                continue;
            };
            let [x0, y0, x1, y1] = el.aabb();
            let p = board.camera.to_screen(a, [x0, y0]);
            let q = board.camera.to_screen(a, [x1, y1]);
            let m = f64::from(Spacing::Xs.px());
            let r = ph2d_vector::Rect::new(p[0] - m, p[1] - m, q[0] + m, q[1] + m);
            scene
                .inner_mut()
                .stroke(&stroke, Affine::IDENTITY, &ink, None, &r);
        }
    }
    let h = f64::from(Spacing::Lg.px() + Spacing::Xs.px());
    let font = TypeToken::Xs.px();
    let pad = Spacing::Sm.px();
    let edge = resolve(ColorToken::PersonInk, theme);
    for o in &lv.others {
        let Some(world) = o.cursor else {
            continue;
        };
        let [x, y] = board.camera.to_screen(a, world);
        let fill = resolve(tint(o), theme);
        let mut path = BezPath::new();
        for (i, (u, v)) in ARROW.iter().enumerate() {
            let pt = (x + u * h, y + v * h);
            if i == 0 {
                path.move_to(pt);
            } else {
                path.line_to(pt);
            }
        }
        path.close_path();
        let s = scene.inner_mut();
        s.fill(Fill::NonZero, Affine::IDENTITY, fill, None, &path);
        s.stroke(&Stroke::new(width), Affine::IDENTITY, edge, None, &path);
        if o.anonymous {
            continue;
        }
        // A etiqueta com o nome, logo abaixo e à direita da ponta (a do Excalidraw).
        let w = crate::paint::rect_for_label(ts.prefix_width(&o.name, font));
        let lh = font + pad;
        let r = Rect::new(x as f32 + (h as f32) * 0.5, y as f32 + h as f32, w, lh); // LITERAL-PX-OK: metade da ponta
        let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
        fill_rounded_rect(scene, r, radius, fill);
        paint_text_centered(ts, scene, &o.name, r, font, edge);
    }
    scene.pop_layer();
}
