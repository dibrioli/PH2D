//! **O TRAÇADO E A DECOMPOSIÇÃO EM PATCHES** — a fase **F3** do plano.
//!
//! Clean-room a partir de Pietroni et al., *Reliable Feature-Line Driven
//! Quad-Remeshing* (SIGGRAPH 2021), **§6**. ⚠️ Nenhuma linha traduzida de fonte
//! GPL — ver **ADR-0162**, Trilha A.
//!
//! # O que ela fecha
//!
//! O **F2** ([`ph2d_crossfield`]) entrega o campo e diz **onde as singularidades
//! ficam**. O **F4** ([`ph2d_quantize`]) já sabe consumir um [`Layout`] — patches,
//! lados, arcos, alvos — e resolvê-lo com ótimo demonstrado. ⭐ **Faltava quem
//! produzisse o layout sem o oráculo, e é esta fase.**
//!
//! # A ideia, em três frases
//!
//! De cada **singularidade** partem curvas que seguem o campo — as
//! *separatrizes*. Elas correm até bater noutra singularidade ou noutra
//! separatriz. O que sobra da superfície, recortada por elas, são os **patches**.
//!
//! ⚠️ **Aqui as separatrizes correm sobre ARESTAS da malha**, vértice a vértice,
//! e não como polilinhas a cortar triângulos. Não é aproximação por preguiça: é o
//! que faz a fronteira de cada patch ser um conjunto de arestas existentes, e por
//! isso o patch é um conjunto de **faces inteiras** — que é exatamente o formato
//! que o oráculo também exporta (medido: as fronteiras dele, reconstruídas em
//! 2026-08-20, caem todas em arestas da malha remalhada).
//!
//! # ⚠️ O que decide um CANTO
//!
//! Não é *"o vértice é um nó do traçado"*. Um nó em T é canto para os dois
//! patches do lado da haste e **meio de lado** para o patch do outro lado — e é
//! por isso que um lado pode ter mais de um arco. O que decide é o **ângulo
//! interno daquele patch naquele vértice**: perto de 90° é canto, perto de 180°
//! não é. *A pergunta é do par (patch, vértice), nunca do vértice sozinho.*
//!
//! # O que esta fase ainda NÃO faz
//!
//! - ⛔ **Feature lines** (arestas vivas) não entram: o QuadWild traça também a
//!   partir delas, e sem isso uma quina dura não vira fronteira de patch.
//! - ⛔ Uma separatriz que não fecha é **descartada**, e o [`TraceReport`] conta
//!   quantas. Ligá-la à mais próxima é trabalho do dia em que o número doer.

#![forbid(unsafe_code)]

/// **A FRONTEIRA de um patch e os cantos dela** — ver [`boundary`].
mod boundary;
/// **A DECOMPOSIÇÃO** — do traçado para patches, lados e arcos — ver [`patches`].
pub mod patches;
/// ⭐⭐⭐ **A PODA DOS TOCOS** — arcos que morrem em vértice regular — ver [`prune`].
pub mod prune;
/// **O ANEL de um vértice e as SEMENTES** — ver [`ring`].
pub mod ring;
/// **A TOPOLOGIA da decomposição** — as contas de `V − E + F` — ver [`topology`].
pub mod topology;
/// **O PASSEIO** que segue o campo — ver [`walk`].
pub mod walk;

use std::collections::BTreeMap;

use ph2d_crossfield::{CrossField, Dual};
use ph2d_mesh::Mesh;
use ph2d_quantize::{ArcSpec, Layout, PatchSpec};

pub use patches::PatchLayout;
pub use ring::Ring;
pub use walk::Walker;

