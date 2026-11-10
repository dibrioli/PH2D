//! Gates do TECLADO — a forma é a do irmão gamepad, e as bordas têm de concordar com ela.

use super::*;

const A: Key = Key(0x41);
const B: Key = Key(0x42);

/// O protocolo do quadro: `begin_frame` fotografa, os eventos aplicam-se, e daí saem as bordas.
#[test]
fn the_edges_fire_once_each() {
    let mut k = KeyboardState::new();

    k.begin_frame();
    k.handle_key_down(A);
    assert!(k.pressed(A), "a borda de descida");
    assert!(k.held(A));
    assert!(!k.released(A));

    k.begin_frame();
    assert!(!k.pressed(A), "segurar nao volta a disparar a borda");
    assert!(k.held(A), "mas continua em baixo");

    k.begin_frame();
    k.handle_key_up(A);
    assert!(k.released(A), "a borda de subida");
    assert!(!k.held(A));
}

/// Uma tecla repetida (o auto-repeat do SO) não pode duplicar a entrada nem re-armar a borda.
#[test]
fn an_os_key_repeat_never_duplicates_the_key() {
    let mut k = KeyboardState::new();
    k.begin_frame();
    k.handle_key_down(A);
    k.handle_key_down(A);
    k.handle_key_down(A);

    assert_eq!(
        k.iter_held().count(),
        1,
        "a tecla entrou tres vezes na lista"
    );

    k.begin_frame();
    assert!(!k.pressed(A), "o repeat re-armou a borda");
}

/// Soltar uma tecla que nunca desceu é inócuo — a shell pode perder um par sem partir a lista.
#[test]
fn releasing_a_key_that_was_never_down_is_harmless() {
    let mut k = KeyboardState::new();
    k.handle_key_up(A);
    assert!(!k.held(A));
    assert_eq!(k.iter_held().count(), 0);
}

/// ⛔ **`release_all` é o que a janela sem foco chama** — e é a metade que esta crate pode resolver
/// do `Up` perdido que deixaria um personagem a andar sozinho para sempre.
///
/// ⚠️ **A fixtura tem de conter o fenómeno, e a primeira versão deste gate NÃO continha:** ela
/// descia e largava as teclas dentro do MESMO quadro, onde não existe borda nenhuma para ver —
/// uma tecla que nunca esteve em baixo numa fronteira de quadro não tem subida a reportar, e o
/// gate reprovava sobre produto correcto. O cenário real é o único que interessa: o jogador
/// **segura ao longo de quadros** e só então a janela perde o foco.
#[test]
fn release_all_empties_the_keyboard() {
    let mut k = KeyboardState::new();

    // Quadro 1: as duas descem.
    k.begin_frame();
    k.handle_key_down(A);
    k.handle_key_down(B);
    assert!(k.held(A) && k.held(B));

    // Quadro 2: continuam seguradas -- e' isto que poe as duas na fotografia do quadro anterior.
    k.begin_frame();
    assert!(
        k.held(A) && k.held(B),
        "a corrida atravessa a fronteira do quadro"
    );

    // Quadro 3: a janela perde o foco a meio da corrida.
    k.begin_frame();
    k.release_all();

    assert!(!k.held(A) && !k.held(B));
    assert_eq!(k.iter_held().count(), 0);
    // ⭐ E as bordas de subida APARECEM: quem largou tudo tem de poder ver que largou, senao um
    // `just_released` desaparece em silencio e quem o esperava fica a segurar para sempre.
    assert!(
        k.released(A) && k.released(B),
        "o release_all tem de produzir a borda de subida, e nao so' esvaziar a lista"
    );
}

/// A ordem de iteração é estável — não é arrumação: ela alimenta a resolução em acções, que
/// alimenta a fita determinística da física.
#[test]
fn the_held_order_is_stable_and_sorted() {
    let mut k = KeyboardState::new();
    k.handle_key_down(B);
    k.handle_key_down(A);
    let got: Vec<Key> = k.iter_held().collect();
    assert_eq!(
        got,
        vec![A, B],
        "a lista tem de sair ordenada, sempre igual"
    );
}

