//! ⭐⭐⭐ **A PRIMEIRA SONDA DESTE REPO QUE PERGUNTA SE O VALOR DE UM CONTROLO CHEGA A ALGUÉM.**
//!
//! > `CLAUDE.md` §5.0: *«⛔ Nenhum instrumento do repo pergunta se o VALOR chega a um consumidor: o
//! > `architecture_panel_wiring_parity` mede focalizabilidade, e os `seam_*` provam que o clique
//! > chega à ferramenta, nunca que a escrita dela chega a um efeito.»*
//!
//! # ⛔⛔ Ela nasceu de um erro medido, e o erro foi meu
//!
//! Enio, 2026-09-01, com foto do trilho: *«esses botões de mover, rot e scale já existiam. só não
//! estavam ligados a cada modo.»* Estavam lá desde sempre — pintados, clicáveis, a acender-se — e o
//! `chrome::rail_tools` fazia deles um **rádio exclusivo que só escrevia a própria luz**. Eu li o
//! sintoma como *«falta um controlo»* e construí um pulldown novo ao lado.
//!
//! ⚠️ **E o censo que devia ter apanhado isso foi feito com `head -20` e mentiu:** ele disse que o
//! `TOOL_PIVOT` também não tinha leitor, quando ele tem **dois** no shell. *Um `grep` truncado
//! devolve «zero consumidores» com a mesma cara de um `grep` completo.*
//!
//! # ⭐ O que esta sonda faz que uma varredura de ids não faz
//!
//! Ela não pergunta *«este id existe?»* nem *«alguém despacha este clique?»* — os dois já têm gate.
//! Ela pergunta **o terceiro passo**: *o que este chip escreve é LIDO por alguém que decide?*
//!
//! | metade | como |
//! |---|---|
//! | a população | **derivada** de [`rail_entries`], nos dois modos — nunca escrita à mão |
//! | o veredito | a tabela [`RAIL_CONSUMERS`] abaixo, uma linha por chip |
//! | ⭐ a **prova** | um consumidor declarado é **procurado no ficheiro que o declara**, e o gate reprova se ele lá não estiver |
//!
//! ⛔ **É a terceira metade que a torna um instrumento em vez de um comentário.** Uma tabela que só
//! afirmasse *«este tem consumidor»* envelheceria no dia em que o consumidor saísse — que é
//! exactamente como o rádio do trilho chegou a 2026 sem ninguém a lê-lo.
//!
//! # ⚠️ E o censo de OBSOLESCÊNCIA é obrigatório
//!
//! `CLAUDE.md` §5.0: *«uma catraca sem censo de obsolescência não desce: ela vira licença.»* A
//! tabela tem os dois sentidos — todo chip pintado tem linha, e toda linha descreve um chip pintado.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::screens::hero::left_rail::rail_entries;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// O que um chip do trilho faz com o valor que escreve.
enum Fate {
    /// ⭐ **Alguém LÊ o que ele escreve, e decide.** O `&str` é o ficheiro que o lê — e o gate
    /// **confere que o nome do id aparece lá dentro**, relativo à raiz da workspace.
    ReadBy(&'static str),
    /// ⛔ **Ninguém lê**, e a linha diz porquê. Só é legítimo quando a ausência é uma DECISÃO com
    /// mecanismo escrito — nunca «ainda não ligámos».
    DeadOnPurpose(&'static str),
}

/// Uma linha da tabela: o NOME do chip (para a mensagem e para a busca no consumidor), o id, e o
/// destino do valor.
///
/// ⚠️ O id vem por **função** e não por valor: os `ids::*` são `const fn` de hash e uma tabela
/// `const` não os pode chamar directamente.
type Row = (&'static str, fn() -> NodeId, Fate);

/// ⭐⭐⭐ **UMA LINHA POR CHIP DA FILA** — e o veredito é sobre o **VALOR**, não sobre o clique.
const RAIL_CONSUMERS: &[Row] = &[
    // ── Os verbos de transformação ─────────────────────────────────────────────────────────────
    //
    // ⛔⛔ **Vivos SÓ com o módulo 3D no canvas**, desde 2026-09-01. No editor 2D o gizmo escolhe o
    // verbo pela ALÇA que se agarra (bbox → mover, canto → escalar, anel → rodar), não por um modo —
    // então não há lá o que ligar, e a fileira é decoração.
    (
        "TOOL_TRANSLATE",
        || ids::TOOL_TRANSLATE,
        Fate::ReadBy("crates/ph2d-panel-model3d/src/area_bar.rs"),
    ),
    (
        "TOOL_ROTATE",
        || ids::TOOL_ROTATE,
        Fate::ReadBy("crates/ph2d-panel-model3d/src/area_bar.rs"),
    ),
    (
        "TOOL_SCALE",
        || ids::TOOL_SCALE,
        Fate::ReadBy("crates/ph2d-panel-model3d/src/area_bar.rs"),
    ),
    // ⭐ **O `PIVOT` é o único dos quatro que sempre teve consumidor** — e a entrega 36 disse o
    // contrário porque o `grep` dela levou `head -20`.
    (
        "TOOL_PIVOT",
        || ids::TOOL_PIVOT,
        Fate::ReadBy("shells/desktop/src/input_dispatch.rs"),
    ),
    // ── O referencial e o enquadrar ────────────────────────────────────────────────────────────
    (
        "TOOL_SPACE",
        || ids::TOOL_SPACE,
        Fate::ReadBy("crates/ph2d-panel-model3d/src/event.rs"),
    ),
    (
        "TOOL_HOME",
        || ids::TOOL_HOME,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_tools.rs"),
    ),
    // ── Desfazer / refazer ─────────────────────────────────────────────────────────────────────
    (
        "TOOL_UNDO",
        || ids::TOOL_UNDO,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs"),
    ),
    (
        "TOOL_REDO",
        || ids::TOOL_REDO,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs"),
    ),
    // ── ⭐ Os VERBOS DO PAINTER ────────────────────────────────────────────────────────────────
    //
    // Todos vivos, e pela mesma porta: o `chrome::rail_painter_tools` converte o id num
    // `EditorAction::ToolPanelEvent(PanelEvent::SelectOption(..))`, que o shell drena e entrega à
    // ferramenta. ⭐ **É o contra-exemplo que dá sentido a esta tabela:** aqui o clique SAI do
    // chrome e vira um efeito; nos quatro verbos de transformação ele parava na própria luz.
    (
        "PAINTER_RAIL_BRUSH",
        || ids::PAINTER_RAIL_BRUSH,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_EYEDROPPER",
        || ids::PAINTER_RAIL_EYEDROPPER,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_ERASER",
        || ids::PAINTER_RAIL_ERASER,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_CLONE",
        || ids::PAINTER_RAIL_CLONE,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_SMEAR",
        || ids::PAINTER_RAIL_SMEAR,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_BLUR",
        || ids::PAINTER_RAIL_BLUR,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_LIQUIFY",
        || ids::PAINTER_RAIL_LIQUIFY,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_TRANSFORM",
        || ids::PAINTER_RAIL_TRANSFORM,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_MASK_GROUP",
        || ids::PAINTER_RAIL_MASK_GROUP,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_INPAINT",
        || ids::PAINTER_RAIL_INPAINT,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    (
        "PAINTER_RAIL_SHAPES",
        || ids::PAINTER_RAIL_SHAPES,
        Fate::ReadBy("crates/ph2d-editor-core/src/screens/hero/chrome/rail_painter_tools.rs"),
    ),
    // ⭐⭐ **A pastilha de COR (`C&F`) tem DOIS consumidores, e o segundo é um gesto:** ela abre o
    // selector, e arrastá-la para o canvas é o *ColorDrop*. Quem lê o id é o `fill_drag` do shell.
    (
        "PAINTER_RAIL_FILL",
        || ids::PAINTER_RAIL_FILL,
        Fate::ReadBy("shells/desktop/src/input_dispatch/fill_drag.rs"),
    ),
    // ⛔ **O `⋯` e os pulldowns da ÁREA ficam FORA**, e não é esquecimento: eles não saem do
    // `rail_entries` — o `⋯` é acrescentado pelo `tool_bar::bar_split` quando a linha transborda, e
    // os pulldowns vêm do módulo que tem o canvas. Pô-los aqui e não na população faria o censo de
    // obsolescência acusá-los para sempre. Os gates deles vivem em
    // `the_tool_bar_never_grows_and_the_rest_is_behind_the_dots` e
    // `the_area_hands_its_commands_to_the_bar_and_the_app_menu`.
];

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Todos os chips que a fila PINTA, nos dois modos, **com o rótulo ao lado**.
///
/// ⚠️ **Derivado, nunca escrito à mão.** E o rótulo viaja porque um `NodeId` é um hash: uma lista de
/// números crus num `assert` obriga quem a lê a adivinhar de que chip se fala — *uma coluna sem
/// rótulo lê-se ao contrário*.
fn painted_chips() -> BTreeMap<NodeId, String> {
    let store = WidgetStore::default();
    let mut out = BTreeMap::new();
    for painter_active in [false, true] {
        for entry in rail_entries(&store, painter_active) {
            if let Some(id) = entry.node_id() {
                out.insert(id, entry.label().unwrap_or("?").to_string());
            }
        }
    }
    out
}

/// Os ids que a tabela cobre.
fn declared_chips() -> BTreeSet<NodeId> {
    RAIL_CONSUMERS.iter().map(|(_, id, _)| id()).collect()
}

/// ⭐⭐⭐ **O CONSUMIDOR DECLARADO EXISTE, e o ficheiro que o declara MENCIONA o chip.**
///
/// ⛔ Sem esta metade a tabela seria um comentário: quem apagasse o leitor deixaria a linha a
/// prometer um consumidor que já não existe, que é a doença exacta que este ficheiro cura.
#[test]
fn every_declared_consumer_really_reads_its_chip() {
    let root = workspace_root();
    let mut broken: Vec<String> = Vec::new();
    for (name, _, fate) in RAIL_CONSUMERS {
        let Fate::ReadBy(path) = fate else { continue };
        let full = root.join(path);
        let Ok(src) = std::fs::read_to_string(&full) else {
            broken.push(format!("{name}: o ficheiro `{path}` nao existe"));
            continue;
        };
        if !src.contains(name) {
            broken.push(format!(
                "{name}: `{path}` foi declarado como consumidor e nao menciona o chip"
            ));
        }
    }
    assert!(
        broken.is_empty(),
        "consumidores declarados que nao consomem:\n  {}\n\n\
         ⚠️ Ou o leitor mudou de sitio (corrija o caminho), ou ele DESAPARECEU — e nesse caso o chip \
         voltou a ser um controlo morto e a cura e' ligar, nao apagar esta linha.",
        broken.join("\n  ")
    );
}

/// ⭐⭐ **TODO chip pintado tem linha na tabela** — a metade que apanha o chip novo sem consumidor.
#[test]
fn every_painted_rail_chip_declares_what_reads_it() {
    let declared = declared_chips();
    let missing: Vec<String> = painted_chips()
        .into_iter()
        .filter(|(id, _)| !declared.contains(id))
        .map(|(_, label)| label)
        .collect();
    assert!(
        missing.is_empty(),
        "estes chips da fila nao dizem quem le' o valor deles: {missing:?}\n\n\
         ⚠️ Acrescente uma linha em `RAIL_CONSUMERS` — `ReadBy(<ficheiro>)` se alguem le', \
         `DeadOnPurpose(<motivo>)` se a ausencia e' uma DECISAO com mecanismo escrito.\n\
         ⛔ *«Ainda nao ligamos»* nao e' um motivo: e' a definicao de um controlo morto."
    );
}

/// ⭐⭐ **E o censo de OBSOLESCÊNCIA** — nenhuma linha descreve um chip que já não se pinta.
///
/// ⚠️ `CLAUDE.md` §5.0: *«uma catraca sem censo de obsolescência não desce: ela vira licença.»*
/// ⛔ E a metade tem de ser justa: os pulldowns da ÁREA são publicados por um MÓDULO e não aparecem
/// numa fila de store vazio, então eles ficam fora da população **e** da tabela — pô-los na tabela
/// e não na população faria este gate acusar toda a gente.
#[test]
fn no_row_describes_a_chip_the_rail_no_longer_paints() {
    let stale: Vec<&str> = RAIL_CONSUMERS
        .iter()
        .filter(|(_, id, _)| !painted_chips().contains_key(&id()))
        .map(|(name, _, _)| *name)
        .collect();
    assert!(
        stale.is_empty(),
        "estas linhas descrevem chips que a fila ja' nao pinta: {stale:?}\n\
         ⚠️ Apague-as — uma tolerancia que sobrevive ao alvo dela e' uma licenca."
    );
}

/// **Este motivo explica a ausência, ou só a declara?**
///
/// ⚠️ A barra é o COMPRIMENTO porque um mecanismo não cabe em duas palavras — é o mesmo critério das
/// justificações do `WIDGET_OPT_OUT` da galeria de widgets.
fn is_a_shrug(reason: &str) -> bool {
    reason.split_whitespace().count() < 8
}

/// ⭐⭐ **O CONTROLE POSITIVO da regra acima** — e ele é obrigatório por duas razões.
///
/// ⛔ **A primeira:** hoje a tabela tem **zero** `DeadOnPurpose`, então o laço do censo nunca entra
/// naquele braço — a regra estaria a ser afirmada sobre uma população vazia, *«verde por vácuo»*.
///
/// ⭐ **A segunda:** é ele que mantém a variante VIVA. Sem um construtor, o compilador acusa-a de
/// morta — e apagá-la deixaria as mensagens dos gates irmãos a mandar usar um vocabulário que já
/// não existe. *A cura de um aviso não é silenciá-lo; é dar-lhe o consumidor que falta.*
#[test]
fn the_shrug_detector_rejects_a_shrug_and_accepts_a_mechanism() {
    let shrug = Fate::DeadOnPurpose("ainda nao ligamos");
    let mechanism = Fate::DeadOnPurpose(
        "o gizmo 2D escolhe o verbo pela ALCA que se agarra, nao por um modo guardado, \
         entao nao ha' estado a ler",
    );
    let Fate::DeadOnPurpose(shrug) = shrug else {
        unreachable!()
    };
    let Fate::DeadOnPurpose(mechanism) = mechanism else {
        unreachable!()
    };
    assert!(
        is_a_shrug(shrug),
        "a regra deixou passar um encolher de ombros"
    );
    assert!(
        !is_a_shrug(mechanism),
        "a regra recusou um mecanismo escrito — a barra ficou alta demais para ser util"
    );
}

/// ⭐⭐⭐ **O CENSO, impresso — e a regra do que um «morto de propósito» pode dizer.**
///
/// ⭐ **Resultado de 2026-09-01: `0` de `20` chips da fila estão declarados mortos.** Antes desta
/// jornada eram quatro (`MOVE`/`ROT`/`SCALE` sem leitor nenhum, e o `SPACE`), e ninguém sabia —
/// porque nada perguntava.
///
/// ⛔ **Um `DeadOnPurpose` tem de trazer o MECANISMO da ausência**, não um encolher de ombros. É a
/// mesma barra que o `WIDGET_OPT_OUT` da galeria já aplica: *«ainda não ligámos»* descreve um
/// controlo morto, não uma decisão.
#[test]
fn the_census_prints_and_a_declared_death_carries_its_mechanism() {
    let mut alive = 0usize;
    let mut dead: Vec<&str> = Vec::new();
    let mut shrugs: Vec<&str> = Vec::new();
    for (name, _, fate) in RAIL_CONSUMERS {
        match fate {
            Fate::ReadBy(_) => alive += 1,
            Fate::DeadOnPurpose(reason) => {
                dead.push(name);
                if is_a_shrug(reason) {
                    shrugs.push(name);
                }
            }
        }
    }
    println!(
        "fila: {alive} chip(s) com consumidor, {} declarado(s) morto(s) {dead:?}",
        dead.len()
    );
    assert!(
        shrugs.is_empty(),
        "estes `DeadOnPurpose` nao explicam a ausencia, so' a declaram: {shrugs:?}\n\
         ⛔ Escreva o MECANISMO — por que e' que nada le' este valor, e por que isso esta' certo."
    );
}

/// ⚠️ **O controle positivo do parser da população.**
///
/// ⛔ Sem ele, um `rail_entries` que passasse a devolver vazio (mudou de nome, mudou de forma)
/// deixaria os dois gates acima **verdes por vácuo** — a doença que o
/// `every_menu_row_is_registered` já pagou uma vez.
#[test]
fn the_population_is_not_empty() {
    let n = painted_chips().len();
    assert!(
        n >= RAIL_CONSUMERS.len(),
        "a fila pintou {n} chips e a tabela tem {} linhas — o `rail_entries` deixou de devolver o \
         que devia, e os gates irmaos ficariam verdes sobre nada",
        RAIL_CONSUMERS.len()
    );
}
