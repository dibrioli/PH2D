//! **O commit da §2 Sprite** — o que uma [`SpriteFieldEdit`] escreve no componente.
//!
//! ⚠️ **Irmão de [`super::inspector_commits`] por CAP de LOC** (HR-18, 600 no shell): aquele
//! ficheiro é o **roteador** de todos os commits do Inspector e cresce uma família por seção — a
//! §11 Animation levou-o a 611 em 2026-08-23. A regra da casa é cortar para o irmão, nunca
//! declarar exceção, e o que sai é o que não é roteamento.

use ph2d_editor::SpriteFieldEdit;
use ph2d_render::Sprite;

/// Apply one [`SpriteFieldEdit`] to a `Sprite`, enforcing the schema
/// invariants the Inspector widgets can't (anatomia §1.6): `hframes`/
/// `vframes >= 1`, `frame < hframes*vframes`, `opacity ∈ [0, 1]`. The
/// frame index is re-clamped whenever the grid shrinks so a stale frame
/// can never index past the sheet. This is the single authoring write
/// boundary for editable Sprite fields (mirrors `Transform::clamp_skew`).
pub(super) fn apply_sprite_field(sprite: &mut Sprite, edit: SpriteFieldEdit) {
    match edit {
        SpriteFieldEdit::FlipX(b) => sprite.flip_x = b,
        SpriteFieldEdit::FlipY(b) => sprite.flip_y = b,
        SpriteFieldEdit::Centered(b) => sprite.centered = b,
        // Per-axis: preserve the OTHER axis (so a bulk edit of one axis
        // can't stomp a diverging sibling — audit D-1).
        SpriteFieldEdit::OffsetX(x) => sprite.offset[0] = x,
        SpriteFieldEdit::OffsetY(y) => sprite.offset[1] = y,
        SpriteFieldEdit::Hframes(n) => {
            sprite.hframes = n.max(1);
            clamp_frame(sprite);
        }
        SpriteFieldEdit::Vframes(n) => {
            sprite.vframes = n.max(1);
            clamp_frame(sprite);
        }
        SpriteFieldEdit::Frame(f) => {
            sprite.frame = f;
            clamp_frame(sprite);
        }
        SpriteFieldEdit::RegionEnabled(b) => sprite.region_enabled = b,
        SpriteFieldEdit::RegionRect(r) => {
            // Schema invariant (anatomia §1.6): w/h kept `>= 0`. A negative
            // extent would invert the sampled UV; x/y may be negative (the
            // extract clamps the rect into the source).
            sprite.region_rect = [r[0], r[1], r[2].max(0.0), r[3].max(0.0)];
        }
        // Per-axis: preserve the other three components (audit D-1). W/H
        // floor at 0 like the whole-vector path.
        SpriteFieldEdit::RegionX(x) => sprite.region_rect[0] = x,
        SpriteFieldEdit::RegionY(y) => sprite.region_rect[1] = y,
        SpriteFieldEdit::RegionW(w) => sprite.region_rect[2] = w.max(0.0),
        SpriteFieldEdit::RegionH(h) => sprite.region_rect[3] = h.max(0.0),
        SpriteFieldEdit::RegionFilterClip(b) => sprite.region_filter_clip = b,
        SpriteFieldEdit::Tint(c) => sprite.tint = c,
        SpriteFieldEdit::SelfTint(c) => sprite.self_tint = c,
        SpriteFieldEdit::TintFill(b) => sprite.tint_fill = b,
        SpriteFieldEdit::Opacity(o) => sprite.opacity = o.clamp(0.0, 1.0),
        // ⚠️ Um canto, lido e escrito **na sprite desta iteração** — é isso que faz o fan-out
        // preservar os cantos divergentes das outras em vez de os atropelar.
        SpriteFieldEdit::PerCornerTintAt(i, rgba) => {
            if let Some(slot) = sprite.per_corner_tint.get_mut(usize::from(i)) {
                *slot = rgba;
            }
        }
        // Cada sprite iguala pelo SEU próprio TL — «igualar» é uma operação, não um valor.
        SpriteFieldEdit::EqualizeCorners => {
            sprite.per_corner_tint = [sprite.per_corner_tint[0]; 4];
        }
    }
}

/// Clamp `frame` into `[0, hframes*vframes - 1]`. `hframes`/`vframes`
/// are always `>= 1` here (set via `apply_sprite_field`), so the grid
/// has at least one cell.
pub(super) fn clamp_frame(sprite: &mut Sprite) {
    let cells = sprite.hframes.saturating_mul(sprite.vframes).max(1);
    if sprite.frame >= cells {
        sprite.frame = cells - 1;
    }
}

// §7 ordering commit handler lives in the sibling `inspector_ordering`
// module (HR-18 LOC + separation): `apply_ordering_edit`.
