//! **REMESH ISOTRÓPICO** — o estágio que faltava inteiro (ADR-0162, F1).
//!
//! # Por que ele existe: a medição
//!
//! O oráculo `quadwild-bimdf` converge para **~9 300 triângulos qualquer que
//! seja a entrada**. Medido em 2026-08-20, na bancada `ph2d-quadbench`:
//!
//! | entrada | vértices antes | vértices **depois** | aresta média depois |
//! |---|---|---|---|
//! | `cube` | **8** | **6 146** | 0,0356 |
//! | `torus 64×32` | 2 048 | 5 176 | 0,0568 |
//! | `sculpt_hooked` | 3 386 | 4 651 | 0,0588 |
//! | `sphere_sculpt_98k` | **98 306** | **4 636** | 0,0576 |
//!
//! ⭐ **Oito vértices e noventa e oito mil saem no mesmo lugar.** É *este* passe
//! que torna a densidade da saída independente da densidade da entrada — e a
//! ausência dele é por que o nosso pipeline devolvia **malha vazia** num cubo: o
//! piso do `edge_for_detail` é derivado da aresta de entrada, e uma entrada de 8
//! vértices não resolve grade nenhuma.
//!
//! # A lei, e o que dela é MEDIDO
//!
//! ⚠️ **O alvo de aresta é uma fração da DIAGONAL DA CAIXA, não da entrada.** É a
//! única forma de a saída não herdar a doença da entrada. O `alpha` do preset
//! `Organic` do oráculo é **0,02**, e sobre o cubo (a única fixtura plana, onde
//! a curvatura não pede refino) ele bate quase exato: `0,02 × 1,732 = 0,0346`
//! contra **0,0356** medidos.
//!
//! ⛔⛔⛔ **E o «item aberto» que aqui esteve desde 2026-08-20 está REFUTADO** (medido
//! 2026-08-28). Ele dizia: *«nas fixturas curvas o oráculo termina mais FINO que
//! `alpha × diag` (0,0566 contra 0,0693 na esfera; 0,0588 contra 0,0859 na
//! `sculpt_hooked`) — ou seja, ele refina abaixo do alvo onde a curvatura pede»*.
//!
//! ⭐ **A malha remalhada DELE foi medida por bandas de curvatura** (`κ ≈ 2·|L(p)|/h²`,
//! a lei do Laplaciano uniforme que dá `1/R` numa esfera), e o expoente de `h ~ κ^e`
//! sobre uma faixa de **`8×`** na curvatura é:
//!
//! | peça | `κ` da banda 0 | `κ` da banda 7 | aresta na banda 0 | na banda 7 | ⇒ `e` |
//! |---|---|---|---|---|---|
//! | `sculpt_eared` | 0,97 | **7,70** | 0,0564 | 0,0490 | **`−0,029`** |
//! | `sculpt_hooked` | 0,73 | **6,64** | 0,0579 | 0,0552 | **`−0,007`** |
//! | `sculpt_wrinkled` | 0,97 | **6,20** | 0,0566 | 0,0558 | **`+0,009`** |
//!
//! ⇒ `e ≈ 0`: **a malha dele é uniforme, tal como esta.** (`−0,5` seria erro geométrico
//! constante; `−1`, ângulo constante.)
//!
//! ⭐⭐⭐ **A inferência de 2026-08-20 confundiu duas afirmações:** *«o alvo GLOBAL dele
//! numa peça curva é menor que `alpha × diag`»* é verdade, e **não implica** *«ele refina
//! LOCALMENTE onde a curvatura pede»*, que é falso. A primeira é sobre como ele escolhe
//! **um** número por peça; a segunda, sobre a variação **dentro** dela.
//!
//! ⚠️ Isto **não** diz que densidade adaptativa seja má ideia — diz que ela é uma **feature
//! de produto** (o *Adaptive Size* do ZBrush/QuadRemesher, com o artista a decidir) e
//! **não** o que separa a nossa saída da dele. Detalhe:
//! [`ACHADO_o_acabamento_e_a_regua_da_densidade.md`](../../../docs/3D/quad-remesh/ACHADO_o_acabamento_e_a_regua_da_densidade.md)
//! §6.
//!
//! # O laço, e de quem é cada peça
//!
//! Clean-room a partir da literatura (Botsch & Kobbelt 2004, *A Remeshing
//! Approach to Multiresolution Modeling*; QuadWild 2021 §4). ⛔ Nenhuma linha
//! traduzida de fonte GPL — ADR-0162.
//!
//! 1. **partir** as arestas acima de `4/3` do alvo — [`ph2d_mesh::refine_in_sphere`];
//! 2. **colapsar** as abaixo de `4/5` — [`ph2d_mesh::collapse_in_sphere`];
//! 3. **trocar** as arestas que melhoram a valência — [`ph2d_mesh::relax_valence`];
//! 4. **relaxar** tangencialmente e **reprojetar** na superfície ORIGINAL.
//!
//! ⚠️ **Os três primeiros passos já existiam na engine, testados**, e são a
//! topologia dinâmica que o pincel usa. O que este crate acrescenta é o passe
//! **global** e o passo 4. *O plano mandava conferir antes de construir outra
//! estrutura de malha, e a resposta foi: não construa.*

