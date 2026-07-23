//! The container's **host-window STATUS readout** — and the [`trail`]/[`depth_of_slot`]
//! machinery the tab strip reuses to render its container tabs ([ADR-0133] §5).
//!
//! # The trail became TABS (Enio, 2026-07-23)
//!
//! The clickable breadcrumb segments — a `Scene` button that just fell into Arrange and a
//! `Container` button beside it — floated OUTSIDE the tab group, one of them redundant with
//! a tab already there. They moved INTO the strip ([`crate::transport_tabs`]): entering a
//! container adds its tab between Containers and Arrange, born checked; leaving is any of
//! the three fixed tabs. This module kept the two things that are NOT tabs: the navigation
//! rule ([`depth_of_slot`]/[`trail`], which the strip walks to build the container cells) and
//! the **status readout**.
//!
//! The original research still stands and is why the trail exists at all: **edit-in-place
//! with a breadcrumb** is the 2D lineage (Flash/Animate, Harmony) and **a new tab** is the
//! compositing lineage (After Effects), whose users lose the parent's context and rebuild it
//! by hand. The pivot keeps the edit-in-place *model* (you are still inside the container,
//! not in a separate document) and only moves the *control* into the group the animator
//! already reaches for.
//!
//! # The readout, and why not a second ruler
//!
//! Inside a container two clocks are on screen at once: the transport chip keeps showing the
//! scene's second, the ruler counts the interior's. Seeing `8.00` in the chip and the
//! playhead at `4` on the ruler is a contradiction until you know where the instance starts.
//! AE answers with a second stacked ruler; we do not need it — what is missing is the
//! RELATION between the two, which is [`status`], one line of text. A no-op at the root and
//! on the Keys tab, where that ruler is not on screen.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ColorToken, ROW_H_PX, Theme, TypeToken};

use crate::ids;

/// Width reserved for the status readout that follows the tabs. Fixed rather than
/// measured: it rides a flow layout that wraps, and a readout that grew with its own
/// number would reflow the whole bar every time the playhead crossed into or out of
/// the instance.
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
    if snap.crumbs.is_empty() || snap.keys_mode {
        // The readout states a fact about the ARRANGE ruler (where the instance plays on
        // it). On the Keys tab that ruler is not on screen and the host readouts publish
        // `None` — printing "not playing here" there was a wrong statement standing next
        // to a clip that plays fine (Enio's screenshot, 2026-07-20). The TRAIL stays: it
        // is navigation, and clicking it is the way back to Arrange.
        return None;
    }
    Some(match snap.host_map {
        // A container instanced NOWHERE still has a map — the identity over its own extent,
        // which is what makes its interior authorable — but printing that as a scene window
        // would label the container's own axis with the scene's name. Three states, because
        // there are three facts: it plays HERE, it plays SOMEWHERE ELSE, it plays nowhere.
        Some(m) if snap.host_placed => Status::Plays(m.t0, m.t1),
        Some(_) => Status::NotPlaced,
        None => Status::NotPlaying,
    })
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
    /// The container is instanced NOWHERE. Its interior is still authorable (the ruler counts
    /// its own seconds), but there is no scene window to name.
    NotPlaced,
}

impl Status {
    fn text(self) -> String {
        match self {
            Self::Plays(a, b) => format!(
                "{} {a:.2} - {b:.2}",
                ph2d_i18n::tr("panel.timeline.host_window")
            ),
            Self::NotPlaying => ph2d_i18n::tr("panel.timeline.host_not_playing").to_owned(),
            Self::NotPlaced => ph2d_i18n::tr("panel.timeline.host_not_placed").to_owned(),
        }
    }
}

/// How wide the STATUS readout paints — the single source the transport flow measures
/// against. Zero when there is no status (the root, or the Keys tab). The trail's own
/// segments moved into the tab strip (`transport_tabs`); this module keeps only the
/// readout that RELATES the two clocks on screen.
pub(crate) fn width(snap: &TimelineViewSnapshot) -> f32 {
    if status(snap).is_some() {
        STATUS_W
    } else {
        0.0
    }
}