/// O que o traçado mediu — o relatório que diz se ele fez sentido.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TraceReport {
    /// Quantos vértices o campo marcou como singulares.
    pub singularities: usize,
    /// Quantas separatrizes fecharam e viraram parede.
    pub separatrices: usize,
    /// ⚠️ Quantas foram **descartadas** por não fecharem dentro do limite de
    /// passos. Cada uma é uma parede que não existe, logo um patch maior do que
    /// devia — o número tem de ser lido junto com o histograma de valências.
    pub dangling: usize,
    /// Quantos patches saíram.
    pub patches: usize,
    /// Histograma **valência → quantos patches**. ⭐ É a linha que se compara com
    /// o oráculo: ele entrega 3 a 6, e um patch de 12 lados é o sintoma de uma
    /// separatriz que faltou.
    pub valence: BTreeMap<usize, usize>,
    /// ⚠️ Quantos patches têm mais de uma fronteira (`b ≠ 1`) — um anel.
    /// O F4 recusa layout aberto; um patch anelar não tem lados definidos.
    ///
    /// ⛔ **Ela é CEGA AO GÉNERO** — ver [`Self::with_genus`].
    pub non_disk: usize,
    /// ⭐⭐ **Quantos patches têm GÉNERO** — `χ(região) ≠ 1`.
    ///
    /// ⛔ **A [`Self::non_disk`] é CEGA a isto, e a cegueira custou o produto**
    /// (2026-08-22): num toro, um patch engolia a asa inteira e saía com **uma**
    /// fronteira, então a única cerca que havia deixava-o passar — e a malha final
    /// vinha com `χ = 2` onde a topologia exige `0`, com 100 % de quads, zero bordo
    /// e zero não-manifold.
    ///
    /// ⚠️ **É diagnóstico, não cerca.** Quem recusa é o
    /// [`PatchLayout::to_layout`], pela régua do COMPLEXO — ver
    /// [`patches::PatchLayout::chi`] para porque é que a régua por patch não serve.
    pub with_genus: usize,
    /// ⭐⭐ **Quantas arestas de parede têm o MESMO patch dos dois lados.**
    ///
    /// ⚠️ **Elas são invisíveis para o passeio da fronteira**, e é isso que este
    /// número mede: o `boundary_loops` só conta uma parede como fronteira quando a
    /// face do outro lado é de **outro** patch, então uma parede interior não
    /// aparece em lado nenhum — nem como fronteira, nem como aviso.
    ///
    /// ⭐ **Ela existe para precificar o CORTE de uma asa** (`PLAN.md`
    /// §4-quatervicies): cortar é acrescentar uma parede interior, e ela só serve
    /// para alguma coisa se o passeio passar a percorrê-la **dos dois lados**.
    /// Antes de mexer nessa regra é preciso saber quantas já existem — porque cada
    /// uma delas mudaria de significado.
    ///
    /// ⚠️ **Ela é do par (paredes, decomposição)**, e por isso mora aqui e não numa
    /// sonda: medir as paredes CRUAS contra os patches LIMPOS dá um número sem
    /// sentido (foi o primeiro que esta sonda produziu, e ele dizia 204).
    pub interior_walls: usize,
    /// Quantos arcos o layout tem.
    pub arcs: usize,
    /// Quantas paredes foram **dissolvidas** para curar patches degenerados.
    /// ⚠️ Cada uma é uma separatriz que o traçado não devia ter posto ali — o
    /// número é o custo da limpeza, e lê-se junto com o das valências.
    pub dissolved: usize,
    /// ⭐⭐⭐ **Quantos TOCOS a poda removeu** — ver [`prune`]. Um toco é um arco que
    /// morre num vértice **regular**, que nada no campo pediu.
    ///
    /// ⛔ **Sem esta contagem, «a poda não mudou nada» não distingue *não funciona* de
    /// *não correu***, que é a mesma lição do `fell_back` e do `slid` do F5.
    pub pruned: usize,
    /// Quantas rondas de limpeza correram.
    pub rounds: usize,
    /// ⭐⭐⭐ **POR QUE a limpeza parou** — `0` nada a fazer · `1` a dissolução não removeu
    /// parede nenhuma · `2` a ronda piorava a topologia · `3` o tecto de rondas.
    ///
    /// ⚠️ **Sem isto, `0 rondas` com degenerados vivos não distingue três avarias
    /// diferentes**, e elas pedem curas opostas: uma dissolução que não dissolve, uma cura
    /// que custa mais do que vale, ou um laço que não converge. *Um contador de rondas diz
    /// que parou; só a razão diz onde mexer.*
    pub cleanup_stop: u8,
    /// ⭐⭐⭐ **Quantas rondas de ABERTURA POR CORTE foram adoptadas** — ver
    /// [`patches::open_rings`].
    pub opened_rings: usize,
    /// ⚠️ **Quantos vértices têm o anel ABERTO** — não-manifold ou de borda.
    ///
    /// ⭐ Ele está aqui para separar *"o traçado é mau"* de *"a malha que
    /// chegou é má"*, e as duas coisas leem igual num histograma de valências.
    /// Um anel aberto não tem índice de singularidade definido, e o campo
    /// devolve `0` ali — o que faz a soma dos índices deixar de bater `4·χ` sem
    /// que nada no campo esteja errado.
    pub open_rings: usize,
    /// ⭐ **Quantos cantos tiveram de ser PROMOVIDOS** por a estrutura não chegar
    /// para o laço ser um patch — ver `patches::MIN_PATCH_CORNERS`.
    ///
    /// ⚠️ **Cada um é um vértice irregular a mais na malha final**, e por isso ele
    /// é contado e não escondido: é a dívida que o traçado ainda deve, medida.
    pub promoted: usize,
}