#![forbid(unsafe_code)]

use ph2d_mesh::{Birth, Mesh, RegionScratch, Remap};

/// **A fração da diagonal da caixa que vira o lado do triângulo** — MEDIDA, e a tabela que
/// a confirma sobre o cubo está no doc do módulo.
///
/// ⛔⛔ **A PROVENIÊNCIA que esta linha alegava era FALSA, e saiu em 2026-08-30.** Ela
/// atribuía o número a um preset do alvo restrito. O número existe lá, e ⛔ **não é um
/// comprimento de remalhagem em leitura nenhuma**: num dos programas é a mistura
/// *regularidade ↔ isometria* do objectivo da quantização, no outro é o peso de alinhamento à
/// curvatura de um alisador de campo.
///
/// ⚠️ **O valor fica porque foi medido AQUI; a frase é que tinha de sair.** *Um número com
/// proveniência falsa lê-se como número com medição*, e o próximo agente que quisesse afinar a
/// densidade iria buscar autoridade a um knob que controla outra coisa.
pub const ALPHA: f32 = 0.02;

/// **A histerese entre partir e colapsar** — a lei clássica `[4/5, 4/3]`.
///
/// ⚠️ **Não é gosto: é o que impede o passe de oscilar.** Se as duas soleiras
/// fossem o alvo, uma aresta ligeiramente longa seria partida em duas
/// ligeiramente curtas, que seriam colapsadas de volta, para sempre. A banda
/// `[4/5·t, 4/3·t]` é a mais estreita que fecha: partir uma aresta de `4/3·t` dá
/// duas de `2/3·t`, e `2/3 > 4/5 · 4/5`… ⚠️ **e ela ainda assim NÃO fecha para
/// toda aresta** — é por isso que o laço tem teto de rodadas e sai quando o
/// número de vértices para de mudar, e não quando "não há mais o que fazer".
const SPLIT_FACTOR: f32 = 4.0 / 3.0;
/// O par da [`SPLIT_FACTOR`].
const COLLAPSE_FACTOR: f32 = 4.0 / 5.0;

/// **Quantas rodadas do laço** — teto de TERMINAÇÃO, não de qualidade.
///
/// ⚠️ Medido: as fixturas do corpus convergem (contagem de vértices estável
/// dentro de 1 %) em **6 a 9** rodadas; o cubo, que parte de 8 vértices e tem de
/// chegar a ~6 000, é o pior caso. O teto existe porque este passe corre sob
/// comando do artista e uma malha patológica que oscilasse travaria a janela.
pub const MAX_ROUNDS: usize = 24;

