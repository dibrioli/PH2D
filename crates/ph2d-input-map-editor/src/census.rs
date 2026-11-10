//! O censo da pintura da aba (plano §3, «prova 22→0 e 38→0»): cada texto pintado (rect, elidido?) e cada campo
//! numérico (de que acção). A pintura enche-o; o teste lê-o.

use ph2d_editor_core::zones::Rect;
use ph2d_input::ActionId;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Census {
    /// A caixa da dica do hover pintada (sempre inteira dentro da aba).
    pub tip: Option<Rect>,
    /// Cada texto: onde, o texto, e se não coube (foi cortado com reticência).
    pub texts: Vec<(Rect, String, bool)>,
    /// Cada campo numérico, pela acção dona.
    pub numbers: Vec<ActionId>,
    /// Cada caixa pintada TRACEJADA (a escuta da A e o ladrilho da B, maquete A2).
    pub dashed: Vec<Rect>,
    /// O recorte da vista que rola (a C): o que sai dele não se pinta nem se clica.
    pub clip: Option<Rect>,
    /// Cada tecla ENCHIDA com a cor de uma acção (a premida do Try it, maquete C2): onde e com que cor. E o fundo e a
    /// faixa de cada frase de AVISO (o pé da A, o cartão aberto da B, a faixa do Play).
    pub filled: Vec<(Rect, ph2d_tokens::ColorToken)>,
    /// Cada ícone pintado ao lado de um texto (a direcção da manche na C3).
    pub icons: Vec<(Rect, ph2d_editor_core::icons::IconId)>,
    /// Cada pedaço de uma leitura em pedaços (a C3): o texto, a cor e se é negrito.
    pub runs: Vec<(String, ph2d_tokens::ColorToken, bool)>,
    /// Cada texto pintado em SemiBold (o nome da acção na A, o título da receita na C).
    pub semibold: Vec<String>,
    /// A A: cada frase do pé (o fundo dela) — UMA por vez.
    pub foot: Vec<Rect>,
    /// A A: cada linha contornada no acento (a que está à escuta, maquete A2).
    pub outlined: Vec<Rect>,
    /// A A: cada «+» que escuta DESENHADO (sob o ponteiro, ou numa linha sem ligação).
    pub plus: Vec<ActionId>,
    /// Cada tecla desenhada (B e C): a face, a tinta, o contorno e a legenda — o olhar inteiro de cada estado.
    pub keys: Vec<KeyLook>,
    /// Cada barra de acção pintada (teclas, comando, «Also:»): onde e de que acção.
    pub bars: Vec<(Rect, ActionId)>,
    /// Cada texto pintado no peso do corpo (Regular, maquetes A e B).
    pub regular: Vec<String>,
    /// Cada desvanecer pintado na borda cortada de uma região que rola (A e B).
    pub fades: Vec<Rect>,
    /// Cada barra de rolagem pintada: (polegar, região que ela rola, calha reservada).
    pub scrollbars: Vec<(Rect, Rect, Rect)>,
    /// Cada ladrilho de ligação da B (o fundo do chip e o token dele).
    pub chips: Vec<(Rect, ph2d_tokens::ColorToken)>,
    /// A A: o olhar de cada controlo da linha no estado em que se pintou (linha, chip, ×, «No key», «+», lixo).
    pub looks: Vec<ControlLook>,
    /// A A: cada emblema redondo que apaga (o × do chip), à vista: o id e a face pintada.
    pub badges: Vec<(ph2d_a11y::NodeId, Rect)>,
}

/// Como um controlo da A se pintou: o fundo, a aresta (e se é tracejada), a tinta e a marca do acento à esquerda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlLook {
    pub id: ph2d_a11y::NodeId,
    /// O alvo (o rect registado): o mesmo em todo estado.
    pub rect: Rect,
    pub fill: Option<ph2d_tokens::ColorToken>,
    pub edge: Option<ph2d_tokens::ColorToken>,
    pub ink: ph2d_tokens::ColorToken,
    /// A barra do acento à esquerda (a linha à escuta).
    pub marker: bool,
}

/// Como uma tecla desenhada se pintou.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyLook {
    pub key: ph2d_input::Key,
    pub rect: Rect,
    pub face: ph2d_tokens::ColorToken,
    pub ink: ph2d_tokens::ColorToken,
    /// O contorno: só o acento da acção aberta (a tecla do sistema não tem nenhum).
    pub edge: Option<ph2d_tokens::ColorToken>,
    /// A face de repouso pintada translúcida sobre o fundo (a tecla do sistema): menos ênfase que o repouso.
    pub dimmed: bool,
    /// A faixa escura por baixo das barras (face cheia: premida ou acesa), para as barras não sumirem nela.
    pub strip: Option<Rect>,
    /// A plaquinha («adesivo») por baixo da legenda e por cima das barras.
    pub plate: Rect,
    /// A legenda em texto e o seu tamanho; `None` = ícone.
    pub text: Option<(String, f32)>,
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

    /// O olhar pintado de `id` (`None` = não se pintou neste quadro).
    #[must_use]
    pub fn look(&self, id: ph2d_a11y::NodeId) -> Option<ControlLook> {
        self.looks.iter().rev().find(|l| l.id == id).copied()
    }

    #[must_use]
    pub fn numbers_of(&self, id: ActionId) -> usize {
        self.numbers.iter().filter(|n| **n == id).count()
    }
}
