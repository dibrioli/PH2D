//! **Que ferramenta está na mão?** — as duas direções da pergunta, lado a lado.
//!
//! [`PainterTool::set_paint_tool_mode`] é a PORTA de entrada (o fio que o rail e a lista TOOL do
//! Impasto publicam) e [`PainterTool::active_paint_mode_id`] é a de saída (o que o rail lê para
//! acender o chip certo, e o que o arrasto de Fill captura para restaurar a ferramenta depois de um
//! ColorDrop). Elas moram juntas porque são **um mapeamento**: um vocabulário só, sem uma terceira
//! grafia para driftar — a mesma razão pela qual `rail_painter_tools` mantém `push_paint_mode` e
//! `sync_from_mode` no mesmo arquivo.
//!
//! Os `is_*_mode` vêm junto: são a mesma pergunta, feita por quem PINTA o painel.
//!
//! Extraído de `stencil.rs` em 2026-08-08 (teto de LOC), e o corte é por ASSUNTO — aquele arquivo é o
//! editor da alça do stencil, e nunca teve nada a ver com qual ferramenta o artista segura.

use super::PainterTool;

impl PainterTool {
    /// Set the active paint operation from the left-rail tool selection: `"smear"` → the Smear drag,
    /// `"blur"` → the Blur/soften, `"eraser"` → normal paint with the Erase-Alpha override, anything
    /// else → normal Brush paint. Keeps `paint_mode` + the eraser override in sync so switching rail
    /// tools never leaves a stuck state (e.g. Brush after Smear returns to normal painting). Beside
    /// `route_brush_dab_event`, which drives it (moved here off `brush_settings.rs` for the LOC cap).
    pub fn set_paint_tool_mode(&mut self, mode: &str) {
        // Leaving Fill with a live ColorDrop → commit it (push its undo) so switching tools never loses
        // the fill or its undo step. No-op unless a fill is in progress.
        self.fill_commit();
        // The transient Mask scratch is NOT discarded on a tool switch: it persists while its target
        // layer stays active (`mask_scratch_active()` self-gates on target == active), so you can leave
        // Mask, retouch the concealed area with the Brush, and return. Apply promotes it to a layer mask;
        // switching layers lets it go dormant. (This setter is on `arch_mode_has_reconcile`'s BENIGN list:
        // it writes only simple mode fields, with no derived cache to settle.)
        // Any tool switch disarms a pending Eyedropper pick; only "eyedropper" re-arms it below.
        self.paint.eyedropper_armed = false;
        use super::PaintMode;
        let old_mode = self.paint.paint_mode;
        let old_method = self.paint.brush.stroke_method;
        let new_mode = match mode {
            "smear" => PaintMode::Smear,
            "blur" => PaintMode::Blur,
            "clone" => PaintMode::Clone,
            "mask" => PaintMode::Mask,
            "inpaint" => PaintMode::Inpaint,
            "fill" => PaintMode::Fill,
            "selection" => PaintMode::Selection,
            // The warp's two halves. ⚠️ There is no `"deform"` wire any more: it entered the mode and
            // left the temperament UNSELECTED, which is a tool that consumes the drag and moves nothing
            // (measured: 0 pixels, `measure_rail_chips`). Each wire now names the half it enters, and
            // the temperament is armed a few lines below — AFTER the mode switch, which is what resets
            // it to NONE.
            "liquify" | "transform" => PaintMode::Deform,
            "sculpt" => PaintMode::Sculpt,
            "knife" => PaintMode::Knife,
            "wetpaint" => PaintMode::WetPaint,
            // "eraser" INSIDE Wet Paint STAYS in Wet Paint (W2.6): the fluid
            // engine has its own eraser (`Tool::Erase`) and leaving the mode
            // would BAKE the very painting the artist is trying to correct —
            // the water must survive its eraser (Rebelle's semantics). The
            // same-mode transition skips every teardown below (`old != new`
            // guards), so the session lives on.
            "eraser" if self.paint.paint_mode == PaintMode::WetPaint => PaintMode::WetPaint,
            // With the Wet Paint checkbox ARMED, "brush" IS the fluid — this
            // is what makes the arm survive tool round-trips (eraser /
            // selection / smear and back), the Watercolor/Impasto pattern
            // (Enio 2026-07-21). Only the explicit "brush" wire: eyedropper
            // stays a momentary Paint, and unknown wires keep their
            // conservative fallback.
            "brush" if self.paint.wetpaint.armed => PaintMode::WetPaint,
            // "brush" / "eraser" / "eyedropper" / anything else → normal Paint.
            _ => PaintMode::Paint,
        };
        // Independent-tools model: load the target mode's OWN brush settings (Smear/Blur/Clone slots seed
        // Spacing 5% for dense dabs — see `state_default`). No-op when settings are linked. Must run while
        // `paint_mode` still holds the OLD mode, so the current tool's edits save to the right slot.
        self.switch_brush_slot(new_mode);
        // Leaving Deform bakes any live Transform float (committing it as one undo entry) and ends the
        // Reshape session (drops the `pre`/displacement) so a later mode's edits aren't re-warped from a
        // stale baseline; the Reshape pixels are already committed per-stroke.
        // Leaving the Smear ends its warp session for the same reason: a `disp` and a frozen `pre`
        // that belong to one tool must never be read by another, and `pre` describes a canvas that
        // the next tool's edits will move out from under it.
        if self.paint.paint_mode.smears() && !new_mode.smears() {
            self.end_warp_session();
        }
        if self.paint.paint_mode == PaintMode::Deform && new_mode != PaintMode::Deform {
            self.end_transform(true);
            self.end_deform_session();
        }
        // ENTERING Deform: the temperament opens UNSELECTED — the artist must pick Reshape or Transform
        // each time the panel is entered, so re-picking Transform always re-lifts a fresh gizmo (Enio
        // 2026-07-04). Any prior transform was already baked on the last leave.
        if self.paint.paint_mode != PaintMode::Deform && new_mode == PaintMode::Deform {
            self.paint.deform.temperament = super::DEFORM_TEMPERAMENT_NONE;
        }
        // Leaving Sculpt parks nothing: the carving is already committed to the layer (the sculpt writes
        // `heights` live, it does not stage it), so all that is left to do is let go of the frozen source.
        // Keeping it would mean a Radius drag made in some OTHER tool re-rendered a stroke the artist has
        // moved on from — a knob reaching back through time.
        if self.paint.paint_mode == PaintMode::Sculpt && new_mode != PaintMode::Sculpt {
            self.end_sculpt_session();
        }
        // Leaving Wet Paint ends the fluid session: the last composite already IS the canvas (ending
        // is the bake), and the sim must stop moving paint the moment another tool owns the pixels.
        if self.paint.paint_mode == PaintMode::WetPaint && new_mode != PaintMode::WetPaint {
            self.wetpaint_end_session();
        }
        self.paint.paint_mode = new_mode;
        // ⚠️ **Order is load-bearing:** entering Deform resets the temperament to NONE (a few lines up),
        // so a wire that names its half has to arm it AFTER that reset — the same trap
        // `set_paint_media` paid for when it armed the medium before picking the tool. Re-picking the
        // half you are already in is not a no-op and must not be: `set_deform_temperament` ends and
        // re-begins, which is what makes Transform re-lift a fresh gizmo (Enio 2026-07-04) — the very
        // behaviour the NONE lobby existed to guarantee.
        match mode {
            "liquify" => self.set_deform_temperament(super::DEFORM_TEMPERAMENT_RESHAPE),
            "transform" => self.set_deform_temperament(super::DEFORM_TEMPERAMENT_TRANSFORM),
            _ => {}
        }
        // Entering Wet Paint by ANY door arms the checkbox — painting wet
        // with the Enable reading OFF would be a lying checkbox. Leaving
        // does NOT disarm (the arm is the persistent authored state; only
        // the checkbox / its reset disarm), exactly like Watercolor's flag.
        if new_mode == PaintMode::WetPaint {
            self.paint.wetpaint.armed = true;
        }
        // Doc 21: crossing the wet boundary with shapes open KEEPS them (the
        // W3 entry coercion is reverted — authoring is flat in wet too) and
        // re-forms the preview under the new mode's rules in both directions:
        // entering peels the relief-bearing Paint preview and re-stamps flat;
        // leaving re-stamps with relief over the baked water. The shape's
        // METHOD rides across the boundary too — pointer routing is by
        // method, so the freshly loaded slot's own method would orphan the
        // editor (open, painted, and unreachable under the mouse).
        if (old_mode == PaintMode::WetPaint) != (new_mode == PaintMode::WetPaint)
            && (self.is_editing_shape() || self.has_parked_shapes())
        {
            if old_method.is_shape() {
                self.paint.brush.stroke_method = old_method;
            }
            self.paint.wet_shape_active = false;
            self.refill_open_shape();
        }
        // The brush slot has just been swapped underneath us, so the "does anything read `Dab::dir`?" answer
        // has to be re-asked: leaving Sculpt must clear the Chisel's heading need, entering it must restore
        // it. (The slot round-trips the flag, but the LIVE brush is what `Stroke::new` reads.)
        self.sync_stroke_heading_need();
        self.arm_tool_falloff_defaults();
        // Leaving the Selection tool auto-hides its gizmos (the "Show Selection Gizmos" checkbox unchecks) —
        // the gizmos belong to Select and would otherwise linger over another tool (Enio 2026-07-03).
        if new_mode != PaintMode::Selection && self.paint.selection_edit_mode {
            self.exit_selection_edit();
        }
        // Per-mode flags. Smear/Blur/Clone leave the eraser override as-is (unchanged behaviour); the
        // colour-painting modes clear it; Eyedropper additionally arms the pick.
        match mode {
            // Sculpt joins Smear/Blur/Clone here for the same reason: it processes what is already on the
            // canvas rather than laying colour, so the Eraser override is none of its business.
            "smear" | "blur" | "clone" | "sculpt" => {}
            "eyedropper" => {
                self.paint.eraser = false;
                self.paint.eyedropper_armed = true;
            }
            "eraser" => self.paint.eraser = true,
            // mask / inpaint / fill / brush / default.
            _ => self.paint.eraser = false,
        }
    }

