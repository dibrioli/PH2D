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

/// O que o cartão precisa de saber da sprite seleccionada e o snapshot não diz.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpriteFacts {
    pub entity_bits: u64,
    /// O quadrado branco de quem ainda não escolheu imagem: o cartão Image é o convite.
    pub sem_imagem: bool,
    pub tem_filhos: bool,
    /// O ponto da grade do Pivot em que o pivô cai: `[coluna, linha]`; `None` = Custom.
    pub pivot_ponto: Option<[u8; 2]>,
    /// O pivô em px a partir do centro da imagem, `y` para cima.
    pub pivot_px: [f32; 2],
    /// Em Still, a grelha da imagem que é folha (colunas, linhas): o aviso de folha.
    pub folha: Option<[u32; 2]>,
    /// Em Animated, a imagem trocada sem corte próprio: o aviso.
    pub corte_velho: Option<ph2d_editor_core::ImageCutNotice>,
}

/// O que o Replace acabou de mudar: os píxeis de antes e de agora, e o tamanho no mundo de antes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Recibo {
    pub entity_bits: u64,
    pub de: [u32; 2],
    pub para: [u32; 2],
    pub tamanho: [f32; 2],
}

thread_local! {
    static RECIBO: std::cell::Cell<Option<Recibo>> = const { std::cell::Cell::new(None) };
    static PINTAR_NO_LUGAR_DE: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
    static FACTS: std::cell::Cell<Option<SpriteFacts>> = const { std::cell::Cell::new(None) };
    static CURRENT: RefCell<Option<SlotImage>> = const { RefCell::new(None) };
    /// A textura estável de cada miniatura à vista (a da seleção e as dos presos) — sem ela o `vello`
    /// reenvia os bytes todo o quadro. A mais antiga sai ao passar de [`TEXTURAS`].
    static TEX: RefCell<Vec<(Arc<Vec<u8>>, StableImage)>> = const { RefCell::new(Vec::new()) };
}

/// Quantas miniaturas ficam em cache: é memória de GPU, não um limite de cartões (acima disto, reenvia).
const TEXTURAS: usize = 16;

/// O shell publica o que o Replace mudou (`None` = nada a dizer).
pub fn set_recibo(r: Option<Recibo>) {
    RECIBO.with(|c| c.set(r));
}

pub(crate) fn recibo() -> Option<Recibo> {
    RECIBO.with(std::cell::Cell::get)
}

pub(crate) fn set_pintar_no_lugar_de(bits: Option<u64>) {
    PINTAR_NO_LUGAR_DE.with(|c| c.set(bits));
}

/// O objecto vazio que a imagem nova vai substituir, se ela nasceu do convite do cartão.
pub fn take_pintar_no_lugar_de() -> Option<u64> {
    PINTAR_NO_LUGAR_DE.with(std::cell::Cell::take)
}

/// O shell publica os factos da sprite seleccionada.
pub fn set_current_sprite_facts(f: Option<SpriteFacts>) {
    FACTS.with(|c| c.set(f));
}

/// Os factos publicados (o retrato do objecto, T61).
pub(crate) fn factos() -> Option<SpriteFacts> {
    FACTS.with(std::cell::Cell::get)
}

fn facto(entity_bits: u64, f: impl Fn(SpriteFacts) -> bool) -> bool {
    factos().is_some_and(|s| s.entity_bits == entity_bits && f(s))
}

/// `entity_bits` é o quadrado branco de quem ainda não escolheu imagem?
pub(crate) fn sem_imagem(entity_bits: u64) -> bool {
    facto(entity_bits, |s| s.sem_imagem)
}

/// `entity_bits` tem filhos?
pub(crate) fn tem_filhos(entity_bits: u64) -> bool {
    facto(entity_bits, |s| s.tem_filhos)
}

/// O shell publica a imagem da sprite seleccionada (`None` = nenhuma da biblioteca).
pub fn set_current_slot_image(img: Option<SlotImage>) {
    CURRENT.with(|c| *c.borrow_mut() = img);
}

/// A imagem publicada (o retrato do objecto, T61).
pub(crate) fn atual() -> Option<SlotImage> {
    CURRENT.with(|c| c.borrow().clone())
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

/// A textura estável da miniatura de `img` (a ranhura e a miniatura da folha partilham-na).
pub(crate) fn stable(img: &SlotImage) -> Option<StableImage> {
    TEX.with(|t| {
        let mut t = t.borrow_mut();
        if let Some((_, s)) = t.iter().find(|(rgba, _)| Arc::ptr_eq(rgba, &img.rgba)) {
            return Some(s.clone());
        }
        let s = StableImage::from_rgba(img.rgba.clone(), img.w, img.h)?;
        if t.len() >= TEXTURAS {
            t.remove(0);
        }
        t.push((img.rgba.clone(), s.clone()));
        Some(s)
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
    if let Some(s) = stable(&img) {
        let (tw, th) = (img.w.max(1) as f32, img.h.max(1) as f32);
        let k = (side / tw).min(side / th);
        let (dw, dh) = (tw * k, th * k);
        let x0 = f64::from(slot.x + inset + (side - dw) * 0.5);
        let y0 = f64::from(slot.y + inset + (side - dh) * 0.5);
        scene.draw_stable_image(
            &s,
            (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
            ph2d_editor_core::ui_image_quality(),
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
