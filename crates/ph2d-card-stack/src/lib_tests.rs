use super::*;

const GAP: f32 = 8.0;

fn items() -> Vec<Item<u32>> {
    vec![
        Item { key: 1, h: 40.0 },
        Item { key: 2, h: 60.0 },
        Item { key: 3, h: 20.0 },
    ]
}

#[test]
fn the_slot_index_counts_the_neighbours_whose_middle_the_centre_passed() {
    let e = [40.0, 60.0, 20.0];
    assert_eq!(slot_index(e, GAP, 0.0), 0);
    assert_eq!(slot_index(e, GAP, 21.0), 1);
    assert_eq!(slot_index(e, GAP, 77.0), 1);
    assert_eq!(slot_index(e, GAP, 79.0), 2);
    assert_eq!(slot_index(e, GAP, 1000.0), 3);
}

#[test]
fn the_edge_depth_is_signed_and_capped() {
    assert!((edge_depth(0.0, 0.0, 600.0) + 1.0).abs() < f32::EPSILON);
    assert!((edge_depth(300.0, 0.0, 600.0)).abs() < f32::EPSILON);
    assert!((edge_depth(600.0, 0.0, 600.0) - 1.0).abs() < f32::EPSILON);
    assert!((edge_depth(568.0, 0.0, 600.0) - 0.5).abs() < f32::EPSILON);
}

#[test]
fn the_target_skips_the_lifted_card_and_opens_its_gap() {
    let f = Flight {
        key: 2,
        at: (0.0, 0.0),
        grab: (0.0, 0.0),
        size: (100.0, 60.0),
        to: Some(0),
    };
    let p = targets(&items(), GAP, Some(&f));
    assert_eq!(p.slot, Some((0.0, 60.0)));
    assert_eq!(p.ys, vec![(1, 68.0), (3, 116.0)]);
    assert!((p.content_h - 136.0).abs() < f32::EPSILON);
}

#[test]
fn a_flight_outside_the_list_closes_the_gap() {
    let f = Flight {
        key: 2,
        at: (0.0, 0.0),
        grab: (0.0, 0.0),
        size: (100.0, 60.0),
        to: None,
    };
    let p = targets(&items(), GAP, Some(&f));
    assert_eq!(p.slot, None);
    assert_eq!(p.ys, vec![(1, 0.0), (3, 48.0)]);
}

#[test]
fn a_landed_card_settles_from_the_hand_with_a_spring() {
    let mut s = Stack::default();
    let law = Some(Spring::default());
    s.place(&items(), GAP, 1.0 / 60.0, law);
    s.lift(1, (10.0, 10.0), (10.0, 10.0), (100.0, 40.0), 0);
    // A mão leva o cartão 1 até ao fundo da lista (topo em 0).
    s.drag(&items(), GAP, (10.0, 200.0), Some(0.0));
    assert_eq!(s.flight().and_then(|f| f.to), Some(2));
    assert!(s.moving());
    let (k, to) = s.land(0.0).expect("caiu na lista");
    assert_eq!((k, to), (1, 2));
    let mut order = vec![1, 2, 3];
    reorder(&mut order, &1, to);
    assert_eq!(order, vec![2, 3, 1]);
    let landed: Vec<_> = order
        .iter()
        .map(|k| items().into_iter().find(|i| i.key == *k).expect("item"))
        .collect();
    let p = s.place(&landed, GAP, 1.0 / 60.0, law);
    let y1 = p.ys.iter().find(|(k, _)| *k == 1).map(|(_, y)| *y);
    // Partiu da mão (190) e ainda não chegou ao alvo (88).
    assert!(y1.is_some_and(|y| y > 88.0 && y < 190.0), "{y1:?}");
    for _ in 0..240 {
        s.place(&landed, GAP, 1.0 / 60.0, law);
    }
    let p = s.place(&landed, GAP, 1.0 / 60.0, law);
    assert_eq!(p.ys, vec![(2, 0.0), (3, 68.0), (1, 96.0)]);
    assert!(!s.moving());
    assert!(s.tilt_deg().abs() < f64::EPSILON);
}

#[test]
fn the_tilt_rises_while_flying_and_reduced_motion_jumps() {
    let mut s = Stack::default();
    s.place(&items(), GAP, 1.0 / 60.0, None);
    s.lift(3, (0.0, 0.0), (0.0, 0.0), (100.0, 20.0), 2);
    s.place(&items(), GAP, 1.0 / 60.0, None);
    assert!((s.tilt_deg() - FLIGHT_TILT_DEG).abs() < f64::EPSILON);
}