/// ⛔⛔⛔ **ZERO — a reparação não-manifold foi construída em QUATRO formas, MEDIDA, e as
/// quatro são PIORES que não reparar.**
///
/// `0` não repara (mede e reporta) · `1` parte os vértices · `2` deita fora a aleta.
/// `PH2D_MANIFOLD_REPAIR` sobrepõe-se, para reabrir a experiência sem recompilar.
///
/// # ⭐⭐⭐ A raiz ESTÁ confirmada — é o «como» que falha
///
/// A escultura do artista entra com **2 arestas não-manifold a raio `1,30×`** (a ponta), e
/// os furos da saída moram a `1,29×` — *o mesmo sítio*
/// (`docs/3D/quad-remesh/ACHADO_ordem_das_fases.md` §11). ⭐ **Partir os vértices leva as
/// transições inexactas de `8` a `0`**: a ligação causal deixou de ser hipótese.
///
/// # ⛔ A tabela das quatro
///
/// | variante | bordo da saída | `χ` | transições inexactas | enviesamento | `>60°` |
/// |---|---|---|---|---|---|
/// | ⭐ **não reparar** | **`8`** | **`1`** | ⛔ `8` | **`7,3°`** | **`5`** |
/// | partir ANTES do remalhe | ⛔ `148` | ⛔ `−16` | ⭐ `0` | ⛔ `13,7°` | ⛔ `63` |
/// | partir + fechar buracos | ⛔ **saída VAZIA** | — | `0` | — | — |
/// | partir DEPOIS do remalhe | `8` | `0` | ⛔ `12` | ⛔ `9,1°` | ⛔ `11` |
/// | deitar a aleta fora | `8` | `1` | ⛔ `10` | ⛔ `8,3°` | ⛔ `11` |
///
/// # ⭐⭐ O mecanismo comum, e ele fecha a família
///
/// ⚠️ **Todas as quatro ABREM a superfície.** Partir uma aresta ambígua numa peça fechada
/// separa-a por construção; deitar a aleta fora deixa o buraco onde ela estava. E as
/// medições dizem que **esta cadeia tolera pior um BURACO do que uma aresta ambígua**: com o
/// defeito, `8` de bordo; sem ele mas com um rasgo, `148`.
///
/// ⛔⛔ **UMA AFIRMAÇÃO DESTE DOC FOI REFUTADA EM 2026-08-26, e era a que escolhia o sítio
/// da cura.** Ela dizia *«o remalhe cria não-manifold sozinho — `4 ⇒ 0` na porta e `2` outra
/// vez depois do laço»*. ⚠️ **O controlo nunca tinha sido corrido:** medido em **onze** peças
/// limpas do corpus, o remalhe cria **zero** (`0 ⇒ 0` em todas). O `4 ⇒ 2` era da `t001`, que
/// **entra** com `4` — o remalhe **propaga**, não cria.
///
/// ⚠️ *Eu tinha comparado dois números da MESMA peça partida sem nunca olhar uma peça limpa.*
/// ⇒ é por isso que a [`DOUBLED_REPAIR`] corre **à entrada** e a chamada gémea depois do laço
/// foi **retirada**: o único motivo dela era esta frase.
///
/// ⇒ **A cura que faltava não era soldar** — ver [`DOUBLED_REPAIR`]: a estrutura medida não
/// era uma aleta nem duas folhas, era um par `(triângulo, espelho)`.
pub const MANIFOLD_REPAIR: u8 = 0;

/// ⭐⭐⭐ **REMOVER AS FOLHAS DE ESPESSURA ZERO À ENTRADA — LIGADO.**
///
/// ⛔⛔ **Esta é a quinta reparação, e a primeira que não abre a peça.** As quatro de
/// 2026-08-25 ([`MANIFOLD_REPAIR`]) foram desenhadas a partir do **nome** do defeito. A
/// sonda `manifold_census` mediu a estrutura em 2026-08-26 e o nome estava errado: as `4`
/// arestas ambíguas da escultura do artista são as arestas de **4 pares
/// `(triângulo, espelho)`** — `0` cópias com a mesma orientação, **`4` com orientação
/// oposta. *Uma bolsa de volume zero, não uma aleta.*
///
/// ⇒ Apagar **as duas** faces de cada par não tira superfície nenhuma, e a medição
/// confirma-o na peça: `ambíguas 4 → 0`, **`bordo 0 → 0`**.
///
/// ⚠️ **À ENTRADA e não no fim**, ao contrário do que a nota de [`MANIFOLD_REPAIR`] concluiu
/// para a partição: uma folha dupla é um **defeito do ficheiro**, e deixá-la entrar no laço
/// põe o remalhe a refinar, colapsar e projectar geometria que não existe.
/// ⚠️ `PH2D_DOUBLED_REPAIR=0` desliga, para bissecar.
pub const DOUBLED_REPAIR: bool = true;

