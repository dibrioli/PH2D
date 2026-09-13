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

// ⭐ **W2: os ganchos de entrada da janela 3D são um trait de extensão sobre o `AppHost`.**
// Os sítios de chamada abaixo ficaram **byte a byte iguais** — o que mudou foi só esta linha.
use ph2d_app_field3d::input::Field3dInput;

use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::event_loop::ActiveEventLoop;

use ph2d_editor_core::Toast;
use ph2d_host::{
    CloseAction, HostHandler, Lifecycle, PlatformHost, PointerEvent, PointerKind, PointerSource,
    WindowSize,
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

/// **As teclas que ENCERRAM um gesto em curso** (Esc cancela, Enter confirma) — irmão do
/// `keyboard`, cortado dele pelo cap de LOC. A ORDEM entre elas é a lei, e é por isso que
/// viajam juntas em vez de por dono.
mod keyboard_escapes;
/// ⭐ As teclas do modelador 3D, numa porta só — ver [`keyboard_field3d`].
mod keyboard_field3d;
/// ⭐⭐⭐ **Que MODO é dono do teclado neste quadro** — irmão do `keyboard`, cortado dele pelo cap
/// de LOC. *Um modo em curso é dono da entrada dele.*
mod keyboard_modal;
mod keyboard_palette; // as teclas do palette de nos -- MODAL, ver o doc do modulo

// Os ramos e os irmãos do despacho: cada `despacho_*.rs` diz no próprio `//!` o que guarda.
mod despacho_clique_flip;
mod despacho_clique_gizmo;
mod despacho_clique_largar;
mod despacho_clique_pick;
mod despacho_clique_prologo;
mod despacho_clique_reclamantes;
mod despacho_clique_roldana;
mod despacho_clique_select;
mod despacho_clique_vetor;
mod despacho_clique_vetor_premido;
mod despacho_clique_vetor_solto;
mod despacho_metodos_janela_e_vetor;
mod despacho_metodos_modos_e_alcas;
mod despacho_metodos_picks_e_arrastos;
mod despacho_mover;
mod despacho_vetor_alinhar_e_forma;
mod despacho_vetor_gradiente;
mod despacho_vetor_ops;
/// **Os acordes de ARQUIVO** — irmão do `keyboard`, cortado dele pelo cap de LOC.
mod keyboard_files;
mod keyboard_hierarchy; // Delete/Ctrl+D sobre a linha da Hierarquia -- ver o doc do modulo
mod keyboard_painter; // a cadeia do Delete do Painter: ancora -> figura -> falloff
mod keyboard_timeline;
pub(crate) mod painter_canvas_input;
mod painter_canvas_mods;
/// ⭐ O menu de alça de um ponto da CURVA no canvas — irmão do `painter_canvas_input` pelo teto
/// de LOC, cortado por assunto.
mod painter_curve_input;
pub(crate) mod painter_falloff_input;
mod painter_grid_erase; // os modificadores que o CanvasPointer nao carrega
pub(crate) mod protect_brush;
pub(crate) use despacho_vetor_alinhar_e_forma::{
    VecAlign, VecDistribute, VecPathShapeOp, VecTransformField, apply_vec_align,
    apply_vec_distribute, apply_vec_path_shape, apply_vec_rotate_by, apply_vec_transform,
    vec_align_for_id, vec_distribute_for_id, vec_path_shape_for_id, vec_transform_field_for_id,
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
    VecFillKind, apply_vec_boolean, apply_vec_compound, apply_vec_delete_vertex,
    apply_vec_duplicate, apply_vec_fill_rule, apply_vec_flip, apply_vec_rotate,
    apply_vec_toggle_closed, apply_vec_vertex_kind, duplicate_vec_paths, screen_offset_world,
    vec_bool_op_for_id, vec_fill_kind_for_id, vec_flip_for_id, vec_reorder_for_id,
    vec_rotate_for_id, vec_vertex_kind_for_id,
};
// Só os `cursor_tests` o leem por `super::` — o `timeline_resize_cursor` já o tem no próprio ficheiro.
#[cfg(test)]
use despacho_metodos_janela_e_vetor::resize_cursor_for_edges;
// Só os testes do eixo espectral o leem por `super::` — os métodos de áudio já o têm no próprio ficheiro.
use despacho_clique_roldana::select_wheel_at;
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
    pub(crate) fn on_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        // Diagnostics: count every raw winit move (input rate), paired with `paint_stamps_this_frame`
        // in the HUD so the coalescing is visible (high events → 1 stamp).
        self.input_events_this_frame = self.input_events_this_frame.saturating_add(1);
        let prev = self.last_pointer;
        self.last_pointer = (position.x as f32, position.y as f32);
        // (Graph keyboard focus is set every frame by `motion_bridge` from the
        // cursor — reliable even when the cursor stops before the panel rect is
        // published; see its `over_graph` gate.)
        // M14.4e: cache the latest cursor for DroppedFile — winit's
        // DroppedFile carries no position, so we project the most-
        // recently-seen cursor to world.
        self.last_cursor = self.last_pointer;
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
            let _ = prev; // silence unused warning when feature shifts
        }
        // Fase 0f: extend the active rubber-band rect, if any.
        if let Some(rb) = self.rubber_band.as_mut() {
            rb.current_screen = self.last_pointer;
        }
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind: PointerKind::Move,
            source: PointerSource::Mouse,
            // Motion Nodes M0.T1: carry the REAL held button (winit's Move has
            // none). A middle/right drag now reaches editor-core with its
            // identity intact — the graph channel needs it (pan/box-select).
            button: self
                .held_button
                .unwrap_or(ph2d_host::PointerButton::Primary),
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        // A reparent only fires on pointer-Up (handled in on_mouse_input);
        // Move never emits one.
        let _ = forward_to_hero(self.gfx.as_mut(), evt);
        // M14.7 C: advance an open gizmo drag against the latest cursor
        // (MovePivot / scale / rotate / translate). Extracted to the
        // `gizmo_drag` sibling to keep this dispatch hub readable.
        self.advance_gizmo_drag();
        // W-J2: and the joint-anchor drag, which is NOT a gizmo drag — it writes
        // a body-local anchor through the bridge's door, not a `Transform`.
        self.advance_joint_anchor_drag();
        // **§12** — e a alça do gizmo de âncora, que também não é arrasto de gizmo: ela publica
        // `InspectorAnchorEdit` no barramento, a MESMA porta por onde o painel escreve.
        self.advance_anchor_gizmo_drag();
        // W-Grab: e a MÃO, que também não é arrasto de gizmo — ela move a âncora
        // de uma mola no solver, e o `Transform` chega pelo readback do dispatch.
        //
        // ⚠️⚠️ **Os três eram `impl App` e passaram a funções da crate da família** (W2/L2 Fase B).
        // A conversão `ecrã → mundo` fica AQUI, e é o ponto inteiro: a câmera e o tamanho da
        // janela são da shell, e o gesto só quer um ponto em mundo. ⭐ *Nenhum sexto método do
        // `AppHost` foi preciso* — o que parecia «precisar da `App`» eram três tipos de crate de
        // módulo que a shell por acaso segurava.
        if let Some(gfx) = self.gfx.as_mut() {
            let window = gfx.surface.size();
            let world = gfx.camera.screen_to_world(self.last_pointer, window);
            let opts = self.physics.interaction.ik_options();
            ph2d_app_physics::body_grab::advance_body_grab(&mut gfx.physics, world);
            // ⚠️ Os dois de POSE devolvem «autorou» e a shell é que marca o quadro: o gesto sabe
            // *que* autorou, e *quando* um quadro conta para o diff de undo é decisão daqui.
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
        // ⭐ **A roda sobre a janela do Input Map é dela** — e vem ANTES do resto, pelo motivo do
        // arrasto: a roda que atravessasse o cartão daria zoom no canvas por baixo dele.
        if self.input_map_wheel(dy) {
            return;
        }
        let over_panel =
            cursor_over_hero_panel(self.gfx.as_ref(), self.last_pointer.0, self.last_pointer.1);
        // ADR-0150 W1/M2: fora de painel, a roda aproxima a câmera 3D. Um
        // "passo" é uma linha de roda (os 16 px acima são a régua do zoom 2D).
        #[cfg(feature = "sculpt3d")]
        if !over_panel && self.sculpt3d_wheel(dy / 16.0) {
            return;
        }
        // ADR-0161 W4: o mesmo para a janela 3D de modelagem.
        if !over_panel && self.field3d_wheel(dy / 16.0) {
            return;
        }
        // **O ajuste modal do Gap Closure** (doc 06 §8): em modo Fill, Ctrl+roda sobre o
        // canvas ajusta o alcance — e os helpers no canvas mostram, ao vivo, quais vãos
        // o valor atual fecha (`flip_gap_live`). A roda CRUA continua sendo zoom
        // (inspecionar o line-art é load-bearing); o GP toma a roda inteira durante o
        // fill, e esta é a divergência deliberada, documentada em vez de silenciosa.
        if !over_panel
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let Some(track) = ph2d_app_flip::gap_live::gap_wheel_track(
                self.flip_state.active,
                self.flip_state.style,
                dy / 16.0,
            )
        {
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                // As DUAS metades do que o próprio slider faz num arrasto (ver
                // `panel-flip/event.rs`): o valor do widget no store (o knob que o
                // artista vê — sem isto ele pinta o valor velho por cima do novo) e o
                // `SetValue` pro tool (o valor autorado, clampado pelo MESMO braço).
                hero.store
                    .set_slider_value(ph2d_tool_flip::ids::FLIP_GAP, track as f32);
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                        ph2d_editor_core::tool::PanelEvent::SetValue(
                            ph2d_tool_flip::ids::FLIP_GAP,
                            track,
                        ),
                    ));
                self.any_input_this_frame = true;
            }
            return;
        }
        // **A ROLAGEM de uma moldura** (o item 3 do estudo dos contêineres): a roda sobre uma
        // moldura que RECORTA e cujo conteúdo NÃO CABE rola essa moldura, em vez de dar zoom.
        //
        // ⚠️ **Ela não rouba o zoom, e é isso que a torna aceitável sem modo nem modificador:** as
        // duas condições juntas só valem numa lista que o artista fez deliberadamente transbordar,
        // e ali rolar é a única coisa que a roda pode querer dizer. Em todo o resto da tela — que é
        // 99% dela — a roda continua a ser o zoom, que é o gesto do dia inteiro. É o precedente do
        // Gap Closure, uma dúzia de linhas acima, com a mesma frase: *a roda crua é load-bearing*.
        //
        // ⚠️ E ela vem **antes** do zoom pela razão de sempre: quem consome tem de decidir primeiro,
        // senão a câmera já se mexeu quando a moldura for perguntada.
        if !over_panel && self.wheel_scrolls_a_frame(dx, dy) {
            return;
        }
        if !over_panel && let Some(gfx) = self.gfx.as_mut() {
            // Wheel up (positive dy) zooms IN (smaller height_world).
            let factor = 0.9_f32.powf(dy / 16.0);
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

    pub(crate) fn on_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        self.any_input_this_frame = true;
        // W-Grab: **soltar a mão vem ANTES de tudo.** Este handler tem muitos
        // early-returns e uma mão que sobrevive ao release fica colada no cursor
        // para sempre; e vale para qualquer botão, porque uma mão não é um
        // modificador (ver `ph2d_app_physics::body_grab::release_body_grab`).
        if state == ElementState::Released {
            if let Some(gfx) = self.gfx.as_mut() {
                ph2d_app_physics::body_grab::release_body_grab(&mut gfx.physics);
                ph2d_app_physics::body_pose::release_body_pose(&mut gfx.physics);
                ph2d_app_physics::body_fk::release_body_fk(&mut gfx.physics);
            }
            // **§12** — e a alça do gizmo de âncora, pela MESMA razão escrita acima: este handler
            // tem muitos early-returns, e uma alça que sobrevive ao release fica colada ao cursor.
            self.end_anchor_gizmo_drag();
        }
        self.ramo_arrasto_biblioteca(state, button);
        self.ramo_aperto_solta_teclado(state);
        if self.ramo_navegacao_3d_e_ancora(state, button) {
            return;
        }
        let kind = match state {
            ElementState::Pressed => PointerKind::Down,
            ElementState::Released => PointerKind::Up,
        };
        let mapped_button = match button {
            MouseButton::Left => ph2d_host::PointerButton::Primary,
            MouseButton::Right => ph2d_host::PointerButton::Secondary,
            MouseButton::Middle => ph2d_host::PointerButton::Middle,
            _ => ph2d_host::PointerButton::Primary,
        };
        // Motion Nodes M0.T1: track the held button so `CursorMoved` can carry
        // its identity (winit Move events don't). Held between Down and Up.
        self.held_button = match kind {
            PointerKind::Down => Some(mapped_button),
            PointerKind::Up => None,
            PointerKind::Move => self.held_button,
        };
        let evt = PointerEvent {
            x: self.last_pointer.0,
            y: self.last_pointer.1,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: mapped_button,
            timestamp_ns: Self::timestamp_ns(),
        };
        self.handler.on_pointer(evt);
        #[cfg(feature = "panel-audio-editor")]
        if self.ramo_editor_audio(kind) {
            return;
        }
        // Was a right-click context menu (or the Fill "Fill adjust" modal) open when this click
        // arrived? If so the click belongs to that overlay (its slider/buttons/items) — chrome dispatch
        // in `forward_to_hero` handles it, so the canvas-consume arms below (paint / gizmo / select /
        // pan) must NOT also fire on a click LANDING on the overlay (which sits over the canvas). The
        // Fill modal counts as a modal exactly like the new-image dialog — without this, clicking its
        // threshold slider started a fresh flood-fill on the canvas underneath (mirror of the
        // new-image-modal "leaked a dab" fix). Captured now because `forward_to_hero` may close it.
        let menu_open_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| {
                h.store.context_menu().is_some() || h.store.fill_modal_pos().is_some()
            });
        // Colour-picker eyedropper armed when this click arrived? `forward_to_hero` services the pick
        // (sampling the pixel) AND clears the pending flag, so by the time the consume arms below run
        // it reads as disarmed. Capture it now so the Painter brush does NOT also paint where the user
        // sampled — the eyedropper must inhibit the brush (the sampled click is consumed, not painted).
        let eyedropper_armed_before = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .is_some_and(|h| h.store.eyedropper_pending().is_some());

        // ADR-0108 cutover: the Vector tool's Pen draws ONLY on empty canvas.
        // A press over ANY UI — a docked panel body, a topbar pill, an open
        // menu, or this tool's own Style panel controls — MUST fall through to
        // the chrome dispatch below, never the pen; otherwise the whole UI is
        // unclickable while drawing (can't even deactivate the tool). Guard
        // mirrors the sprite-pick path: no panel under the cursor AND no
        // interactive widget hit (`hit_index` covers pills / menus / panel
        // controls; `panel_at` covers panel bodies incl. the vector panel).
        let on_canvas = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| {
                h.store.panel_at(evt.x, evt.y).is_none() && h.hit_index.hit(evt.x, evt.y).is_none()
            })
            .unwrap_or(false);
        if self.ramo_preview_e_fechos(kind, mapped_button, evt, menu_open_before) {
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
        if self.ramo_ferramenta_vetorial(mapped_button, kind, on_canvas, evt, menu_open_before) {
            return;
        }

        // Painter layers drag-reparent (W3 T3.8): the dispatch emits a
        // PainterLayerReparent on Up of an active layer-row drag; route it to
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

#[cfg(test)]
#[path = "input_dispatch/despacho_testes.rs"]
mod tests;

#[cfg(test)]
#[path = "input_dispatch/despacho_testes_cursor.rs"]
mod cursor_tests;

#[cfg(all(test, feature = "panel-audio-editor"))]
#[path = "input_dispatch/despacho_testes_espectro.rs"]
mod spectral_axis_tests;
