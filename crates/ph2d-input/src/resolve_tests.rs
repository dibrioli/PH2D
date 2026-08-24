//! Gates da RESOLUÇÃO — a lei dos dois números, o eixo por subtracção, e a ausência de memória.

use super::*;
use crate::action::Binding;
use crate::event::Event;
use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::keyboard::Key;

const LEFT: Key = Key(0xF702);
const RIGHT: Key = Key(0xF703);
const STICK: GamepadAxis = GamepadAxis::LeftStickX;

/// Um mapa com as duas metades do eixo de caminhada, cada uma ligada a uma seta.
fn walk_map() -> InputMap {
    let mut m = InputMap::new();
    let l = m.create("move_left");
    let r = m.create("move_right");
    m.get_mut(l).expect("existe").bindings.push(Binding::Key(LEFT));
    m.get_mut(r).expect("existe").bindings.push(Binding::Key(RIGHT));
    m
}

/// Um tique inteiro: fotografa o quadro anterior, aplica os eventos, resolve as acções.
fn tick(map: &InputMap, dev: &mut InputState, st: &mut ActionState, events: &[Event]) {
    dev.begin_frame();
    for e in events {
        dev.apply_event(*e);
    }
    st.tick(map, dev);
}

/// ⭐ **AS DUAS SEGURADAS DÃO ZERO** — a resposta que o jogador espera, e a que um acumulador
/// `+1`/`−1` não daria. É a lei que o `PlayerKeys::drive` da shell implementa à mão hoje, e que a
/// subtracção do Godot dá de graça.
#[test]
fn two_keys_held_give_zero() {
    let map = walk_map();
    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::KeyDown(LEFT), Event::KeyDown(RIGHT)],
    );

    assert_eq!(
        Input::new(&map, &st).axis("move_left", "move_right"),
        0.0,
        "as duas seguradas tem de dar zero"
    );
}

/// E soltar uma devolve a direcção da outra — a segunda metade da mesma lei, e a que um acumulador
/// com um `Up` perdido erraria.
#[test]
fn releasing_one_gives_back_the_other_direction() {
    let map = walk_map();
    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::KeyDown(LEFT), Event::KeyDown(RIGHT)],
    );
    tick(&map, &mut dev, &mut st, &[Event::KeyUp(RIGHT)]);

    assert_eq!(Input::new(&map, &st).axis("move_left", "move_right"), -1.0);
}

/// ⭐⭐ **A CORRECÇÃO À REFERÊNCIA, e o gate mede exactamente o que a separa dela.**
///
/// Com `dead_zone = 0,2` e `press_point = 0,6` existem **três** regimes, e o do meio é o que o
/// número de duplo propósito do Godot não consegue exprimir:
///
/// | cru | força | premida |
/// |---|---|---|
/// | `0,10` | `0` | `false` |
/// | ⭐ `0,40` | **`> 0`** | **`false`** |
/// | `0,80` | `> 0` | `true` |
#[test]
fn the_dead_zone_and_the_press_point_are_two_numbers() {
    let mut map = InputMap::new();
    let id = map.create("trigger");
    let a = map.get_mut(id).expect("existe");
    a.bindings.push(Binding::PadAxis {
        axis: STICK,
        positive: true,
    });
    a.set_zone(0.2, 0.6);

    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    let axis = |value: f32| Event::GamepadAxis { axis: STICK, value };

    tick(&map, &mut dev, &mut st, &[axis(0.10)]);
    let s = st.sample(id);
    assert_eq!(s.strength, 0.0, "abaixo da dead_zone a forca e' zero");
    assert!(!s.pressed, "e nao esta' premida");

    tick(&map, &mut dev, &mut st, &[axis(0.40)]);
    let s = st.sample(id);
    assert!(
        s.strength > 0.0,
        "acima da dead_zone a forca e' util (foi {})",
        s.strength
    );
    assert!(
        !s.pressed,
        "e AINDA NAO esta' premida -- e' este o regime que um numero so' nao exprime"
    );

    tick(&map, &mut dev, &mut st, &[axis(0.80)]);
    let s = st.sample(id);
    assert!(s.strength > 0.0);
    assert!(s.pressed, "acima do press_point esta' premida");
}