/// ⭐ **Um TOQUE dentro de um quadro não se perde** (Godot: `just_pressed` e `just_released` no
/// quadro seguinte — `docs/CodeEditor/05_api_que_funciona.md` §1). A diferença de dois retratos
/// nunca o via: a tecla não está em baixo em nenhuma fronteira.
#[test]
fn a_tap_inside_one_frame_gives_both_edges() {
    let mut k = KeyboardState::new();
    k.begin_frame();
    k.handle_key_down(A);
    k.handle_key_up(A);
    assert!(!k.held(A));
    assert!(k.pressed(A), "o toque perdeu a descida");
    assert!(k.released(A), "o toque perdeu a subida");
    k.begin_frame();
    assert!(!k.pressed(A) && !k.released(A), "a borda vive um quadro só");
}

/// ⭐ Todo nome de script volta à MESMA tecla, e todo código com rótulo tem nome de script — a
/// tabela é uma só (`NAMED` + letras, dígitos, F1..F12).
#[test]
fn every_named_key_round_trips_through_its_script_name() {
    let mut n = 0;
    for k in Key::all_named() {
        let name = k
            .script_name()
            .expect("toda tecla com rótulo tem nome de script");
        assert_eq!(Key::named_in_script(&name), Some(vec![k]), "{name}");
        assert!(!k.label().starts_with("Key "), "{name} sem rótulo");
        n += 1;
    }
    // 21 de antes + 7 sinais + 4 de navegação + 2 travas + 16 do numérico (plano §13.1).
    assert_eq!(n, 26 + 10 + 12 + 50);
    let names = Key::script_names();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len(), "nome repetido na tabela");
}

/// Os apelidos valem os dois lados; um nome desconhecido é `None` (o script di-lo como erro).
#[test]
fn aliases_name_both_sides_and_unknown_names_are_none() {
    assert_eq!(
        Key::named_in_script("shift"),
        Some(vec![Key(0xF710), Key(0xF711)])
    );
    assert_eq!(Key::named_in_script("left"), Some(vec![Key(0xF702)]));
    assert_eq!(Key::named_in_script("a"), Some(vec![A]));
    assert_eq!(Key::named_in_script("A"), None, "os nomes são minúsculas");
    assert_eq!(Key::named_in_script("spcae"), None);
    assert_eq!(
        Key(0xF702).label(),
        "Left Arrow",
        "o rótulo do artista não mudou"
    );
    assert_eq!(Key(0xF704 + 11).label(), "F12");
}

const KP7: Key = Key(0xF747);
const SEVEN: Key = Key(0x37);
const HOME: Key = Key(0xF729);

/// ⭐ Toda tecla do jogo vai e volta pelos três: rótulo → código (único), código → nome de script
/// → código. Um rótulo repetido faria a lista do artista mostrar duas linhas iguais.
#[test]
fn every_key_round_trips_label_code_and_script_name() {
    let all: Vec<Key> = Key::all_named().collect();
    for &k in &all {
        let label = k.label();
        let same: Vec<Key> = all.iter().copied().filter(|o| o.label() == label).collect();
        assert_eq!(same, vec![k], "o rótulo {label} não é de uma tecla só");
        let name = k.script_name().expect("nome de script");
        assert_eq!(Key::named_in_script(&name), Some(vec![k]), "{name}");
        assert_eq!(k.system_owned(), None, "{label} é do jogo");
    }
    for (code, label, name) in [
        (0x60, "`", "`"),
        (0x5C, "\\", "\\"),
        (0x2F, "/", "/"),
        (0xF727, "Insert", "insert"),
        (0xF72C, "Page Up", "pageup"),
        (0xF716, "Caps Lock", "capslock"),
        (0xF717, "Num Lock", "numlock"),
        (0xF74A, "Numpad .", "kp."),
        (0xF74E, "Numpad +", "kp+"),
        (0xF74F, "Numpad Enter", "kpenter"),
    ] {
        assert_eq!(Key(code).label(), label);
        assert_eq!(Key(code).script_name().as_deref(), Some(name));
    }
}

