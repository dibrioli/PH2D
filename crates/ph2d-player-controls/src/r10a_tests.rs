//! **A shell compõe a família Input Map** (R10a, saídas da shell pela catraca `the_shell_only_shrinks`): o
//! `game_id` e as costuras de TEXTO da shell — a ordem das portas, os cinco leitores do Play, a fotografia e o
//! restauro do undo, o dono único do mapa. As provas que precisam do `ProjectUndo` ficam na shell
//! (`project_input_map_tests.rs`); estas são as metades de texto delas.

use crate::{GameId, PlaySession, game_id_of, host};
use ph2d_input::{InputMap, Key};

/// Um ficheiro da shell, a partir de `shells/desktop/src` (o `include_str!` falha alto se ele mudar de sítio).
macro_rules! shell {
    ($p:literal) => {
        include_str!(concat!("../../../shells/desktop/src/", $p))
    };
}

/// Um projecto novo nasce com um `game_id` não-zero, e dois projectos novos não partilham jogo.
#[test]
fn a_new_project_draws_its_own_game_id() {
    let (mut a, mut b) = (bevy_ecs::world::World::new(), bevy_ecs::world::World::new());
    let id = game_id_of(&mut a);
    assert_ne!(id, 0);
    assert_eq!(
        game_id_of(&mut a),
        id,
        "sorteado UMA vez: a segunda pergunta dá o mesmo"
    );
    assert_ne!(
        game_id_of(&mut b),
        id,
        "dois projectos novos partilhariam as teclas do jogador"
    );
    // O load manda: o mundo passa a ter o do ficheiro.
    a.insert_resource(GameId(42));
    assert_eq!(game_id_of(&mut a), 42);
    assert!(shell!("project.rs").contains("ph2d_player_controls::game_id_of(g.sim.world_mut())"));
}

/// A porta corre ANTES da tecla do jogo e de todo atalho; os cinco leitores do Play passam pelo mesmo acessor.
#[test]
fn the_gate_comes_before_the_game_key_and_the_five_readers_use_play_map() {
    let kb = shell!("input_dispatch/keyboard.rs");
    let gate = kb.find("self.controles_do_jogador(").expect("a porta");
    let game = kb.find("self.tecla_do_jogo(").expect("a tecla do jogo");
    let shortcuts = kb.find("self.handler.on_key(").expect("os atalhos");
    assert!(gate < game && gate < shortcuts);
    for src in [
        shell!("render_loop/fase_pointer_subjects.rs"),
        shell!("app_state_devices.rs"),
        shell!("render_loop/fase_vector_view_and_drives.rs"),
        shell!("render_loop/fase_signal_outbox.rs"),
        shell!("render_loop/fase_physics_step.rs"),
    ] {
        assert!(
            src.contains("controles_do_jogador::play_map(") && !src.contains("|h| &h.input_map"),
            "um leitor do Play sem o mapa efectivo"
        );
        assert!(!src.contains("input_map, &self.input_actions"));
    }
    let play = shell!("undo_jogo.rs");
    assert!(play.contains("self.jogador_entra();") && play.contains("self.jogador_sai();"));
}

/// A pintura da tela corre a cada quadro e recebe o mapa do PROJECTO: é ali que a sessão compara e segue a edição
/// do autor no Play (`PlaySession::follow`), sem nenhuma linha a mais na shell.
#[test]
fn the_player_screen_paint_gets_the_project_map_every_frame() {
    let paint = shell!("render_loop/fase_hero_paint.rs");
    assert!(paint.contains("jogador.paint(_vp, &_hero.input_map,"));
}

