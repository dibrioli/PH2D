//! ⭐ **O envelhecimento pintado** (o Power-Up «Card Aging» do Trello). Regular: o cartão parado
//! DESBOTA, um degrau por nível. Pirata: o papel AMARELECE, as bordas RASGAM-SE (dentes que comem o
//! cartão, mais fundos a cada nível) e racha — desenhado em Vello, pelas cores dos tokens.

use crate::metrics as m;
use crate::paint::c;
use crate::paint_card::{self, CardLook, Ink};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::Aging;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Stroke, VectorScene};

/// A opacidade do cartão regular por nível `0..=3` (o Power-Up: 100 → 40 %).
const FADE: [f32; 4] = [1.0, 0.75, 0.55, 0.4];
/// O amarelo do papel pirata por nível (a opacidade da camada de papel por cima do cartão).
const PAPER: [f32; 4] = [0.0, 0.28, 0.42, 0.56];
/// A fundura dos dentes do rasgão por nível, e o passo deles.
const TEAR: [f64; 4] = [0.0, 2.0, 3.5, 5.0];
const TOOTH: f64 = 9.0;
/// As rachas: quantas por nível.
const CRACKS: [usize; 4] = [0, 1, 2, 3];
const CRACK_W: f64 = 1.0;
/// Onde as rachas nascem (a faixa do meio da borda de baixo), quanto sobem por passo (fracção da
/// altura) e quanto se desviam para os lados (px).
const CRACK_SPAN: (f64, f64) = (0.15, 0.7);
const CRACK_RISE: (f64, f64) = (0.12, 0.6);
const CRACK_SWAY: f64 = 10.0;

/// Um ruído determinístico `0..1` (o mesmo cartão rasga-se sempre igual).
fn noise(seed: u64, i: u64) -> f64 {
    let x = ph2d_kanban_model::splitmix64(seed ^ i.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    #[allow(clippy::cast_precision_loss)]
    let v = (x >> 11) as f64 / (1u64 << 53) as f64;
    v
}

/// O contorno rasgado do cartão `r`: dentes ao longo dos quatro lados, para dentro.
fn torn(r: Rect, depth: f64, seed: u64) -> BezPath {
    let (x0, y0) = (f64::from(r.x), f64::from(r.y));
    let (x1, y1) = (f64::from(r.x + r.w), f64::from(r.y + r.h));
    let mut p = BezPath::new();
    let mut i = 0u64;
    let mut edge =
        |p: &mut BezPath, a: (f64, f64), b: (f64, f64), inward: (f64, f64), first: bool| {
            let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = (len / TOOTH).ceil().max(1.0) as u64;
            for s in 0..=n {
                #[allow(clippy::cast_precision_loss)]
                let t = s as f64 / n as f64;
                let d = depth * noise(seed, i);
                i += 1;
                let pt = (
                    a.0 + (b.0 - a.0) * t + inward.0 * d,
                    a.1 + (b.1 - a.1) * t + inward.1 * d,
                );
                if first && s == 0 {
                    p.move_to(pt);
                } else {
                    p.line_to(pt);
                }
            }
        };
    edge(&mut p, (x0, y0), (x1, y0), (0.0, 1.0), true);
    edge(&mut p, (x1, y0), (x1, y1), (-1.0, 0.0), false);
    edge(&mut p, (x1, y1), (x0, y1), (0.0, -1.0), false);
    edge(&mut p, (x0, y1), (x0, y0), (1.0, 0.0), false);
    p.close_path();
    p
}

/// As rachas: traços quebrados que partem de uma borda.
fn cracks(r: Rect, n: usize, seed: u64) -> BezPath {
    let mut p = BezPath::new();
    for k in 0..n as u64 {
        let base = 100 + k * 10;
        let x = f64::from(r.x) + f64::from(r.w) * (CRACK_SPAN.0 + CRACK_SPAN.1 * noise(seed, base));
        let mut pt = (x, f64::from(r.y + r.h));
        p.move_to(pt);
        for j in 1..4 {
            let dx = (noise(seed, base + j) - 0.5) * CRACK_SWAY;
            let dy = -(f64::from(r.h) * CRACK_RISE.0) * (CRACK_RISE.1 + noise(seed, base + j + 5));
            pt = (pt.0 + dx, pt.1 + dy);
            p.line_to(pt);
        }
    }
    p
}

/// ⭐ Um cartão com o envelhecimento `look.age` no modo `aging` (sem nível = o cartão de sempre).
#[allow(clippy::too_many_arguments)]
pub(crate) fn card(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    ink: &Ink,
    look: &CardLook,
    r: Rect,
    label_text: bool,
    (aging, level, seed): (Option<Aging>, u8, u64),
) {
    let lv = usize::from(level.min(3));
    match aging.filter(|_| lv > 0) {
        None => paint_card::card(scene, ts, theme, ink, look, r, label_text),
        Some(Aging::Regular) => {
            let bounds = ph2d_editor_core::paint::rect_to_vello(Rect::new(
                r.x - m::FOCUS_OFFSET * 4.0,
                r.y - m::FOCUS_OFFSET * 4.0,
                r.w + m::FOCUS_OFFSET * 8.0,
                r.h + m::FOCUS_OFFSET * 8.0,
            ));
            scene.push_object_layer(&bounds, ph2d_vector::VelloBlend::default(), FADE[lv]);
            paint_card::card(scene, ts, theme, ink, look, r, label_text);
            scene.pop_layer();
        }
        Some(Aging::Pirate) => {
            let edge = torn(r, TEAR[lv], seed);
            scene.push_layer_shape(
                ph2d_vector::VelloBlend::default(),
                1.0,
                Affine::IDENTITY,
                &edge,
            );
            paint_card::card(scene, ts, theme, ink, look, r, label_text);
            let paper = c(theme, T::KanbanYellowSubtle).with_alpha(PAPER[lv]);
            scene.fill_path(&edge, &Brush::Solid(paper), Affine::IDENTITY);
            let burn = c(theme, T::KanbanOrangeBold);
            let crack = cracks(r, CRACKS[lv], seed);
            scene.stroke_path(
                &Stroke::new(CRACK_W),
                Affine::IDENTITY,
                &Brush::Solid(burn.with_alpha(PAPER[lv])),
                &crack,
            );
            scene.pop_layer();
            // A borda queimada por cima do rasgão.
            scene.stroke_path(
                &Stroke::new(CRACK_W),
                Affine::IDENTITY,
                &Brush::Solid(burn.with_alpha(PAPER[lv])),
                &edge,
            );
        }
    }
}
