//! ⭐ **A lei do valor dos cartões do núcleo e da imagem** (doc CodeEditor 08 §3.5, d3) — o MESMO
//! saneamento que o `apply_*` do cartão faz e que o script recebe depois de `ph2d.set` (pela tabela
//! [`crate::property_laws::LEIS`]). Onde o componente já tem o método da lei (`clamped`,
//! `sanitized`), a função é ele; onde a lei vivia escrita no `apply_*` da shell, mora aqui e a shell
//! chama-a.

use ph2d_ecs::{
    MaskInteraction, SPEED_MAX_Q16, SliceNine, SpriteAnimator, SpriteEmissive, SpriteGrid,
    SpriteRegion, ZIndexOverride,
};
use ph2d_render::Sprite;

/// A opacidade da §Color & Tint: `0..=1`.
pub fn sanitize_sprite(s: &mut Sprite) {
    s.opacity = s.opacity.clamp(0.0, 1.0);
}

/// A grelha da §Sprite Sheet: pelo menos uma coluna e uma linha, e o quadro dentro delas.
pub fn sanitize_sprite_grid(g: &mut SpriteGrid) {
    g.hframes = g.hframes.max(1);
    g.vframes = g.vframes.max(1);
    let cells = g.cells().max(1);
    if g.frame >= cells {
        g.frame = cells - 1;
    }
}

/// A região da §Render Source: largura e altura nunca negativas (a origem fica).
pub fn sanitize_sprite_region(r: &mut SpriteRegion) {
    r.rect[2] = r.rect[2].max(0.0);
    r.rect[3] = r.rect[3].max(0.0);
}

/// As três leis do cartão da sprite, depois de UMA edição dele.
pub fn sanitize_sprite_editables(s: &mut Sprite, g: &mut SpriteGrid, r: Option<&mut SpriteRegion>) {
    sanitize_sprite(s);
    sanitize_sprite_grid(g);
    if let Some(r) = r {
        sanitize_sprite_region(r);
    }
}

/// A velocidade da §11 Animation: a faixa do motor (±100×).
pub fn sanitize_sprite_animator(p: &mut SpriteAnimator) {
    p.speed_q16 = p.speed_q16.clamp(-SPEED_MAX_Q16, SPEED_MAX_Q16);
}

/// A §5 9-Slice: [`SliceNine::sanitized`] (bordas e tamanho finitos e não-negativos, cantos fixos).
pub(crate) fn sanitize_slice_nine(s: &mut SliceNine) {
    *s = s.sanitized();
}

/// A linha *Emissive*: [`SpriteEmissive::clamped`].
pub(crate) fn sanitize_sprite_emissive(e: &mut SpriteEmissive) {
    e.0 = e.clamped();
}

/// A §7 *Z Index*: [`ZIndexOverride::clamped`].
pub(crate) fn sanitize_z_index(z: &mut ZIndexOverride) {
    z.0 = ZIndexOverride::clamped(z.0);
}

/// A §8 *Mask Alpha Cutoff*: [`MaskInteraction::clamped`].
pub(crate) fn sanitize_mask_interaction(m: &mut MaskInteraction) {
    *m = m.clamped();
}
