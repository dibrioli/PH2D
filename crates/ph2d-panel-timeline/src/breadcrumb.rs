//! The breadcrumb: **where in the nesting the animator is**, and the way back out
//! ([ADR-0133] §5).
//!
//! # Why a trail and not a tab
//!
//! The research split the two lineages cleanly: **edit-in-place with a breadcrumb** is the 2D
//! animation lineage (Flash/Animate's edit bar, Harmony's `Top` button), and **a new tab** is
//! the compositing lineage (After Effects). No formal consensus says the tab is worse — but
//! the *symptom* is documented over and over, always the same one: you lose the parent's
//! context, and AE users rebuild it by hand (locking a viewer, `View > New Viewer`) to get
//! back what edit-in-place gives for free. Adobe answered with navigation aids, never with
//! in-place editing.
//!
//! ⚠️ And the panel's existing `Tab::{Keys, Arrange}` is **not** the same axis: the tab says
//! *which half you are looking at*, the trail says *where you are*. They compose — the trail
//! puts you in a container, and the tabs then show that container's keys or its arrangement.
//!
//! # Zero-width at the root
//!
//! At the scene root there is nothing to go back to, so the trail paints nothing and measures
//! zero. A document that never touches containers pays nothing and sees nothing.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::ids;

/// One segment's width. Fixed rather than measured: the trail sits in a flow layout that
/// wraps, and a segment that grew with a container's name would make the whole bar reflow
/// every rename.
const SEG_W: f32 = 74.0; // LITERAL-PX-OK: breadcrumb segment width

/// Width reserved for the status readout that follows the trail. Fixed for the same reason
/// [`SEG_W`] is: it rides a flow layout, and a readout that grew with its own number would
/// reflow the whole bar every time the playhead crossed into or out of the instance.
const STATUS_W: f32 = 132.0; // LITERAL-PX-OK: host-window readout width

/// **Which depth the segment in slot `slot` pops to**, for a path `len` containers deep — the
/// elision rule, as a function of a LENGTH and nothing else.
///
/// Slot 0 is always the scene root (depth 0). When the path is deeper than
/// [`ids::TIMELINE_CRUMB`] can host, the trail elides from the **outside**: the root and the
/// innermost levels survive. The cap is the id array — chrome cannot mint a hit id at runtime,
/// so a longer trail would paint a segment nothing could click — and it is a cap on the TRAIL,
/// never on nesting depth, which the ADR measured and found no resource to limit. Dropping the
/// outer levels rather than the inner ones is the only honest choice: a trail that shows every
/// level EXCEPT where you are is worse than a short one.
///
/// ⚠️ It takes a length rather than the snapshot because **the click must not need the
/// snapshot**. The panel's own path moves the instant you enter a container; the published
/// crumbs only catch up next frame. Deriving the depth from the panel's path keeps the two
/// from ever disagreeing about what a segment means — the paint asks with the snapshot's
/// length, the click with the panel's own, and the rule between them is this one function.
pub(crate) fn depth_of_slot(slot: usize, len: usize) -> Option<usize> {
    if len == 0 {
        return None; // the root paints no trail at all
    }
    let keep = ids::TIMELINE_CRUMB.len().saturating_sub(1);
    let skip = len.saturating_sub(keep);
    match slot {
        0 => Some(0),
        k if k <= len.min(keep) => Some(skip + k),
        _ => None,
    }
}

/// **The trail, as `(depth, label)`** — what the paint walks.
///
/// Depths come from [`depth_of_slot`], so a segment can never pop somewhere other than what it
/// shows. Empty at the root, where there is nothing to go back to: a document that never
/// touches containers paints nothing and measures zero.
pub(crate) fn trail(snap: &TimelineViewSnapshot) -> Vec<(usize, String)> {
    let len = snap.crumbs.len();
    (0..=len.min(ids::TIMELINE_CRUMB.len().saturating_sub(1)))
        .filter_map(|slot| {
            let d = depth_of_slot(slot, len)?;
            let label = if d == 0 {
                ph2d_i18n::tr("panel.timeline.crumb_root").to_owned()
            } else {
                // `d - 1`: depth 1 is `crumbs[0]` — segment 0 is the root.
                snap.crumbs.get(d - 1)?.1.clone()
            };
            Some((d, label))
        })
        .collect()
}

