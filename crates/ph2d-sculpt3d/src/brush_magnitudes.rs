//! **AS MAGNITUDES** — quanto cada família de verbo desloca, e de onde cada
//! número veio.
//!
//! Cortado por ASSUNTO do [`super`], que virou três coisas num arquivo só: o
//! **catálogo** (que verbos existem), as **portas** (o que cada um lê) e estas
//! constantes. As três crescem por razões diferentes — o catálogo quando entra
//! uma ferramenta, as portas quando entra uma pergunta, e isto quando entra um
//! número —, e foi a terceira que levou o pai ao teto de LOC.
//!
//! ⚠️ **Cada constante daqui diz DE ONDE ELA VEM**, e é a única coisa que este
//! arquivo exige: um `✅` é um literal LIDO da fonte de referência (com o
//! arquivo e a linha), um `⚠️ NOSSO` é um número que a MEDIÇÃO deu, com a tabela
//! ao lado. Um número sem essa etiqueta é um palpite à espera de um smoke.

/// Quanto do RAIO do pincel um dab de força cheia desloca.
///
/// ⚠️ **É fração do raio, nunca uma distância absoluta** — a lição que o impasto
/// do Painter pagou em 2026-07-14: com altura absoluta, um pincel pequeno e um
/// grande picam no mesmo valor, e o grande vira uma poça chata porque a razão
/// *altura ÷ largura* despenca. Amarrando ao raio, a razão de aspecto do domo é
/// constante em toda escala e o falloff lê igual com pincel de 1 mm e de 1 m.
///
/// O NÚMERO é decisão de **smoke**, como o `ORBIT_RAD_PER_PX` da câmera: ele não
/// é teto de recurso nenhum, é o quanto de barro uma pincelada move.
pub const REACH_FRACTION: f32 = 0.1;

/// **Quanto do raio um dab da FAIXA desloca** — e não é o [`REACH_FRACTION`].
///
/// ⚠️ **`clay_strips.cc:327`, verbatim:**
///
/// ```text
/// const float3 offset = plane_normal * ss.cache->bstrength * ss.cache->radius;
/// ```
///
/// O deslocamento é `raio · força`, **fração 1,0** — o `0,1` do
/// [`REACH_FRACTION`] é o `deform = intensidade · raio · 0,1` do `Brush.js`, do
/// SculptGL, que **não tem esta ferramenta**. É a mesma classe do defeito que a
/// §7.21 curou na lei de kernel, uma camada abaixo.
///
/// ⚠️ **MEDIDO, e é o que a foto do Enio mostra:** com `0,1` a faixa era
/// **7,5× mais fraca por dab** que a referência, e por isso deixava estrias
/// macias que ACOMPANHAM a forma em vez das placas chatas que a CORTAM.
///
/// ⚠️ **É por causa dele que o `STRIP_DEPTH_GAIN` morreu:** aquele ganho existia
/// para preservar uma magnitude que era ela própria errada.
pub const STRIP_REACH_FRACTION: f32 = 1.0;

/// O ganho do **Crease**, e o do vinco é MENOR que o do Draw.
///
/// ⚠️ **Os três números desta família saem da referência, não de afinação:**
/// `Brush.js` e `Inflate.js` usam `intensidade · raio · 0,1`, o `Crease.js` usa
/// `intensidade · 0,07` e o `Pinch.js` usa `intensidade · 0,05`. Eles são o que
/// separa *"a lei está certa"* de *"a ferramenta responde como a referência"* —
/// e a sonda `measure_reference_divergence` é quem os cobra.
pub const CREASE_FRACTION: f32 = 0.07;

/// O ganho do **Pinch** e do **Magnify** — ver [`CREASE_FRACTION`].
///
/// ⚠️ **Ele não existia, e a ausência valia 20×:** o alvo era `base + tangente`
/// atenuado pelo peso, ou seja o vértice caminhava até `w` da distância inteira
/// ao centro **num dab**. Medido contra a referência, `16,88×`.
pub const PINCH_GAIN: f32 = 0.05;

