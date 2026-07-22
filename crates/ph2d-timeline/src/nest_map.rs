//! **Where an open container's interior sits on the timeline** — the instance's map, in
//! BOTH directions (ADR-0133 §5).
//!
//! # Why a map and not just a time
//!
//! While the ruler only had to DRAW the playhead, "what second is the interior at?" was
//! enough. But the ruler is also a control:
//! the animator drags it, and the drag has to arrive as a time on the **timeline**, because
//! the timeline's playhead is the only one there is (ADR-0133 §1 — the parent owns the
//! clock). Reading in one clock and writing in another is the bug this module exists to
//! stop: with the instance starting at 4 s, dragging to the interior's second 1 would seek the
//! scene to second 1, silently leaving the container by three seconds.
//!
//! So the answer has to be the *relation*, not a sample of it — and once it is the relation,
//! the second ruler (the timeline's seconds over the interior's axis) is the same object read
//! the other way, which is why there is one of these rather than two.
//!
//! # The map is the ENTRY's (Enio, 2026-07-20)
//!
//! The first cut derived the map from "the sole instance playing at the current playhead
//! time", and its refusals were honest but unusable: in every gap between instances the
//! container plays ZERO times, so the map — and the ruler's scrub with it — flickered away
//! while the artist stood inside; and a container instanced twice refused always. Both
//! ambiguities are manufactured: entering happens by right-clicking a STRIP, so the
//! instance the animator means is named by the walk itself. [`entry_map`] is that walk's
//! map — pure in the document and the path, valid at every playhead time.
//!
//! # When there is still no map
//!
//! - a level of the walk **wraps** ([`StripLoop::Loop`]/[`StripLoop::PingPong`] shorter
//!   than its span): one interior second recurs at several timeline seconds, and choosing
//!   one silently is precisely the class of guess this module refuses everywhere else;
//! - the walk is **stale** — its strip was deleted (or retargeted) under the open panel.
//!
//! There is no message for the refusal, and that is deliberate: the ruler simply does not
//! scrub, the same way a strip too narrow to hold both grips is offered only the trim one.
//! A control that is not offered does not owe a sentence — a control that lies does.

use crate::doc::TimelineDoc;
use crate::stack::{ClipStrip, StripLoop};

/// **One level of the animator's walk into a container** — which container was opened, and
/// through WHICH strip.
///
/// The strip is the half that was missing (Enio, 2026-07-20: *"dentro do conteiner não
/// consigo controlar/arrastar a playhead"*): entering happens by right-clicking a strip, so
/// the INSTANCE the animator means is never ambiguous — but a path that remembered only the
/// container had to rediscover the instance from "the sole strip playing NOW", which
/// flickers away in every gap and refuses outright when the container is instanced more
/// than once at the same instant. Remembering the strip makes the map a pure function of
/// the walk, valid at every playhead time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnterStep {
    /// The container that was opened.
    pub container: usize,
    /// The lane (of the stack it was opened FROM) holding the entry strip. **Ignored when
    /// [`Self::strip`] is `None`** — there is no named instance to look up.
    pub lane: usize,
    /// The instance the animator entered through — or `None` when they did not enter through
    /// one at all (the **Containers tab** opens a container by NAME, not by walking into a
    /// strip).
    ///
    /// `None` is not "no clock": it means *nobody named an instance*, so the walk falls back
    /// to the FIRST instance of this container in the parent host — a pure function of the
    /// document, stable at every playhead time (unlike "the instance playing NOW", which this
    /// module's history is about). Only a container that is instanced **nowhere** has no
    /// relation to its parent, and that case is the identity map plus `placed: false`.
    pub strip: Option<crate::StripId>,
}

/// The affine relation between an open container's own seconds and the timeline's, over the
/// window where the instance actually plays.
///
/// `[u0, u1]` is the interior's window in its own clock; `[t0, t1]` is the same window in
/// timeline seconds. Both mappings **clamp** to it: outside the window the interior is not
/// on screen at all, so there is no honest answer to extrapolate to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContainerMap {
    /// Timeline seconds where the instance's playing window starts.
    pub t0: f64,
    /// Timeline seconds where it ends.
    pub t1: f64,
    /// The interior's own second at `t0`.
    pub u0: f64,
    /// The interior's own second at `t1`.
    pub u1: f64,
}

impl ContainerMap {
    /// The timeline second at which the interior reads `u` — what a scrub of the interior's
    /// ruler must seek to.
    #[must_use]
    pub fn host_time(&self, u: f64) -> f64 {
        remap(u, self.u0, self.u1, self.t0, self.t1)
    }