fn doubled_repair_on() -> bool {
    std::env::var("PH2D_DOUBLED_REPAIR").map_or(DOUBLED_REPAIR, |v| v != "0")
}

/// Se a variante `1` fecha os buracos que abriu. ⛔ Medido: leva a saída a **vazia**.
const FILL_AFTER_SPLIT: bool = false;

fn repair_mode() -> u8 {
    std::env::var("PH2D_MANIFOLD_REPAIR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(MANIFOLD_REPAIR)
}

/// O que o passe fez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    /// Vértices antes.
    pub verts_before: usize,
    /// Vértices depois.
    pub verts_after: usize,
    /// Quantas rodadas do laço correram.
    pub rounds: usize,
    /// Quantas trocas de aresta (flips) o passe fez ao todo.
    pub flips: usize,
    /// ⭐⭐⭐ **A reparação não-manifold da porta** — ⚠️ `bad_edges_after > 0` é vermelho:
    /// a cadeia inteira a jusante assume duas faces por aresta.
    pub manifold: ph2d_mesh::ManifoldReport,
    /// ⭐⭐⭐ **As folhas de espessura zero removidas à ENTRADA** — ver [`DOUBLED_REPAIR`].
    pub doubled_door: ph2d_mesh::DoubledReport,
}

/// **O ALVO DE ARESTA desta malha** — `ALPHA × diagonal da caixa`.
///
/// ⚠️ **Da CAIXA e não da malha.** Uma média de arestas da entrada faria a saída
/// herdar a densidade da entrada, que é exatamente a propriedade que este passe
/// existe para destruir.
#[must_use]
pub fn target_edge(mesh: &Mesh, alpha: f32) -> f32 {
    let b = mesh.bounds();
    let d = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    let diag = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    (alpha.max(1.0e-6) * diag).max(1.0e-6)
}

/// **REMALHA a malha isotropicamente**, no lugar.
///
/// A malha de saída tem triângulos de lado ≈ [`target_edge`] em toda parte,
/// **independentemente** de como a entrada estava.
///
/// ⚠️ **A entrada é TRIANGULADA na porta.** As três operações da engine recusam
/// quads por geometria (partir uma aresta de um quad devolve um triângulo e um
/// pentágono), e deixar a recusa chegar ao chamador seria transformar um detalhe
/// de representação numa condição de erro do produto.
pub fn remesh_isotropic(mesh: &mut Mesh, alpha: f32) -> Report {
    remesh_with(mesh, alpha, border_law_on())
}

