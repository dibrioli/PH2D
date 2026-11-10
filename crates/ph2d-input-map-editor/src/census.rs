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
    /// Cada tecla ENCHIDA com a cor de uma acção (a premida do Try it, maquete C2): onde e com que cor.
    pub filled: Vec<(Rect, ph2d_tokens::ColorToken)>,
    /// Cada ícone pintado ao lado de um texto (a direcção da manche na C3).
    pub icons: Vec<(Rect, ph2d_editor_core::icons::IconId)>,
    /// Cada pedaço de uma leitura em pedaços (a C3): o texto, a cor e se é negrito.
    pub runs: Vec<(String, ph2d_tokens::ColorToken, bool)>,
    /// Cada texto pintado em SemiBold (o nome da acção na A, o título da receita na C).
    pub semibold: Vec<String>,
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
