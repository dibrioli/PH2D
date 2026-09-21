//! **AS OPÇÕES DE SOMBREAMENTO DO BARRO** — o que o artista escolhe sobre como
//! a forma é lida, separado de com que luz ela é acesa.
//!
//! Hoje há uma: a **CAVIDADE** (`docs/3D/05.1` §4), que o doc nomeia como *o
//! canal que mais melhora a leitura de uma escultura por unidade de custo*. As
//! próximas (SSS, AO, IBL) entram aqui pelo mesmo caminho, e é por isso que ele
//! é um struct e não um `f32` solto.
//!
//! # Por que UNIFORM e não `const` de permutação
//!
//! O `docs/3D/05.1` fecha pedindo *"um WGSL único, com `const` de permutação
//! resolvidos na compilação do pipeline … não `if` em runtime"*. Isso vale para
//! **capacidades** (`SSS on/off`, `IBL on/off`) — coisas que mudam o corpo do
//! shader e cujo custo é pago por quem não as usa. Uma **quantidade** que o
//! artista arrasta é o oposto: recompilar um pipeline por posição de slider é
//! uma trava de meio segundo por passo do gesto.
//!
//! ⚠️ E o zero **não** precisa de permutação para ser barato: o termo é uma
//! multiplicação por `1.0`, não um passe.

use bytemuck::{Pod, Zeroable};

/// **QUANTO a cavidade escurece a fresta e clareia a crista.**
///
/// Ela vale para os DOIS sinais, e isso é uma decisão sobre o modelo e não uma
/// economia: a curvatura é *um* número com sinal, e escurecer o côncavo e
/// clarear o convexo são as duas metades da mesma multiplicação
/// (`1 − amount × k`). O `docs/3D/05.1` §4 fala em dois sliders — *Cavity* e
/// *Edge Wear* — e eles são a UI de um MATERIAL (sujeira na fresta × tinta gasta
/// na quina são histórias físicas diferentes, com quantidades diferentes). Para
/// **ler forma**, que é o que esta wave entrega, um número simétrico é o modelo
/// honesto; inventar o segundo agora seria um knob que nenhum gesto alcança.
///
/// ⚠️ **Se o smoke disser que os dois lados querem quantidades diferentes, é ELE
/// que parte este número em dois** — o oráculo disso é o olho, não a aritmética.
pub const DEFAULT_CAVITY: f32 = 0.0;

/// **O GANHO** que leva a curvatura crua à faixa que o olho usa.
///
/// **MEDIDO** (`measure_curvature`, esferas trianguladas com sete traços de
/// Draw — o que uma mão faz nos primeiros segundos):
///
/// | malha | `k` mediano | `\|k\|` p99 | `\|k\|` máximo |
/// |---|---|---|---|
/// | esfera CRUA 48×72 | −0,0372 | 0,045 | 0,045 |
/// | esculpida 48×72 | −0,0372 | **0,305** | 0,685 |
/// | esculpida 96×144 | −0,0189 | **0,140** | 0,704 |
///
/// ⚠️ **A tabela traz um fato que eu não tinha previsto, e ele é a razão de o
/// ganho poder ser fixo:** a curvatura de FUNDO (a da própria esfera) cai pela
/// metade quando a tesselação dobra — exatamente o `−h/(2R)` —, mas a de um
/// VINCO fica onde está (0,685 e 0,704). Um vinco é um vinco em qualquer
/// densidade. Então o que o canal desenha — o CONTRASTE entre fresta e fundo —
/// não é função de quantos triângulos a peça tem, e um ganho constante serve as
/// duas.
///
/// **4,0 satura em `\|k\| ≥ 0,25`**, que fica entre os dois p99 medidos: o 1%
/// mais vincado clampa e todo o resto responde proporcionalmente. O fundo liso
/// escurece 7 a 15% em `cavity = 1` — uniforme, portanto invisível como
/// artefato, que é o que sobra depois de o contraste ter sido gasto no que
/// interessa.
pub const CAVITY_GAIN: f32 = 4.0;

