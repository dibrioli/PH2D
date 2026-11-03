//! **O editor de curva** (Edit ▸ Adjustments ▸ Color Curve do Aseprite, foto
//! `docs/PixelArt/aseprite/83_color_curve.png`): a caixa `0..=255 × 0..=255` com a diagonal, as rectas
//! entre os pontos e as pontas fixas `(0,0)`/`(255,255)`.
//!
//! O widget é sem estado (molde do [`super::range_slider`]): quem o usa guarda os pontos INTERIORES
//! por `x` crescente, regista a caixa como `Plain` e pergunta [`curve_press`] no premir,
//! [`curve_drag`] a cada movimento e [`curve_remove_at`] no duplo-clique. Um ponto nunca cruza os
//! vizinhos (`x` estritamente entre eles), e as pontas não se agarram nem se tiram.

use crate::math::{safe_clamp, safe_clamp_ord};
use crate::paint::{fill_circle, fill_rounded_rect, resolve, stroke_polyline};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// O fim da escala `0..=255` dos dois eixos.
const FIM: f32 = 255.0; // LITERAL-PX-OK: escala 0..=255 do canal (domínio da curva, não medida de desenho)

/// O raio de agarrar um ponto, em px (o mesmo para o dedo e para a pintura do ponto activo).
#[must_use]
pub fn curve_grab_radius() -> f32 {
    Spacing::Sm.px()
}

/// Quantos pontos interiores a caixa `b` ainda distingue ao agarrar: um por diâmetro de agarrar
/// ao longo do `x` (mais juntos, o premir agarra o vizinho em vez de acrescentar).
#[must_use]
pub fn curve_capacity(b: Rect) -> usize {
    let d = curve_grab_radius() * 2.0;
    if b.w <= 0.0 || d <= 0.0 {
        return 0;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "largura positiva sobre um diâmetro positivo"
    )]
    let n = (b.w / d).floor() as usize;
    n
}

/// O ponto de ecrã do valor `(x, y)` (o `y` cresce para cima, como no Aseprite).
#[must_use]
pub fn curve_to_screen(b: Rect, p: (u8, u8)) -> [f32; 2] {
    [
        b.x + b.w * f32::from(p.0) / FIM,
        b.y + b.h * (1.0 - f32::from(p.1) / FIM),
    ]
}

/// O valor `(x, y)` sob o ponto de ecrã `p` (fora da caixa, preso à borda).
#[must_use]
pub fn curve_from_screen(b: Rect, p: [f32; 2]) -> (u8, u8) {
    let f = |v: f32, o: f32, len: f32| {
        if len <= 0.0 || !v.is_finite() {
            return 0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "preso a 0..=255 antes da conversão"
        )]
        let r = safe_clamp((((v - o) / len) * FIM).round(), 0.0, FIM) as u8;
        r
    };
    (f(p[0], b.x, b.w), 255 - f(p[1], b.y, b.h))
}

/// O ponto interior sob `p` (o mais perto dentro do raio de agarrar), se há.
#[must_use]
pub fn curve_point_at(b: Rect, pts: &[(u8, u8)], p: [f32; 2]) -> Option<usize> {
    let r = curve_grab_radius();
    pts.iter()
        .enumerate()
        .map(|(i, q)| {
            let s = curve_to_screen(b, *q);
            (i, (s[0] - p[0]).hypot(s[1] - p[1]))
        })
        .filter(|(_, d)| *d <= r)
        .min_by(|a, c| a.1.total_cmp(&c.1))
        .map(|(i, _)| i)
}

/// O premir em `p`: agarra o ponto sob ele ou, numa zona vazia, ACRESCENTA um (na ordem do `x`).
/// Devolve o índice agarrado; `None` = nada (sobre uma ponta, num `x` já ocupado ou com `max`
/// pontos).
pub fn curve_press(b: Rect, pts: &mut Vec<(u8, u8)>, p: [f32; 2], max: usize) -> Option<usize> {
    if let Some(i) = curve_point_at(b, pts, p) {
        return Some(i);
    }
    let q = curve_from_screen(b, p);
    if q.0 == 0 || q.0 == 255 || pts.len() >= max || pts.iter().any(|o| o.0 == q.0) {
        return None;
    }
    let i = pts.partition_point(|o| o.0 < q.0);
    pts.insert(i, q);
    Some(i)
}

/// O ponto `i` arrastado até `p`: o `x` fica estritamente entre os vizinhos (as pontas contam
/// como vizinhos), o `y` em `0..=255`.
pub fn curve_drag(b: Rect, pts: &mut [(u8, u8)], i: usize, p: [f32; 2]) {
    if i >= pts.len() {
        return;
    }
    let (x, y) = curve_from_screen(b, p);
    let lo = if i == 0 {
        1
    } else {
        pts[i - 1].0.saturating_add(1)
    };
    let hi = pts.get(i + 1).map_or(254, |n| n.0.saturating_sub(1));
    pts[i] = (safe_clamp_ord(x, lo, hi.max(lo)), y);
}

/// Tira o ponto sob `p` (botão direito / duplo-clique). `true` se tirou.
pub fn curve_remove_at(b: Rect, pts: &mut Vec<(u8, u8)>, p: [f32; 2]) -> bool {
    curve_point_at(b, pts, p).is_some_and(|i| {
        pts.remove(i);
        true
    })
}

/// Pinta a caixa, a diagonal de referência, a curva (rectas com as pontas) e os pontos interiores;
/// `activo` = o agarrado.
pub fn paint_curve_editor(
    b: Rect,
    pts: &[(u8, u8)],
    activo: Option<usize>,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let r = crate::paint::frame_radius(theme, Radius::Xs.px());
    let fundo = crate::widget::card_ink::card_field_fill()
        .unwrap_or_else(|| resolve(ColorToken::GraphBg, theme));
    fill_rounded_rect(scene, b, r, fundo);
    let fino = StrokeToken::Thin.px();
    let tup = |p: [f32; 2]| (p[0], p[1]);
    let diag = [
        tup(curve_to_screen(b, (0, 0))),
        tup(curve_to_screen(b, (255, 255))),
    ];
    stroke_polyline(scene, &diag, fino, resolve(ColorToken::GraphGrid, theme));
    let linha: Vec<(f32, f32)> = std::iter::once((0, 0))
        .chain(pts.iter().copied())
        .chain(std::iter::once((255, 255)))
        .map(|q| tup(curve_to_screen(b, q)))
        .collect();
    stroke_polyline(
        scene,
        &linha,
        StrokeToken::Thick.px(),
        resolve(ColorToken::Accent, theme),
    );
    let raio = curve_grab_radius() * 0.5;
    for (i, q) in pts.iter().enumerate() {
        let cor = if activo == Some(i) {
            ColorToken::AccentPress
        } else {
            ColorToken::Text1
        };
        let s = curve_to_screen(b, *q);
        fill_circle(scene, s[0], s[1], raio, resolve(cor, theme));
    }
}

#[cfg(test)]
#[path = "curve_editor/tests.rs"]
mod tests;

/// O nó AccessKit da caixa da curva (HR-12): um grupo com o nome do canal, focável, que se clica
/// (acrescentar ou agarrar um ponto).
#[must_use]
pub fn curve_a11y(label: &str, b: Rect) -> ph2d_a11y::Node {
    ph2d_a11y::NodeBuilder::new(ph2d_a11y::Role::Group)
        .label(label)
        .bounds(
            f64::from(b.x),
            f64::from(b.y),
            f64::from(b.w),
            f64::from(b.h),
        )
        .focusable(true)
        .action(ph2d_a11y::Action::Click)
        .build()
}
