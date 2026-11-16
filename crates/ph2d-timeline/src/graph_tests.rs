use super::*;
use crate::snapshot::TimelineViewSnapshot;
use crate::state::TimelineState;
use ph2d_anim::{AttributeEvaluator, Interp, RationalTime, Track};
use ph2d_core::Playhead;

fn key(t: f64, v: f32, interp: Interp) -> KeyView {
    KeyView {
        retime: None,
        id: ph2d_anim::KeyId::new(0),
        t_seconds: t,
        value: v,
        interp,
        selected: false,
        roving: false,
    }
}

/// The played slope at each end, measured on the curve itself (`sample_keys`, one-sided
/// difference) — an oracle independent of the chain rule the handles use.
fn slope_medida(k0: &KeyView, k1: &KeyView, which: u8) -> f64 {
    // Second-order one-sided difference: `sample_keys` is `f32`, so `h` cannot be tiny.
    let (h, keys) = (1.0e-3, [k0.clone(), k1.clone()]);
    let s = |t: f64| f64::from(sample_keys(&keys, t).unwrap_or(f32::NAN));
    let (t, d) = if which == 0 {
        (k0.t_seconds, h)
    } else {
        (k1.t_seconds, -h)
    };
    (-3.0 * s(t) + 4.0 * s(t + d) - s(t + 2.0 * d)) / (2.0 * d)
}

fn retimado(interp: Interp) -> (KeyView, KeyView) {
    let mut k0 = key(0.0, 0.0, interp);
    k0.retime = Some(ph2d_anim::Retime::new(0.1, 0.6, 0.5, 0.95));
    (k0, key(2.0, 10.0, Interp::Linear))
}

/// Item aberto do 15/11: na vista de VALOR, sobre um trecho retimado, as alças eram as do
/// easing sem o ritmo — não tocavam a curva desenhada. Cada braço tem de ser TANGENTE à curva
/// que toca, para toda família.
#[test]
fn num_trecho_retimado_a_alca_de_valor_e_tangente_a_curva_desenhada() {
    for interp in [
        Interp::Linear,
        Interp::bezier(0.3, 0.1, 0.6, 0.9),
        Interp::bezier_w(0.4, 3.0, 0.7, -1.0),
    ] {
        let (k0, k1) = retimado(interp);
        let pts = segment_handle_points(&k0, &k1);
        for (which, (anc, p)) in [((0.0, 0.0), pts[0]), ((2.0, 10.0), pts[1])]
            .into_iter()
            .enumerate()
        {
            let braco = (p.1 - anc.1) / (p.0 - anc.0);
            let medida = slope_medida(&k0, &k1, which as u8);
            assert!(
                (braco - medida).abs() <= 2.0e-3 * medida.abs().max(1.0),
                "{interp:?} lado {which}: braço {braco} × curva {medida}"
            );
        }
    }
}

/// Um ritmo íngreme (`W'(0) = 18`) punha a ponta do braço muito acima da banda (foto 15/11): nem
/// se via nem se agarrava. A ponta fica dentro dos valores que o trecho toca — num trecho
/// monótono de 0 a 10, entre 0 e 10 — e o braço continua tangente.
#[test]
fn com_ritmo_ingreme_a_ponta_do_braco_fica_dentro_do_que_o_trecho_toca() {
    let (mut k0, k1) = retimado(Interp::Linear);
    k0.retime = Some(ph2d_anim::Retime::new(0.05, 0.9, 0.5, 1.0));
    let pts = segment_handle_points(&k0, &k1);
    for (which, p) in pts.into_iter().enumerate() {
        assert!(
            (-1e-9..=10.0 + 1e-9).contains(&p.1),
            "lado {which}: ponta em {p:?}"
        );
        assert!(
            p.0 > 0.0 && p.0 < 2.0,
            "lado {which}: ponta fora do trecho {p:?}"
        );
    }
    let braco = pts[0].1 / pts[0].0;
    let medida = slope_medida(&k0, &k1, 0);
    // 5e-3: the difference's own truncation on a curve this steep (90 exact × 89,76 measured).
    assert!(
        (braco - medida).abs() <= 5.0e-3 * medida.abs(),
        "{braco} × {medida}"
    );
}

