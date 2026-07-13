//! Gates do save/load de PROJETO (split do `project.rs` pelo cap de 600 LOC do HR-18; declarado
//! lá como irmão via `#[path]`, então `super` é `project`).
//!
//! O fio condutor destes gates: **um load faz a sessão ESQUECER o documento anterior**. O `App`
//! é dirigível sem janela (ver `headless_app`), então o que roda aqui é a função real do Ctrl+O
//! — com um arquivo de verdade no disco —, e não uma cópia da decisão.

use super::*;
use crate::undo::ProjectState;
use ph2d_ecs::scene::WorldSnapshot;
use ph2d_vec_scene::{VecScene, rectangle};

/// Um `App` **sem janela** — e não um dublê dele.
///
/// `App::new()` é headless por construção: no winit 0.30 a janela nasce no `resumed`, então
/// `window`/`host`/`gfx` começam em `None` e é assim que o app roda o seu primeiro frame de
/// verdade. Todo passo do load que precisa de `gfx` (mundo, atlas, Painter, grafo) já degrada
/// para no-op — de propósito, desde que foi escrito. O que sobra é exatamente a decisão de
/// SESSÃO, que é o que estes gates observam.
fn headless_app() -> crate::App {
    let app = crate::App::new();
    assert!(
        app.gfx.is_none() && app.window.is_none(),
        "o App nasce sem janela — se isto mudar, estes gates precisam de outro harness"
    );
    app
}

/// O estado vazio (o projeto em branco) — serve de passo de undo e de conteúdo de arquivo.
fn empty_state() -> ProjectState {
    ProjectState {
        world: WorldSnapshot::new(),
        vec: VecScene::new(),
        flip: ph2d_flip::FlipDoc::new(),
    }
}

/// Grava um arquivo de projeto em `path` com o esquema `schema`. Passar
/// `PROJECT_SCHEMA` produz um arquivo que o loader ACEITA; qualquer outro número
/// produz um que ele RECUSA — os dois caminhos que os gates abaixo separam.
fn write_project(path: &std::path::Path, schema: u32) {
    let file = ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: Vec::new(),
        motion: String::new(),
    };
    let bytes = postcard::to_allocvec(&(schema, &file)).expect("serializa");
    std::fs::write(path, bytes).expect("grava o arquivo de projeto");
}

/// Um caminho temporário por gate (os testes correm em paralelo, no mesmo processo —
/// um arquivo compartilhado, ou uma env var, seria a corrida que não estamos testando).
fn tmp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ph2d_gate_{name}_{}.postcard", std::process::id()))
}

/// **A costura do Ctrl+O: um projeto carregado rebobina o relógio do editor e começa um
/// histórico novo.**
///
/// Esta asserção já existiu — como *"a loaded document starts at its own tick 0"*, em
/// `motion_state_tests` — e morreu no **W4.T7**, quando o Motion perdeu o transporte
/// próprio: ela afirmava um campo que deixou de existir. A rebobinada subiu para o
/// `App::project_load` (o `Playhead` é do editor, não do Motion) e ficou **sem gate**,
/// com a justificativa de que "o `App` exige janela". **Não exige** — ver
/// [`headless_app`].
///
/// O gate dirige a função que o **Ctrl+O** chama, com um arquivo de verdade no disco. Não
/// é uma cópia da decisão posta num helper que ninguém invoca
/// (*pintado ≠ populado*): se o reset sair de `project_load_from`, isto fica vermelho.
///
/// FALSIFICADO por remover qualquer uma das linhas do reset — sem o `rewind` o documento
/// novo abre num relógio herdado do anterior (e o Motion, cujo pump nasce vazio, abre a
/// cena no meio: gate irmão
/// `motion_state_tests::a_clock_that_was_not_rewound_opens_the_document_mid_scene`); sem o
/// `pause` ele abre no início e sai CORRENDO (o `rewind` preserva o play state de
/// propósito); sem o `ProjectUndo::default()` um Ctrl+Z desfaz o projeto que acabou de ser
/// carregado para dentro de um estado do projeto ANTERIOR.
///
/// **O que este gate NÃO cobre** (e nenhum gate headless cobre — exige um `App` com `gfx`,
/// isto é, com janela e GPU): os passos do load que dependem de `gfx` — mundo, atlas,
/// documentos do Painter, grafo de Motion. Eles são no-op aqui. Um deles morde de volta:
/// ver `the_load_re_arms_the_undo_baseline_from_the_world_not_from_the_file`.
#[test]
fn a_loaded_project_rewinds_the_clock_and_starts_a_fresh_history() {
    let mut app = headless_app();

    // A sessão EM CURSO — o que o Ctrl+O encontra: dois segundos de play no relógio e um
    // passo no histórico do documento que está aberto.
    app.playhead.play();
    app.playhead.advance_ticks(120);
    let before = app.playhead.time();
    assert!(before > 0.0, "o relógio tem de estar ADIANTADO: {before}");
    app.undo.push_undo(empty_state());
    assert!(app.undo.can_undo(), "…e o histórico, cheio");

    let path = tmp_path("load_rewinds");
    write_project(&path, PROJECT_SCHEMA);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        app.playhead.time(),
        0.0,
        "um projeto carregado começa no INÍCIO: o relógio ficou em {}",
        app.playhead.time()
    );
    assert!(
        !app.playhead.is_playing(),
        "…e começa PARADO: o `rewind` preserva o play state, então um projeto aberto \
         durante o play sairia correndo no frame seguinte (o boot pausa pela mesma razão)"
    );
    assert!(
        !app.undo.can_undo(),
        "documento novo, histórico novo — o Ctrl+Z não pode alcançar o projeto anterior"
    );
}

