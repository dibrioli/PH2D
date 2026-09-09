//! [`TimelineDoc`] — the app-general timeline **document**: the editable state
//! the panel drives and the project saves.
//!
//! It owns named [`Clip`]s, the [`TargetBinding`]s that say which scene object
//! each clip track drives, and [`Marker`]s. v1 edits a single clip ("Main");
//! multi-clip is data-ready (a `Vec`), UI deferred (W5). The document is the
//! authority; a [`crate::TimelineState`] wraps it with panel selection +
//! history.
//!
//! Targets are **allocated** here (opaque, HR-8) so two entities animating the
//! same [`PropKind`] get distinct tracks in one clip.

use std::collections::BTreeMap;

use ph2d_anim::{AnimTarget, AnimValue, Clip, Interp, KeyId, RationalTime, Track};
use serde::{Deserialize, Serialize};

use crate::binding::TargetBinding;
use crate::nest::NamedContainer;
use crate::prop::PropKind;
use crate::stack::ClipLane;
use crate::stack_frames::StackScratch;

// The marker methods live in a child module (it reaches the private `markers` field)
// to keep this file under the LOC cap — split by responsibility, not allowlist.
#[path = "doc_markers.rs"]
mod markers;

// As portas da TRAJETÓRIA por-clip vivem num módulo irmão pela mesma razão que os markers:
// elas alcançam o campo privado `clips`, e este arquivo está sob o teto de 700 LOC. Split
// por responsabilidade — *onde a trajetória mora e quem a resolve* é um assunto, e é o
// assunto que a wave de 2026-07-30 moveu do binding para o clip.
#[path = "doc_clip_paths.rs"]
mod clip_paths;

