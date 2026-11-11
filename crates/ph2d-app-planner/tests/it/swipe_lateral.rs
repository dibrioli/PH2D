//! InputMap ondas 3–4: o deslize de lado não muda o calendário; cada roda anda como no Kanban: `dy` < 0 (para
//! baixo) = o mês à frente e as horas (Week) e a lista do «Move to» para baixo (a Agenda: `deslize.rs`).
use crate::{Mesa, centro};
use ph2d_editor_core::screens::hero::board_view::{BoardScroll::Pan, scroll};
use ph2d_host::PointerKind;
use ph2d_kanban_views::calendar::Mode;

/// Para baixo, para baixo, para cima em `at`: `f` (o que anda) sobe, sobe e desce.
fn roda<T: PartialOrd + std::fmt::Debug>(m: &mut Mesa, at: (f32, f32), f: impl Fn(&mut Mesa) -> T) {
    for (dy, e) in [(-48.0, 1), (-48.0, 1), (48.0, -1)] {
        let a = f(m);
        let _ = scroll(&mut m.hero, at.0, at.1, Pan([0.0, dy]));
        m.quadro();
        let (b, e) = (f(m), Some(e));
        assert_eq!(b.partial_cmp(&a).map(|o| o as i8), e, "{dy}: {a:?} → {b:?}");
    }
}

#[test]
fn o_deslizar_de_lado_nao_muda_o_calendario_e_cada_roda_anda_como_no_kanban() {
    let (mut m, _) = Mesa::cena();
    let antes = format!("{:?}", m.docs().probe_calendar());
    for i in 0..64u8 {
        let x = 100.0 + 200.0 * f32::from(i % 8);
        let y = 60.0 + 120.0 * f32::from(i / 8);
        let _ = scroll(&mut m.hero, x, y, Pan([-480.0, 0.0]));
        m.quadro();
    }
    assert_eq!(format!("{:?}", m.docs().probe_calendar()), antes);
    let c = centro(m.docs().probe_mode().1);
    roda(&mut m, c, |m| m.docs().probe_calendar().anchor);
    let (x, y) = centro(m.docs().probe_calendar().mode_rect(Mode::Week).unwrap());
    for k in [PointerKind::Down, PointerKind::Up] {
        m.rato(k, x, y); // o modo troca ao carregar
    }
    m.quadro();
    roda(&mut m, c, |m| {
        -m.docs().probe_calendar().hour_y(8, 0).unwrap()
    });
    let (set, agora) = (m.hero.documents.kanbans_shared(), m.agora()); // + Kanbans: a lista transborda
    for _ in 0..4 {
        let _ = ph2d_kanban_views::fixture::short_film(&mut set.borrow_mut(), agora);
    }
    m.clica_em(ph2d_app_planner::Hit::Mode(ph2d_app_planner::Mode::Planner));
    let item = *m.set().agenda().inbox.keys().next().unwrap();
    m.clica_em(ph2d_app_planner::Hit::InboxToCard(item));
    let r = m.docs().probe_dest().unwrap().rect;
    roda(&mut m, centro(r), |m| m.docs().probe_dest().unwrap().scroll);
}