/// **Quanto o plano da FAIXA sobe acima da superfície**, em fração do raio.
///
/// ⚠️ **Este número decide se a faixa NIVELA ou COPIA o relevo, e é a coisa
/// inteira que o report do Enio de 2026-08-15 nomeia** (*"num vale a tool
/// correta tende a fechar o vale, na nossa tende a aumentá-lo"*).
///
/// O portão de profundidade da [`crate::Strip`] é `z·(1−z)`, que **sobe** de
/// `z = 0` (o plano) até `z = 0,5` e **desce** depois. Logo o depósito só cresce
/// com a profundidade enquanto o ponto está a menos de meio raio abaixo do
/// plano; passado o pico, quanto mais fundo **menos** barro. Como a superfície
/// em repouso fica a `lift` raios abaixo do plano, a faixa **enche** relevo até
/// `(0,5 − lift)` raios abaixo da média e **exagera** o que passa disso.
///
/// ⚠️ **O valor anterior era `0,5`, e ele punha a superfície EXATAMENTE no pico
/// — folga de enchimento ZERO.** Era o único valor da família em que nenhum vale
/// enche: tudo abaixo da média recebia menos que a média, então a passada
/// devolvia o relevo amplificado. ⚠️ E o defeito era MEU pela mesma via da
/// `tip_roundness`: eu derivei `0,5` de *"pôr o pico na superfície em repouso"*,
/// que é conveniência interna (o máximo depósito em chapa plana), e não da
/// propriedade que decide.
///
/// **MEDIDO** (`tests/measure_valley.rs`, vale de 0,40 de profundidade, pincel
/// `r = 0,8`, nove dabs; a varredura prendeu a magnitude pela FORÇA, para que
/// a única coisa a mover fosse a forma):
///
/// | lift | vale (Δ profundidade) | miolo ÷ aro numa CÚPULA |
/// |---|---|---|
/// | 0,10 | −0,212 | **0,009** (o miolo não recebe nada) |
/// | 0,18 | −0,121 | 0,393 |
/// | **0,25** | **−0,073** | **0,649** |
/// | 0,30 | −0,048 | 0,756 |
/// | 0,50 | **+0,027 (AUMENTA)** | 0,971 |
///
/// ⚠️ **A tabela foi medida com o `reach` do SculptGL, que era ele próprio
/// errado** (ver [`STRIP_REACH_FRACTION`]). Com o `raio · força` da referência o
/// vale FECHA muito mais forte — `0,4000 → 0,0406` em nove dabs a `r = 0,5` —,
/// mas a FORMA da tabela é a que decide o lift e ela não muda: o enchimento
/// segue monótono no lift, e o miolo segue a esvaziar-se quando ele baixa.
///
/// ⚠️ **As duas colunas puxam em sentidos OPOSTOS, e as duas são a mesma lei.**
/// Numa cúpula o miolo da pegada está acima do plano ajustado e o aro abaixo,
/// então nivelar É depositar mais no aro — o *"displaces vertices toward the
/// brush plane"* do kernel da referência. Baixar o lift nivela mais forte e
/// esvazia o miolo; subi-lo faz a banda ficar uniforme e parar de nivelar.
///
/// ⇒ **`0,25` é o MEIO da subida da parábola** (o plano em `z = 0`, o pico em
/// `z = 0,5`), o que dá à ferramenta folga igual para acrescentar num calombo e
/// para encher um buraco. É um marco da própria lei, não um gosto, e as duas
/// medições o confirmam longe de qualquer extremo.
///
/// ⚠️ **NÃO é citável da referência.** O `clay_strips.cc` lê `brush.plane_offset`
/// e o genérico do DNA é `0.0`; quem declarava o valor por-tool era o
/// `BKE_brush_sculpt_reset`, que **não existe mais em C** (§7.0 do plano) — a
/// mesma lacuna que bloqueou a W1 e o Draw Sharp. O número acima é NOSSO, e a
/// tabela ao lado é a razão dele.
///
/// ⚠️ **E ele existe porque sem ele a ferramenta nasce MORTA:** com o plano
/// rente, `z = 0` em toda parte e o portão fecha — quatro varreduras da suíte
/// (o alpha, o invert, os dois do aplicador) reprovaram exatamente assim, cada
/// uma dizendo *"dab inerte"*. O [`Verb::Clay`] resolve o mesmo problema pela
/// mesma via ([`CLAY_PLANE_FRACTION`]), e o `plane_offset` do artista SOMA a
/// este em vez de o substituir.
pub const STRIP_PLANE_FRACTION: f32 = 0.25;

/// Quanto o plano do **Clay** sobe acima da superfície, em fração do raio.
///
/// ⚠️ **É o literal `0.1` do `Brush.js:52`, e ele não é um knob no original.** O
/// nosso `plane_offset` é um controle do artista e SOMA a este — o default dele
/// (`0`) devolve a referência exata, e girá-lo levanta o plano a mais.
pub const CLAY_PLANE_FRACTION: f32 = 0.1;

