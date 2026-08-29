//! `ph2d-field` — **o documento** do módulo de modelagem 3D ([ADR-0161]).
//!
//! O modelo não é uma malha nem um grid de voxels: é uma **árvore de expressão autorada**.
//! Primitivas, transformações e operações com raio. Perguntar *"esta forma existe no ponto p?"* é
//! avaliar `f(p)` — e é dessa escolha que decorrem, como consequência e não como promessa:
//!
//! - **booleana não pode falhar** (união é `min(a, b)`: não existe geometria degenerada para uma
//!   comparação de dois números);
//! - **o arredondamento não pode falhar**, e funciona onde três ou mais formas se encontram — o
//!   caso que quebra o `Bevel` do Blender e o rolling-ball do CAD;
//! - **o raio fica editável para sempre**, porque é parâmetro da operação e não geometria assada.
//!   ⭐ Nem o Blender nem o MoI dão isto.
//!
//! # ⚠️ Esta crate NÃO avalia
//!
//! Nenhuma linha aqui nomeia o motor de avaliação. Ele vive na `ph2d-field-eval`, e a fronteira é a
//! razão de existir desta crate: trocar de motor tem de ser trabalho de **uma** crate, e — o que
//! importa mais — **nenhum arquivo salvo pode quebrar** quando isso acontecer. O documento do
//! utilizador não pode ter a forma que um terceiro escolheu para a estrutura interna dele.
//!
//! # A arena é ORDENADA POR CONSTRUÇÃO
//!
//! Os nós vivem num `Vec` e referem-se por índice. A invariante é dura: **todo filho tem índice
//! estritamente menor que o do pai**. Isso não é estilo — é o que torna ciclo uma
//! **impossibilidade** em vez de um erro a detectar, e faz a avaliação ser uma passagem de baixo
//! para cima sem recursão nem pilha de visitados.
//!
//! [ADR-0161]: ../../../docs/architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md

pub mod blend;
pub mod dims;
pub mod mods;
pub mod profile;
pub mod radius;
pub mod xform;

pub use blend::{Blend, Character};
pub use dims::{Dim, Param, Span, clamp_round, dims, scale_primitive, set_dim};
pub use mods::{Unary, UnaryKind};
pub use profile::{
    DEFAULT_PROFILE_RESOLUTION, FillRule, MAX_PROFILE_RESOLUTION, Profile, ProfileError, coarsen,
    coarsen_to_normal_error,
};
pub use radius::{
    Bound, bounding_radius, characteristic_size, fillet_inflates, round_limit, set_shape_radius,
};
pub use xform::Xform;

use serde::{Deserialize, Serialize};

