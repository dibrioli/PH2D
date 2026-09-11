//! Gates do dedo do jogador — a lei do eixo, e o que o mapa de fábrica promete.
//!
//! ⚠️ **Eles medem a RESOLUÇÃO, não a `App`**: montar uma `App` exige GPU, janela e atlas. O que
//! aqui se prova é a lei que o [`super::App::resolve_player_input`] aplica, sobre as mesmas portas
//! (`InputMap` + `Input`) — e o gate de arquitectura no fim confirma que é essa a lei que ele usa.

use ph2d_input::{
    ActionState, Event, Input, InputMap, InputState, Key, PLAYER_JUMP, PLAYER_MOVE_LEFT,
    PLAYER_MOVE_RIGHT,
};

const LEFT: Key = Key(0xF702);
const RIGHT: Key = Key(0xF703);
const Z: Key = Key(0x5A);

fn resolved(events: &[Event]) -> (InputMap, ActionState) {
    let map = InputMap::with_player_defaults();
    let mut dev = InputState::new();
    let mut st = ActionState::new();
    dev.begin_frame();
    for e in events {
        dev.apply_event(*e);
    }
    st.tick(&map, &dev);
    (map, st)
}

/// **O mapa de fábrica traz os SEIS verbos** — um projecto novo com mapa vazio seria o jogador a
/// deixar de andar, não um começo limpo.
#[test]
fn a_fresh_project_ships_the_six_player_actions() {
    let m = InputMap::with_player_defaults();
    for n in ["move_left", "move_right", "jump", "down", "dash", "grab"] {
        assert!(m.id(n).is_some(), "a accao `{n}` nao vem de fabrica");
    }
    assert_eq!(m.len(), 6, "e nao vem mais nada junto");
}

/// ⛔ **AS TECLAS DE FÁBRICA SÃO AS DE ONTEM, AO BIT.** Um default "melhor" aqui seria uma mudança
/// de produto escondida numa refactoração.
#[test]
fn the_factory_keys_are_the_ones_the_shell_had_hardcoded() {
    let m = InputMap::with_player_defaults();
    let has = |name: &str, k: u32| {
        m.get(m.id(name).expect("existe"))
            .expect("existe")
            .bindings
            .contains(&ph2d_input::Binding::Key(Key(k)))
    };
    assert!(
        has("move_left", 0xF702) && has("move_left", 0x41),
        "seta esquerda e A"
    );
    assert!(
        has("move_right", 0xF703) && has("move_right", 0x44),
        "seta direita e D"
    );
    assert!(has("jump", 0xF700) && has("jump", 0x5A), "seta cima e Z");
    assert!(has("down", 0xF701) && has("down", 0x53), "seta baixo e S");
    assert!(has("dash", 0x51), "Q");
    assert!(has("grab", 0x52), "R");
    // ⚠️ E o que NAO pode estar la': o Espaco e' o Play/Pause do transporte, e o `W` abre o painel
    // de MUNDO. Um platformer cujo pulo tambem pausa a cena e' uma tecla com dois donos.
    for (name, banned, why) in [
        ("jump", 0x20, "Espaco = Play/Pause do transporte"),
        ("jump", 0x57, "W = painel de MUNDO"),
    ] {
        assert!(
            !m.get(m.id(name).expect("existe"))
                .expect("existe")
                .bindings
                .contains(&ph2d_input::Binding::Key(Key(banned))),
            "`{name}` ficou ligado a uma tecla com outro dono: {why}"
        );
    }
}

/// ⭐ **AS DUAS SEGURADAS DÃO ZERO** — a lei que o `PlayerKeys` implementava à mão e que a
/// subtracção passa a dar de graça.
#[test]
fn holding_both_directions_gives_zero_drive() {
    let (m, st) = resolved(&[Event::KeyDown(LEFT), Event::KeyDown(RIGHT)]);
    assert_eq!(
        Input::new(&m, &st).axis(PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT),
        0.0
    );
}

/// E soltar uma devolve a direcção da outra — a metade que um acumulador com um `Up` perdido
/// erraria, deixando o personagem a andar para sempre.
#[test]
fn releasing_one_gives_back_the_other() {
    let map = InputMap::with_player_defaults();
    let mut dev = InputState::new();
    let mut st = ActionState::new();
    dev.begin_frame();
    dev.apply_event(Event::KeyDown(LEFT));
    dev.apply_event(Event::KeyDown(RIGHT));
    st.tick(&map, &dev);
    dev.begin_frame();
    dev.apply_event(Event::KeyUp(RIGHT));
    st.tick(&map, &dev);
    assert_eq!(
        Input::new(&map, &st).axis(PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT),
        -1.0
    );
}

/// **A tecla ALTERNATIVA vale tanto quanto a principal** — é o ponto inteiro de uma acção ter N
/// ligações, e o que faz o `Z` e a seta para cima serem o mesmo pulo.
#[test]
fn either_bound_key_arms_the_same_action() {
    for k in [Key(0xF700), Z] {
        let (m, st) = resolved(&[Event::KeyDown(k)]);
        assert!(
            Input::new(&m, &st).pressed(PLAYER_JUMP),
            "{k:?} tinha de armar o pulo"
        );
    }
}

/// ⛔⛔ **O JOGADOR LÊ O MAPA, e não teclas cravadas.**
///
/// ⚠️ Gate de arquitectura, e ele existe porque a regressão é **invisível**: alguém que reponha um
/// `match KeyCode::…` aqui devolve o comportamento de hoje **exactamente**, e a janela do Input Map
/// passa a ser decoração — todos os outros gates ficam verdes.
#[test]
fn the_player_resolves_the_map_and_never_a_hardcoded_key() {
    let src = include_str!("player_input.rs");
    assert!(
        src.contains("input.axis(PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT)"),
        "o `drive` deixou de sair da subtraccao de duas ACCOES"
    );
    assert!(
        !src.contains("KeyCode::"),
        "voltou uma tecla CRAVADA ao dedo do jogador -- a janela do Input Map passa a ser decoracao"
    );
    // O controle POSITIVO: se o ficheiro encolher, este gate passa a afirmar sobre quase nada.
    assert!(
        src.len() > 1500,
        "o ficheiro tem {} bytes: este gate parou de olhar para o produto",
        src.len()
    );
}
