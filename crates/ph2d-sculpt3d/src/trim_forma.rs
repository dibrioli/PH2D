//! **A FORMA que o Box Trim corta** — irmão do [`crate::SmearMode`] e do
//! [`crate::ProjectMode`]: um selector do pincel, no sítio onde os selectores
//! do pincel vivem.
//!
//! Ordem do dono (2026-09-15): *«Nos parâmetros botões box e circle (novo) e
//! laço.»*

/// A forma de ecrã que o gesto desenha (espec §3).
///
/// ⚠️ **A polilinha é tratada como um LAÇO** no alvo — a diferença está em como
/// o utilizador a desenha, não na máquina. ⏳ A **linha** (2 pontos, §4.2) é a
/// quarta e ainda não está aqui: ela é a mesma máquina com um quadrilátero
/// fabricado, e o modo dela é **forçado** a subtrair.
///
/// ⭐ **O CÍRCULO não vem da referência** — ela tem caixa, laço, linha e
/// polilinha. Ele é **pedido do dono**, e é a mesma máquina: dois pontos e um
/// anel. *Um anel é um anel, venha ele de dois cantos ou de um centro e um
/// raio.*
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TrimForma {
    /// Dois cantos ⇒ o rectângulo de ecrã.
    #[default]
    Caixa,
    /// Um centro e um raio ⇒ o círculo de ecrã.
    ///
    /// ⚠️ **O gesto é CENTRO-para-fora e não canto-a-canto**, ao contrário da
    /// caixa. A escolha é medida contra o NOME: um arrasto canto-a-canto dá uma
    /// **elipse** sempre que não for quadrado, e um controlo chamado *Circle*
    /// que entrega elipses é a espécie de rótulo que mente. ⏳ Se o dono quiser
    /// a elipse, ela é o outro gesto e merece o nome dela.
    Circulo,
    /// O caminho desenhado, ponto a ponto.
    Laco,
}

impl TrimForma {
    /// As três, na ordem em que o painel as pinta.
    pub const ALL: [Self; 3] = [Self::Caixa, Self::Circulo, Self::Laco];

    /// O rótulo do chip — as palavras do dono (a UI da casa é inglesa).
    ///
    /// ⚠️ **Não é o nome da FERRAMENTA**, que é o `Verb::BoxTrim.label()`
    /// (*«Box Trim»*): aqui o que se escolhe é a forma, e *o nome da ferramenta
    /// e o nome da forma são duas perguntas.*
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Caixa => "Box",
            Self::Circulo => "Circle",
            Self::Laco => "Lasso",
        }
    }

    /// **Esta forma lê o CAMINHO da mão?** — só o laço, e é ele que a
    /// suavização serve.
    ///
    /// ⭐ A caixa e o círculo guardam **dois** pontos: não há traço a suavizar,
    /// e a régua de um deles é exacta por construção. ⇒ o painel esconde a
    /// pista da suavização fora do laço, pela porta e não por um `match` no
    /// pintor.
    #[must_use]
    pub fn le_o_caminho(self) -> bool {
        matches!(self, Self::Laco)
    }
}
