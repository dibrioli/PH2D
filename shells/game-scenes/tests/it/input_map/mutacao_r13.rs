//! As quatro leis provadas por mutação (handoff da linha, «Provas de mutação»): o que cada uma promete ao artista
//! afirma-se aqui com o número ESCRITO À MÃO — os censos que calculam o esperado com `is_analog()` repetem a lei que
//! deviam vigiar e dão verde com ela partida.

use super::Mesa;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_input::{Binding, GamepadAxis, GamepadButton, InputMap, Key};
use ph2d_tokens::ColorToken;

/// O mapa de fábrica com uma acção ligada SÓ a `b` (como o gilrs entrega um gatilho: botão OU eixo).
fn com_gatilho(b: Binding) -> InputMap {
    let mut map = InputMap::with_player_defaults();
    let id = map.create("shoot");
    if let Some(a) = map.get_mut(id) {
        a.bindings.push(b);
    }
    map
}

fn mesa(view: View, map: &InputMap) -> Mesa {
    let mut m = Mesa::aberta(view);
    m.hero.input_map = map.clone();
    m.frame();
    m
}

fn numeros(m: &Mesa, name: &str) -> usize {
    let id = m.hero.input_map.id(name).expect("a acção existe");
    m.tab().doc.census.numbers_of(id)
}

fn abre(m: &mut Mesa, name: &str) {
    let a = m.hero.input_map.id(name).expect("a acção existe");
    m.click(m.at(KbHit::Row(a).id()));
    assert_eq!(m.tab().doc.kb.open, Some(a), "{name} aberta");
}

