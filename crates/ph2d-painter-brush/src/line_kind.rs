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

/// **Teto da `Density` — MEDIDO pela porta do produto, com a tabela ao lado.**
///
/// ⚠️ **Ele era `0,04`, e a medição o derrubou por DEZ vezes.** O número antigo saía de um proxy —
/// `fio/arco`, o comprimento de fio contra o arco do traço — e o doc dele já dizia que o recurso de
/// verdade é outro: *o tempo de rasterização por evento, contra o kill de 8 ms desta casa*, que não
/// podia ser medido antes de o rasterizador existir. Ele existe (a W3 fechou), e o número foi
/// medido: `measure_the_sketchy_budget_per_event`, espiral de 8 voltas, pincel r=24, 2048²,
/// **pela porta `on_canvas_pointer`**.
///
/// | density | reach | width | ms/evento | pior ms |
/// |---:|---:|---:|---:|---:|
/// | 0,00 | 1 | 1 | 0,077 | 0,309 |
/// | 1,00 | 1 | 1 | 0,226 | **0,433** |
/// | 0,10 | 4 | 4 | 0,742 | 2,086 |
/// | 0,30 | 4 | 4 | 1,923 | 5,361 |
/// | **0,40** | **4** | **4** | **2,508** | **6,384** |
/// | 0,50 | 4 | 4 | 3,156 | **8,040** ⇠ estoura |
/// | 1,00 | 4 | 4 | 6,029 | **16,181** |
///
/// **`0,40` é a maior densidade cujo PIOR evento cabe no kill de 8 ms com o alcance E a espessura
/// nos tetos deles** — o canto mais caro que os três sliders alcançam juntos.
///
/// ⚠️ **E o preço deste número ser UM é honesto e está nomeado:** no alcance de 1 diâmetro a mesma
/// densidade custa **0,15 ms** e o slider poderia ir a 1,0 sem encostar no kill — ou seja, o canto
/// caro cobra 20× de margem ao canto barato. É o custo de um escalar governar um PRODUTO de dois
/// (`density × reach²`); o redesenho que o removeria é a densidade passar a significar *"que fração
/// do que o alcance permite"*, e ele **não foi construído** porque muda a lei que os gates do motor
/// pinam e o LOOK que o smoke ainda vai julgar. Deixar a combinação estourar não é opção: quem
/// mexeu no `Reach` derrubaria o quadro sem ter tocado neste slider.
///
/// O que JÁ estava medido e não muda: a GERAÇÃO dos fios custa **0,56 µs/dab e é constante** de 370 a
/// 50 857 dabs (`measure_the_sketchy_scan`) — o trabalho super-linear foi eliminado pelo
/// [`SKETCHY_SCAN_CAP`], não pela densidade.
pub const SKETCHY_DENSITY_MAX: f32 = 0.4;

/// **Teto do `Reach`, em DIÂMETROS de pincel.**
///
/// A W0.3 mediu o gasto nas duas pontas: a **um** diâmetro os fios somam ~50× o arco do traço, a
/// **quatro**, ~700×. Quatro é onde a medição parou porque é onde a teia deixa de ser a decoração de
/// um traço e passa a ser um véu sobre o desenho inteiro — e a [`SKETCHY_DENSITY_MAX`] é quem paga a
/// diferença.
pub const SKETCHY_REACH_MAX: f32 = 4.0;

/// **Teto da `Line Width`, em pixels.**
///
/// ⚠️ O custo de rasterizar é linear em `largura × comprimento total de fio`, então este número
/// multiplica o mesmo orçamento que a [`SKETCHY_DENSITY_MAX`] governa — e por isso ele é pequeno: um
/// fio é um traço de LÁPIS ao lado do traço, não um segundo pincel. Acima de ~4 px a teia deixa de
/// ler como hachura e passa a ser uma segunda pincelada por cima da primeira.
pub const SKETCHY_WIDTH_MAX_PX: f32 = 4.0;

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