/// A alça do tempo no CANTO da caixa (`x2 = 1`) faz o movimento chegar na vertical: aquele
/// braço não tem ponta finita e some (como na vista de velocidade); o outro lado fica.
#[test]
fn com_a_chegada_na_vertical_aquele_braco_some_e_o_outro_fica() {
    let (mut k0, k1) = retimado(Interp::bezier(0.3, 0.1, 0.6, 0.9));
    assert!(segment_handle_drawn(&k0, &k1, 0) && segment_handle_drawn(&k0, &k1, 1));
    k0.retime = Some(ph2d_anim::Retime::new(0.3, 0.3, 1.0, 0.0));
    assert!(
        segment_handle_drawn(&k0, &k1, 0),
        "a saída tem tangente finita"
    );
    assert!(!segment_handle_drawn(&k0, &k1, 1), "a chegada é vertical");
}

/// Dono 15/11 («alças fora de vista»): o braço de uma curva pronta (Elastic, Back…) é a
/// direção do que toca; a ponta encurta NESSA direção até caber nos valores que o trecho toca
/// — a faixa os contém (`drawn_extent` amostra a curva), por isso a ponta nunca sai dela.
#[test]
fn o_braco_de_uma_curva_pronta_fica_dentro_do_que_o_trecho_toca() {
    use ph2d_anim::{Easing, EasingFamily as F, EasingMode as M};
    for fam in [F::Elastic, F::Back, F::Bounce, F::Expo, F::Circ] {
        for modo in [M::In, M::Out, M::InOut] {
            let interp = Interp::Eased(Easing::new(fam, modo));
            let (mut k0, k1) = (key(0.0, 0.0, interp), key(1.0, 10.0, Interp::Linear));
            k0.selected = true;
            let (lo, hi) = drawn_extent(&[k0.clone(), k1.clone()], 0.0, 1.0, 256).unwrap();
            let (h0, h1) = interp.tangent_handles();
            let cruas = [
                handle_point(0.0, 0.0, 1.0, 10.0, h0),
                handle_point(0.0, 0.0, 1.0, 10.0, h1),
            ];
            for (w, (t, v)) in segment_handle_points(&k0, &k1).into_iter().enumerate() {
                let c = cruas[w];
                let (a0, a1) = if w == 0 {
                    ((0.0, 0.0), c)
                } else {
                    ((1.0, 10.0), c)
                };
                assert!(
                    v as f32 >= lo - 1e-3 && v as f32 <= hi + 1e-3,
                    "{fam:?} {modo:?} lado {w}: ponta {v} fora de {lo}..{hi}"
                );
                // A mesma direção do braço cru (a velocidade não muda, só o alcance).
                let cruz = (c.0 - a0.0) * (v - a0.1) - (c.1 - a0.1) * (t - a0.0);
                assert!(
                    cruz.abs() < 1e-6 * (1.0 + a1.1.abs()),
                    "{fam:?} {modo:?}: direção mudou"
                );
            }
        }
    }
}

/// Puxar uma alça de um trecho retimado reconstrói o trecho (Override, o chamador tira o
/// ritmo): o OUTRO lado fica onde estava desenhado — nada salta na outra ponta. A Custom com
/// pontos retimada é a exceção (mantém ritmo e pontos): `graph_pontos` prova-a.
#[test]
fn puxar_a_alca_de_um_trecho_retimado_deixa_a_outra_ponta_onde_estava() {
    {
        let interp = Interp::bezier(0.3, 0.1, 0.6, 0.9);
        let (k0, k1) = retimado(interp);
        let antes = segment_handle_points(&k0, &k1)[1];
        let mut novo = k0.clone();
        novo.interp = weighted_with_handle(&k0, &k1, 0, 0.5, 4.0);
        novo.retime = None;
        assert!(
            matches!(novo.interp, Interp::BezierW { .. }),
            "{:?}",
            novo.interp
        );
        let depois = segment_handle_points(&novo, &k1);
        assert!((depois[1].0 - antes.0).abs() < 1e-9 && (depois[1].1 - antes.1).abs() < 1e-6);
        assert!((depois[0].0 - 0.5).abs() < 1e-9 && (depois[0].1 - 4.0).abs() < 1e-6);
    }
}