/// Quanto o plano do **Clay Thumb** se inclina a MAIS a cada dab, em graus.
///
/// ✅ **É o literal `0.8f` do `clay_thumb.cc:173`**, lido da fonte — não é nosso.
///
/// ⚠️ **E é por DAB, o que torna a lei dependente do ESPAÇAMENTO** — a
/// referência declara graus-por-amostra, e quantas amostras cabem num
/// centímetro de traço é decisão de cada motor. ⇒ o que é citável é *quanto o
/// plano gira por dab*; *quanto ele gira por comprimento de traço* é uma
/// grandeza NOSSA, e o gate `the_thumb_tilt_accumulates_with_the_stroke` mede a
/// nossa em vez de a afirmar.
pub const CLAY_THUMB_TILT_STEP_DEG: f32 = 0.8;

/// O TETO da inclinação do **Clay Thumb**, em graus.
///
/// ✅ **É o literal `60.0f` do `clay_thumb.cc:175`** (`std::clamp(front_angle,
/// 0.0f, 60.0f)`).
///
/// ⚠️ **Ele é ALCANÇÁVEL, e é isso que o torna um teto e não um adorno:** a
/// [`CLAY_THUMB_TILT_STEP_DEG`] o atinge em `60 / 0,8 = 75` dabs, que é um traço
/// longo mas comum. Um teto que ninguém encosta seria um número sem
/// consequência; este muda o desenho de todo traço a partir do 75º dab, e o
/// gate afirma a saturação.
pub const CLAY_THUMB_TILT_MAX_DEG: f32 = 60.0;

/// O TETO do ângulo entre os dois planos do **Multiplane Scrape**, em graus.
///
/// ✅ **É o `RNA_def_property_range(prop, 0.0f, 160.0f)` do `rna_brush.cc:3382`**
/// — a faixa que a referência oferece ao artista, lida da fonte.
///
/// ⚠️ **Ele é o teto do KNOB, não um limite de recurso**, e a distinção importa
/// porque `160°` é quase o V totalmente aberto: cada meio-plano se inclina `80°`
/// contra a normal de área, e ali a projeção deixa de raspar para se tornar um
/// aperto lateral. Quem o alcança está a usar a ferramenta no extremo que a
/// referência declara alcançável, não a bater numa parede nossa.
///
/// ⚠️ **E o que acontece lá em cima está MEDIDO** (a sonda
/// `measure_multiplane_scrape`, esfera unitária, pincel `0,35`, traço de 20
/// dabs) — o V **deixa de abrir e passa a fechar**, e o número diz onde:
///
/// | autorado | diedro medido | crista (raios) |
/// |---|---|---|
/// | 0° | 0,00° | **0,0000** (nada é movido) |
/// | 30° | 19,39° | 0,0677 |
/// | **60°** | **46,85°** | **0,1739** |
/// | 90° | 63,01° | 0,2175 |
/// | 120° | 10,25° | 0,0908 |
/// | 160° | — | 0,0084 |
///
/// ⇒ a fidelidade (medido ÷ autorado) tem joelho em **45-60°** e desaba depois
/// dos 90°, onde os meios-planos estão tão deitados que a projeção mal alcança
/// a superfície. O teto **fica onde a referência o pôs**: um número nosso ali
/// seria a nossa opinião a vestir a autoridade dela.
pub const MULTIPLANE_ANGLE_MAX_DEG: f32 = 160.0;

