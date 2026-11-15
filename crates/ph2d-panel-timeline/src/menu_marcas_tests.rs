//! A marca do estado corrente: a linha que acende é a da chave/faixa sob o clique.

use super::*;
use ph2d_anim::{Easing, EasingFamily};
use ph2d_timeline::{AnimTarget, KeyId, KeyView, PropKind, TrackView};

fn chave(id: u64, t: f64, interp: Interp) -> KeyView {
    KeyView {
        retime: None,
        id: KeyId::new(id),
        t_seconds: t,
        value: 0.0,
        interp,
        selected: false,
        roving: false,
    }
}

fn snap() -> TimelineViewSnapshot {
    let quad_out = Interp::Eased(Easing {
        family: EasingFamily::Quad,
        mode: EasingMode::Out,
    });
    TimelineViewSnapshot {
        tracks: vec![TrackView {
            target: AnimTarget::new(9),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            buffer_ghost: None,
            pre: Extrap::Hold,
            post: Extrap::PingPong,
            expr: None,
            keys: vec![chave(1, 0.0, Interp::Hold), chave(2, 1.0, quad_out)],
        }],
        fps: 24.0,
        ..Default::default()
    }
}

#[test]
fn the_lit_row_is_the_interpolation_and_extrapolation_under_the_click() {
    let s = snap();
    let seg = |key| K::TimelineSegment {
        scope: S::Key { target: 9, key },
    };
    assert_eq!(linhas_acesas(seg(1), &s), Some(vec![c::CTX_MENU_TL_HOLD]));
    // Uma chave suavizada acende o modo E a forma (o cartão diz «Quad» e o ease mantém-na).
    assert_eq!(
        linhas_acesas(seg(2), &s),
        Some(vec![c::CTX_MENU_TL_EASE_OUT, c::CTX_MENU_TL_FAM_QUAD])
    );
    // O título do cartão: o quadro da chave e o canal.
    let t = titulo(seg(2), &s);
    assert!(t.starts_with("Key at frame 24"), "{t:?}");
    assert!(t.contains("Position X"), "{t:?}");
    // O submenu acende a família só no modo dela.
    let ease = |mode| K::TimelineSegmentEase {
        scope: S::Key { target: 9, key: 2 },
        mode,
    };
    assert_eq!(
        linhas_acesas(ease(c::TL_EASE_MODE_OUT), &s),
        Some(vec![c::CTX_MENU_TL_FAM_QUAD])
    );
    assert_eq!(linhas_acesas(ease(c::TL_EASE_MODE_IN), &s), Some(vec![]));
    let extrap = |side| K::TimelineExtrap {
        target: 9,
        side,
        grupo: false,
    };
    assert_eq!(
        linhas_acesas(extrap(crate::ids::TL_EXTRAP_SIDE_PRE), &s),
        Some(vec![c::CTX_MENU_TL_EXTRAP_HOLD])
    );
    assert_eq!(
        linhas_acesas(extrap(crate::ids::TL_EXTRAP_SIDE_POST), &s),
        Some(vec![c::CTX_MENU_TL_EXTRAP_PINGPONG])
    );
    // Uma chave que não existe não acende nada (o cartão abre só com o título).
    assert_eq!(linhas_acesas(seg(77), &s), Some(vec![]));
}

/// O botão do cartão desenha a conta do documento: cada forma × modo é o `Easing::eval` dela, e o
/// que não é forma (Hold, Straight) não tem régua.
/// Dono 15/11: o menu da lane não dizia qual modo está ativo.
#[test]
fn the_lane_menu_lights_the_lanes_mode() {
    use ph2d_timeline::{LaneMode, LaneView};
    let lane = |mode| LaneView {
        name: String::new(),
        muted: false,
        weight: 1.0,
        mode,
        strips: Vec::new(),
    };
    let s = TimelineViewSnapshot {
        lanes: vec![lane(LaneMode::Override), lane(LaneMode::Additive)],
        ..Default::default()
    };
    let acesa = |lane| linhas_acesas(K::TimelineLane { lane }, &s);
    assert_eq!(acesa(0), Some(vec![c::CTX_MENU_TL_LANE_OVERRIDE]));
    assert_eq!(acesa(1), Some(vec![c::CTX_MENU_TL_LANE_ADDITIVE]));
    assert_eq!(acesa(2), None, "lane que não existe: nada aceso");
}

