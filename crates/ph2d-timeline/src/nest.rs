//! Containers: a stack promoted to a reusable thing, and the cycle guard that keeps the
//! promotion honest ([ADR-0133]).
//!
//! # Two layers, because the tree is editable by more than one door
//!
//! The cycle check is asked **when the link is authored** ([`TimelineDoc::try_nest`]) *and*
//! **again when a document is loaded** ([`TimelineDoc::find_nest_cycle`]). That is not belt
//! and braces — it is the shape every editor that survived this bug converged on, and Godot
//! writes the reason into its own source: the first check only sees links that came through
//! the "add" path, and a tree can become cyclic by paths that do not (an undo, a paste, a
//! hand-edited or corrupted file).
//!
//! ⚠️ **The load REJECTS; it does not repair.** Blender's `BKE_collection_cycles_fix()` runs
//! at load and silently zeroes the offending reference — it destroys the artist's link to
//! save the file. A cyclic document here is a bug of ours, and the honest response is to say
//! so, not to mutilate the document on the way in.
//!
//! Each layer carries its own gate: neutralising one must not stay green because the other
//! caught it ([[feedback_layered_defenses_need_per_layer_gates]]).
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use crate::doc::TimelineDoc;
use crate::refusal::NestRefusal;
use crate::stack::{ClipLane, ClipStrip, StripId, StripSource};
use serde::{Deserialize, Serialize};

/// **A container: a stack promoted to a reusable thing** (ADR-0133 §3).
///
/// Its interior is a [`ClipLane`] stack — the *same* type the document's own stack is, on
/// purpose. "What is inside a container" and "what is on the timeline" are one question with
/// one answer; a second structure here would be a second answer, and the two would drift.
///
/// It lives on the DOCUMENT, beside the clips, rather than being an ECS entity: the ECS
/// hierarchy is already the *organisational* half of containment (the `GroupedChildren`
/// marker — Harmony's *group*, which expands in place under one clock), and this is the
/// *temporal* half (Harmony's *symbol*, which you enter and which has a clock of its own).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedContainer {
    /// Author-visible container name.
    pub name: String,
    /// The interior: lanes of strips, bottom to top — a stack like any other.
    pub stack: Vec<ClipLane>,
}

/// Which stack an edit is addressing: the document's own, or a container's interior.
///
/// The document's stack is not itself referenceable — nothing can place *it* inside
/// something — so a strip added there can never close a cycle. Containers can.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum StackHost {
    /// The timeline's own stack.
    #[default]
    Document,
    /// The interior of the container at this index.
    Container(usize),
}

/// Most containers a document may hold.
///
/// **The cap that the old note said would arrive with the widget** — it has. A container is
/// now a ROW in the Containers tab's list, and that row carries a rename button whose id
/// comes from a fixed `NodeId` array (`TIMELINE_CONT_RENAME`), exactly as
/// [`crate::MAX_CLIPS`] comes from the clip selector's. The chrome cannot mint a hit id at
/// runtime, so a 17th container would be an asset the artist could see and never rename —
/// and the refusal lives HERE, with the data, rather than letting the UI silently drop one
/// it cannot address. Raising it means growing the id arrays in lockstep; a gate holds them
/// together.
pub const MAX_CONTAINERS: usize = 16;

/// **The seconds an EMPTY container is born with** (Enio, 2026-07-21: *"seu tamanho inicial
/// (quando o container é vazio) é de 2 segundos"*).
///
/// Read only through [`container_bar_seconds`] — never directly.
pub const EMPTY_CONTAINER_SECONDS: f64 = 2.0;

/// **How long a container is, given its interior's extent** — the ONE door.
///
/// Four things need this number and they must agree: the bar the Containers list draws, the
/// snapshot's `ContainerView::length`, the span the lane's `+` places, and the slice
/// [`TimelineDoc::add_strip_to`] windows. Content answers for itself; an empty container
/// answers [`EMPTY_CONTAINER_SECONDS`], because a zero-length asset is a bar the double-click
/// cannot land on and a strip whose window stays 0 FOREVER — fill the container later and the
/// already-placed instance would keep playing nothing, silently.
#[must_use]
pub fn container_bar_seconds(interior: f64) -> f64 {
    if interior > 0.0 {
        interior
    } else {
        EMPTY_CONTAINER_SECONDS
    }
}

