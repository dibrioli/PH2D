//! ⭐ **A ALÇA de uma borda que se puxa** — a faixa de agarre ([`DOCK_SEAM_PX`]) e a marca pintada
//! nela ([`paint_seam_mark`]). Moram ABAIXO de `screens` e de `panel` porque os dois as pintam: a
//! coluna do Inspector (`screens::hero::dock_reopen`) e as janelas acessórias (`panel::cards::borda`),
//! a mesma alça, o mesmo desenho — sem a aresta `panel → screens` do DAG da fundação.

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

/// **A faixa de agarre que redimensiona uma coluna** — a borda INTERIOR dela.
///
/// ⭐ Enio, 2026-08-30: *«os painéis devem ser redimensionáveis para esquerda e para direita e com
/// setas bidirecionais no cursor; os pontinhos de redimensionamento podem ser retirados. A borda
/// inteira serve para redimensionar»*.
///
/// ⚠️ **Ela vive DENTRO da coluna**, nos últimos [`DOCK_SEAM_PX`] px antes da área de desenho — e
/// não a cavalo da fronteira, como uma costura de divisória costuma ser. A razão é medida: desde
/// que a régua ficou **colada** à coluna, a faixa dela começa exactamente onde a coluna acaba, e
/// uma costura centrada roubaria metade do agarre à régua. *A borda é do painel.*
pub const DOCK_SEAM_PX: f32 = 6.0; // LITERAL-PX-OK: largura de agarre, irmã do GRAB_HALF_PX do 3D

/// A largura do chevron desenhado dentro da faixa.
pub const ARROW_W: f32 = 2.0; // LITERAL-PX-OK: traço do chevron da alça, dentro de uma faixa de 6

/// O rect da marca dentro da faixa `r`: ao meio, na altura de uma linha.
#[must_use]
pub fn seam_mark(r: Rect) -> Rect {
    let mark_h = ph2d_tokens::ROW_H_PX.min(r.h);
    Rect::new(
        r.x + (r.w - ARROW_W) * 0.5,
        r.y + (r.h - mark_h) * 0.5,
        ARROW_W,
        mark_h,
    )
}

/// Pinta a marca da alça sobre a faixa `r` — a da coluna do Inspector e a das janelas acessórias
/// (`panel::cards::borda`): a mesma alça, o mesmo desenho.
pub fn paint_seam_mark(scene: &mut VectorScene, r: Rect, theme: Theme) {
    let mark = seam_mark(r);
    fill_rounded_rect(
        scene,
        mark,
        ARROW_W * 0.5,
        resolve(ColorToken::Text3, theme),
    );
}
