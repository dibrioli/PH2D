//! Os gates do *Add ▸ Text* (dono, 02/11).

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::Transform;
use ph2d_tool_vector::VectorTool;
use ph2d_vec_entities::entities::sync;
use ph2d_vec_entities::transform::world_transform;
use ph2d_vec_scene::{VecScene, rectangle};

fn tools() -> ToolRegistry {
    let mut t = ToolRegistry::new();
    t.register(Box::new(VectorTool::default()));
    t
}

/// Um HUD em `(4, 2)` à escala `2` e um filho dele.
fn hud(sim: &mut SimWorld) -> (Entity, Entity) {
    let mut t = Transform::from_translation(Vec2::new(4.0, 2.0));
    t.scale = Vec2::new(2.0, 2.0);
    let hud = sim.world_mut().spawn((UiCanvas::default(), t)).id();
    let kid = sim
        .world_mut()
        .spawn((Transform::default(), ChildOf(hud)))
        .id();
    (hud, kid)
}

/// ⭐⭐ GATE — **sem HUD seleccionado o texto nasce no centro da vista, sem pai**, e a ferramenta do
/// vetor fica pedida (nada nasce ainda: a sessão abre com ela na mão). Outra entrada não é desta.
///
/// (Mutação: tirar o `arm()` ⇒ RED.)
#[test]
fn without_a_hud_the_text_is_born_at_the_view_center() {
    let sim = SimWorld::default();
    let (mut t, mut armed) = (TextAdd::default(), false);
    let empty = ph2d_editor_core::object_add::EMPTY;
    assert!(!t.add(empty, &sim, None, [0.0; 2], || armed = true));
    assert!(!armed && t.request.is_none());
    assert!(t.add(TEXT, &sim, None, [1.5, -3.0], || armed = true));
    assert!(armed, "a ferramenta não ficou pedida");
    let r = t.request.expect("o pedido");
    assert_eq!((r.origin, r.parent), ([1.5, -3.0], None));
}

/// ⭐⭐ GATE — **com o HUD, ou um filho dele, seleccionado o texto nasce no centro do HUD e filho
/// dele.** O filho conta: o artista clica muitas vezes no rótulo que já está lá dentro.
///
/// (Mutação: `canvas_of` sem subir aos pais ⇒ RED no filho.)
#[test]
fn with_the_hud_or_a_child_of_it_selected_the_text_is_born_in_it() {
    let mut sim = SimWorld::default();
    let (hud, kid) = hud(&mut sim);
    for sel in [hud, kid] {
        let mut t = TextAdd::default();
        assert!(t.add(TEXT, &sim, Some(sel.to_bits()), [9.0, 9.0], || ()));
        let r = t.request.expect("o pedido");
        assert_eq!(r.parent, Some(hud.to_bits()), "o HUD de {sel:?}");
        assert_eq!(r.origin, [4.0, 2.0], "o centro do HUD");
    }
}

/// ⭐⭐ GATE — **a sessão abre UMA vez, e só com a ferramenta na mão** (o Add pede Object primeiro;
/// abrir antes seria abrir sem o Draw ▸ Text que a recebe).
#[test]
fn the_session_opens_once_and_only_with_the_tool_in_hand() {
    let sim = SimWorld::default();
    let (mut t, mut reg) = (TextAdd::default(), tools());
    assert_eq!(t.ready(false, &reg), None, "sem pedido");
    t.add(TEXT, &sim, None, [1.0, 2.0], || ());
    assert_eq!(t.ready(true, &reg), None, "ainda armada");
    assert_eq!(t.ready(false, &reg), None, "a ferramenta não está na mão");
    assert!(reg.set_active(&ToolId::new("vector")));
    assert_eq!(t.ready(false, &reg), Some([1.0, 2.0]));
    assert_eq!(t.ready(false, &reg), None, "abriu duas vezes");
}

/// ⭐⭐ GATE (dono, 02/11: «como no Figma») — **a sessão abre com «Text» e a 1.ª tecla troca a
/// palavra inteira**; a 2.ª já escreve a seguir; fechar a sessão esquece a troca.
///
/// (Mutação: `before_key` sem o `take` ⇒ cada tecla apaga ⇒ RED.)
#[test]
fn the_starting_word_is_replaced_by_the_first_key_only() {
    let (mut t, mut s) = (TextAdd::default(), String::from("old"));
    t.before_key(&mut s);
    assert_eq!(s, "old", "sem abrir não se troca");
    t.opened(&mut s);
    assert_eq!(s, "Text");
    t.before_key(&mut s);
    s.push('C');
    t.before_key(&mut s);
    s.push('o');
    assert_eq!(s, "Co");
    t.opened(&mut s);
    t.session_closed();
    t.before_key(&mut s);
    assert_eq!(s, "Text", "a troca sobreviveu à sessão");
}

/// ⭐⭐ GATE — **a entidade que nasce pendura-se no HUD SEM se mover** (o pai está a `(4, 2)` à
/// escala 2: um `ChildOf` cru atirava o texto para fora dele).
///
/// (Mutação: tirar o `reparent_keeping_world` ⇒ RED.)
#[test]
fn the_born_text_hangs_on_the_hud_without_moving() {
    let mut sim = SimWorld::default();
    let (hud, _) = hud(&mut sim);
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let id = scene.push_path(rectangle([4.0, 2.0], [1.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let e = Entity::from_bits(map[&id]);
    let before = world_transform(&sim, e).translation;
    let mut t = TextAdd::default();
    t.add(TEXT, &sim, Some(hud.to_bits()), [0.0; 2], || ());
    t.upkeep(&mut sim, &map, Some(id), true);
    assert!(t.request.is_some(), "por abrir: não é desta sessão");
    t.request.as_mut().unwrap().started = true;

    t.upkeep(&mut sim, &map, None, true);
    assert!(t.request.is_some(), "sem entidade ainda: espera");
    t.upkeep(&mut sim, &map, Some(id), true);
    assert_eq!(t.request, None, "o pedido acabou");
    assert_eq!(
        sim.world().get::<ChildOf>(e).map(ChildOf::parent),
        Some(hud)
    );
    let after = world_transform(&sim, e).translation;
    assert!((after - before).length() < 1e-4, "{before:?} → {after:?}");
}

/// **Sem sessão o pedido aberto morre** (o artista saiu antes de o texto ganhar entidade).
#[test]
fn a_dead_session_drops_the_request() {
    let mut sim = SimWorld::default();
    let mut t = TextAdd::default();
    t.add(TEXT, &sim, None, [0.0; 2], || ());
    t.request.as_mut().unwrap().started = true;
    t.upkeep(&mut sim, &VecEntityMap::new(), None, false);
    assert_eq!(t.request, None);
}