/// O passe com a [`BORDER_LAW`] **explícita**.
///
/// ⚠️ **Por argumento e não por variável de ambiente:** os testes desta crate correm em
/// paralelo no mesmo processo, e uma env lida lá dentro faria um gate decidir o resultado do
/// outro. *Uma bandeira global é uma corrida escrita à mão.*
fn remesh_with(mesh: &mut Mesh, alpha: f32, rim_law: bool) -> Report {
    let verts_before = mesh.vert_count();
    mesh.triangulate();

    // ⭐⭐⭐ **AS FOLHAS DE ESPESSURA ZERO SAEM ANTES DE TUDO** — ver [`DOUBLED_REPAIR`].
    //
    // ⚠️ **Antes do `reference.clone()`**, de propósito: a superfície de referência contra
    // a qual o laço reprojeta não pode conter a bolsa, senão o alisamento passa o passe
    // inteiro a puxar vértices para uma folha que não é superfície.
    let doubled_door = if doubled_repair_on() {
        ph2d_mesh::drop_doubled_faces(mesh)
    } else {
        ph2d_mesh::DoubledReport::default()
    };

    let reference = mesh.clone();
    let target = target_edge(mesh, alpha);

    let (mut scratch, mut births, mut remap) = (
        RegionScratch::default(),
        Vec::<Birth>::new(),
        Remap::default(),
    );
    let mut flips = 0usize;
    let mut rounds = 0usize;
    let mut last = usize::MAX;
    while rounds < MAX_ROUNDS {
        rounds += 1;
        // A esfera cobre a malha inteira: este é o passe GLOBAL, e as três portas
        // da engine são por-região porque o traço as usa por-dab.
        let (centre, radius) = whole(mesh);

        // ⭐⭐⭐ **A CERCA POR SÍTIO entra nos DOIS passes** — ver [`SizingGrid`]. Ela é
        // reconstruída a cada ronda porque a malha muda, e é isso que a mantém honesta.
        let grid = adaptive_on()
            .then(|| SizingGrid::build(mesh, target))
            .flatten();
        let split = grid
            .as_ref()
            .map(|g| move |p: [f32; 3]| g.at(p) * SPLIT_FACTOR);
        let split_ref: ph2d_mesh::Sizing<'_> = split
            .as_ref()
            .map(|f| f as &(dyn Fn([f32; 3]) -> f32 + Sync));
        ph2d_mesh::refine_in_sphere_sized(
            mesh,
            centre,
            radius,
            target * SPLIT_FACTOR,
            split_ref,
            &mut births,
            &mut scratch,
        );
        let (centre, radius) = whole(mesh);
        let shrink = grid
            .as_ref()
            .map(|g| move |p: [f32; 3]| g.at(p) * COLLAPSE_FACTOR);
        let shrink_ref: ph2d_mesh::Sizing<'_> = shrink
            .as_ref()
            .map(|f| f as &(dyn Fn([f32; 3]) -> f32 + Sync));
        ph2d_mesh::collapse_in_sphere_sized(
            mesh,
            centre,
            radius,
            target * COLLAPSE_FACTOR,
            shrink_ref,
            &mut remap,
            &mut scratch,
        );
        flips += ph2d_mesh::relax_valence(mesh, &mut scratch);
        relax_and_project(mesh, &reference, target, rim_law);

        // ⚠️ **A saída é por ESTABILIDADE, não por "não há mais o que fazer".** A
        // banda de histerese não fecha para toda aresta (ver `SPLIT_FACTOR`), e um
        // laço que esperasse zero operações não terminaria em malha nenhuma.
        let now = mesh.vert_count();
        if last != usize::MAX && (now as f32 - last as f32).abs() <= 0.01 * last as f32 {
            break;
        }
        last = now;
    }

    // ⭐⭐⭐ **A CAUDA ADAPTATIVA — refinar onde a forma APERTA, e só refinar.**
    //
    // ⛔⛔ **A raiz da amputação (report do artista, 2026-08-29):** o laço acima remalha para
    // um alvo **uniforme** (`ALPHA × diagonal = 0,089` na peça dele), e o **raio local** de um
    // espinho dele cai a `0,037`. *Uma agulha mais fina do que UM triângulo não pode ser
    // representada* — nenhum truque de reprojecção a salva, porque não há onde a pôr.
    //
    // ⚠️ **E é por isso que a reprojecção-com-normal (ver [`facing_on`]) curou a fase e
    // partiu a seguinte:** ela guardava os vértices da agulha com o alvo ainda a `0,089`,
    // logo com triângulos gigantes e mal formados. *Guardar a ponta e desenhar a grade não
    // são dois trabalhos: são um.*
    //
    // ⭐ **Aqui a adaptação é DE GRAÇA**, e é a diferença para a tentativa do §8-quater: o F1
    // é uma remalha **métrica**, não uma parametrização — um alvo que varia realiza-se
    // exactamente, sem projecção de mínimos quadrados a lavá-lo.
    //
    // ⚠️ **Só REFINA**, nunca grosseira: o corpo fica com a densidade que sempre teve, e o
    // que muda é a agulha. E corre **depois** do laço, senão o passe de colapso dele comia as
    // faces finas na ronda seguinte.
    // ⭐⭐⭐ **A REPARAÇÃO NÃO-MANIFOLD, e ela vem DEPOIS do remalhe.**
    //
    // ⛔⛔ **Uma aresta reclamada por três faces mente para todo o resto do motor.** O mapa
    // de meias-arestas que o layout percorre é `(a, b) → face`, **uma** face por aresta
    // dirigida, e com três a reclamar a mesma ele guarda uma: a travessia de fronteira
    // entra na face errada ou morre. Medido 2026-08-25 na escultura do artista: **2 arestas
    // não-manifold a raio `1,30×`** — a ponta — e os furos da saída a raio `1,29×`. *O mesmo
    // sítio.* Ver `docs/3D/quad-remesh/ACHADO_ordem_das_fases.md` §11.
    //
    // ⚠️⚠️ **DEPOIS e não antes, e a medição obrigou:** reparar à entrada não pega, porque
    // **o próprio remalhe cria arestas não-manifold** — medido na peça do artista, `4 ⇒ 0` na
    // porta e **`2` outra vez** depois do laço. *Reparar a malha que entra não é reparar a
    // malha que sai*, e quem a cadeia consome é a que sai.
    //
    // ⚠️ **Aqui e não no chamador**, porque este é o passe por onde **toda** a cadeia entra
    // — e uma reparação que dependa de alguém se lembrar de a chamar é uma reparação que um
    // dos chamadores não faz.
    let manifold = match repair_mode() {
        1 => ph2d_mesh::split_non_manifold(mesh),
        2 => ph2d_mesh::drop_extra_faces(mesh),
        _ => ph2d_mesh::ManifoldReport {
            bad_edges_before: ph2d_mesh::non_manifold_edges(mesh),
            bad_edges_after: ph2d_mesh::non_manifold_edges(mesh),
            ..ph2d_mesh::ManifoldReport::default()
        },
    };
    // ⭐⭐⭐ **E FECHAR o que a partição abriu.**
    //
    // ⛔⛔ **Partir uma aresta ambígua RASGA a superfície**, e um rasgo é pior que o defeito
    // que ele cura: medido 2026-08-25 na peça do artista, partir sem fechar leva as arestas
    // de bordo da saída de `8` para **`148`** e o `χ` de `1` para **`−16`**. ⚠️ *A cura
    // certa aplicada sem a metade que a completa é uma cura pior que a doença.*
    //
    // ⚠️ **Só corre se houve partição** — numa malha limpa nada foi aberto, e chamar o
    // fechador ali seria dar-lhe uma malha para julgar que ele não tem de julgar.
    if manifold.copies > 0 && repair_mode() == 1 && FILL_AFTER_SPLIT {
        ph2d_mesh::fill_holes(mesh);
    }

    // ⚠️ **A superfície de referência é uma CÓPIA do estado de entrada**, tirada
    // depois do `triangulate` e antes da primeira edição. Reprojetar contra a
    // malha que está a ser editada seria pedir a uma superfície que se corrigisse
    // contra si mesma — o alisamento então encolhe sem nada a segurá-lo, que é o
    // mecanismo já medido e registrado na recusa 13 do ADR-0160.

    Report {
        verts_before,
        verts_after: mesh.vert_count(),
        rounds,
        flips,
        manifold,
        doubled_door,
    }
}

