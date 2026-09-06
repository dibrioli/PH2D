//! ⭐ **O QUE UMA FORMA É** — o [`Primitive`], a família dele ([`PrimitiveKind`]) e os tetos que
//! cada contagem admite.
//!
//! # Por que ele saiu do `lib.rs`
//!
//! O `lib.rs` desta crate responde por **três** coisas: o que uma forma é, como uma árvore se monta
//! (`Node`/`NodeKind`/`Op`/`FieldDoc`) e o que o documento **recusa**. A W103/W104 acrescentaram
//! cinco primitivas e as notas medidas de cada uma, e o arquivo passou dos **700** que o gate de LOC
//! da workspace fixa. ⚠️ **A cura é partir para irmão, nunca uma entrada na allowlist** — ela existe
//! para os que já estavam acima em 2026-06-20, e o objetivo dela é descer.
//!
//! ⚠️ O `pub use` no [`super`] mantém `ph2d_field::Primitive` — cortar um arquivo não pode custar
//! uma reescrita em cada sítio que o chamava.

use crate::Profile;
use serde::{Deserialize, Serialize};

/// As primitivas. Cada uma é **distância exata** — dentro e fora.
///
/// ⚠️ O `round` de uma primitiva é o **arredondamento da aresta convexa** dela, e ele é feito por
/// **deslocamento da superfície** com a fonte encolhida na mesma medida (ADR-0161 §3). É por isso
/// que ele vive na primitiva e não numa operação: arredondar a aresta de uma caixa não envolve
/// segunda forma nenhuma.
// ⚠️ **Sem `Copy` desde a v2**, e a razão é `Extrude`: um perfil é uma lista de pontos, e um tipo
// que se copia por bit não pode conter um `Vec`. A alternativa — pôr os perfis numa segunda arena e
// referi-los por índice — foi recusada: ela mantinha o `Copy` e comprava, em troca, uma segunda
// classe inteira de erro (índice pendente), que é exatamente o que a arena de nós existe para
// tornar impossível. Uma invariante, um lugar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Primitive {
    /// Caixa de meias-extensões `half`, com as 12 arestas arredondadas em `round`.
    Box {
        half: [f32; 3],
        round: f32,
        chamfer: f32,
    },
    /// Esfera. Não tem aresta, logo não tem `round`.
    Sphere { radius: f32 },
    /// Cilindro no eixo **Z** (outro eixo se obtém pela rotação do nó), com o aro das tampas
    /// arredondado em `round`.
    Cylinder {
        radius: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// Toro no plano XY: `major` é o raio do anel, `minor` a espessura do tubo.
    Torus { major: f32, minor: f32 },
    /// **O perfil puxado ao longo de Z**, de `−half_height` a `+half_height`, com o **aro** (a
    /// aresta entre a parede e a tampa) arredondado em `round`.
    ///
    /// ⚠️ As arestas **verticais** — as quinas do próprio contorno — não são assunto deste `round`:
    /// elas são o que o perfil desenhou. Quem as quer redondas arredonda a quina **no editor
    /// vetorial**, e o raio vivo de lá chega aqui já cozido. *Uma quina, um dono.*
    Extrude {
        profile: Profile,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **O perfil girado em torno do eixo Y.** O `x` do perfil é a distância ao eixo e o `y` é a
    /// altura.
    ///
    /// ⚠️ **Y, e não Z — de propósito, e ao contrário do [`Primitive::Cylinder`] e do
    /// [`Primitive::Torus`], que são simétricos em Z.** A regra que manda aqui não é a coerência
    /// entre primitivas, é a coerência com o **plano de desenho**: o perfil vem do editor vetorial,
    /// que desenha em XY, e o eixo de uma revolução tem de estar **dentro** do plano do perfil. A
    /// extrusão sai do plano (por Z), a revolução gira em torno de uma reta do plano (o Y). Quem
    /// quiser outro eixo roda o nó — é para isso que o [`Xform`] existe.
    ///
    /// ⚠️ O perfil **não pode cruzar o eixo** (`x < 0`): a superfície de revolução de um contorno
    /// que cruza o eixo auto-intersecta, e o campo que sai disso deixa de ser uma distância. O
    /// documento recusa ([`FieldError::ProfileCrossesAxis`]) em vez de produzir a forma errada.
    Revolve { profile: Profile },
    /// ⭐⭐ **Cone reto no eixo Z, possivelmente TRUNCADO** (W101) — raio `bottom` em
    /// `−half_height`, `top` em `+half_height`, com o **aro** arredondado em `round`.
    ///
    /// ⚠️ **Um cone e um tronco de cone são a MESMA forma**, e por isso são a mesma primitiva:
    /// `top = 0` fecha num ápice. Duas variantes dariam duas fórmulas para a mesma superfície — e
    /// a segunda é a que envelhece. A paleta oferece as **duas portas**, com defaults diferentes;
    /// o artista converte uma na outra arrastando um número.
    ///
    /// ⚠️ **Z, como o [`Primitive::Cylinder`]** — um cone é o cilindro cuja parede inclina, e um
    /// eixo diferente faria a mesma peça mudar de orientação ao trocar de forma.
    Cone {
        bottom: f32,
        top: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **Cápsula no eixo Z** — o segmento de `−half_height` a `+half_height` engrossado em
    /// `radius`.
    ///
    /// ⚠️ **Não tem `round`, e a ausência é a forma**: ela já é o cilindro com as tampas
    /// arredondadas ao máximo possível, e um segundo raio não teria onde agir. É a mesma razão da
    /// [`Primitive::Sphere`].
    Capsule { radius: f32, half_height: f32 },
    /// ⭐⭐ **Prisma regular de `sides` lados no eixo Z, possivelmente ESTREITADO** — `bottom` é o
    /// **circunraio** em `−half_height` e `top` o de `+half_height`, com o aro arredondado em
    /// `round`.
    ///
    /// ⚠️ **Circunraio e não apótema**, e a escolha tem consequência visível: com o circunraio, um
    /// prisma de `n` lados **inscreve-se** no cilindro do mesmo raio, então subir os lados converge
    /// para ele por dentro. Com o apótema convergiria por fora, e trocar um cilindro por um prisma
    /// faria a peça **crescer**.
    ///
    /// ⭐⭐⭐ **É o irmão POLIGONAL do [`Primitive::Cone`], e a simetria é a feature** (W102): com
    /// `top == bottom` é o prisma de sempre, com `top == 0` é uma **pirâmide**, e no meio é um
    /// **tronco de pirâmide**. Uma primitiva à parte para a pirâmide daria uma segunda fórmula para
    /// a mesma superfície — e a segunda é a que envelhece.
    Prism {
        sides: u32,
        bottom: f32,
        top: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐ **Cunha: uma caixa cortada por um plano inclinado** — cheia em `−x`, a zero em `+x`.
    ///
    /// ⚠️ **Não é composição, e o motivo é a ausência de um PLANO**: cortar uma caixa com outra
    /// caixa gigante rodada dá a forma certa e deixa na peça um objecto que não é a peça, com um
    /// tamanho que não quer dizer nada. *Uma equivalência que exige uma terceira entidade não é uma
    /// equivalência.*
    ///
    /// ⭐ O plano do corte passa pela **origem** — ele liga `(−hx, +hz)` a `(+hx, −hz)`, e o ponto
    /// médio desses dois é o centro do nó.
    Wedge {
        half: [f32; 3],
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐ **Arco de toro no plano XY** — o toro cortado a `angle` radianos, centrado no `+X`.
    ///
    /// ⚠️ `angle >= 2π` é o toro inteiro, e o corte **não é construído** (não há dois semiplanos que
    /// exprimam «tudo»). Abaixo disso o sector é a interseção de dois semiplanos até `π` e a
    /// **união** deles acima — uma escolha feita ao MONTAR a árvore, em Rust, e não com uma
    /// ramificação dentro do campo (ver `ops::sd_torus_arc`).
    TorusArc {
        major: f32,
        minor: f32,
        angle: f32,
        /// ⭐ **Os dois aros do corte** (W104) — a aresta entre a face cortada e o tubo.
        ///
        /// ⚠️ Ela **não existia** até à W104, e a ausência não era uma decisão: a sonda de arestas
        /// mediu `30 %` da superfície deste arco sobre um vinco de `88°`, e esta era a única forma
        /// do catálogo com aresta autorada e **sem o slider que a trata**.
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐⭐ **Estrela de `points` pontas puxada em Z** (W103) — `outer` é o raio das PONTAS e
    /// `inner` o dos VALES.
    ///
    /// ⚠️ **É a única da fila que a composição não exprime, e o motivo é a PARIDADE.** Uma estrela
    /// de 6 pontas é a união de dois triângulos, e uma de 4 é a de dois losangos — mas uma de **5**
    /// não é a união de polígono nenhum, porque nenhum divisor de 5 dá um polígono regular
    /// rodado. *Uma equivalência que só vale para metade dos valores de um controlo não é uma
    /// equivalência: é uma armadilha à espera do número ímpar.*
    ///
    /// ⭐ **O interior é uma UNIÃO de peças convexas** (o polígono dos vales + uma língua por
    /// ponta), e não uma interseção: uma estrela é **não-convexa** por definição, e a lei das
    /// meias-fatias da W101 só constrói convexos. Ver `ops::sd_star`.
    Star {
        points: u32,
        outer: f32,
        inner: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐ **A GAIOLA de uma caixa** (W103) — as 12 arestas com secção quadrada de lado
    /// `thickness`, e o miolo vazio.
    ///
    /// ⚠️ **Faz-se por composição — com QUATRO objectos** (a caixa menos três caixas atravessadas)
    /// — e é por isso que ela é uma primitiva: a forma é **uma**, e uma peça em que ela ocupa
    /// quatro linhas da Hierarquia obriga o artista a mexer em três números para engrossar uma
    /// aresta. *Compor é a resposta certa quando a composição é o que o artista pensa; aqui ele
    /// pensa «moldura».*
    BoxFrame {
        half: [f32; 3],
        thickness: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐⭐ **Elipsóide de semi-eixos `radii`** (W103).
    ///
    /// ⚠️ **A nota que dizia «não há como achatar, a escala do módulo é uniforme de propósito»
    /// respondia a OUTRA pergunta.** Ela é sobre o [`Xform::scale`], e ali continua certa: uma pose
    /// com escala por eixo estragaria `‖∇f‖ = 1` em toda a árvore abaixo dela. Uma **primitiva** com
    /// três raios não toca nisso — ela é uma folha, e a folha responde por si.
    ///
    /// ⚠️ **Não substitui a [`Primitive::Sphere`], e a diferença é a QUALIDADE DO CAMPO**, não o
    /// número de controlos: a esfera é distância **exata**, e este é um subestimador (a distância
    /// exacta a um elipsóide resolve uma quártica — é por isso que a referência publica
    /// aproximações). Duas linhas do catálogo para a mesma forma justificam-se quando uma delas é
    /// exacta; a do cone e do tronco justificavam-se por defaults.
    Ellipsoid { radii: [f32; 3] },

    // ─────────────────────────────────────────────────────────────────────────────────────────
    // ⭐⭐⭐ **W106 — as catorze que a fila nunca contou.**
    //
    // ⛔ A auditoria de 29/08 fechou a fila contra as **47 formas do catálogo vetorial** e nunca
    // leu a segunda lista do mesmo documento — *«as 3D que o catálogo vetorial nem podia ter»*.
    // E o argumento que cortou as outras (*«já se faz por composição»*) responde *«o motor
    // consegue?»*, não *«a pessoa ACHA?»*: uma forma que exige montagem não está no menu.
    //
    // O mecanismo de cada fórmula, com o que foi portado e o que foi recusado, vive em
    // `ph2d_field_eval::ops_solids` e `::ops_plates`.
    // ─────────────────────────────────────────────────────────────────────────────────────────
    /// **Octaedro regular** — `radius` é o CIRCUNRAIO (centro a vértice), como no prisma.
    Octahedron {
        radius: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Cone de pontas arredondadas** — o casco convexo de duas esferas, no eixo Z.
    ///
    /// ⚠️ Sem `round`, como a cápsula: já é todo arco. E `|bottom − top| < 2·half_height` é
    /// obrigatório — acima disso uma esfera contém a outra e não há tangente comum.
    RoundCone {
        bottom: f32,
        top: f32,
        half_height: f32,
    },
    /// **Esfera cortada** por um plano em `z = cut` — uma cúpula, um botão.
    CutSphere {
        radius: f32,
        cut: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Cúpula oca** — a casca de raio médio `radius` e espessura `thickness`, cortada em `cut`.
    ///
    /// ⚠️ **Não é a [`Primitive::CutSphere`] menos outra:** seriam duas entidades e dois raios que
    /// têm de concordar. A mesma razão da [`Primitive::BoxFrame`].
    HollowDome {
        radius: f32,
        cut: f32,
        thickness: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Elo de corrente** — o toro esticado: o eixo é um estádio de raio `major` com dois trechos
    /// rectos de `length` para cada lado, engrossado em `minor`.
    Link { major: f32, minor: f32, length: f32 },
    /// **Ângulo sólido** — a fatia cónica de uma esfera, meia-abertura `angle` em torno de `+Z`.
    SolidAngle {
        radius: f32,
        angle: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐⭐ **Engrenagem** — a forma que o Enio nomeou, e que tinha sido cortada da fila por ser
    /// *«um dente mais o modificador radial»*.
    ///
    /// `root` é o corpo, `outer` a ponta do dente, `tooth` a fração do passo que o dente ocupa.
    /// ⚠️ O flanco é **recto**, não uma evolvente: para desenhar chega, para transmitir binário não.
    Gear {
        teeth: u32,
        root: f32,
        outer: f32,
        tooth: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Cruz / mais** — `arm` é o meio-comprimento do braço e `width` a meia-largura dele.
    Cross {
        arm: f32,
        width: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Coração** — `size` é o meio-lado do losango que forma a ponta de baixo.
    Heart {
        size: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Lua / crescente** — o disco `radius` menos o disco `bite` deslocado de `offset` em `+X`.
    Moon {
        radius: f32,
        bite: f32,
        offset: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Gota** — o disco `radius` com uma ponta tangente a `height` acima do centro.
    Drop {
        radius: f32,
        height: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Fatia de disco** — `radius` e a meia-abertura `angle`, centrada em `+Y`.
    Pie {
        radius: f32,
        angle: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Trapézio** — `bottom` e `top` são as meias-larguras das duas bases.
    ///
    /// ⚠️ Não é o prisma de 4 lados estreitado: aquele estreita nos **dois** eixos.
    Trapezoid {
        bottom: f32,
        top: f32,
        half_width: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// **Vesica / lente** — a interseção de dois discos `radius` afastados de `2·offset`.
    Vesica {
        radius: f32,
        offset: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐ **SETA no eixo +X, com UMA ponta ou DUAS** (W119) — a haste de meia-espessura `shaft`
    /// unida a uma ponta de meia-largura `head` e comprimento `head_length`.
    ///
    /// ⚠️ **Uma seta e uma seta dupla são a MESMA forma**, e por isso são a mesma primitiva: com
    /// `heads = 2` o contorno é dobrado por `|x|` e a segunda ponta sai de graça. Duas variantes
    /// dariam duas fórmulas para a mesma superfície — a lei do [`Primitive::Cone`], e a segunda é a
    /// que envelhece. ⛔ **E ela não é «um `Mirror` sobre uma seta»**: o critério de entrada de uma
    /// paleta é o ALCANCE, e uma forma que exige montagem é uma forma que não está no menu.
    ///
    /// ⚠️ `head` tem de ser **maior** que `shaft`, senão não há farpa e a peça é um retângulo com um
    /// bico — o documento recusa.
    Arrow {
        heads: u32,
        half_length: f32,
        shaft: f32,
        head: f32,
        head_length: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **CHEVRON** — a faixa em «V» que aponta a `+X`, de espessura perpendicular `thickness`.
    ///
    /// ⚠️ **Não é a [`Primitive::Arrow`] sem haste**: uma seta é um sólido cheio e um chevron é uma
    /// **banda** — a diferença de duas cunhas paralelas. O interior dele é vazio, e é isso que faz
    /// dele o símbolo que se empilha.
    Chevron {
        half_length: f32,
        half_span: f32,
        thickness: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **SETA DOBRADA** — a haste sobe em «L» (de `−X` até `+X`, depois até `+Y`) e acaba numa
    /// ponta virada a `+Y`.
    ///
    /// ⚠️ **O cotovelo é a razão de ela ser uma primitiva**: por composição são três objectos cuja
    /// espessura tem de concordar, e engrossar a haste passaria a ser mexer em três números.
    BentArrow {
        run: f32,
        rise: f32,
        shaft: f32,
        head: f32,
        head_length: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **LOSANGO** — a chapa de diagonais `2·half_width` (em X) e `2·half_span` (em Y).
    ///
    /// ⚠️ **Não é o prisma de 4 lados**: aquele tem as duas diagonais IGUAIS (o circunraio é um
    /// número só), e o losango do fluxograma é largo e baixo. *Uma forma que só se alcança com as
    /// duas diagonais iguais não é a forma que o catálogo pede.*
    Rhombus {
        half_width: f32,
        half_span: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐ **TUBO / anel — a coroa circular puxada em Z, com SECTOR opcional** (W119).
    ///
    /// `outer` e `inner` são os dois raios; `angle` é a **meia-abertura** do sector, e em `π` (o
    /// nascimento do tubo e da anilha) o corte **não existe** — o anel fecha.
    ///
    /// ⚠️ **`inner > 0` é obrigatório, e a cerca é o que impede a segunda fórmula**: sem furo isto
    /// seria a [`Primitive::Pie`], e duas primitivas para a mesma superfície é o defeito que a
    /// [`Primitive::Cone`] evita desde a W101. *Um tubo tem furo por definição; sem furo é uma
    /// fatia.*
    ///
    /// ⚠️ **Três portas da paleta, uma primitiva** — tubo (alto), anilha (chato) e arco de anel
    /// (com sector) diferem só nos números com que nascem.
    Tube {
        outer: f32,
        inner: f32,
        angle: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **SEGMENTO DE CÍRCULO** — o disco `radius` cortado pela **corda** `y = cut`.
    ///
    /// ⚠️ **Não é a [`Primitive::Pie`]**: uma fatia é limitada por dois RAIOS e converge num ápice;
    /// um segmento é limitado por uma corda e tem duas quinas. ⚠️ E não é a [`Primitive::Moon`],
    /// que subtrai um disco.
    CircleSegment {
        radius: f32,
        cut: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **BALÃO RETANGULAR** — o corpo `2·half_width × 2·half_span` com a cauda a sair da base.
    SpeechRect {
        half_width: f32,
        half_span: f32,
        tail: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **BALÃO OVAL** — o mesmo, com o corpo redondo.
    ///
    /// ⚠️ **Não é o retangular com o filete no máximo:** aquele satura numa cápsula (dois
    /// semicírculos e dois lados retos), e um oval não tem lado reto nenhum.
    SpeechOval {
        half_width: f32,
        half_span: f32,
        tail: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐ **NUVEM — e o BALÃO DE PENSAMENTO é ela com `tail > 0`.**
    ///
    /// ⚠️ **Uma nuvem e um balão de pensamento são a MESMA forma**, e por isso são a mesma
    /// primitiva: o segundo é o primeiro com uma fieira de bolhas a descer. É a lei do
    /// [`Primitive::Cone`], e `tail = 0` é o estado que dá nome à porta *Cloud*.
    Cloud {
        lobes: u32,
        half_width: f32,
        half_span: f32,
        tail: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **RAIO** — o zigue-zague, dois triângulos que se cruzam numa banda.
    ///
    /// ⚠️ **A proporção do zigue-zague é a IDENTIDADE da forma**, e por isso não tem knob: os dois
    /// números dizem o tamanho, e um terceiro só a descaracterizava.
    Bolt {
        half_width: f32,
        half_span: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **ESCUDO** — topo reto e dois arcos que fecham numa ponta em baixo.
    ///
    /// ⚠️ **`2·half_span > half_width` é uma cerca de GEOMETRIA, não de gosto**: abaixo dela o
    /// centro do arco cai do outro lado e os lados curvam para dentro.
    Shield {
        half_width: f32,
        half_span: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **ETIQUETA** — o retângulo que afila numa ponta a `+X`, com o furo do cordel.
    Tag {
        half_width: f32,
        half_span: f32,
        point: f32,
        hole: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **VISTO** — duas faixas de espessura `thickness` que se cruzam no vértice de baixo.
    Check {
        half_width: f32,
        half_span: f32,
        thickness: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **FAIXA / fita** — o retângulo com um entalhe em «V» em cada ponta.
    Banner {
        half_width: f32,
        half_span: f32,
        notch: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **CHAVE `{`** — quatro quartos de arco.
    ///
    /// ⚠️ **Sem controle de largura, e é uma decisão**: o raio é `half_span/2` e o alcance sai dele.
    /// Uma chave mais larga do que alta deixa de se ler como uma chave, e um knob que só a estraga é
    /// um knob a menos.
    Brace {
        half_span: f32,
        thickness: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    // ─────────────────────────── W122 — o fluxograma ───────────────────────────
    /// ⭐ **PARALELOGRAMO** (*Data*) — o retângulo inclinado.
    ///
    /// `skew` é o quanto a base de cima escorrega em `+X`, e é a única grandeza desta família que
    /// aceita **negativo e zero**: zero dá o retângulo ao bit, e o sinal escolhe o lado da queda.
    Parallelogram {
        half_width: f32,
        half_span: f32,
        skew: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **ATRASO** (*Delay*) — a face esquerda reta e a direita num semicírculo inteiro.
    ///
    /// ⚠️ **`half_width >= half_span` é a cerca**: a tampa direita é um semicírculo de raio
    /// `half_span`, e abaixo disso o centro dela passa para a esquerda da parede.
    Delay {
        half_width: f32,
        half_span: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **MOSTRADOR** (*Display*) — o atraso com a esquerda a fechar num bico de comprimento
    /// `point`.
    Display {
        half_width: f32,
        half_span: f32,
        point: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐ **CONECTOR DE PÁGINA** (*Off-page*) — o retângulo que fecha num bico de profundidade
    /// `point` em baixo.
    OffPage {
        half_width: f32,
        half_span: f32,
        point: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    // ─────────────────────────── W123 — as duas que estavam «desenhadas» ───────────────────────
    /// ⭐⭐⭐ **ESPIRAL de Arquimedes** — a fita que começa em `radius`, cresce `pitch` por volta e
    /// dá `turns` voltas, com meia-espessura `thickness`.
    ///
    /// ⚠️ **Estava declarada «fica desenhada»**, e a recusa respondia a outra pergunta: o que não é
    /// fechado é a **distância exacta**, e a marcha só precisa de um minorante. Ver
    /// [`ph2d_field_eval::ops_spiral`].
    Spiral {
        radius: f32,
        pitch: f32,
        turns: f32,
        thickness: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐ **DOCUMENTO** (*Document*) — o retângulo cuja base é uma onda de amplitude `wave`.
    ///
    /// ⚠️ **A superfície é a senóide, ao bit**; o que é conservador é só a distância longe dela.
    Document {
        half_width: f32,
        half_span: f32,
        wave: f32,
        half_height: f32,
        round: f32,
        chamfer: f32,
    },
    // ─────────────────────────── W124 — a mola e a rede ───────────────────────────
    /// ⭐⭐⭐ **MOLA / HÉLICE** — o tubo de raio `thickness` que dá `turns` voltas num cilindro de
    /// raio `radius`, subindo `pitch` por volta.
    ///
    /// ⚠️ **Não é uma chapa**: ela não tem `half_height`, porque a altura dela é
    /// `pitch × turns` — um número que já existe.
    Helix {
        radius: f32,
        pitch: f32,
        turns: f32,
        thickness: f32,
        round: f32,
        chamfer: f32,
    },
    /// ⭐⭐⭐ **GYROID** — a parede de meia-espessura `thickness` em volta da superfície de Schoen,
    /// com célula `cell`, recortada pela caixa `half`.
    ///
    /// ⚠️ **A distância a ela não tem forma fechada, e o módulo não precisa dela** — ver
    /// [`ph2d_field_eval::ops_lattice`] para o divisor **medido**.
    Gyroid {
        half: [f32; 3],
        cell: f32,
        thickness: f32,
        round: f32,
        chamfer: f32,
    },
    // ─────────────────────────── W125 ───────────────────────────
    /// ⭐ **CILINDRO COM BARRIGA** — o bordo inteiro num arco de raio `bulge`.
    ///
    /// ⚠️ **Não tem `round`, e é uma decisão**: o `bulge` já É o arredondamento, e um segundo
    /// número para a mesma aresta seriam duas verdades sobre ela.
    RoundedCylinder {
        radius: f32,
        bulge: f32,
        half_height: f32,
    },
    // ─────────────────────────── W127 ───────────────────────────
    /// ⭐⭐⭐ **SUPERQUADRÁTICA** — a bola da norma-`n` encaixada: *um* knob atravessa a família
    /// inteira.
    ///
    /// `exponent_top` governa o que se vê **de cima** (`1` losango · `2` círculo · alto quadrado) e
    /// `exponent_side` o que se vê **de lado** (`1` bipirâmide · `2` elipse · alto prisma). A esfera
    /// é `2` nos dois, e nesse ponto o campo é a distância **exacta**.
    ///
    /// ⚠️ **O eixo de cima desta casa é o `Y`** (a `half[1]` chama-se *Height* em toda forma de
    /// caixa), então o *de cima* é o par `X–Z` e o *de lado* é o `Y`.
    ///
    /// ⚠️ **Não tem `round` nem `chamfer`, e é a mesma decisão do [`Primitive::RoundedCylinder`]**:
    /// o expoente já É o arredondamento desta forma.
    Superquadric {
        half: [f32; 3],
        exponent_top: f32,
        exponent_side: f32,
    },
}