/// **Quanto do AO assado entra por default: NADA.**
///
/// ⚠️ Não é timidez, é a única resposta que não muda a tela sozinha. O canal
/// nasce ausente e sobe como `1` em toda parte (ver `ao_of`), então com uma
/// força qualquer acima de zero a peça renderizaria igual **até o instante do
/// primeiro bake** — e aí
/// escureceria sem ninguém ter tocado num controle, que é a classe de mudança
/// que o artista não consegue reportar.
pub const DEFAULT_AO_STRENGTH: f32 = 0.0;

/// **Quanto do AO DE TELA entra por default: TUDO.**
///
/// ⚠️ **O oposto do [`DEFAULT_AO_STRENGTH`], e a assimetria é a wave inteira.**
/// O assado nasce em zero porque é um canal que **não existe** até alguém apertar
/// um botão, e ligá-lo por default faria a peça escurecer sozinha no instante do
/// primeiro bake. Este é medido **todo frame** a partir do que está na tela: ele
/// existe sempre que a forma existe, e nunca fica velho.
///
/// ⚠️ **E ele não muda nada até alguém MEDIR:** o barro lê a oclusão de um canal
/// que vale zero enquanto ninguém rodar o [`crate::MeshRenderer::render_ssao`] —
/// então este `1.0` é *"mostre o que foi medido"*, não *"escureça"*.
pub const DEFAULT_SSAO_STRENGTH: f32 = 1.0;

/// **Quanto do AMBIENTE COM DIREÇÃO entra por default: NADA.**
///
/// ⚠️ **Ele segue o [`DEFAULT_CAVITY`] e não o [`DEFAULT_SSAO_STRENGTH`], e eu
/// tinha escolhido o irmão errado.** A régua que eu apliquei foi *o canal
/// existe?* — e por ela o ambiente nasceria ligado, porque ele existe sempre que
/// existe uma normal. A régua CERTA é a que a cavidade já escreve uma constante
/// acima: *este canal muda a leitura de toda escultura que já foi feita?* A
/// cavidade nasce em zero porque *"o barro liso é o que a W3 entregou e o Enio
/// aprovou"*, e isto vale aqui palavra por palavra.
///
/// ⚠️ **E DOIS GATES DE GPU cobraram isso antes de qualquer humano ver**, o que
/// torna a escolha um fato e não um gosto: o
/// `the_two_lights_agree_where_the_form_turns_away` afirma que a luz do BARRO e a
/// da TINTA concordam onde a forma vira — a carta da `ph2d-light` em forma
/// executável —, e um piso direcional só no barro as separa. Ligá-lo por default
/// seria shipar essa divergência para todo mundo.
///
/// ⚠️ **Levantar o slider CONTINUA divergindo, e isso é aceito** — é a mesma
/// classe de escolher um MATCAP, que faz o barro acender por uma lei que a tinta
/// não tem e que ninguém chama de defeito: são modos de VISTA do viewport de
/// escultura. O que não se pode é entregá-la como o estado inicial.
///
/// ⚠️ **A adoção pela tinta é o follow-up, e o preço está medido:** o `channel`
/// dos dois passes do impasto (GPU e CPU) **não recebe a normal** — ela morre
/// upstream —, então levar o ambiente para lá é enfiar um parâmetro pela via
/// quente do Painter, com a paridade CPU/GPU pinada byte a byte. É uma wave do
/// dono daquele módulo, não um apêndice desta.
pub const DEFAULT_ENV: f32 = 0.0;

