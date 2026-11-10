//! **Auditoria (determinismo e persistência), InputMap v232**: cada `InputMapEdit` que a aba empurra passa pelo
//! `frame` REAL da família (`ph2d_app_input_map::frame`) e o quadro do undo por diff faz dele UM passo; o Ctrl+Z de
//! cada passo, pela ordem inversa, devolve o mapa BIT A BIT (postcard: acções, `next_id` e `controls`), e o redo
//! devolve o final. O arrasto de zona (`gesture_in_progress`) é UM passo; o «Forget them» não é passo nenhum.

use super::tests::empty_state;
use crate::undo::{ProjectState, ProjectUndo, SelectionMark};
use ph2d_app_input_map::view_list::state::{Scrub, Zone};
use ph2d_app_input_map::{InputMapEdit, InputMapTab, View};
use ph2d_editor_core::HeroScreen;
use ph2d_input::recipes::Recipe;
use ph2d_input::{Binding, GamepadButton, Key};

fn foto(hero: &HeroScreen) -> ProjectState {
    let mut s = empty_state();
    s.do_hero(hero);
    s
}

fn bytes(hero: &HeroScreen) -> Vec<u8> {
    postcard::to_allocvec(&hero.input_map).expect("o mapa serializa")
}

/// O hero com a aba instalada (sem `GameId` no mundo ⇒ sem ficheiro do jogador: nada toca o disco).
fn mesa() -> (HeroScreen, ph2d_ecs::World) {
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(7));
    ph2d_app_input_map::install(&mut h, View::List);
    (h, ph2d_ecs::World::default())
}

fn tab(h: &mut HeroScreen) -> &mut InputMapTab {
    h.documents
        .surfaces_mut()
        .find_mut::<InputMapTab>()
        .expect("a aba está instalada")
}

/// Um quadro da `App`: o `frame` da família aplica os pedidos; o undo regista um passo se o mapa mudou e o gate
/// (`motivo_da_supressao`, aqui só o braço do arrasto da aba) deixa.
fn quadro(
    h: &mut HeroScreen,
    w: &ph2d_ecs::World,
    undo: &mut ProjectUndo,
    base: &mut ProjectState,
) {
    let (live, jogador) = (
        ph2d_input::InputState::default(),
        ph2d_player_controls::PlaySession::default(),
    );
    ph2d_app_input_map::frame(w, h, false, (&live, &jogador));
    if ph2d_app_input_map::gesture_in_progress(h) {
        return;
    }
    let agora = foto(h);
    if agora != *base {
        undo.push_undo(std::mem::replace(base, agora), SelectionMark::default());
    }
}

/// ⭐ Cada pedido da aba é UM passo, e o Ctrl+Z devolve, bit a bit, o mapa de antes de cada um.
#[test]
fn every_tab_edit_is_one_step_and_undo_gives_back_the_exact_bytes() {
    let (mut h, w) = mesa();
    let (mut undo, mut base) = (ProjectUndo::default(), foto(&h));
    let jump = h
        .input_map
        .id("jump")
        .expect("o mapa de fábrica tem `jump`");
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
        let (s, _) = undo
            .undo(foto(&h), SelectionMark::default())
            .expect("há passo");
        s.ao_hero(&mut h);
        assert_eq!(bytes(&h), *antes, "o Ctrl+Z n.º {n} devolveu outros bytes");
    }
    for _ in 0..historia.len() {
        let (s, _) = undo
            .redo(foto(&h), SelectionMark::default())
            .expect("há redo");
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
        tab(&mut h)
            .doc
            .pending
            .push(InputMapEdit::SetZone(jump, dead, 0.5));
        quadro(&mut h, &w, &mut undo, &mut base);
    }
    assert_eq!(undo.depth(), 0, "com o gesto vivo, nenhum passo");
    tab(&mut h).doc.list.scrub = None;
    quadro(&mut h, &w, &mut undo, &mut base);
    assert_eq!(undo.depth(), 1, "soltar: o arrasto inteiro é UM passo");
    let (s, _) = undo
        .undo(foto(&h), SelectionMark::default())
        .expect("passo");
    s.ao_hero(&mut h);
    assert_eq!(
        bytes(&h),
        antes,
        "o Ctrl+Z devolve a zona de antes do arrasto"
    );
    // A `App` pergunta o MESMO gesto no gate do quadro (o braço do botão em baixo).
    let src = include_str!("undo_app_macros.rs");
    let gate = src.find("fn motivo_da_supressao(").expect("o gate");
    let braco = src[gate..].find("botao_do_rato_em_baixo").expect("o braço");
    assert!(
        src[gate..gate + braco].contains("ph2d_app_input_map::gesture_in_progress"),
        "o gate do quadro deixou de perguntar o arrasto da aba"
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