    /// The active paint mode as the `set_paint_tool_mode` string id — its inverse, used by the shell to
    /// CAPTURE the current tool before a momentary Fill drag and RESTORE it after (Enio 2026-07-03).
    #[must_use]
    pub fn active_paint_mode_id(&self) -> &'static str {
        use super::PaintMode;
        match self.paint.paint_mode {
            PaintMode::Smear => "smear",
            PaintMode::Blur => "blur",
            PaintMode::Clone => "clone",
            PaintMode::Mask => "mask",
            PaintMode::Inpaint => "inpaint",
            PaintMode::Fill => "fill",
            PaintMode::Selection => "selection",
            // ⚠️ The mode alone does not say which tool the artist is holding — the temperament does,
            // and the rail has a chip for each. Reporting one string for both would light the wrong
            // chip and, through the Fill drag's capture/restore, hand back the wrong tool.
            PaintMode::Deform
                if self.paint.deform.temperament == super::DEFORM_TEMPERAMENT_TRANSFORM =>
            {
                "transform"
            }
            PaintMode::Deform => "liquify",
            PaintMode::Sculpt => "sculpt",
            PaintMode::Knife => "knife",
            // The wet eraser is still the ERASER in the artist's hand — the
            // rail radio must light that chip, not the brush (W2.6).
            PaintMode::WetPaint if self.paint.eraser => "eraser",
            // Wet Paint is the BRUSH's flavour (the checkbox, like
            // Watercolor) — the rail lights Brush, and a momentary
            // capture/restore round-trips through "brush" back into the
            // fluid because the arm persists.
            PaintMode::WetPaint => "brush",
            PaintMode::Paint if self.paint.eraser => "eraser",
            PaintMode::Paint => "brush",
        }
    }

    /// Whether the active paint operation is **Smear** — the panel snapshot mirrors this so the
    /// incompatible brush controls (colour / blend / ramps / eraser) hide.
    #[must_use]
    pub fn is_smear_mode(&self) -> bool {
        self.paint.paint_mode.smears()
    }

    /// Whether the active paint operation is **Blur** — the panel hides the same colour-family controls
    /// as Smear (Blur processes pixels, it paints no colour).
    #[must_use]
    pub fn is_blur_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Blur)
    }

    /// Whether the active paint operation is **Mask** — paints a grayscale mask value. The panel keeps
    /// the full brush but hides Colour / Randomize / Composite and locks the ramps to B&W.
    #[must_use]
    pub fn is_mask_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Mask)
    }

    /// Whether the active paint operation is **Inpaint** — the content-aware heal brush. Like Smear/Blur
    /// it paints no colour (it marks a defect region), so the panel hides the same colour-family controls.
    #[must_use]
    pub fn is_inpaint_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Inpaint)
    }

    /// Whether the active operation is **Selection** — the panel shows ONLY the selection controls
    /// (mode-exclusive, like Inpaint), nothing shared with the other tools.
    #[must_use]
    pub fn is_selection_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Selection)
    }

    /// Whether the active operation is **Deform** — the panel shows ONLY the deform controls (mode-exclusive,
    /// like Selection): mode segmented · Size/Pressure/Distortion/Momentum/Strength · Freeze · actions.
    #[must_use]
    pub fn is_deform_mode(&self) -> bool {
        matches!(self.paint.paint_mode, super::PaintMode::Deform)
    }
}