/// **AS CHAVES DOS NOMES DOS MATCAPS** — hoje uma re-exportação de
/// [`crate::matcap::MATCAP_NAME_KEYS`], que por sua vez é derivada da tabela.
///
/// ⚠️ **Elas são CHAVES desde 2026-09-17** e quem as resolve é o painel: esta
/// crate desenha pixels e não conhece a tabela de strings.
///
/// ⚠️ **Até 2026-08-10 esta era a lista, e o doc dela defendia a separação** —
/// *"os NÚMEROS ficam no WGSL e os NOMES aqui; o shader é o único consumidor"*.
/// A frase era verdade enquanto um matcap era um punhado de cores e expoentes;
/// hoje ele é uma IMAGEM, e o nome e os pixels moram no mesmo registro. O alias
/// fica porque o painel e o gate do shell já o importam por este caminho.
pub use crate::matcap::MATCAP_NAME_KEYS as MATCAPS;

/// ⭐⭐⭐⭐ **A LUZ COM QUE O APP ABRE — e ela é a LEI QUE ASSA.**
///
/// # O report que a trocou, e ele veio DUAS vezes
///
/// *«O que se vê no objeto 3d não é o que se vê na sprite cozida»* (o dono, 2026-09-20) e, depois de
/// o modo [`Lighting::Pbr`] existir e estar alcançável por um chip, **o mesmo report outra vez**:
/// *«o bake não é idêntico ao que se vê em 3d»*.
///
/// ⛔⛔ **A causa é ESTRUTURAL e não um epsilon:** o `BakedForm` da `ph2d-form-donation` não tem
/// campo de modo de luz **nem de material** — o bake corre SEMPRE a lei OpenPBR —, enquanto o visor
/// corre o que este valor disser. Com um MATCAP aqui, *o que se vê e o que se assa são duas leis
/// diferentes por construção*, e a paridade medida dentro do ramo `Pbr` não o pode ver.
///
/// **Medido** (`bake_light_pbr::mede_cada_modo_do_visor_contra_a_lei_que_assa`, a mesma forma e o
/// mesmo rig, desvio por canal contra a lei que assa):
///
/// ```text
///   Pbr        0,000215 medio / 0,000797 pior
///   Rig        0,088611          / 0,443359
///   Flat       0,301561          / 0,552313
///   Matcap(0)  0,347275          / 0,765497   <-- o que o app mostrava de fábrica
/// ```
///
/// `0,347` por canal é **1 615×** o resíduo do `Pbr` e **177×** a barra de meio código de oito bits.
///
/// ⚠️⚠️ **ISTO REVERTE UMA ORDEM ANTERIOR DO DONO, e ela fica escrita em vez de apagada:** em
/// 2026-08-09 ele mandou *«SculptGL: só tem um tipo; busque e coloque como o padrão do app»*, e o
/// índice `0` da tabela é o do SculptGL. Aquela ordem foi dada sobre *qual matcap* abre; esta troca
/// responde a uma pergunta diferente, que ele levantou depois e duas vezes — *o que se vê tem de ser
/// o que se assa*. ⛔ Uma palavra dele devolve o matcap a este sítio.
///
/// ⚠️ **O RECURSO foi medido antes da troca, porque uma lei mais pesada a correr sempre é um preço
/// que alguém paga** (`mede_o_preco_de_cada_modo`, 1600×900): os quatro modos leem `6,27`–`6,50 ms`
/// com a leitura de volta a dominar, e a diferença entre leis fica **abaixo do ruído** (`±0,14 ms`)
/// a `load 25` e a `load 40`. *Não há argumento de custo aqui* — e há de sobra do outro lado.
///
/// ⚠️ **Os dez matcaps continuam inteiros e a um clique** (a fileira *Material* do painel): eles são
/// a luz do OLHO e continuam a ser o que melhor lê FORMA enquanto se esculpe. O que mudou foi qual
/// deles nasce marcado — e por que razão: *este módulo existe para DOAR sombreamento a um sprite*
/// (ADR-0150), logo o valor de fábrica que mente sobre o produto é o defeito.
pub const DEFAULT_LIGHTING: Lighting = Lighting::Pbr;