/// Quanto o V do **Multiplane Scrape** abre de fábrica, em graus.
///
/// ⚠️ **É NOSSO, e a §4 do plano é o motivo.** O único número citável é o do
/// `DNA_brush_types.h:407` (`multiplane_scrape_angle = 0`), que é o default do
/// pincel **genérico** — o valor de fábrica DESTA ferramenta vive num `.blend`
/// binário desde o 4.3 (§7.0) e não é legível. Herdar o zero seria deixar o
/// fallback definir o produto (CLAUDE.md §0): com os dois planos coincidentes o
/// verbo É o [`Verb::Scrape`], **ao bit** — uma ferramenta escolhida que não faz
/// nada de diferente da que já existe, que é o defeito que o `Rough` da
/// `line/Vector` já pagou.
///
/// ⚠️ **E o zero não é *"um V estreito"*, é a ferramenta DESLIGADA** — medido,
/// ele move **zero vértices** (ver [`crate::Brush::scrape_angle_deg`] para o
/// mecanismo: os dois meios-planos passam pelo plano TANGENTE).
///
/// ⚠️ **O número é MEDIDO** (`tests/measure_multiplane_scrape.rs`), pela
/// propriedade que separa esta ferramenta do Scrape — o sulco tem **duas facetas
/// com uma aresta entre elas**, e não um fundo chato:
///
/// | autorado | diedro medido | fidelidade | crista (raios) |
/// |---|---|---|---|
/// | 15° | 4,65° | 0,31 | 0,0076 |
/// | 30° | 19,39° | 0,65 | 0,0677 |
/// | 45° | 34,18° | 0,76 | 0,1182 |
/// | **60°** | **46,85°** | **0,78** | **0,1739** |
/// | 90° | 63,01° | 0,70 | 0,2175 |
///
/// `60°` é onde a **fidelidade** tem o pico e a **crista** já vale 17% do raio
/// do pincel — visível numa olhada, contra os 0,8% de `15°`. Acima dele a crista
/// ainda cresce e a faceta começa a mentir (ver [`MULTIPLANE_ANGLE_MAX_DEG`]).
///
/// ⚠️ **O gate que o vigia NÃO menciona este literal** — ele pede que o pincel
/// de fábrica deixe crista e que zerar o knob a apague; *um default só é testado
/// por um teste que não o nomeia*.
pub const DEFAULT_MULTIPLANE_ANGLE_DEG: f32 = 60.0;

/// Quanto a ponta do **Multiplane Scrape** é ESPREMIDA ao longo do traço.
///
/// ✅ **É o `local[1] *= 2.0f` do `multiplane_scrape.cc:103`**, e o comentário
/// ao lado dele diz para que serve: *"deform the local space along the Y axis to
/// avoid artifacts on curved strokes. This produces a not round brush tip."*
///
/// ⚠️ **O eixo `Y` da moldura DELES corre ao longo do caminho** (`y = n × x` com
/// `x = n × grab_delta`), então o número aperta a lâmina na direção em que ela
/// avança — a pegada vira uma elipse `raio × raio/2`, com o lado CURTO no
/// sentido da marcha. Herdar a letra `Y` sem herdar o eixo seria estreitar a
/// lâmina no sentido errado, e o sulco sairia largo e curto em vez de fino e
/// longo.
pub const MULTIPLANE_TIP_STRETCH: f32 = 2.0;

/// Quanto do ângulo ANTERIOR sobrevive a cada amostra, no modo dinâmico.
///
/// ✅ **É o `0.2f` do `multiplane_scrape.cc:651`** — `math::interpolate(novo,
/// anterior, 0.2)`, ou seja `0,8 · novo + 0,2 · anterior`. O comentário da
/// referência dá a razão: *"interpolate between the previous and new sampled
/// angles to avoid artifacts when if angle difference between two samples is too
/// big"*.
///
/// ⚠️ **É a única memória do modo dinâmico**, e sem ela a ferramenta pisca: o
/// ângulo amostrado salta quando a lâmina cruza uma quina, e um V que muda de
/// abertura entre dois dabs deixa um degrau no sulco.
pub const MULTIPLANE_ANGLE_SMOOTH: f32 = 0.2;

/// **Onde o slider da DEMÃO para** — a faixa de UI que o `rna_brush.cc:3235`
/// declara (`RNA_def_property_ui_range(prop, 0, 0.2f, 1, 3)`).
///
/// ⚠️ **É a faixa da referência, não um teto de recurso** — uma demão mais alta
/// não custa nada e não degenera nada; o que ela deixa de ser é uma demão. E é
/// por isso que ela é acompanhada do [`LAYER_HEIGHT_HARD_MAX`], que é onde a
/// mesma fonte diz que o número deixa de fazer sentido.
pub const LAYER_HEIGHT_UI_MAX: f32 = 0.2;

/// **O maior valor que a caixa de texto da demão aceita** — a faixa DURA do
/// `rna_brush.cc:3234` (`RNA_def_property_range(prop, 0, 1.0f)`).
///
/// ⚠️ **Na nossa escala isso é o RAIO INTEIRO da esfera de fábrica** (medido,
/// extensão `2,0`): uma demão de `1,0` empurra a superfície pela própria
/// dimensão do modelo. A referência o declara alcançável e nós honramos; o que
/// nenhum dos dois oferece é o *slider* a chegar lá.
pub const LAYER_HEIGHT_HARD_MAX: f32 = 1.0;
