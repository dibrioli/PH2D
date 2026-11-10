//! **O INPUT MAP é documento** (v232, plano InputMap §3–§4): viaja no `ProjectState` (undo e
//! ficheiro, um só dono) e o projecto tem um `game_id` próprio.
//!
//! O quadro do undo é simulado sem GPU pela MESMA porta da `App`: `do_hero` na fotografia, o diff,
//! `push_undo`, e `ao_hero` no restauro (`undo_app` só as chama — gate de texto abaixo).

use super::tests::empty_state;
use crate::undo::{ProjectState, ProjectUndo, SelectionMark};
use ph2d_editor_core::HeroScreen;

/// A fotografia do quadro: o resto do documento vazio, o hero tal como está.
fn foto(hero: &HeroScreen) -> ProjectState {
    let mut s = empty_state();
    s.do_hero(hero);
    s
}

/// O `post_frame_undo` em miniatura: um passo quando a fotografia difere da base.
fn fecha_o_quadro(undo: &mut ProjectUndo, base: &mut ProjectState, hero: &HeroScreen) {
    let agora = foto(hero);
    if agora != *base {
        undo.push_undo(std::mem::replace(base, agora), SelectionMark::default());
    }
}

fn hero() -> HeroScreen {
    HeroScreen::new(ph2d_editor_core::NodeId(7))
}

/// Ctrl+Z devolve o mapa de antes de uma ligação nova, e o Ctrl+Shift+Z volta a pô-la.
#[test]
fn a_new_binding_is_undone_and_redone() {
    let mut h = hero();
    let fabrica = h.input_map.clone();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    let jump = h
        .input_map
        .id("jump")
        .expect("o mapa de fábrica tem `jump`");
    let b = ph2d_input::Binding::PadButton(ph2d_input::GamepadButton::South);
    h.input_map.get_mut(jump).unwrap().bindings.push(b);
    let editado = h.input_map.clone();
    fecha_o_quadro(&mut undo, &mut base, &h);
    assert_eq!(undo.depth(), 1, "a ligação nova é UM passo");
    assert_eq!(base.parts_that_differ(&foto(&hero())), vec!["input_map"]);

    h.store.listen_for_binding(jump);
    let (antes, _) = undo.undo(foto(&h), SelectionMark::default()).unwrap();
    antes.ao_hero(&mut h);
    assert_eq!(h.input_map, fabrica, "o Ctrl+Z devolveu o mapa de antes");
    assert_eq!(
        h.store.input_map_listening(),
        None,
        "e calou a escuta armada"
    );
    let (depois, _) = undo.redo(foto(&h), SelectionMark::default()).unwrap();
    depois.ao_hero(&mut h);
    assert_eq!(h.input_map, editado, "o redo repôs a ligação");
}

/// «Start from… Shooter» sobre o mapa de fábrica é UM passo, e o Ctrl+Z devolve a fábrica inteira.
#[test]
fn starting_from_a_recipe_is_one_undo_step() {
    let mut h = hero();
    let fabrica = h.input_map.clone();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    h.input_map.start_from(ph2d_input::Recipe::Shooter);
    assert_ne!(h.input_map, fabrica, "a receita mudou o mapa");
    fecha_o_quadro(&mut undo, &mut base, &h);
    fecha_o_quadro(&mut undo, &mut base, &h);
    assert_eq!(undo.depth(), 1, "a receita inteira é UM passo");
    let (antes, _) = undo.undo(foto(&h), SelectionMark::default()).unwrap();
    antes.ao_hero(&mut h);
    assert_eq!(h.input_map, fabrica);
}

/// O «Create this action» do Trigger (`fase_criar_accao_do_mapa`) desfaz-se como qualquer edição.
#[test]
fn the_triggers_create_this_action_is_undoable() {
    use ph2d_editor_core::screens::hero::chrome::{PedidoDoMapa, aplica_pedidos_do_mapa};
    let mut h = hero();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    aplica_pedidos_do_mapa(&mut h, vec![PedidoDoMapa::Criar("open_door".into())]);
    assert!(
        h.input_map.id("open_door").is_some(),
        "a fábrica não tinha esta acção"
    );
    fecha_o_quadro(&mut undo, &mut base, &h);
    assert_eq!(undo.depth(), 1);
    let (antes, _) = undo.undo(foto(&h), SelectionMark::default()).unwrap();
    antes.ao_hero(&mut h);
    assert!(
        h.input_map.id("open_door").is_none(),
        "o Ctrl+Z tirou a acção criada pelo Trigger"
    );
    let fase = include_str!("render_loop/fase_criar_accao_do_mapa.rs");
    assert!(fase.contains("aplica_pedidos_do_mapa(hero, pedidos)"));
}

/// A `App` usa a MESMA porta que os testes acima: a fotografia e o restauro passam pelo hero.
#[test]
fn the_app_captures_and_restores_the_map_through_the_hero() {
    let src = include_str!("undo_app.rs");
    assert!(
        src.contains("foto.do_hero(h);"),
        "a fotografia não leva o mapa"
    );
    assert!(
        src.contains("state.ao_hero(h);"),
        "o restauro não devolve o mapa"
    );
    let load = include_str!("project_load.rs");
    assert!(load.contains("self.apply_project(&file.state);"));
    assert!(load.contains("insert_resource(crate::project::GameId(file.game_id))"));
    assert!(load.contains("self.input_actions = ph2d_input::ActionState::new();"));
}

/// Um projecto novo nasce com um `game_id` não-zero, e dois projectos novos não partilham jogo.
#[test]
fn a_new_project_draws_its_own_game_id() {
    use crate::project::{GameId, game_id_of};
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
}

/// ⛔⛔ **O MAPA TEM UM DONO SÓ**: o `HeroScreen`. A `App` guarda só o RESOLVIDO (`input_actions`).
#[test]
fn the_authored_map_has_exactly_one_holder() {
    let src = [
        include_str!("app_state.rs"),
        include_str!("app_state_gfx.rs"),
        include_str!("app_state_image_tools.rs"),
        include_str!("app_state_hero_live.rs"),
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

/// **Um Input Map AUTORADO**, cada campo deliberadamente não-default (a tela do jogador incluída):
/// um mapa vazio faria a ida-e-volta passar sobre o `Default` do serde.
pub(super) fn authored_input_map() -> ph2d_input::InputMap {
    use ph2d_input::{Binding, GamepadAxis, GamepadButton, InputMap, Key};
    let mut m = InputMap::new();
    let jump = m.create("jump");
    let a = m.get_mut(jump).expect("acabou de nascer");
    a.bindings.push(Binding::Key(Key(0x5A)));
    a.bindings.push(Binding::PadButton(GamepadButton::South));
    a.set_zone(0.15, 0.75);
    let right = m.create("move_right");
    m.get_mut(right)
        .expect("acabou de nascer")
        .bindings
        .push(Binding::PadAxis {
            axis: GamepadAxis::LeftStickX,
            positive: true,
        });
    let c = m.controls_mut();
    c.players_can_change_keys = true;
    c.open_key = Key(0x50);
    c.set_shown_in_game(right, false);
    m
}
