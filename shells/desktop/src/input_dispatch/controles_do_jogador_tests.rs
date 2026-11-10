//! A tela «Controls» composta na shell (plano InputMap §4). A lei (mapa efectivo, relógio, Luau) prova-se na crate
//! `ph2d-player-controls`; aqui, a ORDEM das portas e os quatro leitores do Play.

/// A porta corre ANTES da tecla do jogo e de todo atalho; os quatro leitores do Play passam pelo mesmo acessor.
#[test]
fn the_gate_comes_before_the_game_key_and_the_four_readers_use_play_map() {
    let kb = include_str!("keyboard.rs");
    let gate = kb.find("self.controles_do_jogador(").expect("a porta");
    let game = kb.find("self.tecla_do_jogo(").expect("a tecla do jogo");
    let shortcuts = kb.find("self.handler.on_key(").expect("os atalhos");
    assert!(gate < game && gate < shortcuts);
    for src in [
        include_str!("../render_loop/fase_pointer_subjects.rs"),
        include_str!("../app_state_devices.rs"),
        include_str!("../render_loop/fase_vector_view_and_drives.rs"),
        include_str!("../render_loop/fase_signal_outbox.rs"),
    ] {
        assert!(
            src.contains("controles_do_jogador::play_map("),
            "um leitor do Play sem o mapa efectivo"
        );
        assert!(!src.contains("input_map, &self.input_actions"));
    }
    let play = include_str!("../undo_jogo.rs");
    assert!(play.contains("self.jogador_entra();") && play.contains("self.jogador_sai();"));
}
