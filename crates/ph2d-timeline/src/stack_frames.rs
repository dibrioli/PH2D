//! **Resolving** the clip stack: which strips are live right now, in which stack, and at
//! what time — the half that runs BEFORE any value is sampled ([`crate::stack_eval`] is the
//! other half, which blends them).
//!
//! The two were one file until nesting arrived and it went past the LOC cap. The seam is not
//! arbitrary: *"what is playing"* and *"what value does it produce"* are different questions,
//! and only the first one had to learn about containers (ADR-0133).

use crate::clock::ClockIndex;
use crate::doc::TimelineDoc;
use crate::stack::LaneMode;

/// One strip that is live at the current time, with everything the per-binding
/// loop needs — resolved **once per frame**, not once per binding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ActiveStrip {
    /// **Which stack this strip lives in** — an index into [`StackScratch::frames`].
    /// Frame 0 is always the document's own stack; every nested container instance
    /// gets a frame of its own (ADR-0133).
    ///
    /// It is what keeps a container's interior from blending against its parent's
    /// siblings: a lane only ever gathers strips of its OWN frame, so the interior
    /// normalizes among itself and enters the parent as one value — the same "sorts
    /// as one block" rule the z-order answers with `SortingGroup`.
    pub frame: usize,
    /// Which lane it belongs to (its index in its frame's stack).
    pub lane: usize,
    /// What it plays.
    pub source: ActiveSource,
    /// Its blend weight at this instant (the crossfade ramp).
    pub w: f64,
    /// The clip time it is reading.
    pub t_clip: f64,
    /// The first frame of its slice — the reference an additive lane measures
    /// its delta against.
    pub src_in: f64,
    /// This entry is a strip that has **ENDED** and is holding its last frame
    /// ([`crate::ClipLane::hold_at`]) — not one that is playing.
    ///
    /// It contributes to the POSE (that is its whole job: a fade-in against nothing
    /// crossfades from it), but it is not *playing*, so everything that asks "where
    /// is this clip right now" must skip it — `sole_strip_of` above all, or `K` would
    /// key into a strip that is over.
    pub held: bool,
}

/// What a live strip plays, once resolved (ADR-0133).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ActiveSource {
    /// A clip, by index into the document's clips.
    Clip(usize),
    /// A nested container instance.
    Container {
        /// The frame its interior was resolved into. **Not** the container's index: two
        /// strips of the same container are two instances, each reading its own time.
        frame: usize,
        /// The frame holding the same interior resolved at the strip's FIRST frame
        /// (`src_in`) — the reference an **additive** lane measures its delta against.
        ///
        /// `None` on an Override lane, where nothing asks. It is resolved during the
        /// rebuild rather than on demand because the answer is "the interior at another
        /// time", and a scratch only ever describes ONE time: asking later would mean
        /// re-resolving the whole interior mid-sample.
        ///
        /// The rule is deliberately the SAME one a clip strip follows ("its own value at
        /// `src_in`"). A cheaper reference — the rest pose, say — would have been a second
        /// answer to one question, and the two would drift the day someone nested an
        /// additive lane inside an additive lane.
        reference: Option<usize>,
    },
}

impl ActiveSource {
    /// The clip index, or `None` for a container instance.
    pub(crate) fn clip(self) -> Option<usize> {
        match self {
            Self::Clip(c) => Some(c),
            Self::Container { .. } => None,
        }
    }
}

