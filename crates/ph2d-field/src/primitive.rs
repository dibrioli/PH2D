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
}

/// O menor número de lados que um prisma admite — abaixo disto não há polígono.
pub const MIN_PRISM_SIDES: u32 = 3;

/// ⭐⭐⭐ **O TETO de lados de um prisma — e a medição REFUTOU a razão que eu ia escrever.**
///
/// # ⚠️ O erro, porque ele é instrutivo
///
/// A primeira redação deste doc dizia, com confiança: *«o custo **não** é o recurso — o preço por
/// ponto mal se mexe com os lados»*, e citava o `spike_formula_vs_profile`, que tinha medido `7,00×`
/// os nós a dar `1,21×` o relógio. ⛔ **É falso aqui.** A sonda
/// [`measure_prism_sides`](../../ph2d-field-eval/tests/measure_prism_sides.rs) mediu:
///
/// | lados | ns/ponto | × o cilindro | desvio da quina |
/// |---|---|---|---|
/// | 3 | 1,62 | **0,92×** | 50,00 % |
/// | 6 | 1,92 | 1,09× | 13,40 % |
/// | 12 | 2,74 | 1,56× | 3,41 % |
/// | 16 | 3,36 | 1,91× | 1,92 % |
/// | 24 | 4,62 | 2,62× | 0,86 % |
/// | **32** | 6,69 | **3,80×** | **0,48 %** |
/// | 64 | 13,11 | 7,43× | 0,12 % |
/// | 96 | 19,27 | 10,93× | 0,05 % |
///
/// ⚠️ **Porque a conclusão anterior não transferia:** ali a árvore era funda e o que custava era o
/// *caminho crítico*, que o SIMD escondia. Aqui as paredes são uma **cadeia de `max`** — o caminho
/// crítico cresce **linearmente** com `n`, e o relógio segue-o. *Uma recusa medida responde UMA
/// pergunta; reconfira-a quando a sua for outra.*
///
/// ⭐ **E o triângulo é MAIS BARATO que o cilindro** (`0,92×`): três planos não têm `sqrt` nenhum, e
/// a secção circular tem um. *A forma «simples» e a forma «barata» não são a mesma lista.*
///
/// # ⭐ O teto é onde as DUAS curvas dizem o mesmo
///
/// Um prisma de muitos lados **é** um cilindro, e este app tem o cilindro **exato e mais barato**. A
/// 32 lados a quina desvia `0,48 %` do raio — sub-pixel em qualquer enquadramento razoável — e
/// paga-se `3,71×` por isso. ⇒ acima de 32 o artista pede um cilindro, não o recebe, e paga a mais.
///
/// ⚠️ *Um limite legítimo diz de que recurso ele é* (CLAUDE.md §0). Este é dos **dois** ao mesmo
/// tempo, e é isso que o torna o sítio certo: a forma deixa de se distinguir exatamente onde o preço
/// começa a doer.
pub const MAX_PRISM_SIDES: u32 = 32;

/// O menor número de pontas de uma estrela — com duas não há ponta nenhuma, há uma lente.
pub const MIN_STAR_POINTS: u32 = 3;