/// ⭐⭐⭐⭐ **COM QUE LUZ o barro é mostrado** — os três modos, num tipo só.
///
/// ⛔⛔ **Ele substitui um `Option<u8>`, e a razão é o terceiro estado.** Até
/// 2026-09-20 «com que luz» tinha DUAS respostas (`None` = o rig do artista ·
/// `Some(i)` = o matcap `i`), e o report do dono pediu a terceira: *«precisamos
/// como no blender modos de shaders além do matcap para pintar»*. Um `bool flat`
/// ao lado do `Option` seria **dois campos que precisam concordar** — o defeito
/// que o doc do [`ShadeRaw::lighting`] já condena por escrito, e que admitiria
/// o estado sem sentido *«plano E matcap 3»*.
///
/// ⚠️ **E os três são o MESMO eixo, não três features:** cada um responde
/// *«de onde vem a luz?»* — de lado nenhum · das lâmpadas do documento · do
/// olho. É a separação que o Blender faz entre *Lighting* e *Color*, e o eixo
/// da COR desta casa é a cor por vértice, que desde a mesma data **sobrevive
/// aos três** (`mesh.wgsl`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lighting {
    /// **SEM LUZ** — o albedo cru (o barro × a cor pintada).
    ///
    /// ⭐ É o modo em que um pintor JULGA a cor: com sombreamento, a mesma tinta
    /// lê-se mais escura na parte escura da luz, e escolher cor assim é escolher
    /// contra a iluminação. ⚠️ **Com a peça por pintar ele mostra o barro
    /// CHAPADO** — uma silhueta sem forma —, e isso é a resposta certa e não um
    /// defeito: *não há forma sem luz*.
    Flat,
    /// **O RIG DO ARTISTA** — a luz do documento, a mesma que acende a tinta 2D
    /// ao lado (`ph2d-light`).
    #[default]
    Rig,
    /// ⭐⭐⭐⭐ **A LEI QUE ASSA** — o OpenPBR inteiro, com o material desta peça e o
    /// céu que este rig produz.
    ///
    /// ⚠️ **É a MESMA lei que acende o sprite depois do bake**, e é isso que ele
    /// existe para dar: *o que se vê é o que se assa*. O modo [`Self::Rig`] ao
    /// lado é um modelo de ARGILA — uma cor, um expoente e um brilho cravados no
    /// shader —, e duas leis sobre o mesmo objecto respondem diferente por
    /// construção.
    ///
    /// ⛔ **O que ele NÃO promete é o mesmo PIXEL:** a câmera do visor é
    /// perspectiva e o canvas é uma projecção 2D, logo a mesma peça ocupa outra
    /// forma no quadro. O que é igual é a **resposta da superfície** — dado o
    /// mesmo normal, a mesma luz e o mesmo material.
    Pbr,
    /// **A LUZ DO OLHO** — o matcap `i` de [`MATCAPS`], que é sombreamento
    /// função apenas da normal em espaço de vista.
    Matcap(u8),
}

impl Lighting {
    /// **QUE IMAGEM DE MATCAP ESTE MODO PRECISA** — a pergunta que a CPU faz
    /// para residir a textura, e a única que ela faz sobre este tipo.
    ///
    /// ⚠️ **Uma porta e não um `matches!` no sítio de uso:** o `ensure_matcap`
    /// e o painel perguntam o mesmo, e duas cópias divergiriam no dia do quarto
    /// modo.
    /// ⭐⭐ **TODOS os modos, derivados da tabela de matcaps** — a lista que os gates da ponte
    /// percorrem.
    ///
    /// ⚠️ **Ela mora na crate que declara o tipo e não no teste que a consome:** a versão escrita à
    /// mão do outro lado ficou com `2` fixos quando o terceiro chegou, e o gate da ponte acusou
    /// `12` contra `13`. *Uma lista escrita à mão ao lado de um enum é a segunda resposta a «quais
    /// são os modos», e ela envelhece na primeira wave que acrescenta um.*
    #[must_use]
    pub fn todos() -> Vec<Self> {
        let mut v = vec![Self::Flat, Self::Rig, Self::Pbr];
        v.extend((0..MATCAPS.len()).map(|i| Self::Matcap(u8::try_from(i).unwrap_or(u8::MAX))));
        v
    }