fn textos(m: &Mesa) -> String {
    let c = &m.tab().doc.census;
    c.texts
        .iter()
        .map(|t| t.1.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn avisos(m: &Mesa) -> (usize, usize) {
    let f = &m.tab().doc.census.filled;
    let n = |t: ColorToken| f.iter().filter(|x| x.1 == t).count();
    (
        n(ColorToken::InputMapWarningBg),
        n(ColorToken::InputMapWarning),
    )
}

/// ⭐ Lei 1: Dead zone / Press at só numa acção com manche OU gatilho — e o gatilho chega como BOTÃO
/// (`LeftTrigger`/`RightTrigger`) ou como EIXO. Nas três vistas, a acção do gatilho mostra os 2 números; a de
/// teclas, nenhum.
#[test]
fn o_gatilho_mostra_dead_e_press_nas_tres_vistas_e_a_tecla_nenhum() {
    for b in [
        Binding::PadButton(GamepadButton::RightTrigger),
        Binding::PadButton(GamepadButton::LeftTrigger),
        Binding::PadAxis {
            axis: GamepadAxis::RightTrigger,
            positive: true,
        },
    ] {
        let map = com_gatilho(b);
        let m = mesa(View::List, &map);
        assert_eq!(numeros(&m, "shoot"), 2, "A {b:?}: Dead zone e Press at");
        assert_eq!(numeros(&m, "jump"), 0, "A {b:?}: a tecla não tem números");
        let mut m = mesa(View::Keyboard, &map);
        m.click(m.at(KbHit::Sub(Sub::Gamepad).id()));
        abre(&mut m, "shoot");
        assert_eq!(numeros(&m, "shoot"), 2, "B {b:?}: o curso com Dead e Press");
        abre(&mut m, "jump");
        assert_eq!(numeros(&m, "jump"), 0, "B {b:?}: a tecla aberta, nenhum");
        let m = mesa(View::TryIt, &map);
        assert_eq!(numeros(&m, "shoot"), 2, "C {b:?}: o gatilho no círculo");
        assert_eq!(numeros(&m, "jump"), 0, "C {b:?}");
    }
    let m = mesa(View::TryIt, &InputMap::with_player_defaults());
    assert!(
        textos(&m).contains("No action uses a stick"),
        "C sem manche nem gatilho: {}",
        textos(&m)
    );
}

/// O mapa de fábrica com `J` em fire e em laser (ninguém os lê: a tecla repetida é por ENGANO). Sem manche: com ela,
/// o pé da A fala de Dead zone e Press at.
fn j_repetido() -> InputMap {
    let mut map = InputMap::with_player_defaults();
    for name in ["fire", "laser"] {
        let id = map.create(name);
        if let Some(a) = map.get_mut(id) {
            a.bindings.push(Binding::Key(Key(u32::from('J'))));
        }
    }
    map
}

/// ⭐ Lei 2: a tecla repetida por engano AVISA (frase de aviso, fundo e faixa âmbar) na A e no cartão aberto da B.
#[test]
fn a_tecla_repetida_por_engano_avisa_no_pe_da_lista_e_no_cartao() {
    let m = mesa(View::List, &j_repetido());
    let t = textos(&m);
    assert!(t.contains("J is in fire and laser."), "o pé: {t}");
    assert!(
        t.contains("Both react when it is pressed. Check that this is what you want."),
        "o pé: {t}"
    );
    let foot = m.tab().doc.census.foot[0];
    let f = &m.tab().doc.census.filled;
    assert!(
        f.iter()
            .any(|x| x.1 == ColorToken::InputMapWarningBg && x.0 == foot),
        "o fundo do aviso no pé: {f:?}"
    );
    assert!(
        f.iter().any(|x| x.1 == ColorToken::InputMapWarning
            && foot.contains(x.0.x, x.0.y)
            && x.0.w < foot.w),
        "a faixa do aviso dentro do pé: {f:?}"
    );
    let mut m = mesa(View::Keyboard, &j_repetido());
    abre(&mut m, "fire");
    let t = textos(&m);
    assert!(
        t.contains(
            "J is also in laser. Both can fire on the same press: give one of them another key."
        ),
        "o cartão aberto: {t}"
    );
    assert!(!t.contains("never fire together"), "não explica: {t}");
    let (bg, faixa) = avisos(&m);
    assert!(
        bg >= 2 && faixa >= 2,
        "o aviso da tecla e o de ninguém lê: {bg} {faixa}"
    );
}

/// ⭐ Lei 2: a de PROPÓSITO (fábrica: Up Arrow em jump e move_up, cada uma com o seu mover) EXPLICA, sem aviso.
#[test]
fn a_tecla_repetida_de_proposito_explica_sem_aviso() {
    let fab = InputMap::with_player_defaults();
    let m = mesa(View::List, &fab);
    let t = textos(&m);
    assert!(
        t.contains("a Platform Player and a Top-Down Player never run on the same object."),
        "o pé explica: {t}"
    );
    assert!(!t.contains("Check that this is what you want"), "{t}");
    assert_eq!(avisos(&m), (0, 0), "o pé da fábrica não é aviso");
    let mut m = mesa(View::Keyboard, &fab);
    abre(&mut m, "jump");
    let t = textos(&m);
    assert!(
        t.contains("is also in move_up.") && t.contains("never fire together."),
        "o cartão explica: {t}"
    );
    assert!(!t.contains("Both can fire on the same press"), "{t}");
    assert_eq!(avisos(&m), (0, 0), "o cartão da fábrica não avisa");
}

/// ⭐ Lei 2: a frase do hover numa tecla cujas acções ninguém lê diz-o («nobody reads it»), por acção.
#[test]
fn a_frase_do_hover_diz_que_ninguem_le_a_accao() {
    let mut m = mesa(View::Keyboard, &j_repetido());
    abre(&mut m, "fire");
    m.hover(m.at(KbHit::Key(Key(u32::from('J'))).id()));
    let t = textos(&m);
    assert!(
        t.contains("fire (nobody reads it) · laser (nobody reads it)."),
        "o hover: {t}"
    );
    let up = Key::named_in_script("up").expect("tecla com nome")[0];
    m.hover(m.at(KbHit::Key(up).id()));
    let t = textos(&m);
    assert!(
        !t.contains("nobody reads it") && t.contains("jump (Platform Player)"),
        "quem tem leitor diz quem: {t}"
    );
}

/// ⭐ Lei 3: um botão do comando acende a acção no Try it a 1.00 (o `raw` do PadButton chega à UI).
#[test]
fn um_botao_do_comando_acende_a_accao_no_try_it_a_um() {
    let mut map = InputMap::with_player_defaults();
    let jump = map.id("jump").expect("fábrica");
    if let Some(a) = map.get_mut(jump) {
        a.bindings.push(Binding::PadButton(GamepadButton::South));
    }
    let mut m = mesa(View::TryIt, &map);
    m.live.gamepad.handle_button_down(GamepadButton::South);
    m.frame();
    let s = m.tab().doc.tryit.actions.sample(jump);
    assert_eq!(s.strength.to_bits(), 1.0_f32.to_bits(), "o medidor a 1.00");
    assert!(s.pressed, "premida");
    m.live.gamepad.handle_button_up(GamepadButton::South);
    m.frame();
    assert_eq!(m.tab().doc.tryit.actions.sample(jump).strength, 0.0);
}
