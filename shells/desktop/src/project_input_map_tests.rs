//! **O INPUT MAP é documento** (v232): viaja no `ProjectState`; o undo simulado sem GPU pelas portas da `App`
//! (`do_hero`, o diff, `ao_hero`) e o `frame` REAL da aba devolvem o mapa BIT A BIT. As metades de texto e o
//! `game_id` provam-se em `ph2d-player-controls` (`r10a_tests.rs`).

use super::tests::empty_state;
use crate::undo::{ProjectState, ProjectUndo, SelectionMark};
use ph2d_app_input_map::view_list::state::{Scrub, Zone};
use ph2d_app_input_map::{InputMapEdit, InputMapTab, View};
use ph2d_editor_core::HeroScreen;
use ph2d_input::recipes::Recipe;
use ph2d_input::{Binding, GamepadButton, Key};

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

fn bytes(hero: &HeroScreen) -> Vec<u8> {
    postcard::to_allocvec(&hero.input_map).expect("o mapa serializa")
}

/// O hero com a aba instalada (sem `GameId` no mundo ⇒ sem ficheiro do jogador: nada toca o disco).
fn mesa() -> (HeroScreen, ph2d_ecs::World) {
    let mut h = hero();
    ph2d_app_input_map::install(&mut h, View::List);
    (h, ph2d_ecs::World::default())
}

fn tab(h: &mut HeroScreen) -> &mut InputMapTab {
    let t = h.documents.surfaces_mut().find_mut::<InputMapTab>();
    t.expect("a aba está instalada")
}

/// Um quadro da `App`: o `frame` aplica os pedidos; o passo só sem o arrasto vivo (`motivo_da_supressao`).
fn quadro(h: &mut HeroScreen, w: &ph2d_ecs::World, u: &mut ProjectUndo, b: &mut ProjectState) {
    let live = ph2d_input::InputState::default();
    let jogador = ph2d_player_controls::PlaySession::default();
    ph2d_app_input_map::frame(w, h, false, (&live, &jogador));
    if !ph2d_app_input_map::gesture_in_progress(h) {
        fecha_o_quadro(u, b, h);
    }
}

/// Ctrl+Z devolve o mapa de antes de uma ligação nova, e o Ctrl+Shift+Z volta a pô-la.
#[test]
fn a_new_binding_is_undone_and_redone() {
    let mut h = hero();
    let fabrica = h.input_map.clone();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    let jump = h.input_map.id("jump").expect("a fábrica tem `jump`");
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
}

/// ⭐ Cada pedido da aba é UM passo, e o Ctrl+Z devolve, bit a bit, o mapa de antes de cada um.
#[test]
fn every_tab_edit_is_one_step_and_undo_gives_back_the_exact_bytes() {
    let (mut h, w) = mesa();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    let jump = h.input_map.id("jump").expect("a fábrica tem `jump`");
    let open = Key::named_in_script("f2").expect("f2")[0];
    let pedidos = |novo: ph2d_input::ActionId| {
        vec![
            InputMapEdit::Add("dash_air".into()),
            InputMapEdit::AddBinding(novo, Binding::PadButton(GamepadButton::East)),
            InputMapEdit::SetZone(novo, 0.2, 0.6),
            InputMapEdit::RemoveBinding(jump, 0),
            InputMapEdit::SetPlayersCanChange(true),
            InputMapEdit::SetShown(novo, false),
            InputMapEdit::SetOpenKey(open),
            InputMapEdit::Remove(jump),
            InputMapEdit::StartFrom(Recipe::Shooter),
        ]
    };
    // O id que o `Add` vai dar: o mapa de fábrica cria-o pela mesma porta.
    let novo = h.input_map.clone().create("dash_air");
    let mut historia = vec![bytes(&h)];
    for (n, e) in pedidos(novo).into_iter().enumerate() {
        tab(&mut h).doc.pending.push(e.clone());
        quadro(&mut h, &w, &mut undo, &mut base);
        assert_eq!(undo.depth(), n + 1, "{e:?} é UM passo");
        assert_ne!(historia.last(), Some(&bytes(&h)), "{e:?} mudou o mapa");
        historia.push(bytes(&h));
    }
    let fim = historia.pop().expect("o estado final");
    for (n, antes) in historia.iter().enumerate().rev() {
        let (s, _) = undo.undo(foto(&h), SelectionMark::default()).unwrap();
        s.ao_hero(&mut h);
        assert_eq!(bytes(&h), *antes, "o Ctrl+Z n.º {n} devolveu outros bytes");
    }
    for _ in 0..historia.len() {
        let (s, _) = undo.redo(foto(&h), SelectionMark::default()).unwrap();
        s.ao_hero(&mut h);
    }
    assert_eq!(bytes(&h), fim, "o redo inteiro devolveu o mapa final");
    // O contador nunca desce: a próxima acção do mapa refeito não reusa nenhum id já dado.
    let mut m = h.input_map.clone();
    assert!(
        m.create("brand_new").0 > novo.0,
        "o `next_id` voltou com o mapa"
    );
}

/// ⭐ O arrasto de zona: três quadros de `SetZone` com o gesto vivo, soltar, e é UM passo.
#[test]
fn a_zone_drag_over_many_frames_is_one_step() {
    let (mut h, w) = mesa();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    let antes = bytes(&h);
    let jump = h.input_map.id("jump").expect("jump");
    tab(&mut h).doc.list.scrub = Some(Scrub {
        action: jump,
        zone: Zone::Dead,
        start_x: 0.0,
        start: 0.0,
        width: 100.0,
        moved: true,
    });
    for dead in [0.1, 0.2, 0.3] {
        let e = InputMapEdit::SetZone(jump, dead, 0.5);
        tab(&mut h).doc.pending.push(e);
        quadro(&mut h, &w, &mut undo, &mut base);
    }
    assert_eq!(undo.depth(), 0, "com o gesto vivo, nenhum passo");
    tab(&mut h).doc.list.scrub = None;
    quadro(&mut h, &w, &mut undo, &mut base);
    assert_eq!(undo.depth(), 1, "soltar: o arrasto inteiro é UM passo");
    let (s, _) = undo.undo(foto(&h), SelectionMark::default()).unwrap();
    s.ao_hero(&mut h);
    assert_eq!(
        bytes(&h),
        antes,
        "o Ctrl+Z devolve a zona de antes do arrasto"
    );
}

/// O «Forget them» mexe só no ficheiro do jogador: o mapa fica, e o undo não ganha passo.
#[test]
fn forget_them_is_not_an_undo_step() {
    let (mut h, w) = mesa();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    let antes = bytes(&h);
    tab(&mut h).doc.pending.push(InputMapEdit::ForgetPlayerKeys);
    quadro(&mut h, &w, &mut undo, &mut base);
    assert_eq!(bytes(&h), antes);
    assert_eq!(undo.depth(), 0);
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