/// **What the open container is doing at this instant**, in SCENE seconds — or `None` at the
/// root, where there is nothing to relate.
///
/// # Why this exists, and why a second RULER does not
///
/// Inside a container two clocks are on screen at once: the transport chip keeps showing the
/// scene's second (it always did — that is worth saying, because the note this replaced
/// claimed the timeline's time became invisible in here, and the code says otherwise), while
/// the ruler counts the interior's. Seeing `8.00` in the chip and the playhead at `4` on the
/// ruler is a contradiction until you know where the instance starts.
///
/// After Effects answers this with a second, stacked time ruler, and the research called that
/// the only mitigation anybody shipped. We do not need it: AE's Layer panel has no other
/// readout of comp time, and ours does. A second ruler would spend a permanent row of a short
/// panel re-displaying a number already on screen — what is actually missing is the RELATION
/// between the two, which is one line of text.
///
/// And when the container is not playing at the current second, this is what says so. The
/// ruler already refuses to draw a playhead there ([`crate::ruler_clock`]); an absent marker
/// with no explanation reads as broken, and naming the refusal is this module's whole idiom.
pub(crate) fn status(snap: &TimelineViewSnapshot) -> Option<Status> {
    if snap.crumbs.is_empty() {
        return None;
    }
    Some(
        snap.host_map
            .map_or(Status::NotPlaying, |m| Status::Plays(m.t0, m.t1)),
    )
}

/// What the readout says. An enum rather than a string so `width` can ask whether there IS
/// one without building it — one door for "is there a status", one for "what does it read".
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Status {
    /// The instance's window, in scene seconds.
    Plays(f64, f64),
    /// The container does not play at the current second — which is why the ruler draws no
    /// playhead.
    NotPlaying,
}

impl Status {
    fn text(self) -> String {
        match self {
            Self::Plays(a, b) => format!(
                "{} {a:.2} - {b:.2}",
                ph2d_i18n::tr("panel.timeline.host_window")
            ),
            Self::NotPlaying => ph2d_i18n::tr("panel.timeline.host_not_playing").to_owned(),
        }
    }
}

/// How wide the trail paints — the single source the transport flow measures against.
pub(crate) fn width(snap: &TimelineViewSnapshot) -> f32 {
    let n = trail(snap).len();
    if n == 0 {
        return 0.0;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "n is at most the id array's length"
    )]
    let n = n as f32;
    let gap = Spacing::Sm.px() * 0.5;
    n * SEG_W
        + (n - 1.0) * gap
        + if status(snap).is_some() {
            gap + STATUS_W
        } else {
            0.0
        }
}

