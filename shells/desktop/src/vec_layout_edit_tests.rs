//! Os gates da PORTA de autoria do auto layout.
//!
//! O que só se pode afirmar aqui: que a tabela `enum ↔ chip` responde igual nos dois sentidos,
//! que **Off remove** (e não neutraliza), que um campo escreve o slot que o seu id nomeia — e que
//! o SUJEITO da seção é resolvido pela mesma porta do W0.

use super::*;
use ph2d_ecs::VecFrame;
use ph2d_vec_scene::{VecScene, rectangle};

/// Uma moldura com `n` filhos, já sincronizada. Devolve `(sim, scene, map, ids)` onde o
/// **primeiro** id é a moldura.
fn frame_with(n: usize) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame_id = scene.push_path(rectangle([0.0, 0.0], [100.0, 40.0]));
    let kids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let frame = Entity::from_bits(map[&frame_id]);
    sim.world_mut()
        .entity_mut(frame)
        .insert(VecFrame { clip: false });
    for k in &kids {
        let kid = Entity::from_bits(map[k]);
        sim.world_mut()
            .entity_mut(kid)
            .insert(ph2d_ecs::ChildOf(frame));
    }
    let mut ids = vec![frame_id];
    ids.extend(kids);
    (sim, scene, map, ids)
}

