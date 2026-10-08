//! **A vista dos presets do Wet Painter** que a ponte da app publica (`painter_bridge_presets.rs`) e o
//! bloco do painel pinta (`paint_wetpaint_presets.rs`). Não cabe no `BrushSettings` (`Copy`): mora
//! numa célula própria, como a pilha de camadas. O painel nunca lê o disco.

use ph2d_editor_core::zones::Rect;
use std::cell::{Cell, RefCell};
use std::sync::Arc;

/// Um preset como o navegador o mostra.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VistaPreset {
    /// O caminho na biblioteca — a chave de tudo (o pedido `Escolher` leva-o).
    pub relativo: String,
    pub nome: String,
    /// `meio · grupo` (as pastas).
    pub grupo: String,
    pub favorito: bool,
}

/// O navegador inteiro.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VistaPresets {
    /// Pela ordem da biblioteca (meio, grupo, nome).
    pub entradas: Vec<VistaPreset>,
    /// Caminhos dos recentes, o mais recente primeiro.
    pub recentes: Vec<String>,
    pub ativo: Option<String>,
    /// A política dos volumes, índice em `Politica::TODAS`.
    pub politica: u8,
    /// A miniatura do activo, já recortada à faixa do traço: `(rgba, largura, altura)`.
    pub miniatura: Option<(Arc<Vec<u8>>, u32, u32)>,
    /// Quantos ficheiros da biblioteca não se leram (o navegador di-lo; a ponte diz quais).
    pub ilegiveis: usize,
}

impl VistaPresets {
    #[must_use]
    pub fn procurar(&self, relativo: &str) -> Option<&VistaPreset> {
        self.entradas.iter().find(|e| e.relativo == relativo)
    }
}

thread_local! {
    static CURRENT: RefCell<Option<Arc<VistaPresets>>> = const { RefCell::new(None) };
    pub(crate) static PENDING_PRESET_DD: Cell<Option<Rect>> = const { Cell::new(None) };
    pub(crate) static PENDING_POLICY_DD: Cell<Option<(Rect, u8)>> = const { Cell::new(None) };
    pub(crate) static PENDING_QUICK_DD: Cell<Option<(Rect, usize)>> = const { Cell::new(None) };
}

/// Publica a vista (a ponte, quando a biblioteca ou o estado mudam). `None` = sem biblioteca: o
/// bloco pinta só os volumes.
pub fn set_current_wet_presets(v: Option<Arc<VistaPresets>>) {
    CURRENT.with(|c| *c.borrow_mut() = v);
}

pub(crate) fn current_wet_presets() -> Option<Arc<VistaPresets>> {
    CURRENT.with(|c| c.borrow().clone())
}