/// ⭐⭐ **O TETO de pontas de uma estrela — e ele NÃO é o do prisma, porque uma ponta custa QUATRO
/// semiplanos.**
///
/// Uma estrela de `n` pontas é o disco dos vales unido a `n` pipas de quatro semiplanos cada —
/// `4n`, contra `n` de um prisma do mesmo número. A sonda
/// [`measure_star_points`](../../ph2d-field-eval/tests/measure_star_points.rs) mediu, com a **mesma
/// régua do prisma** (o cilindro exato = `1,00×`):
///
/// | pontas | semiplanos | nós | ns/ponto | × o cilindro |
/// |---|---|---|---|---|
/// | 3 | 12 | 88 | 2,78 | 1,28× |
/// | 5 | 20 | 137 | 3,50 | 1,61× |
/// | 8 | 32 | 191 | 4,64 | 2,13× |
/// | 12 | 48 | 288 | 6,42 | 2,95× |
/// | **16** | **64** | **357** | **7,96** | **3,66×** |
/// | 24 | 96 | 555 | 11,24 | 5,17× |
/// | 32 | 128 | 750 | 14,52 | 6,68× |
///
/// ⭐ **O número sai de um preço que este módulo já aceitou**, e não de um gosto: o
/// [`MAX_PRISM_SIDES`] shipa a `3,80×` o cilindro. A estrela chega a esse preço às **16** pontas
/// (`3,66×`) e passa-o às 24 (`5,17×`). ⇒ 16.
///
/// ⚠️ **E aqui o teto TIRA alguma coisa, ao contrário do prisma.** Um prisma de 64 lados é um
/// cilindro, e o cilindro exato está na porta ao lado — acima do teto o artista não perde nada. Uma
/// estrela de 24 pontas continua a ser uma estrela de 24 pontas, e não há segunda porta para ela.
/// *Um limite que retira tem de o dizer.*
pub const MAX_STAR_POINTS: u32 = 16;

/// O menor número de dentes que uma engrenagem admite — abaixo de três não há coroa.
pub const MIN_GEAR_TEETH: u32 = 3;

/// ⭐⭐⭐ **O TETO de dentes — MEDIDO, e o número está na tabela ao lado** (W106).
///
/// A sonda é [`measure_gear_teeth`](../../ph2d-field-eval/tests/measure_gear_teeth.rs), e a régua é
/// a mesma que escolheu o [`MAX_STAR_POINTS`]: o preço contra o **cilindro**, que é a referência
/// que o [`MAX_PRISM_SIDES`] usa e shipa a `3,80×`.
///
/// ⚠️ **A coluna que decide é a CONTAGEM DE NÓS**, não o relógio: ela é determinística, e um
/// relógio desta workstation não vale nada acima de `load ~5` (`CLAUDE.md` §5.0). O tempo aparece
/// ao lado como confirmação, pela mediana de cinco corridas.
///
/// | dentes | nós | ns/ponto | × o cilindro | × a ESTRELA no tecto dela |
/// |---|---|---|---|---|
/// | 6 | 160 | 5 489 | 3,37× | 0,40× |
/// | 8 | 192 | 6 718 | 4,13× | 0,50× |
/// | 12 | 300 | 10 021 | 6,16× | 0,74× |
/// | 16 | 390 | 12 324 | 7,57× | 0,91× |
/// | 24 | 586 | 18 526 | 11,38× | 1,37× |
/// | **32** | **741** | **23 473** | **14,42×** | **1,73×** |
/// | 48 | 1 155 | 48 467 | 29,78× | 3,57× |
/// | 64 | 1 482 | 65 407 | 40,20× | 4,82× |
///
/// *(referências medidas na MESMA corrida: cilindro `25` nós · prisma no tecto `308` · **estrela no
/// tecto `423` nós, `8,34×` o cilindro — a forma mais cara que esta casa shipa**.)*
///
/// # ⛔ Não há JOELHO na contagem de nós, e dizê-lo é o resultado
///
/// A contagem é **linear** de ponta a ponta: `26,7 · 24,0 · 25,0 · 24,4 · 24,4 · 23,2 · 24,1 ·
/// 23,2` nós por dente. ⇒ *não existe um número onde a física pare*, e um teto aqui é um **orçamento**
/// e não uma parede. Escrever «o joelho está em N» seria inventar uma medição que a tabela não deu.
///
/// A única não-linearidade é o **relógio** entre 32 e 48: `2,06×` o tempo para `1,5×` os dentes,
/// quando a contagem só sobe `1,56×`. ⚠️ É um sinal fraco (um relógio desta workstation não vale
/// nada acima de `load ~5`), e por isso ele **confirma** o número em vez de o escolher.
///
/// # Por que 32 e não 16
///
/// Aplicar a barra da estrela à letra daria **16** (`0,91×` dela). ⛔ Mas o doc do
/// [`MAX_STAR_POINTS`] escreve a própria regra: *«um limite que RETIRA tem de o dizer»* — e este
/// retira. Uma engrenagem de 24 ou 32 dentes é uma engrenagem comum; a 8 (que é onde ela custa o
/// que a estrela custa) ela mal se lê como uma. ⇒ o teto paga **`1,73×`** a forma mais cara da casa,
/// de propósito, porque *ter dentes é a razão de existir desta forma*.
///
/// ⚠️ **E o que este número mede é um LIMITE SUPERIOR:** o traçador especializa a fita por
/// ladrilho × fatia de profundidade, então um quadro real paga muito menos do que a árvore inteira.
/// Movê-lo pede a medição do **quadro** com uma cena cheia delas — que não foi feita, e é o que
/// desbloqueia um teto maior.
pub const MAX_GEAR_TEETH: u32 = 32;