    /// The interior's own second at timeline second `t` — the inverse, and what labels the
    /// second ruler.
    #[must_use]
    pub fn local_time(&self, t: f64) -> f64 {
        remap(t, self.t0, self.t1, self.u0, self.u1)
    }
}

/// Linear remap of `x` from `[a0, a1]` onto `[b0, b1]`, clamped to the source window.
fn remap(x: f64, a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    let d = a1 - a0;
    if d <= 0.0 {
        return b0; // a degenerate window is a single instant, not an axis
    }
    let f = ((x - a0) / d).clamp(0.0, 1.0); // CLAMP-OK: fraction of a measured window
    b0 + f * (b1 - b0)
}

/// The playing window of one strip, from its own fields: `(t0, t1)` in the clock of the
/// stack it LIVES in, `(u0, u1)` in the clock of what it READS — `None` when it is not a
/// bijection (a wrap, a pose-length slice, a non-positive speed).
fn window_of(s: &ClipStrip) -> Option<(f64, f64, f64, f64)> {
    let (span, slice) = (s.span(), s.slice());
    if span <= 0.0 || slice <= 0.0 || s.speed <= 0.0 {
        // A zero-length slice is a POSE (`ClipStrip::fold` says so), and a pose has no axis
        // to scrub along. A non-positive speed is the same degeneracy from the other side.
        return None;
    }
    // Timeline seconds one full pass through the slice takes.
    let pass = slice / s.speed;
    match s.loop_mode {
        // `Once` CLAMPS past the slice, so the tail holds a still frame: the moving part is
        // the window, and the hold is honestly outside it.
        StripLoop::Once => {
            let t1 = s.t_start + pass.min(span);
            Some((
                s.t_start,
                t1,
                s.src_in,
                s.src_in + (t1 - s.t_start) * s.speed,
            ))
        }
        // A wrap that actually wraps has no inverse (module docs). One that does not reach
        // its own end never folds, so it is the `Once` case wearing another name.
        StripLoop::Loop | StripLoop::PingPong => {
            (span <= pass).then_some((s.t_start, s.t_end, s.src_in, s.src_in + span * s.speed))
        }
    }
}

/// The entry strip one [`EnterStep`] names, verified to actually PLAY the container it
/// claims — a stale path (the strip was deleted, or retargeted by an undo) must read as
/// "no map", never as some other strip's window.
///
/// With no strip named ([`EnterStep::strip`] `None`) the answer is the **first** instance of
/// the container in this host, in lane order: a container opened by name has no walk to read
/// an instance off, and document order is the one stable answer. `None` here then means the
/// container is instanced NOWHERE — a different fact from a stale walk, and the caller
/// separates them.
fn entry_strip<'d>(
    doc: &'d TimelineDoc,
    host: crate::StackHost,
    step: &EnterStep,
) -> Option<&'d ClipStrip> {
    let lanes = doc.host_stack(host)?;
    let plays = |s: &ClipStrip| s.source.container_index().map(usize::from) == Some(step.container);
    match step.strip {
        Some(id) => {
            let s = lanes.get(step.lane)?.strips.iter().find(|s| s.id == id)?;
            plays(s).then_some(s)
        }
        None => lanes.iter().flat_map(|l| &l.strips).find(|s| plays(s)),
    }
}

/// **The map of the walked-into instance**, derived from the ENTRY PATH instead of from
/// "the sole strip playing now" (which this function replaced — module docs).
///
/// Pure in the document and the path: no scratch, no priming, and valid at every playhead
/// time. This is what unfroze the ruler (Enio, 2026-07-20) — the scratch-derived map
/// vanished in every gap between instances (the container plays ZERO times there) and the
/// marker/scrub flickered with it, even though the animator had named the instance they
/// meant by entering through it. Where the walk is stale or a level wraps (loop/ping-pong
/// inside its span), there is still no honest map, and `None` still refuses.
#[must_use]
pub fn entry_map(doc: &TimelineDoc, path: &[EnterStep]) -> Option<ContainerMap> {
    entry_clock(doc, path).map(|c| c.map)
}

