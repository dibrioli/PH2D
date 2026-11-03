//! A ENTRADA do [`HeroScreen`]: ponteiro, tecla, texto e roda seguem para o despacho, e o
//! [`HeroScreen::apply_event`] passa cada [`WidgetEvent`] às regiões em ordem z.

use super::*;

impl HeroScreen {
    pub fn handle_pointer<'frame>(
        &mut self,
        event: PointerEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        if document_tabs_menu::pointer(self, event) {
            return &[];
        }
        dispatch_pointer(&mut self.store, &self.hit_index, event, arena)
    }

    /// [`Self::handle_pointer`] with a live `TextSystem`, so a click snaps the caret to the glyph.
    /// ⚠️ Both ask the document tabs first (right-click menu + drag of a board tab).
    pub fn handle_pointer_with_text<'frame>(
        &mut self,
        event: PointerEvent,
        text_system: &mut TextSystem,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        if document_tabs_menu::pointer(self, event) {
            return &[];
        }
        dispatch_pointer_with_text(
            &mut self.store,
            &self.hit_index,
            event,
            Some(text_system),
            arena,
        )
    }

    pub fn handle_key<'frame>(
        &mut self,
        event: KeyEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        crate::interaction::dispatch_key(&mut self.store, event, arena)
    }

    /// Forward a printable character into the focused widget's buffer (`TextInput.text` /
    /// `Combobox.query` / `NumberInput.buffer`). NumberInput only accepts `[0-9.eE+-]`;
    /// TextInput/Combobox accept anything non-control.
    pub fn handle_text_input<'frame>(
        &mut self,
        ch: char,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        if surface_input::compose(self, crate::doc_surface::Compose::Commit(ch)) == Some(true) {
            return &[];
        }
        crate::interaction::dispatch_text_input(&mut self.store, ch, arena)
    }

    /// Forward a wheel/trackpad scroll into the dispatch. Painters publish their panel rects each
    /// frame (`WidgetStore::set_panel_rect`), so the dispatch knows which panel is under the cursor.
    pub fn handle_wheel<'frame>(
        &mut self,
        event: ph2d_host::WheelEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        crate::interaction::dispatch_wheel(&mut self.store, event, arena)
    }

    /// Translate a [`WidgetEvent`] from the dispatcher into a
    /// hero-level state mutation. Walks each region's
    /// `apply_event` in z-order; first region that consumes the
    /// event wins. Returns true iff some region consumed it.
    pub fn apply_event(&mut self, event: WidgetEvent) -> bool {
        // ⛔ **Antes de tudo** — ver `pre_dispatch`, que explica por que a ordem é load-bearing.
        if pre_dispatch::run(self, event) {
            return true;
        }
        // ⭐ CartoesUI: a lista suspensa de um cartão abre/fecha ANTES dos painéis — a opção
        //    escolhida segue para o painel dono, e a lista fecha no mesmo clique.
        if let WidgetEvent::Click(id) = event
            && (crate::panel::cards::dropdown::on_click(id)
                || crate::panel::cards::apply_loose_click(&mut self.store, id))
        {
            return true;
        }
        // T44: o alfinete cicla três estados — o 2.º clique rápido chega como duplo e conta.
        if let WidgetEvent::DoubleClick(id) = event
            && crate::panel::cards::apply_loose_click(&mut self.store, id)
        {
            return true;
        }
        if crate::panel::cards::estilos::on_event(&mut self.store, event) {
            return true;
        }
        // ADR-0029 Phase D: legacy fn-pointer dispatch deleted — every
        // in-tree panel lives in `crate::panel::PANEL_REGISTRY` as a
        // typed `Panel<State>`. Walk only the typed registry.
        // Tripartite outcome semantics (audit B2 + A4): Consumed stops
        // iteration entirely (returns `true`); Observed records a side
        // effect but continues; Ignored is a no-op.
        let mut observed = false;
        let consumed = crate::panel::with_registry_opt(|reg| {
            for panel in reg.panels_mut() {
                match panel.apply_event(self, event) {
                    crate::panel::EventOutcome::Consumed => return true,
                    crate::panel::EventOutcome::Observed => observed = true,
                    crate::panel::EventOutcome::Ignored => {}
                }
            }
            false
        })
        .unwrap_or(false);
        if consumed {
            return true;
        }
        // ADR-0029 Phase C.1: host-level showcase event handler —
        // covers `CTX_MENU_OUTLINE_*`, `CTX_MENU_CREATE_NOTE`,
        // `SECTION_IDS`, `SECTION_COLOR_IDS`, radio/tab/tree pin
        // clicks. Shared across the live Inspector (when typed
        // panel is installed) and the Widget Gallery (legacy);
        // running at host level means the gallery keeps working
        // when the typed Inspector is absent.
        if theme_menu::apply_section_theme_click(&mut self.store, event)
            || crate::panel::cards::apply_list_menu_click(&mut self.store, event)
            || crate::widget::showcase::apply_showcase_event(&mut self.store, event)
        {
            return true;
        }
        // Wave 9 Eixo A.1: chrome affordances split per file under
        // `chrome/` — theme menu, radius presets, view toggles, rail
        // panel/tool toggles, file menu, Settings cascades, scene
        // picker, image-edit actions. Adding a new chrome affordance
        // = drop a new `chrome/<feature>.rs` + one line in
        // `chrome::dispatch_all`. Multi-agent parallel work no longer
        // collides on this function.
        if chrome::dispatch_all(self, event) {
            return true;
        }
        if topbar::apply_event(&mut self.store, event) {
            return true;
        }
        if left_rail::apply_event(&mut self.store, event) {
            return true;
        }
        // Wave 8 Phase 4: return `observed` so a panel that did a
        // side-effect via `EventOutcome::Observed` (e.g. hierarchy
        // Blur(HIER_RENAME_INPUT) commits) propagates as "handled"
        // even when no chrome region consumed.
        observed
    }
}