/// ⭐⭐⭐ **A FAMÍLIA de uma primitiva, sem os números dela** (2026-08-26) — a lista que um gate pode
/// percorrer.
///
/// # ⛔ Por que ela nasceu
///
/// O gate `every_primitive_the_engine_can_make_has_a_button` promete, no próprio doc, que *«uma
/// primitiva nova aparece aqui **sozinha**, no dia em que nascer»*. ⚠️ **Não aparecia:** ele
/// percorria uma lista **escrita à mão** (*«uma de cada, construída à mão: é a enumeração que o
/// `Primitive` não oferece»*), e a contagem no fim só defendia a lista **de si mesma**. Um
/// `Primitive` novo compilava, o painel não lhe dava botão, e o gate ficava **verde** — que é
/// exactamente o defeito que a W53 pagou com uma **família de features inteira, completa e
/// invisível** (o `Extrude`/`Revolve` existiam desde a W3 sem nenhum botão a alcançá-los).
///
/// ⭐ **A corrente que fecha o buraco:** um `Primitive` novo é erro de compilação em
/// [`Primitive::kind`] ⇒ obriga uma variante nova aqui ⇒ [`PrimitiveKind::ALL`] é um array de
/// tamanho fixo, e não compila sem ela. *É a mesma corrente do [`crate::UnaryKind`], e ela existia
/// para os modificadores enquanto as formas ficavam com uma lista à mão.*
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveKind {
    Box,
    Sphere,
    Cylinder,
    Torus,
    Extrude,
    Revolve,
    Cone,
    Capsule,
    Prism,
    Wedge,
    TorusArc,
    Star,
    BoxFrame,
    Ellipsoid,
    Octahedron,
    RoundCone,
    CutSphere,
    HollowDome,
    Link,
    SolidAngle,
    Gear,
    Cross,
    Heart,
    Moon,
    Drop,
    Pie,
    Trapezoid,
    Vesica,
}

impl PrimitiveKind {
    /// **A fonte da contagem** — quem quiser saber *«que formas o motor sabe fazer?»* pergunta aqui.
    pub const ALL: [PrimitiveKind; 28] = [
        PrimitiveKind::Box,
        PrimitiveKind::Sphere,
        PrimitiveKind::Cylinder,
        PrimitiveKind::Torus,
        PrimitiveKind::Extrude,
        PrimitiveKind::Revolve,
        PrimitiveKind::Cone,
        PrimitiveKind::Capsule,
        PrimitiveKind::Prism,
        PrimitiveKind::Wedge,
        PrimitiveKind::TorusArc,
        PrimitiveKind::Star,
        PrimitiveKind::BoxFrame,
        PrimitiveKind::Ellipsoid,
        PrimitiveKind::Octahedron,
        PrimitiveKind::RoundCone,
        PrimitiveKind::CutSphere,
        PrimitiveKind::HollowDome,
        PrimitiveKind::Link,
        PrimitiveKind::SolidAngle,
        PrimitiveKind::Gear,
        PrimitiveKind::Cross,
        PrimitiveKind::Heart,
        PrimitiveKind::Moon,
        PrimitiveKind::Drop,
        PrimitiveKind::Pie,
        PrimitiveKind::Trapezoid,
        PrimitiveKind::Vesica,
    ];

