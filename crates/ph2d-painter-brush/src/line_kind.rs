//! **O TIPO DE LINHA PROCEDURAL** — a lei que decora o traço além do depósito de dabs (plano 38 §1,
//! pesquisa no [doc 37](../../../docs/Painter/37_pesquisa_tracos_procedurais.md)).
//!
//! ⚠️ **Não é o [`crate::StrokeMethod`], e a distinção decide onde cada coisa mora:** o
//! `StrokeMethod` responde *como este caminho é AUTORADO* (mão livre, linha, elipse, polígono…) e o
//! `LineKind` responde *que lei procedural decora o caminho autorado*. As duas perguntas são
//! ortogonais — um `Speed` sobre um caminho de mão livre e um `Speed` sobre um arco são o mesmo
//! tipo de linha sobre dois métodos.
//!
//! Clean-room: o Alchemy é GPL-3, e tudo o que este módulo sabe dele saiu do **manual**
//! (comportamento), nunca do fonte.

/// O tipo de linha procedural do pincel. `None` é o neutro **byte-idêntico**: com ele nada neste
/// módulo alcança um dab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineKind {
    /// Sem lei procedural — o traço é o depósito de dabs de sempre.
    #[default]
    None,
    /// **Speed Shapes** (manual do Alchemy, verbatim: *"Accentuates the pen speed to create shapes
    /// that throw the line beyond the actual pen position"*): a tinta é ARREMESSADA à frente do
    /// dedo na proporção da velocidade do gesto — é INÉRCIA, o oposto exato do estabilizador, que
    /// atrasa a linha.
    Speed,
}

impl LineKind {
    /// Wire `u8` para o round-trip com o painel (`0` = None · `1` = Speed).
    pub fn to_wire(self) -> u8 {
        match self {
            LineKind::None => 0,
            LineKind::Speed => 1,
        }
    }

    /// Inversa de [`Self::to_wire`]; qualquer valor desconhecido cai no neutro.
    pub fn from_wire(w: u8) -> Self {
        match w {
            1 => LineKind::Speed,
            _ => LineKind::None,
        }
    }
}

/// **A JANELA DE ANTECIPAÇÃO, em segundos** — quanto do futuro o `Amount = 1` arremessa.
///
/// ⚠️ **O número é MEDIDO, não escolhido** (plano 38 W0.1): a mesma curva de um quarto de círculo
/// (raio 200, arco 314 px) desenhada em 8 quadros dá **~39 px de arco por quadro** ⇒ um gesto ligeiro
/// corre a **~2 340 px/s**. Com a janela num quadro de 60 fps, `Amount = 1` arremessa exatamente o
/// arco de um quadro — **~39 px** naquele gesto —, e o teto do slider ([`MAX_SPEED_AMOUNT`]) diz
/// quantos quadros à frente a tinta pode ir.
pub const SPEED_LOOKAHEAD_S: f32 = 1.0 / 60.0;

/// O teto do `Amount`: **oito quadros** de antecipação.
///
/// ⚠️ Ele diz de que recurso é: no gesto medido acima são `8 × 39 = 312 px` de arremesso — o arco do
/// quarto de círculo INTEIRO, o *"and possibly off the screen itself"* que o manual promete. Acima
/// disso a tinta está mais de um oitavo de segundo à frente da mão e o traço deixa de ser um traço.
pub const MAX_SPEED_AMOUNT: f32 = 8.0;
