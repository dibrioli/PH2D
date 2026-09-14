//! O PINCEL — o verbo, o falloff e os knobs que o artista gira.
//!
//! Ver `docs/3D/04.1`. A divisão que este arquivo faz, e que decide tudo o
//! resto: **o `Dab` é onde e com que força a mão apertou; o `Brush` é que
//! ferramenta está na mão.** Um traço é uma lista de dabs contra UM brush.

use crate::falloff::Falloff;
use crate::pose_controlos::PoseControlos;
use crate::{Alpha, AlphaStencil};

/// **O CATÁLOGO** — ver [`verb`].
#[path = "brush_verb.rs"]
mod verb;

/// ⭐ **O que um verbo É** — os predicados, irmãos do [`verb`]. O corte é *o que
/// o verbo É* (aqui) contra *quais verbos existem e como se chamam* (lá), e ele
/// nasceu de o ficheiro do catálogo passar o tecto de LOC em 2026-09-06.
///
/// ⚠️ **O tecto estava vermelho ANTES desta jornada** (`714` de `700`) e nenhum
/// portão da linha o via: o `cargo test --bins` não alcança os gates que vivem
/// em `tests/`, e este vive lá.
#[path = "brush_verb_predicados.rs"]
mod verb_predicados;
pub use verb::{
    BLENDER_REACH_FRACTION, CLAY_PLANE_FRACTION, CLAY_THUMB_TILT_MAX_DEG, CLAY_THUMB_TILT_STEP_DEG,
    CREASE_FRACTION, DEFAULT_MULTIPLANE_ANGLE_DEG, FilterKind, LAYER_HEIGHT_HARD_MAX,
    LAYER_HEIGHT_UI_MAX, MULTIPLANE_ANGLE_MAX_DEG, MULTIPLANE_ANGLE_SMOOTH, MULTIPLANE_TIP_STRETCH,
    PINCH_GAIN, REACH_FRACTION, STRIP_PLANE_FRACTION, Verb,
};