/// One resolved stack: whose it is, and what time it is reading.
///
/// **The time is per FRAME, not per document, and that is the whole nesting mechanism.**
/// Frame 0 reads the playhead; a container instance reads whatever its strip's map hands
/// it (`timeline -> strip.source_time -> container time`), and its own strips map from
/// there. Composition is one link of the same kind, repeated — never a second clock
/// invented alongside the first (ADR-0133 §1).
#[derive(Debug, Clone, Copy)]
pub(crate) struct StackFrame {
    /// Which container this frame is an instance of; `None` = the document's own stack.
    pub(crate) container: Option<usize>,
    /// The time this frame's strips are read at, in this frame's own seconds.
    t: f64,
    /// This frame's slice of [`StackScratch::active`] — `active[first..first + count]`.
    ///
    /// The frames are built breadth-first and each one pushes ALL of its strips before the
    /// sweep moves on, so a frame's strips are contiguous. Recording the range turns the
    /// per-lane gather from "scan every live strip in the document and filter by frame" into
    /// "scan this frame's own", which is the difference between a nested document paying for
    /// its own depth and paying for everybody's.
    pub(crate) first: usize,
    pub(crate) count: usize,
}

/// Per-frame scratch: which strips are live, and each one's entity clocks.
///
/// The clocks are per **strip**, not per document, because two strips can play
/// two different clips, and an entity's Time Remap track lives *inside* a clip.
/// One index per live strip is the price of that, and it is a small price: live
/// strips are counted on one hand.
///
/// Zero-alloc in steady state (HR-3): both vectors are cleared and refilled, and
/// the clock indices are retained so their own buffers stay warm.
#[derive(Debug, Clone)]
pub(crate) struct StackScratch {
    pub(crate) active: Vec<ActiveStrip>,
    /// Parallel to `active`.
    pub(crate) clocks: Vec<ClockIndex>,
    /// One entry per resolved stack: index 0 is the document's own, and every nested
    /// container INSTANCE gets one of its own (ADR-0133).
    ///
    /// A flat arena rather than a tree of `Vec`s: the recursion is in how it is *read*,
    /// not in how it is stored, which keeps the whole scratch zero-alloc in steady state
    /// (HR-3) instead of allocating a fresh interior every frame.
    pub(crate) frames: Vec<StackFrame>,
    /// The stack this scratch is rooted in (`None` = the scene). See [`Self::root`].
    root: Option<usize>,
    /// The timeline time this was built at. `NaN` until the first rebuild — and
    /// `NaN != NaN`, so a fresh scratch is never mistaken for a valid one.
    ///
    /// It exists because the scratch is a CACHE, and every reader of it (the
    /// autokey's refusal, the key's landing time) implicitly asks "what is the
    /// stack doing at t?" while actually being answered "what was it doing at
    /// whatever t the apply last used". Those coincide in production — the apply
    /// runs first, same frame, same playhead — and that is precisely the shape of
    /// coupling that keeps breaking this module. Recording `t` lets a caller
    /// PRIME (`TimelineDoc::prime_stack`) and turns the invisible dependency into
    /// a checked one.
    t: f64,
}

/// Scratch is not identity — see [`ClockIndex`]'s own note.
impl PartialEq for StackScratch {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Default for StackScratch {
    fn default() -> Self {
        Self {
            active: Vec::new(),
            clocks: Vec::new(),
            frames: Vec::new(),
            t: f64::NAN, // never equal to a real time: the first prime always builds
            root: None,
        }
    }
}

impl StackScratch {
    /// The time this scratch describes (`NaN` if never built).
    pub(crate) fn built_at(&self) -> f64 {
        self.t
    }

    /// **Which stack this scratch is ROOTED in** — `None` for the document's own
    /// (the scene), `Some(c)` when it describes container `c`'s interior soloed
    /// (the Containers-tab editing view, Enio 2026-07-22). Every authoring gate
    /// that used to ask `doc.stack().is_empty()` asks THIS instead: rooted, the
    /// scene's stack is the wrong stack to interrogate.
    pub(crate) fn root(&self) -> Option<usize> {
        self.root
    }

    /// Resolve the live strips at `t` and every remapped entity's clock inside
    /// each one. Call once per frame, after the liveness pass.
    ///
    /// With an empty stack this leaves exactly one entry: the active clip, read
    /// at the playhead. That is not a special case bolted on — it is what "no
    /// stack" *means*, and it keeps the single-clip path on the same code.
    pub(crate) fn rebuild(&mut self, doc: &TimelineDoc, t: f64) {
        self.rebuild_rooted(doc, None, t);
    }

