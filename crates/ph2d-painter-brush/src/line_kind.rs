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
    /// **Sketchy** — o traço costura-se a si mesmo: a cada dab, fios de opacidade baixa ligam os
    /// pontos vizinhos do MESMO traço, e o acúmulo desenha o hachurado de lápis (Ze Frank →
    /// Harmony → Krita *Sketch*). Ver [`crate::stroke::sketchy`].
    Sketchy,
}

impl LineKind {
    /// Wire `u8` para o round-trip com o painel (`0` = None · `1` = Speed · `2` = Sketchy).
    pub fn to_wire(self) -> u8 {
        match self {
            LineKind::None => 0,
            LineKind::Speed => 1,
            LineKind::Sketchy => 2,
        }
    }

    /// Inversa de [`Self::to_wire`]; qualquer valor desconhecido cai no neutro.
    pub fn from_wire(w: u8) -> Self {
        match w {
            1 => LineKind::Speed,
            2 => LineKind::Sketchy,
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

/// **Teto da `Density` — PROVISÓRIO, e o doc diz por quê.**
///
/// ⚠️ **O teto que o plano derivou não sobreviveu à porta do produto.** A W0.3 mediu o comprimento de
/// fio contra o arco num **quarto de círculo** — um gesto que não se sobrepõe — e derivou 4% para
/// manter o gasto em 2× o traço. Medido pelo motor numa **ESPIRAL**, que é o gesto para o qual esta
/// feature existe, a mesma densidade deposita **8,45× · 10,64× · 3,54×** o arco (4, 16 e 64 voltas):
/// quatro a cinco vezes o que a derivação prometia, *e variando com o gesto* — um traço que volta
/// sobre si mesmo tem muito mais vizinhos dentro do alcance, que é precisamente o ponto.
///
/// ⚠️ **Logo `fio/arco` não é o orçamento: ele não é uma propriedade da FERRAMENTA, é do desenho.**
/// O recurso de verdade é o **tempo de rasterização por evento**, contra o kill de 8 ms desta casa —
/// e ele não pode ser medido antes de o rasterizador de fios existir. Este número fica como ponto de
/// partida declarado; quem construir a rasterização **mede e o substitui**, com a tabela ao lado.
///
/// O que JÁ está medido e não muda: a GERAÇÃO dos fios custa **0,56 µs/dab e é constante** de 370 a
/// 50 857 dabs (`measure_the_sketchy_scan`) — o trabalho super-linear foi eliminado pelo
/// [`SKETCHY_SCAN_CAP`], não pela densidade.
pub const SKETCHY_DENSITY_MAX: f32 = 0.04;

/// **Quantos pontos da memória do traço um dab novo consulta.**
///
/// ⚠️ **Não é uma janela de arco, e a distinção é a feature:** o arco entre dois pontos limita a
/// distância entre eles por baixo, então cortar por arco pareceria seguro — mas o Sketchy existe
/// justamente para costurar o traço que VOLTA sobre si mesmo, e ali dois pontos vizinhos no canvas
/// estão a um arco enorme. O teto é de CONSULTAS, o que mantém o custo linear no traço em vez de
/// quadrático sem decidir por geometria nenhuma qual par é legítimo.
///
/// O número é MEDIDO (`measure_the_sketchy_scan`, sonda desta crate).
pub const SKETCHY_SCAN_CAP: usize = 512;