/// A ferramenta na mão. Um traço inteiro corre contra um destes.
/// ⚠️ **`Copy` MORREU quando o alpha ganhou IMAGEM** — ver [`crate::Alpha`], onde
/// a decisão e o preço medido estão escritos. Nada no caminho quente copiava um
/// `Brush` por vértice; o que sobrou foram três `derive` e cinco `.clone()` num
/// arquivo de teste, contra a estimativa herdada de *"~20 arquivos"*.
#[derive(Clone, Debug, PartialEq)]
pub struct Brush {
    pub verb: Verb,
    /// **De qual REFERÊNCIA este pincel herda o que ele é** — [`crate::RefMode`].
    ///
    /// ⚠️ Ele só existe porque agora há um consumidor ([`Brush::weight`]); a W0
    /// deliberadamente não o adicionou, porque campo que ninguém lê é estado
    /// morto. Nasce em `S`, e o `ref_mode` diz por quê: o `S` é o **contrato de
    /// paridade**, e mover o default é decisão de PRODUTO.
    pub mode: crate::RefMode,
    /// **ACUMULAR na mesma pincelada** — o *Accumulate* do Blender, e o
    /// irmão exato do campo de mesmo nome do pincel 2D.
    ///
    /// Desarmado (o default) a lei é o ENVELOPE: cruzar o próprio traço não
    /// intensifica nada, e uma pincelada deposita no máximo a força do pincel.
    /// Armado ela é uma **integral de linha** — ver [`crate::ACCUM_PER_DAB`] —, e
    /// passar duas vezes soma duas vezes.
    ///
    /// ⚠️ **Só os verbos de CARIMBO o leem** ([`Verb::accumulates`]): quem tem
    /// âncora carrega o gesto TOTAL desde o pen-down, e somar totais seria somar
    /// a mesma coisa N vezes.
    pub accumulate: bool,
    pub falloff: Falloff,
    /// **O PADRÃO que decide onde, dentro da pegada, o verbo age** — ver
    /// [`crate::Alpha`]. `None` é o pincel liso, e ele é **byte-idêntico** ao
    /// mundo pré-alpha (a porta devolve `1.0` exato, e `x * 1.0 == x` no
    /// IEEE-754).
    pub alpha: Option<Alpha>,
    /// O tamanho de uma feature do alpha, em unidades de OBJETO — ver
    /// [`crate::DEFAULT_ALPHA_SCALE`].
    pub alpha_scale: f32,
    /// **O AZIMUTE do eixo do padrão**, em graus (`0..360`) — só os padrões
    /// direcionais o leem ([`Alpha::is_directional`]).
    ///
    /// ⚠️ **Graus INTEIROS, e é a convenção da lâmpada** (`ph2d_light::Light`):
    /// o rotor deste app anda de um grau em um grau, então um ângulo fracionário
    /// não teria como ser resolvido sem um segundo caminho. Um número, uma
    /// régua, um rotor.
    pub alpha_az_deg: u16,
    /// **A ELEVAÇÃO do eixo**, em graus (`0..=`[`crate::MAX_AXIS_ELEV_DEG`]).
    ///
    /// ⚠️ **Sem o piso que a LÂMPADA tem.** Lá o `MIN_ELEV_DEG` existe porque
    /// uma luz rasante degenera a resposta plana; um EIXO não degenera em lugar
    /// nenhum — o frame é ortonormal por identidade em qualquer elevação —, e um
    /// piso aqui seria um limite copiado de um vizinho em vez de medido.
    pub alpha_elev_deg: u16,
    /// **ONDE o padrão POUSA**, no plano do frame e em unidades de OBJETO.
    ///
    /// ⚠️ **É a terceira metade de COLOCAR um carimbo** — tamanho
    /// (`alpha_scale`), giro (`alpha_az_deg`, que no zênite ROLA o padrão no
    /// plano) e posição. As duas primeiras já existiam; esta faltava, e sem ela
    /// o artista podia dizer *quão grande* e *para que lado*, nunca *onde*.
    ///
    /// ⚠️ **Ele só alcança o motor com uma IMAGEM armada** — ver
    /// [`Brush::alpha_frame`], que é onde a neutralidade é garantida por
    /// CONSTRUÇÃO em vez de por convenção.
    pub alpha_offset: [f32; 2],
    /// **A VISTA, quando ela é conhecida** — o que faz de uma imagem um
    /// ESTÊNCIL preso ao viewport. Ver [`AlphaStencil`].
    ///
    /// ⚠️ **`None` é o mundo pré-estêncil, ao bit**: sem ele o
    /// [`Brush::alpha_frame`] devolve exatamente o frame autorado de antes. É o
    /// que mantém os nove procedurais — e todo gate de kernel — intocados.
    ///
    /// ⚠️ **Ele é ESTADO DE QUADRO, não estado autorado:** quem o preenche é a
    /// cena, uma vez por peça por quadro, porque só ela conhece a câmera e a
    /// pose. Guardá-lo como ajuste do pincel seria congelar uma vista que o
    /// artista move a cada gesto.
    pub alpha_stencil: Option<AlphaStencil>,
    /// **O tamanho de um ladrilho do ESTÊNCIL, em fração da ALTURA DA TELA.**
    ///
    /// ⚠️ **Um campo PRÓPRIO, e não uma reinterpretação do `alpha_scale`.** Os
    /// dois respondem a perguntas diferentes — *que tamanho tem esta feature no
    /// MODELO* contra *que tamanho tem este carimbo na TELA* — e um número só
    /// com duas unidades trocaria de significado em silêncio no instante em que
    /// o artista trocasse de padrão, que é a doença que este módulo varre a cada
    /// wave. Duas perguntas, dois números, duas rows: cada uma aparece no modo
    /// em que está viva.
    pub alpha_stencil_scale: f32,
    /// Raio de influência, em unidades de MUNDO.
    pub radius: f32,
    /// Intensidade em `[0, 1]` — o que multiplica o falloff para virar o peso.
    pub strength: f32,
    /// **A ESPESSURA da demão**, em unidades de OBJETO — é a ALTURA que o
    /// *Layer* da referência lê, e só o [`crate::Verb::Layer`] a usa aqui.
    ///
    /// ⚠️ **ABSOLUTO, e é o que separa a demão do [`crate::Verb::Draw`]:** o
    /// depósito do Draw é `força · RAIO · 0,1`, então mudar o pincel muda a
    /// altura; aqui o número é uma altura escolhida e um pincel maior só cobre
    /// mais área com a **mesma** espessura. A referência declara-o como
    /// DISTÂNCIA pela mesma razão.
    ///
    /// ⚠️ **O DEFAULT é nosso, e o da referência foi RECUSADO com medição.** Ela
    /// declara três números para este campo — faixa dura
    /// `[0, 1]`, faixa de UI `[0, 0,2]` e default `0,5` — e o terceiro cai
    /// **FORA** do segundo: copiá-lo shiparia um slider encostado no máximo, com
    /// o artista só podendo descer. (E o §7.0 deste plano já mediu que os
    /// defaults por-ferramenta do Blender vivem num `.blend` binário, não no
    /// código, então este `0,5` é o default do CAMPO e não o da demão.) Ficam as
    /// duas faixas, que a fonte declara, e o default sai do meio da que ela
    /// chama de trabalhável.
    ///
    /// ⚠️ **E o `0,1` tem um número no NOSSO mundo:** a esfera de fábrica tem
    /// raio `1,0` (extensão medida `2,0`), então uma demão de `0,1` é **10 % do
    /// raio** — da ordem de **2,5 dabs de Draw a raio 0,4** (`reach` `0,04`
    /// cada). É uma camada que se vê e não um bloco.
    pub layer_height: f32,
    /// O `Ctrl` de todo app de escultura: cava em vez de levantar.
    pub invert: bool,
    /// Desloca o plano ao longo da normal da área, em fração do raio. Positivo
    /// adiciona matéria (é o que faz do Flatten um Clay). Só os verbos de plano
    /// o leem — ver [`Verb::uses_plane`].
    pub plane_offset: f32,
    /// Quanto o `Crease` aperta lateralmente, em `[0, 1]`.
    pub pinch: f32,
    /// **QUÃO REDONDA É A PONTA**, em `[0, 1]` — `1` é um disco, `0` é uma caixa
    /// de quina viva. Só a [`Verb::ClayStrips`] a lê.
    ///
    /// ⚠️ **`1` devolve a distância euclidiana AO BIT** (ver
    /// [`crate::rounded_box`]), então este knob no default não é uma segunda
    /// silhueta: é a mesma.
    pub tip_roundness: f32,
    /// **QUANTOS RAIOS A FAIXA MEDE AO LONGO DO TRAÇO** — `1` é uma pegada
    /// quadrada, `3` é uma tira. Só a [`Verb::ClayStrips`] a lê.
    ///
    /// ⚠️ **O nome diz o que ele FAZ, e a referência o chama de `tip_scale_x`
    /// enquanto escala o eixo `y` da moldura dela.** Herdar o nome seria herdar
    /// uma confusão: o eixo que o número estica é o que corre AO LONGO do
    /// caminho, e é isso que o artista vê.
    pub strip_length: f32,
    /// **QUANTO O V DO MULTIPLANE SCRAPE ABRE**, em graus (`0..=`
    /// [`MULTIPLANE_ANGLE_MAX_DEG`]). Só a [`Verb::MultiplaneScrape`] o lê.
    ///
    /// ⚠️ **Em `0` a ferramenta fica INERTE, e MEDIDO isso é mais forte do que
    /// eu ia escrever.** A primeira versão deste doc dizia *"`0` é o
    /// [`Verb::Scrape`] AO BIT"* — é falso, e a diferença é a ORIGEM: o Scrape
    /// projeta no plano de ÁREA (a média da pegada, que numa superfície convexa
    /// fica ABAIXO da superfície, então a crista sobra acima dele e é raspada),
    /// e os dois meios-planos deste verbo passam pelo **centro do dab**, ou seja
    /// pelo plano TANGENTE — e acima de um plano tangente, num convexo, não há
    /// nada. Medido no mesmo traço: **994 vértices movidos pelo Scrape contra
    /// ZERO** com o V fechado.
    ///
    /// ⚠️ **Ele fica alcançável na mesma**, e não por descuido: a continuidade é
    /// real (o V fecha, o corte desaparece), e um piso acima de zero seria um
    /// número nosso a esconder um degrau que a física não tem. O que zero não
    /// pode ser é o **default** — ver [`DEFAULT_MULTIPLANE_ANGLE_DEG`].
    ///
    /// ⚠️ **No modo dinâmico ele deixa de ser o ângulo e passa a ser um
    /// ACRÉSCIMO** ao que a superfície ditou (o ângulo autorado soma-se,
    /// escalado pela pressão, ao ângulo amostrado) — o mesmo número com dois
    /// papéis, como na referência, e é por isso que o rótulo dela diz *"Plane
    /// Angle"* nos dois modos.
    pub scrape_angle_deg: f32,
    /// **O V É LIDO DA SUPERFÍCIE** em vez de ser autorado — o modo dinâmico do
    /// *Multiplane Scrape* da referência.
    ///
    /// Armado, cada dab amostra a normal média dos DOIS lados da lâmina, mede o
    /// ângulo entre elas e usa isso como a abertura do V — a ferramenta encontra
    /// a dobra que já existe e raspa **ao longo dela** em vez de impor um vinco
    /// próprio. O [`Brush::scrape_angle_deg`] passa a somar-se ao que foi lido.
    ///
    /// ⚠️ **Ele também sente o SINAL da dobra:** numa aresta côncava o V é
    /// invertido e a ferramenta passa a ENCHER a dobra em vez de a
    /// cavar. É o que faz um único pincel servir a crista e o vale.
    ///
    /// ⚠️ **Desarmado por default, e o motivo é o mesmo do resto deste módulo:**
    /// o conjunto de flags ZERADO do pincel genérico é o único valor citável, e um
    /// modo que se arma sozinho é um pincel cujo desenho muda por uma razão que
    /// o artista não vê. Quem o quer, marca.
    pub scrape_dynamic: bool,
    /// **A dureza da borda do canal de MÁSCARA**, em `[0, 1]` — o `_hardness`
    /// da tool `Masking` do original (`Masking.js:14`).
    ///
    /// ⚠️ **Ele existe porque a máscara tem CURVA PRÓPRIA, e isso é da
    /// referência, não nosso.** A afirmação *"o SculptGL tem uma curva para
    /// tudo"* vale para as dez tools que movem GEOMETRIA; o canal usa
    /// `(1 − d)^{2(1 − hardness)}` (ver [`Brush::mask_weight`]), e é por isso
    /// que o [`Falloff`] — o nosso seletor, que é um superconjunto do que a
    /// referência oferece à geometria — **não** o alcança. Foi exatamente o
    /// pedido *"cada tool deve ter seu falloff apropriado"*: aqui ele não é uma
    /// escolha de produto, é o que o modelo já fazia.
    ///
    /// ⚠️ **`0.25` é o valor DE FÁBRICA da tool**, não um número escolhido —
    /// ele dá expoente `1.5`, uma borda visivelmente mais apertada que a
    /// quártica da geometria (a `Plateau` vale `0,6875` a meio raio, esta vale
    /// `0,3536`). Em `1.0` o expoente é ZERO e o canal vira um disco duro.
    pub mask_hardness: f32,
    /// **A DUREZA DO DAB** em `[0, 1]` — o `hardness` do Blender, e **`0` é a
    /// identidade**.
    ///
    /// ⚠️ **Ele NÃO é o [`Brush::mask_hardness`], e os dois nomes se parecem o
    /// bastante para se trocarem em silêncio.** Aquele é a forma da CURVA do
    /// canal de máscara (`(1 − t)^{2(1 − h)}`, do `Masking.js`); este reescreve
    /// a **DISTÂNCIA** que qualquer curva depois lê — o remapeamento de DUREZA
    /// da referência, portado em
    /// [`Brush::shaped_distance`]. Curva e distância são perguntas diferentes, e
    /// é por isso que os dois coexistem em vez de um vencer o outro.
    ///
    /// ⚠️ **O default é `0.0` e ele é o NEUTRO do próprio código de origem** —
    /// lá o remapeamento devolve cedo, sem tocar em nada, quando a dureza é zero.
    /// Não é um número que eu escolhi: é o early-out deles.
    ///
    /// ⚠️ **E o valor de FÁBRICA de um pincel do Blender não é legível** (§7.0
    /// do plano: desde o 4.3 ele vive num `.blend` binário), então este knob
    /// nasce no neutro e o número passa a ser do ARTISTA — nunca uma tabela
    /// inventada com o nome de outro produto.
    pub hardness: f32,
    /// **QUE FRACÇÃO DO RAIO A NORMAL DO GESTO LÊ** — o miolo de que o
    /// [`Verb::Thumb`] e o [`Verb::Nudge`] tiram o plano tangente. Ver
    /// `stroke_normal_do_gesto.rs`.
    ///
    /// ⚠️ **É um knob da REFERÊNCIA, não um número nosso** (a opção pública
    /// existe lá e nasce em `0,5`), e ele **não** é o raio do pincel: a pegada
    /// que se move continua a ser a inteira. O que esta fracção decide é *de
    /// que superfície o gesto se declara paralelo*.
    ///
    /// ⚠️ **Duas fixtures do oráculo dependem dele** (as `…raionormal03`, a
    /// `0,3`): sem o knob elas seriam inexplicáveis, e com ele fecham no mesmo
    /// resíduo das outras — é a diferença entre uma constante e um controlo.
    pub normal_radius_frac: f32,
    /// **A ÂNCORA CAI NUM VÉRTICE** — a opção pública do agarrar
    /// (`use_grab_active_vertex` na referência). Ver
    /// [`crate::ancora::ancora_do_gesto`], onde o número está.
    ///
    /// ⚠️ **Ela só muda o PEN-DOWN**, e mais nada: escolhido o ponto, o gesto
    /// segue idêntico. Por isso ela vive aqui e não numa lei de kernel.
    pub grab_active_vertex: bool,
    /// **OS SEIS CONTROLOS PRÓPRIOS DO PINCEL DE POSE** — ⛔ só o
    /// [`crate::Verb::Pose`] os lê.
    ///
    /// ⚠️ Eles vivem agrupados num tipo da crate da lei, e não soltos aqui, de
    /// propósito: são **seis** e crescem juntos, e uma struct de pincel com
    /// seis campos soltos que só um verbo lê é onde o sétimo nasce esquecido
    /// num dos consumidores. O raio, a força e a curva **não** entram aqui —
    /// esses são partilhados e já existem no pincel.
    pub pose: PoseControlos,
    /// Os controlos próprios do pincel de CONTORNO — ⚠️ **quatro, e é a espec
    /// que os conta**. Só o [`crate::Verb::Boundary`] os lê.
    pub boundary: crate::boundary_controlos::BoundaryControlos,
    /// **AS DUAS DIRECÇÕES DO PINCEL DE DENSIDADE** — ⛔ só o
    /// [`crate::Verb::Density`] o lê, e ele é o ÚNICO ajuste próprio daquele
    /// pincel (todo o resto — raio, força, curva — não tem lei por-vértice onde
    /// agir, ver [`crate::Verb::sem_lei_por_vertice`]).
    ///
    /// ⚠️ **Ele vive no PINCEL e não na cena, ao contrário do alvo**, e a
    /// escolha é da espec §9.8: existe um pedido público aberto para tirar os
    /// ajustes de topologia da cena dele e os pôr no pincel. *Não copiámos o
    /// modelo que a referência está a caminho de abandonar.*
    pub density_modo: crate::DensityModo,
    /// **SÓ AS FACES DE FRENTE** — a opção de pincel *"Front Faces Only"* da
    /// referência (rótulo público: é o que o artista vê na tela dela).
    ///
    /// ⭐ **ESTA É A CASA CANÓNICA DO FACTO.** Os outros sítios que dependem
    /// dele apontam para aqui em vez de o repetir — *uma lei escrita em quatro
    /// sítios ainda não é uma lei; só uma PORTA é.*
    ///
    /// Ligado, o fator de cada vértice é escalado por `max(n · olho, 0)` — o
    /// [`crate::FrontFace::Continuous`] que o modo declara. Desligado, a linha
    /// **não corre**, que é o que a referência faz por omissão: lá o teste é
    /// **condicionado ao bit** em todo verbo que deposita, e **nada no programa
    /// o LIGA** (varrido: todos os sítios que o mencionam são LEITURAS).
    ///
    /// ⚠️ **A lei e o interruptor são coisas diferentes, e é por isso que são
    /// dois campos.** O [`crate::KernelLaw::front_face`] responde *qual* lei a
    /// referência deste modo aplica quando o artista a pede — o `S` não tem
    /// nenhuma (o `_culling` do SculptGL é outro eixo, binário e no PLANO), o
    /// `B` tem a contínua. Colapsar os dois num só faria *escolher a referência*
    /// e *marcar um checkbox* serem o mesmo gesto.
    ///
    /// ⚠️ **O default vem do VERBO** ([`Verb::default_front_faces_only`]), a
    /// mesma tabela de `falloff`/`strength`/`accumulate`, e ela diz `false` para
    /// tudo menos a faixa — com o número da medição ao lado.
    pub front_faces_only: bool,