/// **O load esquece a TIMELINE do documento anterior** — e isto não é higiene, é o
/// antídoto de uma corrupção silenciosa.
///
/// A timeline ainda não é persistida (W4.T6/B5), então ela SOBREVIVE ao Ctrl+O. As
/// bindings dela nomeiam entidades que o `apply_project` acabou de despawnar → ficam
/// `missing` → e o `timeline_persist::upkeep` **reconecta binding órfã pelo hash do
/// `Name`** (é o que faz delete+undo curar a animação). Nomes se repetem entre projetos
/// ("Layer 1", "sprite_001"): sem este reset, a animação do projeto A adota os objetos
/// homônimos do projeto B no frame seguinte e passa a dirigir a pose deles — uma animação
/// que não está em arquivo nenhum, sobre um projeto que nunca a teve, e com a fila de undo
/// zerada pelo próprio load.
///
/// (A dupla `autokey` + `timeline_insert_key` vai junto pelo mesmo motivo do
/// `MotionState::install`: são pins e pedidos keyados por bits de entidade que morreram.)
#[test]
fn a_load_forgets_the_previous_documents_timeline() {
    use ph2d_anim::RationalTime;
    use ph2d_timeline::{AnimValue, Interp, PropKind, TimelineIntent};
    let mut app = headless_app();

    // O documento anterior tinha animação: um objeto bound, uma key, um intent na fila.
    app.timeline.doc.upsert_key(
        7, // bits de uma entidade do projeto ANTERIOR
        PropKind::TranslationX,
        RationalTime::from_seconds(0.5),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    assert!(!app.timeline.doc.bindings().is_empty());
    app.timeline_intents.push(TimelineIntent::DeleteSelection);
    app.timeline_insert_key = true;

    let path = tmp_path("load_forgets_timeline");
    write_project(&path, PROJECT_SCHEMA);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert!(
        app.timeline.doc.bindings().is_empty(),
        "as bindings do projeto ANTERIOR sobreviveram ao load — no próximo frame o \
         `upkeep` as reconecta, por NOME, aos objetos do projeto novo"
    );
    assert!(
        app.timeline_intents.is_empty(),
        "um intent enfileirado contra o documento morto seria aplicado ao novo"
    );
    assert!(!app.timeline_insert_key, "…e um K pendente idem");
}

/// **O baseline do undo sai do MUNDO, não do arquivo — e é a última palavra do load.**
///
/// Sem isto a fila "nova" nasce com um passo dentro: o `apply_project` arma o baseline com
/// o estado do ARQUIVO, o `restore_painted_docs` mexe no mundo DEPOIS (textura individual
/// nova por sprite pintado — `Sprite` é componente registrado, logo entra no snapshot), e o
/// `post_frame_undo` roda no MESMO frame (o Ctrl+O é input) e registra a diferença. O
/// primeiro Ctrl+Z do artista então não desfazia a ação dele — devolvia um `texture_id`
/// morto.
///
/// **Limite honesto:** o VALOR do baseline só é observável com `gfx` (mundo + GPU), que
/// exige janela. Headless o `capture_project()` devolve `None`, e é justamente isso que
/// torna este gate afiado: `None` só pode ter vindo DESTA linha — se ela sumir, o baseline
/// do projeto ANTERIOR fica lá (o `apply_project`, sem `gfx`, nem chega a escrever). O que
/// fica sem gate é o valor em produção; o mecanismo está rastreado no comentário do load.
#[test]
fn the_load_re_arms_the_undo_baseline_from_the_world_not_from_the_file() {
    let mut app = headless_app();
    app.undo_baseline = Some(empty_state()); // o baseline do documento ANTERIOR

    let path = tmp_path("load_rearms_baseline");
    write_project(&path, PROJECT_SCHEMA);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert!(
        app.undo_baseline.is_none(),
        "o load tem de RE-ARMAR o baseline a partir do mundo (headless: `None`); o baseline \
         do documento anterior ficou pendurado, e o primeiro diff do frame vira um passo"
    );
}

/// …e um load **RECUSADO** não encosta na sessão.
///
/// O reset pertence ao caminho de SUCESSO. Se fosse um blanket no topo da função, abrir um
/// caminho errado — ou um save de esquema velho, que o loader recusa de propósito — jogaria
/// fora o relógio e o **histórico de undo do trabalho que está aberto**, por causa de um
/// arquivo que nem chegou a ser lido.
#[test]
fn a_refused_load_leaves_the_clock_and_the_history_alone() {
    let mut app = headless_app();
    app.playhead.play();
    app.playhead.advance_ticks(120);
    let before = app.playhead.time();
    app.undo.push_undo(empty_state());

    // (a) o arquivo não existe.
    app.project_load_from(&tmp_path("nunca_gravado").to_string_lossy());
    // (b) o arquivo existe, mas é de outra era do esquema (postcard é posicional: ler é
    //     que seria o erro).
    let path = tmp_path("refused");
    write_project(&path, PROJECT_SCHEMA + 1);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        app.playhead.time(),
        before,
        "load recusado NÃO rebobina o relógio"
    );
    assert!(
        app.undo.can_undo(),
        "…nem joga fora o histórico do documento que continua aberto"
    );
}

