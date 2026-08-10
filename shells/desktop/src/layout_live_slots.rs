//! **A RÉGUA que o gesto de reordenar lê** — irmão do [`super::layout_live`] pelo teto de LOC, e o
//! corte é por ASSUNTO: aqui mora *onde os filhos ficaram, e como essa fila se lê*, e o que fica lá
//! responde outra pergunta (*como é que se colocam*).
//!
//! ⚠️ **A régua é PUBLICADA por quem colocou, e nunca re-derivada por quem arrasta.** Um gesto que
//! recalculasse as posições seria a segunda resposta a *"onde este filho está?"*, e as duas
//! divergiriam no primeiro `grow` — o artista veria a forma numa posição e o slot ser escolhido por
//! outra.

use ph2d_ecs::Entity;

/// A caixa de MUNDO em que uma coisa ficou: `(canto inferior-esquerdo, canto superior-direito)`.
pub(crate) type Box2 = ([f64; 2], [f64; 2]);

/// **Onde o último passe PÔS os filhos de uma moldura** — a régua que o gesto de reordenar lê.
///
/// ⚠️ Ela é PUBLICADA por quem colocou, e nunca re-derivada por quem arrasta. Um gesto que
/// recalculasse as posições seria a segunda resposta a *"onde este filho está?"*, e as duas
/// divergiriam no primeiro `grow` — o artista veria a forma numa posição e o slot ser escolhido
/// por outra.
pub(crate) struct FlowSlots {
    /// **Como esta fila se LÊ** — e é a pergunta inteira do gesto de reordenar.
    pub(crate) reading: Reading,
    /// Os filhos na ORDEM do fluxo, cada um com a CAIXA de mundo em que ficou.
    ///
    /// ⚠️ A caixa, e não o centro num eixo. Uma fila 1-D só precisa do centro, mas uma fila em
    /// LINHAS precisa de saber em que linha um filho caiu — e isso é a banda `y` dele. Publicar o
    /// facto cru e deixar a PERGUNTA para quem arrasta é o que evita que a régua tenha de escolher
    /// antecipadamente qual das duas leituras o gesto vai querer.
    pub(crate) kids: Vec<(Entity, Box2)>,
}

/// **Como a ordem de uma fila se lê a partir de um ponto.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reading {
    /// Uma fila só, ao longo de X.
    RowX,
    /// Uma fila só, ao longo de Y.
    ColumnY,
    /// **Em LINHAS** — a ordem é a de leitura (linha, depois coluna). É o `RowWrap` e a `Grid`.
    Rows,
}