/// Versão do formato serializado (**HR-14**: save format é versionado e migrável).
///
/// ⚠️ **Este número SOMA entre linhas** — se duas o incrementarem em paralelo, o git funde os dois
/// lados sem saber que são o mesmo degrau. Ao mexer, **conte**, não escolha
/// ([`CLAUDE.md §5.0`]).
///
/// v2: as primitivas de **perfil** ([`Primitive::Extrude`] / [`Primitive::Revolve`]) — o desenho do
/// editor vetorial virando sólido.
///
/// v3: o [`Node`] ganhou a pilha de **modificadores** ([`mods::Unary`] — casca, afastamento,
/// espelho e matriz). É
/// campo novo numa struct, e postcard é **posicional**, então um documento v2 não desserializa aqui.
/// ⚠️ **A migração é vazia, e isso tem de estar escrito:** nada persiste um [`FieldDoc`] — ele é
/// **cozido** da cena a cada quadro, e o que o arquivo de projeto guarda são os *componentes* ECS.
/// O degrau sobe na mesma, porque a alternativa é o número deixar de querer dizer alguma coisa no
/// dia em que alguém o persistir.
///
/// v4: o [`NodeKind`] ganhou a **escultura** ([`NodeKind::Sampled`]) — a ponte da W5. É variante
/// nova num `enum`, e postcard escreve o discriminante por índice, então um documento v3 leria um nó
/// `Sampled` onde havia outra coisa. ⚠️ **A migração continua vazia pelo mesmo motivo de sempre**
/// (nada persiste um [`FieldDoc`]), e o degrau sobe pela mesma razão.
///
/// v5: o [`Node`] ganhou o **verbo** ([`Node::verb`]) — cada forma traz a operação com que dobra
/// sobre o resultado das anteriores, em vez de a herdar toda do pai. É campo novo numa struct, e
/// postcard é **posicional**; a migração continua vazia pelo motivo de sempre.
///
/// v6: o [`Blend`] ganhou o **chanfro** ([`Blend::Chamfer`]) e o campo do orgânico passou de `k`
/// (o alcance cru) para `radius` (o **entregue**, calibrado por [`Blend::ORGANIC_REACH`]). São
/// variante nova num `enum` **e** mudança de significado de um número: um documento v5 leria o
/// alcance de um orgânico como se fosse raio, e a peça mudaria de forma em silêncio.
///
/// v7: o [`Primitive`] ganhou **três formas** ([`Primitive::Cone`], [`Primitive::Capsule`],
/// [`Primitive::Prism`]). São variantes **acrescentadas no fim** do `enum`, então nenhum índice
/// existente se move e um documento v6 continua a ler-se certo — o degrau sobe na mesma, pela lei
/// do módulo: *um número que se lê errado em silêncio é pior do que um load que recusa em voz alta*.
///
/// v8: o [`Primitive::Prism`] passou a ter **duas pontas** (`bottom`/`top`, o que o torna também a
/// pirâmide e o tronco dela), e entraram a [`Primitive::Wedge`] e o [`Primitive::TorusArc`]. ⚠️ O
/// prisma **mudou de forma**, não só a lista cresceu: um documento v7 leria o `half_height` dele
/// como `top`, e a peça mudaria em silêncio.
///
/// v9: entraram a [`Primitive::Star`], o [`Primitive::BoxFrame`] e o [`Primitive::Ellipsoid`]. São
/// variantes **acrescentadas no fim** do `enum` — nenhum índice existente se move —, e o degrau sobe
/// pela lei do módulo, como o v7.
///
/// v10: o [`Primitive::TorusArc`] ganhou `round`. É campo novo numa variante, e postcard é
/// **posicional**: um documento v9 leria o ângulo dele como filete.
///
/// [`CLAUDE.md §5.0`]: ../../../CLAUDE.md
pub const FIELD_DOC_VERSION: u32 = 10;

/// Índice de um nó na arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

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
    Box { half: [f32; 3], round: f32 },
    /// Esfera. Não tem aresta, logo não tem `round`.
    Sphere { radius: f32 },
    /// Cilindro no eixo **Z** (outro eixo se obtém pela rotação do nó), com o aro das tampas
    /// arredondado em `round`.
    Cylinder {
        radius: f32,
        half_height: f32,
        round: f32,
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
    Wedge { half: [f32; 3], round: f32 },
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
}

impl PrimitiveKind {
    /// **A fonte da contagem** — quem quiser saber *«que formas o motor sabe fazer?»* pergunta aqui.
    pub const ALL: [PrimitiveKind; 14] = [
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
        }
    }
}

/// As três operações booleanas.
///
/// ⚠️ Só a **união** precisa de fórmula própria: intersecção e subtração saem por **De Morgan**
/// (`A ∩ B = ¬(¬A ∪ ¬B)`), sem fórmula nova. Duplicar a fórmula seria uma segunda resposta à mesma
/// pergunta, com uma chance a mais de divergir — e quem avalia (`ph2d-field-eval`) faz exatamente
/// essa derivação.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Op {
    Union(Blend),
    Intersection(Blend),
    /// `children[0]` menos todos os seguintes.
    Difference(Blend),
}

impl Op {
    #[must_use]
    pub fn blend(self) -> Blend {
        match self {
            Op::Union(b) | Op::Intersection(b) | Op::Difference(b) => b,
        }
    }
}

