//! Os FUROS do Vector ▸ Edit (ordem do dono, 05/10: *«vamos corrigir os furos»*), pelo MESMO quadro
//! que a shell corre e pelas portas que ela chama — e em DOIS quadros onde o programa os espalha (a
//! forma consumida só perde a entidade no `sync` do quadro seguinte).

use super::tests::{Cena, cena};
use super::*;
use ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows;
use ph2d_vec_edit::PenClick;
use ph2d_vec_entities::entities::{sync, view_state_for_pick};
use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VecViewState};

/// Um traço aberto de `a` a `b` (o que a caneta deixa).
fn linha(c: &mut Cena, a: [f64; 2], b: [f64; 2]) -> VecPathId {
    c.scene.push_path(VecPath {
        verts: vec![VecVertex::corner(a), VecVertex::corner(b)],
        stroke: Some(StrokeSpec::new(Rgba8::new(9, 9, 9, 255), 0.02)),
        ..VecPath::default()
    })
}

fn sincroniza(c: &mut Cena) {
    sync(&mut c.sim, &mut c.scene, &mut c.vec.entities);
}

/// Um clique da caneta em `p` (premir + soltar), como o artista o dá.
fn clique(c: &mut Cena, p: [f64; 2]) -> PenClick {
    let click = c.vec.pen.on_press(&mut c.scene, p, 0.01, false, &mut |q| q);
    c.vec.pen.on_release();
    click
}

/// ⭐⭐ GATE (furo 1, `…_O_VETOR.md` §7a) — **o Soldar em Edit deixa a REDE em Edit, com o gizmo**: a
/// solda escreve a rede no traço mais ao fundo e CONSOME o outro; com o Edit trancado no consumido,
/// o Edit passa à rede (a [`heir`]) em vez de cair a Object com ela seleccionada.
/// CONTROLO: a solda de facto consumiu um traço e deixou uma rede de quatro arcos.
/// (Mutação: o `heir` da família devolver `None` ⇒ o Edit cai a Object ⇒ RED.)
#[test]
fn the_weld_leaves_the_net_in_edit_with_its_gizmo() {
    let mut c = cena();
    let ia = linha(&mut c, [0.0, 0.0], [2.0, 2.0]);
    let ib = linha(&mut c, [0.0, 2.0], [2.0, 0.0]);
    sincroniza(&mut c);
    let (a, b) = (c.vec.entities[&ia], c.vec.entities[&ib]);
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(b), "o activo é o último");
    // Os dois traços pela caneta (o Weld trabalha sobre a selecção dela).
    c.vec.pen.select_many(&[ia, ib]);
    c.quadro(None);
    let xf = ph2d_vec_entities::transform::build(&c.sim, &c.vec.entities);
    crate::weld::apply_vec_weld(&mut c.scene, &mut c.vec.pen, &xf, 0.0);
    assert_eq!(
        c.scene.paths().len(),
        1,
        "controlo: a solda não consumiu o traço"
    );
    assert_eq!(c.scene.path(ia).map(VecPath::contour_count), Some(4));
    assert_eq!(
        c.vec.pen.selected_paths(),
        [ia],
        "controlo: a caneta fica com a rede"
    );
    c.quadro(None);
    sincroniza(&mut c);
    // A shell copia a caneta para a selecção (`vec_selection::sync_selection`).
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a), "a rede não ficou em Edit");
    assert!(c.na_mao(), "o Soldar largou a ferramenta");
    assert!(object_gizmo_shows(&c.hero), "a rede ficou sem gizmo");
    assert_eq!(
        c.avisos_de_entrada(),
        1,
        "a passagem à rede repetiu o aviso"
    );
}

/// ⭐⭐ GATE (furo 2, `…_O_VETOR.md` §7b) — **a caneta DENTRO do Edit continua o caminho aberto**:
/// premir a PONTA de um traço do Edit retoma-o (o mesmo objecto ganha o vértice, nenhum nasce), e
/// premir depois a ponta de OUTRO traço aberto junta-o ao primeiro — o juntado, que era o trancado,
/// some, e o Edit passa ao que ficou.
/// CONTROLO: um clique longe das pontas começa OUTRO objecto, e o Edit passa a ele (`newborn`).
#[test]
fn the_pen_in_edit_continues_the_open_path_and_joins_another() {
    let mut c = cena();
    c.quadro(None);
    c.vec.edit.arm();
    c.quadro(None);
    assert!(c.na_mao());
    assert_eq!(clique(&mut c, [0.0, 0.0]), PenClick::Started);
    assert_eq!(clique(&mut c, [1.0, 0.0]), PenClick::Added);
    c.vec.pen.finish();
    sincroniza(&mut c);
    c.quadro(None);
    let l1 = *c.vec.entities.keys().next().expect("o 1.º traço");
    let e1 = c.vec.entities[&l1];
    assert_eq!(c.em_edit(), Some(e1));
    assert_eq!(
        clique(&mut c, [1.0, 0.0]),
        PenClick::Grabbed,
        "a ponta não retomou o traço"
    );
    assert_eq!(clique(&mut c, [1.0, 1.0]), PenClick::Added);
    c.vec.pen.finish();
    sincroniza(&mut c);
    c.quadro(None);
    assert_eq!(
        c.scene.paths().len(),
        1,
        "a continuação nasceu outro objecto"
    );
    assert_eq!(c.scene.path(l1).map(|p| p.verts.len()), Some(3));
    assert_eq!(c.em_edit(), Some(e1), "a continuação largou o Edit");
    // CONTROLO: longe das pontas nasce outro objecto, e o Edit passa a ele.
    assert_eq!(clique(&mut c, [3.0, 0.0]), PenClick::Started);
    assert_eq!(clique(&mut c, [4.0, 0.0]), PenClick::Added);
    c.vec.pen.finish();
    sincroniza(&mut c);
    c.quadro(None);
    let l2 = *c
        .vec
        .entities
        .keys()
        .find(|id| **id != l1)
        .expect("o 2.º traço");
    let e2 = c.vec.entities[&l2];
    assert_eq!(
        c.em_edit(),
        Some(e2),
        "controlo: o traço novo não pediu o Edit"
    );
    // Retomar o 1.º (é do Edit: o Edit é do tipo) e juntar-lhe o 2.º pela ponta dele.
    assert_eq!(clique(&mut c, [1.0, 1.0]), PenClick::Grabbed);
    clique(&mut c, [3.0, 0.0]);
    c.vec.pen.finish();
    assert_eq!(c.scene.paths().len(), 1, "o 2.º traço não se juntou ao 1.º");
    assert_eq!(c.scene.path(l1).map(|p| p.verts.len()), Some(5));
    c.quadro(None);
    sincroniza(&mut c);
    c.hero.gizmo.replace_selection(Some(e1));
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        Some(e1),
        "o Edit não passou ao traço que ficou"
    );
    assert!(c.na_mao(), "juntar largou a ferramenta");
}

