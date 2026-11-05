//! O canal das pré-visualizações do dropdown de fontes do Text (Pixel Art): o painel pede os nomes
//! que vai mostrar ([`want`]); a família os rasteriza com o próprio rasterizador do Pixel Art
//! ([`take_wanted`] → [`publish`]) e o painel desenha a máscara em células ([`with_preview`]).
//! Preguiçoso como o do Vector: só as linhas VISÍVEIS, uma vez por nome.

use std::cell::RefCell;
use std::collections::BTreeMap;

/// A altura de tinta que as linhas da lista procuram, para as 12 fontes se lerem do mesmo tamanho.
pub const ALTURA_DA_PREVIA: f32 = 16.0;

/// O nome de uma fonte rasterizado nela, recortado à tinta: `cover` tem `w·h` valores 0..=255.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PixelFontPreview {
    pub w: u32,
    pub h: u32,
    pub cover: Vec<u8>,
    /// O tamanho nativo da fonte em px, que a linha mostra à direita (0 = não mostra).
    pub native: u32,
}

impl PixelFontPreview {
    /// **A escala da prévia numa linha de `row_h` px** — a INTEIRA (célula = quadrado inteiro de
    /// ecrã) que leva a tinta para mais perto de [`ALTURA_DA_PREVIA`], sem passar da linha menos a
    /// folga [`Spacing::Xs`] em cima e em baixo. Com a maior que coubesse, a Tom Thumb saía a 20 px
    /// e pesada ao lado da Press Start a 14 (foto da cena 19, 05/11). Medido na linha de 28:
    ///
    /// | tinta | fontes | escala | px |
    /// |---|---|---|---|
    /// | 5 | Micro 5, Silkscreen, Tom Thumb | 3× | 15 |
    /// | 7–8 | Press Start 2P, Pixel Operator 8, Unscii 8 | 2× | 14–16 |
    /// | 9–10 | Cozette, Departure Mono, Pixelify Sans, Pixel Operator, Unscii 16, VT323 | 2× | 18–20 |
    #[must_use]
    pub fn scale_in(&self, row_h: f32) -> f32 {
        if self.h == 0 {
            return 1.0;
        }
        #[expect(clippy::cast_precision_loss, reason = "altura de uma máscara de nome")]
        let h = self.h as f32;
        let cabe = ((row_h - 2.0 * ph2d_tokens::Spacing::Xs.px()) / h).floor();
        (ALTURA_DA_PREVIA / h).round().min(cabe).max(1.0)
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