/// ⭐⭐⭐ **A RECEITA de uma combinação, numa frase:** as formas dobram na **ordem** em que estão, e
/// cada uma traz o **verbo** com que se junta ao resultado das anteriores.
///
/// `((c₀ ⊕₁ c₁) ⊕₂ c₂) …`, onde `⊕ᵢ` é o verbo de `cᵢ` — ou o **do pai**, quando `cᵢ` não trouxe
/// nenhum. É a mesma lei que o vetorial desta casa já paga desde 2026-08-22
/// (`docs/Vector Module/27_um_verbo_por_forma.md`), e ela vale aqui **pela mesma razão pela qual foi
/// barata lá**: os dois avaliadores já eram uma dobra à esquerda; o que estava fixo era só o verbo.
///
/// # ⚠️ Ausência é HERANÇA, não «sem verbo»
///
/// `None` não quer dizer *«esta forma não se combina»* — quer dizer *«use o do pai»*. As duas
/// consequências pesam para o mesmo lado:
///
/// - **todo documento anterior a esta versão avalia byte-idêntico**, porque nele ninguém se
///   pronunciou;
/// - **o seletor do pai não morre**: ele deixa de ser *a* operação e passa a ser o **padrão** de
///   quem não se pronunciou. Sem essa escolha ele ficaria inerte, que é o defeito *«parâmetro que
///   não muda nada»*.
///
/// # ⚠️ O verbo do PRIMEIRO filho nunca é perguntado
///
/// Ele **semeia** o acumulado — não há nada antes dele com que dobrar. Guardá-lo mesmo assim é
/// deliberado: *reordenar não pode destruir a escolha de quem passou pelo topo.* Arrastar o
/// terceiro filho para cima torna-o base sem nada a consertar, e arrastá-lo de volta devolve o
/// verbo que ele tinha.
///
/// ⛔ **E não é «começar do vazio»**, que seria a outra forma de o dizer: com o acumulado a nascer
/// vazio, uma subtração no topo apagaria a peça inteira (`∅ − a = ∅`) — uma reordenação que
/// destrói o modelo em silêncio.
#[must_use]
pub fn fold_verb(parent: Op, child: Option<Op>) -> Op {
    child.unwrap_or(parent)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Leaf(Primitive),
    Combine {
        op: Op,
        children: Vec<NodeId>,
    },
    /// ⭐ **Uma ESCULTURA**, referida pelo nome — a ponte da W5.
    ///
    /// ⚠️ **É um `NodeKind` e não uma `Primitive`, e a diferença é a razão de ele existir.** Uma
    /// primitiva é uma forma com **números** (raio, meia-extensão, `round`), e o painel deriva as
    /// linhas dela. Uma escultura não tem números: ela é uma malha, e o que a define vive noutro
    /// módulo. Metê-la entre as primitivas obrigaria toda a tabela de dimensões a ter um caso que
    /// não devolve nada.
    ///
    /// ⚠️ **O documento guarda o NOME, nunca a grade.** Uma grade de 128³ pesa 12 MB; pô-la aqui
    /// faria cada `cook` — que corre por quadro — copiar isso, e faria um projeto guardado carregar
    /// a grade em vez de a **regenerar** da malha, que é a fonte. Quem resolve nome → campo é o
    /// registo do avaliador (`ph2d_field_eval::hybrid::Registry`).
    ///
    /// ⚠️ **Um nome desconhecido lê como espaço VAZIO**, e não como sólido: numa união some, numa
    /// subtração não corta. O oposto encheria a cena de um bloco que ninguém autorizou.
    Sampled {
        key: String,
    },
}

impl NodeKind {
    /// O que este nó **é**, sem os filhos. Ver [`NodeShape`].
    #[must_use]
    pub fn shape(&self) -> NodeShape {
        match self {
            NodeKind::Leaf(p) => NodeShape::Leaf(p.clone()),
            NodeKind::Combine { op, .. } => NodeShape::Combine(*op),
            NodeKind::Sampled { key } => NodeShape::Sampled { key: key.clone() },
        }
    }
}

