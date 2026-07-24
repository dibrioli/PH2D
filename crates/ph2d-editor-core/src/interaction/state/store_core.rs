use super::*;

impl WidgetStore {
    /// Construct an empty store. The capacity hint pre-sizes the
    /// `focus_order` vec; the BTreeMap grows on demand at register
    /// time. Hot-path operations (`get`/`get_mut`) never allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            states: BTreeMap::new(),
            focus_order: Vec::with_capacity(capacity),
            hot_id: None,
            active_id: None,
            focus_id: None,
            active_rect: None,
            slider_to_number: BTreeMap::new(),
            number_to_slider: BTreeMap::new(),
            number_to_slider_mapping: BTreeMap::new(),
            number_range: BTreeMap::new(),
            number_drag_rate: BTreeMap::new(),
            number_to_slider_snap_integer: std::collections::BTreeSet::new(),
            chips_without_steppers: std::collections::BTreeSet::new(),
            number_commit_always: std::collections::BTreeSet::new(),
            collapsible_sections: std::collections::BTreeSet::new(),
            hex_to_blender_parent: BTreeMap::new(),
            palette_name_to_parent: BTreeMap::new(),
            parent_to_palette_name: BTreeMap::new(),
            blender_channel_chip: BTreeMap::new(),
            last_down_id: None,
            last_down_at_ns: 0,
            pending_double_click: None,
            blender_palettes: BTreeMap::new(),
            blender_picker_offset: BTreeMap::new(),
            blender_drag_anchor: None,
            panel_resize_delta: BTreeMap::new(),
            panel_resize_anchor: None,
            panel_resize_anchor_bl: None,
            pending_clipboard_copy: None,
            pending_clipboard_paste: None,
            current_scene_name: String::from("Level_01"),
            tool_space_local: false,
            tool_view_mode: 0,
            painter_shapes_flyout_open: false,
            painter_mask_flyout_open: false,
            panel_z_order: Vec::new(),
            eyedropper_pending: None,
            palette_io_pending: None,
            palette_dropdown_open: None,
            panel_scroll: BTreeMap::new(),
            panel_rects: BTreeMap::new(),
            panel_content_h: BTreeMap::new(),
            panel_visible_h: BTreeMap::new(),
            tooltips: BTreeMap::new(),
            collapsed: BTreeMap::new(),
            context_menu: None,
            new_image_size: 512, // default square size highlighted when the modal opens
            new_image_bg: 0,     // default background = transparent
            new_image_request: None,
            fill_modal: None,
            onion_modal: None,
            section_outline_color: BTreeMap::new(),
            notes_per_panel: BTreeMap::new(),
            last_context_menu: None,
            picker_target: None,
            widget_colors: BTreeMap::new(),
            scrollbar_drag: None,
            dropdown_popover: None,
            radius_scale: 1.0,
            rail_button_size: crate::widget::RailButtonSize::default(),
            present_vsync: true,
            hierarchy_order: Vec::new(),
            hierarchy_parent: BTreeMap::new(),
            hierarchy_collapsed: std::collections::BTreeSet::new(),
            hierarchy_drag: None,
            hierarchy_row_ids: std::collections::BTreeSet::new(),
            multiline_text_ids: std::collections::BTreeSet::new(),
            cancel_on_escape_ids: std::collections::BTreeSet::new(),
            number_input_drag: None,
            number_stepper_hold: None,
            shift_held: false,
            cmd_held: false,
            painter_layer_drag: None,
            curve_point_drag: None,
            painter_layer_row_ids: std::collections::BTreeSet::new(),
            picker_swatch_ids: std::collections::BTreeSet::new(),
            graph_gestures: Vec::new(),
            graph_zoom: BTreeMap::new(),
            graph_keys: Vec::new(),
            graph_canvas: BTreeMap::new(),
            graph_focused: None,
            graph_moved: false,
            graph_double: false,
            timeline_gestures: Vec::new(),
            timeline_moved: false,
            timeline_press: (0.0, 0.0),
            timeline_double: false,
            timeline_wheel: BTreeMap::new(),
            timeline_canvas: BTreeMap::new(),
            flip_strip: Default::default(),
            alt_held: false,
        }
    }

    /// Register a bidirectional link: when `slider`'s value changes,
    /// `number`'s value follows; when `number` commits a new value,
    /// `slider` follows. Caller is responsible for both ids being
    /// pre-registered as Slider and NumberInput respectively.
    ///
    /// Post-2026-05-24: this no longer auto-marks the chip as
    /// no-stepper. The canon `paint_number_chip` always paints up/down
    /// arrows now, and the dispatch's stepper hit-test is the desired
    /// behavior for every chip — including chips linked to a slider.
    /// Phantom-stepper-while-still is impossible because there is no
    /// "no-arrow chip" variant anymore. See
    /// [`mark_chip_no_stepper`](Self::mark_chip_no_stepper) for the
    /// (deprecated) opt-out and the gate
    /// `architecture_no_chip_without_steppers` that prevents
    /// re-introducing pills sans-arrows.
    pub fn link_slider_number(&mut self, slider: NodeId, number: NodeId) {
        self.slider_to_number.insert(slider, number);
        self.number_to_slider.insert(number, slider);
    }

    /// Like [`link_slider_number`](Self::link_slider_number) but
    /// registers an affine projection between the slider's `0..1`
    /// storage and the chip's user-visible value:
    ///
    /// ```text
    /// chip_display = slider_storage * scale + offset
    /// slider_storage = (chip_display - offset) / scale
    /// ```
    ///
    /// Use whenever the chip paints a non-identity transform via
    /// `display_override` (Grow's signed `±1`, Min Px integer count,
    /// Upscale "2.00×", padding pixels, etc.). The dispatch then
    /// inverse-projects on every chip mutation (Enter commit, stepper
    /// arrow click, drag scrub, continuous hold) and forward-projects
    /// on every slider mutation (drag, programmatic set), so the
    /// chip's stored value lives in display-space throughout — exactly
    /// what the buffer shows on focus, exactly what the user types.
    ///
    /// `scale` must be non-zero (asserted in debug). Identity is
    /// `scale=1.0, offset=0.0`, equivalent to `link_slider_number`.
    pub fn link_slider_number_mapped(
        &mut self,
        slider: NodeId,
        number: NodeId,
        scale: f32,
        offset: f32,
    ) {
        self.link_slider_number_mapped_inner(slider, number, scale, offset, false);
    }

    /// Like [`link_slider_number_mapped`] but the chip's typed display
    /// value is **rounded to the nearest integer** before being written
    /// to the chip and inverse-projected to the slider. Use for chips
    /// whose painted unit is an integer count (BgRemoval Min Px,
    /// Color-Eq Tile Grid / Posterize Dither Grain, etc.) — without
    /// this, a user typing "50.5" left the chip stuck at fractional 50.5
    /// while the painter's `display_override` showed the rounded "50",
    /// and Tab-away / re-focus revealed the inconsistency (audit
    /// finding #3, 2026-05-28).
    ///
    /// The mapping itself is still the same affine `display = storage *
    /// scale + offset`; the snap is applied on TOP at the chip-write
    /// boundary so the slider's `0..1` storage can stay continuous (the
    /// painter rounds on its own from that continuous track when needed).
    pub fn link_slider_number_mapped_integer(
        &mut self,
        slider: NodeId,
        number: NodeId,
        scale: f32,
        offset: f32,
    ) {
        self.link_slider_number_mapped_inner(slider, number, scale, offset, true);
    }

    fn link_slider_number_mapped_inner(
        &mut self,
        slider: NodeId,
        number: NodeId,
        scale: f32,
        offset: f32,
        snap_integer: bool,
    ) {
        debug_assert!(
            scale.abs() > f32::EPSILON,
            "link_slider_number_mapped: scale must be non-zero"
        );
        self.slider_to_number.insert(slider, number);
        self.number_to_slider.insert(number, slider);
        if (scale - 1.0).abs() > f32::EPSILON || offset.abs() > f32::EPSILON {
            self.number_to_slider_mapping
                .insert(number, (scale, offset));
        } else {
            // Identity — keep the map clean so default-lookup is fast.
            self.number_to_slider_mapping.remove(&number);
        }
        if snap_integer {
            self.number_to_slider_snap_integer.insert(number);
        } else {
            self.number_to_slider_snap_integer.remove(&number);
        }
    }

    pub fn linked_number(&self, slider: NodeId) -> Option<NodeId> {
        self.slider_to_number.get(&slider).copied()
    }

    pub fn linked_slider(&self, number: NodeId) -> Option<NodeId> {
        self.number_to_slider.get(&number).copied()
    }

    /// `true` iff the chip is registered with
    /// [`link_slider_number_mapped_integer`] — the dispatch's
    /// [`apply_chip_value_with_mirror`] will `.round()` the typed
    /// display value before writing chip+slider.
    pub fn linked_slider_snap_integer(&self, number: NodeId) -> bool {
        self.number_to_slider_snap_integer.contains(&number)
    }

    /// Projection `(scale, offset)` for the chip→slider mirror.
    /// Returns identity `(1.0, 0.0)` when no mapping is registered —
    /// callers can always safely forward/inverse-apply without
    /// branching on the link kind.
    pub fn linked_slider_mapping(&self, number: NodeId) -> (f32, f32) {
        self.number_to_slider_mapping
            .get(&number)
            .copied()
            .unwrap_or((1.0, 0.0))
    }

    /// Register a NumberInput's **(min, max, step)** range — the single source the drag-scrub uses to
    /// map the cursor displacement PROPORTIONALLY to `[min, max]` + clamp, and the stepper uses for its
    /// increment. `min`/`max` may be given in either order (the dispatch normalises). Panels SHOULD call
    /// this for every bounded number box (e.g. when painting it) so the scrub feel matches the range.
    pub fn set_number_range(&mut self, id: NodeId, min: f64, max: f64, step: f64) {
        self.number_range.insert(id, (min, max, step));
    }

    /// The registered `(min, max, step)` for `id`, if any (see [`set_number_range`](Self::set_number_range)).
    #[must_use]
    pub fn number_range(&self, id: NodeId) -> Option<(f64, f64, f64)> {
        self.number_range.get(&id).copied()
    }

    /// Register a horizontal drag-scrub rate (**value-units per cursor pixel**)
    /// for an UNBOUNDED number box — the drag adds `rate·dx` (vertical `rate/10`),
    /// no clamp.
    ///
    /// ⚠️ **Este texto dizia para NÃO combinar com
    /// [`set_number_range`](Self::set_number_range)** — *"o modelo proporcional do alcance
    /// venceria e re-imporia limites"*. **O código faz o oposto** desde sempre: em
    /// `dispatch::pointer_move` o rate é consultado ANTES do alcance (e vence), e o cálculo
    /// de `bounds` devolve `None` assim que existe um rate (e vence de novo). Combinar os
    /// dois é a forma **certa** de uma caixa que precisa de `step` e de piso mas não tem
    /// TETO: o alcance serve de `step` para o stepper e de piso para o clamp dele, e o rate
    /// dá ao arrasto uma escala calibrada em vez de uma proporção sobre um intervalo que não
    /// termina. É o que os chips do transporte da timeline fazem
    /// (`ph2d-panel-timeline::transport_widgets::chip`), e o texto anterior era exatamente
    /// o que desencorajava a combinação correta ([[feedback_stale_comment_and_dead_code_lie]]).
    pub fn set_number_drag_rate(&mut self, id: NodeId, rate: f64) {
        self.number_drag_rate.insert(id, rate);
    }

    /// The registered unbounded drag rate for `id`, if any.
    #[must_use]
    pub fn number_drag_rate(&self, id: NodeId) -> Option<f64> {
        self.number_drag_rate.get(&id).copied()
    }

    /// **Deprecated (2026-05-24).** Marking a NumberInput as
    /// no-stepper made sense when `paint_number_chip` painted a bare
    /// pill — clicking the (invisible) right column armed a
    /// continuous-hold that climbed silently. After unification, every
    /// chip paints arrows and the click→step behavior is the desired
    /// affordance everywhere. Kept as a back-compat no-op for one wave
    /// while in-tree callers are removed; CI gate
    /// `architecture_no_chip_without_steppers` prevents reintroducing
    /// a chip variant that needs it. To be deleted in Wave 12.
    #[deprecated(
        since = "0.0.0",
        note = "all chips paint arrows now; the dispatch's stepper hit-test is the canon (Wave 11)"
    )]
    pub fn mark_chip_no_stepper(&mut self, id: NodeId) {
        self.chips_without_steppers.insert(id);
    }

    /// Whether the given NumberInput id is painted without stepper
    /// arrows. Always `false` for new code (post-2026-05-24 chip
    /// canon) — kept for back-compat with any in-flight call to the
    /// deprecated [`mark_chip_no_stepper`](Self::mark_chip_no_stepper).
    pub fn is_chip_no_stepper(&self, id: NodeId) -> bool {
        self.chips_without_steppers.contains(&id)
    }

    /// Mark a NumberInput so an explicit ENTER commits its buffer even when the
    /// typed value is unchanged — see [`Self::number_commit_always`]
    /// (`number_commit_always` field). The panel calls this once at populate for
    /// the timeline Dur(s) chip.
    pub fn set_number_commit_always(&mut self, id: NodeId) {
        self.number_commit_always.insert(id);
    }

    /// Whether an unchanged ENTER commit on `id` should still emit `ValueChanged`.
    #[must_use]
    pub fn number_commit_always(&self, id: NodeId) -> bool {
        self.number_commit_always.contains(&id)
    }

    /// Mark a section header NodeId as collapse-toggle eligible.
    /// Called from `pre_populate` / panel `populate` for every
    /// `paint_section_header` site so the dispatch knows a left-click
    /// on `id` should flip the collapse state via the existing
    /// [`toggle_collapsed`](Self::toggle_collapsed) API. UI canon
    /// post-2026-05-24: every section is collapsible (vide
    /// `docs/UI_Padrao/components/section_header.md`).
    pub fn mark_collapsible_section(&mut self, id: NodeId) {
        self.collapsible_sections.insert(id);
    }

    /// True iff the section is registered as collapse-toggle eligible.
    /// Dispatch consults this before firing the toggle on a click.
    pub fn is_collapsible_section(&self, id: NodeId) -> bool {
        self.collapsible_sections.contains(&id)
    }

    /// Record the latest pointer-Down for double-click detection.
    /// Returns true iff this Down should be treated as a double-click
    /// (same id as the previous Down + within `DOUBLE_CLICK_WINDOW_NS`
    /// of it).
    pub fn record_pointer_down(&mut self, id: Option<NodeId>, timestamp_ns: u128) -> bool {
        const DOUBLE_CLICK_WINDOW_NS: u128 = 350_000_000; // 350 ms
        let is_double = id.is_some()
            && id == self.last_down_id
            && timestamp_ns.saturating_sub(self.last_down_at_ns) < DOUBLE_CLICK_WINDOW_NS;
        // Reset the counter on a confirmed double-click so a third
        // rapid click doesn't register as another double.
        self.last_down_id = if is_double { None } else { id };
        self.last_down_at_ns = timestamp_ns;
        // Stash the upgrade hint so the matching Up emits
        // `WidgetEvent::DoubleClick(id)` in place of the regular
        // `Click(id)`. Cleared by `take_pending_double_click`.
        if is_double {
            self.pending_double_click = id;
        }
        is_double
    }

    /// Take + clear the `pending_double_click` slot, returning the
    /// id stored on the matching Mouse Down. `apply_click` consumes
    /// this to upgrade `Click(id)` → `DoubleClick(id)` when the id
    /// matches the click target.
    pub fn take_pending_double_click(&mut self) -> Option<NodeId> {
        self.pending_double_click.take()
    }

    /// Register a widget at construction time. Idempotent — repeat
    /// calls overwrite the state but never grow capacity. Should NOT
    /// be called during the paint/dispatch hot path.
    pub fn register(&mut self, id: NodeId, initial: InteractiveState) {
        if self.states.insert(id, initial).is_none() {
            self.focus_order.push(id);
        }
    }

    /// Register `id` only when it isn't already in the store. Unlike
    /// [`Self::register`] (which always replaces and is the right call
    /// for one-shot construction-time wiring), this is safe to call
    /// every frame from live-mode `repopulate` paths without
    /// clobbering user-typed text / cursor state. Returns true iff
    /// the entry was freshly inserted.
    pub fn register_if_absent(&mut self, id: NodeId, initial: InteractiveState) -> bool {
        if self.states.contains_key(&id) {
            return false;
        }
        self.states.insert(id, initial);
        self.focus_order.push(id);
        true
    }

    pub fn get(&self, id: NodeId) -> Option<&InteractiveState> {
        self.states.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut InteractiveState> {
        self.states.get_mut(&id)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.states.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub fn hot_id(&self) -> Option<NodeId> {
        self.hot_id
    }

    pub fn set_hot(&mut self, id: Option<NodeId>) {
        self.hot_id = id;
    }

    pub fn active_id(&self) -> Option<NodeId> {
        self.active_id
    }

    pub fn set_active(&mut self, id: Option<NodeId>) {
        self.active_id = id;
    }

    /// Programmatically set a slider's value (clamped `0..=1`) and, if a NumberInput is
    /// linked, its mirrored display value — WITHOUT a pointer drag.
    ///
    /// The shell uses this to RE-CENTER a live slider after a commit: the Offset slider, for
    /// instance, rests at "no offset" (`d = 0`) once the offset is baked, so the next grab
    /// starts fresh instead of jumping the shape by the last value. No-op if `id` is not a
    /// registered slider.
    pub fn set_slider_value(&mut self, id: NodeId, value: f32) {
        let v = value.clamp(0.0, 1.0);
        match self.get_mut(id) {
            Some(InteractiveState::Slider { value: slot, .. }) => *slot = v,
            _ => return,
        }
        if let Some(num) = self.linked_number(id) {
            let (scale, offset) = self.linked_slider_mapping(num);
            let display = f64::from(v * scale + offset);
            if let Some(InteractiveState::NumberInput { value, .. }) = self.get_mut(num) {
                *value = display;
            }
        }
    }

    /// Geometry of the active widget at the moment of Down. Used by
    /// drag-handling dispatch (Slider) to compute new value.
    pub fn active_rect(&self) -> Option<Rect> {
        self.active_rect
    }

    pub fn set_active_rect(&mut self, rect: Option<Rect>) {
        self.active_rect = rect;
    }

    pub fn focus_id(&self) -> Option<NodeId> {
        self.focus_id
    }

    pub fn set_focus(&mut self, id: Option<NodeId>) {
        self.focus_id = id;
    }

    /// Iterate registered widgets in registration order. Used by
    /// keyboard Tab traversal (insertion order is the focus order).
    pub fn focus_order(&self) -> &[NodeId] {
        &self.focus_order
    }

    /// M14.A: read the in-progress NumberInput drag (Down on the box
    /// body). `None` when no NumberInput is being dragged or the user
    /// is currently editing one (focus → caret mode, not drag mode).
    pub fn number_input_drag(&self) -> Option<NumberInputDragState> {
        self.number_input_drag
    }

    pub fn begin_number_input_drag(&mut self, drag: NumberInputDragState) {
        self.number_input_drag = Some(drag);
    }

    /// Flip the in-flight drag past the threshold (idempotent). Called
    /// by `dispatch_pointer` Move once the cursor has moved >
    /// `NUMBER_INPUT_DRAG_THRESHOLD_PX` from the Down position.
    ///
    /// `axis_horizontal` locks the active scrub axis for the rest of
    /// the drag — true = horizontal, false = vertical. The caller
    /// decides at the moment of promotion based on `|dx| vs |dy|`.
    /// Subsequent calls (the threshold is already crossed) are no-ops
    /// so the axis can't flip mid-drag.
    ///
    /// `cursor_x`/`cursor_y` re-anchor the incremental `last_x`/`last_y`
    /// to the cursor position AT promotion time. Without this, the
    /// SAME Move that crosses the threshold would compute its step
    /// delta from `start_x` (Down position) → applies the entire
    /// threshold-crossing distance (≈5 px × DRAG_RATE) as an instant
    /// JUMP before the user perceives any drag motion. Re-anchoring
    /// makes the promotion frame contribute a zero-delta and
    /// subsequent Moves compute their deltas from "here".
    pub fn promote_number_input_drag_to_slider(
        &mut self,
        axis_horizontal: bool,
        cursor_x: f32,
        cursor_y: f32,
    ) {
        if let Some(drag) = self.number_input_drag.as_mut()
            && !drag.crossed_threshold
        {
            drag.crossed_threshold = true;
            drag.axis_horizontal = axis_horizontal;
            drag.last_x = cursor_x;
            drag.last_y = cursor_y;
        }
    }

    /// Advance the incremental-drag anchor `last_x` / `last_y` to the
    /// cursor's current position. Called by `dispatch_pointer` Move
    /// after each per-Move delta has been applied so the NEXT Move
    /// computes its delta from "here", not from Down. This is the
    /// Blender/AE scrub model — a reversal after a clamp produces a
    /// non-zero step_dx on the very next Move (the absolute-delta
    /// model kept the value pegged at the clamp edge until the cursor
    /// returned all the way to `start_x`).
    pub fn advance_number_input_drag_anchor(&mut self, x: f32, y: f32) {
        if let Some(drag) = self.number_input_drag.as_mut() {
            drag.last_x = x;
            drag.last_y = y;
        }
    }

    /// Guarda o valor CONTÍNUO do scrub (ver [`NumberInputDragState::accum`]). Só as caixas que
    /// arredondam o leem — nas contínuas o acumulador nunca é consultado.
    pub fn set_number_input_drag_accum(&mut self, v: f64) {
        if let Some(drag) = self.number_input_drag.as_mut() {
            drag.accum = v;
        }
    }

    pub fn end_number_input_drag(&mut self) -> Option<NumberInputDragState> {
        self.number_input_drag.take()
    }

    /// M14.A: read the in-progress NumberInput stepper continuous-
    /// hold. `None` when no arrow is held.
    pub fn number_stepper_hold(&self) -> Option<NumberStepperHoldState> {
        self.number_stepper_hold
    }

    pub fn begin_number_stepper_hold(&mut self, hold: NumberStepperHoldState) {
        self.number_stepper_hold = Some(hold);
    }

    /// Update the `last_tick_ns` after `dispatch_tick` applied a
    /// repeat. Returns `None` if there's no hold in flight (no-op).
    pub fn record_number_stepper_tick(&mut self, now_ns: u128) {
        if let Some(h) = self.number_stepper_hold.as_mut() {
            h.last_tick_ns = now_ns;
        }
    }

    pub fn end_number_stepper_hold(&mut self) {
        self.number_stepper_hold = None;
    }

    /// M14.A: latest Shift modifier state. Shell pushes via
    /// [`Self::set_shift_held`] on every `WindowEvent::ModifiersChanged`.
    /// `dispatch_pointer` Move reads this to scale the drag delta
    /// (Shift = fine adjustment).
    pub fn shift_held(&self) -> bool {
        self.shift_held
    }

    pub fn set_shift_held(&mut self, held: bool) {
        self.shift_held = held;
    }

    /// Fase 0c: latest Cmd (macOS) / Ctrl (Linux/Windows) modifier
    /// state. Shell pushes via [`Self::set_cmd_held`] on every
    /// `WindowEvent::ModifiersChanged`. Hierarchy / canvas multi-
    /// select handlers read this to map Click → toggle-select.
    pub fn cmd_held(&self) -> bool {
        self.cmd_held
    }

    pub fn set_cmd_held(&mut self, held: bool) {
        self.cmd_held = held;
    }
}
