// O ÍNDICE do despacho da janela: o tecto numerado (7 115 L) SAIU com a `line/input-dispatch` (2026-09-13) — os corpos
// grandes são ramos em `input_dispatch/despacho_*.rs`, chamados PELA MESMA ORDEM, e os ajudantes e testes moram em
// irmãos. Um gate que leia o despacho como texto lê o `tests/it/input_text.rs::dispatch()`, nunca este ficheiro.
//! Window-event dispatch — one method per `WindowEvent` variant.
//!
//! PR 9b of `docs/Migracao/2026-05-convention-by-discovery.md`:
//! `window_event()` in `main.rs` used to inline ~700 LOC across 13
//! `WindowEvent` arms, with single arms (CursorMoved 166 LOC,
//! MouseInput 325 LOC, KeyboardInput 83 LOC) violating HR-18's 200-LOC
//! per-function cap. This module hosts each arm as a `pub(crate) fn
//! on_<arm>(&mut self, …)` method on `App` — bodies are verbatim
//! former arms (no behaviour change), so smoke parity is
//! byte-for-byte.
//!
//! `window_event()` in `main.rs` becomes a 13-line dispatch table.
//! Adding a new arm: one method here + one line in the table.
//!
//! Rust allows `impl App` to be split across files within the same
//! crate as long as both files are reachable via `mod`. `App` is
//! `private` to `main.rs`, but submodules see their parent's private
//! items — so this `impl` block compiles without exposing any field
//! visibility upstream.

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;

use ph2d_editor_core::Toast;
use ph2d_host::{
    CloseAction, HostHandler, Lifecycle, PlatformHost, PointerEvent, PointerKind, WindowSize,
};

use crate::App;
use crate::Transform;
use crate::forwarding::{
    cursor_over_hero_panel, forward_blur_to_hero, forward_text_to_hero, forward_to_hero,
    forward_wheel_to_hero, resolve_live_entry,
};
use ph2d_tool_vector::params::MarqueeShape;

// `impl App` is split across sibling modules (see the eyedropper /
// keyboard handlers) to keep this file under the HR-18 LOC cap.
mod eyedropper;
pub(crate) mod fill_drag;
mod gizmo_drag;
mod keyboard;
/// **A escuta do Input Map no topo do teclado** — irmão de [`keyboard`], cortado por teto de LOC.
mod keyboard_bind_capture;
mod keyboard_board; // o teclado de um QUADRO activo (MiroClone) -- antes dos atalhos da cena
mod trackpad_board; // W23 -- os dois dedos e a pinça num QUADRO activo

/// A tela «Controls» do jogador no Play (InputMap §4).
pub(crate) mod controles_do_jogador;
/// **As teclas que ENCERRAM um gesto em curso** (Esc cancela, Enter confirma) — irmão do
/// `keyboard`, cortado dele pelo cap de LOC. A ORDEM entre elas é a lei, e é por isso que
/// viajam juntas em vez de por dono.
mod keyboard_escapes;
/// ⭐⭐⭐ **Que MODO é dono do teclado neste quadro** — irmão do `keyboard`, cortado dele pelo cap
/// de LOC. *Um modo em curso é dono da entrada dele.*
mod keyboard_modal;
mod keyboard_palette; // as teclas do palette de nos -- MODAL, ver o doc do modulo
pub(crate) mod posse_do_jogo; // de quem e o teclado e o rato com a cena a correr (CodeEditor item 7)

