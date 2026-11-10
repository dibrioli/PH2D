//! **C · Try it** pelas portas reais (plano §3): a receita com confirmação, o bench que acende as acções sem tocar
//! no projecto, o Play que fica com as teclas, a manche a 0.62, as caixas da tela do jogador e o «Forget them».

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::ph2d_player_controls::{self as pc, GameId};
use ph2d_app_input_map::view_try::state::TryHit;
use ph2d_app_input_map::{InputMapEdit, View};
use ph2d_input::recipes::Recipe;
use ph2d_input::{GamepadAxis, InputMap, Key, PlayerKeys};

fn key(name: &str) -> Key {
    Key::named_in_script(name).expect("tecla com nome")[0]
}

fn strength(m: &Mesa, name: &str) -> f32 {
    let id = m.hero.input_map.id(name).expect("acção");
    m.tab().doc.tryit.actions.sample(id).strength
}

fn texts(m: &Mesa) -> String {
    let c = &m.tab().doc.census;
    c.texts
        .iter()
        .map(|t| t.1.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn release(m: &mut Mesa, k: Key) -> bool {
    let took = ph2d_app_input_map::surface_raw_key(&mut m.hero, k.0, false, None);
    m.frame();
    took
}

#[test]
fn a_receita_pede_confirmacao_e_substitui_num_passo() {
    let mut m = Mesa::aberta(View::TryIt);
    m.hero.input_map = onze();
    m.frame();
    let before = m.hero.input_map.clone();
    m.click(m.at(TryHit::Recipe(Recipe::Platformer).id()));
    assert_eq!(m.tab().doc.tryit.confirm, Some(Recipe::Platformer));
    assert_eq!(m.hero.input_map, before, "pedir não troca nada");
    assert!(texts(&m).contains("Replace your 11 actions with Platformer? Undo brings them back."));
    m.physical(key("escape"));
    assert_eq!(m.tab().doc.tryit.confirm, None, "Esc fecha a pergunta");
    m.click(m.at(TryHit::Recipe(Recipe::Platformer).id()));
    // Replace: UM pedido, aplicado no mesmo quadro (o undo por diff faz dele UM passo). R3-C: o botão do registo
    // dispara no Up dentro (o Down só captura).
    let at = m.at(TryHit::Replace.id());
    let ts = &mut m.ts;
    if let Some((f, id)) = m.hero.documents.active_surface_mut() {
        for kind in [ph2d_host::PointerKind::Down, ph2d_host::PointerKind::Up] {
            let input = ph2d_editor_core::doc_surface::PointerInput {
                text: &mut *ts,
                mods: ph2d_host::Modifiers::default(),
                now_ns: 0,
            };
            f.pointer(
                id,
                input,
                kind,
                ph2d_host::PointerButton::Primary,
                at.0,
                at.1,
            );
        }
    }
    let pending = m.tab().doc.pending.clone();
    assert_eq!(pending, vec![InputMapEdit::StartFrom(Recipe::Platformer)]);
    m.frame();
    let mut want = before.clone();
    want.start_from(Recipe::Platformer);
    assert_eq!(m.hero.input_map, want);
    // O Ctrl+Z do projecto devolve o `ProjectState.input_map` de antes: o mapa volta inteiro.
    m.hero.input_map = before.clone();
    m.frame();
    assert_eq!(m.hero.input_map.len(), 11);
    // Mapa vazio: a receita entra sem perguntar.
    m.hero.input_map = InputMap::new();
    m.frame();
    m.click(m.at(TryHit::Recipe(Recipe::TopDown).id()));
    assert_eq!(m.hero.input_map.len(), 4);
}

#[test]
fn o_bench_acende_jump_e_move_up_sem_tocar_o_projecto() {
    let mut m = Mesa::aberta(View::TryIt);
    m.hero.input_map = onze();
    m.frame();
    let before = m.hero.input_map.clone();
    assert!(
        m.physical(key("up")),
        "o bench toma a tecla com o jogo parado"
    );
    m.frame();
    assert!((strength(&m, "jump") - 1.0).abs() < f32::EPSILON);
    assert!((strength(&m, "move_up") - 1.0).abs() < f32::EPSILON);
    assert_eq!(strength(&m, "move_left"), 0.0);
    let all = texts(&m);
    assert!(all.contains("You are pressing Up Arrow:"), "{all}");
    assert!(all.contains("jump and move_up are on"), "{all}");
    assert!(all.contains("The game is not running"));
    release(&mut m, key("up"));
    assert_eq!(strength(&m, "jump"), 0.0, "soltar devolve 0.00");
    assert_eq!(m.hero.input_map, before, "o bench nunca escreve no mapa");
    assert!(m.tab().doc.pending.is_empty());
}

#[test]
fn com_o_play_a_correr_o_bench_nao_toma_teclas() {
    let mut m = Mesa::aberta(View::TryIt);
    m.sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
    m.frame();
    assert!(!m.physical(key("up")), "a tecla é do jogo");
    m.live.keyboard.handle_key_down(key("up"));
    m.frame();
    assert!(
        (strength(&m, "jump") - 1.0).abs() < f32::EPSILON,
        "lê o estado do jogo"
    );
    assert!(
        m.tab().doc.tryit.held_keys().is_empty(),
        "o bench ficou vazio"
    );
    assert!(texts(&m).contains("The game is running"));
}

#[test]
fn a_manche_a_062_com_dead_015_da_055_e_premida() {
    let mut m = Mesa::aberta(View::TryIt);
    m.hero.input_map = onze();
    m.live.gamepad.handle_axis(GamepadAxis::LeftStickX, -0.62);
    m.frame();
    let want = ph2d_input::resolve::strength_of(0.62, 0.15);
    assert!(ph2d_input::resolve::pressed_of(0.62, 0.5));
    assert_eq!(
        strength(&m, "move_left").to_bits(),
        want.to_bits(),
        "a mesma lei, bit a bit"
    );
    let runs: Vec<&str> = (m.tab().doc.census.runs.iter())
        .map(|r| r.0.as_str())
        .collect();
    let want_s = format!("{want:.2}");
    assert_eq!(
        runs,
        vec!["Pushed", "0.62", "Strength", want_s.as_str(), "Pressed"]
    );
    assert_eq!(want_s, "0.55");
}

#[test]
fn as_caixas_e_a_tecla_que_abre_chegam_ao_mapa() {
    let mut m = Mesa::aberta(View::TryIt);
    m.click(m.at(TryHit::PlayersCanChange.id()));
    assert!(m.hero.input_map.controls().players_can_change_keys);
    let jump = m.hero.input_map.id("jump").expect("fábrica");
    m.click(m.at(TryHit::Shown(jump).id()));
    assert!(!m.hero.input_map.controls().shown_in_game(jump));
    m.click(m.at(TryHit::OpenKey.id()));
    m.physical(key("escape"));
    assert!(
        !m.tab().doc.tryit.listening_open_key,
        "Esc cancela a escuta"
    );
    assert_eq!(m.hero.input_map.controls().open_key, key("f1"));
    m.physical(key("f2"));
    assert_eq!(
        m.hero.input_map.controls().open_key,
        key("f1"),
        "cancelada, a tecla seguinte não muda nada"
    );
    m.click(m.at(TryHit::OpenKey.id()));
    m.physical(key("f5"));
    assert!(
        m.tab().doc.tryit.listening_open_key,
        "F5 recusado, a escuta continua"
    );
    assert!(texts(&m).contains("F5"), "a recusa com o porquê");
    m.physical(key("f2"));
    assert_eq!(m.hero.input_map.controls().open_key, key("f2"));
}

#[test]
fn forget_apaga_o_ficheiro_do_jogador() {
    let dir = std::env::temp_dir().join(format!("ph2d-try-forget-{}", std::process::id()));
    let mut m = Mesa::new(View::TryIt);
    m.tab_mut().doc.tryit.base = Some(dir.clone());
    m.sim.world_mut().insert_resource(GameId(7));
    let file = pc::player_file(&dir, 7);
    let mut keys = PlayerKeys::new();
    keys.set("jump", vec![ph2d_input::Binding::Key(key("x"))]);
    pc::file::save(&file, &keys).expect("grava");
    m.menu();
    assert!(
        texts(&m).contains("On this computer, a player changed 1 key."),
        "{}",
        texts(&m)
    );
    m.click(m.at(TryHit::Forget.id()));
    m.frame();
    assert!(!file.exists(), "o ficheiro saiu");
    assert!(!texts(&m).contains("a player changed"));
    let _ = std::fs::remove_dir_all(&dir);
}