    /// [`Self::rebuild`], ROOTED: `root = Some(c)` resolves container `c`'s
    /// interior as if it were the scene — frame 0 is the container, and the
    /// breadth-first walk below recurses into whatever it nests, on the same
    /// code. This is what solos a container for editing (`apply_container`):
    /// one evaluator, two roots, so the interior you author and the interior an
    /// instance plays can never be two different answers.
    pub(crate) fn rebuild_rooted(&mut self, doc: &TimelineDoc, root: Option<usize>, t: f64) {
        self.t = t;
        self.root = root;
        self.active.clear();
        self.frames.clear();
        self.frames.push(StackFrame {
            container: root,
            t,
            first: 0,
            count: 0,
        });

        if root.is_none() && doc.stack().is_empty() {
            self.active.push(ActiveStrip {
                frame: 0,
                lane: 0,
                source: ActiveSource::Clip(doc.active_index()),
                w: 1.0,
                t_clip: t,
                src_in: 0.0,
                held: false,
            });
        } else {
            // **Breadth-first over FRAMES, not recursion.** `frames` grows while the loop
            // walks it: resolving a container strip appends the frame for its interior, and
            // the loop reaches it later. Depth therefore costs list entries, never call
            // stack — which is what lets a deeply nested document be a perf question rather
            // than a crash question.
            //
            // It terminates because the container graph is ACYCLIC, and that is not a hope:
            // `try_nest` refuses a cycle at the gesture and `from_bytes` refuses a document
            // that contains one (ADR-0133 §4). Both layers are gated.
            let mut fi = 0;
            while fi < self.frames.len() {
                let frame = self.frames[fi];
                let Some(lanes) = frame
                    .container
                    .map_or_else(|| Some(doc.stack()), |c| doc.container_stack(c))
                else {
                    fi += 1;
                    continue; // a container that was deleted under a live strip
                };
                let first = self.active.len();
                self.build_frame(doc, fi, lanes, frame.t);
                self.frames[fi].first = first;
                self.frames[fi].count = self.active.len() - first;
                fi += 1;
            }
        }

        // Grow (never shrink) the clock pool, then refill it in place.
        while self.clocks.len() < self.active.len() {
            self.clocks.push(ClockIndex::default());
        }
        for i in 0..self.active.len() {
            let a = self.active[i];
            // Only a CLIP has tracks, so only a clip strip has an entity clock to resolve.
            // A container instance's clocks live in the frames its interior produced.
            if let Some(c) = a.source.clip()
                && let Some(named) = doc.clips().get(c)
            {
                self.clocks[i].rebuild(doc, &named.clip, a.t_clip);
            }
        }
    }