/// A normalização usa **todo** o curso acima da `dead_zone`: o fundo do curso continua a valer `1`.
/// Sem isto, uma `dead_zone` de `0,2` cortaria 20% do curso **no topo** também, e o jogador nunca
/// alcançaria a força máxima.
#[test]
fn the_far_end_of_the_stick_is_still_full_strength() {
    let mut map = InputMap::new();
    let id = map.create("push");
    let a = map.get_mut(id).expect("existe");
    a.bindings.push(Binding::PadAxis {
        axis: STICK,
        positive: true,
    });
    a.set_zone(0.2, 0.6);

    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::GamepadAxis {
            axis: STICK,
            value: 1.0,
        }],
    );

    assert_eq!(st.sample(id).strength, 1.0);
}

/// ⚠️ **Meia haste.** Um eixo empurrado para a esquerda dá **zero** à metade positiva — e é isso
/// que faz a subtracção dizer a verdade em vez de somar duas leituras do mesmo movimento.
#[test]
fn the_other_half_of_a_stick_contributes_nothing() {
    let mut map = InputMap::new();
    let pos = map.create("right");
    let neg = map.create("left");
    map.get_mut(pos)
        .expect("existe")
        .bindings
        .push(Binding::PadAxis {
            axis: STICK,
            positive: true,
        });
    map.get_mut(neg)
        .expect("existe")
        .bindings
        .push(Binding::PadAxis {
            axis: STICK,
            positive: false,
        });

    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::GamepadAxis {
            axis: STICK,
            value: -1.0,
        }],
    );

    assert_eq!(
        st.sample(pos).strength,
        0.0,
        "a metade positiva nao ve' o lado negativo"
    );
    assert_eq!(st.sample(neg).strength, 1.0);
    assert_eq!(Input::new(&map, &st).axis("left", "right"), -1.0);
}

/// N ligações: o **máximo** manda. Teclado *ou* comando *ou* a segunda tecla — qualquer um serve.
#[test]
fn many_bindings_take_the_strongest() {
    let mut map = InputMap::new();
    let id = map.create("jump");
    let a = map.get_mut(id).expect("existe");
    a.bindings.push(Binding::PadAxis {
        axis: STICK,
        positive: true,
    });
    a.bindings.push(Binding::Key(Key(0x5A)));

    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[
            Event::GamepadAxis {
                axis: STICK,
                value: 0.30,
            },
            Event::KeyDown(Key(0x5A)),
        ],
    );

    assert_eq!(
        st.sample(id).strength,
        1.0,
        "a tecla (1,0) tinha de vencer o analogico a 0,30"
    );
}

/// Um botão do comando e uma tecla na **mesma** acção — o caso que torna o código agnóstico ao
/// dispositivo, e o ponto inteiro da referência.
#[test]
fn a_pad_button_and_a_key_drive_the_same_action() {
    let mut map = InputMap::new();
    let id = map.create("jump");
    let a = map.get_mut(id).expect("existe");
    a.bindings.push(Binding::Key(Key(0x5A)));
    a.bindings.push(Binding::PadButton(GamepadButton::South));

    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::GamepadButtonDown(GamepadButton::South)],
    );
    assert!(st.sample(id).pressed, "o comando arma a accao");

    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::GamepadButtonUp(GamepadButton::South), Event::KeyDown(Key(0x5A))],
    );
    assert!(st.sample(id).pressed, "e a tecla tambem, sozinha");
}

