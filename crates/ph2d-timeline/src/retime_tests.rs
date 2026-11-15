use super::*;
use crate::{PropKind, TimelineDoc, TimelineIntent, TimelineState, apply_intent, remapped_time};
use ph2d_anim::{
    AnimValue, AttributeEvaluator, Easing, EasingFamily, EasingMode, Interp, RationalTime,
};
use ph2d_core::Playhead;

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn bounce() -> Interp {
    Interp::Eased(Easing {
        family: EasingFamily::Bounce,
        mode: EasingMode::Out,
    })
}

fn kv(t: f64, v: f32, interp: Interp, retime: Option<Retime>) -> KeyView {
    KeyView {
        retime,
        id: KeyId::new(0),
        t_seconds: t,
        value: v,
        interp,
        selected: true,
        roving: false,
    }
}

/// Um ritmo que arranca devagar e acaba depressa — longe da identidade nas duas pontas.
const LENTO_DEPOIS_RAPIDO: Retime = Retime {
    x1: 0.6,
    y1: 0.1,
    x2: 0.85,
    y2: 0.5,
};

/// ⭐ **A medição do plano 18 §2** — o Time Remap do OBJECTO não exprime o retime por trecho: o
/// mesmo reajuste do relógio que retima o Bounce de X arrasta também a recta de Y, porque o relógio
/// é da entidade inteira. O Retime da chave de X deixa Y onde estava.
#[test]
fn o_time_remap_do_objeto_retima_todas_as_propriedades_e_o_retime_so_o_trecho() {
    let mut doc = TimelineDoc::new();
    let (x, y, tr) = (
        PropKind::TranslationX,
        PropKind::TranslationY,
        PropKind::TimeRemap,
    );
    let f = AnimValue::Float;
    let (tx, kx) = doc.insert_key(1, x, s(0.0), f(0.0), bounce());
    doc.insert_key(1, x, s(2.0), f(100.0), Interp::Linear);
    let (ty, _) = doc.insert_key(1, y, s(0.0), f(0.0), Interp::Linear);
    doc.insert_key(1, y, s(2.0), f(50.0), Interp::Linear);
    let y_sem = |doc: &TimelineDoc, t: f64| doc.active_clip().track(ty).unwrap().sample(t);
    let antes: Vec<_> = (1..20).map(|i| y_sem(&doc, f64::from(i) * 0.1)).collect();

    // O relógio do objecto com o mesmo ritmo: (0,0) → (2,2) pela Bézier de W.
    let w = LENTO_DEPOIS_RAPIDO;
    let bez = Interp::bezier(w.x1.into(), w.y1.into(), w.x2.into(), w.y2.into());
    doc.insert_key(1, tr, s(0.0), f(0.0), bez);
    doc.insert_key(1, tr, s(2.0), f(2.0), Interp::Linear);
    let y_pelo_relogio: Vec<_> = (1..20)
        .map(|i| y_sem(&doc, remapped_time(&doc, 1, f64::from(i) * 0.1)))
        .collect();
    assert_ne!(
        antes, y_pelo_relogio,
        "o relógio do objecto mexeu no Y que ninguém pediu"
    );

    // O Retime da chave de X: Y lê-se igual, X muda.
    let mut doc = TimelineDoc::new();
    doc.insert_key(1, x, s(0.0), f(0.0), bounce());
    doc.insert_key(1, x, s(2.0), f(100.0), Interp::Linear);
    doc.insert_key(1, y, s(0.0), f(0.0), Interp::Linear);
    doc.insert_key(1, y, s(2.0), f(50.0), Interp::Linear);
    let x_antes = doc.active_clip().track(tx).unwrap().sample(0.7);
    doc.active_clip_mut()
        .track_mut(tx)
        .unwrap()
        .set_retime(kx, Some(w));
    let depois: Vec<_> = (1..20).map(|i| y_sem(&doc, f64::from(i) * 0.1)).collect();
    assert_eq!(antes, depois, "o Retime de X não toca em Y");
    assert_ne!(x_antes, doc.active_clip().track(tx).unwrap().sample(0.7));
}

/// A velocidade que a vista desenha num trecho retimado é a derivada do valor que TOCA
/// (`E'(W(u))·W'(u)·dv/span`), conferida pela diferença central de [`crate::sample_keys`].
#[test]
fn a_velocidade_de_um_bounce_retimado_e_a_derivada_do_que_toca() {
    let keys = [
        kv(0.0, 0.0, bounce(), Some(LENTO_DEPOIS_RAPIDO)),
        kv(2.0, 100.0, Interp::Linear, None),
    ];
    let h = 1.0e-5;
    for i in 1..200 {
        let t = 2.0 * f64::from(i) / 200.0;
        let (a, b) = (
            crate::sample_keys(&keys, t - h).unwrap(),
            crate::sample_keys(&keys, t + h).unwrap(),
        );
        let numerica = (f64::from(b) - f64::from(a)) / (2.0 * h);
        let desenhada = f64::from(crate::sample_speed(&keys, t).unwrap());
        // f32 nas amostras: a diferença central tem erro ~ 1e-5/h no valor de 100.
        let tol = 0.5 + 0.01 * numerica.abs();
        // O quique é um bico: perto dele as duas leituras são de lados diferentes.
        if (numerica - desenhada).abs() > tol {
            let perto = |d: f64| crate::sample_speed(&keys, t + d).unwrap();
            assert!(
                perto(-2.0 * h).signum() != perto(2.0 * h).signum(),
                "t={t}: desenhada {desenhada} × numérica {numerica}"
            );
        }
    }
}