/// **TRAÇA e DECOMPÕE** — a porta desta fase.
///
/// ⚠️ **A malha tem de estar TRIANGULADA e fechada**, que é o que o
/// [`ph2d_crossfield::Dual`] já exige.
#[must_use]
pub fn trace_patches(mesh: &Mesh, dual: &Dual, field: &CrossField) -> PatchLayout {
    trace_patches_with(mesh, dual, field, prune::stems_enabled())
}

/// **A MESMA porta, com a PODA à vista** — ver [`prune`].
///
/// ⭐⭐ **Ela existe porque a poda precisa das paredes DEPOIS da limpeza**, e essas só
/// existem aqui dentro. ⛔ *Um gate que chamasse o [`prune::prune_stems`] sobre as
/// paredes cruas do passeio mediria outra coisa* — mediu, e devolveu `0` remoções,
/// porque o layout cru ainda tem lascas e as guardas julgam contra um estado que a
/// limpeza ainda ia mudar.
///
/// ⚠️ **O [`trace_patches`] lê a constante; esta lê o argumento.** É o que permite ao
/// gate correr os dois lados sobre o caminho REAL, em vez de sobre uma reconstrução
/// dele.
#[must_use]
pub fn trace_patches_with(
    mesh: &Mesh,
    dual: &Dual,
    field: &CrossField,
    prune_stems: bool,
) -> PatchLayout {
    let walker = Walker::new(mesh, dual, field);
    let (mut walls, base) = walker.trace_all();
    let mut out = patches::decompose(mesh, &walls, base.clone());

    // ⭐⭐ **A PONTE DO ANEL — e ela é a PRIMEIRA cura tentada, antes de dissolver.**
    //
    // Medido em 2026-08-22: nas três fixturas do corpus com parede interior, ela vive
    // **sempre** no patch anel e é um **caminho entre as duas fronteiras dele**. ⇒ *A
    // ponte que abre o anel em disco já está traçada;* o passeio da fronteira é que
    // se recusava a percorrê-la, porque exigia que a face do outro lado fosse de
    // outro patch.
    //
    // ⚠️ **Ela é julgada pela MESMA guarda que as dissoluções** — só entra se a
    // distância topológica melhorar. Ligá-la sempre foi medido e reprovado: no toro
    // 32×16 levava o complexo de `0` para `−1`. *A ponte é uma cura, e uma cura que
    // piora não é uma cura.*
    //
    // ⚠️⚠️ **A melhoria tem de ser ESTRITA e da DISTÂNCIA, não do par.** Comparar a
    // saúde inteira deixava entrar um movimento lateral: no toro 32×16 a ponte
    // empatava na distância (`1`) e ganhava nos degenerados (`0` contra `1`), era
    // adoptada, e o laço parava contente num complexo de **`−1`** — pior do que os
    // `0` a que uma dissolução chegava. *Um critério que aceita empates deixa a cura
    // barata expulsar a cura certa.*
    let bridged = patches::decompose_with(mesh, &walls, base.clone(), true);
    // ⚠️ **A sonda da guarda da PONTE.** A `health` é um PAR — distância topológica **e**
    // contagem de degenerados — e esta comparação lê só o primeiro. *Uma ponte que cure
    // quatro patches degenerados e empate na distância é recusada por um empate.*
    if std::env::var("PH2D_BRIDGE_LOG").as_deref() == Ok("1") {
        eprintln!(
            "  [ponte] sem {:?} · com {:?}",
            health(&out),
            health(&bridged)
        );
    }
    if health(&bridged).0 < health(&out).0 {
        out = bridged;
    }
    // ⭐ **A limpeza é iterativa porque dissolver uma lasca pode fazer nascer
    // outra**: as faces dela passam para o vizinho, e o vizinho ganha cantos que
    // não tinha. Ela pára sozinha quando não há degenerado nenhum — e o teto
    // existe só para o caso de duas lascas se curarem uma à outra em ciclo.
    let mut dissolved = 0usize;
    let mut rounds = 0usize;
    let mut stop = 0u8;
    for _ in 0..MAX_CLEANUP_ROUNDS {
        let victims = out.degenerate();
        if victims.is_empty() {
            stop = 0;
            break;
        }
        stop = 3;
        let before = health(&out);
        // ⭐ **A ronda corre sobre uma CÓPIA e só é adoptada se passar.** Dissolver
        // no original e desfazer depois deixaria uma janela em que `walls` está
        // errado — e o `break` dessa janela é justamente o caminho do defeito.
        // *Assim a recusa não tem nada para desfazer.*
        let mut trial = walls.clone();
        if !patches::dissolve(&mut trial, &out, &victims) {
            stop = 1;
            break;
        }
        let mut next_report = base.clone();
        next_report.dissolved = dissolved + victims.len();
        next_report.rounds = rounds + 1;
        let next = patches::decompose(mesh, &trial, next_report);
        let after = health(&next);

        // ⛔⛔ **UMA CURA QUE PIORA A TOPOLOGIA NÃO É UMA CURA.** Medido em
        // 2026-08-22 no toro 48×24: a ronda 10 levava o complexo de `1` para `2`
        // **e** deixava a lista de degenerados vazia — ou seja, ela *escondia* o
        // defeito ao mesmo tempo que o agravava, e a cadeia devolvia uma malha de
        // género errado com 100 % de quads e zero arestas de bordo.
        // ⭐ **A SONDA DA GUARDA.** `PH2D_CLEANUP_FORCE=1` deixa a limpeza continuar mesmo
        // quando ela piora a topologia — não para shipar, mas para responder à pergunta que
        // decide se vale a pena construir a reparação por CORTE: *se forçar a cura que
        // existe já apagar os furos, os patches maus são a causa; se não, são um beco.*
        // ⛔ Sem esta sonda, a única forma de saber era construir a outra metade primeiro.
        if after.0 > before.0 && std::env::var("PH2D_CLEANUP_FORCE").as_deref() != Ok("1") {
            stop = 2;
            break;
        }
        walls = trial;
        dissolved += victims.len();
        rounds += 1;
        out = next;
    }
    out.report.dissolved = dissolved;
    out.report.rounds = rounds;
    out.report.cleanup_stop = stop;

    // ⭐⭐⭐ **A ABERTURA POR CORTE — a metade que faltava** (ver [`patches::open_rings`]).
    //
    // ⚠️ **Ela corre DEPOIS da fusão e não em vez dela**: as duas curam defeitos opostos —
    // a lasca é *uma parede a mais* e o anel é *uma parede a menos*. ⛔ Correr o corte
    // primeiro daria-lhe um layout com lascas dentro, e a guarda julgaria contra um estado
    // que a fusão ainda ia mudar (a mesma razão pela qual a poda corre por último).
    //
    // ⚠️⚠️ **A guarda lê o PAR inteiro, e a da fusão lê só o primeiro.** A `health` é
    // `(distância topológica, degenerados)`, e a fusão compara `.0` — *um corte que cure
    // quatro anéis e empate na distância seria recusado por um empate*. Aqui a melhoria
    // tem de ser estrita **no par**, o que aceita «mesma distância, menos degenerados» e
    // continua a recusar «menos degenerados, pior topologia».
    let mut opened = 0usize;
    for _ in 0..MAX_OPEN_ROUNDS {
        if !open_rings_enabled() {
            break;
        }
        if out.loops.iter().all(|l| l.len() < 2) {
            break;
        }
        let before = health(&out);
        let mut trial = walls.clone();
        let cut_ok = patches::open_rings(mesh, &mut trial, &out);
        if std::env::var("PH2D_BRIDGE_LOG").as_deref() == Ok("1") {
            let rings = out.loops.iter().filter(|l| l.len() >= 2).count();
            eprintln!(
                "  [corte] {rings} aneis · cortou {cut_ok} · paredes {} -> {}",
                walls.edges.len(),
                trial.edges.len()
            );
        }
        if !cut_ok {
            break;
        }
        let mut next_report = base.clone();
        next_report.dissolved = dissolved;
        next_report.rounds = rounds;
        next_report.cleanup_stop = stop;
        let next = patches::decompose(mesh, &trial, next_report);
        let after = health(&next);
        if std::env::var("PH2D_BRIDGE_LOG").as_deref() == Ok("1") {
            eprintln!("  [corte] saude {before:?} -> {after:?}");
        }
        if after >= before {
            break;
        }
        walls = trial;
        opened += 1;
        out = next;
    }
    out.report.opened_rings = opened;

    // ⭐⭐⭐ **A PODA DOS TOCOS** — ver [`prune`]. Ela corre **depois** da limpeza de
    // propósito: a limpeza cura patches **degenerados** (uma lasca é uma parede a
    // mais) e a poda ataca outra coisa — arcos que morrem num vértice **regular**,
    // que o campo nunca pediu. ⚠️ *Podar antes daria à poda um layout com lascas
    // dentro, e as guardas dela julgariam contra um estado que a limpeza ainda ia
    // mudar.*
    if prune_stems {
        let index = ph2d_crossfield::vertex_index(mesh, dual, field);
        let (_, next, n) = prune::prune_stems(mesh, walls, &out.report, &index);
        if n > 0 {
            out = next;
        }
        out.report.pruned = n;
    }
    // ⭐⭐⭐ **O CAMPO VIAJA COM O LAYOUT** — ver [`PatchLayout::face_dir`].
    //
    // ⚠️ **É aqui e não num parâmetro do F5**, e a razão é medida: a montagem
    // enviesava o interior dos patches por interpolar a fronteira, e o motivo
    // apurado em 2026-08-22 foi que ela **não recebia o campo**. Um parâmetro novo
    // pode ser esquecido em qualquer um dos dezoito sítios que chamam o F5; *quem
    // tem o layout tem, por construção, o campo que o gerou.*
    out.face_dir = (0..mesh.faces().len())
        .map(|f| field.direction(dual, f))
        .collect();
    out
}

