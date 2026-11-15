//! O editor da curva Custom: a Bézier que se publica e o arrasto que vira intents.

use super::*;
use ph2d_timeline::{AnimTarget, Extrap, KeyId, PropKind};

fn chave(id: u64, t: f64, value: f32, interp: Interp, selected: bool) -> KeyView {
    KeyView {
        retime: None,
        id: KeyId::new(id),
        t_seconds: t,
        value,
        interp,
        selected,
        roving: false,
    }
}

fn faixa(target: u64, keys: Vec<KeyView>) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop: PropKind::TranslationX,
        entity: 1,
        missing: false,
        buffer_ghost: None,
        pre: Extrap::Hold,
        post: Extrap::Hold,
        expr: None,
        keys,
    }
}

fn snap(tracks: Vec<TrackView>) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        tracks,
        fps: 24.0,
        ..Default::default()
    }
}

/// A `Bezier` publica-se como a curva sem pontos; a `Spline` como está; a `BezierW` normaliza-se
/// pela variação até à seguinte (é a mesma curva); parada, ou sem seguinte, não tem forma.
#[test]
fn the_card_editor_shows_the_keys_own_normalized_curve() {
    let bez = Interp::bezier(0.25, 0.1, 0.75, 1.2);
    let t = faixa(
        9,
        vec![
            chave(1, 0.0, 0.0, bez, false),
            chave(2, 1.0, 5.0, bez, false),
        ],
    );
    let c = normalizada(&t, 0).unwrap();
    assert!(c.points().is_empty());
    assert_eq!(c.end_handles(), ([0.25, 0.1], [0.75, 1.2]));
    let mut com_ponto = c;
    com_ponto.insert(0.5);
    let sp = Interp::Spline(com_ponto);
    let t = faixa(
        9,
        vec![chave(1, 0.0, 0.0, sp, false), chave(2, 1.0, 5.0, sp, false)],
    );
    assert_eq!(normalizada(&t, 0), Some(com_ponto));
    let w = Interp::bezier_w(0.3, 2.0, 0.6, -5.0);
    let t = faixa(
        9,
        vec![chave(1, 0.0, 0.0, w, false), chave(2, 1.0, 10.0, w, false)],
    );
    let n = normalizada(&t, 0).unwrap();
    // O valor ao longo da normalizada é o mesmo que a BezierW dá (é a mesma curva).
    for u in [0.25, 0.5, 0.75] {
        let a = Interp::Spline(n).value(0.0, 10.0, u);
        assert!((a - w.value(0.0, 10.0, u)).abs() < 1e-4, "u={u}");
    }
    let parada = faixa(
        9,
        vec![chave(1, 0.0, 3.0, w, false), chave(2, 1.0, 3.0, w, false)],
    );
    assert_eq!(normalizada(&parada, 0), None);
    assert_eq!(normalizada(&parada, 1), None);
    // Sem pontos a chave volta a ser a Bézier de duas alças; com pontos é a Spline.
    assert!(matches!(interp_da_curva(c), Interp::Bezier { .. }));
    assert_eq!(interp_da_curva(com_ponto), Interp::Spline(com_ponto));
}

/// Um arrasto = um passo de undo; uma chave fora da seleção muda só ela, dentro dela a seleção
/// inteira; uma coluna passa a ser a seleção antes de mudar.
#[test]
fn a_handle_drag_is_one_undo_step_over_the_menus_scope() {
    let lin = Interp::Linear;
    let s = snap(vec![
        faixa(
            9,
            vec![
                chave(1, 0.0, 0.0, lin, false),
                chave(2, 1.0, 5.0, lin, true),
            ],
        ),
        faixa(8, vec![chave(3, 1.0, 0.0, lin, false)]),
    ]);
    let curva = EaseSpline::from_bezier(0.2, 0.0, 0.8, 1.0);
    let edit = |scope, begin, end| TimelineBezierEdit {
        scope,
        curve: curva,
        begin,
        end,
    };
    let interp = Interp::bezier(f64::from(0.2_f32), 0.0, f64::from(0.8_f32), 1.0);
    let so_ela = S::Key { target: 9, key: 1 };
    assert_eq!(
        intents(edit(so_ela, true, true), &s),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SetInterp {
                target: AnimTarget::new(9),
                key: KeyId::new(1),
                interp,
            },
            TimelineIntent::EndEdit,
        ]
    );
    let na_selecao = S::Key { target: 9, key: 2 };
    assert_eq!(
        intents(edit(na_selecao, false, false), &s),
        vec![TimelineIntent::SetSelectedInterp { interp }]
    );
    let coluna = S::Column {
        t_bits: 1.0_f64.to_bits(),
    };
    let v = intents(edit(coluna, false, false), &s);
    assert_eq!(v.first(), Some(&TimelineIntent::ClearSelection));
    assert_eq!(v.len(), 4, "limpa, junta as duas chaves do instante, muda");
    assert_eq!(
        v.last(),
        Some(&TimelineIntent::SetSelectedInterp { interp })
    );
}