/// ⭐ **Declarada e por atribuir NÃO é inexistente.** A acção aparece, responde `0`, e não está
/// premida — e é isso que deixa o painel oferecer *"agora escolha a tecla"* sem inventar um estado.
#[test]
fn an_action_with_zero_bindings_is_silent_not_absent() {
    let mut map = InputMap::new();
    let id = map.create("unbound");
    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(&map, &mut dev, &mut st, &[]);

    let input = Input::new(&map, &st);
    assert_eq!(input.id("unbound"), Some(id), "ela EXISTE");
    assert_eq!(input.strength("unbound"), 0.0);
    assert!(!input.pressed("unbound"));
}

/// Um nome que ninguém declarou lê como silêncio — nunca um `panic`. Um jogo não pode morrer porque
/// alguém escreveu mal uma acção.
#[test]
fn an_unknown_name_reads_as_silence() {
    let map = walk_map();
    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(&map, &mut dev, &mut st, &[]);
    let input = Input::new(&map, &st);

    assert_eq!(input.strength("no_such_action"), 0.0);
    assert!(!input.pressed("no_such_action"));
    assert!(!input.just_pressed("no_such_action"));
}

/// As bordas: uma vez, e só uma.
#[test]
fn the_edge_fires_once_and_only_once() {
    let map = walk_map();
    let (mut dev, mut st) = (InputState::new(), ActionState::new());

    tick(&map, &mut dev, &mut st, &[Event::KeyDown(RIGHT)]);
    assert!(
        Input::new(&map, &st).just_pressed("move_right"),
        "a borda de descida"
    );

    tick(&map, &mut dev, &mut st, &[]);
    assert!(
        !Input::new(&map, &st).just_pressed("move_right"),
        "segurar nao volta a disparar"
    );
    assert!(
        Input::new(&map, &st).pressed("move_right"),
        "mas continua premida"
    );

    tick(&map, &mut dev, &mut st, &[Event::KeyUp(RIGHT)]);
    assert!(
        Input::new(&map, &st).just_released("move_right"),
        "a borda de subida"
    );
}

/// ⛔ **O `Up` perdido, e a metade que ESTA crate resolve.**
///
/// A janela perde o foco a meio de uma corrida; o `Up` nunca chega. Sem o `FocusLost`, o
/// personagem anda para sempre — o modo de falha que o `player_input.rs` da shell nomeia no
/// doc-comment dele.
#[test]
fn losing_focus_releases_everything() {
    let map = walk_map();
    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::KeyDown(LEFT), Event::KeyDown(RIGHT)],
    );

    tick(&map, &mut dev, &mut st, &[Event::FocusLost]);

    let input = Input::new(&map, &st);
    for n in ["move_left", "move_right"] {
        assert_eq!(input.strength(n), 0.0, "{n} ficou com forca depois do FocusLost");
        assert!(!input.pressed(n));
    }
}

/// Resolver duas vezes sem eventos novos dá a **mesma** resposta — a camada de acções não acumula.
#[test]
fn resolving_twice_without_new_events_gives_the_same_answer() {
    let map = walk_map();
    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    let id = map.id("move_right").expect("existe");

    tick(&map, &mut dev, &mut st, &[Event::KeyDown(RIGHT)]);
    let first = st.sample(id);
    tick(&map, &mut dev, &mut st, &[]);
    let second = st.sample(id);

    assert_eq!(first, second);
}

/// Um eixo com valor não-finito lê como **repouso**, e não envenena a subtracção.
#[test]
fn a_non_finite_axis_reads_as_rest() {
    let mut map = InputMap::new();
    let id = map.create("push");
    map.get_mut(id)
        .expect("existe")
        .bindings
        .push(Binding::PadAxis {
            axis: STICK,
            positive: true,
        });

    let (mut dev, mut st) = (InputState::new(), ActionState::new());
    tick(
        &map,
        &mut dev,
        &mut st,
        &[Event::GamepadAxis {
            axis: STICK,
            value: f32::NAN,
        }],
    );

    assert_eq!(st.sample(id).strength, 0.0);
}