/// On-disk schema version for the timeline document (HR-14). Written explicitly
/// as the first field (never trust `serde(default)` under a positional format).
/// v2: tracks carry per-key roving flags (`TrackData.roving`, appended field —
/// postcard is positional, so a v1 blob is rejected rather than misread).
/// v3: each clip carries its own loop (`NamedClip.loop_range` + `loop_ping_pong`,
/// appended) — a loop belongs to the animation it brackets, not to the document.
/// v4: the clip **stack** (`TimelineDoc.stack`) and each binding's captured
/// `rest` value (ADR-0115). Both appended; a document with an empty stack behaves
/// byte-for-byte as it did in v3.
/// v5: each clip carries a SECOND loop — one per view
/// (`NamedClip.keys_loop_range`/`keys_loop_ping_pong`, appended). The Arrange tab
/// loops the timeline (the original `loop_range`); the Keys tab loops the clip's own
/// clock, independently (Enio, 2026-07-16). Appended; both `None`/`false` behaves
/// exactly as v4.
/// v6: a strip's fade-in can reach OUTWARD into the gap before it
/// (`ClipStrip.lead_in`, appended) — the travel fade (Enio, 2026-07-16). `0.0` is
/// the old behaviour byte-for-byte.
/// v7: each strip remembers **what its four corners last did** (`ClipStrip.marks`,
/// appended) — the change bars the panel draws over a trim or a stretch (Enio,
/// 2026-07-16). All-zero is the old behaviour, and zero draws nothing.
/// v8: **nesting** ([ADR-0133]) — `ClipStrip.clip: u16` became
/// `ClipStrip.source: StripSource{Clip,Container}`, and the document grew a list of
/// [`NamedContainer`]s beside its clips. ⚠️ Unlike every bump above, this one **replaces**
/// a field instead of appending one, so a v7 blob is not merely missing data — its bytes
/// mean something else from that field on. It is rejected, which is what
/// [`TimelineDoc::from_bytes`] has always done with a version it does not know.
/// v9: a strip's fade-out can reach OUTWARD into the gap after it
/// (`ClipStrip.lead_out`, appended) — the mirror of `lead_in` (Enio, 2026-07-19). `0.0` is
/// the old behaviour byte-for-byte.
/// v10: each container carries its OWN loop (`NamedContainer.loop_range` +
/// `loop_ping_pong`, appended) — the interior transport's cycle, independent of the
/// scene's and of every clip's (Enio, 2026-07-22: *"o loop deve ser independente em
/// cada modo"*). `None`/`false` behaves exactly as v9.
///
/// v11: the three **explicit durations** (`NamedClip.length_override`,
/// `NamedContainer.length_override`, `TimelineDoc.scene_length`, all appended) — the
/// AE composition-duration model (Enio, 2026-07-23): an authored end that go-to-end
/// and a fresh loop read, cutting content past it non-destructively. `None` behaves
/// exactly as v10.
///
/// v12: **the motion path** ([ADR-0141]) — `TargetBinding.path`, appended: the
/// trajectory a [`crate::PropKind::Position`] binding follows, whose track measures
/// distance along it. `None` behaves exactly as v11, and every binding a v11
/// document held is one of the kinds that never has a path.
///
/// v13: **timeline signals** ([ADR-0143]) — `Marker.signal: Option<String>`, appended:
/// a marker can carry a named signal that emits a decoupled event when the play crosses
/// it. `None` is a pure annotation — byte-for-byte what a v12 marker was.
///
/// v14: **per-track extrapolation** (crown-jewels plan §6) — `ph2d_anim::Track`'s
/// `pre`/`post` ([`ph2d_anim::Extrap`]), appended to its serde proxy: loopOut / cycle /
/// pingpong / continue beyond the keyed range. Default [`ph2d_anim::Extrap::Hold`] is
/// the flat-clamp — a v13 document re-reads as `Hold/Hold` and samples byte-for-byte as
/// before (the fade fingerprint pin). Postcard is positional, so the appended fields
/// still force the version bump: a v13 blob is refused on load.
///
/// v15: **property expressions** (ADR-0144) — `TargetBinding.expr: Option<String>`,
/// the formula that drives a property in a SEPARATE post-composition pass. `None`
/// (the default) is byte-identical to v14 (no pass runs), but postcard is positional
/// so the appended field forces the bump: a v14 blob is refused on load.
///
/// v16: **per-clip expressions** ([ADR-0151]) — `NamedClip.expr:
/// BTreeMap<AnimTarget, String>`, appended: the formula lives in the CLIP (like
/// keyframes) so a strip that plays the clip WINDOWS it. Empty (the default) is
/// byte-identical to v15, but postcard is positional so the appended field forces the
/// bump: a v15 blob is refused on load.
///
/// v17: **a trajetória é do CLIP** (Enio, 2026-07-30: *"cada clip novo deve ser um branco
/// e criar do zero seu próprio PATH"*) — `NamedClip.paths: BTreeMap<AnimTarget,
/// MotionPath>` apendado, e `TargetBinding.path` **REMOVIDO**. É a única mudança desta
/// escada que tira um campo em vez de só acrescentar: o v16 não fica meramente sem dados,
/// os bytes dele passam a significar outra coisa a partir dali. Recusado no load, como
/// todo bump deste documento desde o ADR-0133.
///
/// [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md
/// [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md
/// [ADR-0143]: ../../../docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md
/// [ADR-0144]: ../../../docs/architecture/decisions/0144-timeline-expressions-frozen-ir-separate-post-composition-pass.md
/// [ADR-0151]: ../../../docs/architecture/decisions/0151-timeline-expressions-are-per-clip-so-a-strip-windows-them.md
pub const DOC_VERSION: u32 = 18;

/// The default display frame rate for a fresh document.
pub const DEFAULT_FPS: f64 = 24.0;

/// The **authored** duration a freshly opened timeline gets (Enio, 2026-07-23:
/// *"a timeline abre com Duração zero. Por padrão coloque 4 segundos"*) — the AE
/// "new composition is N seconds long" default. Applied by
/// [`crate::TimelineState::with_default_duration`] (the shell's startup path), NOT by
/// [`TimelineDoc::new`], so the crate's "fresh document = derived end" invariant — and
/// every test that leans on it — is untouched.
pub const DEFAULT_DURATION_SECONDS: f64 = 4.0;

/// How many clips a document may hold.
///
/// A real bound, not a guess at what an animator needs: the clip selector is a
/// dropdown, and a dropdown's option ids are a FIXED array of `NodeId`s
/// (`TIMELINE_CLIP_OPT`) — the chrome has no way to mint a hit id at runtime. So
/// the cap is whatever that array is, and it lives HERE, with the data, where
/// [`TimelineDoc::add_clip`] can refuse rather than let the UI silently drop a
/// clip it cannot address. Raising it means growing the id array in lockstep, and
/// a gate holds the two together.
pub const MAX_CLIPS: usize = 16;

