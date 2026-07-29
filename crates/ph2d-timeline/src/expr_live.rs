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
    /// **The pose to hand back**, as `(target, pre-expression value)`.
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
    /// ⚠️ The value is the pass's own **pre-expression `value`**, refreshed every
    /// previewed frame — not a snapshot taken when the preview began. A snapshot would
    /// be a second answer to what this property is (and would go stale the moment the
    /// artist scrubbed); `value` is the number the pass already computes to feed the
    /// formula, so restoring it is by construction "what it would have been".
    static RESTORE: RefCell<Option<(u64, f32)>> = const { RefCell::new(None) };
}

/// Install (or clear) the live preview. Called by the shell each frame — the same
/// shape as `ph2d_panel_timeline::set_current_timeline`.
pub fn set_live_expr(v: Option<LiveExpr>) {
    LIVE.with(|c| *c.borrow_mut() = v);
}

/// Remember what the previewed property WOULD have been, so it can be handed back.
/// Called by the pass every previewed frame (see [`RESTORE`]).
pub(crate) fn remember(target: u64, pre_expression_value: f32) {
    RESTORE.with(|c| *c.borrow_mut() = Some((target, pre_expression_value)));
}

/// The pose to hand back, ONCE, on the first apply after the preview stopped.
///
/// Returns `None` while the same target is still previewing — the pose is not owed back
/// until the card is gone. Taking it clears it, so the write happens exactly once and a
/// later authored change is never clobbered by a stale restore.
pub(crate) fn take_restore() -> Option<(u64, f32)> {
    let live_target = LIVE.with(|c| c.borrow().as_ref().map(|l| l.target));
    RESTORE.with(|c| {
        let owed = matches!(*c.borrow(), Some((t, _)) if live_target != Some(t));
        if owed { c.borrow_mut().take() } else { None }
    })
}

/// Whether a pose is owed back — the apply asks, so the frame that hands it back is not
/// skipped by the formula-free fast path.
#[must_use]
pub fn has_pending_restore() -> bool {
    let live_target = LIVE.with(|c| c.borrow().as_ref().map(|l| l.target));
    RESTORE.with(|c| matches!(*c.borrow(), Some((t, _)) if live_target != Some(t)))
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

    /// **A pose is owed back only once, and only after the card is gone.**
    #[test]
    fn the_pose_is_owed_back_once_and_only_after_the_card_closes() {
        set_live_expr(Some(LiveExpr {
            target: 3,
            formula: "value".into(),
            time: 0.0,
        }));
        remember(3, 7.5);
        assert!(
            !has_pending_restore(),
            "nothing is owed while the same card is still open"
        );
        assert!(take_restore().is_none());

        set_live_expr(None);
        assert!(has_pending_restore(), "closing the card owes the pose back");
        assert_eq!(take_restore(), Some((3, 7.5)));
        assert!(
            take_restore().is_none() && !has_pending_restore(),
            "and it is handed back exactly ONCE — a stale restore would clobber a later edit"
        );
    }
}
