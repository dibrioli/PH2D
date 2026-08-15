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
    /// **Ribbon** — o *Ribbon Shapes* do Alchemy (*"Leaves a trail of ribbon like shapes"*) e o
    /// *Dyna* do Krita (*massa e arrasto*): a tinta é uma **massa presa ao cursor por uma mola**, com
    /// atrito e peso. O traço **PESA** — atrasa na curva, chicoteia na saída e pende sob a gravidade.
    ///
    /// ⚠️ **É o oposto exato do [`Self::Speed`]**, e por isso os dois são o mesmo dropdown: um
    /// arremessa a tinta ADIANTE do dedo, o outro a deixa ATRÁS. Pedir os dois ao mesmo tempo é
    /// pedir duas leis contrárias sobre o mesmo ponto.
    ///
    /// ⚠️ **Ele move o CAMINHO, não a tinta — e essa é a diferença que decide tudo.** O `Speed`
    /// desloca onde o dab cai e deixa o percurso intacto (senão a velocidade se realimentaria); a
    /// fita é um **filtro passa-baixa do percurso**, exatamente como o estabilizador, e realimentar
    /// um passa-baixa é estável por construção. É por isso que o espaçamento, os fios, o preenchedor
    /// de vão, a Symmetry e o Spray saem todos de graça: para eles a fita **é** o traço.
    Ribbon,
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
            LineKind::Ribbon => 4,
        }
    }

    /// Inversa de [`Self::to_wire`]; qualquer valor desconhecido cai no neutro.
    pub fn from_wire(w: u8) -> Self {
        match w {
            1 => LineKind::Speed,
            2 => LineKind::Sketchy,
            3 => LineKind::Wire,
            4 => LineKind::Ribbon,
            _ => LineKind::None,
        }
    }

    /// **Este tipo ARREMESSA a tinta?** — a porta única que o [`crate::stroke::Stroke::throw`]
    /// pergunta antes de deslocar o ponto onde o dab cai.
    ///
    /// ⚠️ **Ela nasceu de um defeito MEDIDO, e o mecanismo vale mais que a linha:** a guarda daquele
    /// sítio era `line_kind == None`, o que era **correto enquanto o Speed era o único tipo** — e a
    /// W3 e a W4 herdaram o arremesso em silêncio. Medido no mesmo traço reto, com o tique do produto
    /// entre os eventos (a mão foi até `x = 900`): `None` deixa a tinta em **899,2** e `Speed`,
    /// `Sketchy` **e** `Wire` a jogam em **1 139,2** — escolher o hachurado dava *Speed Shapes* de
    /// brinde, 240 px além do dedo, e dobrava a contagem de dabs (334 → 688).
    ///
    /// ⚠️ **É a `enumeração apodrece` no eixo oposto ao que a [`crate::BrushSpec::sews_threads`]
    /// cobre:** aquela é sobre o dia em que entra um costurador novo; esta foi o dia em que entrou o
    /// segundo tipo que NÃO arremessa. Uma guarda escrita como *"todo mundo menos o neutro"* é uma
    /// lista de um item só, e ela apodreceu na primeira wave seguinte.
    ///
    /// ⚠️ **E ele fica no ENUM porque a resposta não depende de parâmetro nenhum** — arremessar é
    /// propriedade do tipo. A pergunta irmã (*este pincel costura?*) depende do knob daquele tipo, e
    /// por isso mora no `BrushSpec`; ter as duas no enum foi o que deixou uma delas apodrecer.
    ///
    /// ⚠️ **O que NÃO é gateado aqui é a MEDIÇÃO da velocidade** ([`crate::stroke::Stroke::speed_px_s`]):
    /// ela é do GESTO, não de um tipo — o Sketchy a quer para o *distance-opacity* e um Splatter
    /// futuro para a direção do respingo. Um lugar computa, todos leem; o que o tipo decide é se a
    /// tinta é **deslocada** por ela.
    #[must_use]
    pub fn throws(self) -> bool {
        matches!(self, LineKind::Speed)
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

/// **O ATRASO da fita no peso máximo, em SEGUNDOS.**
///
/// ⚠️ **É um TEMPO, e a unidade é a lei** — a mesma escolha do [`SPEED_LOOKAHEAD_S`], pelo mesmo
/// motivo e no sentido oposto: um atraso em tempo AUTO-ESCALA, enquanto um atraso em pixels teria de
/// ser re-escolhido por tamanho de tela, por zoom e por temperamento de mão.
///
/// **E a lei está MEDIDA** (`measure_the_ribbon_lag_against_speed`, peso 0,45 ⇒ `τ = 0,1125 s`):
///
/// | gesto | atraso | `atraso / (v·τ)` |
/// |---:|---:|---:|
/// | 300 px/s | 43,0 px | 1,274 |
/// | 600 | 86,0 | 1,274 |
/// | 1 200 | 172,0 | 1,274 |
/// | 2 400 | 341,6 | 1,265 |
/// | 4 800 | 683,2 | 1,265 |
///
/// **A razão é CONSTANTE sobre dezasseis vezes a velocidade** ⇒ o atraso é `v · τ` (a constante 1,27
/// é o regime permanente de um segundo-ordem a perseguir uma rampa, `2ζ`). *É esta tabela que
/// separa a fita do estabilizador*, cujo atraso não responde à velocidade.
///
/// ⚠️ **E o teto NÃO é de recurso — é de LOOK, e a medição é que o diz.** O orçamento por evento é
/// PLANO no peso (`measure_the_ribbon_budget_per_event`, tabela no [`RIBBON_TAIL_MAX_S`]): nenhuma
/// combinação encosta no kill de 8 ms, e a fita chega a custar MENOS que o traço comum. O que um
/// quarto de segundo escolhe é *quão longe a tinta pode ficar do dedo* — a 2 400 px/s são ~760 px —,
/// e movê-lo é UMA linha. O que não se pode é escrevê-lo sem dizer de que ele é.
pub const RIBBON_LAG_MAX_S: f32 = 0.25;

/// **O PISO do amortecimento (`ζ`) — MEDIDO, e o recurso é TINTA, não tempo de quadro.**
///
/// ⚠️ **`ζ = 0` é o oscilador PERPÉTUO**, e num pincel isso não é teoria: enquanto o botão está
/// preso o tique percorre o caminho todo quadro, então uma fita que nunca para de balançar **pinta
/// para sempre com a mão parada**. Medido por `measure_the_ribbon_settling_ink` — peso máximo, a mão
/// corre 400 px e PARA, e a sonda conta os dabs dos 30 s seguintes e o instante do último:
///
/// | `ζ` | dabs com a mão parada | silêncio |
/// |---:|---:|---:|
/// | **0,00** | **11 840** | **nunca** (a janela de 30 s acabou) |
/// | 0,04 | 2 402 | 29,97 s |
/// | 0,10 | 948 | 19,72 s |
/// | **0,15** | **622** | **10,98 s** |
/// | 0,24 | 378 | 6,57 |
/// | 0,43 | 214 | 5,78 |
/// | 0,71 | 143 | 1,70 |
/// | 2,00 | 145 | 5,10 |
///
/// **`0,15` é o menor amortecimento cujo silêncio chega em tempo de GESTO** — dezanove vezes menos
/// tinta que o oscilador perpétuo, e ainda bem sub-amortecido, que é onde vive o chicote. ⚠️ Subir
/// mais o piso **removeria a feature**: um `ζ` alto não ultrapassa, e ultrapassar é o que uma fita
/// faz.
///
/// ⚠️ A cauda de ~145 dabs que sobra no fundo da tabela **não é oscilação, é CHEGADA** — a fita
/// estava 400 px atrás e tem de percorrer isso. Ela é o piso irredutível, não desperdício.
///
/// ## ⚠️ O RECURSO QUE ESTE PISO LIMITAVA DEIXOU DE EXISTIR — e o número fica pelo LOOK
///
/// A tabela acima mede *"a mão corre 400 px e PARA"*, e desde **sem gesto, sem tempo**
/// ([`crate::stroke::Stroke::tick_ribbon`]) uma mão parada não entrega tempo à física: a linha
/// `ζ = 0` mede hoje **zero dabs e silêncio imediato**, e o mesmo vale para as outras sete. *Pintar
/// para sempre com a mão parada* passou a ser **estruturalmente impossível**, não *caro*.
///
/// **O piso FICA, e a justificação dele mudou de eixo:** ele já não bounda tinta, ele bounda o
/// **LOOK** — `ζ = 0` faz a fita oscilar em torno do caminho *enquanto o artista desenha*, e a
/// cauda do pen-up (que corre com a coleira cortada) ficaria a balançar até o
/// [`RIBBON_TAIL_MAX_S`] a cortar. **A tabela continua válida como o que mediu**, e é por isso que
/// não foi apagada: ela é a razão de `0,15` e não de `0,04`, e quem a reler tem de saber que o
/// número que ela conta hoje é outro. *Quem move o número que tornava algo caro tem de reconferir a
/// nota.*
pub const RIBBON_DAMPING_MIN: f32 = 0.15;

/// **O TETO do amortecimento (`ζ`).**
///
/// `ζ < 1` é sub-amortecido (a fita ultrapassa e volta — o chicote), `ζ = 1` é crítico (chega e
/// para) e `ζ > 1` é super-amortecido (chega devagar, sem nunca ultrapassar). Dois é onde a fita
/// deixa de ser uma fita e passa a ser um estabilizador lento — e um estabilizador já existe, com
/// slider próprio.
///
/// O `Friction` do card percorre a faixa inteira: `ζ = MIN + friction × (MAX − MIN)`.
pub const RIBBON_DAMPING_MAX: f32 = 2.0;

/// **A GRAVIDADE no máximo, em px/s².**
///
/// ⚠️ **O que ela produz é um PENDURAR, e ele vale `g · τ²`** — a posição de equilíbrio da mola sob
/// peso. A fórmula não é derivada de memória: `measure_the_ribbon_hang` põe a mão parada 3 s e mede
/// a queda contra a previsão, e as duas coincidem **ao dígito** onde o espaçamento de dab as deixa
/// coincidir:
///
/// | peso | gravity | queda medida | `g·τ²` |
/// |---:|---:|---:|---:|
/// | 0,45 | 0,50 | 12,0 px | 12,7 |
/// | 0,45 | 1,00 | 24,0 | 25,3 |
/// | 1,00 | 0,25 | **31,2** | **31,2** |
/// | 1,00 | 0,50 | **62,4** | **62,5** |
/// | 1,00 | 1,00 | **124,8** | **125,0** |
///
/// (A folga nas linhas de peso baixo é a QUANTIZAÇÃO do espaçamento — 2,4 px por dab —, não erro do
/// modelo: a fita pende onde pende, e a sonda só a vê onde um dab caiu.)
///
/// ⚠️ **O teto é de LOOK**, como o do atraso: 125 px de queda no canto máximo é uma fita que pende
/// bem, e o orçamento por evento não se move com ele.
///
/// ⚠️ **E isso ACOPLA a gravidade ao peso, o que é FÍSICA e está nomeado em vez de escondido:** uma
/// fita mais pesada pende mais sob a mesma gravidade. Não é a falha de ergonomia que o Conserve
/// pagou (*"o valor CERTO do knob é função de outro knob"*) — aqui não há valor certo, há duas
/// propriedades do mundo, e o artista arrasta até pender como ele quer.
pub const RIBBON_GRAVITY_MAX_PX_S2: f32 = 2000.0;

/// **A fração do atraso que um sub-passo do integrador pode valer.**
///
/// ⚠️ **A fita é integrada por SUB-PASSOS e não uma vez por quadro, e o motivo é ESTABILIDADE:** o
/// Euler semi-implícito de uma mola só é estável enquanto `ω · h` é pequeno, e `ω = 1/τ` cresce sem
/// limite quando o peso encosta no zero. Com um quarto do atraso por sub-passo, **`ω · h = 0,25`
/// sempre** — em qualquer peso e em qualquer taxa de quadros.
///
/// ⚠️ **E o "sempre" é literal: nada tem permissão de o violar.** Ver [`RIBBON_MAX_STEP_S`], que é
/// a razão de o teto de sub-passos ter deixado de ser um `clamp`.
pub const RIBBON_SUBSTEP_FRACTION: f32 = 0.25;

/// **O PISO do atraso, em segundos — MEDIDO.**
///
/// ⚠️ **Ele existe porque `ω = 1/τ` é ILIMITADO**: um peso de `1e-9` pede uma mola infinitamente
/// rígida, e nenhum número de sub-passos a integra. **Ele é um piso de ESTABILIDADE**, e o que o
/// fixa neste valor é o orçamento de sub-passos do outro lado ([`RIBBON_MAX_SUBSTEPS`]) — os dois
/// números são um par, e escolher um sem olhar o outro foi como o teto virou um clamp.
///
/// ⚠️ **E a 1ª versão deste doc dizia que ele é o piso de VISIBILIDADE — a medição o desmentiu.**
/// `measure_the_ribbon_floor_of_visibility` mede o deslocamento da tinta contra o traço neutro e a
/// travessia de *um dab* acontece **entre `τ = 0,005` e `τ = 0,010`**, não aqui: em `τ = 0,005` a
/// fita já desloca **0,00 px**. A afirmação *"abaixo dele a tinta desloca-se menos que um dab"*
/// continua **verdadeira** neste valor — o que era falso é o *"é ONDE isso passa a valer"*, e a
/// consequência de produto fica nomeada: **o fundo do slider é inerte** por uma faixa, o que é o
/// comportamento que se quer de um controle cujo mínimo significa *desligado*.
///
/// ⚠️ **O piso NÃO foi subido para os 0,005 medidos, de propósito:** aquele número é de UMA
/// velocidade de gesto (2 400 px/s) e de UM espaçamento (2,4 px) — um gesto mais rápido ou um
/// pincel maior o movem. Mover um limite de estabilidade por uma medição de aparência de um único
/// ponto de operação seria trocar a razão do número pela mais frágil das duas.
///
/// **A tabela medida está no doc de [`RIBBON_MAX_SUBSTEPS`]**, com o `n` que cada linha custa.
pub const RIBBON_LAG_MIN_S: f32 = 0.002;

/// **O TETO DE TRABALHO de um passo, em segundos** — quanto de relógio um tique pode consumir.
///
/// ⚠️ **Ele substitui um `clamp(1, 32)` no número de SUB-PASSOS, e a diferença entre os dois é a
/// wave inteira.** O clamp limitava a **RESOLUÇÃO**: com `n` capado, `h = dt/n` parava de encolher,
/// `ω · h` saía dos 0,25 que a [`RIBBON_SUBSTEP_FRACTION`] promete, e a mola **DIVERGIA**. Medido
/// analiticamente no extremo que o gate escolhe (peso 0,02 ⇒ `τ = 5 ms`, `dt = 200 ms`): o `n`
/// necessário é **160** e o clamp aplicava **32**, levando `ω · h` de 0,25 a **1,25** e o maior
/// autovalor da matriz de amplificação a **1,7586 > 1** — `1,7586^320 ≈ 1e78` em dez quadros.
///
/// ⚠️ **E o custo disso não foi um desenho torto: foi a MÁQUINA.** Com a ponta em `1e78` o
/// percurso emite um dab a cada `spacing` até lá, e o processo morreu com **90,2 GB de RSS**,
/// derrubando a janela do editor junto (achado externo, 2026-08-14).
///
/// ⚠️ **O doc do teto antigo chamava-o de *"o irmão do `MAX_AIRBRUSH_DABS_PER_TICK`"*, e era o
/// OPOSTO dele:** aquele **descarta o backlog** — limita o TRABALHO e deixa o passo intacto. Esta
/// const faz o mesmo, e é a lei do [`ph2d_core::time::FixedStep`], que cappa quantos tiques rodar e
/// **joga fora o tempo excedente** (*"roda em câmara lenta"*, o trade que o comentário dele nomeia).
/// **Uma limita o TRABALHO, a outra limita a RESOLUÇÃO — e só a primeira é segura.**
///
/// Quatro quadros de 60 fps: uma engasgada perde tempo de física (a fita atrasa um pouco mais do que
/// atrasaria), nunca precisão.
pub const RIBBON_MAX_STEP_S: f32 = 4.0 / 60.0;

/// **Quantos sub-passos um tique pode precisar — DERIVADO, não escolhido.**
///
/// `ceil(RIBBON_MAX_STEP_S / (RIBBON_SUBSTEP_FRACTION · RIBBON_LAG_MIN_S))`, e o gate
/// `the_substep_ceiling_can_never_bind` afirma exactamente essa aritmética. ⚠️ **Ele é um
/// BATENTE, não um regulador:** se ele algum dia morder, a `ω · h = 0,25` deixou de valer e a mola
/// diverge — então o gate existe para que mover qualquer uma das outras duas consts fique VERMELHO
/// em vez de sair a pintar `1e78`.
///
/// **O que a fita desenha em cada `τ`** — `measure_the_ribbon_floor_of_visibility`, gesto reto de
/// 2 400 px/s por 2 s, pincel r=12 com espaçamento de 2,40 px, atrito 0,30 (`ζ = 0,705`). O
/// deslocamento é medido **contra o traço neutro**, que termina 0,80 px atrás do dedo:
///
/// | peso | τ | deslocamento | em dabs | `2ζvτ` (lei contínua) | `n` no `dt` CAPADO |
/// |---:|---:|---:|---:|---:|---:|
/// | 1,000 | 0,2500 s | 804,00 px | 335 | 846,00 | 2 |
/// | 0,400 | 0,1000 | 300,00 | 125 | 338,40 | 3 |
/// | 0,160 | 0,0400 | 105,60 | 44 | 135,36 | 7 |
/// | 0,080 | 0,0200 | 43,20 | 18 | 67,68 | 14 |
/// | 0,040 | 0,0100 | 12,00 | 5 | 33,84 | 27 |
/// | 0,020 | 0,0050 | **0,00** | **0** | 16,92 | 54 |
/// | 0,008 | **0,0020** | **0,00** | **0** | 6,77 | **134** |
///
/// ⚠️ **A lei contínua `L = 2ζvτ` é um TETO, não uma previsão** — a razão medida/lei cai
/// monotonicamente (0,95 · 0,89 · 0,78 · 0,64 · 0,35 · 0) porque o alvo que a fita persegue é uma
/// **ESCADA**: o dedo salta 40 px e fica parado o resto do quadro. Quando `τ` encolhe a ponto de a
/// mola convergir dentro de um quadro, o atraso **colapsa a zero** — e é isso, não a aritmética da
/// lei, que decide onde a feature deixa de desenhar.
///
/// ⚠️ **A escada é do RELÓGIO, nunca do mouse:** a sonda mede a 60 Hz e a 960 Hz de ponteiro e dá
/// **os mesmos números**, porque `step_ribbon` lê o `last_raw_pos` **uma vez por tique** e as
/// amostras intermediárias não a alcançam. É o gate `the_ribbon_is_a_fact_of_the_clock…` visto por
/// outro ângulo — e, por ser identidade por construção, aquela coluna **não pode falhar**: ela é
/// controle, não experimento.
///
/// **`0,002 s` fica DENTRO da zona onde a fita já não move a tinta um dab inteiro** — a travessia
/// medida é entre `τ = 0,005` e `τ = 0,010`, então o piso não é a fronteira: é um valor confortável
/// abaixo dela, escolhido pelo orçamento de sub-passos. A zona morta que isso deixa na pista vai do
/// zero até **~2 %** do slider (peso 0,02), e é o comportamento certo para um controle cujo mínimo
/// deve significar *desligado*.
///
/// ⚠️ **A coluna de `n` é a do `dt` CAPADO, e eu escrevi a const contra a errada.** A 1ª versão
/// deste teto dizia **34** — o `n` de um quadro NOMINAL — e o que ele tem de cobrir é o `dt` que o
/// [`RIBBON_MAX_STEP_S`] deixa passar, que são **quatro** quadros. O batente MORDIA no piso do
/// slider (134 precisos contra 34 aplicados), `ω · h` ia de 0,25 a **0,98**, e com o atrito no topo
/// da pista o maior autovalor voltava a **3,68 > 1**: a divergência reinstalada dentro do commit
/// que a curava. É por isso que o gate afirma a ARITMÉTICA das três consts em vez de comparar o
/// literal com um número escrito à mão — um número à mão erra junto com quem o escreveu.
pub const RIBBON_MAX_SUBSTEPS: usize = 134;

/// **Quanto tempo a CAUDA da fita tem para chegar, em segundos.**
///
/// No pen-up a mão soltou mas a fita ainda tem inércia: a cauda é a física a correr até ela assentar.
/// O teto existe porque `ζ` pequeno faz uma fita balançar por muito tempo, e um traço não pode
/// continuar a crescer depois de o artista o ter terminado.
///
/// **O ORÇAMENTO DA FITA INTEIRA, medido pela porta do PRODUTO** (`measure_the_ribbon_budget_per_event`,
/// traço reto de 2 s a 2 400 px/s, 2048², com a linha `None` como CONTROLE):
///
/// | raio | tipo | pior Move | pior tique | pen-up |
/// |---:|---|---:|---:|---:|
/// | 24 | *None (controle)* | **1,294** | 0,001 | 0,141 |
/// | 24 | Ribbon 1,00 / 0,05 | 0,001 | 1,080 | 0,152 |
/// | 100 | *None (controle)* | **1,183** | 0,001 | 0,453 |
/// | 100 | Ribbon 1,00 / 0,30 | 0,000 | 1,092 | 0,214 |
/// | 200 | *None (controle)* | **1,018** | 0,002 | 0,553 |
/// | 200 | Ribbon 1,00 / 1,00 | 0,000 | 0,919 | **0,390** |
///
/// ⚠️ **Duas leituras, e as duas são achados:** (1) o custo **MIGRA do `Move` para o TIQUE**, que é
/// exactamente o desenho (o `extend` não percorre, o tique percorre) — uma sonda que medisse só o
/// movimento diria que a fita é de graça; (2) **o pen-up fica MAIS BARATO com a fita** (0,553 →
/// 0,390 no pincel grande), porque ela atrasa, logo percorre menos caminho no mesmo tempo.
///
/// ⚠️ **Nenhuma combinação encosta no kill de 8 ms** ⇒ **não há teto de recurso a derivar aqui**, e
/// dizer isso é a metade honesta do §0: os três knobs da fita são limitados por LOOK, com a régua de
/// cada um escrita no doc dele.
pub const RIBBON_TAIL_MAX_S: f32 = 0.6;

/// **A velocidade abaixo da qual a cauda considera a fita ASSENTADA, em px/s.**
///
/// Meio pixel por quadro a 60 fps — o mesmo limiar sub-pixel que o `SETTLE_EPS_PX` do estabilizador
/// usa, dito em velocidade porque o que decide aqui é se ainda há movimento a pintar.
pub const RIBBON_TAIL_REST_PX_S: f32 = 30.0;

/// **Quanto o dedo tem de andar entre dois tiques para a fita ganhar TEMPO, em pixels.**
///
/// ⚠️ **É o piso de *sem gesto, sem tempo*** ([`crate::stroke::Stroke::tick_ribbon`]): abaixo dele o
/// tique não integra e a fita congela, porque uma mola que converge para um alvo parado desenha uma
/// LINHA RETA de largura cheia — a espícula que o smoke reprovou duas vezes.
///
/// **O número é DERIVADO, não escolhido:** meio pixel é o mesmo limiar sub-pixel com que o
/// estabilizador declara ter alcançado o cursor (`SETTLE_EPS_PX`), e a razão é a mesma nos dois —
/// **um deslocamento menor que meio pixel não move um dab**, então tratá-lo como movimento é
/// entregar tempo de física a um gesto que não existe.
pub const RIBBON_PARK_EPS_PX: f32 = 0.5;

/// **O espaçamento das TRAVESSAS da faixa no `Rungs = 0⁺`, em DIÂMETROS do pincel** — a ponta
/// esparsa da pista.
///
/// A unidade é o diâmetro pela MESMA razão da [`SKETCHY_REACH_MAX`] e da [`WIRE_HISTORY_MAX`]: é o
/// que torna a lei livre de escala, e um pincel do dobro do tamanho desenha a mesma faixa maior em
/// vez de outra faixa.
pub const RIBBON_RUNG_SPARSE_D: f32 = 2.0;

/// **O espaçamento das travessas no `Rungs = 1`, em DIÂMETROS** — a ponta densa.
///
/// ⚠️ **O teto de densidade NÃO é de RECURSO, e a medição é que o diz:** a faixa custa `2` fios por
/// travessa (a travessa e o segmento do trilho longe), e no ponto mais denso um traço de 312 px com
/// pincel de raio 10 emite **~125 fios** — contra os **16 mil px de fio** que a densidade cheia do
/// Sketchy põe no mesmo traço. O que limita é o LOOK: passado o [`RIBBON_RUNG_DUTY`] as travessas
/// encostam umas nas outras e a faixa deixa de ler como travessas para ler como um bloco chapado.
pub const RIBBON_RUNG_DENSE_D: f32 = 0.25;

/// **Quantas larguras-de-fio um vão entre travessas tem de ter, no mínimo.**
///
/// ⚠️ **É um piso de APARÊNCIA com mecanismo, não um número de gosto:** uma travessa tem a
/// espessura do fio (`BrushSpec::thread_width_px`), então um espaçamento igual à espessura é uma
/// duty cycle de 100% — tinta contínua, e a faixa **preenche SÓLIDO**. Dois é a duty cycle de 50%,
/// que é onde a travessa e o vão medem o mesmo e a alternância ainda é legível.
///
/// ⚠️ **Ele é lido em PIXELS e não em diâmetros, e a razão é o pincel PEQUENO:** a lei em diâmetros
/// escala com o pincel e a espessura do fio **não** — num pincel de raio 2 o passo denso vale
/// `0,25 × 4 = 1 px`, exactamente a espessura default de um fio, e a faixa chaparia. O piso é o que
/// mantém a mesma escolha legível nos dois extremos de tamanho.
pub const RIBBON_RUNG_DUTY: f32 = 2.0;
