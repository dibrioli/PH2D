use super::*;

impl WidgetStore {
    /// Construct an empty store. The capacity hint pre-sizes the
    /// `focus_order` vec; the BTreeMap grows on demand at register
    /// time. Hot-path operations (`get`/`get_mut`) never allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            asset_drag: None,
            asset_drag_origin: (0.0, 0.0),
            asset_drag_label: None,
            asset_drop_fields: Vec::new(),
            asset_cells: std::collections::BTreeMap::new(),
            asset_doc_cells: std::collections::BTreeSet::new(),
            states: super::estados_dos_widgets::EstadosDosWidgets::default(),
            focus_order: Vec::with_capacity(capacity),
            hot_id: None,
            active_id: None,
            focus_id: None,
            composition: None,
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
            accessory_widths: super::AccessoryWidths::default(),
            panel_slot: std::collections::BTreeMap::new(),
            panel_tab_order: Vec::new(),
            tab_drag: None,
            tab_drop: None,
            active_layout: crate::screens::task_layout::TaskLayout::default(),
            pending_clipboard_copy: None,
            pending_clipboard_paste: None,
            current_scene_name: String::from("Level_01"),
            tool_space_local: false,
            tool_view_mode: 0,
            painter_shapes_flyout_open: false,
            painter_mask_flyout_open: false,
            painter_rail_wet: false,
            panel_z_order: Vec::new(),
            eyedropper_pending: None,
            palette_io_pending: None,
            palette_dropdown_open: None,
            panel_scroll: BTreeMap::new(),
            panel_scroll_live: BTreeMap::new(),
            fold_live: BTreeMap::new(),
            fold_body_h: core::cell::RefCell::new(BTreeMap::new()),
            hover_live: BTreeMap::new(),
            hover_acesos: std::collections::BTreeSet::new(),
            caret_off: std::collections::BTreeSet::new(),
            panel_rects: BTreeMap::new(),
            floating_panels: std::collections::BTreeSet::new(),
            sub_scroll_rects: BTreeMap::new(),
            panel_content_h: BTreeMap::new(),
            panel_visible_h: BTreeMap::new(),
            scroll: super::scroll_state::ScrollState::default(),
            tooltips: BTreeMap::new(),
            collapsed: BTreeMap::new(),
            context_menu: None,
            new_image_size: 512, // default square size highlighted when the modal opens
            new_image_bg: 0,     // default background = transparent
            new_image_custom: None,
            // O lado de textura do dispositivo antes de a shell o publicar: 8192 é o tier Default que o `GpuContext` pede
            // (medido 06/11: dispositivo 8192, adaptador 32768; `o_custom_nasce_com_os_lados_pedidos_ate_ao_tecto_do_dispositivo`).
            new_image_max_side: 8192,
            new_image_request: None,
            // ⚠️ Semeado no `open_sheet_size_dialog` com o que o empacotador SUGERE para aquelas
            // peças; este valor é só o que aparece antes de haver sugestão nenhuma.
            sheet: super::chrome_ops::SheetModal {
                size: 1024,
                request: None,
            },
            pixel: crate::pixel_panel::PixelDialogs::default(),
            fill_modal: None,
            input_map_tab_request: false,
            joint_editor_request: None,
            input_map_listening: None,
            input_map_captured: None,
            input_map_listen_replaces: false,
            input_map_listen_cancelled_by_down: None,
            onion_modal: None,
            command_palette: None,
            command_pick: None,
            command_palette_query: String::new(),
            command_palette_toggled: false,
            command_palette_scope: Default::default(),
            section_outline_color: BTreeMap::new(),
            section_prefs: Default::default(),
            notes_per_panel: BTreeMap::new(),
            last_context_menu: None,
            picker_target: None,
            picker_pinned: false,
            widget_colors: BTreeMap::new(),
            scrollbar_drag: None,
            body_scroll_drag: None,
            dropdown_popover: None,
            own_list: None,
            own_list_dismissed: None,
            card_dropdown_open: false,
            radius_scale: 1.0,
            rail_button_size: crate::widget::RailButtonSize::default(),
            dock_w_left: None,
            dock_w_right: None,
            tool_overflow: Vec::new(),
            area_menus: crate::interaction::AreaMenus::default(),
            tool_bars: crate::interaction::ToolBarsSlot::default(),
            menu_chain: Default::default(),
            faixa: Default::default(),
            menu_contrib: Vec::new(),
            dock_h_bottom: None,
            present_vsync: true,
            hierarchy_order: Vec::new(),
            hierarchy_parent: BTreeMap::new(),
            hierarchy_collapsed: std::collections::BTreeSet::new(),
            hier_rows: Default::default(),
            hierarchy_drag: None,
            hierarchy_row_ids: std::collections::BTreeSet::new(),
            hierarchy_board_rows: std::collections::BTreeMap::new(),
            hierarchy_entity_rows: std::collections::BTreeMap::new(),
            multiline_text_ids: std::collections::BTreeSet::new(),
            cancel_on_escape_ids: std::collections::BTreeSet::new(),
            mixed_numbers: std::collections::BTreeSet::new(),
            inherited_numbers: std::collections::BTreeSet::new(),
            number_input_drag: None,
            number_stepper_hold: None,
            shift_held: false,
            cmd_held: false,
            panel_row_drag: None,
            curve_point_drag: None,
            panel_row_family: std::collections::BTreeMap::new(),
            picker_swatch_ids: std::collections::BTreeSet::new(),
            graph_gestures: Vec::new(),
            graph_zoom: BTreeMap::new(),
            graph_keys: Vec::new(),
            list_nav: None,
            atomic_words: None,
            graph_canvas: BTreeMap::new(),
            graph_focused: None,
            graph_slop: crate::interaction::PressSlop::default(),
            graph_double: false,
            timeline_gestures: Vec::new(),
            timeline_slop: crate::interaction::PressSlop::default(),
            timeline_double: false,
            timeline_tinta_do_cursor: None,
            timeline_wheel: BTreeMap::new(),
            timeline_canvas: BTreeMap::new(),
            flip_strip: Default::default(),
            pixel_timeline: Default::default(),
            alt_held: false,
            card_fields: Default::default(),
            card_lists: Default::default(),
            menu_marks: Default::default(),
            key_card: Default::default(),
        }
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

    /// The sibling of [`mark_collapsible_section`](Self::mark_collapsible_section): this id is
    /// **no longer** a section header.
    ///
    /// ⚠️ For the twenty-three hand-written panels this never fires — their tables are `const`,
    /// and a header stays a header for the lifetime of the build. It exists for the **authored**
    /// panel, whose table is the artist's document: a row that wore a `SectionHeader` a moment
    /// ago can wear a `Checkbox` now. The mark is consulted by the dispatch **before** the widget
    /// switch, so one that outlives the type change makes the click fold a section that no longer
    /// exists instead of ticking the box — measured, 2026-08-09.
    ///
    /// Idempotent, and a no-op for an id that was never marked: the caller asks *"is this a
    /// header?"* once and states the answer, rather than remembering whether it had said the
    /// opposite before.
    pub fn unmark_collapsible_section(&mut self, id: NodeId) {
        self.collapsible_sections.remove(&id);
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
        use crate::interaction::DOUBLE_CLICK_WINDOW_NS;
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
        self.mixed_numbers.remove(&id);
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

    /// **A porta de SAÍDA das duas acima** — este id não descreve mais um widget que existe.
    ///
    /// ⚠️ Para os vinte e três painéis escritos à mão isto nunca dispara: as tabelas deles são
    /// `const`, e um widget existe pelo tempo de vida do build. Ela existe para o painel
    /// **AUTORADO**, cuja tabela é o documento do artista — uma row some quando ele apaga a forma,
    /// a despe, ou a renomeia.
    ///
    /// ⚠️ **E o dano de não a ter é herança de valor, não lixo:** o id de uma row autorada é o hash
    /// do RÓTULO, então uma forma nova de mesmo nome e mesmo tipo cai no mesmo id — e o `adopt`,
    /// vendo o discriminante bater, PRESERVA o estado. O controle novo nasce na posição de um que
    /// o artista apagou. Medido, 2026-08-09; e o store é do APP, então a herança atravessa a troca
    /// de projeto.
    ///
    /// ⚠️ **O `focus_order` sai junto, e não é higiene:** ele só cresce, e o `is_focusable` não
    /// sabe de morte — deixar o id lá é o Tab a pousar num controle que já não existe, o que num
    /// `TextInput` é digitar num campo invisível.
    ///
    /// Idempotente, e no-op para um id que nunca foi registado.
    pub fn unregister(&mut self, id: NodeId) {
        self.mixed_numbers.remove(&id);
        self.inherited_numbers.remove(&id);
        if self.states.remove(&id).is_some() {
            self.focus_order.retain(|&f| f != id);
        }
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
            let display = f64::from(self.linked_chip_display(num, v));
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
        if self.focus_id != id {
            self.composition = None;
        }
        self.focus_id = id;
    }

    /// A pré-edição do IME no campo FOCADO ([`crate::ime`]); nunca parte do valor.
    pub fn composition(&self) -> Option<&crate::ime::Composition> {
        self.composition
            .as_ref()
            .filter(|c| Some(c.id) == self.focus_id)
    }

    pub fn set_composition(&mut self, c: Option<crate::ime::Composition>) {
        self.composition = c;
    }

    /// Iterate registered widgets in registration order. Used by
    /// keyboard Tab traversal (insertion order is the focus order).
    pub fn focus_order(&self) -> &[NodeId] {
        &self.focus_order
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