/// Dragging an END handle of a Custom curve with points (`Spline`) moves that handle and keeps
/// the points — converting to `BezierW` would silently delete them.
#[test]
fn dragging_an_end_handle_of_a_curve_with_points_keeps_the_points() {
    let mut sp = ph2d_anim::EaseSpline::from_bezier(0.3, 0.0, 0.7, 1.0);
    sp.insert(0.5);
    let (k0, k1) = (
        key(0.0, 0.0, Interp::Spline(sp)),
        key(2.0, 10.0, Interp::Linear),
    );
    let out = weighted_with_handle(&k0, &k1, 0, 0.5, 4.0);
    let Interp::Spline(novo) = out else {
        panic!("a curva com pontos virou {out:?}");
    };
    assert_eq!(novo.points().len(), 1);
    assert_eq!(novo.end_handles().0, [0.25, 0.4]);
}

/// THE golden assertion (plan W3.E6): the polyline the panel draws is the
/// animation that plays. A second interpolation implementation is exactly the
/// bug this test exists to prevent, so compare bit-for-bit, not approximately.
#[test]
fn sample_keys_matches_the_real_track_sampler_bit_for_bit() {
    use crate::prop::PropKind;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    let interps = [
        Interp::Linear,
        Interp::Hold,
        Interp::bezier(0.2, 1.4, 0.8, -0.3), // overshoot both ends
        Interp::bezier_w(0.3, 4.0, 0.7, -2.5), // weighted (value-space)
        Interp::Eased(ph2d_anim::Easing::new(
            ph2d_anim::EasingFamily::Cubic,
            ph2d_anim::EasingMode::InOut,
        )),
    ];
    for (i, interp) in interps.iter().enumerate() {
        crate::apply_intent(
            &mut st,
            &mut ph,
            crate::TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TranslationX,
                t: RationalTime::from_seconds(i as f64),
                value: AnimValue::Float(i as f32 * 3.0 - 1.0),
                interp: *interp,
            },
        );
    }
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    let track: &Track = st.doc.active_clip().track(target).unwrap();
    let keys = &snap.tracks[0].keys;

    // Sweep past both ends so the flat clamps are covered too.
    for i in -20..=420 {
        let t = f64::from(i) / 100.0;
        let want = match track.sample(t) {
            AnimValue::Float(v) => v,
            other => panic!("expected Float, got {other:?}"),
        };
        assert_eq!(
            sample_keys(keys, t),
            Some(want),
            "graph polyline diverged from the sampler at t = {t}"
        );
    }
}

#[test]
fn sampling_an_empty_track_has_no_value() {
    assert_eq!(sample_keys(&[], 0.5), None);
}

#[test]
fn a_single_key_holds_its_value_everywhere() {
    let keys = [key(1.0, 7.0, Interp::Linear)];
    assert_eq!(sample_keys(&keys, -5.0), Some(7.0));
    assert_eq!(sample_keys(&keys, 100.0), Some(7.0));
}

#[test]
fn coincident_keys_do_not_divide_by_zero() {
    // Two keys at the same instant: the segment has no span; `u` is 0, so the
    // sampler holds the first value rather than producing NaN.
    let keys = [
        key(1.0, 2.0, Interp::Linear),
        key(1.0, 9.0, Interp::Linear),
        key(2.0, 9.0, Interp::Linear),
    ];
    assert!(sample_keys(&keys, 1.0).unwrap().is_finite());
}

#[test]
fn value_extent_spans_the_keys() {
    let keys = [
        key(0.0, 3.0, Interp::Linear),
        key(1.0, -2.0, Interp::Linear),
        key(2.0, 0.5, Interp::Linear),
    ];
    assert_eq!(value_extent(&keys), Some((-2.0, 3.0)));
    assert_eq!(value_extent(&[]), None);
}

