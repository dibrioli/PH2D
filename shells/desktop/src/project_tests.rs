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
///
/// `timeline` são os bytes do `TimelineDoc` (vazio = projeto sem animação).
fn write_project_with(path: &std::path::Path, schema: u32, timeline: Vec<u8>) {
    let file = ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: Vec::new(),
        motion: String::new(),
        timeline,
    };
    let bytes = postcard::to_allocvec(&(schema, &file)).expect("serializa");
    std::fs::write(path, bytes).expect("grava o arquivo de projeto");
}

fn write_project(path: &std::path::Path, schema: u32) {
    write_project_with(path, schema, Vec::new());
}

/// O documento de uma animação: uma track em `hero`, com o `wire_id` (hash do nome) carimbado
/// como o save carimba. Devolve os bytes que o arquivo de projeto carregaria.
fn animation_of_hero() -> Vec<u8> {
    use ph2d_ecs::{Name, SimWorld};
    let mut sim = SimWorld::new();
    let hero = sim.world_mut().spawn(Name::new("hero")).id().to_bits();
    let mut timeline = ph2d_timeline::TimelineState::new();
    timeline.doc.upsert_key(
        hero,
        ph2d_timeline::PropKind::TranslationX,
        ph2d_anim::RationalTime::from_seconds(1.0),
        ph2d_timeline::AnimValue::Float(42.0),
        ph2d_timeline::Interp::Linear,
    );
    crate::timeline_persist::serialize(&mut timeline, sim.world()).expect("serializa a timeline")
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
/// A timeline do documento ANTERIOR não pode sobreviver ao Ctrl+O — e o perigo não é lixo, é
/// ADOÇÃO: as bindings dela nomeiam entidades que o `apply_project` acabou de despawnar → ficam
/// `missing` → e o `timeline_persist::upkeep` **reconecta binding órfã pelo hash do `Name`** (é o
/// que faz delete+undo curar a animação). Nomes se repetem entre projetos ("Layer 1",
/// "sprite_001"): sem este reset, a animação do projeto A adotaria os objetos homônimos do
/// projeto B no frame seguinte e passaria a dirigir a pose deles — uma animação que não está em
/// arquivo nenhum, sobre um projeto que nunca a teve, e com a fila de undo zerada pelo próprio
/// load. (O arquivo AGORA carrega a sua própria timeline — W4.T6/B5 — e é ela que entra no lugar:
/// ver `a_loaded_project_brings_its_animation_back_pending_the_name_heal`.)
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
        "o load tem de DESARMAR o baseline; o do documento anterior ficou pendurado, e o \
         primeiro diff do frame vira um passo espúrio"
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

/// **A ANIMAÇÃO SOBREVIVE AO FECHAR O APP** (W4.T6/B5) — e volta pronta para reencontrar os
/// objetos pelo nome.
///
/// Até aqui a timeline não era salva por caminho NENHUM: o "sidecar" que dizia salvá-la era
/// código morto (o Ctrl+S global de projeto retornava antes dele), então fechar o app perdia a
/// animação inteira. Agora o `TimelineDoc` viaja dentro do arquivo de projeto.
///
/// As bindings voltam **destacadas** (`entity == 0`, `missing`) de propósito: os bits de
/// entidade do save não valem nada nesta sessão — e, pior, poderiam colidir com um objeto
/// DIFERENTE e vivo. Quem as recola é o `upkeep` do frame, pelo hash do `Name` — a mesma
/// função que cura delete+undo, gateada em `timeline_persist`
/// (`the_animation_crosses_the_project_file_and_finds_its_objects_by_name`).
#[test]
fn a_loaded_project_brings_its_animation_back_pending_the_name_heal() {
    let mut app = headless_app();
    assert!(app.timeline.doc.bindings().is_empty(), "sessão em branco");

    let path = tmp_path("load_animation");
    write_project_with(&path, PROJECT_SCHEMA, animation_of_hero());
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    let bindings = app.timeline.doc.bindings();
    assert_eq!(
        bindings.len(),
        1,
        "a animação do arquivo entrou na sessão — o projeto reabre COM ela"
    );
    assert!(
        !bindings[0].wire_id.is_null(),
        "…carregando a identidade por NOME, que é o que sobrevive a um respawn"
    );
    assert!(
        bindings[0].missing && bindings[0].entity == 0,
        "…e DESTACADA: bits de outra sessão nunca podem colar num objeto vivo por acidente"
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
        timeline: animation_of_hero(),
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
    // A animação viaja como postcard do `TimelineDoc` — e volta LEGÍVEL do outro lado do
    // arquivo (bytes iguais não bastariam: o que importa é o documento reabrir).
    let back_timeline = crate::timeline_persist::install_from_project(&back.timeline)
        .expect("a animação volta legível do outro lado do arquivo");
    assert_eq!(
        back_timeline.doc.bindings().len(),
        1,
        "a animação preservada: uma track, do outro lado do arquivo"
    );
}

/// **Estopim de esquema.** O `ProjectState` embute o `FlipDoc` E a `VecScene` inteiros, e o
/// postcard é POSICIONAL: qualquer campo novo em qualquer struct deles muda o layout do
/// arquivo de projeto. Sem bump, o loader aceita o arquivo velho (a versão bate) e o lê com
/// o layout novo — sai geometria embaralhada, não um erro. Foi o que quase aconteceu na W4
/// (`holes`/`hide_stroke`).
///
/// Esta tripla existe para que bumpar UM sem pensar nos OUTROS fique vermelho. E o pin só
/// protege quem ele NOMEIA: enquanto ele era um par (só o Flip), um campo novo no
/// `VecVertex` teria bumpado o `VEC_SCENE_SCHEMA_VERSION`, deixado o `PROJECT_SCHEMA` para
/// trás, e **este teste teria passado**.
///
/// O `PROJECT_SCHEMA` é **14** — e não o 8 que esta linha trazia sozinha, nem o 9 que outras
/// duas traziam. Ele conta TODAS as quebras de layout do arquivo, de TODOS os módulos:
/// v3/v4 do Painter (documentos + impasto) · v5 do Motion (o grafo) · v6/v7 e v8/v9 do Flip
/// (o balde; depois `selected` + `offset`) · v10 do Vector (o `corner_radius` do `VecVertex`)
/// · v11/v12 do Painter (o `mats` do impasto, e o `mats` mudando de FORMA: 4 → 7 bytes) ·
/// v13 a timeline (5º campo do `ProjectFile`) · v14 a pose AFIM do Flip (W7.5:
/// `FlipFrame.offset: Vec2` → `pose: Pose([f32; 6])`, FLIP v5→6) · v15 a seleção no
/// domínio Point do Flip (W8: `FlipStroke.point_sel`, FLIP v6→7) · v16 os corpos de
/// física (ADR-0131 W1: `RigidBody`/`Collider` registrados → blobs novos nas linhas do
/// `WorldSnapshot`; nem o FlipDoc nem a VecScene mudaram, mas o layout do arquivo sim) ·
/// **v17** os campos `restitution`/`friction` APENDADOS ao `Collider` (ADR-0131 W2, a autoria
/// no Inspector). Nenhuma constante de esquema mudou, então **nenhum gate podia ver isto** —
/// postcard é posicional, e um save v16 lido como v17 devolveria lixo bem-formado.
///
/// Na integração de 2026-07-13, QUATRO linhas bumparam este contador ao mesmo tempo, cada uma
/// a partir do 7, cada uma por um motivo diferente. **O valor certo não existia em nenhum lado
/// do conflito: ele se CONTA.** Escolher um dos lados faria os saves das outras passarem na
/// checagem de versão e serem lidos com o layout errado — e postcard não tem nome de campo
/// para reclamar; ele devolve lixo bem-formado.
#[test]
fn a_schema_bump_anywhere_must_bump_the_project_schema() {
    assert_eq!(
        (
            PROJECT_SCHEMA,
            ph2d_flip::FLIP_SCHEMA_VERSION,
            ph2d_vec_scene::VEC_SCENE_SCHEMA_VERSION,
        ),
        (17, 7, 8),
        "a forma do FlipDoc ou da VecScene mudou (ou o esquema do projeto): suba o \
         PROJECT_SCHEMA junto e atualize esta tripla. Postcard nao avisa - ele so le errado."
    );
}

/// **O LOOP mora no CLIP — e o loop do projeto anterior não pode vazar pro novo.**
///
/// `NamedClip.loop_range` é salvo (DOC v3), mas o `Playhead` é só a cópia viva dele, e quem
/// publica a cópia é o `sync_transport_loop` — chamado por todo intent que troca de clip. Um LOAD
/// troca o documento inteiro sem passar por intent nenhum: sem publicar aqui, o loop salvo nunca
/// voltava e — pior — o transporte continuava repetindo sobre o intervalo de um arquivo que o
/// artista já fechou.
#[test]
fn a_load_publishes_the_clips_loop_and_never_leaks_the_previous_ones() {
    let mut app = headless_app();
    // A sessão anterior estava em loop [1, 2).
    app.playhead.set_loop(1.0, 2.0);
    assert_eq!(app.playhead.loop_range(), Some((1.0, 2.0)));

    // Um projeto SEM loop: o do anterior tem de morrer junto.
    let path = tmp_path("load_loop_clear");
    write_project_with(&path, PROJECT_SCHEMA, animation_of_hero());
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        app.playhead.loop_range(),
        None,
        "o loop do projeto ANTERIOR continuou armado sobre o projeto novo"
    );
}

/// Um documento de animação que este binário não sabe ler **RECUSA o arquivo inteiro** — não
/// abre o projeto sem a animação.
///
/// Abrir sem ela é a pior das opções: a cena aparece, parece certa, a timeline aparece vazia — e
/// o próximo Ctrl+S grava esse vazio POR CIMA do arquivo. A animação não sumiria por um bug;
/// sumiria porque o app abriu, mentiu e salvou. E a recusa acontece ANTES de qualquer mutação:
/// a sessão em curso fica intacta.
#[test]
fn an_unreadable_animation_refuses_the_whole_file_and_leaves_the_session_alone() {
    let mut app = headless_app();
    app.playhead.play();
    app.playhead.advance_ticks(120);
    let before = app.playhead.time();
    app.undo.push_undo(empty_state());

    let path = tmp_path("load_bad_timeline");
    write_project_with(&path, PROJECT_SCHEMA, vec![0xff, 0xff, 0xff]); // não é um TimelineDoc
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert_eq!(app.playhead.time(), before, "o relógio não foi tocado");
    assert!(
        app.undo.can_undo(),
        "o histórico do trabalho aberto sobreviveu"
    );
    assert!(
        app.timeline.doc.bindings().is_empty(),
        "e nada da animação do arquivo entrou pela metade"
    );
}