/// Paint the trail: `[ Scene ][ C1 ][ C2 ]`, each segment a button that pops out to it.
///
/// The LAST segment is where you already are, so clicking it is a no-op — it is painted as
/// part of the trail rather than special-cased, because a trail whose current position is
/// missing reads as "you are nowhere".
pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32, snap: &TimelineViewSnapshot) {
    let entries = trail(snap);
    if entries.is_empty() {
        return;
    }
    let gap = Spacing::Sm.px() * 0.5;
    let mut x = x;
    for (slot, (_, label)) in entries.iter().enumerate() {
        let id = ids::TIMELINE_CRUMB[slot];
        let rect = Rect::new(x, y, SEG_W, ROW_H_PX);
        let st = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        paint_button(
            &Button::new(id, label.clone()).state(st),
            rect,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        ctx.host.hit_index_mut().register(id, rect);
        x += SEG_W + gap;
    }
    // The status is a READOUT, not a control: no id, no hit. It states a fact about where
    // the interior plays; there is nothing to click, and registering it would make a hit
    // that swallows a press and does nothing.
    if let Some(st) = status(snap) {
        let text = st.text();
        let font = TypeToken::Xs.px();
        ph2d_editor_core::text_elide::paint_text_elided(
            ctx.text_system,
            ctx.scene,
            &text,
            x,
            y + (ROW_H_PX - font) * 0.5,
            font,
            STATUS_W,
            resolve(ColorToken::Text3, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(crumbs: bool, map: Option<ph2d_timeline::ContainerMap>) -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            crumbs: if crumbs {
                vec![(0, "Walk".into())]
            } else {
                Vec::new()
            },
            host_map: map,
            ..TimelineViewSnapshot::default()
        }
    }

    fn map() -> ph2d_timeline::ContainerMap {
        ph2d_timeline::ContainerMap {
            t0: 4.0,
            t1: 12.0,
            u0: 0.0,
            u1: 8.0,
        }
    }

    /// **At the root the trail costs nothing at all** — no segments, no readout, no width.
    /// A document that never touches containers must not pay a pixel for the feature.
    #[test]
    fn the_root_pays_nothing() {
        let s = snap(false, None);
        assert_eq!(status(&s), None);
        assert!(width(&s).abs() < f32::EPSILON);
    }

    /// **Inside a container the readout says where the interior plays, in SCENE seconds.**
    ///
    /// This is the number that makes the two clocks on screen legible: the transport chip
    /// reads the scene, the ruler reads the interior, and the difference between them is
    /// exactly `t0`.
    #[test]
    fn a_playing_container_reads_out_its_window_in_scene_seconds() {
        let s = snap(true, Some(map()));
        assert_eq!(status(&s), Some(Status::Plays(4.0, 12.0)));
        let text = status(&s).unwrap().text();
        assert!(
            text.contains("4.00") && text.contains("12.00"),
            "the readout has to carry both ends, got {text:?}"
        );
    }

    /// **When it is not playing here, the readout SAYS so.**
    ///
    /// The ruler already refuses to draw a playhead in this case, and an absent marker with
    /// no explanation reads as broken. Naming the refusal is this module's idiom.
    #[test]
    fn a_container_that_is_not_playing_says_so_instead_of_going_quiet() {
        let s = snap(true, None);
        assert_eq!(status(&s), Some(Status::NotPlaying));
        assert_ne!(
            status(&s).unwrap().text(),
            Status::Plays(4.0, 12.0).text(),
            "the two states must not read the same"
        );
    }

    /// **A trilha mais funda que a régua de ids elide POR FORA** — a raiz e os níveis de
    /// dentro sobrevivem.
    ///
    /// Uma trilha que mostrasse todos os níveis MENOS onde você está é pior que uma curta: o
    /// segmento que importa é o último. O teto é a régua de ids (o chrome não cunha id em
    /// runtime), nunca a profundidade do aninhamento.
    #[test]
    fn a_deeper_trail_elides_from_the_outside_never_from_where_you_are() {
        let cap = ids::TIMELINE_CRUMB.len();
        // Rasa: cada slot é a sua própria profundidade, sem elisão.
        for len in 1..cap {
            for slot in 0..=len {
                assert_eq!(
                    depth_of_slot(slot, len),
                    Some(slot),
                    "len={len} slot={slot}"
                );
            }
            assert_eq!(depth_of_slot(len + 1, len), None, "não há slot além do fim");
        }
        // Funda: a raiz continua na 0, e o ÚLTIMO slot é o nível mais interno.
        let len = cap + 5;
        assert_eq!(depth_of_slot(0, len), Some(0), "a raiz nunca some");
        assert_eq!(
            depth_of_slot(cap - 1, len),
            Some(len),
            "o último slot tem de ser onde você ESTÁ, não um ancestral"
        );
        assert_eq!(depth_of_slot(cap, len), None, "a régua de ids é o teto");
        // E os níveis de fora são os que caem.
        assert!(
            depth_of_slot(1, len).unwrap() > 1,
            "com a trilha estourada o segundo segmento já pula os de fora"
        );
    }

    /// **A raiz não tem trilha nenhuma** — nem slot 0.
    #[test]
    fn the_scene_root_has_no_segments_at_all() {
        assert_eq!(depth_of_slot(0, 0), None);
        assert!(trail(&snap(false, None)).is_empty());
    }

    /// **O que a trilha PINTA é o que ela POPA.**
    ///
    /// Rótulo e profundidade saem da mesma lista, então um segmento não pode dizer "A" e
    /// levar para B ([[feedback_two_doors_to_the_same_question_diverge]]).
    #[test]
    fn every_segment_pops_to_the_depth_it_shows() {
        let mut s = snap(true, None);
        s.crumbs = vec![(7, "A".into()), (9, "B".into())];
        let t = trail(&s);
        assert_eq!(t.len(), 3, "raiz + dois níveis, veio {t:?}");
        assert_eq!(t[0].0, 0);
        assert_eq!((t[1].0, t[1].1.as_str()), (1, "A"));
        assert_eq!((t[2].0, t[2].1.as_str()), (2, "B"));
        for (slot, (d, _)) in t.iter().enumerate() {
            assert_eq!(
                depth_of_slot(slot, s.crumbs.len()),
                Some(*d),
                "o slot {slot} pinta a profundidade {d} e popa para outra"
            );
        }
    }

    /// **The flow layout reserves the readout's room.**
    ///
    /// The trail rides a flow layout that wraps; if `width` did not count the readout, the
    /// next item would be painted on top of it. The gate is a comparison, not a literal, so
    /// it survives a change to either constant.
    #[test]
    fn the_width_makes_room_for_the_readout() {
        let with = width(&snap(true, Some(map())));
        let bare = {
            let mut s = snap(true, Some(map()));
            s.crumbs.clear();
            // One segment's worth of trail, without a status: the root case has no trail at
            // all, so the honest comparison is the same trail minus the readout.
            width(&s) + 2.0 * SEG_W + Spacing::Sm.px() * 0.5
        };
        assert!(
            with > bare,
            "the readout must widen the trail: {with} vs {bare}"
        );
    }
}