impl TimelineDoc {
    /// Append an empty container and return its index. Refuses past [`MAX_CONTAINERS`],
    /// returning the last existing index (the shape [`TimelineDoc::add_clip`] already uses:
    /// a refusal must still name a container that exists, or the caller navigates into a
    /// hole).
    pub fn add_container(&mut self, name: String) -> usize {
        if self.containers().len() >= MAX_CONTAINERS {
            return self.containers().len().saturating_sub(1);
        }
        self.containers_mut().push(NamedContainer {
            name,
            stack: Vec::new(),
        });
        self.containers().len() - 1
    }

    /// Rename container `index` (out of range: no-op) — the sibling of
    /// [`TimelineDoc::rename_clip`], and one of the Containers list's verbs (the others are
    /// delete, and entering by double-click).
    pub fn rename_container(&mut self, index: usize, name: String) {
        if let Some(c) = self.containers_mut().get_mut(index) {
            c.name = name;
        }
    }

    /// Delete container `index` — the asset AND every instance of it, everywhere.
    ///
    /// Deleting an asset that is placed cannot leave the placements behind: a strip whose
    /// source is gone plays nothing and can never be re-pointed (a strip has no "choose
    /// source" edit), so the honest delete is the cascade — the After Effects rule (deleting
    /// a comp removes its layers), with the global undo as the safety net.
    ///
    /// And because [`StripSource::Container`] is an INDEX, every reference above the hole
    /// steps down one as the list closes over it — a surviving instance keeps playing the
    /// asset the artist placed, not the neighbour that slid into its slot.
    pub fn remove_container(&mut self, index: usize) -> bool {
        if index >= self.containers().len() {
            return false;
        }
        self.containers_mut().remove(index);
        let hole = u16::try_from(index).unwrap_or(u16::MAX);
        let fix = |lanes: &mut Vec<ClipLane>| {
            for lane in lanes {
                lane.strips
                    .retain(|s| s.source != StripSource::Container(hole));
                for s in &mut lane.strips {
                    if let StripSource::Container(j) = s.source
                        && j > hole
                    {
                        s.source = StripSource::Container(j - 1);
                    }
                }
            }
        };
        if let Some(lanes) = self.host_stack_mut(StackHost::Document) {
            fix(lanes);
        }
        for c in self.containers_mut() {
            fix(&mut c.stack);
        }
        true
    }

    /// A container's interior stack.
    #[must_use]
    pub fn container_stack(&self, ix: usize) -> Option<&[ClipLane]> {
        self.containers().get(ix).map(|c| &c.stack[..])
    }

    /// A container's interior stack, for editing.
    pub fn container_stack_mut(&mut self, ix: usize) -> Option<&mut Vec<ClipLane>> {
        self.containers_mut().get_mut(ix).map(|c| &mut c.stack)
    }

    /// The lanes of whichever stack `host` names.
    #[must_use]
    pub fn host_stack(&self, host: StackHost) -> Option<&[ClipLane]> {
        match host {
            StackHost::Document => Some(self.stack()),
            StackHost::Container(ix) => self.container_stack(ix),
        }
    }

    /// The lanes of whichever stack `host` names, for editing.
    pub fn host_stack_mut(&mut self, host: StackHost) -> Option<&mut Vec<ClipLane>> {
        match host {
            StackHost::Document => Some(self.stack_mut()),
            StackHost::Container(ix) => self.container_stack_mut(ix),
        }
    }

    /// Append an empty lane to `host`'s stack and return its index, or `None` past
    /// [`crate::MAX_LANES`] / for a host that does not exist.
    pub fn add_lane_in(&mut self, host: StackHost, name: String) -> Option<usize> {
        let lanes = self.host_stack_mut(host)?;
        if lanes.len() >= crate::MAX_LANES {
            return None;
        }
        lanes.push(ClipLane::new(name));
        Some(lanes.len() - 1)
    }

    /// Remove a lane (and everything on it) from `host`'s stack.
    pub fn remove_lane_in(&mut self, host: StackHost, lane: usize) -> bool {
        let Some(lanes) = self.host_stack_mut(host) else {
            return false;
        };
        if lane >= lanes.len() {
            return false;
        }
        lanes.remove(lane);
        true
    }

    /// A strip of `host`'s stack, by identity.
    #[must_use]
    pub fn strip_in(&self, host: StackHost, lane: usize, id: StripId) -> Option<&ClipStrip> {
        let l = self.host_stack(host)?.get(lane)?;
        l.strips.get(l.index_of(id)?)
    }