/// A esfera que cobre a malha inteira.
fn whole(mesh: &Mesh) -> ([f32; 3], f32) {
    let b = mesh.bounds();
    let c = [
        (b.min[0] + b.max[0]) * 0.5,
        (b.min[1] + b.max[1]) * 0.5,
        (b.min[2] + b.max[2]) * 0.5,
    ];
    let d = [b.max[0] - c[0], b.max[1] - c[1], b.max[2] - c[2]];
    // ⚠️ **Com folga**: um raio exatamente igual ao da caixa deixa de fora, por
    // arredondamento, as arestas que tocam a fronteira — e são justamente as da
    // silhueta, que é onde o artista olha.
    let r = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
    (c, r * 1.5 + 1.0e-4)
}

/// **PASSO 4 — relaxação tangencial com reprojeção.**
///
/// ⚠️ **SÓ a componente tangente do deslocamento**, e a razão está medida: a
/// média dos vizinhos de um nó sobre uma superfície curva cai **para dentro**
/// dela (é a corda, não o arco), e essa componente normal é a que tira volume.
/// A projeção de volta transforma o encolhimento num deslize **ao longo** da
/// superfície, que é tudo o que se quer aqui.
/// ⛔ A lei do rebordo, recusada por medição — ver o módulo irmão.
#[path = "border.rs"]
mod border;
mod project;
mod sizing;

