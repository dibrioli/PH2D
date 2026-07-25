//! [`TimelineState`] — the full editor state the shell embeds in `AppGfx`:
//! the [`TimelineDoc`] plus panel-side selection, undo [`TimelineHistory`], and
//! transport-adjacent flags.
//!
//! Only the `doc` is undoable; selection + flags are panel state (never
//! snapshotted). Pan/zoom live on the panel itself (`Panel::State`), not here.

use ph2d_anim::{AnimTarget, KeyId};

use crate::clipboard::TimelineClipboard;
use crate::doc::TimelineDoc;
use crate::history::TimelineHistory;

/// One selected key in the active clip: which track (`target`) and which key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectedKey {
    /// The track's opaque target.
    pub target: AnimTarget,
    /// The stable key id within that track.
    pub key: KeyId,
}

impl SelectedKey {
    /// Build from raw ids — lets a UI layer that carried the key's identity as
    /// primitives (e.g. a `TimelineHitKind::Key { target, key }` gesture)
    /// reconstruct the typed selection without naming the `ph2d-anim` types.
    #[must_use]
    pub const fn new(target_raw: u64, key_raw: u64) -> Self {
        Self {
            target: AnimTarget::new(target_raw),
            key: KeyId::new(key_raw),
        }
    }
}

/// The set of selected keys (dope-sheet / graph). Small, order-insensitive; a
/// `Vec` is fine at the key counts a hand-authored clip holds.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    keys: Vec<SelectedKey>,
}

impl Selection {
    /// An empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` if `key` is selected.
    #[must_use]
    pub fn contains(&self, key: SelectedKey) -> bool {
        self.keys.contains(&key)
    }

    /// Whether nothing is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Number of selected keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// All selected keys.
    #[must_use]
    pub fn keys(&self) -> &[SelectedKey] {
        &self.keys
    }

    /// Replace the selection with a single key.
    pub fn set_single(&mut self, key: SelectedKey) {
        self.keys.clear();
        self.keys.push(key);
    }

    /// Toggle a key's membership (shift-click).
    pub fn toggle(&mut self, key: SelectedKey) {
        if let Some(i) = self.keys.iter().position(|k| *k == key) {
            self.keys.remove(i);
        } else {
            self.keys.push(key);
        }
    }

