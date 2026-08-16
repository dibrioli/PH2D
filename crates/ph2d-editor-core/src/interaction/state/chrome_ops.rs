//! Editor chrome state on [`WidgetStore`] — clipboard outbox, scene
//! name, tool toggles, tooltips, collapsed flags, context menu,
//! section outlines, notes, picker target, widget colors,
//! radius scale.
//!
//! Extracted from [`super`] (Track D6). All these are short
//! key/value mutators that don't fit the Blender / panel / widget-
//! accessor / drag clusters but share the "side-table on WidgetStore"
//! pattern.

use super::WidgetStore;
use crate::interaction::ContextMenuRequest;
use crate::interaction::InteractiveState;
use crate::interaction::types::NoteData;
use crate::widget::{ButtonState, SliderOrientation, SliderState};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// Drain the pending clipboard-copy text (set by a Cmd+C / Cmd+X
    /// dispatch); shell should call once per frame and write to OS
    /// clipboard when non-None.
    pub fn take_clipboard_copy(&mut self) -> Option<String> {
        self.pending_clipboard_copy.take()
    }

    pub fn set_clipboard_copy(&mut self, text: String) {
        self.pending_clipboard_copy = Some(text);
    }

    /// Drain the pending clipboard-paste request; shell should read
    /// the OS clipboard and call `apply_clipboard_paste` with the
    /// result when non-None.
    pub fn take_clipboard_paste_request(&mut self) -> Option<NodeId> {
        self.pending_clipboard_paste.take()
    }

    pub fn set_clipboard_paste_request(&mut self, id: NodeId) {
        self.pending_clipboard_paste = Some(id);
    }

    pub fn current_scene_name(&self) -> &str {
        &self.current_scene_name
    }

    pub fn set_current_scene_name(&mut self, name: impl Into<String>) {
        self.current_scene_name = name.into();
    }

    pub fn tool_space_local(&self) -> bool {
        self.tool_space_local
    }

    pub fn set_tool_space_local(&mut self, local: bool) {
        self.tool_space_local = local;
    }

    pub fn tool_view_mode(&self) -> u8 {
        self.tool_view_mode
    }

    pub fn set_tool_view_mode(&mut self, mode: u8) {
        self.tool_view_mode = mode % 3;
    }

    /// `true` while the Painter rail's **Shapes** flyout is open (the
    /// column of shape chips to the right of the Shapes button).
    pub fn painter_shapes_flyout_open(&self) -> bool {
        self.painter_shapes_flyout_open
    }

    /// Open / close the Painter rail's Shapes flyout.
    pub fn set_painter_shapes_flyout_open(&mut self, open: bool) {
        self.painter_shapes_flyout_open = open;
    }

    /// `true` while the Painter rail's **Mask** group flyout is open (the column
    /// of Mask + Selection chips to the right of the Mask button).
    pub fn painter_mask_flyout_open(&self) -> bool {
        self.painter_mask_flyout_open
    }

    /// Open / close the Painter rail's Mask group flyout.
    pub fn set_painter_mask_flyout_open(&mut self, open: bool) {
        self.painter_mask_flyout_open = open;
    }

    /// Register a tooltip string for `id`. Called by `populate`
    /// passes or directly inside painters; read by the hover-tooltip
    /// pass via [`Self::tooltip_for`]. Empty strings are treated as
    /// no-tooltip and removed from the table.
    pub fn set_tooltip(&mut self, id: NodeId, text: impl Into<String>) {
        let s = text.into();
        if s.is_empty() {
            self.tooltips.remove(&id);
        } else {
            self.tooltips.insert(id, s);
        }
    }

    /// Lookup the tooltip for a widget id. Returns `None` if the id
    /// has no registered tooltip.
    pub fn tooltip_for(&self, id: NodeId) -> Option<&str> {
        self.tooltips.get(&id).map(|s| s.as_str())
    }

    /// `true` iff the section/panel at `id` is currently collapsed.
    /// Missing entries default to expanded — newly-registered
    /// sections start open without any setup.
    pub fn is_collapsed(&self, id: NodeId) -> bool {
        self.collapsed.get(&id).copied().unwrap_or(false)
    }

    /// Set the collapsed state for a section/panel. `true` collapses,
    /// `false` expands.
    pub fn set_collapsed(&mut self, id: NodeId, collapsed: bool) {
        self.collapsed.insert(id, collapsed);
    }

    /// Flip the collapsed state for `id`. Convenience for click
    /// handlers — equivalent to
    /// `set_collapsed(id, !is_collapsed(id))`.
    pub fn toggle_collapsed(&mut self, id: NodeId) {
        let was = self.is_collapsed(id);
        // ⚠️ **A PARTIDA é gravada ANTES de o estado virar, e é isso que faz a PRIMEIRA dobra de
        //    cada secção animar.** A lei do substrato é que a *primeira vista de um id CHEGA ao
        //    alvo*; sem esta linha o relógio vê a secção pela primeira vez já no destino e a
        //    estreia de cada dobra SALTA — todas as seguintes animariam, e o defeito seria uma
        //    vez por secção por sessão, que é a forma mais fácil de ninguém reproduzir.
        //
        // ⚠️ `or_insert` e não `insert`: com uma dobra EM VOO o valor publicado é a verdade, e
        //    re-clicar a meio tem de RETOMAR de onde ela está — é a interruptibilidade que o
        //    substrato promete, e escrever por cima dela devolveria a secção ao extremo.
        self.fold_live
            .entry(id)
            .or_insert(if was { 0.0 } else { 1.0 });
        self.collapsed.insert(id, !was);
    }

    /// **Quanto desta secção está ABERTO** (`0.0` fechada … `1.0` aberta) — o número que o
    /// chevron veste.
    ///
    /// ⚠️ **O neutro é o BINÁRIO de hoje**, não zero: uma secção que o relógio nunca viu devolve
    /// exatamente `1.0`/`0.0` conforme [`Self::is_collapsed`], então um store não-tiquado — e um
    /// pintor ainda não migrado — desenha **byte a byte** o que desenhava antes desta wave. É a
    /// mesma neutralidade do `hover_live`, e é ela que torna a migração dos ~34 sítios segura de
    /// fazer aos poucos: um sítio esquecido fica com a troca binária, nunca meio-animado.
    #[must_use]
    pub fn section_open_live(&self, id: NodeId) -> f32 {
        self.fold_live
            .get(&id)
            .copied()
            .unwrap_or(if self.is_collapsed(id) { 0.0 } else { 1.0 })
    }

    /// **O tique publica aqui.** Único escritor — o gêmeo exacto do
    /// [`super::WidgetStore::set_hover_live`] e do `set_panel_scroll_live`.
    pub fn set_section_open_live(&mut self, id: NodeId, t: f32) {
        self.fold_live.insert(id, t);
    }

    /// **Quão alto o CORPO desta secção mediu da última vez que foi pintado**, ou `None` se ela
    /// nunca foi pintada aberta. Alimenta o RECORTE da dobra e mais nada — ver
    /// [`super::WidgetStore::fold_body_h`] para o porquê de ser um valor lembrado e não medido
    /// no próprio quadro.
    #[must_use]
    pub fn section_body_h(&self, id: NodeId) -> Option<f32> {
        self.fold_body_h.borrow().get(&id).copied()
    }

    /// **O pintor publica o que acabou de medir.** ⚠️ `&self` de propósito — ver
    /// [`super::WidgetStore::fold_body_h`].
    pub fn remember_section_body_h(&self, id: NodeId, h: f32) {
        self.fold_body_h.borrow_mut().insert(id, h.max(0.0));
    }

    /// As secções que o tique tem de animar: **as que o utilizador tocou**.
    ///
    /// ⚠️ Uma secção nunca tocada não está no mapa e **não custa nada** — ela está aberta, o
    /// neutro devolve `1.0`, e não há nada a integrar. Uma que nasce fechada no `populate`
    /// (`set_collapsed(id, true)` no arranque) entra no mapa **sem partida gravada**, então o
    /// alvo e o neutro coincidem e ela assenta sem animar: *nascer fechada não é dobrar-se*.
    pub fn collapse_states(&self) -> impl Iterator<Item = (NodeId, bool)> + '_ {
        self.collapsed.iter().map(|(k, v)| (*k, *v))
    }

    /// Open a right-click context menu at `(x, y)` with the given
    /// `kind`. Replaces any previously-open menu (only one menu
    /// can be visible at a time).
    pub fn open_context_menu(&mut self, request: ContextMenuRequest) {
        self.context_menu = Some(request);
    }

    /// Open the New-image modal (Cmd/Ctrl+N) — centred (its painter ignores `x`/`y`), seeded with the
    /// remembered size/background selection.
    pub fn open_new_image_dialog(&mut self) {
        self.context_menu = Some(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: crate::interaction::ContextMenuKind::NewImageDialog,
        });
    }

    /// Selected square size (px) for the New-image modal.
    pub fn new_image_size(&self) -> u32 {
        self.new_image_size
    }

    /// Set the New-image modal's selected square size (px).
    pub fn set_new_image_size(&mut self, px: u32) {
        self.new_image_size = px;
    }

    /// Selected background for the New-image modal (`0` transparent / `1` black / `2` white).
    pub fn new_image_bg(&self) -> u8 {
        self.new_image_bg
    }

    /// Set the New-image modal's selected background (`0` transparent / `1` black / `2` white).
    pub fn set_new_image_bg(&mut self, bg: u8) {
        self.new_image_bg = bg;
    }

    /// Commit the modal's current `(size, bg)` as a pending new-image request (the shell polls + clears
    /// it via [`Self::take_new_image_request`]) and close the modal.
    pub fn request_new_image(&mut self) {
        self.new_image_request = Some((self.new_image_size, self.new_image_bg));
        self.close_context_menu();
    }

    /// Take (and clear) the pending new-image request. The shell polls this each frame; on `Some` it
    /// spawns a blank canvas of that size + background.
    pub fn take_new_image_request(&mut self) -> Option<(u32, u8)> {
        self.new_image_request.take()
    }

    /// Open the Fill (Bucket) "Fill adjust" floating modal with its top-left at `(x, y)` (screen px),
    /// seeding the threshold slider to `threshold` (`0..1`). Registers the slider + Done/Cancel/handle
    /// widgets so the generic pointer dispatch can drive them (idempotent — the slider VALUE is refreshed
    /// on every open so it mirrors the tool's current threshold). The shell calls this on the ColorDrop
    /// release; the painter then re-fills live as the slider moves.
    pub fn open_fill_modal(&mut self, x: f32, y: f32, threshold: f32) {
        // ⚠️ A âncora viaja NO MESMO campo que a posição, e não ao lado dela: assim é
        // estruturalmente impossível ter uma sem a outra, ou fechar o modal e deixar a âncora viva.
        // O `move_fill_modal` toca só a posição — a âncora é onde a coisa NASCEU e não se move.
        self.fill_modal = Some(((x, y), (x, y)));
        self.register(
            crate::ids::PAINTER_FILL_MODAL_SLIDER,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: threshold.clamp(0.0, 1.0),
                orientation: SliderOrientation::Horizontal,
            },
        );
        // Re-seed the value even if the slider was already registered (`register` overwrites).
        if let Some(InteractiveState::Slider { value, .. }) =
            self.get_mut(crate::ids::PAINTER_FILL_MODAL_SLIDER)
        {
            *value = threshold.clamp(0.0, 1.0);
        }
        for id in [
            crate::ids::PAINTER_FILL_MODAL_DONE,
            crate::ids::PAINTER_FILL_MODAL_CANCEL,
            crate::ids::PAINTER_FILL_MODAL_HANDLE,
        ] {
            self.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }

    /// Close the Fill (Bucket) "Fill adjust" modal (Done / Cancel / a canvas dismissal).
    pub fn close_fill_modal(&mut self) {
        self.fill_modal = None;
    }

    /// Open the **Onion settings** floating modal (ADR-0142 W3b) with its top-left at `(x, y)` (screen
    /// px), seeding the three sliders (`0..1`) and two colour swatches. Registers the sliders, the
    /// close and handle buttons, and marks the swatches as picker swatches so the generic pointer
    /// dispatch drives them. Takes ONLY primitives — editor-core stays timeline-agnostic; the shell
    /// owns the count↔slider mapping and the `OnionSettings` type. Idempotent: re-opening re-seeds
    /// every value.
    ///
    /// `opacity` / `before_frac` / `after_frac` are the slider tracks (`0..1`); `color_before` /
    /// `color_after` are `[u8; 4]` RGBA seeds for the swatches.
    // The eight primitives ARE the design (see above): bundling them would mean either a second
    // shape of the same data living here, or `OnionSettings` itself — and the whole point is that
    // editor-core never learns what a timeline is. Same stance as the other seeder/painter entry
    // points in this crate.
    #[allow(clippy::too_many_arguments)]
    pub fn open_onion_modal(
        &mut self,
        x: f32,
        y: f32,
        opacity: f32,
        before_frac: f32,
        after_frac: f32,
        color_before: [u8; 4],
        color_after: [u8; 4],
    ) {
        self.onion_modal = Some((x, y));
        for (id, v) in [
            (crate::ids::TIMELINE_ONION_MODAL_OPACITY, opacity),
            (crate::ids::TIMELINE_ONION_MODAL_BEFORE, before_frac),
            (crate::ids::TIMELINE_ONION_MODAL_AFTER, after_frac),
        ] {
            self.register(
                id,
                InteractiveState::Slider {
                    state: SliderState::Normal,
                    value: v.clamp(0.0, 1.0),
                    orientation: SliderOrientation::Horizontal,
                },
            );
            // Re-seed even if already registered (`register` overwrites, but be explicit — the value
            // must mirror the current onion every time the card is (re)opened).
            if let Some(InteractiveState::Slider { value, .. }) = self.get_mut(id) {
                *value = v.clamp(0.0, 1.0);
            }
        }
        for id in [
            crate::ids::TIMELINE_ONION_MODAL_CLOSE,
            crate::ids::TIMELINE_ONION_MODAL_HANDLE,
        ] {
            self.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        for (id, rgba) in [
            (crate::ids::TIMELINE_ONION_MODAL_COLOR_BEFORE, color_before),
            (crate::ids::TIMELINE_ONION_MODAL_COLOR_AFTER, color_after),
        ] {
            self.register_picker_swatch(id);
            self.set_widget_color(id, rgba);
        }
    }

    /// Close the Onion settings modal (the X button / a dismissal).
    pub fn close_onion_modal(&mut self) {
        self.onion_modal = None;
    }

    /// The Onion modal's top-left `(x, y)` in screen px, or `None` when closed. The painter gates the
    /// card's render + hit registration on this being `Some`; the shell gates the store→onion read-back
    /// on it too.
    #[must_use]
    pub fn onion_modal_pos(&self) -> Option<(f32, f32)> {
        self.onion_modal
    }

    /// Offset the Onion modal's position by `(dx, dy)` screen px (the title-band drag). No-op when
    /// closed. Not clamped here — the painter clamps to the viewport when it draws (so the accumulated
    /// delta always equals `cursor − grab_offset` and the drag never dead-zones), mirroring
    /// [`Self::move_fill_modal`].
    pub fn move_onion_modal(&mut self, dx: f32, dy: f32) {
        if let Some((x, y)) = self.onion_modal.as_mut() {
            *x += dx;
            *y += dy;
        }
    }

    /// Open the full-screen command palette with `model` (Motion's "Add Node"). Clears any stale pick so
    /// the shell's read-back only ever sees a fresh choice. The model is set ONCE here — never rebuilt
    /// per frame — mirroring [`Self::open_onion_modal`]'s value-seeding.
    pub fn open_command_palette(&mut self, model: crate::widget::command_palette::PaletteModel) {
        self.command_palette = Some(model);
        self.command_pick = None;
        self.command_palette_query.clear();
    }

    /// Close the command palette (the close-X, a click on the dimmed scrim, or Esc). Leaves any pending
    /// pick alone — the shell drains it via [`Self::take_command_pick`] on the same frame it routes.
    pub fn close_command_palette(&mut self) {
        self.command_palette = None;
        self.command_palette_query.clear();
    }

    /// The live search text typed into the open palette (empty = no filter). The widget reads this to
    /// filter the model + highlight; the shell reads it to compute the `Enter` top-match.
    #[must_use]
    pub fn command_palette_query(&self) -> &str {
        &self.command_palette_query
    }

    /// Append a printable character to the palette's search text (a full-screen modal captures the key).
    /// Control characters are ignored — the shell filters them, but this is the single door, so it does
    /// too. No-op when the palette is closed (nothing to type into).
    pub fn command_palette_push_char(&mut self, c: char) {
        if self.command_palette.is_some() && !c.is_control() {
            self.command_palette_query.push(c);
        }
    }

    /// Drop the last character of the palette's search text (Backspace). No-op when closed or empty.
    pub fn command_palette_backspace(&mut self) {
        if self.command_palette.is_some() {
            self.command_palette_query.pop();
        }
    }

    /// The open palette's model, or `None` when closed. The painter gates the whole full-screen render +
    /// hit registration on this being `Some`.
    #[must_use]
    pub fn command_palette_model(&self) -> Option<&crate::widget::command_palette::PaletteModel> {
        self.command_palette.as_ref()
    }

    /// Is the command palette open? (Convenience for the shell's input gating.)
    #[must_use]
    pub fn command_palette_open(&self) -> bool {
        self.command_palette.is_some()
    }

    /// Record the item id the user picked (the chrome handler calls this on an item click). The shell
    /// reads it back with [`Self::take_command_pick`] and maps it to a real action.
    pub fn set_command_pick(&mut self, id: ph2d_a11y::NodeId) {
        self.command_pick = Some(id);
    }

    /// Take the pending command-palette pick, clearing it (so a pick is routed exactly once). `None` when
    /// nothing was picked since the last take.
    ///
    /// ⚠️ **Com DOIS consumidores no canal, prefira [`Self::take_command_pick_if`].** Um take
    /// incondicional é o que torna a ORDEM dos drenos load-bearing: quem corre primeiro engole o
    /// pick do outro, e o sintoma é um comando que não faz nada de vez em quando.
    #[must_use]
    pub fn take_command_pick(&mut self) -> Option<ph2d_a11y::NodeId> {
        self.command_pick.take()
    }

    /// **Toma o pick só se ele for MEU** — a porta de um canal com mais de um consumidor.
    ///
    /// Há duas paletas neste app (a de nós do Motion e a global do chrome) e **um** canal de pick.
    /// Com dois `take` incondicionais, qual dos dois recebe o pick passa a ser a ordem em que os
    /// drenos correm — um facto invisível, que muda quando alguém reordena o frame. Com os dois a
    /// perguntar *«este id é meu?»* a ordem deixa de importar: cada um reconhece os próprios ids e
    /// deixa o resto onde está.
    ///
    /// `pred` devolve `true` quando o id pertence a este consumidor.
    #[must_use]
    pub fn take_command_pick_if(
        &mut self,
        pred: impl FnOnce(ph2d_a11y::NodeId) -> bool,
    ) -> Option<ph2d_a11y::NodeId> {
        match self.command_pick {
            Some(id) if pred(id) => self.command_pick.take(),
            _ => None,
        }
    }

    /// The Fill modal's top-left `(x, y)` in screen px, or `None` when the modal is closed. The painter
    /// gates the modal's render + hit registration on this being `Some`.
    #[must_use]
    pub fn fill_modal_pos(&self) -> Option<(f32, f32)> {
        self.fill_modal.map(|(pos, _)| pos)
    }

    /// Onde o modal NASCEU (o ponto de largada do ColorDrop) — a outra ponta da corda. `None` com o
    /// modal fechado, pela mesma `Option`: as duas metades vivem no mesmo campo.
    #[must_use]
    pub fn fill_modal_anchor(&self) -> Option<(f32, f32)> {
        self.fill_modal.map(|(_, anchor)| anchor)
    }

    /// Offset the Fill modal's position by `(dx, dy)` screen px (the title-band drag). No-op when the
    /// modal is closed. Not clamped here — the painter clamps the position to the viewport when it draws
    /// (so the accumulated delta always equals `cursor − grab_offset` and the drag never dead-zones).
    pub fn move_fill_modal(&mut self, dx: f32, dy: f32) {
        if let Some(((x, y), _)) = self.fill_modal.as_mut() {
            *x += dx;
            *y += dy;
        }
    }

    /// Close any currently-open context menu. Snapshots the request
    /// into `last_context_menu` so `apply_event` can still read the
    /// menu's `kind` when handling the item-click that triggered
    /// the close (the click → Click event arrives AFTER the close).
    pub fn close_context_menu(&mut self) {
        if let Some(req) = self.context_menu.take() {
            self.last_context_menu = Some(req);
        }
    }

    /// Read the currently-open context menu request, if any.
    pub fn context_menu(&self) -> Option<ContextMenuRequest> {
        self.context_menu
    }

    /// Read the most recently closed context-menu request — used by
    /// `apply_event` to recover the original `kind` when routing an
    /// item Click into a side-table mutation. Cleared by
    /// `consume_last_context_menu` once the click has been applied.
    pub fn last_context_menu(&self) -> Option<ContextMenuRequest> {
        self.last_context_menu
    }

    /// Take + clear the last-context-menu snapshot. Called by
    /// `apply_event` after it has routed the click.
    pub fn consume_last_context_menu(&mut self) -> Option<ContextMenuRequest> {
        self.last_context_menu.take()
    }

    /// Read the outline-color index for a section header id (0..4
    /// referencing the highlighter palette). `None` = no outline.
    pub fn section_outline_color(&self, section: NodeId) -> Option<u8> {
        self.section_outline_color.get(&section).copied()
    }

    /// Set the outline-color index for a section. Pass `None` to
    /// clear (the "No outline" menu item).
    pub fn set_section_outline_color(&mut self, section: NodeId, color: Option<u8>) {
        match color {
            Some(c) => {
                self.section_outline_color.insert(section, c);
            }
            None => {
                self.section_outline_color.remove(&section);
            }
        }
    }

    /// Read the per-panel note list. Returns an empty slice when no
    /// notes have been created for the panel.
    pub fn notes_for_panel(&self, panel: NodeId) -> &[NoteData] {
        self.notes_per_panel
            .get(&panel)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Append a new sticky-note to the panel's list. Caller passes the
    /// highlighter color index + the optional section-anchor to the
    /// panel's note list. Cap at 12 notes per panel
    /// to keep paint bounded. `before_section: Some(i)` makes the
    /// painter slot the note immediately above `SECTION_IDS[i]`;
    /// `None` appends at the bottom.
    pub fn notes_push(&mut self, panel: NodeId, color_idx: u8, before_section: Option<u8>) {
        const CAP: usize = 12;
        let list = self.notes_per_panel.entry(panel).or_default();
        if list.len() >= CAP {
            return;
        }
        list.push(NoteData {
            color_idx,
            title: format!("Note {}", list.len() + 1),
            body: String::new(),
            before_section,
        });
    }

    /// Update an existing note's color index.
    pub fn note_set_color(&mut self, panel: NodeId, index: usize, color_idx: u8) {
        if let Some(list) = self.notes_per_panel.get_mut(&panel)
            && let Some(note) = list.get_mut(index)
        {
            note.color_idx = color_idx.min(4);
        }
    }

    /// Currently-active color picker target. The floating
    /// BlenderColorPicker is hidden when this is `None`.
    pub fn picker_target(&self) -> Option<NodeId> {
        self.picker_target
    }

    /// Open the picker editing the color at `target`. Pass `None`
    /// to hide the picker. The caller is responsible for seeding
    /// `widget_colors[target]` if it doesn't yet have a value.
    pub fn set_picker_target(&mut self, target: Option<NodeId>) {
        self.picker_target = target;
        // Opening the picker → it's the most recently summoned panel.
        if target.is_some() {
            // Picker is keyed by INSP_BLENDER_PICKER in the z-order
            // (canonical floating-panel id). Hard-coded here rather
            // than re-exporting the screens::hero::ids const into the
            // interaction crate, since the picker is the single
            // floating panel for the whole editor.
            const INSP_BLENDER_PICKER: NodeId = NodeId(380);
            self.bump_panel_z(INSP_BLENDER_PICKER);
        } else {
            // Dismissing the picker drops its transient dropdown popover so it
            // doesn't linger (or flash) the next time the picker is summoned.
            self.palette_dropdown_open = None;
        }
    }

    /// Read the current color of a color-target widget. Returns
    /// `None` when no color has been stored for `id`. Default
    /// colors are seeded lazily by callers (e.g. `apply_event`
    /// inserts a neutral gray when first opening the picker).
    pub fn widget_color(&self, id: NodeId) -> Option<[u8; 4]> {
        self.widget_colors.get(&id).copied()
    }

    /// Set the current color of a color-target widget. Called by
    /// the picker each frame to mirror its edits into the
    /// target's stored color, and by `apply_event` to seed the
    /// default when the picker first opens for that target.
    pub fn set_widget_color(&mut self, id: NodeId, rgba: [u8; 4]) {
        self.widget_colors.insert(id, rgba);
    }

    /// Editor-wide corner-radius scale (1.0 = canonical, 0.0 =
    /// sharp). Painters that want to honor the user's theme preset
    /// multiply their `Radius::*.px()` by this factor.
    pub fn radius_scale(&self) -> f32 {
        self.radius_scale
    }

    pub fn set_radius_scale(&mut self, scale: f32) {
        self.radius_scale = scale.max(0.0);
    }

    /// Current rail-button size preset (Small / Medium / Large) — set
    /// from the Themes menu (2026-05-24).
    pub fn rail_button_size(&self) -> crate::widget::RailButtonSize {
        self.rail_button_size
    }

    pub fn set_rail_button_size(&mut self, size: crate::widget::RailButtonSize) {
        self.rail_button_size = size;
    }

    /// Cached VSync flag the shell last applied. Mirrors the present
    /// mode so the Settings → Display submenu paint can mark the
    /// active row (the swap chain itself is owned by the shell).
    pub fn present_vsync(&self) -> bool {
        self.present_vsync
    }

    pub fn set_present_vsync(&mut self, vsync: bool) {
        self.present_vsync = vsync;
    }
}