/// A named point in time on the timeline (UI in W4; data lives here from W1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    /// Marker position (drift-free rational time).
    pub t: RationalTime,
    /// Author-visible label (a raw string — markers are user content, not HR-15
    /// UI chrome).
    pub label: String,
    /// The signal this marker emits when the play crosses it, if any ([ADR-0143]).
    /// `None` is a pure annotation — the v12 behaviour, byte-for-byte. The name is
    /// the decoupled contract a consumer matches on (the timeline never calls the
    /// consumer — ADR-0075), and it is deliberately distinct from `label` (the human
    /// text): conflating the two is the After Effects trap. Appended field — postcard
    /// is positional, hence `DOC_VERSION` 12 -> 13.
    ///
    /// [ADR-0143]: ../../../docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md
    pub signal: Option<String>,
}

/// A clip with an author-visible name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedClip {
    /// Author-visible clip name.
    pub name: String,
    /// The animation data.
    pub clip: Clip,
    /// This clip's `[start, end)` loop range in seconds, if the animator set one.
    ///
    /// **Per clip, not per document** (Enio, 2026-07-12): a loop is a property of
    /// the animation it brackets — "walk" cycles over its own two seconds and
    /// "run" over its own — so one range shared across every clip was simply the
    /// wrong range for all but the one it was drawn on. The `Playhead` still owns
    /// the LIVE loop (it is what wraps the transport); this is where each clip
    /// parks its own, and switching clips swaps it in.
    ///
    /// Appended field — postcard is positional, hence `DOC_VERSION` 2 -> 3.
    pub loop_range: Option<(f64, f64)>,
    /// `true` when this clip's loop **ping-pongs** (plays back and forth) instead
    /// of wrapping. Rides with the range because it IS part of it: a loop is a
    /// span plus what happens at its end. Appended (v3).
    pub loop_ping_pong: bool,
    /// The **Keys view's** loop — this clip's own two seconds, in the clip's OWN
    /// clock, looped independently of the Arrange loop above (Enio, 2026-07-16).
    ///
    /// The two are different clocks with different braces: [`Self::loop_range`]
    /// wraps the TIMELINE playhead (the stack the Arrange tab shows), this one wraps
    /// the CLIP playhead the Keys tab scrubs while you author keys. Setting one never
    /// touches the other — the loop area is independent per clip AND per view. Both
    /// appended (`DOC_VERSION` 4 -> 5).
    pub keys_loop_range: Option<(f64, f64)>,
    /// Mirror of [`Self::loop_ping_pong`] for the Keys-view loop. Appended (v5).
    pub keys_loop_ping_pong: bool,
    /// The clip's **explicit duration** in seconds, when authored — the AE
    /// composition-duration model (Enio, 2026-07-23): it defines "the end" for
    /// go-to-end and a freshly armed loop, and a value SHORTER than the content
    /// **cuts** it non-destructively (keys past it stay authored, they just never
    /// play — the eval clamps its clock at the cut). `None` = derived from
    /// content, exactly as before. Appended (v11).
    pub length_override: Option<f64>,
    /// **A EXPRESSÃO por-clip** (ADR-0151) — a fórmula que dirige `target` DENTRO
    /// deste clip, keyada por `AnimTarget` como os tracks. Um strip que toca este
    /// clip a JANELA (avaliada no tempo LOCAL do strip); fora do strip ela fica
    /// quieta com os keys. É o modelo precomp do AE, e é o que faz uma expressão
    /// PURA (sem keyframes) obedecer o strip. O `binding.expr` (document-wide,
    /// ADR-0144) segue como o driver GLOBAL. Apêndice posicional ⇒ `DOC_VERSION`
    /// 15 -> 16.
    #[serde(default)]
    pub expr: BTreeMap<AnimTarget, String>,
    /// **A TRAJETÓRIA deste clip** (ADR-0141), por alvo — a geometria que uma track de
    /// [`crate::PropKind::Position`] percorre, cujo valor é a *distância* ao longo dela.
    ///
    /// ⚠️ **Mora no CLIP, e é a decisão inteira** (Enio, 2026-07-30): um clip novo nasce
    /// **em branco**, sem trajetória a herdar, contaminar nem desalinhar, e a primeira key
    /// dele cria a sua. O porquê e as duas portas de leitura estão no módulo irmão
    /// [`crate::doc::clip_paths`]; o report que o motivou, em
    /// `docs/Timeline/BUGS_timeline.md` #2b.
    ///
    /// Apêndice posicional ⇒ `DOC_VERSION` 16 -> 17.
    #[serde(default)]
    pub paths: BTreeMap<AnimTarget, crate::path::MotionPath>,
}