/// **A SAÚDE de uma decomposição** — `(distância topológica, degenerados)`.
///
/// ⭐ **A primeira componente é a que manda**, e é a única que a limpeza está
/// proibida de piorar: `|complexo − χ(peça)|` é zero exactamente quando a
/// decomposição ainda descreve a superfície que entrou. A segunda é o progresso
/// visível, e serve para reconhecer uma ronda que não andou.
fn health(layout: &PatchLayout) -> (i64, usize) {
    (
        (layout.complex_euler() - layout.mesh_chi).abs(),
        layout.degenerate().len(),
    )
}

/// ⛔⛔ **UM TETO DE RONDAS PARADAS foi construído, MEDIDO e REJEITADO** —
/// 2026-08-22, e a razão é que ele não distingue o caso bom do mau.
///
/// A ideia era: *"nove dissoluções com o estado idêntico não são cura, corte-as"*.
/// Ela reprovou porque a paciência é do FENÓMENO e não uma constante:
///
/// | fixtura | rondas com `(distância, degenerados)` idêntico | e depois |
/// |---|---|---|
/// | ⭐ **esfera 48×72** | `(1, 1)` da ronda 0 à 4 — **cinco** | a ronda 5 fecha em `(0, 0)` ✓ |
/// | ⛔ toro 48×24 | `(1, 1)` da ronda 0 à 9 — **dez** | a ronda 10 vai para `(2, 0)` ⛔ |
///
/// ⇒ **As duas são indistinguíveis enquanto correm.** Um teto de `2` matava a
/// esfera; um de `9` deixava o toro passar na mesma. *Uma paciência que decide
/// certo num caso e errado no outro não é uma constante — é um palpite.*
///
/// ⚠️ **A guarda que FICA é a outra**, e essa é demonstrável sem paciência
/// nenhuma: uma ronda que aumenta a distância topológica é recusada, sempre. Ela
/// apanha exactamente a ronda 10 do toro e **não toca** nas cinco da esfera.
const _MEASURED_AND_REJECTED_STALE_CAP: () = ();