    /// Resolve one frame's lanes into `active`, appending a frame for every container
    /// strip it finds.
    ///
    /// Split out of [`Self::rebuild`] because it is the body that used to BE the rebuild:
    /// keeping it one function would have made the nesting look like a rewrite of the
    /// stack evaluator, when it is the same evaluator run once per stack.
    fn build_frame(
        &mut self,
        doc: &TimelineDoc,
        frame: usize,
        lanes: &[crate::stack::ClipLane],
        t: f64,
    ) {
        for (li, lane) in lanes.iter().enumerate() {
            if lane.muted {
                continue; // muting REMOVES the lane; it is not a zero weight
            }
            let additive = lane.mode == LaneMode::Additive;
            for (si, strip) in lane.strips.iter().enumerate() {
                // **O que decide se um strip está ATIVO é COBRIR o tempo, não pesar mais que
                // zero.** O peso é DADO — a resposta da lane —, não um filtro.
                //
                // Descartar o peso zero aqui apagava a diferença entre "esta lane não keya o
                // canal" (silêncio) e "ela keya, e neste instante não influi" (o primeiro
                // instante de um fade-in). O avaliador lia as duas como silêncio, ninguém
                // escrevia, e o objeto segurava a pose do frame anterior: o mesmo `t` dava
                // poses diferentes conforme o lado de onde o playhead chegava.
                //
                // Custo: no máximo um punhado de strips de peso zero na lista, e só nas beiras
                // exatas de um fade (o clamp da soma garante que o peso só zera nos extremos).
                let w = lane.weight_at(si, t);
                // `source_time_with_lead`, not `source_time`: in the strip's outward
                // lead-in window (in the gap before it) this returns the FROZEN first
                // frame, so the strip contributes a still pose there — the travel the
                // lead-in is. `weight_at` above already ramps over the same window, so
                // the two agree on where the strip is present.
                let Some(t_local) = strip.source_time_with_lead(t) else {
                    continue; // outside the strip (+ its lead-in)
                };
                let Some(source) = self.resolve(doc, strip.source, t_local, additive, strip.src_in)
                else {
                    continue; // a deleted clip or a deleted container
                };
                self.active.push(ActiveStrip {
                    frame,
                    lane: li,
                    source,
                    w,
                    t_clip: t_local,
                    src_in: strip.src_in,
                    held: false,
                });
            }
            // Whatever coverage the live strips leave unaccounted for is HELD by
            // the last strip to end — the gap is not silence, and a fade-in
            // against nothing crosses from there rather than from the rest pose
            // (`ClipLane::hold_at` carries the whole argument).
            // The TIMELINE loop (`false` = the Arrange view's), because the stack
            // runs on the timeline's clock: under it the lane is CYCLIC and the
            // hold at the top wraps to what the loop's end leaves asserting.
            //
            // ⚠️ Only frame 0 has that loop. A container's interior has no loop of its
            // own yet, and borrowing the DOCUMENT's would be a loop from another clock
            // wrapping a stack it knows nothing about.
            let loop_range = (frame == 0).then(|| doc.active_loop_for(false)).flatten();
            if let Some((strip, t_local, w)) = lane.hold_at(t, loop_range)
                && let Some(source) =
                    self.resolve(doc, strip.source, t_local, additive, strip.src_in)
            {
                self.active.push(ActiveStrip {
                    frame,
                    lane: li,
                    source,
                    w,
                    t_clip: t_local,
                    src_in: strip.src_in,
                    held: true,
                });
            }
        }
    }

    /// Turn a strip's declared source into a resolved one, appending the interior frame
    /// when it is a container. `None` when the clip or container no longer exists.
    fn resolve(
        &mut self,
        doc: &TimelineDoc,
        source: crate::stack::StripSource,
        t_local: f64,
        additive: bool,
        src_in: f64,
    ) -> Option<ActiveSource> {
        match source {
            crate::stack::StripSource::Clip(c) => {
                let c = c as usize;
                (c < doc.clips().len()).then_some(ActiveSource::Clip(c))
            }
            crate::stack::StripSource::Container(c) => {
                let c = c as usize;
                if c >= doc.containers().len() {
                    return None;
                }
                // A frame PER INSTANCE, not per container: two strips of the same
                // container read two different times, and sharing a frame would make the
                // second silently adopt the first's clock.
                self.frames.push(StackFrame {
                    container: Some(c),
                    t: t_local,
                    first: 0,
                    count: 0,
                });
                let frame = self.frames.len() - 1;
                // The additive reference costs a SECOND resolution of the same interior,
                // at the strip's first frame — and it is paid only on an additive lane,
                // which is the only place anything asks for it.
                let reference = additive.then(|| {
                    self.frames.push(StackFrame {
                        container: Some(c),
                        t: src_in,
                        first: 0,
                        count: 0,
                    });
                    self.frames.len() - 1
                });
                Some(ActiveSource::Container { frame, reference })
            }
        }
    }

    /// The one live strip of the empty-stack path (the active clip at the
    /// playhead), and the entity clock resolved inside it.
    pub(crate) fn solo_clock(&self) -> &ClockIndex {
        &self.clocks[0]
    }
}