    /// Add a key if not already present (box-select accumulation).
    pub fn add(&mut self, key: SelectedKey) {
        if !self.keys.contains(&key) {
            self.keys.push(key);
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    /// The selected key ids for one track (for a bulk op on that track).
    #[must_use]
    pub fn ids_for(&self, target: AnimTarget) -> Vec<KeyId> {
        self.keys
            .iter()
            .filter(|k| k.target == target)
            .map(|k| k.key)
            .collect()
    }
}

/// Panel/transport flags that are not part of the undoable document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineFlags {
    /// Auto-key armed: moving the object records a key at the playhead (W4).
    pub auto_key: bool,
    /// Snap edited/scrubbed times to whole display frames.
    pub frame_snap: bool,
    /// **Performing / record** armed (W5): while the transport is PLAYING,
    /// dragging a bound object records its pose live along the playhead — mocap
    /// by hand. Distinct from `auto_key`, which is inert during play precisely
    /// so the tocando animation cannot mint keys on its own: record fires ONLY
    /// with an active manipulation gesture, never the passive pose the curve is
    /// driving. Off by default — it is a modal, explicit mode.
    pub performing: bool,
    /// **Play drives the rigid simulation too** (ADR-0131).
    ///
    /// The transport is ONE clock with two consumers: the curves in this
    /// document, and the rapier world. Left implicit they run together, and
    /// that is a conflict rather than a feature — scrubbing to review an
    /// animation would also drop every dynamic body a little further, so the
    /// scene the artist is judging is never the scene they authored.
    ///
    /// So the two are separated at the transport, where the artist can see
    /// which one is armed. **Off by default:** this is an animation timeline,
    /// and the answer to "what does Play do?" has to be the same on the frame
    /// after a project loads as it was before it was saved — a simulation that
    /// starts itself is a scene that has already changed by the time it is
    /// looked at. Physics is opted INTO, per session.
    ///
    /// ⚠️ Disarming does not freeze the objects, it stops *simulating* them:
    /// a body whose pose the timeline drives (a baked one, W4) still follows
    /// its curves, because those are animation. That is the whole shape of
    /// Bake — it converts a simulation into an animation, and an animation is
    /// exactly what plays with this off.
    pub simulate_physics: bool,
}

impl Default for TimelineFlags {
    fn default() -> Self {
        Self {
            // DISARMED by default (Enio, 2026-07-22 — reverses the launch default):
            // auto-key silently converts every pose nudge into animation, and a
            // recorder that arms itself is the same class of surprise as a physics
            // sim that starts itself (`simulate_physics` below). Keying is opted
            // INTO per session; the transport pill is the switch.
            auto_key: false,
            frame_snap: true,
            // Record is modal and deliberate — never armed on its own.
            performing: false,
            // Physics is opted into: Play means "play my animation" until the
            // artist says otherwise. See the field docs.
            simulate_physics: false,
        }
    }
}

/// The editor state for the timeline: document + selection + history + flags.
#[derive(Debug, Default, Clone)]
pub struct TimelineState {
    /// The undoable document.
    pub doc: TimelineDoc,
    /// Current key selection (panel state, not undoable).
    pub selection: Selection,
    /// Undo/redo over the document.
    pub history: TimelineHistory,
    /// Transport/panel flags (not undoable).
    pub flags: TimelineFlags,
    /// Copied keys (panel state, not undoable, not serialized).
    pub clipboard: TimelineClipboard,
    /// **The view the panel last painted is the Keys tab** — the shell stamps this
    /// each frame from `ph2d_panel_timeline::state::keys_mode()`, before draining
    /// intents. It picks which of the active clip's two loops an edit (or a
    /// clip-switch sync) targets: the Keys-view clip-clock loop, or the Arrange
    /// timeline loop. Session state, like [`Self::flags`]; defaults to Arrange.
    pub keys_mode: bool,
    /// **The panel is showing the Containers LIST** (the Containers tab at its root
    /// level) — stamped by the shell each frame from
    /// `ph2d_panel_timeline::state::containers_list()`, the same channel
    /// [`Self::keys_mode`] rides, and `false` while the panel is hidden.
    ///
    /// The list is a LIBRARY, not a view of time, so **playback does not exist
    /// there** (Enio, 2026-07-22): the transport's play/pause is refused, the loop
    /// mark is not drawn, and a clock that was already running is paused. Everything
    /// that enforces that reads THIS field — one stamped fact, not three private
    /// re-derivations. It comes back the moment the animator enters a container or
    /// returns to Keys/Arrange.
    pub containers_list: bool,
    /// **The container whose interior the transport is EDITING** (`Some(c)` only while
    /// inside container `c`'s lanes — not Keys, not the scene root; Enio, 2026-07-22).
    /// Stamped by the shell each frame, the nav-side sibling of [`Self::keys_mode`]. It
    /// is what makes the third clock work: the auto-key pass primes its scratch ROOTED
    /// here (`prime_rooted`), so keying inside a container reads the CONTAINER's blend
    /// and lands on the CONTAINER's clock, not the scene's.
    pub container_open: Option<usize>,
    /// **How deep into the nesting the animator has walked** — outermost first, empty at the
    /// scene root (ADR-0133 §5).
    ///
    /// A PATH and not a single container, because the trail has to answer two things that a
    /// single index cannot: *where you came from* (entering B from inside A must still offer
    /// the way back to A) and *where you are* (the trail would otherwise read `Scene > B`
    /// while you stand in `A > B`, which is a wrong statement, not a terse one).
    ///
    /// Routing only ever needs the innermost — that is [`Self::edit_host`], DERIVED here
    /// rather than stamped beside the path, so the two can never disagree about where the
    /// animator is.
    ///
    /// Stamped by the shell each frame from `ph2d_panel_timeline::state::edit_path()`, before
    /// intents drain — the same channel [`Self::keys_mode`] rides, for the same reason: the
    /// panel knows where the animator is looking, and an edit has to land where they are
    /// looking. Session state; a document does not remember which container was open, exactly
    /// as Animate does not.
    pub edit_path: Vec<crate::EnterStep>,
}

impl TimelineState {
    /// **Which stack an edit lands in** — the innermost container of [`Self::edit_path`], or
    /// the document's own stack at the root.
    #[must_use]
    pub fn edit_host(&self) -> crate::nest::StackHost {
        self.edit_path
            .last()
            .map_or(crate::nest::StackHost::Document, |step| {
                crate::nest::StackHost::Container(step.container)
            })
    }