/// ⚠️ **O teto de rondas de limpeza.** Ele não escolhe qualidade nenhuma: a
/// limpeza pára sozinha quando não há patch degenerado. Existe para o caso
/// patológico de duas lascas se recriarem uma à outra, e quem o atinge sai com
/// degenerados ainda na lista — que o F4 recusa, alto e claro.
const MAX_CLEANUP_ROUNDS: usize = 32;

/// Quantas rondas de abertura por corte, no máximo.
///
/// ⚠️ **O critério de paragem é a guarda** (a saúde tem de melhorar estritamente no par),
/// que termina sozinha; isto é a rede contra um corte que oscile.
const MAX_OPEN_ROUNDS: usize = 16;

/// ⛔⛔⛔ **FALSE — a reparação por CORTE foi construída, MEDIDA e REJEITADA, e ela corrige
/// a leitura que a motivou.**
///
/// A leitura era: *«quatro dos cinco patches maus têm 2–3 fronteiras ⇒ são anéis, e a cura
/// publicada de um anel é cortar entre as fronteiras»*. O corte foi construído
/// ([`patches::open_rings`]) e a medição diz outra coisa.
///
/// # ⛔ A tabela (peça do artista, 2026-08-25)
///
/// | | valor |
/// |---|---|
/// | anéis encontrados | `4` |
/// | ⛔ **paredes acrescentadas** | **`4`** — ou seja **UMA aresta por anel** |
/// | saúde `(distância, degenerados)` | `(1, 5)` ⇒ ⛔ **`(2, 6)`** |
///
/// # ⭐⭐⭐ O mecanismo, e ele corrige o diagnóstico
///
/// ⚠️ **Um caminho de UMA aresta entre as duas fronteiras significa que elas se TOCAM.**
/// Estes patches não são anéis gordos com um buraco no meio: são patches **ESTRANGULADOS**,
/// cujas duas fronteiras passam a um triângulo uma da outra. ⇒ *cortar ali não abre nada —
/// acrescenta um toco*, e o toco cria mais um patch degenerado: `5` ⇒ `6`, e a distância
/// topológica sobe de `1` para `2`.
///
/// ⇒ **É uma TERCEIRA espécie**, e nenhuma das duas curas serve: a lasca é *uma parede a
/// mais* (funde-se), o anel gordo é *uma parede a menos* (corta-se), e o estrangulado é
/// *uma parede no sítio errado*. ⛔ **A contagem de fronteiras não distingue o anel gordo do
/// estrangulado** — só a **distância entre elas** distingue, e nenhuma régua a media.
///
/// # ⛔⛔⛔ E a RÉGUA foi construída — e mostrou que o corte NÃO TEM CASO neste corpus
///
/// A [`patches::ring_gaps`] mede o **vão** entre as duas fronteiras, e a
/// [`patches::MIN_RING_GAP`] passou a barrar o estrangulado. Com a porta de pé:
///
/// | peça | patch | lados | fronteiras | vão | faces | saúde antes ⇒ depois |
/// |---|---|---|---|---|---|---|
/// | do artista | 10 | 4 | 2 | `2` | **16** | `(1,5)` ⇒ ⛔ `(2,6)` |
/// | do artista | 21 | 3 | 3 | `2` | **8** | — |
/// | do artista | 24 · 33 | 8 · 2 | 3 · 2 | ⛔ `1` | 13 · 7 | *barrados pela porta* |
/// | furada | 2 | 16 | 6 | `4` | ⭐ **1011** | `(5,6)` ⇒ `(5,6)` — **empate** |
/// | furada | 14 · 19 · 21 | 10 · 2 · 2 | 4 · 2 · 2 | ⛔ `1` | 38 · 6 · 2 | *barrados* |
///
/// ⭐⭐⭐ **NÃO EXISTE UM ANEL GORDO NO CORPUS INTEIRO.** O maior patch multi-fronteira tem
/// **1 011 faces** e as duas fronteiras dele passam a **4 arestas** uma da outra — ele
/// também está estrangulado, só que em grande. ⇒ *o vão nunca cresce com o patch*, e é isso
/// que diz que a espécie «anel gordo» é uma hipótese sem exemplar aqui.
///
/// ⚠️ **A porta do vão não resgata o corte:** com ela, a peça do artista continua a piorar
/// (`(1,5)` ⇒ `(2,6)`) e a furada **empata**. *Uma cura que empata no melhor caso e piora
/// no resto não é uma cura.*
///
/// ⇒ **O defeito é o ESTRANGULAMENTO**, e ele não se cura acrescentando nem tirando uma
/// parede: cura-se **movendo-a**, que é re-traçar aquela região. Fica como a obra seguinte,
/// e é maior que esta.
///
/// ⚠️ **Fica desligado com a maquinaria construída** — `PH2D_OPEN_RINGS=1` reabre a
/// experiência sem recompilar, e a régua [`patches::ring_gaps`] fica **viva no
/// instrumento**, porque é ela que nomeia a espécie.
const OPEN_RINGS: bool = false;