/// O arrasto da alça do ritmo é o inverso da sua ponta: pôr a ponta onde ela já está não muda o
/// ritmo, e subir a ponta de saída acelera o arranque (`W'(0)` = a velocidade ÷ `ref`).
#[test]
fn a_alca_do_ritmo_e_a_ponta_dela_sao_inversas() {
    let (k0, k1) = (
        kv(1.0, 0.0, bounce(), Some(LENTO_DEPOIS_RAPIDO)),
        kv(3.0, 100.0, Interp::Linear, None),
    );
    for which in [0u8, 1u8] {
        let (t, v) = rhythm_handle_tip(&k0, &k1, which).unwrap();
        let w = retime_with_speed_handle(&k0, &k1, which, t, v).unwrap();
        let d = |a: f32, b: f32| (a - b).abs() < 1.0e-5;
        let o = LENTO_DEPOIS_RAPIDO;
        assert!(
            d(w.x1, o.x1) && d(w.y1, o.y1) && d(w.x2, o.x2) && d(w.y2, o.y2),
            "{which}: {w:?}"
        );
    }
    let r = rhythm_reference(&k0, &k1).unwrap();
    assert!((r - 50.0).abs() < 1.0e-9, "ref = 100 / 2 s");
    let (t, _) = rhythm_handle_tip(&k0, &k1, 0).unwrap();
    let w = retime_with_speed_handle(&k0, &k1, 0, t, 75.0).unwrap();
    assert!(
        (w.slope(0.0) - 1.5).abs() < 1.0e-4,
        "W'(0) = 75 / 50: {w:?}"
    );
    let parado = kv(3.0, 0.0, Interp::Linear, None);
    assert_eq!(
        rhythm_handle_tip(&k0, &parado, 0),
        None,
        "trecho parado: sem ritmo"
    );
}

/// Uma alça puxada para baixo do zero pára o trecho nessa ponta — nunca o anda para trás.
#[test]
fn o_ritmo_nunca_anda_para_tras() {
    let (k0, k1) = (
        kv(0.0, 0.0, Interp::Linear, None),
        kv(1.0, 10.0, Interp::Linear, None),
    );
    let w = retime_with_speed_handle(&k0, &k1, 0, 0.3, -40.0).unwrap();
    assert_eq!(w.y1, 0.0);
    let mut antes = 0.0;
    for i in 0..=1000 {
        let v = w.eval(f64::from(i) / 1000.0);
        assert!(v >= antes - 1.0e-9, "W desceu em {i}");
        antes = v;
    }
}

/// O comando: `Set` grava o ritmo da chave num passo de desfazer; `ResetSelected` repõe o das
/// seleccionadas; o Ctrl+Z volta ao ritmo de antes.
#[test]
fn o_comando_grava_repoe_e_desfaz() {
    let mut state = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    let (target, key) = state.doc.insert_key(
        1,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        bounce(),
    );
    state.doc.insert_key(
        1,
        PropKind::TranslationX,
        s(1.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    let ritmo = |st: &TimelineState| {
        st.doc
            .active_clip()
            .track(target)
            .unwrap()
            .key(key)
            .unwrap()
            .retime
    };
    let set = RetimeIntent::Set {
        target,
        key,
        retime: Some(LENTO_DEPOIS_RAPIDO),
    };
    apply_intent(&mut state, &mut ph, TimelineIntent::Retime(set));
    assert_eq!(ritmo(&state), Some(LENTO_DEPOIS_RAPIDO));
    apply_intent(&mut state, &mut ph, TimelineIntent::Undo);
    assert_eq!(ritmo(&state), None, "um passo de desfazer");
    apply_intent(&mut state, &mut ph, TimelineIntent::Redo);
    state
        .selection
        .add(crate::state::SelectedKey { target, key });
    apply_intent(
        &mut state,
        &mut ph,
        TimelineIntent::Retime(RetimeIntent::ResetSelected),
    );
    assert_eq!(ritmo(&state), None, "Reset timing");
}

/// Copiar e colar (e duplicar) leva o ritmo com a chave — uma cópia que perde o ritmo tocaria
/// diferente do original.
#[test]
fn copiar_colar_e_duplicar_levam_o_ritmo() {
    let mut state = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    let x = PropKind::TranslationX;
    let (target, key) = state
        .doc
        .insert_key(1, x, s(0.0), AnimValue::Float(0.0), bounce());
    state
        .doc
        .insert_key(1, x, s(1.0), AnimValue::Float(1.0), Interp::Linear);
    let set = RetimeIntent::Set {
        target,
        key,
        retime: Some(LENTO_DEPOIS_RAPIDO),
    };
    apply_intent(&mut state, &mut ph, TimelineIntent::Retime(set));
    state
        .selection
        .add(crate::state::SelectedKey { target, key });
    apply_intent(&mut state, &mut ph, TimelineIntent::CopySelection);
    apply_intent(&mut state, &mut ph, TimelineIntent::Scrub(3.0));
    apply_intent(&mut state, &mut ph, TimelineIntent::Paste);
    let tr = state.doc.active_clip().track(target).unwrap();
    let colada = tr.keys().iter().find(|k| k.t == s(3.0)).unwrap();
    assert_eq!(colada.retime, Some(LENTO_DEPOIS_RAPIDO), "colar");
    let mut tr = tr.clone();
    let dup = tr.duplicate_keys(&[key], s(5.0));
    assert_eq!(
        tr.key(dup[0]).unwrap().retime,
        Some(LENTO_DEPOIS_RAPIDO),
        "duplicar"
    );
}