// Os ramos e os irmãos do despacho: cada `despacho_*.rs` diz no próprio `//!` o que guarda.
mod despacho_clique_flip;
mod despacho_clique_gizmo;
mod despacho_clique_hud;
mod despacho_clique_largar;
mod despacho_clique_osso;
mod despacho_clique_pick;
mod despacho_clique_prologo;
mod despacho_clique_reclamantes;
mod despacho_clique_roldana;
mod despacho_clique_select;
mod despacho_clique_vetor;
mod despacho_clique_vetor_premido;
mod despacho_clique_vetor_solto;
mod despacho_constraints;
mod despacho_cursor_sobre;
mod despacho_metodos_janela_e_vetor;
mod despacho_metodos_modos_e_alcas;
mod despacho_metodos_picks_e_arrastos;
mod despacho_mover;
mod despacho_peso_do_osso;
mod despacho_pixel;
mod despacho_vetor_alinhar_e_forma;
mod despacho_vetor_gradiente;
mod despacho_vetor_ops;
/// **Os acordes de ARQUIVO** — irmão do `keyboard`, cortado dele pelo cap de LOC.
mod keyboard_files;
mod keyboard_hierarchy; // Delete/Ctrl+D sobre a linha da Hierarquia -- ver o doc do modulo
mod keyboard_painter; // a cadeia do Delete do Painter: ancora -> figura -> falloff
mod keyboard_timeline;
#[cfg(test)]
mod o_pivo_da_alca_tests;
pub(crate) mod painter_canvas_input;
mod painter_canvas_keys;
mod painter_canvas_mods;
/// ⭐ O menu de alça de um ponto da CURVA no canvas — irmão do `painter_canvas_input` pelo teto
/// de LOC, cortado por assunto.
mod painter_curve_input;
pub(crate) mod painter_falloff_input;
mod painter_grid_erase; // os modificadores que o CanvasPointer nao carrega
pub(crate) mod protect_brush;
use despacho_clique_roldana::select_wheel_at;
pub(crate) use despacho_vetor_alinhar_e_forma::{
    VecAlign, VecDistribute, VecPathShapeOp, apply_vec_align, apply_vec_distribute,
    apply_vec_path_shape, vec_align_for_id, vec_distribute_for_id, vec_path_shape_for_id,
    vec_transform_field_for_id,
};
use despacho_vetor_alinhar_e_forma::{
    gizmo_anchor_half, shape_constraint, shape_kind_for_mode, shape_up_consumes,
};
pub(crate) use despacho_vetor_gradiente::{
    apply_vec_grad_add_point, apply_vec_grad_add_stop, apply_vec_grad_influence,
    apply_vec_grad_jitter, apply_vec_grad_remove_point, apply_vec_grad_remove_stop,
    apply_vec_set_fill_kind, apply_vec_set_grad_angle,
};
pub(crate) use despacho_vetor_ops::{
    VecFillKind, apply_vec_delete_vertex, apply_vec_duplicate, apply_vec_fill_rule,
    apply_vec_toggle_closed, apply_vec_vertex_kind, duplicate_vec_paths, screen_offset_world,
    vec_bool_op_for_id, vec_fill_kind_for_id, vec_flip_for_id, vec_reorder_for_id,
    vec_rotate_for_id, vec_vertex_kind_for_id,
};
pub(crate) use ph2d_vec_entities::campos_transform::{
    VecTransformField, apply_vec_rotate_by, apply_vec_transform,
};

// Só os `cursor_tests` o leem por `super::` — o `timeline_resize_cursor` já o tem no próprio ficheiro.
#[cfg(test)]
use despacho_metodos_janela_e_vetor::resize_cursor_for_edges;
// Só os testes do eixo espectral o leem por `super::` — os métodos de áudio já o têm no próprio ficheiro.
#[cfg(all(test, feature = "panel-audio-editor"))]
use despacho_metodos_picks_e_arrastos::freq_at_y;

/// Deslocamento diagonal de um paste/duplicate, em pixels de tela (o zoom converte
/// para world) — a cópia não nasce exatamente sob o original.
///
/// ⚠️ **Em pixels de TELA, e é isso que o torna a resposta certa a *"onde nasce uma cópia?"***:
/// ele lê o mesmo em qualquer zoom. O `Place` de instância (plano UI/UX W5) nasceu com um número
/// próprio em unidades de MUNDO, e a conta que isso deu está escrita no
/// [`crate::vec_component_edit::cascade_offset`].
pub(crate) const PASTE_OFFSET_PX: f64 = 12.0;

/// O raio de acerto de uma alça de canvas, em **pixels de tela**. O MESMO número que as
/// ferramentas de quina usam para agarrar um vértice — uma alça é uma alça, e dois raios
/// diferentes fariam o artista aprender duas mãos.
const HANDLE_HIT_PX: f64 = 12.0;