/// O arquivo de projeto sobrevive ao round-trip postcard: geometria, versão e os
/// pixels embutidos voltam idênticos.
#[test]
fn project_file_round_trips_through_postcard() {
    let mut vec = VecScene::new();
    vec.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    // ADR-0114: o FlipDoc entra no ProjectState (3º campo) → o save o carrega
    // de graça (mesma captura do undo).
    let mut flip = ph2d_flip::FlipDoc::new();
    flip.push_object("Anim");
    let state = ProjectState {
        world: WorldSnapshot::new(),
        vec,
        flip,
    };
    // O grafo de Motion viaja como TEXTO canônico — a forma real que o `MotionDoc`
    // serializa (doc 56), não uma string inventada: se o formato mudar, o teste viaja
    // junto em vez de mentir que sobreviveu.
    let motion = ph2d_motion_doc::MotionDoc::new().to_text();
    let file = ProjectFile {
        state: state.clone(),
        assets: vec![SavedAsset {
            key: 16,
            width: 2,
            height: 2,
            rgba: vec![10, 20, 30, 40],
        }],
        painted: Vec::new(),
        motion: motion.clone(),
    };
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).unwrap();
    let (ver, back): (u32, ProjectFile) = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(ver, PROJECT_SCHEMA);
    assert_eq!(back.state, state, "estado (mundo + geometria) preservado");
    assert_eq!(back.assets.len(), 1);
    assert_eq!(back.assets[0].key, 16);
    assert_eq!(
        back.assets[0].rgba,
        vec![10, 20, 30, 40],
        "pixels preservados"
    );
    assert_eq!(back.motion, motion, "o grafo de Motion preservado");
    assert!(
        ph2d_motion_doc::MotionDoc::from_text(&back.motion).is_ok(),
        "…e ainda parseável do outro lado do arquivo"
    );
}

/// **Estopim de esquema.** O `ProjectState` embute o `FlipDoc` inteiro, e o
/// postcard é POSICIONAL: qualquer campo novo em qualquer struct do Flip muda
/// o layout do arquivo de projeto. Sem bump, o loader aceita o arquivo velho
/// (a versão bate) e o lê com o layout novo — sai geometria embaralhada, não
/// um erro. Foi o que quase aconteceu na W4 (`holes`/`hide_stroke`).
///
/// Este par existe para que bumpar UM sem pensar no OUTRO fique vermelho.
///
/// O `PROJECT_SCHEMA` é 7 (e não o 4 que a linha Flip trazia sozinha) porque na
/// árvore integrada ele conta TODAS as quebras de layout, não só as do Flip: v3/v4
/// do Painter (documentos + impasto), v5 do Motion (o grafo), v6/v7 do Flip. Cada
/// linha subiu o mesmo contador por um motivo diferente — e o contador é um só.
#[test]
fn a_flip_schema_bump_must_bump_the_project_schema() {
    assert_eq!(
        (PROJECT_SCHEMA, ph2d_flip::FLIP_SCHEMA_VERSION),
        (7, 3),
        "a forma do FlipDoc mudou (ou o esquema do projeto): suba o PROJECT_SCHEMA \
         junto e atualize este par. Postcard nao avisa - ele so le errado."
    );
}