fn open_rings_enabled() -> bool {
    std::env::var("PH2D_OPEN_RINGS").as_deref() == Ok("1") || OPEN_RINGS
}

impl PatchLayout {
    /// **GRADUA a densidade por um campo de TAMANHO por vértice.**
    ///
    /// ⭐⭐ **É o que faz o `Follow Curvature` significar alguma coisa na cadeia
    /// global.** Até 2026-08-21 o knob existia no painel, era lido pelo motor local
    /// e **nada** na cadeia global o consumia — o log dizia isso em voz alta, e o
    /// artista lia *"não funciona"*, que é a leitura correcta.
    ///
    /// `size[v]` é o lado de quad que se quer no vértice `v`, na mesma unidade do
    /// mundo — exactamente o que a [`ph2d_quadflow::ScaleField`] já calcula a partir
    /// da curvatura. Cada troço da cadeia passa a valer
    ///
    /// ```text
    ///     |Δ| × alvo / tamanho_local
    /// ```
    ///
    /// ⇒ onde o campo pede quads **menores** que o alvo, o troço "mede" **mais** e
    /// recebe mais segmentos. ⭐ **O alvo entra dividido de propósito:** assim o
    /// `τ` continua a ser um comprimento efectivo e o [`Self::to_layout`] continua a
    /// dividir por `alvo` — *um campo uniforme igual ao alvo devolve o τ original,
    /// bit a bit*, e a graduação nunca é um caso especial.
    ///
    /// ⚠️ **O tamanho de um troço é a MÉDIA HARMÓNICA das duas pontas**, e não a
    /// aritmética: densidade é `1/tamanho`, e o que se integra ao longo do arco é a
    /// densidade. Com a média aritmética, um troço entre um vértice muito denso e um
    /// muito folgado receberia menos segmentos do que a ponta densa exige.
    pub fn grade(&mut self, mesh: &Mesh, size: &[f32], target_edge: f32) {
        let pos = mesh.positions();
        let scale = if target_edge > 0.0 { target_edge } else { 1.0 };
        for (a, chain) in self.arc_chain.iter().enumerate() {
            let Some(tau) = self.arc_tau.get_mut(a) else {
                continue;
            };
            tau.clear();
            tau.push(0.0);
            let mut run = 0.0f32;
            for w in chain.windows(2) {
                let (i, j) = (w[0] as usize, w[1] as usize);
                let (Some(p), Some(q)) = (pos.get(i), pos.get(j)) else {
                    continue;
                };
                let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                let len = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
                // ⚠️ **Média HARMÓNICA** — ver o doc acima.
                let (si, sj) = (
                    size.get(i).copied().unwrap_or(scale).max(1.0e-9),
                    size.get(j).copied().unwrap_or(scale).max(1.0e-9),
                );
                let harmonic = 2.0 / (1.0 / si + 1.0 / sj);
                run += len * scale / harmonic;
                tau.push(run);
            }
        }
    }