/// **The open container's clock, and whether it came from an instance** — the ONE walk, two
/// facts.
///
/// [`entry_map`] delegates here rather than walking a second time: the readout ("plays 4.00 -
/// 12.00" versus "not placed") and the ruler's relation are statements about the same walk,
/// and two walks would be two answers to drift apart.
///
/// # The unplaced container is the identity, not a refusal
///
/// A container reached by NAME (the Containers tab) may be instanced nowhere. It still has an
/// interior to author, so the honest relation is the identity over its own extent: the ruler
/// counts the container's own seconds and scrubbing writes them straight through. Nothing is
/// invented — [`HostClock::placed`] says the relation is to nothing, and the readout prints
/// that instead of a scene window it does not have.
///
/// An unplaced level RESETS the frame of reference: everything deeper composes from the
/// identity, because "where does this play in the scene" has no answer above it.
#[must_use]
pub fn entry_clock(doc: &TimelineDoc, path: &[EnterStep]) -> Option<HostClock> {
    let mut host = crate::StackHost::Document;
    let mut acc: Option<(f64, f64, f64, f64)> = None;
    let mut placed = true;
    for step in path {
        let win = match entry_strip(doc, host, step) {
            Some(s) => Some(window_of(s)?),
            // A walk that NAMED a strip and cannot find it is STALE — refuse, as ever. One
            // that named none simply has no instance: fall back to the interior's own axis.
            None if step.strip.is_some() => return None,
            None => None,
        };
        acc = match win {
            Some((t0, t1, u0, u1)) => Some(match acc {
                None => (t0, t1, u0, u1),
                // The child's t-axis is the clock of `host` — the parent's u-axis. Remap the
                // child's window through the parent's: the same composition `container_map`
                // walks inward-out, here outward-in because the path is outermost-first.
                Some((pt0, pt1, pu0, pu1)) => (
                    remap(t0, pu0, pu1, pt0, pt1),
                    remap(t1, pu0, pu1, pt0, pt1),
                    u0,
                    u1,
                ),
            }),
            None => {
                placed = false;
                let e = interior_extent(doc, step.container);
                Some((0.0, e, 0.0, e))
            }
        };
        host = crate::StackHost::Container(step.container);
    }
    let (t0, t1, u0, u1) = acc?;
    (t1 > t0 && u1 > u0).then_some(HostClock {
        map: ContainerMap { t0, t1, u0, u1 },
        placed,
    })
}

/// The open container's clock plus the fact that makes it readable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostClock {
    /// The relation between the interior's seconds and the timeline's.
    pub map: ContainerMap,
    /// Whether [`Self::map`] came from an INSTANCE. `false` means the container is instanced
    /// nowhere and the map is the identity over its own extent — the interior is authorable,
    /// but "where does it play" has no answer, and the readout must say so rather than print
    /// a scene window that is really the container's own.
    pub placed: bool,
}

/// The axis an UNPLACED container's ruler counts along: its own content's end, through the
/// SAME door everything else measures a container with ([`crate::container_bar_seconds`]) —
/// an empty one counts 2 s, which is both "never degenerate" (it is where the first strip
/// goes) and the length its bar in the Containers list shows. Two numbers for one axis would
/// be the ruler disagreeing with the bar you double-clicked to get here.
fn interior_extent(doc: &TimelineDoc, container: usize) -> f64 {
    crate::container_bar_seconds(
        doc.host_end_seconds(crate::StackHost::Container(container))
            .unwrap_or(0.0),
    )
}

/// **The timeline window the walked-into instance OCCUPIES**, leads included — what a
/// transport loop must bracket so everything the instance contributes actually plays.
///
/// Differs from [`entry_map`]'s window on both ends: the map covers where the interior
/// MOVES (an axis to scrub), this covers where the instance is HEARD — the outward
/// `lead_in`/`lead_out` travel, and the held tail of a strip whose source finishes early.
/// Bracketing the map instead cut the deepest strip's fades out of the cycle, which is the
/// scene-level bug (`stack_end_seconds`) all over again, one level down.
///
/// Composed through the same parent windows as the map; a parent's remap CLAMPS, so a
/// lead that pokes out of a parent instance honestly stops at that parent's edge.
#[must_use]
pub fn entry_reach(doc: &TimelineDoc, path: &[EnterStep]) -> Option<(f64, f64)> {
    let (last, outer) = path.split_last()?;
    let mut host = crate::StackHost::Document;
    let mut acc: Option<(f64, f64, f64, f64)> = None;
    for step in outer {
        let s = entry_strip(doc, host, step)?;
        let (t0, t1, u0, u1) = window_of(s)?;
        acc = Some(match acc {
            None => (t0, t1, u0, u1),
            Some((pt0, pt1, pu0, pu1)) => (
                remap(t0, pu0, pu1, pt0, pt1),
                remap(t1, pu0, pu1, pt0, pt1),
                u0,
                u1,
            ),
        });
        host = crate::StackHost::Container(step.container);
    }
    let s = entry_strip(doc, host, last)?;
    let (lo, hi) = (s.lead_start(), s.lead_end());
    if hi <= lo {
        return None;
    }
    Some(match acc {
        None => (lo, hi),
        Some((pt0, pt1, pu0, pu1)) => {
            (remap(lo, pu0, pu1, pt0, pt1), remap(hi, pu0, pu1, pt0, pt1))
        }
    })
}
