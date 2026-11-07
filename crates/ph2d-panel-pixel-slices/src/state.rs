//! O que o painel guarda: o menu do botão direito aberto, e se o seletor de cor estava aberto nele.

use crate::canal::Menu;

/// O estado da janela SLICES.
#[derive(Default)]
pub struct PixelSlicesPanelState {
    /// Onde o menu abriu e o que ele oferece.
    pub(crate) menu: Option<Menu>,
    /// O seletor partilhado estava nesta amostra no quadro anterior.
    pub(crate) a_escolher_cor: bool,
    /// A cor do seletor ao abrir, enquanto a mão não a mudou: com valor misto, abrir não é escolher.
    pub(crate) cor_ao_abrir: Option<[u8; 4]>,
}
