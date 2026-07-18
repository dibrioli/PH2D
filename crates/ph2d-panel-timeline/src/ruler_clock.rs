//! **Which clock a ruler measures** — the question, split out of the painting.
//!
//! It lived in `ruler.rs` until nesting gave it a third answer and the file went past the
//! panel LOC cap. The seam is the one that module's own docs already drew: *"answering here
//! rather than inside the paint loop is what gives that line an oracle at all."* Painting a
//! ruler and deciding what it is a ruler OF are different jobs.

use ph2d_timeline::TimelineViewSnapshot;

/// stand at the wrong second, and the Keys ruler carries none there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulerClock {
    /// Where the playhead stands **on this ruler**, or `None` when the clock it
    /// measures has no answer here (an Arrange-side clip that plays zero/two times).
    pub now: Option<f64>,
    /// This ruler scrubs — registers the scrub hit and mirrors the playhead into it.
    /// Both tabs do (the Keys ruler scrubs the clip clock).
    pub scrub: bool,
    /// This ruler carries the TIMELINE's markers. Arrange always; Keys only without
    /// a stack (then the clip IS the timeline). The loop is NOT gated by this — it is
    /// each view's own, drawn against the very clock this ruler scrubs.
    pub markers: bool,
}

/// **Which clock this tab's ruler measures.** Pure, and tested as such.
///
/// The playhead is paint, not a widget — no hit index remembers it, so a seam test
/// can prove the ruler's *hits* and never its *line*. Answering here rather than
/// inside the paint loop is what gives that line an oracle at all.
pub(crate) fn clock_for(tab: crate::tab::Tab, snap: &TimelineViewSnapshot) -> RulerClock {
    if tab.shows_lanes() {
        // **Inside a container the Arrange lanes are ITS interior**, laid out in its own
        // seconds (ADR-0133 §5) — so the ruler marks its clock, not the timeline's. Marking
        // `time_seconds` there put the playhead off by exactly where the instance starts: the
        // strip under the marker was not the strip the marker claimed.
        //
        // Scrubbing stays on the TIMELINE playhead (the transport owns it, and a container has
        // no playhead of its own to move) and the markers stay off: they are stamped in
        // timeline seconds and would sit at the wrong second in here.
        if let Some(now) = snap.host_time {
            return RulerClock {
                now: Some(now),
                scrub: true,
                markers: false,
            };
        }
        if !snap.crumbs.is_empty() {
            // Open, but the container is not playing exactly once here: there is no single
            // "now" in it. Draw NO playhead rather than one at a second it is not at —
            // the same refusal `clip_playhead` makes, for the same reason.
            return RulerClock {
                now: None,
                scrub: true,
                markers: false,
            };
        }
        // Arrange IS the timeline: the transport's playhead, and it owns the markers
        // because they are drawn against the very clock it scrubs.
        return RulerClock {
            now: Some(snap.time_seconds),
            scrub: true,
            markers: true,
        };
    }
    // Keys scrubs the active clip's own clock (published as `clip_time`). It carries
    // the markers **only when there is no stack** — then the clip IS the timeline,
    // one clock, and the Keys ruler is the timeline ruler it has always been. Under a
    // stack the clip clock ≠ the timeline clock, so timeline markers would sit at the
    // wrong second and belong on the Arrange ruler only. Its LOOP, however, is the
    // clip's own and is always drawn.
    RulerClock {
        now: snap.clip_time,
        scrub: true,
        markers: !snap.stacked(),
    }
}

#[cfg(test)]
mod tests {

    fn snap(stacked: bool, playhead: f64, clip_time: Option<f64>) -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            time_seconds: playhead,
            clip_time,
            lanes: if stacked {
                vec![ph2d_timeline::LaneView {
                    name: "L".into(),
                    muted: false,
                    weight: 1.0,
                    mode: ph2d_timeline::LaneMode::Override,
                    strips: Vec::new(),
                }]
            } else {
                Vec::new()
            },
            ..TimelineViewSnapshot::default()
        }
    }
    use super::*;
    use crate::tab::Tab;
    #[test]
    fn without_a_stack_both_tabs_rule_the_one_clock_there_is() {
        let s = snap(false, 1.25, Some(1.25));
        for tab in [Tab::Keys, Tab::Arrange] {
            assert_eq!(
                clock_for(tab, &s),
                RulerClock {
                    now: Some(1.25),
                    scrub: true,
                    markers: true
                },
                "{tab:?} lost the ruler it always had"
            );
        }
    }
    #[test]
    fn each_tab_reads_its_own_clock() {
        let s = snap(true, 4.0, Some(2.0));
        assert_eq!(clock_for(Tab::Keys, &s).now, Some(2.0), "the CLIP's clock");
        assert_eq!(
            clock_for(Tab::Arrange, &s).now,
            Some(4.0),
            "the TIMELINE's clock"
        );
    }
    #[test]
    fn both_tabs_scrub_their_own_clock() {
        for stacked in [false, true] {
            assert!(
                clock_for(Tab::Keys, &snap(stacked, 4.0, Some(2.0))).scrub,
                "the Keys ruler scrubs the clip clock, stacked={stacked}"
            );
            assert!(clock_for(Tab::Arrange, &snap(stacked, 4.0, Some(2.0))).scrub);
        }
    }
    #[test]
    fn the_markers_follow_the_timeline_clock() {
        // No stack: the Keys ruler is the timeline ruler, so it keeps the markers.
        assert!(clock_for(Tab::Keys, &snap(false, 4.0, Some(4.0))).markers);
        // Under a stack: only Arrange carries them.
        assert!(!clock_for(Tab::Keys, &snap(true, 4.0, Some(2.0))).markers);
        for stacked in [false, true] {
            assert!(clock_for(Tab::Arrange, &snap(stacked, 4.0, Some(2.0))).markers);
        }
    }
}