    /// O sufixo da chave do botão que a cria — `panel.model3d.add.<key>`.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            PrimitiveKind::Box => "box",
            PrimitiveKind::Sphere => "sphere",
            PrimitiveKind::Cylinder => "cylinder",
            PrimitiveKind::Torus => "torus",
            PrimitiveKind::Extrude => "extrude",
            PrimitiveKind::Revolve => "revolve",
            PrimitiveKind::Cone => "cone",
            PrimitiveKind::Capsule => "capsule",
            PrimitiveKind::Prism => "prism",
            PrimitiveKind::Wedge => "wedge",
            PrimitiveKind::TorusArc => "torus_arc",
            PrimitiveKind::Star => "star",
            PrimitiveKind::BoxFrame => "box_frame",
            PrimitiveKind::Ellipsoid => "ellipsoid",
            PrimitiveKind::Octahedron => "octahedron",
            PrimitiveKind::RoundCone => "round_cone",
            PrimitiveKind::CutSphere => "cut_sphere",
            PrimitiveKind::HollowDome => "hollow_dome",
            PrimitiveKind::Link => "link",
            PrimitiveKind::SolidAngle => "solid_angle",
            PrimitiveKind::Gear => "gear",
            PrimitiveKind::Cross => "cross",
            PrimitiveKind::Heart => "heart",
            PrimitiveKind::Moon => "moon",
            PrimitiveKind::Drop => "drop",
            PrimitiveKind::Pie => "pie",
            PrimitiveKind::Trapezoid => "trapezoid",
            PrimitiveKind::Vesica => "vesica",
        }
    }
}

impl Primitive {
    /// A família desta forma. ⚠️ **O `match` é exaustivo, e é ele que fecha a corrente** — ver
    /// [`PrimitiveKind`].
    #[must_use]
    pub fn kind(&self) -> PrimitiveKind {
        match self {
            Primitive::Box { .. } => PrimitiveKind::Box,
            Primitive::Sphere { .. } => PrimitiveKind::Sphere,
            Primitive::Cylinder { .. } => PrimitiveKind::Cylinder,
            Primitive::Torus { .. } => PrimitiveKind::Torus,
            Primitive::Extrude { .. } => PrimitiveKind::Extrude,
            Primitive::Revolve { .. } => PrimitiveKind::Revolve,
            Primitive::Cone { .. } => PrimitiveKind::Cone,
            Primitive::Capsule { .. } => PrimitiveKind::Capsule,
            Primitive::Prism { .. } => PrimitiveKind::Prism,
            Primitive::Wedge { .. } => PrimitiveKind::Wedge,
            Primitive::TorusArc { .. } => PrimitiveKind::TorusArc,
            Primitive::Star { .. } => PrimitiveKind::Star,
            Primitive::BoxFrame { .. } => PrimitiveKind::BoxFrame,
            Primitive::Ellipsoid { .. } => PrimitiveKind::Ellipsoid,
            Primitive::Octahedron { .. } => PrimitiveKind::Octahedron,
            Primitive::RoundCone { .. } => PrimitiveKind::RoundCone,
            Primitive::CutSphere { .. } => PrimitiveKind::CutSphere,
            Primitive::HollowDome { .. } => PrimitiveKind::HollowDome,
            Primitive::Link { .. } => PrimitiveKind::Link,
            Primitive::SolidAngle { .. } => PrimitiveKind::SolidAngle,
            Primitive::Gear { .. } => PrimitiveKind::Gear,
            Primitive::Cross { .. } => PrimitiveKind::Cross,
            Primitive::Heart { .. } => PrimitiveKind::Heart,
            Primitive::Moon { .. } => PrimitiveKind::Moon,
            Primitive::Drop { .. } => PrimitiveKind::Drop,
            Primitive::Pie { .. } => PrimitiveKind::Pie,
            Primitive::Trapezoid { .. } => PrimitiveKind::Trapezoid,
            Primitive::Vesica { .. } => PrimitiveKind::Vesica,
        }
    }
}
