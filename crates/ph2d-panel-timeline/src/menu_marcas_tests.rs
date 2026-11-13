//! A marca do estado corrente: a linha que acende é a da chave/faixa sob o clique.

use super::*;
use ph2d_anim::{Easing, EasingFamily};
use ph2d_timeline::{AnimTarget, KeyId, KeyView, PropKind, TrackView};

fn chave(id: u64, t: f64, interp: Interp) -> KeyView {
    KeyView {
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