/// **O que um nó é, SEM a lista de filhos.**
///
/// ⭐ Existe porque a mesma árvore vive em dois sítios, e só um deles pode ser dono dos filhos:
///
/// | Onde | Quem são os filhos |
/// |---|---|
/// | [`FieldDoc`] (o **cozido**, o que se avalia) | índices da arena, em `NodeKind::Combine` |
/// | a **cena** (a fonte, o que o artista vê e move) | a hierarquia ECS (`Children`) |
///
/// Guardar a lista nos dois seria a segunda verdade clássica, e o sintoma seria específico e feio:
/// uma peça cuja **forma discorda da Hierarquia** — arrastar um objeto para dentro de outro no
/// painel mudaria a árvore que o artista vê e não a que o traçador avalia.
///
/// *Uma árvore, um dono dos filhos.* É a mesma lei que o vetorial paga como **fonte ≠ cozido**
/// (ADR-0121/0132).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeShape {
    Leaf(Primitive),
    Combine(Op),
    /// A escultura, pelo nome. Ver [`NodeKind::Sampled`].
    Sampled {
        key: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub xform: Xform,
    pub kind: NodeKind,
    /// ⭐ **A pilha de modificadores**, aplicada ao campo deste nó **depois** do que ele é e
    /// **antes** da pose dele. Vazia na esmagadora maioria dos nós. Ver [`crate::mods`].
    #[serde(default)]
    pub mods: Vec<Unary>,
    /// ⭐⭐⭐ **O VERBO com que este nó dobra sobre o resultado dos irmãos anteriores** — `None`
    /// herda o do pai. A lei inteira, com o que cada metade compra, está em [`fold_verb`].
    ///
    /// ⚠️ **Aqui e não no pai**, e a diferença é estrutural: um verbo guardado no pai como lista
    /// paralela a `children` seria uma segunda resposta a *«quantos filhos há»*, e ela ficaria
    /// obsoleta em todo sítio que desloca índices — o [`FieldDoc::union_all`] é um deles. Preso ao
    /// nó, ele viaja com o nó de graça.
    #[serde(default)]
    pub verb: Option<Op>,
}

impl Node {
    /// Um nó sem modificadores e que **herda** o verbo do pai — a forma curta, que é o caso de
    /// quase todo nó.
    #[must_use]
    pub fn new(xform: Xform, kind: NodeKind) -> Self {
        Self {
            xform,
            kind,
            mods: Vec::new(),
            verb: None,
        }
    }

    /// ⭐ **O verbo com que este nó dobra**, dado o do pai. Ver [`fold_verb`] — a lei vive lá, e
    /// esta é a forma curta para quem tem o nó em mãos.
    #[must_use]
    pub fn fold_verb(&self, parent: Op) -> Op {
        fold_verb(parent, self.verb)
    }
}

/// O documento: a arena de nós e a raiz.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDoc {
    pub version: u32,
    nodes: Vec<Node>,
    root: NodeId,
}

/// Por que um documento foi recusado.
///
/// ⚠️ Cada variante corresponde a um jeito de o campo **deixar de ser uma distância** ou de a
/// árvore deixar de ser uma árvore. Nenhuma é zelo: um documento inválido não produz um erro — ele
/// produz uma forma errada, em silêncio, três waves adiante.
// ⚠️ Sem `Eq`: `RoundTooLarge` carrega os `f32` que explicam a recusa (o raio pedido e o limite),
// e `f32` não é `Eq` por causa do NaN. Guardar os números vale mais do que a igualdade total —
// uma recusa que diz *"0,08 não cabe em 0,06"* poupa a próxima pessoa de ir medir.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldError {
    /// A arena está vazia, ou a raiz aponta para fora dela.
    BadRoot,
    /// Um filho tem índice ≥ o do pai — a invariante topológica (ver o doc da crate).
    ForwardReference { parent: u32, child: u32 },
    /// Uma operação sem filhos não tem o que combinar.
    EmptyCombine { node: u32 },
    /// Dimensão não-positiva (raio, altura, escala).
    NonPositive { node: u32, what: &'static str },
    /// O arredondamento não cabe na forma: a fonte encolhida ficaria negativa.
    RoundTooLarge { node: u32, round: f32, limit: f32 },
    /// Escala não-uniforme, ou não-finita (ver [`Xform::scale`]).
    BadScale { node: u32 },
    /// O perfil de um [`Primitive::Revolve`] tem ponto com `x < 0` — a superfície de revolução
    /// auto-intersecta e o campo deixa de ser uma distância.
    ProfileCrossesAxis { node: u32, min_x: f32 },
    /// Uma escultura sem nome não pode ser resolvida contra registo nenhum.
    EmptySampledKey { node: u32 },
    /// ⚠️ Modificadores sobre uma escultura — ver a nota de [`NodeKind::Sampled`] na validação.
    ModsOnSampled { node: u32 },
}

