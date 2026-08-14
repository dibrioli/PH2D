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
/// ⚠️ **É um TEMPO, e é o único número que calibra o `Speed`** — o Alchemy não oferece controle
/// nenhum ao artista (Enio 2026-08-13: *"em alchemy o slider não é necessário"*), então o produto
/// tem de acertar de fábrica em vez de delegar.
///
/// **De onde ele vem:** a mesma curva de um quarto de círculo (raio 200, arco 314 px) desenhada em
/// 8 quadros dá **~39 px de arco por quadro** ⇒ um gesto ligeiro corre a **~2 340 px/s** (plano 38
/// W0.1, medido). A um décimo de segundo isso são **~234 px** de arremesso — a ordem da cauda que as
/// figuras do Alchemy mostram para além do bico do pássaro, e num chicote de 12 000 px/s são
/// 1 200 px, o *"and possibly off the screen itself"* que o manual promete.
///
/// ⚠️ **É uma decisão de LOOK, e o SMOKE é quem a julga** — não há aqui um recurso a medir (o custo
/// de tinta é linear no caminho que ela percorre, e o `fill_thrown_gap` o paga honestamente). Movê-la
/// é UMA linha; o que não se pode é devolvê-la ao artista como slider, que é o que o Alchemy
/// deliberadamente não faz.
///
/// ⚠️ **Um tempo AUTO-ESCALA e um comprimento não:** a mesma constante arremessa 20 px num traço
/// lento e 400 num rápido, que é precisamente a lei que o `Speed Shapes` afirma. Um número em
/// pixels teria de ser re-escolhido por tamanho de tela, por zoom e por temperamento de mão.
pub const SPEED_LOOKAHEAD_S: f32 = 1.0 / 10.0;
