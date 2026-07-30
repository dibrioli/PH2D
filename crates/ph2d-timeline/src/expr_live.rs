//! **The LIVE expression preview** — a formula the expression pass runs *as if* it
//! were authored, on its own real-time clock, while an editor is open on it.
//!
//! ⚠️ **Display state, never document state.** Nothing here is serialised, nothing
//! here can be undone, and nothing here writes to a [`crate::TimelineDoc`]. It exists
//! so the artist can watch the effect on the OBJECT — the real one, in the scene —
//! while they tune it, which is what the smoke asked for: *"vamos fazer o efeito
//! correr no objeto selecionado em tempo real mesmo que o clip esteja pausado, desde
//! que o painel esteja aberto"* (Enio, 2026-07-29). The preview evaporates the frame
//! the editor closes, and the next apply rewrites the property from the curves.
//!
//! ⚠️ **It REPLACES the binding's own expression, it does not compose with it.** The
//! card seeds itself from whatever formula the track already carries, so the sheet
//! ALREADY contains it; running both would apply it twice, and the artist would be
//! tuning against a doubled version of their own work.
//!
//! ⚠️ **The clock is the caller's, and it must be WALL-CLOCK.** *"Em tempo real mesmo
//! que o clip esteja pausado"* is exactly the case where the playhead is not moving,
//! so the preview cannot ride it — and it cannot ride a frame COUNT either, or the
//! wobble the artist is judging would run at a speed that depends on the frame rate.
//! That is the artwork half of this repo's clock law, and a preview of artwork is on
//! the artwork side of it.
//!
//! ⚠️ **The SEED is the binding's own, not the preview's.** It comes from the pass
//! (`b.target * SEED_SPACING`), untouched — so what runs while the card is open is
//! the same noise that runs after Apply. A preview with its own seed would show a
//! different wobble from the one it is previewing, which is the one thing it must
//! never do.

use std::cell::RefCell;
use std::collections::BTreeMap;

/// A formula being previewed live on one binding.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveExpr {
    /// The `AnimTarget` (raw) this drives — the binding the editor is open on.
    pub target: u64,
    /// The formula, exactly as the editor projects it.
    pub formula: String,
    /// The preview's own clock, in seconds. Advances in real time even when the
    /// transport is paused.
    pub time: f64,
}

thread_local! {
    static LIVE: RefCell<Option<LiveExpr>> = const { RefCell::new(None) };
    /// **The formula-drive LEDGER** — for every channel a formula is driving, the pose to
    /// hand back if it stops: `target -> pre-expression value`.
    ///
    /// ⚠️ This exists because of a gate that was RIGHT and a comment of mine that was
    /// WRONG. I wrote that stopping the preview is the whole of the undo — the keyed
    /// pass rewrites the property from the curves every frame anyway — and that is
    /// true for a KEYED channel and false for the commonest one: a **bare** binding
    /// (no keys, no formula) is deliberately sparse, `solo_source_value` returns
    /// `None` for it so a just-created binding is never forced to a default, and
    /// therefore NOBODY writes it. Measured: cancel the card and the object stayed
    /// where the preview left it.
    ///
    /// ⚠️ **And it was wired to ONE event — the end of a live preview — which is the
    /// smaller half.** Deleting an AUTHORED formula is the same event and had no
    /// hand-back at all: measured (auditoria 2026-07-29, §4 D-I) a bare binding driven by
    /// `value + 250` stayed at **250.0000** after DELETE + Apply, and on every frame
    /// after. That is *"mesmo deletando as expressões, elas ficam atuando"*, literally.
    /// So this is a MAP now, filled by every site that drives a formula (the global
    /// post-pass, the per-clip blend, the preview), and drained by the one place that
    /// knows nobody answered for a channel.
    ///
    /// ⚠️ The value is the driver's own **pre-expression `value`**, refreshed every driven
    /// frame — not a snapshot taken when the formula was installed. A snapshot would be a
    /// second answer to what this property is (and would go stale the moment the artist
    /// scrubbed); `value` is the number the driver already computes to feed the formula, so
    /// restoring it is by construction "what it would have been".
    static OWED: RefCell<BTreeMap<u64, f32>> = const { RefCell::new(BTreeMap::new()) };
}

/// Install (or clear) the live preview. Called by the shell each frame — the same
/// shape as `ph2d_panel_timeline::set_current_timeline`.
pub fn set_live_expr(v: Option<LiveExpr>) {
    LIVE.with(|c| *c.borrow_mut() = v);
}

/// Remember what a DRIVEN property would have been without its formula, so it can be
/// handed back if the formula goes away. Called by every driver, every driven frame
/// (see [`OWED`]).
pub(crate) fn remember(target: u64, pre_expression_value: f32) {
    OWED.with(|c| {
        c.borrow_mut().insert(target, pre_expression_value);
    });
}