fn ent(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Entity {
    let _ = sim;
    Entity::from_bits(map[&id])
}

/// **Toda variante dos três enums tem um chip, e ele volta a ser a variante.**
///
/// ⚠️ Os `match` exaustivos são o ponto do gate: uma variante nova **não compila** este teste até
/// entrar na tabela. Sem eles seria uma lista escrita à mão, que é exactamente a segunda cópia que
/// a tabela existe para não ter — e o sintoma seria um chip aceso que o clique não resolve.
#[test]
fn every_layout_variant_has_a_chip_and_the_chip_names_it_back() {
    for d in [LayoutDir::Row, LayoutDir::Column, LayoutDir::RowWrap] {
        // Exaustividade: o compilador cobra a variante nova aqui.
        match d {
            LayoutDir::Row | LayoutDir::Column | LayoutDir::RowWrap => {}
        }
        let chip = chip_of(DIRS, d);
        assert_eq!(
            layout_edit_for_id(chip),
            Some(LayoutEdit::Dir(Some(d))),
            "{d:?} nao volta do proprio chip"
        );
    }
    for a in [
        LayoutAlign::Start,
        LayoutAlign::Center,
        LayoutAlign::End,
        LayoutAlign::Stretch,
    ] {
        match a {
            LayoutAlign::Start | LayoutAlign::Center | LayoutAlign::End | LayoutAlign::Stretch => {}
        }
        assert_eq!(
            layout_edit_for_id(chip_of(ALIGNS, a)),
            Some(LayoutEdit::Align(a)),
            "{a:?}"
        );
    }
    for j in [
        LayoutJustify::Start,
        LayoutJustify::Center,
        LayoutJustify::End,
        LayoutJustify::SpaceBetween,
        LayoutJustify::SpaceAround,
    ] {
        match j {
            LayoutJustify::Start
            | LayoutJustify::Center
            | LayoutJustify::End
            | LayoutJustify::SpaceBetween
            | LayoutJustify::SpaceAround => {}
        }
        assert_eq!(
            layout_edit_for_id(chip_of(JUSTIFIES, j)),
            Some(LayoutEdit::Justify(j)),
            "{j:?}"
        );
    }
}

/// **O chip de direção ARMA o fluxo, e o Off o REMOVE.**
///
/// ⚠️ A 2ª metade é a que se pode "consertar" por engano para um no-op silencioso: se o Off
/// apenas zerasse os números, a moldura continuaria empilhando com vão zero e o artista veria os
/// filhos AMONTOADOS na origem em vez de voltarem para onde ele os pôs.
#[test]
fn a_direction_arms_the_flow_and_off_removes_it() {
    let (mut sim, _scene, map, ids) = frame_with(2);
    let sel = vec![ids[0]];
    let frame = ent(&sim, &map, ids[0]);

    assert!(sim.world().get::<VecLayout>(frame).is_none());
    apply_layout_edit(
        &mut sim,
        &map,
        &sel,
        LayoutEdit::Dir(Some(LayoutDir::Column)),
    );
    assert_eq!(
        sim.world().get::<VecLayout>(frame).map(|l| l.dir),
        Some(LayoutDir::Column)
    );

    apply_layout_edit(&mut sim, &map, &sel, LayoutEdit::Dir(None));
    assert!(
        sim.world().get::<VecLayout>(frame).is_none(),
        "Off tem de REMOVER — uma moldura que ainda flui com tudo em zero amontoa os filhos"
    );
}

/// **Um campo escreve o slot que o seu id nomeia — e o *All* escreve os quatro.**
#[test]
fn each_field_writes_the_slot_its_id_names() {
    let (mut sim, _scene, map, ids) = frame_with(1);
    let sel = vec![ids[0]];
    let frame = ent(&sim, &map, ids[0]);
    apply_layout_edit(&mut sim, &map, &sel, LayoutEdit::Dir(Some(LayoutDir::Row)));

    apply_layout_field(&mut sim, &map, &sel, LayoutField::Gap(0), 7.0);
    apply_layout_field(&mut sim, &map, &sel, LayoutField::Gap(1), 3.0);
    apply_layout_field(&mut sim, &map, &sel, LayoutField::Pad(2), 5.0);
    let l = *sim.world().get::<VecLayout>(frame).expect("flui");
    assert_eq!(l.gap, [7.0, 3.0]);
    assert_eq!(l.pad, [0.0, 0.0, 5.0, 0.0], "so' a BASE foi escrita");

    apply_layout_field(&mut sim, &map, &sel, LayoutField::PadAll, 2.0);
    let l = *sim.world().get::<VecLayout>(frame).expect("flui");
    assert_eq!(l.pad, [2.0; 4], "o campo *All* escreve os quatro lados");
}

/// **Vão e recuo NEGATIVOS são recusados na porta** — o flexbox não os exprime, e é o DOMÍNIO do
/// modelo, não um teto de gosto (por isso o widget não tem faixa: ver `populate_layout`).
#[test]
fn the_model_refuses_a_negative_gap_at_the_door() {
    let (mut sim, _scene, map, ids) = frame_with(1);
    let sel = vec![ids[0]];
    let frame = ent(&sim, &map, ids[0]);
    apply_layout_edit(&mut sim, &map, &sel, LayoutEdit::Dir(Some(LayoutDir::Row)));
    apply_layout_field(&mut sim, &map, &sel, LayoutField::Gap(0), -9.0);
    assert_eq!(
        sim.world().get::<VecLayout>(frame).expect("flui").gap[0],
        0.0
    );
}

/// **Um campo de FLUXO não liga o fluxo** — ele só é pintado sobre uma moldura que já empilha.
#[test]
fn a_flow_field_never_arms_the_flow_by_itself() {
    let (mut sim, _scene, map, ids) = frame_with(1);
    let sel = vec![ids[0]];
    let frame = ent(&sim, &map, ids[0]);
    let changed = apply_layout_field(&mut sim, &map, &sel, LayoutField::Gap(0), 4.0);
    assert!(!changed);
    assert!(sim.world().get::<VecLayout>(frame).is_none());
}

/// **O SUJEITO do bloco de item: um filho de fluxo, e a moldura ANINHADA.**
///
/// ⚠️ A 3ª asserção é a que impede o bloco de aparecer numa forma solta: sem fluxo no pai,
/// Grow/Shrink seriam dois campos que ninguém honra.
#[test]
fn the_item_block_is_offered_to_a_child_of_a_flowing_frame() {
    let (mut sim, _scene, map, ids) = frame_with(2);
    let frame_sel = vec![ids[0]];
    let kid_sel = vec![ids[1]];

    assert!(
        selected_item(&sim, &map, &kid_sel).is_none(),
        "sem fluxo no pai o bloco nao existe"
    );
    apply_layout_edit(
        &mut sim,
        &map,
        &frame_sel,
        LayoutEdit::Dir(Some(LayoutDir::Row)),
    );
    assert!(
        selected_item(&sim, &map, &kid_sel).is_some(),
        "o filho de uma moldura que flui TEM os dois campos"
    );
    assert!(
        selected_item(&sim, &map, &frame_sel).is_none(),
        "a moldura RAIZ nao esta' dentro de fluxo nenhum"
    );
}

/// **Grow/Shrink chegam ao componente, e o neutro DESTACA** (um componente que não faz nada não
/// viaja no arquivo).
#[test]
fn grow_reaches_the_component_and_the_neutral_detaches() {
    let (mut sim, _scene, map, ids) = frame_with(2);
    let frame_sel = vec![ids[0]];
    let kid_sel = vec![ids[1]];
    let kid = ent(&sim, &map, ids[1]);
    apply_layout_edit(
        &mut sim,
        &map,
        &frame_sel,
        LayoutEdit::Dir(Some(LayoutDir::Row)),
    );

    apply_layout_field(&mut sim, &map, &kid_sel, LayoutField::Grow, 2.0);
    assert_eq!(
        sim.world().get::<VecLayoutItem>(kid).map(|i| i.grow),
        Some(2.0)
    );
    assert_eq!(
        selected_item(&sim, &map, &kid_sel).map(|i| i.grow),
        Some(2.0)
    );

    apply_layout_field(&mut sim, &map, &kid_sel, LayoutField::Grow, 0.0);
    assert!(
        sim.world().get::<VecLayoutItem>(kid).is_none(),
        "o neutro tem de destacar"
    );
}

/// **O que a shell PUBLICA é o que a moldura guarda** — e `None` quando ela não flui.
#[test]
fn the_published_flow_mirrors_the_component() {
    let (mut sim, _scene, map, ids) = frame_with(1);
    let sel = vec![ids[0]];
    assert!(selected_flow(&sim, &map, &sel).is_none());

    apply_layout_edit(
        &mut sim,
        &map,
        &sel,
        LayoutEdit::Dir(Some(LayoutDir::RowWrap)),
    );
    apply_layout_edit(
        &mut sim,
        &map,
        &sel,
        LayoutEdit::Justify(LayoutJustify::SpaceBetween),
    );
    apply_layout_field(&mut sim, &map, &sel, LayoutField::Gap(1), 6.0);
    let f = selected_flow(&sim, &map, &sel).expect("flui");
    assert_eq!(f.dir, ids::VECTOR_LAYOUT_DIR_WRAP);
    assert_eq!(f.justify, ids::VECTOR_LAYOUT_JUSTIFY_BETWEEN);
    assert_eq!(f.gap[1], 6.0);
}

/// **Um id estrangeiro não é nem chip nem campo** — o roteador da shell testa os dois em cadeia,
/// e uma porta larga demais roubaria o clique de outra seção.
#[test]
fn a_foreign_id_belongs_to_neither_door() {
    let foreign = ids::VECTOR_FRAME_CLIP_ON;
    assert!(layout_edit_for_id(foreign).is_none());
    assert!(layout_field_for_id(foreign).is_none());
}

/// **Digitar um vão SOLTA o token dele** — o *detach* do Figma, no eixo autorado e só nele (W4c.4).
///
/// ⚠️ Sem isto o artista escreveria um número, o token continuaria a espaçar, e o campo mostraria
/// um valor que a moldura não usa: exactamente o estado que a rachura existe para denunciar, e que
/// nenhuma marca conserta.
#[test]
fn typing_a_gap_detaches_that_axis_token_and_only_that_one() {
    let (mut sim, _scene, map, ids) = frame_with(1);
    let sel = vec![ids[0]];
    let frame = ent(&sim, &map, ids[0]);
    apply_layout_edit(
        &mut sim,
        &map,
        &sel,
        LayoutEdit::Dir(Some(LayoutDir::RowWrap)),
    );

    let mut b = ph2d_ecs::VecBindings::default();
    b.set(ph2d_ecs::BoundProp::LayoutGapMain, "spacing.md");
    b.set(ph2d_ecs::BoundProp::LayoutGapCross, "spacing.lg");
    sim.world_mut().entity_mut(frame).insert(b);

    apply_layout_field(&mut sim, &map, &sel, LayoutField::Gap(0), 4.0);
    let b = sim
        .world()
        .get::<ph2d_ecs::VecBindings>(frame)
        .expect("o eixo transversal sobrevive, entao o componente fica");
    assert_eq!(
        b.get(ph2d_ecs::BoundProp::LayoutGapMain),
        None,
        "o eixo autorado soltou o token"
    );
    assert_eq!(
        b.get(ph2d_ecs::BoundProp::LayoutGapCross),
        Some("spacing.lg"),
        "e NAO soltou o outro eixo"
    );

    // ⚠️ CONTROLE: um campo que NÃO é vão não solta nada — o recuo não tem token nesta wave, e
    // soltar por ele seria a lei a alcançar mais do que ela diz.
    apply_layout_field(&mut sim, &map, &sel, LayoutField::PadAll, 2.0);
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::VecBindings>(frame)
            .and_then(|b| b.get(ph2d_ecs::BoundProp::LayoutGapCross)),
        Some("spacing.lg"),
        "editar o RECUO nao pode soltar o token de um VAO"
    );
}
