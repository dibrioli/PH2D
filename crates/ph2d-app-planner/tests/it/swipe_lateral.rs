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