impl App {
    /// ⭐ A porta ÚNICA do movimento: o rato do winit e a caneta (`input_drop/caneta.rs`) acabam aqui.
    pub(crate) fn pointer_moved(&mut self, evt: PointerEvent) {
        ph2d_pen::porta::poe_tinta(&evt);
        ph2d_pen::porta::poe_dentro(true);
        // Diagnostics: count every raw winit move (input rate), paired with `paint_stamps_this_frame`
        // in the HUD so the coalescing is visible (high events → 1 stamp).
        self.input_events_this_frame = self.input_events_this_frame.saturating_add(1);
        self.recovery.hover(evt.x, evt.y);
        if self.recovery.is_modal() {
            return; // a janela da recuperação é modal: nem o hover passa por baixo.
        }
        self.last_pointer = (evt.x, evt.y);
        // M14.4e: o `DroppedFile` do winit não traz posição — projecta-se o último cursor visto.
        self.last_cursor = self.last_pointer;
        // A tela «Controls» do jogador aberta é dona do ponteiro (o hover dela); no palco, só os botões do jogo.
        if self.movimento_dos_controles(true) {
            return;
        }
        if self.no_palco() {
            let _ = forward_to_hero(self.gfx.as_mut(), evt);
            return;
        }
        // ⭐ A BORDA da coluna é dona do Move até o Up, ANTES da aba de família (o Down dela não arma o `chrome_press`).
        if self.dock_seam_move(self.last_pointer.0) {
            return;
        }
        // Um arrasto da vista de um QUADRO (MiroClone) em curso é dono do movimento.
        let mods = Self::convert_modifiers(self.modifiers);
        if let Some(g) = self.gfx.as_mut()
            && let Some(hero) = g.hero_screen.as_mut()
            && ph2d_editor_core::screens::hero::board_view::pointer_move(
                hero,
                board_input(&mut g.text_system, mods),
                self.last_pointer.0,
                self.last_pointer.1,
            )
        {
            return;
        }
        if self.ramo_pixel_mover() {
            return;
        }
        if self.ramo_mover_arrastos_de_topo() {
            return;
        }
        if self.ramo_mover_pintura_flip_e_modais() {
            return;
        }
        if self.ramo_mover_vetor() {
            return;
        }
        // M14.4b.bis: middle-drag camera pan. Applied BEFORE pointer
        // forwarding so widgets receive the move event but the camera
        // also follows.
        if let Some(anchor) = self.pan_anchor
            && let Some(gfx) = self.gfx.as_mut()
        {
            let dx = self.last_pointer.0 - anchor.0;
            let dy = self.last_pointer.1 - anchor.1;
            // ⚠️ A janela INTEIRA: é a porta (`pan_scene_camera`) que aplica o split. A varredura
            // de 09/09 (`fbf6a0f30`) pôs aqui a janela da cena e o split passava DUAS vezes
            // (`h·t²`) — apanhado em F6, ao fazer a pinça passar por esta porta.
            let size = gfx.surface.size();
            // ⚠️ **O mundo-por-pixel é o da CENA, não o da JANELA** (report do Enio,
            // 2026-08-25: *«no modo motion a imagem de referência sofre um drift no pan
            // com o mouse»*). Sob o split da tool Motion a cena renderiza num
            // sub-retângulo e a projeção dela MUDA — `pan_screen_delta` com a janela
            // cheia movia o mundo `t` vezes o que o cursor andava, e a imagem ficava
            // para trás do rato. Ver `field_gizmo::pan_scene_camera`.
            let split = gfx
                .hero_screen
                .as_ref()
                .map_or(ph2d_editor_core::screens::layout::CenterSplit::None, |h| {
                    h.view.center_split
                });
            ph2d_app_motion::field_gizmo::pan_scene_camera(&mut gfx.camera, split, size, dx, dy);
            self.pan_anchor = Some(self.last_pointer);
        }
        // Fase 0f: extend the active rubber-band rect, if any.
        if let Some(rb) = self.rubber_band.as_mut() {
            rb.current_screen = self.last_pointer;
        }
        // Motion Nodes M0.T1: `evt.button` is the REAL held button (winit's Move has none) — a
        // middle/right drag reaches editor-core with its identity (graph pan/box-select).
        self.handler.on_pointer(evt);
        // O reparent só sai no Up; o Move nunca o emite.
        let _ = forward_to_hero(self.gfx.as_mut(), evt);
        // M14.7 C: o arrasto de gizmo aberto segue o cursor (pivô / escala / rotação / translação).
        self.advance_gizmo_drag();
        // W-J2: and the joint-anchor drag, which is NOT a gizmo drag — it writes
        // a body-local anchor through the bridge's door, not a `Transform`.
        self.advance_joint_anchor_drag();
        // **§12** — e a alça do gizmo de âncora, que também não é arrasto de gizmo: ela publica
        // `InspectorAnchorEdit` no barramento, a MESMA porta por onde o painel escreve.
        self.advance_anchor_gizmo_drag();
        // W-Grab: a MÃO, a pose e o FK são da crate da física (W2/L2 Fase B); a conversão `ecrã → mundo` fica AQUI
        // (a câmera e a janela são da shell). Os dois de POSE devolvem «autorou»: quando um quadro conta para o undo
        // decide-se aqui.
        if let Some(gfx) = self.gfx.as_mut() {
            let window = gfx.scene_window();
            let world = gfx.camera.screen_to_world(self.last_pointer, window);
            let opts = self.physics.interaction.ik_options();
            ph2d_app_physics::body_grab::advance_body_grab(&mut gfx.physics, world);
            let autorou = ph2d_app_physics::body_pose::advance_body_pose(
                &mut gfx.physics,
                &mut gfx.sim,
                world,
                opts,
            ) | ph2d_app_physics::body_fk::advance_body_fk(
                &mut gfx.physics,
                &mut gfx.sim,
                world,
            );
            if autorou {
                self.any_input_this_frame = true;
            }
        }
        // Enio 2026-07-10: snap vetorial em TEMPO REAL — depois de o advance seguir o
        // cursor, gruda a forma arrastada no vizinho mais próximo (ponta p/ aberta,
        // vértice p/ fechada). Roda todo Move, então a forma prende/solta ao vivo.
        self.snap_dragged_vec_during_drag();
        // Drag-in-progress: forward pointer to active tool panel
        // hit-test → updates slider value continuously.
        if self.dragging.is_some() {
            self.dispatch_panel_pointer(self.last_pointer.0, self.last_pointer.1, false);
        }
    }