    /// A strip of `host`'s stack, for editing.
    pub fn strip_in_mut(
        &mut self,
        host: StackHost,
        lane: usize,
        id: StripId,
    ) -> Option<&mut ClipStrip> {
        let l = self.host_stack_mut(host)?.get_mut(lane)?;
        let i = l.index_of(id)?;
        l.strips.get_mut(i)
    }

    /// Drop a strip from `host`'s stack.
    pub fn remove_strip_in(&mut self, host: StackHost, lane: usize, id: StripId) -> bool {
        let Some(l) = self.host_stack_mut(host).and_then(|s| s.get_mut(lane)) else {
            return false;
        };
        let Some(i) = l.index_of(id) else {
            return false;
        };
        l.strips.remove(i);
        true
    }

    /// **Move a strip to `t_start`, on `to_lane`** — the one door for a body slide, whichever
    /// axis it travelled on.
    ///
    /// The strip is TAKEN and RE-INSERTED, never rebuilt: a strip carries identity, fades,
    /// leads, trim marks, loop mode and speed, and "remove then add" would silently drop all
    /// of them (and mint a new [`StripId`], breaking the drag that is still holding the old
    /// one). Re-inserting also puts it back in `t_start` order, which is the lane's documented
    /// invariant and what `ClipLane::blend_in` reads to find a strip's neighbour — so a slide
    /// past a neighbour re-derives the crossfade against the right one.
    ///
    /// `false` when either lane, or the strip, is not there. A `to_lane` out of range is a
    /// REFUSAL, never a silent drop: the strip stays where it is.
    pub fn move_strip_in(
        &mut self,
        host: StackHost,
        lane: usize,
        to_lane: usize,
        id: StripId,
        t_start: f64,
    ) -> bool {
        let Some(lanes) = self.host_stack_mut(host) else {
            return false;
        };
        if lane >= lanes.len() || to_lane >= lanes.len() {
            return false;
        }
        let Some(i) = lanes[lane].index_of(id) else {
            return false;
        };
        let mut strip = lanes[lane].strips.remove(i);
        let span = strip.span();
        strip.t_start = t_start;
        strip.t_end = t_start + span; // rigid: the span rides along
        lanes[to_lane].insert(strip);
        true
    }

    /// Copy a strip of `host`'s stack and lay the copy immediately after the original.
    ///
    /// Same rule as the document-level [`TimelineDoc::duplicate_strip`] — adjacent, never
    /// overlapping, because a duplicate laid on top of its source is a crossfade of a clip
    /// with itself: a null edit dressed as a blend.
    pub fn duplicate_strip_in(
        &mut self,
        host: StackHost,
        lane: usize,
        id: StripId,
    ) -> Option<StripId> {
        let mut copy = self.strip_in(host, lane, id)?.clone();
        let new_id = self.alloc_strip_id();
        let span = copy.span();
        copy.id = new_id;
        copy.t_start += span;
        copy.t_end += span;
        self.host_stack_mut(host)?.get_mut(lane)?.insert(copy);
        Some(new_id)
    }

    /// A name no container is using yet ("Container 1", "Container 2", …).
    #[must_use]
    pub fn fresh_container_name(&self) -> String {
        let mut n = self.containers().len() + 1;
        loop {
            let name = format!("Container {n}");
            if !self.containers().iter().any(|c| c.name == name) {
                return name;
            }
            n += 1;
        }
    }

    /// **May `source` be placed inside `host`?** — the authoring-time cycle guard.
    ///
    /// Answers `Ok(())` for anything that is not a container-into-container link: a clip can
    /// never cycle, and the document's own stack is not referenceable.
    ///
    /// The walk is over the container graph only, so it is bounded by the number of
    /// containers, not by the number of strips.
    pub fn try_nest(&self, host: StackHost, source: StripSource) -> Result<(), NestRefusal> {
        let StackHost::Container(host_ix) = host else {
            return Ok(()); // the document's stack cannot be nested INTO anything
        };
        if host_ix >= self.containers().len() {
            return Err(NestRefusal::NoSuchContainer);
        }
        let Some(src) = source.container_index() else {
            return Ok(()); // a clip has no interior to loop back through
        };
        let src = src as usize;
        if src >= self.containers().len() {
            return Err(NestRefusal::NoSuchContainer);
        }
        if src == host_ix {
            return Err(NestRefusal::SelfNest);
        }
        // Does `src` already reach `host`? If so, putting `src` inside `host` closes the loop.
        if self.reaches(src, host_ix) {
            return Err(NestRefusal::WouldCycle);
        }
        Ok(())
    }