    #[must_use]
    pub const fn matcap_index(self) -> Option<u8> {
        match self {
            Self::Matcap(i) => Some(i),
            Self::Flat | Self::Rig | Self::Pbr => None,
        }
    }
}

/// **COMO O BARRO É MOSTRADO** — as opções de vista, num tipo só.
///
/// ⚠️ Um struct e não três argumentos soltos no [`crate::MeshRenderer::render`]:
/// ele já carrega nove, e cada opção de vista nova o empurraria mais. E o corte
/// é honesto — *quanto de cavidade*, *com que luz* e *com ou sem a malha* são as
/// três respostas à mesma pergunta, e elas viajam juntas do painel ao device.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shade {
    /// Quanto a curvatura escurece a fresta e clareia a crista.
    pub cavity: f32,
    /// **Com que luz** — ver [`Lighting`]. A conversão para a escada do uniform
    /// acontece **uma vez**, no [`ShadeRaw::pack`].
    pub lighting: Lighting,
    /// **Quanto do AO assado entra.** `0` = o barro sem oclusão, e é o default:
    /// um canal que nem foi assado não pode escurecer nada.
    ///
    /// ⚠️ **Ele NÃO é irmão da `cavity`, e a diferença decide o default.** A
    /// cavidade é derivada e existe em toda malha, então ligá-la por padrão é
    /// mostrar um dado que já está lá. O AO só existe depois de um BAKE
    /// explícito, e um default > 0 faria a peça não-assada renderizar igual (o
    /// canal é 1 em toda parte) até o instante do primeiro bake — quando ela
    /// escureceria sozinha, sem ninguém ter mexido num controle.
    pub ao: f32,
    /// **Quanto do AO de TELA entra.** `1` = todo ele, e é o default.
    ///
    /// ⚠️ Ele e o [`Self::ao`] respondem a mesma pergunta em ALCANCES
    /// diferentes: o assado mede metros de campo SDF e enxerga o corpo inteiro
    /// em qualquer direção; este mede um raio em torno do pixel e só vê o que
    /// está na tela. O barro os compõe pelo MENOS-OCLUÍDO (ver `mesh.wgsl`), e
    /// não pelo produto — dois canais que descrevem a mesma sombra multiplicados
    /// a escureceriam em dobro exatamente onde os dois acertam.
    pub ssao: f32,
    /// **O ESPALHAMENTO SUB-SUPERFICIAL** (`crate::sss`) — quanto, e até onde.
    ///
    /// ⚠️ Um struct aninhado e não dois `f32` soltos: `strength` e `scatter` são
    /// as duas metades de UMA resposta (*como este material conduz luz por
    /// dentro*), o [`crate::sss::SssParams`] é quem sabe semeá-las pelo tamanho
    /// da peça, e ele é a porta única de empacotamento.
    pub sss: crate::sss::SssParams,
    /// **QUANTO DO AMBIENTE COM DIREÇÃO entra** — `0` = o piso escalar de
    /// ontem, ao byte; `1` = o estúdio ([`ph2d_light::env_ambient`]).
    ///
    /// ⚠️ **Ele NÃO é um segundo AMBIENT, é o MESMO redistribuído:** a média
    /// sobre todas as normais continua sendo `ph2d_light::AMBIENT` (gate na
    /// crate dele), então subir este knob não clareia a peça — ele tira luz de
    /// baixo e põe em cima, que é o que separa *"o ambiente tem direção"* de
    /// *"a cena ficou mais clara"*.
    pub env: f32,
    /// ⭐⭐⭐ **DE QUE MATÉRIA A PEÇA É FEITA** — lido só pelo [`Lighting::Pbr`].
    ///
    /// ⚠️ **Ele viaja no `Shade`, que é *as opções de VISTA*, e isso é uma dívida
    /// DECLARADA e não um descuido:** hoje a escultura tem **um** barro para a
    /// cena inteira (o `CLAY` do shader é uma constante), logo um material por
    /// cena é exactamente a granularidade que existe. ⏳ No dia em que ele for
    /// **por peça** — que é o que o modelador já faz, com um
    /// `ph2d_field_ecs::FieldMaterial` por nó — ele muda de GRUPO: passa a viver
    /// no bind do objecto, ao lado da pose, porque a frequência dele deixa de ser
    /// a do quadro e passa a ser a do desenho.
    ///
    /// ⚠️ **Nos outros três modos ele não é lido** e o caminho fica byte-idêntico
    /// ao de antes deste campo existir — há gate.
    pub material: ph2d_material::OpenPbr,
    /// ⭐⭐⭐ **O OLHAR com que a cena linear vira ecrã** — exposição + curva de vista.
    ///
    /// ⚠️ **Ele é lido SÓ pelo [`Lighting::Pbr`]**, e a razão é que ele é o ÚLTIMO ACTO daquela lei:
    /// a [`ph2d_form_pbr::acende_texel`] recebe-o como argumento e devolve já display-referred. Os
    /// outros três modos escrevem uma resposta **relativa** (dividida pela de uma superfície plana
    /// sob a mesma luz), que não tem unidade de cena para expor.
    ///
    /// ⛔⛔ **A omissão é [`ph2d_view_transform::Look::default`] — neutra — e NÃO o olhar com que
    /// esta casa assa.** Esta crate desenha malhas e não sabe o que é um sprite; quem diz *«este
    /// visor mostra a lei que assa, com o olho com que ela assa»* é a APP, que escreve o
    /// `OLHAR_DA_FORMA` da `ph2d-form-donation` aqui — e há gate a exigi-lo. *Uma constante de
    /// produto escrita numa folha é a segunda resposta que ninguém sabe que existe.*
    pub look: ph2d_view_transform::Look,
    /// A malha desenhada por cima da forma.
    ///
    /// ⚠️ Ele viaja aqui e **não entra no [`ShadeRaw`]**: é um segundo PASSE, não
    /// um termo do sombreamento. Uma flag no uniform o faria parecer uma opção do
    /// fragment, e o dia em que alguém a lesse lá dentro o wireframe passaria a
    /// pintar as FACES.
    pub wireframe: bool,
}