/// C5: largar FORA da lista devolve o voo sem assentar nada (o cartão passa a solto).
#[test]
fn detaching_hands_back_the_flight_and_lands_nothing() {
    let items = [Item { key: 1u8, h: 40.0 }, Item { key: 2u8, h: 40.0 }];
    let mut s: Stack<u8> = Stack::default();
    s.lift(1, (10.0, 10.0), (5.0, 5.0), (100.0, 40.0), 0);
    s.drag(&items, 8.0, (500.0, 300.0), None);
    let f = s.detach().expect("o voo");
    assert_eq!((f.key, f.to, f.ghost()), (1, None, (495.0, 295.0)));
    assert!(s.flight().is_none() && s.land(0.0).is_none());
}

/// D32: o texto ganha a largura que precisa; os segmentos só de ícone ficam estreitos.
/// *Mutação: voltar a `avail / n` ⇒ o «Auto (CCW)» recebe 52 e corta.*
#[test]
fn a_text_segment_gets_its_width_and_icon_segments_stay_narrow() {
    use crate::metrics::seg_widths;
    let bend = [(80.0, false), (30.0, true), (30.0, true), (50.0, false)];
    let w = seg_widths(&bend, 210.0);
    assert!(w[0] >= 80.0, "o texto cortou: {w:?}");
    assert!((w[1] - w[2]).abs() < 1e-4 && w[1] < w[3], "{w:?}");
    assert!((w.iter().sum::<f32>() - 210.0).abs() < 1e-3, "{w:?}");
    // Sem lugar: os ícones guardam a deles e só o texto encolhe.
    let w = seg_widths(&bend, 150.0);
    assert!((w[1] - 30.0).abs() < 1e-4, "{w:?}");
    assert!((w.iter().sum::<f32>() - 150.0).abs() < 1e-3, "{w:?}");
    // Iguais continuam iguais (a fila só de ícones não muda).
    let w = seg_widths(&[(30.0, true); 3], 120.0);
    assert!(w.iter().all(|x| (x - 40.0).abs() < 1e-4), "{w:?}");
}

/// T41: o cartão que regressa do canvas parte de onde estava (`x` e `y`) e assenta com mola.
/// *Mutação: `arrive` sem o `landed` ⇒ o `y` nasce no alvo (salta).*
#[test]
fn an_arriving_card_leaves_from_the_canvas_and_settles_in_its_place() {
    let mut s = Stack::default();
    let law = Some(Spring::default());
    s.place(&items(), GAP, 1.0 / 60.0, law);
    s.arrive(3, (600.0, 400.0));
    let p = s.place(&items(), GAP, 1.0 / 60.0, law);
    let y3 = p.ys.iter().find(|(k, _)| *k == 3).map(|(_, y)| *y);
    let dx = s.arriving_dx(3);
    assert!(y3.is_some_and(|y| y > 116.0 && y < 400.0), "{y3:?}");
    assert!(dx.is_some_and(|x| x > 0.0 && x < 600.0), "{dx:?}");
    assert!(s.moving());
    for _ in 0..240 {
        s.place(&items(), GAP, 1.0 / 60.0, law);
    }
    assert_eq!(s.arriving_dx(3), None, "não assentou na coluna");
    assert!(!s.moving());
}

/// T41: dobrar um cartão encolhe a altura VISTA com mola (não salta), e a 1.ª medida chega já.
/// *Mutação: `smooth_heights` sem a mola (`it.h` intacto) ⇒ a altura cai no mesmo quadro.*
#[test]
fn a_folding_card_shrinks_with_a_spring() {
    let mut s = Stack::default();
    let law = Some(Spring::default());
    let mut a = items();
    assert!(s.smooth_heights(&mut a, 1.0 / 60.0, law).is_empty());
    assert_eq!(a[1].h, 60.0);
    let mut b = items();
    b[1].h = 20.0;
    let vivas = s.smooth_heights(&mut b, 1.0 / 60.0, law);
    assert_eq!(vivas, vec![2]);
    assert!(b[1].h > 20.0 && b[1].h < 60.0, "{}", b[1].h);
    for _ in 0..240 {
        b[1].h = 20.0;
        s.smooth_heights(&mut b, 1.0 / 60.0, law);
    }
    assert!((b[1].h - 20.0).abs() < 0.01);
}