    /// **O LAYOUT que o F4 consome**, com os alvos derivados de `target_edge`.
    ///
    /// ⚠️ **`target_edge` é o comprimento de aresta desejado no quad final**, na
    /// mesma unidade do mundo. O alvo de cada arco é o comprimento geométrico dele
    /// a dividir por esse número — é a mesma régua que a bancada usa para medir o
    /// oráculo, e é o que torna as duas colunas comparáveis.
    ///
    /// ⚠️ **Os dois parágrafos acima estavam colados ao [`Self::grade`]** até
    /// 2026-08-22 — um `///` sem item entre eles, e o `rustdoc` do `to_layout`
    /// aparecia vazio enquanto o do `grade` prometia devolver um `Layout`.
    ///
    /// # Errors
    /// Devolve [`ph2d_quantize::LayoutError`] quando a decomposição não fecha — um
    /// arco de bordo, um patch de menos de 3 lados, ou ⭐ **um patch que não é um
    /// disco** ([`ph2d_quantize::LayoutError::GenusLost`]).
    ///
    /// ⛔ **A última cerca nasceu de um remesh que mudava o GÉNERO da peça em
    /// silêncio** (2026-08-22): num toro, um patch engolia a asa inteira, saía com
    /// `χ = −1` e **uma** fronteira, e a malha final vinha com `χ = 2` onde a
    /// topologia exige `0` — com 100 % de quads, zero bordo e zero não-manifold.
    /// *Todas as réguas que o produto tinha continuavam verdes.*
    ///
    /// ⚠️ **Recusar é o certo enquanto o F3 não souber CORTAR a asa**, e é a
    /// escolha explícita: uma recusa com nome manda o artista para o conserto; uma
    /// malha que perdeu o buraco manda-o para lado nenhum, porque ela parece boa.
    pub fn to_layout(&self, target_edge: f32) -> Result<Layout, ph2d_quantize::LayoutError> {
        let scale = if target_edge > 0.0 { target_edge } else { 1.0 };
        // ⭐⭐ **A CERCA DO COMPLEXO, e ela é a PRIMEIRA de propósito.** As outras
        // recusas do `Layout::new` falam de contagens, e um patch com uma asa lá
        // dentro produz contagens perfeitamente válidas — cada arco usado duas
        // vezes, toda fronteira um laço só, valências entre 3 e 6.
        let complex = self.complex_euler();
        if complex != self.mesh_chi {
            return Err(ph2d_quantize::LayoutError::GenusLost {
                complex,
                surface: self.mesh_chi,
            });
        }
        // ⭐⭐ **O ALVO SAI DO `τ`, e não do comprimento geométrico.** Sem
        // graduação os dois são o MESMO número por construção; com ela, o `τ` é o
        // comprimento **efectivo** e é ele que carrega a densidade pedida. Ver
        // [`PatchLayout::arc_tau`].
        let arcs = self
            .arc_tau
            .iter()
            .map(|t| {
                let tau = t.last().copied().unwrap_or(0.0);
                // ⭐⭐ **QUADRÁTICA, como a referência a escreve** — ver
                // `ArcSpec::isometric`. Sobre um custo LINEAR o ótimo é indiferente
                // entre esmagar um arco longo e espalhar o erro, e com peso
                // `1/alvo` ele passava a *preferir* esmagar. É a marginal crescente
                // do quadrático que faz o solver distribuir.
                ArcSpec::isometric(f64::from(tau / scale))
            })
            .collect();
        let patches = self
            .sides()
            .into_iter()
            .map(|sides| PatchSpec { sides })
            .collect();
        Layout::new(arcs, patches)
    }
}
