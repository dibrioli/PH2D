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
    /// Harmony → Krita *Sketch*). Ver [`crate::stroke::threads`].
    Sketchy,
    /// **Wire** — o *Curve brush engine* do Krita (*"a fun tool that you can use for jazzing up your
    /// linework"*): o primo do [`Self::Sketchy`] com memória **LIMITADA**. Em vez de perguntar quem
    /// está perto no CANVAS, ele liga o ponto atual aos que estão perto no PERCURSO — sai o arame /
    /// laço, porque numa curva a corda corta a quina e sobra um laço para fora do traço.
    ///
    /// ⚠️ **O mesmo produtor, o mesmo canal, o mesmo rasterizador** ([`crate::stroke::threads`]) —
    /// o que muda é a pergunta que escolhe os pares. Se esta variante tivesse precisado de geometria
    /// nova, o Sketchy teria sido construído errado.
    Wire,
}

impl LineKind {
    /// Wire `u8` para o round-trip com o painel (`0` = None · `1` = Speed · `2` = Sketchy ·
    /// `3` = Wire).
    ///
    /// ⚠️ O nome do método é o do CANAL (um `u8` que atravessa o painel), e desde 2026-08-14 existe
    /// um tipo chamado `Wire` — a coincidência é infeliz e está nomeada aqui para ninguém a ler como
    /// *"o wire do tipo Wire"*.
    pub fn to_wire(self) -> u8 {
        match self {
            LineKind::None => 0,
            LineKind::Speed => 1,
            LineKind::Sketchy => 2,
            LineKind::Wire => 3,
        }
    }

    /// Inversa de [`Self::to_wire`]; qualquer valor desconhecido cai no neutro.
    pub fn from_wire(w: u8) -> Self {
        match w {
            1 => LineKind::Speed,
            2 => LineKind::Sketchy,
            3 => LineKind::Wire,
            _ => LineKind::None,
        }
    }

    /// **Este tipo COSTURA fios?** — a porta única que o motor pergunta para decidir se guarda a
    /// memória do traço, e o depósito para decidir se drena o canal.
    ///
    /// ⚠️ Ela existe porque a família tem DOIS membros e vai ter mais: enumerar `Sketchy | Wire` nos
    /// sítios de despacho é exatamente a lista que apodrece quando entra o terceiro — a cicatriz que
    /// o `PaintMode::smears()` deste módulo já pagou (o Composite Brush foi o terceiro, e a sessão de
    /// smear dele nunca era encerrada).
    #[must_use]
    pub fn sews_threads(self) -> bool {
        matches!(self, LineKind::Sketchy | LineKind::Wire)
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
pub const THREAD_WIDTH_MAX_PX: f32 = 4.0;

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

/// **Teto do `History` do Wire, em DIÂMETROS de ARCO — MEDIDO, com a tabela ao lado.**
///
/// ⚠️ **A unidade é ARCO, e a escolha é a lei desta casa, não gosto.** O Krita mede a história em
/// PONTOS (*"History size determines the distance for the formation of curve lines"*, default 40~60)
/// e num motor de dabs isso seria uma janela cujo tamanho depende do **Spacing**: apertar o
/// espaçamento encurtaria o arame sem ninguém ter tocado no slider. É a doença que este módulo já
/// curou quatro vezes no relevo — *a lei é função do CAMINHO, nunca de quão fino o motor amostrou o
/// caminho* —, e a cura é medir a janela no percurso.
///
/// E o **DIÂMETRO** é a mesma unidade do [`SKETCHY_REACH_MAX`], pelo mesmo motivo: um pincel grande
/// desenha o mesmo arame, maior. Um número em pixels teria de ser re-escolhido por tamanho.
///
/// **O recurso é o tempo de rasterização por evento, contra o kill de 8 ms desta casa.** Medido pela
/// porta `on_canvas_pointer` (`measure_the_wire_worst_case_on_a_straight_stroke`), pincel r=24,
/// 2048², com a **espessura NO TETO** (o outro fator do mesmo orçamento):
///
/// | history | ms/evento | pior ms |
/// |---:|---:|---:|
/// | 1 | 0,070 | 0,197 |
/// | 3 | 0,113 | 0,171 |
/// | 6 | 0,228 | 0,448 |
/// | 12 | 0,609 | 0,920 |
/// | **24** | **1,709** | **3,914** |
/// | 48 | 2,965 | **12,862** ⇠ estoura |
///
/// ⚠️ **A fixture é um traço RETO, e a escolha dela é o que torna o número honesto.** A espiral que
/// mede o Sketchy **SUB-MEDE** o Wire: nela o traço enrola sobre si mesmo, então uma corda de
/// 1 152 px *de arco* liga dois pontos a ~200 px um do outro no canvas — e o que custa a rasterizar
/// é o COMPRIMENTO da corda, não o arco que ela pula. Medido na espiral, `history 24` custava 1,038
/// no pior evento; medido na reta, **3,914**. Um teto tirado da espiral seria um teto que o produto
/// dobra assim que o artista desenha uma linha.
///
/// ⚠️ **E o achado que este teto carrega: o custo NÃO era o que apertava.** A primeira versão desta
/// wave escreveu `6.0` por raciocínio (*"o meio da pista do Krita"*), e a medição diz que a 6 sobra
/// **17× de margem** contra o kill. O §0 é explícito — *escreva o número que a MEDIÇÃO deu* —, então
/// o teto é 24 e não uma opinião sobre onde o laço fica bonito. Onde ele fica bonito é o DEFAULT, que
/// é outra pergunta e a decide o smoke.
pub const WIRE_HISTORY_MAX: f32 = 24.0;

/// **Quantas cordas um dab novo desenha para trás.**
///
/// ⚠️ **É uma CONTAGEM FIXA amostrada por ARCO, e é por isso que o Wire não tem slider de densidade
/// — nem aqui nem no Krita.** *"Ligue o ponto atual aos últimos N"* lido ao pé da letra são `N` fios
/// por dab, e `N` é a contagem de dabs na janela: a um espaçamento fino um arame de 24 diâmetros
/// pediria centenas de fios por movimento do dedo, e o custo seria função do Spacing outra vez.
/// Amostrando `WIRE_CURVES_PER_DAB` posições **igualmente espaçadas em arco** dentro da janela, a
/// contagem é constante, o desenho é o mesmo em qualquer espaçamento, e o orçamento é conhecido
/// antes de o artista tocar num slider.
///
/// ⚠️ **Este número e o [`WIRE_HISTORY_MAX`] são UM par, não dois números** — o custo é linear em
/// `contagem × janela × espessura`, então dobrar a contagem para 8 **metade** o teto da janela (o
/// pior evento a `history 24` iria de 3,9 para ~7,8 ms, encostando no kill). Quem mexer num tem de
/// re-medir o outro; a tabela do teto foi levantada com a contagem em 4.
pub const WIRE_CURVES_PER_DAB: usize = 4;
