//! The wet AUTHORED-ACTION doors (child of [`super`] — workspace file-LOC
//! cap): the canvas one-shots (Wet/Dry/Fast dry), the display-flag
//! recomposite, the session birth and the facts carrier the reconcile reads.

use super::*;

impl PainterTool {
    /// The authored engine facts (the reconcile's input, doc 22).
    pub(super) fn wet_facts(&self) -> WetEngineFacts {
        let w = &self.paint.wetpaint;
        WetEngineFacts {
            knobs: w.knobs,
            tilt: (w.tilt_on, w.tilt_ring, w.tilt_spoke),
            km_mixing: w.km_mixing,
        }
    }

    /// Create the session if none is live (the stamp's lazy birth, also the
    /// Wet-canvas button's door — wetting the sheet before the first stroke
    /// IS its use case). `false` when the canvas has no size.
    pub(super) fn ensure_wet_session(&mut self) -> bool {
        if self.paint.wetpaint.session.is_some() {
            return true;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return false;
        }
        self.paint.wetpaint.session = Some(WetSession {
            engine: EngineSlot::new_here(Engine::new(w as usize, h as usize)),
            worker: None,
            seen_steps: 0,
            base: Arc::clone(&self.canvas_rgba),
            // The identity token is WEAK on purpose ([`WetSession::canvas`]);
            // `base` is the one strong handle the session needs, and it is what
            // makes the FIRST composite copy — correctly, since the frozen base
            // has to survive the write.
            canvas: Arc::downgrade(&self.canvas_rgba),
            pigment: vec![0u8; w as usize * h as usize * 4],
            lanes: Vec::new(),
            stroke_open: false,
            paper_key: None,
            applied: WetEngineFacts::BOOT,
        });
        true
    }

    /// **Wet canvas** (one-shot): raise the sheet's wetness everywhere via
    /// max — no water injected, the sim stays idle, the next stroke bleeds
    /// anywhere. Creates the session if none is live.
    pub(crate) fn wetpaint_wet_canvas(&mut self) {
        if self.wet_authoring_hold() {
            return;
        }
        self.wetpaint_guard();
        if !self.ensure_wet_session() {
            return;
        }
        let facts = self.wet_facts();
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            // A PORTA, explícita. ⚠️ Não delegue ao `reconcile_facts`: ele sai
            // ANTES de trazer o motor quando os facts não mudaram (o caso
            // comum), e uma porta que só às vezes abre não é porta.
            sess.bring_home();
            sess.reconcile_facts(facts);
            sess.engine.wet_canvas_now();
        }
        self.wetpaint_composite();
    }

    /// **Dry canvas** (one-shot): settle all suspended pigment, zero water /
    /// velocity / wetness. No live session = nothing wet = an honest no-op.
    pub(crate) fn wetpaint_dry_canvas(&mut self) {
        if self.wet_authoring_hold() {
            return;
        }
        self.wetpaint_guard();
        let facts = self.wet_facts();
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            // A PORTA, explícita. ⚠️ Não delegue ao `reconcile_facts`: ele sai
            // ANTES de trazer o motor quando os facts não mudaram (o caso
            // comum), e uma porta que só às vezes abre não é porta.
            sess.bring_home();
            sess.reconcile_facts(facts);
            sess.engine.dry_canvas_now();
            self.wetpaint_composite();
        }
    }

    /// **Fast dry** (one-shot): accelerated evaporation+settle passes until
    /// the fluid is gone; the edge rims still darken. No session = no-op.
    pub(crate) fn wetpaint_fast_dry(&mut self) {
        if self.wet_authoring_hold() {
            return;
        }
        self.wetpaint_guard();
        let facts = self.wet_facts();
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            // A PORTA, explícita. ⚠️ Não delegue ao `reconcile_facts`: ele sai
            // ANTES de trazer o motor quando os facts não mudaram (o caso
            // comum), e uma porta que só às vezes abre não é porta.
            sess.bring_home();
            sess.reconcile_facts(facts);
            sess.engine.fast_dry_now();
            self.wetpaint_composite();
        }
    }

    /// Recomposite the whole session after a display-flag flip (Show wet /
    /// Paper / Glaze). During an authoring hold only the dirty mark lands —
    /// the resume tick composites (doc 21 law D forbids composites here).
    pub(crate) fn wet_recomposite_full(&mut self) {
        self.wetpaint_guard();
        let hold = self.wet_authoring_hold();
        if let Some(sess) = self.paint.wetpaint.session.as_mut() {
            sess.bring_home();
            sess.engine.mark_dirty_full();
            if !hold {
                self.wetpaint_composite();
            }
        }
    }

    /// End the session (mode switch / explicit teardown). The last composite
    /// is already in `canvas_rgba`, so ending IS the bake — the water just
    /// stops moving. The show-wet VEIL must never bake (doc 22 §2.7): with
    /// the overlay on and the canvas still ours, recomposite clean first.
    pub(crate) fn wetpaint_end_session(&mut self) {
        self.wetpaint_guard();
        if self.paint.wetpaint.show_wet && self.paint.wetpaint.session.is_some() {
            if let Some(sess) = self.paint.wetpaint.session.as_mut() {
                sess.bring_home();
                sess.engine.mark_dirty_full();
            }
            self.wetpaint_composite_veiled(false);
        }
        self.paint.wetpaint.session = None;
        // Doc 21: mode-leave hygiene — a stash must not survive into another
        // mode's commit. (The GUARD-kill branch deliberately does NOT clear
        // it: a mid-authoring undo keeps authoring, and its later commit
        // still deposits — into a fresh session over the peeled canvas.)
        self.paint.wetpaint.pending_deposit.clear();
    }
}