pub use project::{project_facing, project_onto};

use border::{border_lambda, border_law_on, border_polyline, project_onto_polyline};
use sizing::{SizingGrid, adaptive_on, facing_on};

fn relax_and_project(mesh: &mut Mesh, reference: &Mesh, target: f32, rim_law: bool) {
    let n = mesh.vert_count();
    if n == 0 {
        return;
    }
    let neighbours: Vec<Vec<u32>> = {
        let adj = mesh.adjacency();
        (0..n)
            .map(|v| adj.vert_verts.neighbours(v).to_vec())
            .collect()
    };
    let normals: Vec<[f32; 3]> = mesh.normals().to_vec();

    // ⭐⭐⭐ **O REBORDO TEM OUTRA LEI, e sem ela ele VAGUEIA.**
    //
    // ⛔⛔ Até 2026-08-26 este passe não sabia o que era um bordo: um vértice do rebordo
    // era alisado na direcção dos vizinhos — que são quase todos **interiores** — e depois
    // projectado na **superfície** de referência, que perto de um rebordo aberto não é a
    // curva. Medido na `sculpt_t002` do artista, o perímetro do buraco ia de `0,6046` a
    // **`0,7841`** (**+30 %**) sem que laço nenhum se perdesse. ⚠️ *A contagem de arestas de
    // bordo não via nada disto: ela é função do passo* (ver `ACHADO_ordem_das_fases.md` §15).
    //
    // ⇒ A lei: um vértice de bordo é alisado **ao longo do rebordo** (só os vizinhos de
    // bordo contam) e projectado na **poligonal** da referência, nunca na superfície.
    // ⚠️ **Inerte em peça fechada por construção** — `border_of` sai vazio.
    let border_ref = if rim_law {
        border_polyline(reference)
    } else {
        Vec::new()
    };
    let border_nbrs: Vec<Vec<u32>> = if border_ref.is_empty() {
        Vec::new()
    } else {
        let mut count: std::collections::BTreeMap<(u32, u32), usize> =
            std::collections::BTreeMap::new();
        for f in mesh.faces() {
            let v = f.verts();
            for k in 0..v.len() {
                let (a, b) = (v[k], v[(k + 1) % v.len()]);
                *count
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_default() += 1;
            }
        }
        let mut out: Vec<Vec<u32>> = vec![Vec::new(); n];
        for ((a, b), c) in count {
            if c == 1 {
                out[a as usize].push(b);
                out[b as usize].push(a);
            }
        }
        out
    };

    let mut target_pos = vec![[0.0f32; 3]; n];
    {
        let pos = mesh.positions();
        for v in 0..n {
            // ⭐ O rebordo primeiro: ele nunca cai na lei da superfície.
            if let Some(bn) = border_nbrs.get(v).filter(|b| !b.is_empty()) {
                let mut sum = [0.0f32; 3];
                for &w in bn {
                    let q = pos[w as usize];
                    for i in 0..3 {
                        sum[i] += q[i];
                    }
                }
                #[allow(clippy::cast_precision_loss)]
                let inv = 1.0 / bn.len() as f32;
                let p = pos[v];
                let lambda = border_lambda();
                let smoothed = [
                    lambda.mul_add(sum[0].mul_add(inv, -p[0]), p[0]),
                    lambda.mul_add(sum[1].mul_add(inv, -p[1]), p[1]),
                    lambda.mul_add(sum[2].mul_add(inv, -p[2]), p[2]),
                ];
                target_pos[v] = project_onto_polyline(&border_ref, smoothed);
                continue;
            }
            let ns = &neighbours[v];
            if ns.len() < 3 {
                target_pos[v] = pos[v];
                continue;
            }
            let mut sum = [0.0f32; 3];
            for &w in ns {
                let p = pos[w as usize];
                for i in 0..3 {
                    sum[i] += p[i];
                }
            }
            let inv = 1.0 / ns.len() as f32;
            let p = pos[v];
            let d = [
                sum[0].mul_add(inv, -p[0]),
                sum[1].mul_add(inv, -p[1]),
                sum[2].mul_add(inv, -p[2]),
            ];
            let nv = normals[v];
            let along = dot(d, nv);
            target_pos[v] = [
                LAMBDA.mul_add(along.mul_add(-nv[0], d[0]), p[0]),
                LAMBDA.mul_add(along.mul_add(-nv[1], d[1]), p[1]),
                LAMBDA.mul_add(along.mul_add(-nv[2], d[2]), p[2]),
            ];
        }
    }
    // ⚠️ **O rebordo passa por aqui também, e é INERTE por prova, não por sorte:** ele é um
    // subconjunto da superfície de referência, logo o pé dele nela é ele próprio. Medido:
    // com e sem um salto para o rebordo, o perímetro sai igual — nos dois valores de
    // `BORDER_LAMBDA` que a varredura correu. *Um salto que nada muda é uma linha a
    // defender para sempre.*
    // ⭐⭐⭐ **O PÉ TEM DE CONCORDAR COM A NORMAL, e é isto que salva uma AGULHA.**
    //
    // ⛔⛔ **Reproduzido em 2026-08-29 com a peça do artista:** a fase zero sozinha come
    // **`15,9 %`** do alcance dela (`2,355 → 1,981`), e a cadeia inteira a jusante perde mais
    // `0,018` — *a amputação acontece toda aqui*. Nas fixturas de espinhos a perda segue a
    // espessura: `−1,6 %` a `σ = 0,30`, `−5,8 %` a `0,10`, `−12,9 %` a `0,07`, `−15,8 %` a
    // `0,05`.
    //
    // ⭐ **O mecanismo:** [`project_onto`] pede o ponto **mais próximo** da referência. Num
    // tubo mais fino que o espaçamento, o mais próximo está do **OUTRO LADO** — o vértice
    // atravessa a agulha, e o tubo fecha-se sobre si.
    //
    // ⚠️ **A cura já existia nesta crate e este chamador não a usava:**
    // [`project_facing`] recusa um pé cuja normal de face **discorda** da direcção dada, e
    // cai no de recurso quando nenhum concorda. *Uma capacidade construída e não ligada é
    // uma capacidade que não existe.*
    //
    // ⛔⛔ **Ela nasce DESLIGADA, e a razão é medida:** cura esta fase e parte a seguinte —
    // ver [`facing_on`], que traz a tabela. `PH2D_ISO_FACING=1` liga-a.
    let facing = facing_on();
    for (v, t) in target_pos.iter_mut().enumerate() {
        let n = facing.then(|| normals[v]);
        *t = project_facing(reference, *t, target, n);
    }
    mesh.positions_mut().copy_from_slice(&target_pos);
    // ⚠️ **O `rebuild` paga a dívida que o `positions_mut` nomeia**: sem ele a
    // caixa, o octree e as normais descrevem a malha de antes — e a rodada
    // seguinte leria a caixa errada para montar a esfera global.
    mesh.rebuild();
}

/// Meio passo de Laplaciano — o amortecimento que o torna monótono.
const LAMBDA: f32 = 0.5;

pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