/// ⭐⭐ GATE (furo 4, `…_O_VETOR.md` §7c) — **o press do Width e do Trim em Edit, pelas portas que a
/// shell chama** (`width_handles::press_at`, `trim::hit_at`, com a vista do clique que ela publica):
/// a OUTRA forma do Edit responde (o Edit é do tipo), o Edit não cai, e uma forma TRAVADA não
/// responde — nem ao Width quando a Hierarquia a deixou seleccionada.
/// CONTROLO: longe de toda curva nenhuma das duas responde.
/// (Mutação: o `press_at` não filtrar a selecção pelo `is_pickable` ⇒ a travada ganha uma parada ⇒ RED.)
#[test]
fn width_and_trim_pressed_in_edit_reach_the_shapes_and_skip_a_locked_one() {
    let mut c = cena();
    let ia = linha(&mut c, [0.0, 0.0], [2.0, 2.0]);
    let ib = linha(&mut c, [0.0, 2.0], [2.0, 0.0]);
    sincroniza(&mut c);
    let (a, b) = (c.vec.entities[&ia], c.vec.entities[&ib]);
    c.quadro(None);
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), Some(a));
    let vista = |c: &mut Cena| {
        let v = view_state_for_pick(&c.sim, &c.vec.entities, &VecViewState::default());
        c.vec.pen.set_view(v);
    };
    vista(&mut c);
    let r = 0.05;
    let em_b = [0.5, 1.5];
    let press = |c: &mut Cena, p| {
        crate::width_handles::press_at(&mut c.vec.pen, &mut c.sim, &c.scene, &c.vec.entities, p, r)
    };
    assert!(
        press(&mut c, [5.0, 5.0]).is_none(),
        "controlo: o Width respondeu no vazio"
    );
    assert!(
        press(&mut c, em_b).is_some(),
        "o Width não respondeu na outra forma do Edit"
    );
    assert_eq!(c.vec.pen.selected(), Some(ib));
    let xf = ph2d_vec_entities::transform::build(&c.sim, &c.vec.entities);
    assert!(crate::trim::hit_at(&c.vec.pen, &c.scene, &xf, [5.0, 5.0], r).is_none());
    let hit = crate::trim::hit_at(&c.vec.pen, &c.scene, &xf, [0.25, 1.75], r);
    assert_eq!(
        hit.map(|h| h.path),
        Some(ib),
        "o Trim não respondeu na outra forma do Edit"
    );
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(a), "o press largou o Edit");
    // TRAVADA (e ainda seleccionada pela Hierarquia): nem o Width nem o Trim.
    c.sim
        .world_mut()
        .entity_mut(Entity::from_bits(b))
        .insert(ph2d_ecs::Locked);
    vista(&mut c);
    c.vec.pen.select(Some(ib));
    assert!(
        press(&mut c, em_b).is_none(),
        "o Width deu uma parada à forma travada"
    );
    let hit = crate::trim::hit_at(&c.vec.pen, &c.scene, &xf, [0.25, 1.75], r);
    assert!(hit.is_none(), "o Trim respondeu na forma travada");
}

/// ⭐ GATE (dono, 05/10) — **o Node que falha o vetor escolhe o objecto de OUTRO tipo por baixo**,
/// só num modo do TIPO: o 1.º dos hits que não é forma. CONTROLO: só formas por baixo ⇒ nada (o Node
/// desselecciona), e fora de um modo do tipo ⇒ nada.
#[test]
fn a_node_miss_picks_the_object_of_another_kind_underneath() {
    let mut c = cena();
    let ia = linha(&mut c, [0.0, 0.0], [2.0, 2.0]);
    sincroniza(&mut c);
    let a = c.vec.entities[&ia];
    let sprite = c.sim.world_mut().spawn_empty().id().to_bits();
    let map = &c.vec.entities;
    assert_eq!(another_kind_under(true, &[a, sprite], map), Some(sprite));
    assert_eq!(
        another_kind_under(true, &[a], map),
        None,
        "controlo: só formas"
    );
    assert_eq!(
        another_kind_under(false, &[sprite], map),
        None,
        "controlo: fora de um modo do tipo"
    );
}
