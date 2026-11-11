//! InputMap onda 3 item 7: o deslizar de lado do trackpad não muda o calendário do Planner.

#[test]
fn o_deslizar_de_lado_nao_muda_o_calendario() {
    use ph2d_editor_core::screens::hero::board_view::{BoardScroll::Pan, scroll};
    let (mut m, _) = crate::Mesa::cena();
    let antes = format!("{:?}", m.docs().probe_calendar());
    for i in 0..64u8 {
        let (x, y) = (
            100.0 + 200.0 * f32::from(i % 8),
            60.0 + 120.0 * f32::from(i / 8),
        );
        let _ = scroll(&mut m.hero, x, y, Pan([-480.0, 0.0]));
        m.quadro();
    }
    assert_eq!(format!("{:?}", m.docs().probe_calendar()), antes);
}
/// Onda 4 item 5: a roda anda o mês para o lado do Kanban (`a_roda_do_calendario_e_o_deslize_*`).
#[test]
fn a_roda_do_calendario_anda_para_o_mesmo_lado_que_no_kanban() {
    let ((mut m, _), mut antes) = (crate::Mesa::cena(), None);
    let p = crate::centro(m.docs().probe_mode().1);
    for (i, dy) in [-480.0, -48.0, -48.0, 48.0].into_iter().enumerate() {
        let _ = ph2d_editor_core::screens::hero::board_view::wheel(&mut m.hero, p.0, p.1, dy);
        m.quadro();
        let a = m.docs().probe_calendar().anchor;
        assert_eq!(
            (i, a.cmp(&antes) as i8),
            (i, [1, 1, 1, -1][i]),
            "{antes:?} → {a:?}"
        );
        antes = a;
    }
}