impl FieldDoc {
    /// Constrói e **valida**. Só há esta porta: um `FieldDoc` que exista está válido.
    ///
    /// # Errors
    /// Ver [`FieldError`].
    pub fn new(nodes: Vec<Node>, root: NodeId) -> Result<Self, FieldError> {
        let doc = Self {
            version: FIELD_DOC_VERSION,
            nodes,
            root,
        };
        doc.validate()?;
        Ok(doc)
    }

    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    #[must_use]
    pub fn root(&self) -> NodeId {
        self.root
    }

    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0 as usize)
    }

    /// Une vários documentos num só — **uma cena É a união dos seus objetos**.
    ///
    /// As arenas são concatenadas com deslocamento de índice e um nó de combinação novo recebe as
    /// raízes. ⚠️ **A invariante topológica sobrevive de graça**: cada arena já vem ordenada, o
    /// deslocamento preserva a ordem relativa, e a raiz nova é o último índice — logo todo filho
    /// continua vindo antes do pai, sem precisar de ordenação nem de verificação extra.
    ///
    /// Devolve `None` para uma lista vazia: uma cena sem objetos não tem campo, e um documento
    /// vazio inventado aqui seria uma forma que ninguém pediu.
    ///
    /// # Errors
    /// Só se o resultado violar a validação — o que, dadas entradas válidas, não pode acontecer;
    /// o `Result` existe para que isso seja verificado e não assumido.
    pub fn union_all(docs: &[FieldDoc], blend: Blend) -> Option<Result<Self, FieldError>> {
        match docs.len() {
            0 => return None,
            1 => return Some(Ok(docs[0].clone())),
            _ => {}
        }
        let mut nodes: Vec<Node> = Vec::new();
        let mut roots: Vec<NodeId> = Vec::new();
        for doc in docs {
            let base = nodes.len() as u32;
            for node in &doc.nodes {
                let mut node = node.clone();
                if let NodeKind::Combine { children, .. } = &mut node.kind {
                    for c in children.iter_mut() {
                        c.0 += base;
                    }
                }
                nodes.push(node);
            }
            roots.push(NodeId(doc.root.0 + base));
        }
        // ⚠️ **A raiz adotada perde o verbo dela**, e isto é decisão e não zelo: um verbo autorado
        // dentro de uma peça fala dos **irmãos dela**, e aqui ele passaria a falar das **outras
        // peças** da cena — uma peça inteira a subtrair-se de outra sem ninguém o ter pedido. Esta
        // porta chama-se `union_all`; a união é o contrato, e não uma omissão a herdar.
        for r in &roots {
            nodes[r.0 as usize].verb = None;
        }
        let root = NodeId(nodes.len() as u32);
        nodes.push(Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Combine {
                op: Op::Union(blend),
                children: roots,
            },
            mods: Vec::new(),
            verb: None,
        });
        Some(Self::new(nodes, root))
    }

    fn validate(&self) -> Result<(), FieldError> {
        if self.nodes.is_empty() || self.root.0 as usize >= self.nodes.len() {
            return Err(FieldError::BadRoot);
        }
        for (i, node) in self.nodes.iter().enumerate() {
            let idx = i as u32;
            if !node.xform.scale.is_finite() || node.xform.scale <= 0.0 {
                return Err(FieldError::BadScale { node: idx });
            }
            match &node.kind {
                NodeKind::Combine { children, .. } => {
                    if children.is_empty() {
                        return Err(FieldError::EmptyCombine { node: idx });
                    }
                    for c in children {
                        // A invariante topológica: filho SEMPRE antes do pai.
                        if c.0 >= idx {
                            return Err(FieldError::ForwardReference {
                                parent: idx,
                                child: c.0,
                            });
                        }
                    }
                }
                NodeKind::Leaf(p) => validate_primitive(idx, p)?,
                NodeKind::Sampled { key } => {
                    if key.is_empty() {
                        return Err(FieldError::EmptySampledKey { node: idx });
                    }
                    // ⚠️ **A pilha de modificadores NÃO corre sobre uma escultura, e recusar é a
                    // única resposta honesta.** Aplicá-la exigiria a casca, a matriz e a inclinação
                    // escritas uma segunda vez em números — cada uma com o gate de paridade que a
                    // segure. Deixá-la passar em silêncio daria um botão que não faz nada, que é o
                    // modo de falha que nenhum smoke apanha.
                    if !node.mods.is_empty() {
                        return Err(FieldError::ModsOnSampled { node: idx });
                    }
                }
            }
            // ⚠️ **A pilha valida com a MESMA porta que a escreve** (`Unary::set_value`), e não com
            // uma segunda cópia das regras aqui: duas listas de *"o que é um número aceitável"*
            // divergem na primeira variante nova, e a que fica errada é sempre a que ninguém lê.
            for m in &node.mods {
                for (field, d) in m.dims().into_iter().enumerate() {
                    let mut probe = *m;
                    probe.set_dim(idx, field as u8, d.value)?;
                }
            }
        }
        Ok(())
    }
}