impl Default for Shade {
    fn default() -> Self {
        Self {
            cavity: DEFAULT_CAVITY,
            env: DEFAULT_ENV,
            ao: DEFAULT_AO_STRENGTH,
            ssao: DEFAULT_SSAO_STRENGTH,
            sss: crate::sss::SssParams::default(),
            lighting: DEFAULT_LIGHTING,
            // ⚠️ **O OpenPBR de omissão, que é o mesmo que a lei do sprite usa** — é isso que faz o
            // visor e a sprite responderem a mesma coisa antes de existir um material autorável.
            material: ph2d_material::OpenPbr::default(),
            // ⚠️ **Neutro, e não o olhar da casa** — ver o doc do campo. Com exposição `0` e a vista
            // `Standard`, o `vt_to_display` devolve a entrada para luz dentro de `0..=1`: o modo
            // `Pbr` construído sem uma decisão mostra a CENA, que é honesto e é escuro.
            look: ph2d_view_transform::Look::default(),
            wireframe: false,
        }
    }
}

/// **A ESCADA DA LUZ, como o device a lê** — e ela é escrita **duas vezes de
/// propósito**: aqui e no `mesh.wgsl`, porque um uniform não partilha
/// constantes com Rust. ⚠️ O gate `a_escada_da_luz_concorda_com_o_shader` lê o
/// WGSL por [`include_str!`] e exige os mesmos três números — *duas cópias sem
/// gate divergem na primeira wave que acrescentar um modo*.
pub const LIGHTING_FLAT: u32 = 0;
/// Ver [`LIGHTING_FLAT`].
pub const LIGHTING_RIG: u32 = 1;
/// ⭐⭐⭐ Ver [`LIGHTING_FLAT`] — a lei que ASSA, no visor.
///
/// ⚠️ **Ele entrou no MEIO da escada e o `LIGHTING_FIRST_MATCAP` desceu de `2` para `3`.** ⭐ A
/// renumeração é de graça porque o [`Shade`] **não é serializado** — ele é estado de VISTA, e este
/// `u32` é só a codificação do uniform, refeita pelo [`ShadeRaw::pack`] a cada quadro. *Um número
/// que ninguém grava não tem compatibilidade para trás a pagar.*
pub const LIGHTING_PBR: u32 = 2;
/// Ver [`LIGHTING_FLAT`]. O matcap `i` é `LIGHTING_FIRST_MATCAP + i`.
pub const LIGHTING_FIRST_MATCAP: u32 = 3;

