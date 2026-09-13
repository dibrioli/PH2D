//! **O clique, os reclamantes do fim** — ramos do `on_mouse_input` ([`super`]). Os corpos MUDARAM-SE
//! verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da chamada, pela mesma ordem.

use super::*;

impl crate::App {
    /// O pan do botão do meio, a barra lateral espelhada, a paleta legada e o painel da ferramenta.
    pub(super) fn ramo_pan_e_barra_lateral(&mut self, state: ElementState, button: MouseButton) {
        // M14.4b.bis: middle button = camera pan anchor. Tracked here
        // so CursorMoved can drive the pan. Motion Nodes M1 / timeline W2.E6:
        // NOT over the graph or the timeline dock — there middle-drag pans that
        // editor (via its own surface gesture), not the camera underneath. The
        // hovered component owns the pan, Blender-style.
        let over_pan_editor = self.cursor_over_motion_graph() || self.cursor_over_timeline();
        if button == MouseButton::Middle && !(over_pan_editor && state == ElementState::Pressed) {
            match state {
                ElementState::Pressed => {
                    self.pan_anchor = Some(self.last_pointer);
                }
                ElementState::Released => {
                    self.pan_anchor = None;
                }
            }
        }
        match state {
            ElementState::Pressed => {
                // Mirror-sidebar chip takes precedence over the panel
                // hit-test (different zone, no overlap).
                let mut consumed = false;
                if let Some(gfx) = self.gfx.as_mut()
                    && !gfx.zen.is_active()
                    && let Some(btn) = gfx.layout.mirror_button_rect()
                    && btn.contains(self.last_pointer.0, self.last_pointer.1)
                {
                    gfx.layout.mirror_sidebar();
                    gfx.toasts.push(Toast::info(format!(
                        "Sidebar · {:?}",
                        gfx.layout.sidebar_side
                    )));
                    self.title_dirty = true;
                    consumed = true;
                }
                // Tool palette icon click — switch active tool.
                //
                // CRITICAL: only hit-test the palette where it is actually
                // PAINTED — the legacy no-hero (demo) path. In the editor
                // (`hero_screen` is `Some`) the palette is NOT painted (the
                // editor switches tools via the LeftRail + Image Tools
                // pills), yet this hit-test used to run unconditionally.
                // Zone::TopRight is the right HALF of the toolbar strip —
                // exactly where the TopBar paints its right clusters incl.
                // the Settings gear — so a click on "Config" also landed on
                // an INVISIBLE palette slot and silently switched tools
                // ("Tool · Move"/"Tool · Padding"). Gating on
                // `hero_screen.is_none()` (the paint condition) makes the
                // top-right belong solely to the TopBar in the editor.
                //
                // The visible-tools filter below still applies in the demo
                // path so its indices match the paint mapping (no drift).
                if !consumed
                    && let Some(gfx) = self.gfx.as_mut()
                    && !gfx.zen.is_active()
                    && gfx.hero_screen.is_none()
                {
                    let mode_on = gfx
                        .hero_screen
                        .as_ref()
                        .map(|h| h.image_edit.mode_on)
                        .unwrap_or(false);
                    let visible = crate::palette_visible_tool_indices(&gfx.tools, mode_on);
                    let palette = gfx.layout.tool_palette_rects(visible.len());
                    let hit_idx = palette
                        .iter()
                        .position(|r| r.contains(self.last_pointer.0, self.last_pointer.1));
                    if let Some(slot) = hit_idx {
                        let tool_idx = visible[slot];
                        let tool_id = gfx.tools.tools()[tool_idx].id();
                        let tool_label = gfx.tools.tools()[tool_idx].label().to_string();
                        if gfx.tools.set_active(&tool_id) {
                            gfx.toasts.push(Toast::info(format!("Tool · {tool_label}")));
                            self.title_dirty = true;
                        }
                        consumed = true;
                    }
                }
                if !consumed {
                    // Mouse down — start hit-test against active panel.
                    self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, true);
                }
            }
            ElementState::Released => {
                // End any drag-in-progress.
                self.dragging = None;
            }
        }
    }

    /// Os reclamantes que vêm antes do gizmo: o Fill que se arrasta, os modais pelo título, e o `match` do
    /// conta-gotas, do pincel de protecção, das curvas, do Grid Stamp e do pincel do Painter — pela mesma ordem.
    pub(super) fn ramo_reclamantes(
        &mut self,
        mapped_button: ph2d_host::PointerButton,
        kind: PointerKind,
        evt: PointerEvent,
        menu_open_before: bool,
        eyedropper_armed_before: bool,
    ) -> bool {
        // BgRemoval eyedropper (SHELL-only). A Secondary Down on an
        // extra-colour swatch deletes it; a Primary Down/drag over the
        // sprite samples colours. Both consume the event so the normal
        // canvas/gizmo/context-menu logic below does not run.
        // Fill (Bucket) ColorDrop: a Primary Down on the Fill rail button arms the drag-to-canvas gesture
        // + activates Fill. Self-gates on the hit id; the normal Up-click still selects the tool when the
        // press is released ON the button, and is suppressed when it drags off (release outside the rect).
        if matches!(mapped_button, ph2d_host::PointerButton::Primary)
            && matches!(kind, PointerKind::Down)
        {
            // A Down on the C&F button arms the ColorDrop drag AND consumes the event — otherwise it fell
            // through to `painter_canvas_down` below and the active shape tool dropped a stray point on the
            // canvas behind the button (Enio 2026-07-03). The rail button's own press/click already ran in
            // `forward_to_hero` above; the picker opens on release, Fill activates only if the drag reaches
            // the canvas.
            if self.arm_fill_drag_if_on_button(evt.x, evt.y) {
                return true;
            }
            // A Primary Down on the Fill modal's title band starts a modal-move (the card follows the
            // cursor via CursorMoved) — consume it so it doesn't click through / start anything else.
            if self.arm_input_map_drag_if_on_handle(evt.x, evt.y) {
                return true;
            }
            if self.arm_fill_modal_drag_if_on_handle(evt.x, evt.y) {
                return true;
            }
            // A Primary Down on the onion modal's title band starts a modal-move (ADR-0142 W3b).
            if self.arm_onion_modal_drag_if_on_handle(evt.x, evt.y) {
                return true;
            }
        }
        match (mapped_button, kind) {
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.try_eyedropper_delete(evt.x, evt.y) =>
            {
                return true;
            }
            // Protection brush ERASE: a Secondary Down with the brush armed
            // erases the first dab + starts an erase drag (continued in
            // CursorMoved). Consumes so it doesn't open a context menu.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.try_protect_erase(evt.x, evt.y) =>
            {
                return true;
            }
            // Painter Falloff curve: right-click a control point → open the
            // handle-type menu (Vector / Auto). No-op off a point.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_falloff_open_point_menu(evt.x, evt.y) =>
            {
                return true;
            }
            // On-canvas motion-path anchor: right-click a trajectory node → open the
            // handle-type menu (Corner / Smooth / Symmetric, ADR-0141). No-op off an anchor.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.motion_path_open_anchor_menu(evt.x, evt.y) =>
            {
                return true;
            }
            // On-canvas Curve / Free Hand: right-click a control point → open the
            // handle-kind menu (Free / Aligned / Vector / Auto). No-op off a point.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_curve_open_point_menu(evt.x, evt.y) =>
            {
                return true;
            }
            // On-canvas Line polyline: right-click ENDS point-creation (Blender/CAD
            // convention). No-op when no Line is being drawn (falls through to the
            // context menu).
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if self.painter_line_finish_points() =>
            {
                return true;
            }
            // Grid Stamp: o botão direito APAGA a célula (o esquerdo pinta). É o ÚLTIMO reivindicante do
            // secundário de propósito — todos os arms acima são de outros métodos/ferramentas, e este só
            // diz sim no Grid Stamp, então nenhum right-click que já funcionava é tirado de ninguém; o
            // que não for dele continua caindo no menu de contexto abaixo.
            (ph2d_host::PointerButton::Secondary, PointerKind::Down)
                if !menu_open_before && self.painter_grid_erase_down(evt.x, evt.y) =>
            {
                return true;
            }
            (ph2d_host::PointerButton::Secondary, PointerKind::Up) => {
                // End any erase drag (no-op when not erasing).
                self.end_protect_paint();
                // Fecha um gesto de apagar do Grid Stamp (no-op quando não há traço aberto).
                self.painter_grid_erase_up();
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_eyedropper_sample(evt.x, evt.y) =>
            {
                self.eyedropper_dragging = true;
                return true;
            }
            // Painter Falloff curve: left-click the empty graph (Custom preset) →
            // add a control point where clicked. A press on a handle falls through
            // (the panel's drag dispatch grabs it); a click on an open context menu
            // is the menu's, not a canvas-add (`menu_open_before`).
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.painter_falloff_canvas_add(evt.x, evt.y) =>
            {
                return true;
            }
            // Protection brush: a Primary Down with the brush armed paints
            // the first dab + starts the drag (drag continues in
            // CursorMoved). Consumes the event so it doesn't pick/move the
            // sprite.
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_protect_paint(evt.x, evt.y) =>
            {
                return true;
            }
            // "Add area" automatic selector: a Primary Down with the
            // selector armed runs a single-click flood-fill from the
            // clicked source pixel into the force-remove mask
            // (Enio 2026-05-26). Mirror of the eyedropper sample dispatch.
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.try_add_area_click(evt.x, evt.y) =>
            {
                return true;
            }
            // Colour-picker eyedropper sample: when the picker eyedropper was armed, `forward_to_hero`
            // already sampled the pixel — consume the click so the Painter brush does NOT paint there
            // and the sprite isn't picked/moved. Must precede the painter brush arm below.
            (ph2d_host::PointerButton::Primary, PointerKind::Down) if eyedropper_armed_before => {
                return true;
            }
            // Painter brush: a Primary Down with the Painter active + a sprite
            // selected, inside the footprint, starts a stroke (the first dab) and
            // arms the drag (continues in CursorMoved). Consumes the event so it
            // doesn't pick / move the sprite. A click on an open modal / context
            // menu is the menu's (`menu_open_before`) — never a stroke on the
            // canvas below it (Enio 2026-06-24: new-image modal leaked a dab).
            (ph2d_host::PointerButton::Primary, PointerKind::Down)
                if !menu_open_before && self.painter_canvas_down(evt.x, evt.y, evt.pressure) =>
            {
                return true;
            }
            (ph2d_host::PointerButton::Primary, PointerKind::Up) => {
                self.eyedropper_dragging = false;
                self.end_protect_paint();
                // End a Falloff add-drag (no-op when not dragging).
                self.painter_falloff_release();
                // Close an open painter brush stroke (no-op when not painting).
                self.painter_canvas_up();
                // Finish a Fill ColorDrop drag (fill on the canvas, or open the picker for a plain click
                // on the Fill button). No-op when no fill drag is armed.
                self.fill_drag_up();
                // End a Fill "Fill adjust" modal title-band drag. No-op when not dragging the modal.
                self.input_map_drag_up();
                self.fill_modal_drag_up();
                // End an onion settings modal title-band drag (ADR-0142 W3b). No-op when not dragging.
                self.onion_modal_drag_up();
            }
            _ => {}
        }
        false
    }
}