/// O lote do store: o 1.º leva o começo, o último o fim; depois do fim o editor volta a desenhar o
/// que a timeline publica. Criar um ponto vai no lote do começo; apagar é um lote só.
#[test]
fn the_store_hands_the_edit_over_in_batches() {
    use ph2d_editor_core::interaction::state::BezAlvo;
    let mut store = WidgetStore::default();
    let scope = S::Key { target: 9, key: 1 };
    let publicada = EaseSpline::from_bezier(0.1, 0.1, 0.9, 0.9);
    store.set_key_card_curve(Some(publicada));
    let mut com = publicada;
    let i = com.insert(0.5).unwrap();
    store.begin_key_card_bez(scope, BezAlvo::Ponto(i), com);
    store.move_key_card_bez((0.4, 1.3));
    let a = store.take_key_card_bez_edit().unwrap();
    assert!(a.begin && !a.end);
    assert_eq!(a.curve.points().len(), 1);
    assert_eq!(a.curve.node(1).p, [0.4, 1.3]);
    assert_eq!(store.key_card_curve(), Some(a.curve));
    assert_eq!(
        store.take_key_card_bez_edit(),
        None,
        "nada de novo, nada a drenar"
    );
    store.end_key_card_bez();
    let b = store.take_key_card_bez_edit().unwrap();
    assert!(!b.begin && b.end);
    assert_eq!(store.key_card_curve(), Some(publicada));
    store.end_key_card_bez();
    assert_eq!(
        store.take_key_card_bez_edit(),
        None,
        "largar sem arrasto não faz nada"
    );
    let mut sem = a.curve;
    assert!(sem.remove(1));
    store.apply_key_card_bez(scope, sem);
    let c = store.take_key_card_bez_edit().unwrap();
    assert!(c.begin && c.end && c.curve.points().is_empty());
}

/// Dono 15/11: «a mesma curva aparece invertida em y no editor de easing custom». Com o valor a
/// DESCER até à chave seguinte, a timeline publica `desce` e o cartão vira o gráfico; a subir, não.
#[test]
fn a_chave_que_desce_publica_o_cartao_virado() {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    let curva = Interp::bezier(0.2, 0.0, 0.3, 1.0);
    for (v1, desce) in [(-10.0, true), (10.0, false)] {
        let mut store = WidgetStore::default();
        store.open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::TimelineSegment {
                scope: S::Key { target: 9, key: 1 },
            },
        });
        let keys = vec![
            chave(1, 0.0, 0.0, curva, true),
            chave(2, 1.0, v1, Interp::Linear, false),
        ];
        publica_e_drena(&mut store, &snap(vec![faixa(9, keys)]));
        assert_eq!(store.key_card_desce(), desce, "v1 = {v1}");
        assert!(
            store.key_card_curve().is_some(),
            "a curva publica-se nos dois sentidos"
        );
    }
}

/// «Reset timing» do cartão (dono 15/11): a timeline publica `retimado` só quando o trecho da chave
/// tem ritmo; o pedido do cartão vira um passo de undo que tira o ritmo de cada trecho do alcance
/// que o tem (o da coluna inteira, no Summary) — e nada quando nenhum tem.
#[test]
fn o_reset_timing_do_cartao_so_existe_com_ritmo_e_tira_o_num_passo() {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_timeline::RetimeIntent;
    let ritmo = ph2d_anim::Retime::new(0.1, 0.6, 0.5, 0.95);
    for (com, esperado) in [(Some(ritmo), true), (None, false)] {
        let mut store = WidgetStore::default();
        let scope = S::Key { target: 9, key: 1 };
        store.open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::TimelineSegment { scope },
        });
        let mut k0 = chave(1, 0.0, 0.0, Interp::Linear, true);
        k0.retime = com;
        let s = snap(vec![faixa(
            9,
            vec![k0, chave(2, 1.0, 10.0, Interp::Linear, false)],
        )]);
        let _ = crate::state::drain_intents();
        publica_e_drena(&mut store, &s);
        assert_eq!(store.key_card_retimado(), esperado, "{com:?}");
        store.request_key_card_reset_timing(scope);
        publica_e_drena(&mut store, &s);
        let got = crate::state::drain_intents();
        if esperado {
            assert_eq!(
                got,
                vec![
                    TimelineIntent::BeginEdit,
                    TimelineIntent::Retime(RetimeIntent::Set {
                        target: AnimTarget::new(9),
                        key: KeyId::new(1),
                        retime: None,
                    }),
                    TimelineIntent::EndEdit,
                ]
            );
        } else {
            assert!(got.is_empty(), "sem ritmo nada muda: {got:?}");
        }
        assert_eq!(
            store.take_key_card_reset_timing(),
            None,
            "o pedido drena-se"
        );
    }
}