/// The editable timeline document (see module docs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineDoc {
    /// Schema version (first field on the wire).
    pub version: u32,
    /// Display frame rate for the ruler + frame readouts.
    pub fps_display: f64,
    clips: Vec<NamedClip>,
    active_clip: usize,
    bindings: Vec<TargetBinding>,
    markers: Vec<Marker>,
    /// Monotonic opaque-target allocator (see module docs).
    next_target: u64,
    /// Monotonic strip-id allocator (see `StripId`). Appended (v4).
    next_strip: u64,
    /// The clip stack: lanes of clip instances, bottom to top (ADR-0115).
    ///
    /// **Empty is the default and is not a degenerate case** — an empty stack
    /// means "the active clip drives the scene", which is what this document did
    /// before the stack existed, on the same code path and to the same bytes.
    /// Appended field (v4).
    stack: Vec<ClipLane>,
    /// Reusable nested stacks ([`NamedContainer`], ADR-0133). Appended (v8).
    ///
    /// **Empty is the default and is not a degenerate case** — a document with no containers
    /// behaves exactly as v7 did, on the same code path.
    containers: Vec<NamedContainer>,
    /// The SCENE's (Arrange) **explicit duration** in seconds, when authored — the
    /// third of the three scopes ([`NamedClip::length_override`] carries the clip's,
    /// [`crate::NamedContainer::length_override`] each container's). Same contract:
    /// defines "the end", cuts content past it non-destructively, `None` = derived.
    /// Appended (v11).
    pub scene_length: Option<f64>,
    /// This frame's live strips and their per-entity clocks (`stack_eval.rs`).
    /// Runtime scratch, not document identity: never serialized, always compares
    /// equal, and its buffers are retained frame to frame (zero-alloc, HR-3).
    #[serde(skip)]
    scratch: StackScratch,
    /// A direção do relógio DESTE frame — transiente, prosa em [`Self::set_reverse_play`].
    #[serde(skip)]
    reverse_play: bool,
}

impl Default for TimelineDoc {
    fn default() -> Self {
        Self::new()
    }
}

/// ⭐ **A AUTORIA de clips** — filho pelo teto de 700 LOC da workspace, e o corte é por
/// RESPONSABILIDADE: aqui vive *o que o documento É* (as bindings, as chaves, o serializar); ali,
/// *como o artista cria, nomeia, copia e apaga uma animação*.
///
/// ⚠️ **FILHO e não irmão**, e é o que torna o corte barato: um módulo filho vê os campos privados
/// do pai, então o `clips`/`active_clip` continuam fechados ao resto da crate.
#[path = "doc_clips.rs"]
mod clips;