/// **Every pose that is owed back**, and taking it clears it.
///
/// `still_driven` answers *"is a formula driving this channel right now?"* — a channel
/// whose formula is still installed (or whose card is still open) owes nothing, and
/// handing its pose back would fight the driver. The caller supplies that predicate
/// because it is the caller who knows the frame: the document's formulas, the preview
/// channel, and — critically — whether anything ELSE wrote the property this frame.
///
/// Draining is exactly-once per entry, so a later authored change is never clobbered by a
/// stale hand-back.
pub(crate) fn drain_owed(still_driven: &dyn Fn(u64) -> bool) -> Vec<(u64, f32)> {
    OWED.with(|c| {
        let mut owed = c.borrow_mut();
        let handing: Vec<(u64, f32)> = owed
            .iter()
            .filter(|(t, _)| !still_driven(**t))
            .map(|(t, v)| (*t, *v))
            .collect();
        for (t, _) in &handing {
            owed.remove(t);
        }
        handing
    })
}

/// Whether any pose is owed back — the apply asks, so the frame that hands it back is not
/// skipped by the formula-free fast path (`frame_solve::any_formula`).
///
/// ⚠️ True while a formula is still driving, too, and that is deliberate: keeping the pass
/// scheduled is what makes `composed` available on the frame the hand-back needs it, and a
/// document with a formula was taking that path anyway.
#[must_use]
pub fn has_pending_restore() -> bool {
    OWED.with(|c| !c.borrow().is_empty())
}

/// Forget every owed pose — for a host installing a different document.
///
/// ⚠️ Without this, loading project B would hand project A's poses to whatever bindings
/// happened to reuse those targets. The load already forgets the clock, the undo queue and
/// the timeline for exactly this reason.
pub fn forget_owed_poses() {
    OWED.with(|c| c.borrow_mut().clear());
}

/// What is being previewed, if anything.
#[must_use]
pub fn live_expr() -> Option<LiveExpr> {
    LIVE.with(|c| c.borrow().clone())
}

/// Whether a live preview is driving the scene right now.
///
/// The shell's undo asks this: while a preview drives a property, the world is not in
/// an authored state, so a diff against it would record a pose nobody wrote.
#[must_use]
pub fn is_previewing() -> bool {
    LIVE.with(|c| c.borrow().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_channel_is_empty_until_someone_fills_it_and_clears_on_none() {
        set_live_expr(None);
        assert!(!is_previewing(), "nothing previews by default");
        set_live_expr(Some(LiveExpr {
            target: 7,
            formula: "value + 1".into(),
            time: 0.5,
        }));
        assert!(is_previewing());
        assert_eq!(live_expr().unwrap().target, 7);
        set_live_expr(None);
        assert!(
            !is_previewing(),
            "closing the editor must leave nothing driving the scene"
        );
    }

    /// **A pose is owed back only once, and only after the driver is gone.**
    ///
    /// ⚠️ The "is it still driven?" question is now the CALLER's — the ledger cannot
    /// answer it, because a formula can be driving from three places (the global channel,
    /// any clip, the preview) and only the pass sees all three. So the predicate is passed
    /// in, and this test plays the caller: `driving` stands for *"the card is still open"*.
    #[test]
    fn the_pose_is_owed_back_once_and_only_after_the_driver_goes_away() {
        forget_owed_poses();
        remember(3, 7.5);
        assert!(has_pending_restore(), "the ledger has a note");
        assert!(
            drain_owed(&|t| t == 3).is_empty(),
            "nothing is handed back while that channel is still driven"
        );
        assert!(
            has_pending_restore(),
            "and a refused drain must not consume the note"
        );

        assert_eq!(drain_owed(&|_| false), vec![(3, 7.5)]);
        assert!(
            drain_owed(&|_| false).is_empty() && !has_pending_restore(),
            "and it is handed back exactly ONCE — a stale restore would clobber a later edit"
        );
    }

    /// **Several channels can owe at once**, which the single-slot version could not
    /// represent: two rows cleared in one gesture is one Apply.
    #[test]
    fn every_channel_that_owes_is_handed_back_and_the_driven_ones_are_left_alone() {
        forget_owed_poses();
        remember(1, 10.0);
        remember(2, 20.0);
        remember(3, 30.0);
        let handed = drain_owed(&|t| t == 2);
        assert_eq!(handed, vec![(1, 10.0), (3, 30.0)]);
        assert!(
            has_pending_restore(),
            "channel 2 is still driven, so its note stays"
        );
        forget_owed_poses();
        assert!(
            !has_pending_restore(),
            "and a host can drop the whole ledger"
        );
    }
}