/// Paint the host-window STATUS readout (`plays 4.00 - 12.00` / `not playing here` /
/// `not placed`) — a READOUT, not a control: no id, no hit. The trail's clickable
/// segments are now tabs in the strip (`transport_tabs`); what stays here is the one
/// line that says WHERE the open container plays on the scene ruler, which no tab can
/// carry. A no-op at the root and on the Keys tab (`status` returns `None`).
pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32, snap: &TimelineViewSnapshot) {
    let Some(st) = status(snap) else {
        return;
    };
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
            // A map that came from an INSTANCE — the ordinary case. The unplaced one has its
            // own gate below, and it is a different sentence.
            host_placed: map.is_some(),
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

    /// **On the Keys tab the readout is SILENT** — it states a fact about the Arrange
    /// ruler, which is not on screen there; "not playing here" beside a clip that plays
    /// fine was a wrong statement (Enio's screenshot, 2026-07-20). The trail still paints.
    #[test]
    fn the_status_is_silent_on_the_keys_tab() {
        let mut s = snap(true, None);
        s.keys_mode = true;
        assert_eq!(status(&s), None);
        s.keys_mode = false;
        assert_eq!(
            status(&s),
            Some(Status::NotPlaying),
            "de volta ao Arrange, ele fala"
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

    /// **"Não está colocado" e "não toca aqui" são frases DIFERENTES, e as três diferem.**
    ///
    /// Um container aberto pela aba Containers pode não ter instância nenhuma: ele ganha o
    /// mapa IDENTIDADE sobre a própria extensão (é o que torna o interior autorável), e ler
    /// isso como `Plays(0, 3)` rotularia o eixo do PRÓPRIO container com o nome da cena — um
    /// número errado apresentado como certo.
    #[test]
    fn a_container_that_is_placed_nowhere_says_that_and_not_a_scene_window() {
        let mut s = snap(true, Some(map()));
        s.host_placed = false;
        assert_eq!(status(&s), Some(Status::NotPlaced));
        let (placed, unplaced, absent) = (
            Status::Plays(4.0, 12.0).text(),
            Status::NotPlaced.text(),
            Status::NotPlaying.text(),
        );
        assert_ne!(
            unplaced, placed,
            "colocado e não-colocado não podem coincidir"
        );
        assert_ne!(
            unplaced, absent,
            "'não colocado' (não existe instância) ≠ 'não toca aqui' (existe, noutro segundo)"
        );
        assert!(
            !unplaced.contains("4.00") && !unplaced.contains("0.00"),
            "sem instância não há janela de cena a imprimir, veio {unplaced:?}"
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

    /// **The status readout reserves its own room, and nothing else does.**
    ///
    /// The trail's clickable segments moved into the tab strip; this module now measures
    /// ONLY the readout. A view WITH a status is wider than one without (the Keys tab,
    /// which prints nothing), and the trail's depth no longer changes the width — the
    /// segments are not here anymore.
    #[test]
    fn the_width_is_the_status_and_nothing_but() {
        let inside = snap(true, Some(map())); // Arrange inside a container: a status
        assert!(
            width(&inside) > 0.0,
            "a placed container reads out its window"
        );

        let mut keys = snap(true, Some(map()));
        keys.keys_mode = true; // the Keys tab: no readout
        assert!(
            width(&keys).abs() < f32::EPSILON,
            "the Keys tab prints nothing"
        );

        // A DEEPER trail does not widen this module anymore — the extra segments are tabs.
        let mut deeper = snap(true, Some(map()));
        deeper.crumbs = vec![(0, "A".into()), (1, "B".into()), (2, "C".into())];
        assert!(
            (width(&deeper) - width(&inside)).abs() < f32::EPSILON,
            "trail depth is the tab strip's business now, not the readout's"
        );
    }
}