/// As opções, como o fragment shader as lê.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ShadeRaw {
    /// Quanto da cavidade entra. `0` = o barro liso da W3, **ao byte**.
    pub cavity: f32,
    /// **COM QUE LUZ**, na escada que o shader lê: `0` = [`Lighting::Flat`] ·
    /// `1` = [`Lighting::Rig`] · `2 + i` = o matcap `i`.
    ///
    /// ⚠️ A escada existe SÓ aqui, na fronteira do device: um uniform não tem
    /// enum, e a alternativa (um segundo `u32` dizendo *«tem matcap?»*) seriam
    /// dois campos que precisam concordar.
    ///
    /// ⛔⛔ **E o `0` MUDOU de significado em 2026-09-20** (era o rig): quem
    /// esquecer um sítio na conversão produz o modo **PLANO**, que é visível na
    /// primeira olhada — *uma escada nova cujo valor esquecido é silencioso é
    /// uma escada que ninguém conserta*. As três constantes vivem no
    /// `mesh.wgsl` ao lado do `if` que as lê.
    pub lighting: u32,
    /// Quanto do AO assado entra. `0` = byte-idêntico ao barro sem o canal.
    ///
    /// ⚠️ **Este é um dos dois `f32` que o `_pad` reservava** dizendo *"é aqui
    /// que SSS e AO vão pousar sem mexer no layout"*. A promessa foi cobrada e o
    /// layout não mudou — sobra um, e ele continua nomeando o SSS.
    pub ao: f32,
    /// Quanto do AO de TELA entra.
    ///
    /// ⚠️ **Este era o slot que o `_pad` guardava para o SSS.** A promessa foi
    /// cobrada por outro inquilino, e quando o SSS de fato chegou o `size_of`
    /// cresceu para 32 por alinhamento — exatamente o que este comentário
    /// anunciava, e o gate `the_uniform_grew_the_way_it_said_it_would` é quem
    /// cobra que o WGSL tenha crescido junto.
    pub ssao: f32,
    /// **Quanto do espalhamento sub-superficial entra**, `0..1`. `0` = o barro de
    /// sempre, **ao byte** — a tabela nem é consultada.
    pub sss_strength: f32,
    /// O `scatter` já dividido pelo teto da tabela ([`crate::sss::T_MAX`]) — a
    /// coordenada `v` sai de `|κ| ×` isto. A divisão mora no
    /// [`crate::sss::SssRaw::pack`], e só lá.
    pub sss_scale: f32,
    /// **O coeficiente da TRANSMITÂNCIA**: `1 / scatter`, para o shader escrever
    /// `exp(-espessura × isto × k_canal)`.
    ///
    /// ⚠️ **Ele ocupa um dos dois `_pad` que o comentário acima reservava** — e a
    /// promessa era exatamente esta: *"é aqui que o SSS pousa sem mexer no
    /// layout"*. O `size_of` fica em 32 B.
    ///
    /// ⚠️ **`1/scatter` e não `scatter`**, e a divisão mora numa porta só (o
    /// [`crate::sss::SssRaw::pack`]): o device multiplicando é mais barato que o
    /// device dividindo por frame, e — o que decide — `scatter = 0` vira um
    /// número **grande e finito** aqui, que é a leitura certa (*a luz não anda
    /// nada dentro deste material* ⇒ opaco), em vez de um `inf` que o
    /// interpolador transformaria em `NaN`.
    pub trans_scale: f32,
    /// **Quanto do ambiente com direção entra.** `0` = o piso escalar, ao byte.
    ///
    /// ⚠️ **Ele ocupa o ÚLTIMO `_pad`**, exatamente o que o comentário do
    /// `trans_scale` acima anunciava (*"sobra um"*). O `size_of` fica em 32 B e o
    /// gate de layout não se move — a terceira vez seguida que este uniform
    /// cresce sem crescer.
    pub env: f32,
}

