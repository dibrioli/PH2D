//! O canal das pré-visualizações do dropdown de fontes do Text (Pixel Art): o painel pede os nomes
//! que vai mostrar ([`want`]); a família os rasteriza com o próprio rasterizador do Pixel Art
//! ([`take_wanted`] → [`publish`]) e o painel desenha a máscara em células ([`with_preview`]).
//! Preguiçoso como o do Vector: só as linhas VISÍVEIS, uma vez por nome.

use std::cell::RefCell;
use std::collections::BTreeMap;

/// O nome de uma fonte rasterizado nela: `cover` tem `w·h` valores 0..=255, linha a linha.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PixelFontPreview {
    pub w: u32,
    pub h: u32,
    pub cover: Vec<u8>,
}

impl PixelFontPreview {
    /// **A escala da prévia numa linha de `row_h` px** — a maior INTEIRA (célula = quadrado inteiro
    /// de ecrã) com que a tinta cabe na linha (`≥ 1`). Medido (02/11, linha `ROW_H_PX` = 22; gate
    /// `every_pixel_font_preview_fills_the_list_row_at_an_integer_scale`), altura da tinta do nome:
    ///
    /// | fonte | tinta | antes (0,7 da linha) | agora |
    /// |---|---|---|---|
    /// | cozette | 8 | 1× = 8 px | 2× = 16 px |
    /// | tom-thumb | 5 | 3× = 15 px | 4× = 20 px |
    /// | unscii-8 | 8 | 1× = 8 px | 2× = 16 px |
    /// | unscii-16 | 16 | 1× = 16 px | 1× = 16 px |
    ///
    /// Todas ≥ 16 px: a linha de 22 basta, não precisa de crescer.
    #[must_use]
    pub fn scale_in(&self, row_h: f32) -> f32 {
        if self.h == 0 {
            return 1.0;
        }
        #[expect(clippy::cast_precision_loss, reason = "altura de uma máscara de nome")]
        let h = self.h as f32;
        (row_h / h).floor().max(1.0)
    }
}

thread_local! {
    static PREVIEWS: RefCell<BTreeMap<String, PixelFontPreview>> = const { RefCell::new(BTreeMap::new()) };
    static WANTED: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// O painel quer a pré-visualização de `name` (sem efeito se já existe ou já foi pedida).
pub fn want(name: &str) {
    if PREVIEWS.with(|p| p.borrow().contains_key(name)) {
        return;
    }
    WANTED.with(|w| {
        let mut w = w.borrow_mut();
        if !w.iter().any(|n| n == name) {
            w.push(name.to_owned());
        }
    });
}

/// A família drena os pedidos.
pub fn take_wanted() -> Vec<String> {
    WANTED.with(|w| std::mem::take(&mut *w.borrow_mut()))
}

/// A família publica a pré-visualização de `name`.
pub fn publish(name: String, preview: PixelFontPreview) {
    PREVIEWS.with(|p| p.borrow_mut().insert(name, preview));
}

/// Roda `f` com a pré-visualização de `name`, se já existe.
pub fn with_preview<R>(name: &str, f: impl FnOnce(Option<&PixelFontPreview>) -> R) -> R {
    PREVIEWS.with(|p| f(p.borrow().get(name)))
}