/// ⛔ Numpad 7 ≠ 7 ≠ Home: três códigos, três rótulos, três nomes (a tecla FÍSICA, com o Num Lock
/// aceso ou apagado). E Numpad Enter ≠ Enter, mas os dois CONFIRMAM.
#[test]
fn numpad_7_is_neither_7_nor_home() {
    let keys = [KP7, SEVEN, HOME];
    let labels: Vec<String> = keys.iter().map(|k| k.label()).collect();
    assert_eq!(labels, ["Numpad 7", "7", "Home"]);
    let names: Vec<Option<String>> = keys.iter().map(|k| k.script_name()).collect();
    assert_eq!(
        names,
        [Some("kp7".into()), Some("7".into()), Some("home".into())]
    );
    assert_eq!(Key::named_in_script("kp7"), Some(vec![KP7]));
    assert_ne!(Key::NUMPAD_ENTER, Key::ENTER);
    assert!(Key::NUMPAD_ENTER.is_enter_like() && Key::ENTER.is_enter_like());
    assert!(!SEVEN.is_enter_like() && !Key(0x20).is_enter_like());
}

/// ⛔ As do SISTEMA têm rótulo e legenda, mas NENHUM nome de script e um porquê i18n.
#[test]
fn system_keys_have_a_reason_and_no_script_name() {
    let sys: Vec<Key> = Key::system_keys().collect();
    assert_eq!(sys.len(), 4, "Left/Right Windows, Fn, Print Screen");
    let names = Key::script_names();
    for k in sys {
        assert_eq!(k.script_name(), None, "{}", k.label());
        assert!(!k.label().starts_with("Key "), "{:#X} sem rótulo", k.0);
        assert!(!k.cap().is_empty());
        let why = k.system_owned().expect("o porquê");
        assert!(why.starts_with("input_map_keyboard.system."), "{why}");
        assert!(!Key::all_named().any(|o| o == k), "fora da tabela do jogo");
        for n in [k.label().to_lowercase(), k.cap().to_lowercase()] {
            assert!(!names.contains(&n), "{n}");
            assert_eq!(Key::named_in_script(&n), None, "{n}");
        }
    }
    assert_eq!(
        Key(0xF71A).system_owned(),
        Some("input_map_keyboard.system.fn")
    );
    assert_eq!(
        Key(0xF718).system_owned(),
        Some("input_map_keyboard.system.windows")
    );
    assert_eq!(
        Key(0xF72E).system_owned(),
        Some("input_map_keyboard.system.print_screen")
    );
}

/// ⭐ Toda tecla da foto tem legenda, e a curta nunca é mais longa: as da foto e as dos compactos.
#[test]
fn every_key_of_the_photo_has_a_cap_and_a_short_cap() {
    for k in Key::all_named().chain(Key::system_keys()) {
        let (c, s) = (k.cap(), k.cap_short());
        assert!(!c.is_empty() && !s.is_empty(), "{} sem legenda", k.label());
        assert!(s.chars().count() <= c.chars().count(), "{c} / {s}");
    }
    for (code, cap, short) in [
        (0x1B, "Esc", "Esc"),
        (0x41, "A", "A"),
        (0x30, "0", "0"),
        (0xF704 + 11, "F12", "F12"),
        (0x08, "Backspace", "Bksp"),
        (0xF716, "CapsLk", "Caps"),
        (0xF717, "Num Lock", "Num"),
        (0xF727, "Insert", "Ins"),
        (0xF728, "Delete", "Del"),
        (0xF72C, "PgUp", "PgUp"),
        (0xF72D, "PgDn", "PgDn"),
        (0xF72E, "PrtSc", "PrtSc"),
        (0xF710, "Shift", "Shift"),
        (0xF718, "Win", "Win"),
        (0xF747, "7", "7"),
        (0xF74F, "Enter", "Enter"),
    ] {
        assert_eq!(
            (Key(code).cap(), Key(code).cap_short()),
            (cap, short),
            "{code:#X}"
        );
    }
    assert_eq!(Key(0x1234).cap(), "", "código sem tecla");
}
