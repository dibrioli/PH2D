//! Unit tests for [`super`] (`types.rs`) — extracted to a `#[path]` sibling so
//! the type-heavy module stays under the 700-LOC cap (sibling of `types_menu.rs`,
//! which the same cap already carved off). A CHILD module of `types`, so `super`
//! is `types` and `super::super` is `interaction`, exactly as when it lived inline.

use super::TimelineHitKind as K;

/// **Exactly one surface upgrades a second tap, and the list is pinned.**
///
/// The container bar (double-click enters). Every other surface treats the pair as a plain
/// Click on purpose — this gate is what makes widening or narrowing that list a decision
/// instead of a drive-by: the container bar was dead under the mouse precisely because the
/// list lived inline in `pointer_up.rs` where no test enumerated it. ⚠️ The MARKER left this
/// list on 2026-07-25 — its rename / set-signal moved to the right-click menu, so a second
/// tap on a pennant is a plain Click (re-seek) and MUST NOT synthesize a `DoubleClick`
/// `marker_drag` no longer handles.
#[test]
fn exactly_the_container_bar_wants_a_double_click() {
    let wants = [K::ContainerRow { index: 0 }];
    let plain = [
        K::Lane,
        K::Marker { index: 0 },
        K::Key { target: 0, key: 0 },
        K::SummaryKey { t_bits: 0 },
        K::Twirl { target: 0 },
        K::Row {
            target: 0,
            menu: super::super::TrackMenuKind::Plain,
        },
        K::SummaryLock,
        K::LabelSplitter,
        K::CurveAnchor { target: 0, key: 0 },
        K::CurveHandle {
            target: 0,
            key: 0,
            which: 0,
        },
        K::GraphResize,
        K::LoopBrace { edge: 0 },
        K::Strip {
            lane: 0,
            strip: 0,
            edge: 2,
        },
        K::LaneHeader { lane: 0 },
        K::ResizeEdge { edges: 0 },
        K::DurationHandle,
    ];
    assert!(wants.iter().all(|k| k.wants_double_click()));
    assert!(plain.iter().all(|k| !k.wants_double_click()));
}