/// «Procure problemas similares» (dono 15/11): a faixa (Once/Loop/Ping-Pong), a forma do fade e o
/// Auto-Orient também não diziam o estado.
#[test]
fn the_strip_fade_and_auto_orient_menus_light_their_state() {
    use ph2d_anim::EasingFamily as F;
    use ph2d_timeline::{LaneMode, LaneView, StripId, StripLoop, StripView};
    let quad_out = Easing {
        family: F::Quad,
        mode: EasingMode::Out,
    };
    let linear = Easing {
        family: F::Linear,
        mode: EasingMode::In,
    };
    let faixa = StripView {
        id: StripId(7),
        clip_name: "C".into(),
        container: None,
        t_start: 0.0,
        t_end: 1.0,
        blend_in: 0.0,
        blend_out: 0.0,
        lead_in: 0.0,
        lead_out: 0.0,
        volta: 0.0,
        ease_locked_in: false,
        ease_locked_out: false,
        curve_in: Some(quad_out),
        curve_out: Some(linear),
        seam: None,
        loop_mode: StripLoop::PingPong,
        speed: 1.0,
        marks: [0.0; 4],
    };
    let mut s = snap();
    s.lanes = vec![LaneView {
        name: String::new(),
        muted: false,
        weight: 1.0,
        mode: LaneMode::Override,
        strips: vec![faixa],
    }];
    let acesa = |k| linhas_acesas(k, &s);
    assert_eq!(
        acesa(K::TimelineStrip { lane: 0, strip: 7 }),
        Some(vec![c::CTX_MENU_TL_STRIP_PINGPONG])
    );
    let fade = |edge| K::TimelineSegment {
        scope: S::StripFade {
            lane: 0,
            strip: 7,
            edge,
        },
    };
    assert_eq!(acesa(fade(0)), Some(vec![c::CTX_MENU_TL_EASE_OUT]));
    assert_eq!(acesa(fade(1)), Some(vec![c::CTX_MENU_TL_LINEAR]));
    let sub = |mode| K::TimelineSegmentEase {
        scope: S::StripFade {
            lane: 0,
            strip: 7,
            edge: 0,
        },
        mode,
    };
    assert_eq!(
        acesa(sub(c::TL_EASE_MODE_OUT)),
        Some(vec![c::CTX_MENU_TL_FAM_QUAD])
    );
    assert_eq!(acesa(sub(c::TL_EASE_MODE_IN)), Some(Vec::new()));
    // Fade de fábrica: Smooth.
    s.lanes[0].strips[0].curve_in = None;
    assert_eq!(
        linhas_acesas(fade(0), &s),
        Some(vec![c::CTX_MENU_TL_FADE_SMOOTH])
    );
    // Auto-Orient: aceso só quando pedido.
    let path = K::TimelineTrackPath { target: 9 };
    assert_eq!(linhas_acesas(path, &s), Some(Vec::new()));
    s.auto_orient_on = vec![1];
    assert_eq!(
        linhas_acesas(path, &s),
        Some(vec![c::CTX_MENU_TL_AUTO_ORIENT])
    );
}

#[test]
fn the_card_buttons_draw_the_documents_own_easing() {
    use ph2d_anim::EasingMode;
    let modos = [
        (c::TL_EASE_MODE_IN, EasingMode::In),
        (c::TL_EASE_MODE_OUT, EasingMode::Out),
        (c::TL_EASE_MODE_INOUT, EasingMode::InOut),
    ];
    for row in c::TIMELINE_EASE_MENU {
        for (fio, modo) in modos {
            let Some(Preset::Eased(e)) = preset_for(row.0, fio) else {
                panic!("toda forma do cartão resolve para um easing");
            };
            assert_eq!(e.mode, modo);
            for i in 0..=8 {
                let u = i as f32 / 8.0;
                let got = amostra_ease(row.0, fio, u).expect("forma tem régua");
                assert!((f64::from(got) - e.eval(f64::from(u))).abs() < 1e-6);
            }
        }
    }
    assert_eq!(
        amostra_ease(
            c::CTX_MENU_TL_HOLD,
            ph2d_editor_core::interaction::TL_NO_EASE_MODE,
            0.5
        ),
        None
    );
    assert_eq!(
        amostra_ease(c::CTX_MENU_TL_LINEAR, c::TL_EASE_MODE_IN, 0.5),
        None
    );
}
