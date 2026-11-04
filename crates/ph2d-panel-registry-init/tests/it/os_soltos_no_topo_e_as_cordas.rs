//! ⭐ **Dois defeitos de 05 §9, no hero montado.** O solto colado ao topo do canvas tinha o cabeçalho
//! sob a barra de modos e o alfinete não se clicava; a corda só passava por cima dos soltos do painel
//! dela. Os dois pedem a mão e a ordem real da pintura, que o mock não tem.
#![cfg(all(feature = "panel-skeleton", feature = "panel-physics"))]

use super::o_cartao_solto_no_canvas::{
    bones, centro, clica, hero_com_o_bones, osso_em_foco, pinta,
};
use ph2d_editor_core::aquecimento::{PASSO_DE_QUADRO_S, com_passo_fixo};
use ph2d_editor_core::interaction::loose_pin_of;
use ph2d_editor_core::screens::hero::{HeroScreen, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_skeleton::cards::CARD_IK;
use ph2d_text::TextSystem;
use ph2d_vector::{DrawOp, Shape, VectorScene};

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

/// *Mutação: sem `abaixo_da_barra` o cabeçalho fica sob a barra: a mão acerta nela, não no alfinete.*
#[test]
fn a_loose_card_at_the_top_of_the_canvas_keeps_its_pin_within_reach() {
    let mut h = hero_com_o_bones();
    osso_em_foco(8, "Bone 8", 0.5);
    pinta(&mut h, 20);
    h.store.detach_card(CARD_IK, bones(), (600.0, 0.0), 300.0);
    pinta(&mut h, 10);
    let barra = h.last_layout.expect("houve um quadro").tool_bar;
    assert!(barra.h > 0.0, "esta cena tem a barra de modos");
    let pino = h
        .hit_index
        .rect_for(loose_pin_of(CARD_IK))
        .expect("o alfinete pinta-se");
    assert!(
        pino.y >= barra.y + barra.h,
        "o alfinete {pino:?} está sob a barra {barra:?}"
    );
    let (x, y) = centro(pino);
    assert_eq!(
        h.hit_index.hit_with_rect(x, y).map(|(id, _)| id),
        Some(loose_pin_of(CARD_IK)),
        "outro controlo fica com o clique no alfinete"
    );
    clica(&mut h, loose_pin_of(CARD_IK));
    assert_eq!(h.store.loose_card(CARD_IK).expect("solto").pin, Some(8));
}

/// O índice do último preenchimento cuja caixa é `r` (folga de 1 px).
fn ultimo_fill_de(ops: &[DrawOp], r: Rect) -> Option<usize> {
    ops.iter().rposition(|op| {
        let DrawOp::Fill {
            path, transform, ..
        } = op
        else {
            return false;
        };
        let b = (*transform * path.clone()).bounding_box();
        (b.x0 - f64::from(r.x)).abs() <= 1.0
            && (b.y0 - f64::from(r.y)).abs() <= 1.0
            && (b.x1 - f64::from(r.x + r.w)).abs() <= 1.0
            && (b.y1 - f64::from(r.y + r.h)).abs() <= 1.0
    })
}

fn um_quadro_gravado(h: &mut HeroScreen) -> Vec<DrawOp> {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    scene.record();
    com_passo_fixo(PASSO_DE_QUADRO_S, || {
        paint_hero_screen(h, JANELA, &mut scene, &mut text);
    });
    scene.take_record()
}

/// *Mutação: as cordas de volta ao passe do painel ⇒ o nó da corda do Bones pinta-se antes do painel
/// que vem depois dele, e os soltos desse painel cobrem a corda.*
#[test]
fn a_rope_paints_after_the_panels_that_come_after_its_own() {
    let mut h = hero_com_o_bones();
    h.panel_visibility.insert("physics", true);
    pinta(&mut h, 30);
    let fisica = ph2d_editor_core::ids::PHYSICS_PANEL;
    let lista = h.store.panel_rect(fisica).expect("o Physics pinta a lista");
    osso_em_foco(8, "Bone 8", 0.5);
    pinta(&mut h, 5);
    h.store.detach_card(CARD_IK, bones(), (400.0, 200.0), 300.0);
    pinta(&mut h, 5);
    clica(&mut h, loose_pin_of(CARD_IK));
    clica(&mut h, loose_pin_of(CARD_IK));
    assert!(
        h.store.loose_card(CARD_IK).expect("solto").rope,
        "sem corda"
    );
    h.store.bump_panel_z(bones());
    h.store.bump_panel_z(fisica);
    for _ in 0..30 {
        ph2d_card_rope::anda_com(&mut h.store, bones(), |_| Some((500.0, 500.0)), 1.0 / 60.0);
        pinta(&mut h, 1);
    }

    let ops = um_quadro_gravado(&mut h);
    let ponta = *h
        .store
        .loose_rope(CARD_IK)
        .last()
        .expect("a corda tem pontos");
    let r = ph2d_card_stack::metrics::LOOSE_ROPE_KNOT_R;
    let no = Rect::new(ponta[0] - r, ponta[1] - r, r * 2.0, r * 2.0);
    let do_no = ultimo_fill_de(&ops, no).expect("o nó da corda pinta-se");
    let da_lista = ultimo_fill_de(&ops, lista).expect("o fundo da lista do Physics pinta-se");
    assert!(
        do_no > da_lista,
        "o nó ({do_no}) pinta-se antes da lista do painel seguinte ({da_lista})"
    );
}