#[test]
fn drawn_extent_covers_an_overshoot_the_keys_do_not() {
    // The bug: fitting the band to `value_extent` (the keys) left an
    // overshooting curve drawn far outside the band, over the next row.
    let mut k0 = key(0.0, 0.0, Interp::bezier(0.2, 2.5, 0.8, -1.5));
    k0.selected = true;
    let keys = [k0, key(1.0, 10.0, Interp::Linear)];
    assert_eq!(value_extent(&keys), Some((0.0, 10.0)));
    let (lo, hi) = drawn_extent(&keys, 0.0, 1.0, 200).unwrap();
    assert!(hi > 10.0, "the curve rises past the last key: {hi}");
    assert!(lo < 0.0, "and dips below the first: {lo}");
}

#[test]
fn drawn_extent_includes_the_handles_of_a_selected_segment() {
    // A handle can sit outside its own curve (a bezier stays inside the
    // convex hull of its control points), so fitting the curve alone would
    // still let the handle escape the band.
    let mut k0 = key(0.0, 0.0, Interp::bezier(0.5, 3.0, 0.5, 3.0));
    let keys_unselected = [k0.clone(), key(1.0, 1.0, Interp::Linear)];
    let curve_only = drawn_extent(&keys_unselected, 0.0, 1.0, 200).unwrap();
    k0.selected = true;
    let keys = [k0, key(1.0, 1.0, Interp::Linear)];
    let with_handles = drawn_extent(&keys, 0.0, 1.0, 200).unwrap();
    assert_eq!(with_handles.1, 3.0, "the handle at v = 3 is in range");
    assert!(with_handles.1 > curve_only.1);
}

#[test]
fn drawn_extent_only_sees_the_visible_window() {
    let keys = [
        key(0.0, 0.0, Interp::Linear),
        key(1.0, 100.0, Interp::Linear),
    ];
    // A window entirely before the first key: the curve flat-clamps to it.
    assert_eq!(drawn_extent(&keys, -2.0, -1.0, 8), Some((0.0, 0.0)));
    assert_eq!(drawn_extent(&keys, 0.0, 1.0, 8), Some((0.0, 100.0)));
    assert_eq!(drawn_extent(&[], 0.0, 1.0, 8), None);
}

#[test]
fn drawn_extent_survives_a_degenerate_sample_count() {
    let keys = [key(0.0, 2.0, Interp::Linear), key(1.0, 4.0, Interp::Linear)];
    let (lo, hi) = drawn_extent(&keys, 0.0, 1.0, 0).unwrap();
    assert!(lo.is_finite() && hi.is_finite() && lo <= hi);
}

#[test]
fn a_handle_round_trips_through_the_time_value_plane() {
    let (t0, v0, t1, v1) = (1.0, 10.0, 3.0, 20.0);
    let h = (0.25, 1.4); // overshoot past the end value
    let (t, v) = handle_point(t0, v0, t1, v1, h);
    assert_eq!((t, v), (1.5, 24.0));
    let (hx, hy) = handle_coords(t0, v0, t1, v1, t, v);
    assert_eq!((hx, hy), (0.25, Some(1.4)));
}

#[test]
fn handle_x_is_clamped_but_y_is_free_to_overshoot() {
    let (hx, hy) = handle_coords(0.0, 0.0, 1.0, 1.0, 5.0, 9.0);
    assert_eq!(hx, 1.0, "dragging past the next key pins x at the endpoint");
    assert_eq!(hy, Some(9.0), "y is unbounded — that is what overshoot is");
    let (hx, _) = handle_coords(0.0, 0.0, 1.0, 1.0, -5.0, 0.0);
    assert_eq!(hx, 0.0);
}

#[test]
fn a_flat_segment_has_no_representable_handle_y() {
    // v0 == v1: `Interp::Bezier` stores y as a fraction of the value change,
    // so there is nothing to express. The caller must keep the old y.
    let (hx, hy) = handle_coords(0.0, 5.0, 2.0, 5.0, 1.0, 99.0);
    assert_eq!(hx, 0.5);
    assert_eq!(hy, None);
}

#[test]
fn a_zero_length_segment_pins_the_handle_at_its_start() {
    let (hx, hy) = handle_coords(2.0, 0.0, 2.0, 1.0, 2.0, 0.5);
    assert_eq!((hx, hy), (0.0, Some(0.5)));
}
