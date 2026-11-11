//! O deslizar de lado (onda 3 item 7) = o Shift+roda; o quadro e a Tabela andam para o MESMO lado.
use crate::{AREA, Mesa, centro};
use ph2d_editor_core::screens::hero::board_view::{BoardScroll, scroll};
use ph2d_kanban_model::{FieldDef, FieldId, FieldKind, FracKey, KanbanOp, Meta, Record};
use ph2d_kanban_views::table::Column;
fn gemeos(mut g: Mesa, mut r: Mesa, p: (f32, f32), x: impl Fn(&mut Mesa) -> f64) {
    let passos = [
        ([-48.0_f32, 0.0], &[(-48.0_f32, true)][..]),
        ([0.0, -48.0], &[(-48.0, false)][..]),
        ([-32.0, -16.0], &[(-32.0, true), (-16.0, false)][..]),
    ];
    for (i, ([dx, dy], rodas)) in passos.into_iter().enumerate() {
        let antes = x(&mut g);
        let _ = scroll(&mut g.hero, p.0, p.1, BoardScroll::Pan([dx, dy]));
        g.quadro();
        for &(dy, shift) in rodas {
            r.hero.store.set_shift_held(shift);
            let _ = r.roda(p.0, p.1, dy);
        }
        assert_eq!(x(&mut g), x(&mut r), "passo {i}: = o Shift+roda");
        let anda = x(&mut g).total_cmp(&antes) as i8; // `x` = quanto a vista foi para a direita
        assert_eq!(anda, [1, 0, 1][i], "passo {i} de {antes} (1 = controlo)");
    }
}
#[test]
fn o_deslizar_de_lado_e_o_shift_roda_no_quadro_e_na_tabela() {
    let ((g, id), (r, _)) = (Mesa::cena1(), Mesa::cena1());
    let v = |m: &mut Mesa| m.docs().probe_view(id.0).unwrap().scroll_x;
    gemeos(g, r, centro(AREA), v);
    let mut def = FieldDef {
        id: FieldId(9200),
        name: "Field".into(),
        kind: FieldKind::Checkbox,
        on_front: false,
        pos: FracKey::between(None, None),
        meta: Meta::default(),
    }; // + vinte campos: a Tabela transborda na horizontal
    let [mut g, r] = [0; 2].map(|_| {
        let (mut m, _) = Mesa::cena1();
        let set = m.hero.documents.kanbans_shared();
        for i in 0..20 {
            def.id = FieldId(9200 + i);
            set.borrow_mut()
                .apply(id, vec![KanbanOp::put(Record::Field(def.clone()))]);
        }
        crate::j1_vistas::vista(&mut m, id, 1);
        m
    });
    let cel = move |m: &mut Mesa| m.docs().probe_table_cell(id.0, 0, Column::Due).unwrap().1;
    let p = centro(cel(&mut g));
    gemeos(g, r, p, move |m| -f64::from(cel(m).x));
}
fn vs(m: &mut Mesa, id: u64) -> &ph2d_kanban_views::Views {
    m.docs().probe_views(id).unwrap()
}
/// Onda 4: a roda anda o mês como no Planner (o mesmo `-480, -48, -48, 48`); o deslize só rola a grelha da Linha do
/// tempo (controlo) e não mexe no Calendário (nem no mês, o vertical), no Dashboard nem na gaveta.
#[test]
fn a_roda_do_calendario_e_o_deslize_de_lado_nas_outras_vistas() {
    let ((mut m, id), c) = (Mesa::cena1(), centro(AREA));
    crate::j1_vistas::vista(&mut m, id, 2);
    let mut antes = None;
    for (i, dy) in [-480.0, -48.0, -48.0, 48.0].into_iter().enumerate() {
        m.roda(c.0, c.1 + AREA.h / 4.0, dy);
        let agora = vs(&mut m, id.0).calendar.anchor;
        assert_eq!(
            agora.cmp(&antes) as i8,
            [1, 1, 1, -1][i],
            "{dy}: {antes:?} → {agora:?}"
        );
        antes = agora;
    }
    for (i, k) in [(2, 0), (4, 0), (3, 1), (3, 2)] {
        crate::j1_vistas::vista(&mut m, id, i);
        let t = vs(&mut m, id.0).timeline.painted().clone();
        let p = match (k, t.drawer, t.ruler) {
            (1, Some(r), _) => centro(r),
            (2, _, Some(g)) => (g.grid.x + g.grid.w / 3.0, g.grid.y + g.grid.h / 2.0),
            _ => (c.0, c.1 + AREA.h / 4.0),
        };
        let antes = format!("{:?}", vs(&mut m, id.0));
        for dx in [-48.0, 48.0, -480.0] {
            let _ = scroll(&mut m.hero, p.0, p.1, BoardScroll::Pan([dx, 0.0]));
            m.quadro();
        }
        assert_eq!(
            format!("{:?}", vs(&mut m, id.0)) != antes,
            k == 2,
            "vista {i}, {k}"
        );
    }
}