    /// ⭐⭐⭐ ***Connected Only*** — o pincel age só no que a superfície LIGA ao
    /// ponto que o artista aponta.
    ///
    /// # O defeito que ele cura, medido
    ///
    /// Um dab junta os vértices dentro de uma **esfera** e pesa cada um pela
    /// distância **pelo ar**. Numa malha com duas partes vizinhas — um modelo
    /// importado em duas peças, o resultado de um *Extract*, dois dedos — a
    /// esfera alcança o outro lado. Medido (duas peças, folga `0,05`, raio
    /// `0,35`): **`47,1 %` do peso do carimbo cai na peça ERRADA**, e um dab real
    /// move `60` vértices dela.
    ///
    /// ⭐ **Ligado, o peso de quem FICA não muda um bit** — a lei é uma máscara e
    /// não uma régua nova ([`crate::dab_alcance`]), então nada do que já estava
    /// certo se mexe por causa dela.
    ///
    /// # ⚠️ O default é `true` por DECISÃO DO DONO (2026-09-10)
    ///
    /// *«As duas opções devem existir com a segunda como default»* — a segunda
    /// era o lado curado da comparação da cena `=39`. ⛔ A minha proposta era
    /// nascer desligado, porque um gate de arquitectura tinha apanhado a
    /// justificação que eu dera para o contrário; o veredito é dele e o registo
    /// da minha objecção está no [`crate::dab_alcance`].
    ///
    /// ⚠️ **Ele é um CAMPO e não uma variável de ambiente**, e isso paga-se em
    /// duas coisas que uma env não podia dar: o artista escolhe (era a ordem), e
    /// uma **fixtura pode PREGÁ-LO** para continuar a reproduzir a geometria em
    /// que foi calibrada — que é o que a [`shells/desktop`] faz com a orelha.
    pub surface_only: bool,
    /// **O ALISAMENTO QUE CORRE DEPOIS DE CADA DAB**, em `[0, 1]` — o
    /// factor de AUTO-ALISAMENTO da referência, e **`0` é o neutro**.
    ///
    /// ⚠️ **Ele é o que faltava, e a medição é que o nomeia.** O report de
    /// 2026-08-16 (*"tanto hardness como falloffs apresenta problemas graves"*,
    /// com foto de traços em escamas) foi atribuído medindo, e a atribuição
    /// ABSOLVEU as duas leis: um dab só reproduz a curva analítica a três
    /// decimais nas doze, e o `hardness` é a mesma lei da etapa de dureza da
    /// referência, no mesmo ponto do pipeline — medida pelo gate de dureza de
    /// `brush_tests.rs` (`the_hardness_remaps_the_distance_the_way_the_reference_does`).
    /// O que a foto mostra é o **degrau**
    /// que uma curva de PLATÔ necessariamente escreve, e nenhuma das duas
    /// referências o limita na aritmética — o Blender o limita **noutro passe**.
    ///
    /// Medido num traço de raio `0,30` e força `0,5` sobre a esfera de fábrica
    /// (98 306 vértices), o pior diedro entre triângulos vizinhos:
    ///
    /// | curva | diedro |
    /// |---|---|
    /// | esfera intocada (controle) | 0,89° |
    /// | `Plateau` (o de fábrica) | 4,88° |
    /// | `Smooth` | 4,52° |
    /// | `Sphere` | 41,79° |
    /// | `Constant` | **90,93°** |
    ///
    /// e pela dureza, com a curva de fábrica: `0` → 4,52° · `0,5` → 11,56° ·
    /// `0,75` → 30,23° · `0,9` → 55,99° · `1,0` → **90,93°**. Acima de 90° a
    /// superfície não é um penhasco, é uma **dobra**.
    ///
    /// ⚠️ **Na referência ele é declarado ao LADO da dureza**, e a adjacência
    /// não é acaso: são os dois knobs que trocam
    /// **borda dura** por **superfície que a malha consegue carregar**.
    ///
    /// ⚠️ **Nasce em `0`, que é o default do Blender** — subir isto mudaria o
    /// desenho de todo traço de todo documento, e é decisão de produto.
    pub auto_smooth: f32,
    /// **QUÃO LARGO é o campo elástico** — a família de escalas do
    /// [`crate::kelvinlet`], e o único knob que o `l-mode` tinha e não oferecia.
    ///
    /// ⚠️ **É o conteúdo que o pincel *Elastic Deform* do Blender vende, e o
    /// verbo dele foi RECUSADO por medição:** os cinco tipos de deformação
    /// daquele pincel são *Grab · Grab Biscale · Grab Triscale · Scale · Twist*,
    /// e os cinco já existem aqui como verbos com `l-mode` ([`Verb::Move`],
    /// [`Verb::LocalScale`], [`Verb::Twist`]) — os três primeiros diferem
    /// **apenas nesta família**. Um `Verb::ElasticDeform` seria um sexto nome
    /// para ferramentas que o rail já tem, com um sub-dropdown a duplicar o
    /// seletor de modo; o que faltava de verdade era este número.
    ///
    /// ⚠️ **O ALCANCE não é função da família, e a tentativa foi REFUTADA pela
    /// medição que já estava escrita.** Eu ia fazer o [`crate::KELVINLET_REACH`]
    /// variar por família, porque o resíduo de borda cru difere muito
    /// (`0,3162 · 0,0778 · 0,0347`) e igualá-lo custaria alcance `28,8 · 4,2 ·
    /// 3,0` — a `Mono` seria o modelo inteiro dentro da pegada. O doc-comment do
    /// [`crate::kelvinlet::rim_landing`] já dizia o contrário, com número:
    /// *"alargar o alcance NÃO é a cura — nunca dá zero; a janela dá exatamente
    /// zero, por construção, a QUALQUER alcance"*. Medido depois da janela, as
    /// três famílias chegam a **0,00000** na borda e a diferença é só a
    /// LARGURA, que é a feature:
    ///
    /// | família | meia-largura (`r/ε`) | fração do bico em `0,75·reach` |
    /// |---|---|---|
    /// | `Mono` | **1,74** | 0,40614 |
    /// | `Bi` | 1,04 | 0,14791 |
    /// | **`Tri`** | **0,93** | 0,08662 |
    ///
    /// ⚠️ **O preço da `Mono` fica NOMEADO:** no último quarto da pegada quem
    /// desenha o perfil é a janela, não o kernel (ela engole 40 % do bico contra
    /// os 8,7 % da `Tri`). É um ombro `C¹`, não um degrau — mas é o ombro da
    /// aterrissagem, e quem escolher a família mais larga está a escolhê-lo.
    ///
    /// **O default é [`Scales::Tri`]**, o que já shipava em todos os oito sítios
    /// do [`crate::stroke_target`] ⇒ o mundo pré-knob é **byte-idêntico**.
    pub elastic_scales: crate::kelvinlet::Scales,
    /// **α do HC — *Shape Preservation*** ([`Verb::SurfaceSmooth`]): quanto o
    /// ponto de referência da correção é a pose do **pen-down** em vez da de
    /// agora. Ver [`crate::HC_SHAPE_DEFAULT`].
    pub hc_shape: f32,
    /// **β do HC — *Per Vertex Displacement*** ([`Verb::SurfaceSmooth`]): que
    /// fração da correção vem do próprio vértice em vez da média dos vizinhos.
    /// Ver [`crate::HC_VERTEX_DEFAULT`].
    ///
    /// ⚠️ **Os dois são INERTES em vinte e um dos vinte e dois verbos**, e é o
    /// `fill_hc_disp` que os lê — nenhum outro braço do alvo os toca, então o
    /// mundo pré-wave é byte-idêntico com eles em qualquer valor.
    pub hc_vertex: f32,
    /// **COMO o pincel de tecido deforma** ([`crate::ClothMode`]) — só o
    /// [`Verb::Cloth`] o lê.
    ///
    /// ⚠️ **Ele existia como VARIÁVEL DE AMBIENTE**, e por isso o pincel tinha
    /// oito comportamentos e um alcançável. A lei responde aos oito desde
    /// 2026-09-05; o que faltava era o campo e a fileira que o escreve.
    pub cloth_mode: crate::ClothMode,
    /// **QUE PEDAÇO da malha entra na simulação** ([`crate::ClothArea`]).
    ///
    /// ⚠️ **A omissão é `Dynamic`**, que é a dos treze presets do alvo — ⛔ e
    /// **não** `Local`, que é a omissão do código dele. A distinção está medida:
    /// a `Local` constrói a lista de restrições em duplicado e entrega no centro
    /// `0,34×` o que a `Global` entrega.
    pub cloth_area: crate::ClothArea,
    /// **A FORMA ESPACIAL do peso da força** ([`crate::ClothForceFalloff`]).
    ///
    /// ⚠️ **Ele era um LITERAL na tradução `Brush → Pincel`**, e o corpus do
    /// oráculo tem quatro traços que só a forma de PLANO exercita.
    pub cloth_force_falloff: crate::ClothForceFalloff,
    /// ***Simulation Limit* `L`** — quantos raios de pincel a área simulada
    /// alcança: o limite é `R·(1+L)` e a banda começa em `R·(1+L·F)` (espec §2.2).
    ///
    /// ⚠️ **É de TEMPO e de ALCANCE ao mesmo tempo** (espec §8.1): mais limite é
    /// mais malha na simulação. Faixa `0,1..10`, omissão `2,5`.
    pub cloth_limit: f32,
    /// ***Simulation Falloff* `F`** — onde, dentro do limite, a banda começa a
    /// descer. `1` põe o início no próprio limite (banda de largura zero) e `0`
    /// põe-no no raio do pincel. Faixa `0..1`, omissão `0,75`.
    pub cloth_falloff: f32,
    /// ***Pin Simulation Boundary*** — os vértices da franja da banda ganham uma
    /// restrição ao repouso com força `1 − w` (espec §2.3).
    ///
    /// ⚠️ **Só a área *Local* o tem**, e a lei recusa-o nas outras — o painel
    /// pergunta o mesmo antes de o pintar, senão é um interruptor de coisa
    /// nenhuma nos outros dois terços do selector.
    pub cloth_pin: bool,
    /// ***Cloth Mass*** — ganho inverso puro sobre o passo de tempo (espec §5.4):
    /// dobrar divide exactamente por dois o deslocamento por força num passo.
    /// Faixa `0,01..2`, omissão `1`.
    pub cloth_mass: f32,
    /// ***Cloth Damping*** — a fracção de velocidade PERDIDA por passo (espec
    /// §5.3). ⚠️ Não é Rayleigh nem viscosidade. Faixa `0,01..1`, omissão `0,01`
    /// (⇒ `99 %` de retenção, que é o que faz o traço assentar).
    pub cloth_damping: f32,
    /// ***Soft Body Plasticity* `ρ`** (espec §5.2): `0` = a memória de forma
    /// segue o vértice e nunca o puxa; `1` = o vértice volta à memória.
    /// Faixa `0..1`, omissão `0`.
    pub cloth_plasticity: f32,
    /// ⭐⭐⭐ ***Cloth Quality*** — quantas varreduras de relaxação por passo.
    ///
    /// ⚠️⚠️ **O ALVO FIXA ISTO EM `5`** ([`ph2d_cloth::verlet::VARREDURAS`]) e
    /// não o oferece; é o maior controlo que este pano ganha sobre o dele. O que
    /// ele compra está medido no doc do gémeo do filtro
    /// ([`crate::ClothFilterProps::sweeps`]) — **de `5` para `32` o pior esticão
    /// de um aperto cai `4,9×`** —, e o teto de `32` sai da mesma tabela: a `64`
    /// o pior caso **piora**.
    ///
    /// ⚠️ **Omissão `5` ⇒ byte-idêntico ao que shipava**, que é o que mantém os
    /// 86 traços da bancada do pincel onde estão.
    ///
    /// ⚠️ **O recurso é TEMPO e cresce com a MALHA** — ver o doc do gémeo.
    pub cloth_sweeps: u32,
    /// ***Persistent*** (espec §6.4) — a construção lê a **base congelada** no
    /// lugar das posições de repouso do traço.
    ///
    /// ⭐⭐ **O efeito é SATURAÇÃO, não atenuação:** o mesmo traço repetido dá
    /// `0,169 → 0,306 → 0,415` sem base e `0,169 → 0,171 → 0,176` com ela — *a
    /// deformação pára de acumular e assenta no que UM traço faz*.
    ///
    /// ⚠️ **Ligado SEM base gravada é um no-op exacto** (a construção cai no
    /// repouso do traço), e é por isso que o painel oferece os dois: o
    /// interruptor e o botão que o torna observável.
    pub cloth_persistent: bool,
    /// ***Use Collisions*** (espec §5.6) — o pano pára nas OUTRAS peças da cena.
    ///
    /// ⚠️ **A lista de colisores é montada UMA vez, no 1.º passo do traço**, na
    /// pose desse instante ⇒ *uma peça que se mova durante o traço não se move
    /// para o pano, e uma que apareça a meio não entra.*
    ///
    /// ⛔ **Nasce desligada**, e não por conservadorismo: cada vértice activo
    /// paga um raio por colisor e por passo.
    pub cloth_collisions: bool,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            verb: Verb::Draw,
            mode: crate::RefMode::S,
            // ⚠️ **DERIVADO do verbo, nunca um literal ao lado dele.** O
            // `Brush.js:16` da referência ship `_accumulate = true`, e a tool
            // `Brush` dele é a nossa Draw+Clay — nós shipávamos os dois
            // desarmados, que é o *"com accumulate checado por padrão"* do
            // pedido original. Escrever `true` aqui poria o MESMO fato em dois
            // lugares, e o dia em que a tabela do verbo mudasse este literal
            // ficaria a contradizê-la em silêncio.
            accumulate: Verb::Draw.default_accumulate(),
            // ⚠️ **E a CURVA delega pela mesma razão, que o comentário logo
            // acima já enunciava e que este literal contradizia** (corrigido em
            // 2026-08-12, plano 21 W0). Ele dizia `Falloff::Smooth`, uma curva
            // que **nenhuma referência declara** — o D1 do
            // `docs/3D/20_divergencias_tools.md`, e o achado que o artista
            // encontra sem tocar em nada: a quártica da referência é `1,08× a
            // 1,44×` mais cheia ao longo do raio.
            //
            // ⚠️ **E o literal aqui não era só *um* default errado — ele
            // TRAVAVA o arming:** a lei *"arma se o artista não mexeu"* compara
            // com o que o verbo que SAI declara, então um pincel de fábrica em
            // `Smooth` contra uma tabela que diz `Plateau` parece *mexido* e
            // nunca mais seria armado por ninguém.
            falloff: Verb::Draw.default_falloff(crate::RefMode::S),
            alpha: None,
            alpha_scale: crate::DEFAULT_ALPHA_SCALE,
            // ⚠️ **O eixo nasce em +Y — as camadas saem HORIZONTAIS**, que é a
            // leitura que um estrato tem no mundo e a que o olho resolve na
            // primeira olhada. Com `az = 0` o eixo seria +X e as camadas
            // sairiam de pé; com `elev = 90` ele apontaria para a CÂMERA (a
            // vista é `+Z`) e o artista veria uma camada só, o que é
            // indistinguível de *"o padrão não funciona"*.
            alpha_az_deg: 90,
            alpha_elev_deg: 0,
            alpha_offset: [0.0, 0.0],
            alpha_stencil: None,
            // Um quarto da altura da tela por ladrilho — quatro carimbos
            // atravessando o que se vê. ⚠️ E ele **não** é semeado do modelo: um
            // estêncil não sabe o tamanho da peça, e é essa independência que o
            // artista pediu.
            alpha_stencil_scale: 0.25,
            radius: 0.25,
            strength: 0.5,
            // O meio da faixa que a referência declara trabalhável — ver o doc
            // do campo, e a medição que recusou o `0,5` dela.
            layer_height: 0.1,
            invert: false,
            plane_offset: 0.0,
            pinch: 0.5,
            // ⚠️ **PONTA QUADRADA, e o `1.0` que eu shipei era o §0 mordendo em
            // casa.** O único número citável — a redondeza de ponta `1,0` do
            // pincel GENÉRICO da referência — não é o desta ferramenta (a tabela
            // por-ferramenta vive na rotina de reset, que já não existe, §7.1). Deixei o
            // fallback definir o produto, e o
            // preço foi a ferramenta inteira: com `1` a caixa É a distância
            // euclidiana, e a faixa saía redonda. *"parece redondo"* (Enio).
            //
            // ⚠️ **A byte-identidade nunca dependeu deste número.** Quem a
            // carrega é a [`crate::Footprint::Disc`], que é a rota dos outros
            // dezasseis verbos; a faixa é nova e não tem mundo anterior a
            // preservar.
            //
            // ⚠️ **E o número é MEDIDO, declarado como NOSSO.** A propriedade que
            // separa uma faixa de um domo é o traço ter **lados paralelos** —
            // medida a largura do depósito em sete secções ao longo do caminho:
            //
            // | roundness | larguras | ponta ÷ meio |
            // |---|---|---|
            // | 0,00 | `0,8` nas sete | **1,00** |
            // | 0,25 | `0,7` nas sete | **1,00** |
            // | 1,00 | `0,5 0,5 0,6 0,6 0,6 0,5 0,5` | **0,83** |
            //
            // `0,25` dá o mesmo lado paralelo que a quina viva, o maior platô da
            // varredura (**23,7 %** dos vértices movidos contra 16,3 % em `0`) e
            // ainda arredonda a quina o bastante para ela não virar um degrau
            // numa malha grossa.
            tip_roundness: 0.25,
            // ⚠️ **Uma pegada de LADOS IGUAIS por default, e a medição diz que
            // é o certo:** a tira nasce do TRAÇO, não de um dab esticado — é a
            // quina reta que faz os lados ficarem paralelos, e o esticão é um
            // segundo eixo de estilo. O pincel genérico da referência diz `1,0`
            // e aqui ele concorda com o que a sonda mostra.
            strip_length: 1.0,
            // ⚠️ **DELEGA à constante MEDIDA**, e não repete o número: o dia em
            // que a varredura mudar de veredito, um literal aqui ficaria a
            // contradizê-la em silêncio — e o gate do V é escrito para não
            // mencionar nenhum dos dois.
            scrape_angle_deg: DEFAULT_MULTIPLANE_ANGLE_DEG,
            // O conjunto de flags zerado do pincel genérico — ver o campo.
            scrape_dynamic: false,
            // O `_hardness` de fábrica da `Masking` do original.
            mask_hardness: 0.25,
            // O neutro da etapa de dureza — ver o campo.
            hardness: 0.0,
            // ⚠️ **`0,5` é o valor de fábrica da referência, lido do cabeçalho
            // das fixtures** (`fator_raio_da_normal`), não um palpite — e as
            // duas fixtures que o movem para `0,3` são o controlo de que ele
            // chega ao resultado.
            normal_radius_frac: 0.5,
            // ⚠️ **Desligada, como na referência** — e a diferença só se vê em
            // malha grossa (ver o doc da porta).
            grab_active_vertex: false,
            pose: PoseControlos::default(),
            boundary: crate::boundary_controlos::BoundaryControlos::default(),
            // ⚠️ **IGUALAR de omissão** — ver o doc do enum: é o ajuste de
            // refino que a referência ship, e é a metade do report do dono
            // (*«por que não pode aumentar a densidade também?»*).
            density_modo: crate::DensityModo::default(),
            // ⚠️ **DERIVADO do verbo, como o `accumulate` e o `falloff` logo
            // acima** — e pela mesma razão: um literal aqui seria o MESMO fato
            // em dois lugares, e no dia em que a tabela do verbo mudasse ele
            // ficaria a contradizê-la em silêncio. Ele também TRAVARIA o arming,
            // que compara com o que o verbo que sai declara.
            front_faces_only: Verb::Draw.default_front_faces_only(),
            surface_only: true,
            // O default do Blender, e o neutro deste passe — ver o campo.
            auto_smooth: 0.0,
            // ⚠️ **DELEGA, e não repete a palavra `Tri`:** a família que shipa
            // é a que a MEDIÇÃO escolheu (o resíduo de borda, em
            // [`crate::kelvinlet::Scales`]), e escrevê-la aqui poria o mesmo
            // fato em dois lugares — no dia em que a medição mudar de veredito,
            // este literal ficaria a contradizê-la em silêncio.
            elastic_scales: crate::kelvinlet::Scales::default(),
            hc_shape: crate::HC_SHAPE_DEFAULT,
            hc_vertex: crate::HC_VERTEX_DEFAULT,
            cloth_mode: crate::ClothMode::default(),
            cloth_area: crate::ClothArea::default(),
            // ⚠️ **As sete omissões são as do CÓDIGO do alvo** (espec §8.1), e
            // não as dos presets — as dos presets já governam o `cloth_area`,
            // com o número medido ao lado dele. Com estes valores a tradução
            // `Brush → Pincel` entrega exactamente o `Pincel::default()` que a
            // bancada de paridade corre ⇒ o mundo pré-wave é byte-idêntico.
            cloth_force_falloff: crate::ClothForceFalloff::default(),
            cloth_limit: 2.5,
            cloth_falloff: 0.75,
            cloth_pin: false,
            cloth_mass: 1.0,
            cloth_damping: 0.01,
            cloth_plasticity: 0.0,
            cloth_sweeps: ph2d_cloth::verlet::VARREDURAS,
            cloth_persistent: false,
            cloth_collisions: false,
        }
    }
}

/// As portas entre o número do artista e o que o kernel consome — ver o
/// cabeçalho dele.
#[path = "brush_scale.rs"]
mod brush_scale;
// ⚠️ O teto do knob viaja com a porta que o consome, e o caminho público do
// chamador não muda: quem escreve `brush::MAX_MASK_HARDNESS` continua certo.
pub use brush_scale::MAX_MASK_HARDNESS;

/// **Quantos passes este pincel faz** — ver o cabeçalho dele.
#[path = "brush_pass.rs"]
mod brush_pass;
pub use brush_pass::{Pass, RingOperator, TAUBIN_LAMBDA, TAUBIN_MU, TAUBIN_PASS_BAND};

/// **AS PORTAS DO ALPHA** — ver [`alpha_doors`].
#[path = "brush_alpha.rs"]
mod alpha_doors;

/// **OS ESPELHOS QUE MULTIPLICAM UM DAB** — ver [`symmetry`].
#[path = "brush_symmetry.rs"]
mod symmetry;
pub use symmetry::Symmetry;

#[cfg(test)]
#[path = "brush_tests.rs"]
mod tests;