    pub(crate) fn on_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 16.0, y * 16.0),
            MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
        };
        // M14.4b.bis: wheel over the canvas zooms the camera. Wheel
        // over a hero panel keeps the existing panel-scroll behavior
        // (forward to hero).
        // ⭐ **A roda pertence À PALETA enquanto ela estiver aberta** (F3 / ADR-0166), e vem antes
        // de tudo — inclusive do Input Map: ela é um modal de TELA CHEIA com scrim, então não há
        // «por fora dela». Sem esta linha a lista transbordava o ecrã e o que sobrava era
        // inalcançável, com a roda a dar zoom no canvas por baixo do scrim (report do Enio, 25/08).
        if self.command_palette_wheel(dy) {
            return;
        }
        // Um menu da aba de família por cima de um painel é da aba.
        let (px, py) = self.last_pointer;
        let over = cursor_over_hero_panel(self.gfx.as_ref(), px, py);
        let over_panel =
            (self.hero_ref()).map_or(over, |h| h.documents.chrome_owns_wheel(over, px, py));
        // ⭐ Um QUADRO activo (MiroClone) é dono da área de desenho: a roda dá zoom nele, nunca na cena.
        if !over_panel
            && let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            && ph2d_editor_core::screens::hero::board_view::wheel(
                hero,
                self.last_pointer.0,
                self.last_pointer.1,
                dy,
            )
        {
            self.any_input_this_frame = true;
            return;
        }
        // O ajuste modal do Gap Closure (Ctrl+roda em modo Fill): a lei em `ph2d_app_flip::gap_live::gap_wheel`.
        if !over_panel
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let Some(mexeu) = ph2d_app_flip::gap_live::gap_wheel(
                self.flip_state.active,
                self.flip_state.style,
                dy / 16.0,
                self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()),
            )
        {
            self.any_input_this_frame |= mexeu;
            return;
        }
        // A rolagem de uma moldura que recorta e transborda, ANTES do zoom (o porquê no `wheel_scrolls_a_frame`).
        if !over_panel && self.wheel_scrolls_a_frame(dx, dy) {
            return;
        }
        if !over_panel && let Some(gfx) = self.gfx.as_mut() {
            // Wheel up (positive dy) zooms IN (smaller height_world). A lei vive na câmera: a
            // pinça de dois dedos entra por aqui com a inversa dela (`wheel_px_for`).
            let factor = ph2d_render::Camera2d::wheel_factor(dy);
            // ⚠️ **A roda escreve o DESTINO; quem move a câmera é o quadro** (`crate::canvas_zoom`).
            // O `camera.zoom(factor)` que morava aqui fazia do gesto mais repetido do app o único
            // movimento da tela que salta.
            gfx.canvas_zoom.wheel(gfx.camera.height_world, factor);
            self.any_input_this_frame = true;
        } else {
            let evt = ph2d_host::WheelEvent {
                x: self.last_pointer.0,
                y: self.last_pointer.1,
                delta_x: dx,
                delta_y: dy,
                modifiers: Self::convert_modifiers(self.modifiers),
                timestamp_ns: Self::timestamp_ns(),
            };
            forward_wheel_to_hero(self.gfx.as_mut(), evt);
        }
    }

    /// ⭐ A porta ÚNICA do clique (ver [`Self::pointer_moved`]); `button` = o do winit.
    pub(crate) fn pointer_button(&mut self, evt: PointerEvent, button: MouseButton) {
        ph2d_pen::porta::poe_tinta(&evt);
        let (kind, mapped_button) = (evt.kind, evt.button);
        let state = match kind {
            PointerKind::Down => ElementState::Pressed,
            _ => ElementState::Released,
        };
        self.any_input_this_frame = true;
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            if kind == PointerKind::Down {
                h.store.desarma_escuta_no_clique();
            }
            h.documents.note_button(kind, mapped_button);
        }
        // W-Grab: soltar a mão (e a alça da âncora, §12) vem ANTES dos early-returns, qualquer botão.
        if state == ElementState::Released {
            if let Some(gfx) = self.gfx.as_mut() {
                ph2d_app_physics::body_grab::release_body_grab(&mut gfx.physics);
                ph2d_app_physics::body_pose::release_body_pose(&mut gfx.physics);
                ph2d_app_physics::body_fk::release_body_fk(&mut gfx.physics);
            }
            self.end_anchor_gizmo_drag();
        }
        // A faixa única (arrastar, redimensionar a janela) só depois das portas: desarmar, soltar, a pressão.
        if self.ramo_aperto_solta_teclado(state, evt) {
            return;
        }
        if self.ramo_faixa_unica(evt, button) {
            return;
        }
        self.ramo_arrasto_biblioteca(state, button);
        // O rato do JOGO (itens 6–7): com a cena a correr o clique na cena é SÓ dele (`posse_do_jogo`). Antes dele, a
        // tela «Controls» do jogador aberta é dona do clique (InputMap §4).
        let principal = mapped_button == ph2d_host::PointerButton::Primary;
        let controles = self.clique_dos_controles((state == ElementState::Pressed, principal));
        if controles || self.rato_do_jogo(mapped_button, state == ElementState::Pressed) {
            if !controles {
                self.clique_so_do_jogo(mapped_button, state == ElementState::Pressed);
            }
            return;
        }
        // ⭐ No palco não há editor: o que não é do jogo só pode ser dos botões do jogo (dono 07/11). Os ramos abaixo são
        // do editor, e tomavam o clique antes de ele chegar aos botões.
        if self.no_palco() {
            let _ = forward_to_hero(self.gfx.as_mut(), evt);
            return;
        }
        if self.ramo_alca_da_ancora(state, button) {
            return;
        }
        // Motion Nodes M0.T1: o botão preso entre Down e Up, para o `CursorMoved` o levar.
        self.held_button = match kind {
            PointerKind::Down => Some(mapped_button),
            PointerKind::Up => None,
            PointerKind::Move => self.held_button,
        };
        self.handler.on_pointer(evt);
        #[cfg(feature = "panel-audio-editor")]
        if self.ramo_editor_audio(kind) {
            return;
        }
        // Um menu de contexto (ou o modal «Fill adjust») aberto ANTES do clique é dono dele: os ramos do canvas abaixo
        // não disparam por baixo do overlay. Lido agora porque o `forward_to_hero` pode fechá-lo.
        let menu_open_before = self.menu_aberto();
        // O conta-gotas armado antes do clique (o `forward_to_hero` desarma-o): o pincel não pinta onde se amostrou.
        let eyedropper_armed_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.eyedropper_pending().is_some());

        // ADR-0108: a caneta do Vector só desenha no canvas VAZIO — sobre qualquer UI (painel, pílula, menu) o clique
        // cai no cromo; sem painel sob o cursor E sem widget acertado (`hit_index` + `panel_at`).
        let on_canvas = self.pointer_on_canvas(evt.x, evt.y);
        if self.ramo_pixel(kind, mapped_button, on_canvas, menu_open_before) {
            return;
        }
        if self.ramo_preview_e_fechos(kind, mapped_button, evt, menu_open_before) {
            return;
        }
        // ⭐ Com um QUADRO activo (MiroClone) o clique na área de desenho é dele — nunca da cena.
        let mods = Self::convert_modifiers(self.modifiers);
        if let Some(g) = self.gfx.as_mut()
            && let Some(hero) = g.hero_screen.as_mut()
            && ph2d_editor_core::screens::hero::board_view::pointer(
                hero,
                board_input(&mut g.text_system, mods),
                kind,
                mapped_button,
                evt.x,
                evt.y,
                on_canvas,
            )
        {
            return;
        }
        // ⭐⭐⭐ **O BOTÃO DO HUD** (TOP-20 #20) — cedo de propósito: durante a CORRIDA um clique num
        // botão é do jogo, não do editor, e deixá-lo cair na selecção tornaria o menu inalcançável.
        // ⛔ Fora da corrida este ramo devolve `false` e a forma selecciona-se como qualquer outra.
        if self.ramo_botao_do_hud(kind, mapped_button, on_canvas) {
            return;
        }
        if self.ramo_flip_premidos(kind, mapped_button, menu_open_before, on_canvas) {
            return;
        }
        if self.ramo_select_modais(kind, mapped_button, evt, menu_open_before, on_canvas) {
            return;
        }
        if self.ramo_picks_de_fisica_e_alcas(kind, mapped_button, evt, menu_open_before) {
            return;
        }
        if self.ramo_alcas_soltas(kind, mapped_button, evt) {
            return;
        }
        if self.ramo_ferramenta_osso(mapped_button, kind, on_canvas, evt, menu_open_before) {
            return;
        }
        if self.ramo_ferramenta_vetorial(mapped_button, kind, on_canvas, evt, menu_open_before) {
            return;
        }

        // Painter layers drag-reparent (W3 T3.8): the dispatch emits a
        // PanelRowReparent (família PainterLayer) no Up de um arrasto activo; encaminha-o para
        // the active PainterTool, which reverses NodeId→LayerId and applies
        // move_into_group / reorder. The concrete-tool downcast lives in the
        // allowlisted painter bridge so central dispatch stays downcast-free
        // (architecture_no_downcast_to_concrete_tool_in_shell gate).
        if let Some((dragged, drop)) = forward_to_hero(self.gfx.as_mut(), evt)
            && let Some(gfx) = self.gfx.as_mut()
        {
            ph2d_app_painter::painter_bridge_queries::apply_layer_reparent(
                &mut gfx.tools,
                dragged,
                drop,
            );
        }

        if self.ramo_reclamantes(
            mapped_button,
            kind,
            evt,
            menu_open_before,
            eyedropper_armed_before,
        ) {
            return;
        }
        // ⭐ O botão direito no canvas livre (spec/06 F2) — DEPOIS de todos os reivindicantes dele.
        if matches!(kind, PointerKind::Down)
            && mapped_button == ph2d_host::PointerButton::Secondary
            && on_canvas
            && !menu_open_before
            && let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
            && ph2d_editor_core::screens::hero::mode_drive::right_click_on_canvas(hero)
        {
            return;
        }

        // M14.7 C: gizmo drag begin/end. A Primary Down that lands on
        // a gizmo handle starts a drag (snapshot Transform + cursor
        // world pos); Up clears it. Move handling lives in CursorMoved
        // so every motion event gets the live cursor.
        if mapped_button == ph2d_host::PointerButton::Primary {
            match kind {
                PointerKind::Down => {
                    if self.ramo_gizmo_premido(evt, menu_open_before) {
                        return;
                    }
                }
                PointerKind::Up => {
                    self.ramo_gizmo_largar(evt);
                }
                _ => {}
            }
        }
        self.ramo_pan_e_barra_lateral(state, button);
    }
}

/// O botão do rato do winit vira o evento único do ponteiro.
#[path = "input_dispatch/despacho_rato.rs"]
mod despacho_rato;

#[cfg(test)]
#[path = "input_dispatch/despacho_testes.rs"]
mod tests;

#[cfg(test)]
#[path = "input_dispatch/despacho_testes_cursor.rs"]
mod cursor_tests;

#[cfg(all(test, feature = "panel-audio-editor"))]
#[path = "input_dispatch/despacho_testes_espectro.rs"]
mod spectral_axis_tests;

/// O que o quadro (MiroClone) recebe com cada evento do rato: a letra, os modificadores e o relógio.
fn board_input(
    text: &mut ph2d_text::TextSystem,
    mods: ph2d_host::Modifiers,
) -> ph2d_editor_core::screens::hero::board_view::Input<'_> {
    ph2d_editor_core::screens::hero::board_view::Input {
        text,
        mods,
        now_ns: crate::App::timestamp_ns(),
    }
}