    /// A fresh state around an empty document.
    ///
    /// **Derived end, no authored duration** — the invariant every test leans on
    /// (`view_end_seconds` reads the content). The PRODUCT default is
    /// [`Self::with_default_duration`]; the shell uses that so a freshly opened
    /// timeline is a 4 s composition, not an open-ended one pinned at t = 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A fresh state whose active clip carries the **product default duration**
    /// ([`crate::DEFAULT_DURATION_SECONDS`], 4 s) — what the app opens with (Enio,
    /// 2026-07-23: *"a timeline abre com Duração zero. Por padrão coloque 4 segundos"*).
    ///
    /// An authored duration (not a derived one) so the composition end — and the veil
    /// that darkens past it — is there from the first frame, before a single key. On
    /// the Keys tab with no stack the clip IS the timeline ([`crate::TimelineDoc::view_authored_end`]),
    /// so the clip's override is the right scope; the Dur(s) chip and the veil drag both
    /// edit this same number. Kept OUT of [`Self::new`] so the crate's "fresh = derived"
    /// invariant (and the tests on it) stay intact.
    #[must_use]
    pub fn with_default_duration() -> Self {
        let mut st = Self::default();
        // Both the active clip (the Keys tab) AND the scene (the Arrange tab) open as
        // 4 s compositions (Enio, 2026-07-23: *"ao entrar em arrange, dur = 0, deveria
        // ser 4 seg por padrão"*). Each tab authors its OWN scope; defaulting both is
        // what makes every tab open at 4 s, veil and all, rather than a derived 0.
        st.doc
            .set_clip_length_override(0, Some(crate::DEFAULT_DURATION_SECONDS));
        st.doc
            .set_scene_length(Some(crate::DEFAULT_DURATION_SECONDS));
        st
    }

    /// Undo one document step (returns `true` if something was undone). The
    /// selection is cleared, since undone keys may no longer exist.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.undo(&self.doc) {
            self.doc = prev;
            self.selection.clear();
            true
        } else {
            false
        }
    }

    /// Redo one document step (returns `true` if something was redone).
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.history.redo(&self.doc) {
            self.doc = next;
            self.selection.clear();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod flag_default_tests {
    use super::TimelineFlags;

    /// **Os defaults dos flags são decisões de PRODUTO, pinadas** — cada um já foi
    /// decidido pelo Enio em separado, e um default que se move em silêncio inverte
    /// o sentido de toda fixture que chega ao estado por toggle.
    ///
    /// - `auto_key` DESARMADO (2026-07-22): keyar é opt-in por sessão — um gravador
    ///   que se arma sozinho transforma ajuste de pose em animação sem aviso.
    /// - `performing` e `simulate_physics` desarmados pelas decisões já documentadas
    ///   nos campos; `frame_snap` armado (autoria em quadros inteiros é o normal).
    #[test]
    fn the_flag_defaults_are_the_product_decisions() {
        let f = TimelineFlags::default();
        assert!(!f.auto_key, "AutoKey nasce DESMARCADO (Enio, 2026-07-22)");
        assert!(!f.performing, "Record é modal e deliberado");
        assert!(!f.simulate_physics, "a sim não se arma sozinha (ADR-0131)");
        assert!(f.frame_snap, "snap de quadro é o normal da autoria");
    }
}

#[cfg(test)]
mod default_duration_tests {
    use super::TimelineState;

    /// **A timeline do PRODUTO abre com 4 s autorados** (Enio, 2026-07-23), e a
    /// veil fecha desde o 1º frame — enquanto `new()` fica DERIVADO (o invariante
    /// dos testes). As duas coisas de uma vez: o clip carrega o override, e a vista
    /// (sem pilha, o clip É a timeline) responde autorada.
    #[test]
    fn the_product_default_is_a_four_second_authored_composition() {
        let plain = TimelineState::new();
        assert_eq!(
            plain.doc.clip_length_override(0),
            None,
            "new() fica DERIVADO — o invariante da crate"
        );
        assert_eq!(
            plain.doc.view_authored_end(None, false),
            None,
            "e a vista abre aberta (sem veil) num doc cru"
        );

        let product = TimelineState::with_default_duration();
        assert_eq!(
            product.doc.clip_length_override(0),
            Some(crate::DEFAULT_DURATION_SECONDS),
            "o produto abre com o override de 4 s no CLIP (a aba Keys)"
        );
        assert_eq!(
            product.doc.scene_length,
            Some(crate::DEFAULT_DURATION_SECONDS),
            "e 4 s na CENA (a aba Arrange), senão Arrange abre com Dur 0"
        );
        assert_eq!(
            product.doc.view_authored_end(None, false),
            Some(4.0),
            "a vista fecha (a veil aparece) desde o 1º frame"
        );
        assert!(
            (product.doc.view_end_seconds(false) - 4.0).abs() < 1e-9,
            "a caixa Dur mostra 4"
        );
    }
}