impl TimelineDoc {
    /// A fresh document: one empty clip named "Main", 24 fps, no bindings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: DOC_VERSION,
            fps_display: DEFAULT_FPS,
            clips: vec![NamedClip {
                name: "Main".to_string(),
                clip: Clip::new(RationalTime::from_seconds(0.0)),
                loop_range: None,
                loop_ping_pong: false,
                keys_loop_range: None,
                keys_loop_ping_pong: false,
                length_override: None,
                expr: BTreeMap::new(),
                paths: BTreeMap::new(),
            }],
            active_clip: 0,
            bindings: Vec::new(),
            markers: Vec::new(),
            next_target: 0,
            next_strip: 0,
            stack: Vec::new(),
            containers: Vec::new(),
            scene_length: None,
            scratch: StackScratch::default(),
            reverse_play: false,
        }
    }

    /// The bindings vec + every clip, mutably — what [`Self::purge_binding`]
    /// (in `binding.rs`, the module that owns the document↔object link) needs
    /// and nothing else should reach for. `bindings_mut` hands out a slice on
    /// purpose (nobody else may REMOVE), and this file sits at its LOC cap.
    pub(crate) fn purge_parts(&mut self) -> (&mut Vec<TargetBinding>, &mut [NamedClip]) {
        (&mut self.bindings, &mut self.clips)
    }

    /// Hand out a fresh strip identity. Monotonic and never reused: a stale drag
    /// or undo entry must resolve to "gone", never to somebody else's strip.
    pub(crate) fn alloc_strip_id(&mut self) -> crate::stack::StripId {
        self.next_strip += 1;
        crate::stack::StripId(self.next_strip)
    }

    /// The clip stack, bottom lane first. Empty = the active clip drives.
    #[must_use]
    pub fn stack(&self) -> &[ClipLane] {
        &self.stack
    }

    /// The clip stack, for editing.
    pub fn stack_mut(&mut self) -> &mut Vec<ClipLane> {
        &mut self.stack
    }

    /// Every container, in index order (ADR-0133). Empty until one is authored.
    #[must_use]
    pub fn containers(&self) -> &[NamedContainer] {
        &self.containers
    }

    /// The containers, for editing. Crate-internal: authoring goes through
    /// [`crate::nest`], which is where the cycle guard lives — a caller that could push a
    /// container strip directly would be a door around it.
    pub(crate) fn containers_mut(&mut self) -> &mut Vec<NamedContainer> {
        &mut self.containers
    }

    // The scratch plumbing (take/put/scratch/stash_composed_links/prime_stack) lives in the
    // `scratch` child module, split out under the LOC cap (ADR-0152 W6).

    /// All clips.
    #[must_use]
    pub fn clips(&self) -> &[NamedClip] {
        &self.clips
    }

    /// The index of the clip currently edited.
    #[must_use]
    pub fn active_index(&self) -> usize {
        self.active_clip
    }

    /// Select which clip is edited (clamped to a valid index).
    pub fn set_active(&mut self, index: usize) {
        if index < self.clips.len() {
            self.active_clip = index;
        }
    }

    /// Serialize the document to the versioned on-disk format (postcard). The
    /// schema `version` is the first field, so a loader can reject or migrate an
    /// older file before trusting the rest (W4 save; HR-14).
    ///
    /// Bindings serialize by `wire_id`; the live `entity` bits are `#[serde(skip)]`
    /// — stamp the wire ids first (see [`crate::stamp_wire_ids`]).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(self).map_err(|e| e.to_string())
    }

    /// Load a document saved by [`Self::to_bytes`], rejecting a schema version
    /// this build does not understand. The loaded bindings have null `entity`
    /// bits — resolve them with [`crate::resolve_entities`].
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let doc: TimelineDoc = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
        if doc.version != DOC_VERSION {
            return Err(format!(
                "timeline schema version {} != {DOC_VERSION}",
                doc.version
            ));
        }
        // **The second cycle layer** (ADR-0133 §4). The authoring guard cannot see links that
        // did not come through it — a file edited by hand, corrupted, or written by a build
        // whose guard had a hole. We REJECT rather than repair: Blender's load-time
        // `BKE_collection_cycles_fix` silently zeroes the offending reference, which saves the
        // file by destroying the artist's link. A cyclic document is our bug to fix, not the
        // document's to lose.
        if let Some(c) = doc.find_nest_cycle() {
            let name = doc.containers().get(c).map_or("?", |n| n.name.as_str());
            return Err(format!(
                "timeline document has a container cycle through \"{name}\" (index {c})"
            ));
        }
        Ok(doc)
    }

    /// The clip currently edited.
    #[must_use]
    pub fn active_clip(&self) -> &Clip {
        &self.clips[self.active_clip].clip
    }

    /// The clip currently edited, mutably.
    pub fn active_clip_mut(&mut self) -> &mut Clip {
        &mut self.clips[self.active_clip].clip
    }

    /// All document bindings.
    #[must_use]
    pub fn bindings(&self) -> &[TargetBinding] {
        &self.bindings
    }

    /// All document bindings, mutably (used by liveness + wire-id resolution).
    pub fn bindings_mut(&mut self) -> &mut [TargetBinding] {
        &mut self.bindings
    }

    /// The binding for a live `(entity, prop)`, if bound.
    #[must_use]
    pub fn binding_for(&self, entity: u64, prop: PropKind) -> Option<&TargetBinding> {
        self.bindings
            .iter()
            .find(|b| b.entity == entity && b.prop == prop)
    }

    /// The binding a target names, if any.
    #[must_use]
    pub fn binding(&self, target: AnimTarget) -> Option<&TargetBinding> {
        self.bindings.iter().find(|b| b.target == target)
    }

    /// Bind `(entity, prop)`, returning the (existing or freshly allocated)
    /// opaque target. Idempotent per `(entity, prop)`; does **not** create a
    /// track (that happens lazily on first key — see [`TimelineDoc::insert_key`]).
    pub fn bind(&mut self, entity: u64, prop: PropKind) -> AnimTarget {
        if let Some(b) = self.binding_for(entity, prop) {
            return b.target;
        }
        let target = AnimTarget::new(self.next_target);
        self.next_target += 1;
        self.bindings.push(TargetBinding::new(target, entity, prop));
        target
    }

    /// Remove a binding and its track from the active clip. Returns `true` if a
    /// binding was removed.
    pub fn unbind(&mut self, entity: u64, prop: PropKind) -> bool {
        let Some(pos) = self
            .bindings
            .iter()
            .position(|b| b.entity == entity && b.prop == prop)
        else {
            return false;
        };
        let target = self.bindings.remove(pos).target;
        self.active_clip_mut().remove_track(target);
        // A trajetória deste clip vai junto: ela é a geometria que a track percorria, e
        // deixá-la para trás faria o próximo `bind` do mesmo alvo herdar a curva de uma
        // animação que já não existe (`NamedClip::paths`).
        self.take_active_path(target);
        true
    }

    /// Insert a key on `(entity, prop)`'s track in the active clip, binding +
    /// creating the track if needed. Returns the target and the new key id.
    pub fn insert_key(
        &mut self,
        entity: u64,
        prop: PropKind,
        t: RationalTime,
        value: AnimValue,
        interp: Interp,
    ) -> (AnimTarget, KeyId) {
        let target = self.bind(entity, prop);
        let track = self
            .active_clip_mut()
            .track_or_insert(target, || Track::new(vec![]).with_default(value));
        let id = track.insert_key(t, value, interp);
        (target, id)
    }

    /// Like [`TimelineDoc::insert_key`] but **updates** the key at exactly `t`
    /// rather than stacking a duplicate — the capture-the-pose path (K /
    /// auto-key) upserts one key per playhead time.
    ///
    /// Re-keying an existing instant records the new pose and **keeps that key's
    /// interpolation**: nudging the sprite on canvas with auto-key armed must not
    /// silently undo the easing the author drew in the graph editor. `interp` is
    /// therefore only the default for a key this call creates.
    pub fn upsert_key(
        &mut self,
        entity: u64,
        prop: PropKind,
        t: RationalTime,
        value: AnimValue,
        interp: Interp,
    ) -> (AnimTarget, KeyId) {
        let target = self.bind(entity, prop);
        let track = self
            .active_clip_mut()
            .track_or_insert(target, || Track::new(vec![]).with_default(value));
        let id = track.upsert_value(t, value, interp);
        // Auto-key reaches the document directly (no intent), so this is its
        // roving choke point: a re-keyed value shifts the derived times.
        track.resolve_roving();
        (target, id)
    }

    /// After loading, reseat the target allocator past every bound target so new
    /// bindings never collide with loaded ones (defensive against hand-edited
    /// or older files).
    pub fn reseat_allocator(&mut self) {
        let max = self.bindings.iter().map(|b| b.target.get()).max();
        if let Some(m) = max {
            self.next_target = self.next_target.max(m + 1);
        }
    }
}

/// **Where the content ends** — one question, four answers (clip / any clip /
/// stack / this view). A CHILD module (not a sibling) so it reads the document's
/// private fields, the idiom `Track`'s `rove` already uses.
#[path = "doc_extent.rs"]
mod extent;

/// **The per-clip, per-view loop pair** — same child-module idiom as `extent`.
#[path = "doc_loops.rs"]
mod loops;

/// **The runtime scratch plumbing** (take/put/stash/prime) — same child-module idiom, so it
/// reads the private `scratch` field. Split out under the 700-LOC cap (ADR-0152 W6).
#[path = "doc_scratch.rs"]
mod scratch;