/// A `App` usa a MESMA porta que as provas do undo: a fotografia e o restauro passam pelo hero.
#[test]
fn the_app_captures_and_restores_the_map_through_the_hero() {
    let src = shell!("undo_app.rs");
    assert!(
        src.contains("foto.do_hero(h);"),
        "a fotografia não leva o mapa"
    );
    assert!(
        src.contains("state.ao_hero(h);"),
        "o restauro não devolve o mapa"
    );
    let load = shell!("project_load.rs");
    assert!(load.contains("self.apply_project(&file.state);"));
    assert!(load.contains("insert_resource(crate::project::GameId(file.game_id))"));
    assert!(load.contains("self.input_actions = ph2d_input::ActionState::new();"));
}

/// A `App` pergunta o MESMO gesto da aba no gate do quadro (o braço do botão em baixo), e o «Create this
/// action» do Trigger passa pela porta do chrome (`fase_criar_accao_do_mapa`).
#[test]
fn the_frame_gate_asks_the_tab_drag_and_the_trigger_uses_the_chrome_port() {
    let src = shell!("undo_app_macros.rs");
    let gate = src.find("fn motivo_da_supressao(").expect("o gate");
    let braco = src[gate..].find("botao_do_rato_em_baixo").expect("o braço");
    assert!(
        src[gate..gate + braco].contains("ph2d_app_input_map::gesture_in_progress"),
        "o gate do quadro deixou de perguntar o arrasto da aba"
    );
    let fase = shell!("render_loop/fase_criar_accao_do_mapa.rs");
    assert!(fase.contains("aplica_pedidos_do_mapa(hero, pedidos)"));
}

/// ⛔⛔ **O MAPA TEM UM DONO SÓ**: o `HeroScreen`. A `App` guarda só o RESOLVIDO (`input_actions`).
#[test]
fn the_authored_map_has_exactly_one_holder() {
    let src = [
        shell!("app_state.rs"),
        shell!("app_state_gfx.rs"),
        shell!("app_state_image_tools.rs"),
        shell!("app_state_hero_live.rs"),
    ]
    .concat();
    let declares_field = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .any(|l| l.contains("input_map:"));
    assert!(!declares_field, "a `App` voltou a DECLARAR o Input Map");
    // O controle POSITIVO: sem o campo derivado, este gate lê o ficheiro errado.
    assert!(src.contains("input_actions:"));
}

/// A porta da shell ([`host`]): parado ou com o jogo fora da vista a tecla não é da tela; a tecla que a abre pausa
/// o relógio; aberta, toda tecla (até a sem nome) e todo clique são dela, e fechar retoma o relógio que corria.
#[test]
fn the_host_key_and_click_follow_the_screen() {
    const F1: Key = Key(0xF704);
    const ESC: Key = Key(0x1B);
    let mut m = InputMap::with_player_defaults();
    m.controls_mut().players_can_change_keys = true;
    let mut s = PlaySession::default();
    let at = Some(((0.0, 0.0), &m));
    assert_eq!(
        host::key(&mut s, Some(F1), (true, false), true, (Some(&m), true)),
        (false, None)
    );
    s.start(&m, None);
    assert_eq!(
        host::key(&mut s, Some(F1), (true, false), false, (Some(&m), true)),
        (false, None)
    );
    assert_eq!(
        host::click(&mut s, true, at, true),
        (false, None),
        "fechada"
    );
    let abre = host::key(&mut s, Some(F1), (true, false), true, (Some(&m), true));
    assert_eq!(abre, (true, Some(false)), "abre e pausa");
    assert_eq!(
        host::key(&mut s, None, (true, false), false, (None, true)),
        (true, None)
    );
    assert_eq!(
        host::key(&mut s, Some(ESC), (true, true), false, (Some(&m), true)),
        (true, None)
    );
    assert_eq!(
        host::click(&mut s, false, at, false),
        (true, None),
        "o rato é dela"
    );
    let fecha = host::key(&mut s, Some(ESC), (true, false), false, (Some(&m), false));
    assert_eq!(fecha, (true, Some(true)), "fecha e retoma");
    assert_eq!(
        host::key(&mut s, Some(F1), (true, false), true, (None, true)),
        (false, None)
    );
}
