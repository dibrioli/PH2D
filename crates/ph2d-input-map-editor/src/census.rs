//! O censo da pintura da aba (plano §3, «prova 22→0 e 38→0»): cada texto pintado (rect, elidido?) e cada campo
//! numérico (de que acção). A pintura enche-o; o teste lê-o.

use ph2d_editor_core::zones::Rect;
use ph2d_input::ActionId;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Census {
    /// Cada texto: onde, o texto, e se não coube (foi cortado com reticência).
    pub texts: Vec<(Rect, String, bool)>,
    /// Cada campo numérico, pela acção dona.
    pub numbers: Vec<ActionId>,
    /// Cada caixa pintada TRACEJADA (a escuta da A e o ladrilho da B, maquete A2).
    pub dashed: Vec<Rect>,
    /// O recorte da vista que rola (a C): o que sai dele não se pinta nem se clica.
    pub clip: Option<Rect>,
}

impl Census {
    #[must_use]
    pub fn elided(&self) -> Vec<&str> {
        self.texts
            .iter()
            .filter(|t| t.2)
            .map(|t| t.1.as_str())
            .collect()
    }

    #[must_use]
    pub fn numbers_of(&self, id: ActionId) -> usize {
        self.numbers.iter().filter(|n| **n == id).count()
    }
}
