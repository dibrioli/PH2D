//! ⭐ **A imagem que a ranhura mostra** ([plano GEF 12](../../../docs/GameEngineFuncional/12_imagem_como_asset_plano.md)):
//! a miniatura e o nome da imagem da biblioteca que a sprite seleccionada usa — o campo *Sprite* do *Sprite
//! Renderer* da Unity. Publicado pelo shell por quadro, como os outros instantâneos do painel.

use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::{ImageQuality, StableImage, VectorScene};
use std::cell::RefCell;
use std::sync::Arc;

/// A imagem de uma sprite: de quem, o nome da biblioteca e a miniatura (RGBA8 reto).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotImage {
    pub entity_bits: u64,
    pub name: String,
    pub rgba: Arc<Vec<u8>>,
    pub w: u32,
    pub h: u32,
}

thread_local! {
    static CURRENT: RefCell<Option<SlotImage>> = const { RefCell::new(None) };
    /// A textura estável da miniatura — sem ela o `vello` reenvia os bytes todo o quadro.
    static TEX: RefCell<Option<(Arc<Vec<u8>>, StableImage)>> = const { RefCell::new(None) };
}

/// O shell publica a imagem da sprite seleccionada (`None` = nenhuma da biblioteca).
pub fn set_current_slot_image(img: Option<SlotImage>) {
    CURRENT.with(|c| *c.borrow_mut() = img);
}

/// A imagem publicada para `entity_bits`.
#[must_use]
pub fn current_slot_image(entity_bits: u64) -> Option<SlotImage> {
    CURRENT.with(|c| {
        c.borrow()
            .as_ref()
            .filter(|i| i.entity_bits == entity_bits)
            .cloned()
    })
}

/// Pinta a miniatura (quadrada à altura da ranhura, aspecto preservado) e o nome. `false` se não há imagem.
pub(crate) fn paint(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    slot: Rect,
    entity_bits: u64,
) -> bool {
    let Some(img) = current_slot_image(entity_bits) else {
        return false;
    };
    let inset = Spacing::Xxs.px();
    let side = (slot.h - inset * 2.0).max(0.0);
    let stable = TEX.with(|t| {
        let mut t = t.borrow_mut();
        if let Some((rgba, s)) = t.as_ref()
            && Arc::ptr_eq(rgba, &img.rgba)
        {
            return Some(s.clone());
        }
        let s = StableImage::from_rgba(img.rgba.clone(), img.w, img.h)?;
        *t = Some((img.rgba.clone(), s.clone()));
        Some(s)
    });
    if let Some(s) = stable {
        let (tw, th) = (img.w.max(1) as f32, img.h.max(1) as f32);
        let k = (side / tw).min(side / th);
        let (dw, dh) = (tw * k, th * k);
        let x0 = f64::from(slot.x + inset + (side - dw) * 0.5);
        let y0 = f64::from(slot.y + inset + (side - dh) * 0.5);
        scene.draw_stable_image(
            &s,
            (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
            ImageQuality::Medium,
        );
    }
    let text_x = slot.x + inset + side + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        &img.name,
        text_x,
        slot.y + (slot.h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        (slot.x + slot.w - text_x - Spacing::Xs.px()).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
    true
}
