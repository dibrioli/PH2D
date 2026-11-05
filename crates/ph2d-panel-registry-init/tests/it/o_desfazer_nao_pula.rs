//! ⭐ **Ctrl+Z e Ctrl+Y não pulam os cartões** (`docs/CartoesUI/10`, smoke 3, defeito B), por gesto real
//! sobre a `Mesa` do C2. O restauro respawna o mundo: os bits mudam e nada mais. Medido na app, antes da
//! cura: um quadro sem seleção punha os 7 cartões a sair, e a lista ia de 681 a 450 px e voltava em 27
//! quadros. A porta é `ph2d_host_presos::depois_do_restauro`.

use super::*;
use ph2d_editor_core::panel::cards::entra_sai;

/// O que a mão vê da lista, a cada quadro: a altura dela, a altura e o `y` de cada cartão, a rolagem,
/// e a altura de cada solto.
#[derive(Debug, PartialEq)]
struct Lista {
    h: Option<f32>,
    cartoes: Vec<(NodeId, f32)>,
    y: Vec<Option<f32>>,
    rolagem: f32,
    soltos: Vec<Option<f32>>,
}

fn lista(m: &mut Mesa, soltos: &[NodeId]) -> Lista {
    let y = [
        v::VECTOR_WIDTH_NUM,
        v::VECTOR_FILL_OPACITY_NUM,
        insp::ids::INSP_TRANSFORM_POS_X,
    ]
    .map(|id| m.h.hit_index.rect_for(id).map(|r| r.y))
    .to_vec();
    let h = m.h.store.panel_rect(INSP_PANEL).map(|r| r.h);
    let rolagem = m.h.store.panel_scroll(INSP_PANEL);
    let cartoes =
        m.h.store
            .card_list_mut(INSP_PANEL)
            .items
            .iter()
            .map(|i| (i.key, i.h))
            .collect();
    let soltos = soltos
        .iter()
        .map(|k| m.h.store.panel_rect(loose_rect_of(*k)).map(|r| r.h))
        .collect();
    Lista {
        h,
        cartoes,
        y,
        rolagem,
        soltos,
    }
}

fn id_estavel(i: usize) -> ph2d_ecs::StableId {
    ph2d_ecs::StableId(900 + i as u64)
}

fn depois_do_restauro(m: &mut Mesa) {
    let mut sel = ph2d_app_vec::selection_sync::VecSelSync::default();
    ph2d_host_presos::depois_do_restauro(
        &mut m.h, &m.sim, &m.scene, &m.map, &mut m.pen, &mut sel, &m.tools,
    );
}

/// O respawn do restauro: as três formas com bits novos, e o resto igual.
fn respawn(m: &mut Mesa) {
    for i in 0..3 {
        let velho = ph2d_ecs::Entity::from_bits(m.bits[i]);
        m.sim.world_mut().despawn(velho);
        let e = m
            .sim
            .world_mut()
            .spawn((
                Transform::default(),
                Name::new(nome(i)),
                VecPathRef(m.ids[i]),
                id_estavel(i),
            ))
            .id();
        m.bits[i] = e.to_bits();
        m.map.insert(m.ids[i], e.to_bits());
    }
}

/// A lista assentou: as molas pararam e dois quadros seguidos são iguais.
fn assenta(m: &mut Mesa) {
    for _ in 0..400 {
        m.quadro();
        if !m.h.store.card_list_mut(INSP_PANEL).stack.moving() {
            return;
        }
    }
    panic!("a lista não assentou");
}

fn mesa() -> Mesa {
    let mut m = Mesa::nova();
    for sec in [insp::CARD_TRANSFORM, insp::CARD_APPEARANCE] {
        m.h.store.set_collapsed(sec, false);
    }
    assenta(&mut m);
    m
}

/// ⭐ O desfazer que não muda a estrutura: nenhum quadro difere, em nenhum cartão. *Mutação: a régua é
/// viva: o buraco de um quadro, abaixo, é vermelho.*
#[test]
fn an_undo_that_keeps_the_cards_moves_nothing() {
    let mut m = mesa();
    let foto = lista(&mut m, &[]);
    assert!(foto.cartoes.len() >= 3, "{foto:?}");
    // Ctrl+Z e Ctrl+Y são o mesmo gesto: dois respawns.
    for _ in 0..2 {
        respawn(&mut m);
        m.escolhe(&[0]);
        for q in 0..40 {
            m.quadro();
            assert_eq!(lista(&mut m, &[]), foto, "quadro {q}");
            assert!(
                entra_sai::saindo(INSP_PANEL).is_empty(),
                "quadro {q}: há cartões a sair"
            );
        }
    }
}

/// A régua: o buraco de um quadro sem seleção (a app antes da cura) pula os cartões.
#[test]
fn one_frame_without_a_selection_is_what_made_the_cards_jump() {
    let mut m = mesa();
    let foto = lista(&mut m, &[]);
    insp::set_current_inspector_transform(None);
    insp::set_current_inspector_name(None);
    appearance::set_current_appearance(None);
    appearance::set_current_vector_style(None);
    m.h.store.set_card_subject(INSP_PANEL, None);
    m.h.gizmo.clear_all_selection();
    pinta(&mut m.h, 1);
    assert!(
        !entra_sai::saindo(INSP_PANEL).is_empty(),
        "a régua não vê o buraco"
    );
    m.escolhe(&[0]);
    m.quadro();
    let h = lista(&mut m, &[]).h.expect("a lista");
    assert!(
        h < foto.h.expect("a lista") - 20.0,
        "o buraco não mexeu: {h} × {foto:?}"
    );
}

/// O cartão preso, com os bits novos ligados pela chave: o quadro seguinte mostra o mesmo.
/// *Mutação: tirar `chave::resolve` de `depois_do_restauro` ⇒ o Fill preso de A perde a camada por um
/// quadro (na app: 145 → 99 → 145 px).*
#[test]
fn an_undo_does_not_flicker_a_pinned_card() {
    let mut m = mesa();
    for i in 0..3 {
        let e = ph2d_ecs::Entity::from_bits(m.bits[i]);
        m.sim.world_mut().entity_mut(e).insert(id_estavel(i));
    }
    let k = m.prende(CARD_FILL, 0, &[1]);
    depois_do_restauro(&mut m); // o quadro da app liga os bits à chave durável
    assenta(&mut m);
    let foto = lista(&mut m, &[k]);
    for _ in 0..2 {
        respawn(&mut m);
        depois_do_restauro(&mut m);
        assert_eq!(
            m.h.gizmo.iter_selected().collect::<Vec<_>>(),
            [m.bits[1]],
            "o gizmo não voltou"
        );
        assert_eq!(
            m.h.store.loose_card(k).and_then(|c| c.pin),
            Some(m.bits[0]),
            "o alfinete não voltou"
        );
        for q in 0..20 {
            m.quadro();
            assert_eq!(lista(&mut m, &[k]), foto, "quadro {q}");
        }
    }
}