fn validate_primitive(idx: u32, p: &Primitive) -> Result<(), FieldError> {
    let positive = |v: f32, what: &'static str| -> Result<(), FieldError> {
        if !v.is_finite() || v <= 0.0 {
            Err(FieldError::NonPositive { node: idx, what })
        } else {
            Ok(())
        }
    };
    let round_fits = |round: f32, limit: f32| -> Result<(), FieldError> {
        if !round.is_finite() || round < 0.0 || round >= limit {
            Err(FieldError::RoundTooLarge {
                node: idx,
                round,
                limit,
            })
        } else {
            Ok(())
        }
    };
    match *p {
        Primitive::Box { half, round } => {
            for h in half {
                positive(h, "half")?;
            }
            // ⚠️ O limite é a MENOR meia-extensão: a receita do arredondamento encolhe a caixa em
            // `round` nos três eixos, e uma delas ficando ≤ 0 não é "quase" — é uma caixa que
            // deixou de existir naquele eixo, e o campo que sai disso não é uma distância.
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Sphere { radius } => positive(radius, "radius"),
        Primitive::Cylinder {
            radius,
            half_height,
            round,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")?;
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Torus { major, minor } => {
            positive(major, "major")?;
            positive(minor, "minor")
        }
        Primitive::Extrude {
            profile: _,
            half_height,
            round,
        } => {
            positive(half_height, "half_height")?;
            // ⚠️ O limite é a meia-altura, e **só** ela. Um `round` maior do que a meia-largura do
            // perfil não é um erro: a receita (encolher a fonte, depois deslocar) é uma **abertura
            // morfológica**, e o que ela faz a um pescoço mais fino que `2·round` é exatamente o
            // que arredondar com esse raio deveria fazer — o pescoço desaparece. O campo continua a
            // ser um limite conservador de distância; a forma é a certa.
            //
            // Na altura não é assim: com `round ≥ half_height` o termo axial inverte de sinal e o
            // sólido deixa de existir — isso não é abertura, é uma forma que ninguém pediu.
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Revolve { ref profile } => {
            let min_x = profile.bounds().0[0];
            if min_x < 0.0 {
                return Err(FieldError::ProfileCrossesAxis { node: idx, min_x });
            }
            Ok(())
        }
        // ⚠️ **O `top` pode ser ZERO, e é o cone fechado** — só ele entre todos os números deste
        // arquivo. Exigir `> 0` proibiria a forma que dá nome à primitiva.
        Primitive::Cone {
            bottom,
            top,
            half_height,
            round,
        } => {
            positive(bottom, "bottom")?;
            if !top.is_finite() || top < 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "top",
                });
            }
            positive(half_height, "half_height")?;
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Capsule {
            radius,
            half_height,
        } => {
            positive(radius, "radius")?;
            positive(half_height, "half_height")
        }
        Primitive::Prism {
            sides,
            bottom,
            top,
            half_height,
            round,
        } => {
            // ⚠️ **A contagem é COAGIDA na porta, não recusada**: um prisma de 2 lados não é uma
            // forma degenerada que o artista queira ver recusada — é um valor que a UI nunca
            // oferece e que só um documento estragado traz. Recusar aqui rejeitaria a peça inteira.
            if !(MIN_PRISM_SIDES..=MAX_PRISM_SIDES).contains(&sides) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "sides",
                });
            }
            positive(bottom, "bottom")?;
            // ⚠️ Zero é a **pirâmide** — a mesma excepção do [`Primitive::Cone`], e pela mesma razão.
            if !top.is_finite() || top < 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "top",
                });
            }
            positive(half_height, "half_height")?;
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Wedge { half, round } => {
            for h in half {
                positive(h, "half")?;
            }
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::TorusArc {
            major,
            minor,
            angle,
            round,
        } => {
            positive(major, "major")?;
            positive(minor, "minor")?;
            round_fits(round, round_limit(p).unwrap_or(0.0))?;
            // ⚠️ **O ângulo é o único número deste arquivo cujo teto importa tanto quanto o piso**:
            // acima de `2π` o sector deixa de ser exprimível por semiplanos, e a porta coage em vez
            // de recusar (o slider pára lá, e só um documento estragado traz mais).
            if !angle.is_finite() || angle <= 0.0 {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "angle",
                });
            }
            Ok(())
        }
        Primitive::Star {
            points,
            outer,
            inner,
            half_height,
            round,
        } => {
            // ⚠️ **COAGIDA na porta como a contagem de lados** — a UI nunca oferece fora da faixa,
            // então um valor de fora só chega por um documento estragado, e recusar ali rejeitaria
            // a peça inteira por causa de um número que o documento sabe arredondar.
            if !(MIN_STAR_POINTS..=MAX_STAR_POINTS).contains(&points) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "points",
                });
            }
            positive(outer, "outer")?;
            positive(inner, "inner")?;
            // ⚠️ **O vale TEM de estar dentro da ponta**, e isto é validade e não gosto: com
            // `inner >= outer` as línguas invertem-se e a união devolve **o polígono dos vales** —
            // uma estrela que, ao arrastar um número, deixa de ser uma estrela **sem dizer nada**.
            if inner >= outer {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "inner",
                });
            }
            positive(half_height, "half_height")?;
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::BoxFrame {
            half,
            thickness,
            round,
        } => {
            for h in half {
                positive(h, "half")?;
            }
            positive(thickness, "thickness")?;
            // ⚠️ **Uma aresta mais grossa do que a meia-extensão fecha a gaiola** — as vigas
            // opostas encontram-se e a moldura vira uma caixa maciça. O `>` (e não `>=`) é
            // deliberado: com a igualdade elas tocam-se e o miolo some, que é a forma-limite.
            if thickness > half[0].min(half[1]).min(half[2]) {
                return Err(FieldError::NonPositive {
                    node: idx,
                    what: "thickness",
                });
            }
            round_fits(round, round_limit(p).unwrap_or(0.0))
        }
        Primitive::Ellipsoid { radii } => {
            for r in radii {
                positive(r, "radius")?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;
