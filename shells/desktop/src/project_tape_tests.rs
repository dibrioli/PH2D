//! **A CORRIDA GRAVADA no arquivo de projeto** (W17) — filho de `project_tests`
//! (declarado lá via `#[path]`), então `super::*` alcança as fixtures dele.
//!
//! O último item aberto do §4 do plano 06 (*"Persistir a fita (W7)"*), e o que o
//! torna útil é o **bake da W16**: a fita é a entrada que o bake replaya, então
//! uma corrida que morre com a sessão só pode ser assada no dia em que foi
//! jogada.

use super::*;

/// A corrida do gate: os três botões em períodos diferentes, um eixo que varre o
/// sinal. Uma fita de zeros passaria por qualquer tradução.
fn a_recorded_run() -> ph2d_physics_ecs::InputTape {
    let mut t = ph2d_physics_ecs::InputTape::new();
    for k in 1..=90u64 {
        t.record(
            k,
            ph2d_physics_ecs::PlayerInput {
                drive: (k as f32 - 45.0) / 45.0,
                jump: k % 2 == 0,
                down: k % 3 == 0,
                dash: k % 5 == 0,
            },
        );
    }
    t
}

/// **A CORRIDA GRAVADA sobrevive ao arquivo** (W17) — o último item aberto do §4
/// do plano 06, e o que o torna útil é o **bake da W16**: a fita é a entrada que
/// o bake replaya, então reabrir um projeto e apertar Bake devolve a corrida de
/// ontem.
///
/// ⚠️ **Ele dirige o `project_load_from` com um arquivo de verdade no disco**, e
/// não uma cópia da decisão: um campo que o `ProjectFile` carrega e que o load
/// esquece de instalar compila, salva certo, e perde toda corrida em silêncio.
///
/// ⚠️ **Tique a tique, e não só o comprimento.** Uma instalação que perdesse o
/// `first` daria uma fita do mesmo tamanho descrevendo a corrida noutro instante.
#[test]
fn a_recorded_run_survives_the_file() {
    use ph2d_physics_ecs::PlayerInputAtTick;

    let mut app = headless_app();
    // A sessão em curso tem uma corrida OUTRA — é ela que o load tem de esquecer.
    app.player_tape.record(
        7,
        ph2d_physics_ecs::PlayerInput {
            drive: -1.0,
            ..Default::default()
        },
    );

    let path = tmp_path("recorded_run");
    let mut saved = a_recorded_run();
    let file = ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: Vec::new(),
        motion: String::new(),
        timeline: Vec::new(),
        physics: Default::default(),
        tokens: Vec::new(),
        sculpt: Vec::new(),
        baked_forms: Vec::new(),
        player_tape: saved.to_wire(),
    };
    std::fs::write(
        &path,
        postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).unwrap(),
    )
    .unwrap();
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert_eq!(
        app.player_tape.len(),
        saved.len(),
        "a corrida do arquivo nao chegou a sessao"
    );
    for k in 0..=100u64 {
        assert_eq!(
            app.player_tape.input(k),
            saved.input(k),
            "o tique {k} da corrida carregada difere do que estava no arquivo"
        );
    }
}

/// **E um projeto SEM corrida esquece a da sessão anterior.**
///
/// ⚠️ Ele não é o mesmo gate acima ao contrário: aquele passaria com um load que
/// FUNDISSE as duas fitas (a nova cobriria a velha tique a tique, já que a
/// gravada é mais longa). Uma fita costurada com a da sessão anterior descreveria
/// uma corrida que ninguém deu — e este é o gate que separa *instalar* de
/// *fundir*, a mesma lei que o `project_forget` aplica ao relógio e à timeline.
#[test]
fn loading_a_project_without_a_run_forgets_the_one_in_the_session() {
    let mut app = headless_app();
    app.player_tape = a_recorded_run();
    assert!(!app.player_tape.is_empty(), "a sessao tem de comecar cheia");

    let path = tmp_path("no_run");
    write_project(&path, PROJECT_SCHEMA);
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);

    assert!(
        app.player_tape.is_empty(),
        "a corrida do documento ANTERIOR sobreviveu ao load: {} tiques",
        app.player_tape.len()
    );
}

/// **O SAVE grava a fita VIVA, não uma vazia** — arch-gate sobre o fonte.
///
/// ⚠️ **Nenhum gate de comportamento alcança isto**, e é a mesma cicatriz que o
/// `project_save` já carrega escrita: ele exige `gfx` (janela + GPU), então o
/// harness headless não o dirige. O modo de falha é exato e silencioso —
/// `TapeWire::default()` no lugar do `to_wire()` compila, passa os dois gates
/// acima (que constroem o arquivo à mão) e perde toda corrida que alguém jogar.
#[test]
fn the_save_writes_the_live_tape() {
    let src = include_str!("project.rs");
    // ⚠️ **Ancorado na CONSTRUÇÃO, não no nome do campo.** A primeira ocorrência
    // de `player_tape:` no arquivo é a DECLARAÇÃO do struct, e um gate que a
    // lesse ficaria verde afirmando uma coisa sobre a outra.
    let at = src
        .find("let file = ProjectFile {")
        .expect("o save constroi um `ProjectFile`");
    let rest = &src[at..];
    let body = &rest[..rest.find("\n        };").expect("a construcao fecha")];
    assert!(
        body.len() > 200,
        "o scanner leu {} bytes: a construcao mudou de forma e este gate parou de olhar \
         para o produto",
        body.len()
    );
    assert!(
        body.contains("player_tape: self.player_tape.to_wire()"),
        "o save nao grava a fita VIVA da sessao. Construcao:\n{body}"
    );
}