impl Default for ShadeRaw {
    fn default() -> Self {
        Self::pack(Shade::default())
    }
}

impl ShadeRaw {
    /// Bytes do uniform. Constante, para o buffer nascer com o tamanho certo.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Empacota o que o artista escolheu, **clampado na porta**.
    ///
    /// ⚠️ O clamp mora aqui e não no chamador porque o device não tem opinião: um
    /// `cavity` de 3 faria `1 − 3k` ficar negativo numa fresta funda e o barro
    /// sairia com a cor invertida. Clampar no shader seria a segunda cópia da
    /// mesma regra, e ela divergiria no dia em que o painel chegasse.
    ///
    /// ⚠️ **E o índice do matcap é clampado pelo MESMO motivo:** um `Matcap(9)`
    /// numa tabela de seis cairia num material que não é o pedido, e o artista
    /// veria outra cera sem nada dizendo que o pedido era inválido.
    #[must_use]
    pub fn pack(shade: Shade) -> Self {
        let n = u8::try_from(MATCAPS.len()).unwrap_or(u8::MAX);
        Self {
            cavity: shade.cavity.clamp(0.0, 1.0),
            lighting: match shade.lighting {
                Lighting::Flat => LIGHTING_FLAT,
                Lighting::Rig => LIGHTING_RIG,
                Lighting::Pbr => LIGHTING_PBR,
                Lighting::Matcap(i) => LIGHTING_FIRST_MATCAP + u32::from(i.min(n - 1)),
            },
            // Clampado pela mesma razão da cavidade: o device não tem opinião, e
            // um `ao` de 3 faria o `mix` extrapolar para além do canal.
            ao: shade.ao.clamp(0.0, 1.0),
            ssao: shade.ssao.clamp(0.0, 1.0),
            // ⚠️ Pelo `SssRaw`, nunca à mão: é ele que sabe que o `scatter` viaja
            // dividido pelo teto da tabela, e uma segunda cópia dessa divisão
            // divergiria no dia em que o teto mudasse.
            sss_strength: crate::sss::SssRaw::pack(shade.sss).params[0],
            sss_scale: crate::sss::SssRaw::pack(shade.sss).params[1],
            trans_scale: crate::sss::SssRaw::pack(shade.sss).params[2],
            // Clampado como os vizinhos, e pelo mesmo motivo: o device não tem
            // opinião, e um `env` de 3 extrapolaria o `mix` para fora do
            // gradiente — a sombra de cima estouraria e a de baixo iria a
            // negativo.
            env: shade.env.clamp(0.0, 1.0),
        }
    }
}

#[cfg(test)]
#[path = "shade_tests.rs"]
mod tests;