    /// Can `from` reach `target` by following container strips? DFS, visited-marked, so a
    /// document that is *already* cyclic cannot spin this forever.
    fn reaches(&self, from: usize, target: usize) -> bool {
        let n = self.containers().len();
        let mut seen = vec![false; n];
        let mut stack = vec![from];
        while let Some(c) = stack.pop() {
            if c == target {
                return true;
            }
            if c >= n || core::mem::replace(&mut seen[c], true) {
                continue;
            }
            for lane in &self.containers()[c].stack {
                for st in &lane.strips {
                    if let Some(next) = st.source.container_index() {
                        stack.push(next as usize);
                    }
                }
            }
        }
        false
    }

    /// **Is any container reachable from itself?** — the load-time check.
    ///
    /// Returns the index of a container that participates in a cycle, or `None`. Iterative
    /// and visited-marked: it is run against bytes that may be arbitrary, so it must
    /// terminate on input that is already broken.
    #[must_use]
    pub fn find_nest_cycle(&self) -> Option<usize> {
        (0..self.containers().len()).find(|&c| self.reaches_strictly(c, c))
    }

    /// [`Self::reaches`], but a container does not count as reaching itself for free — only
    /// by taking at least one edge.
    fn reaches_strictly(&self, from: usize, target: usize) -> bool {
        let n = self.containers().len();
        if from >= n {
            return false;
        }
        for lane in &self.containers()[from].stack {
            for st in &lane.strips {
                if let Some(next) = st.source.container_index() {
                    let next = next as usize;
                    if next == target || self.reaches(next, target) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Place `source` on `lane` of `host` over `[t_start, t_end)`.
    ///
    /// The cycle guard runs here, so **there is no way to author a cycle through this API** —
    /// which is the layer the load-time check exists to back up, not to replace.
    pub fn add_strip_to(
        &mut self,
        host: StackHost,
        lane: usize,
        source: StripSource,
        t_start: f64,
        t_end: f64,
    ) -> Result<StripId, NestRefusal> {
        self.try_nest(host, source)?;
        let src_len = self.source_length(source);
        let id = self.alloc_strip_id();
        let lanes = self
            .host_stack_mut(host)
            .ok_or(NestRefusal::NoSuchContainer)?;
        let l = lanes.get_mut(lane).ok_or(NestRefusal::NoSuchContainer)?;
        let mut strip = ClipStrip::new(source, t_start, t_end, src_len).with_id(id);
        // `slice == span * speed` must be true at BIRTH — see `add_strip`'s note: a strip
        // whose span its slice does not fill jumps to a wildly different rate on the first
        // hair of a stretch.
        let span = strip.span();
        if strip.slice() > 0.0 && span > 0.0 {
            strip.speed = strip.slice() / span;
        }
        l.insert(strip);
        Ok(id)
    }

    /// How long a source runs, in its own seconds.
    ///
    /// For a container this is [`container_bar_seconds`] over its interior's extent — content
    /// answers for itself, an EMPTY one answers [`EMPTY_CONTAINER_SECONDS`]. ⚠️ The note that
    /// stood here (*"an empty interior is zero-length … the caller's problem to see, not a
    /// reason to invent a length"*) was retired by Enio's rule (2026-07-21): an empty
    /// container IS 2 s long, everywhere — and the zero it defended made `src_out` freeze at
    /// 0 on a place-then-fill, so the instance played nothing forever.
    ///
    /// ⚠️ **It reads [`TimelineDoc::host_end_seconds`], the SAME door the panel sizes a new
    /// strip's span from.** It used to fold `t_end` while that one folds `lead_end`, and the
    /// two disagreed by exactly the last strip's outward lead-out — which is `slice != span`
    /// at birth, i.e. a container instance born at a speed nobody asked for (`add_strip`'s
    /// note, one level up). Two answers to "how long is this" is how a strip comes to be
    /// retimed by a fade somebody authored inside it a week earlier.
    fn source_length(&self, source: StripSource) -> f64 {
        match source {
            StripSource::Clip(i) => {
                let i = i as usize;
                self.clips().get(i).map_or(0.0, |c| {
                    c.clip.duration().to_seconds().max(self.clip_end_seconds(i))
                })
            }
            // A MISSING container stays 0.0 (the callers refuse it) — the door is for
            // containers that exist and are empty, not for indices that point at nothing.
            StripSource::Container(i) => self.containers().get(i as usize).map_or(0.0, |_| {
                container_bar_seconds(
                    self.host_end_seconds(StackHost::Container(i as usize))
                        .unwrap_or(0.0),
                )
            }),
        }
    }
}
