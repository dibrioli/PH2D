//! **O voo de um arrasto** — o que está agarrado descola: sombra de voo em duas camadas, inclinado
//! em torno de `(50 %, 60 %)`, por cima de tudo. Nasceu nos cartões do Kanban (o Trello); o bloco de
//! quadros da linha do tempo do Pixel voa com a MESMA lei.

use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Color, VectorScene};

/// O que voa inclina 4° (o Trello).
pub const FLIGHT_TILT_DEG: f64 = 4.0;
/// O pivô da inclinação, em fracção da altura.
pub const FLIGHT_PIVOT_Y: f64 = 0.6;
/// `(dy, desfoque)` da sombra de voo: a longe no escuro e no claro, e a de perto.
pub const SHADOW_FLIGHT_DARK: (f64, f64) = (14.0, 30.0);
pub const SHADOW_FLIGHT_LIGHT: (f64, f64) = (16.0, 32.0);
pub const SHADOW_FLIGHT_NEAR: (f64, f64) = (3.0, 8.0);

/// Uma camada de sombra desfocada por baixo de `r`.
pub fn shadow(scene: &mut VectorScene, r: Rect, radius: f32, color: Color, (dy, blur): (f64, f64)) {
    let rr = ph2d_vector::Rect::new(
        f64::from(r.x),
        f64::from(r.y) + dy,
        f64::from(r.x + r.w),
        f64::from(r.y + r.h) + dy,
    );
    scene.blurred_rounded_rect(Affine::IDENTITY, rr, color, f64::from(radius), blur / 2.0);
}

/// A sombra de voo por baixo de `r` (com o `radius` de quem voa).
pub fn flight_shadow(scene: &mut VectorScene, theme: Theme, r: Rect, radius: f32) {
    let far = if theme.is_dark() {
        SHADOW_FLIGHT_DARK
    } else {
        SHADOW_FLIGHT_LIGHT
    };
    let cor = |t| crate::paint::resolve(t, theme);
    shadow(scene, r, radius, cor(ColorToken::KanbanShadowFlight), far);
    shadow(
        scene,
        r,
        radius,
        cor(ColorToken::KanbanFlightNear),
        SHADOW_FLIGHT_NEAR,
    );
}

/// Pinta em `r` o que foi desenhado em `local` (origem no canto de cima à esquerda), inclinado
/// `tilt_deg` em torno do pivô.
pub fn fly(scene: &mut VectorScene, local: &VectorScene, r: Rect, tilt_deg: f64) {
    let pivot = (f64::from(r.w) / 2.0, f64::from(r.h) * FLIGHT_PIVOT_Y);
    let t = Affine::translate((f64::from(r.x), f64::from(r.y)))
        * Affine::translate(pivot)
        * Affine::rotate(tilt_deg.to_radians())
        * Affine::translate((-pivot.0, -pivot.1));
    scene.append_transformed(local, t);
}
