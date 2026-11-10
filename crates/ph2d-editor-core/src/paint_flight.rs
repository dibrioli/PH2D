//! **O voo de um arrasto** — o que está agarrado descola: sombra de voo em duas camadas, inclinado
//! em torno de `(50 %, 60 %)`, por cima de tudo. Nasceu nos cartões do Kanban (o Trello); o bloco de
//! quadros da linha do tempo do Pixel voa com a MESMA lei.

use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, Color, VectorScene};

/// A lei mora em `ph2d-card-stack`: a inclinação, e `(dy, desfoque)` da sombra de voo.
pub use ph2d_card_stack::FLIGHT_TILT_DEG;
use ph2d_card_stack::flight_pivot;
pub use ph2d_card_stack::shadow::{SHADOW_FLIGHT_DARK, SHADOW_FLIGHT_LIGHT, SHADOW_FLIGHT_NEAR};

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
    let pivot = flight_pivot(r.w, r.h);
    let t = Affine::translate((f64::from(r.x), f64::from(r.y)))
        * Affine::translate(pivot)
        * Affine::rotate(tilt_deg.to_radians())
        * Affine::translate((-pivot.0, -pivot.1));
    scene.append_transformed(local, t);
}

/// ⭐ A sombra de uma alça do canvas (a lei da câmera e dos contornos, GEF02): `(cor, alargamento)` —
/// a forma pintada antes da alça, `Thin` mais larga de cada lado, em `BgScrim` a
/// [`ph2d_tokens::OUTLINE_SHADOW_ALPHA`].
#[must_use]
pub fn sombra_de_alca(theme: Theme) -> (Color, f32) {
    let mut c = crate::paint::resolve(ColorToken::BgScrim, theme);
    c.components[3] = ph2d_tokens::OUTLINE_SHADOW_ALPHA;
    (c, ph2d_tokens::StrokeToken::Thin.px())
}
