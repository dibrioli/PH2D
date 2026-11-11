//! InputMap ondas 3–4: o deslize de lado não muda o calendário; a roda anda o mês como no Kanban (`dy` < 0 = à frente).

#[test]
fn o_deslizar_de_lado_nao_muda_o_calendario_e_a_roda_anda_como_no_kanban() {
    use ph2d_editor_core::screens::hero::board_view::{BoardScroll::Pan, scroll};
    let (mut m, _) = crate::Mesa::cena();
    let antes = format!("{:?}", m.docs().probe_calendar());
    for i in 0..64u8 {
        let x = 100.0 + 200.0 * f32::from(i % 8);
        let y = 60.0 + 120.0 * f32::from(i / 8);
        let _ = scroll(&mut m.hero, x, y, Pan([-480.0, 0.0]));
        m.quadro();
    }
    assert_eq!(format!("{:?}", m.docs().probe_calendar()), antes);
    let (p, mut a0) = (crate::centro(m.docs().probe_mode().1), None);
    for (dy, e) in [(-480.0, 1), (-48.0, 1), (48.0, -1)] {
        let _ = scroll(&mut m.hero, p.0, p.1, Pan([0.0, dy]));
        m.quadro();
        let a = m.docs().probe_calendar().anchor;
        assert_eq!(a.cmp(&a0) as i8, e, "{dy}: {a0:?} → {a:?}");
        a0 = a;
    }
}
