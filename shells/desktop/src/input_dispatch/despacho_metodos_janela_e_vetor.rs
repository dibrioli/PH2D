//! **Os métodos do despacho: janela e vetor** ([`super`]) — o pick do Blend, o fecho, o redimensionar, os ficheiros
//! largados, modificadores, IME, cursores e os verbos do vetor, mudados VERBATIM; os privados passam a `pub(super)`.

use super::*;
use ph2d_i18n::{tr, tr_with};

impl crate::App {
    /// **Pick Shapes** (ADR-0128 C2b): alterna a forma FECHADA sob `world` na lista de escolhidas
    /// ([`ph2d_app_vec::state::VecState::blend_picks`]), na ordem de clique. Já escolhida → removida (corrigir a
    /// ordem sem recomeçar); nova → anexada, até o teto de [`crate::blend_live::MAX_BLEND_SOURCES`].
    /// Só FECHADAS entram — uma curva aberta não tem interior para interpolar. Marca
    /// `any_input_this_frame` para a prévia do spine redesenhar.
    pub(crate) fn blend_pick_at(&mut self, world: [f64; 2]) {
        let px = self.vec_px_to_world();
        let hit = {
            let Some(gfx) = self.gfx.as_ref() else { return };
            self.vec
                .pen
                .path_at(&gfx.vec_scene, world, 10.0 * px)
                .filter(|id| {
                    gfx.vec_scene
                        .paths()
                        .iter()
                        .any(|p| p.id == *id && p.closed)
                })
        };
        let Some(id) = hit else { return };
        if let Some(pos) = self.vec.blend_picks.iter().position(|&p| p == id) {
            self.vec.blend_picks.remove(pos);
        } else if self.vec.blend_picks.len() < crate::blend_live::MAX_BLEND_SOURCES {
            self.vec.blend_picks.push(id);
        }
        self.any_input_this_frame = true;
    }

    pub(crate) fn on_close_request(&mut self, event_loop: &ActiveEventLoop) {
        match self.handler.on_close_request() {
            CloseAction::Close => {
                self.handler.on_lifecycle(Lifecycle::WillTerminate);
                // Tear the audio system down HERE, deterministically, while the
                // main thread is quiescent — instead of letting the `cpal::Stream`
                // (ALSA/PipeWire, `!Send`) drop LAST in the `App` field cascade at
                // the end of `main`, where `snd_pcm_close` on the pipewire-alsa
                // plugin segfaults on teardown (benign — fires after "exited
                // cleanly" — but returns 139, which pollutes exit-code checks).
                #[cfg(feature = "panel-audio-editor")]
                {
                    self.audio = None;
                }
                // **E a GPU pela MESMA razão, medida no mesmo lugar.** O `EventLoop` é CONSUMIDO por
                // `run_app`, então ele — e com ele a conexão Wayland — morre quando `run_app` retorna,
                // e só DEPOIS o `App` desenrola seus campos. A `SurfaceContext` do `AppGfx` cai nesse
                // rabo, e destruir uma superfície EGL sobre um `wl_display` que já se foi marshala num
                // proxy morto: `wl_proxy_marshal_array_flags` <- libnvidia-egl-wayland2 <- libEGL_nvidia,
                // dentro do epílogo do `main` (stack de 217 coredumps desde 2026-07-22, idêntica nas
                // SEIS worktrees — é da shell, não de linha nenhuma). Benigno, porque dispara depois do
                // "exited cleanly"; mas devolve 139 e some com todo `$status` que um smoke checaria.
                //
                // A ordem aqui é a INVERSA da construção, e cada passo é uma dependência real: a
                // superfície/dispositivo primeiro (o que fala EGL), o host depois, a janela por último —
                // ela é quem possui o `wl_surface` que os outros dois referenciam.
                self.gfx = None;
                self.host = None;
                self.window = None;
                // Todo frame daqui em diante é um frame sem dispositivo (`render_frame` desiste).
                self.exiting = true;
                event_loop.exit();
            }
            CloseAction::Cancel => {}
        }
    }

    /// **O oráculo do teardown: feche a janela sozinho depois de `n` frames.**
    /// `PH2D_EXIT_AFTER_FRAMES=<n>` (não-setado = inerte, zero custo além de um `Relaxed` load).
    ///
    /// O defeito que ele mede vive no DESLIGAMENTO, então nenhum teste headless o alcança: só um app
    /// de verdade, com janela de verdade e superfície EGL de verdade, tem o que destruir na ordem
    /// errada. Com este gancho o oráculo passa a ser o `$?` do processo — **139 = a superfície morreu
    /// depois do `wl_display`; 0 = a ordem está certa** —, o que qualquer smoke pode checar sem olho
    /// humano nenhum.
    ///
    /// ⚠️ Ele passa pela **MESMA porta** que o X da janela (`on_close_request`), nunca por um
    /// `exit()` próprio. Um caminho de saída paralelo provaria a ordem de destruição de um caminho
    /// que o artista nunca toma — verde sobre nada.
    pub(crate) fn exit_after_frames_tick(&mut self, event_loop: &ActiveEventLoop) {
        use std::sync::OnceLock;
        use std::sync::atomic::{AtomicU32, Ordering};
        static LIMIT: OnceLock<Option<u32>> = OnceLock::new();
        static SEEN: AtomicU32 = AtomicU32::new(0);
        let Some(limit) = *LIMIT.get_or_init(|| {
            std::env::var("PH2D_EXIT_AFTER_FRAMES")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
        }) else {
            return;
        };
        if SEEN.fetch_add(1, Ordering::Relaxed) + 1 >= limit && !self.exiting {
            println!(
                "PH2D_EXIT_AFTER_FRAMES={limit} atingido — fechando pela porta do X da janela."
            );
            self.on_close_request(event_loop);
        }
    }

    pub(crate) fn on_resized(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.pending_resize = Some(WindowSize::new(size.width, size.height));
    }

    /// M14.4e drag-and-drop. winit emits one HoveredFile per path when
    /// multiple files are dragged together. Buffer paths into
    /// `self.hovered_files` and push to the hero (for the overlay) on
    /// every HoveredFile event.
    pub(crate) fn on_hovered_file(&mut self, path: std::path::PathBuf) {
        self.hovered_files.push(path);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = Some((self.hovered_files.clone(), self.last_cursor));
        }
        self.handler.on_file_hover(&self.hovered_files);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_hovered_file_cancelled(&mut self) {
        self.hovered_files.clear();
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.dragging_files = None;
        }
        self.handler.on_file_hover_cancel();
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_dropped_file(&mut self, path: std::path::PathBuf) {
        // M14.7 polish (7.3 fix): winit fires `DroppedFile` once PER
        // FILE on macOS but the events arrive across multiple loop
        // iterations. Importing inline on each event was racy — some
        // imports silently dropped when an event came in mid-render.
        // Buffer the path here; `render_frame` drains `pending_drops`
        // atomically.
        self.pending_drops.push(path);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    pub(crate) fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        if let Some(host) = &self.host {
            host.scale().set(scale_factor as f32);
            if let Some(gfx) = self.gfx.as_ref() {
                self.pending_resize = Some(gfx.surface.size());
            }
        }
    }

    pub(crate) fn on_modifiers_changed(&mut self, mods: winit::event::Modifiers) {
        self.modifiers = mods.state();
        // M14.A: push the Shift state to the hero's WidgetStore so
        // `dispatch_pointer` Move can scale the NumberInput drag delta
        // correctly (Shift = fine adjustment). The ph2d-host
        // `PointerEvent` schema doesn't carry modifiers natively — the
        // store cache is the canonical bridge for now.
        // Fase 0c: also push the Cmd (macOS super) / Ctrl modifier
        // OR'd together — used by hierarchy + canvas multi-select to
        // map a click into `SelectModifier::Toggle`.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.store.set_shift_held(self.modifiers.shift_key());
            hero.store
                .set_cmd_held(self.modifiers.super_key() || self.modifiers.control_key());
            // Motion Nodes M0.T3 — Alt cache, folded into `GestureMods.alt` for
            // graph gestures (mirror of shift/cmd; pointer events carry no mods).
            hero.store.set_alt_held(self.modifiers.alt_key());
        }
        // Shift (1:1) / Alt (do centro) durante um gesto de FORMA: reconstrói o preview
        // na hora, sem esperar o próximo Move — apertar a tecla e a forma não reagir é
        // o comportamento errado (o usuário costuma apertar com o mouse parado).
        let c = shape_constraint(self.modifiers);
        if let Some(gfx) = self.gfx.as_mut() {
            self.vec.shape.set_constraint(&mut gfx.vec_scene, c);
        }
    }

    /// IME composition commits — PT-BR / Spanish / French accent
    /// dead-key sequences arrive here on macOS, NOT in `KeyEvent::text`
    /// (the system text-input service swallows the dead-key keystroke
    /// and emits the composed char via `Ime::Commit`).
    pub(crate) fn on_ime_commit(&mut self, text: String) {
        for ch in text.chars() {
            if !ch.is_control() {
                forward_text_to_hero(self.gfx.as_mut(), ch);
            }
        }
        // `Preedit` (in-progress composition) is ignored for now — no
        // visible preedit caret yet. Future: render the preedit text
        // in italics at the caret.
    }

    /// Reflect the current hover context in the OS cursor. Called each
    /// CursorMoved (winit dedups the icon). Priority: an armed colour-picker
    /// eyedropper wins (a crosshair "target"), else the Motion graph's split
    /// divider shows a double-arrow resize cursor (`NsResize` ↕ for a horizontal
    /// divider, `EwResize` ↔ for a vertical one), else the 3D canvas split seam
    /// (same law, plus `Move` on the crossing where both seams travel together),
    /// else a timeline grab band
    /// (panel edge, label splitter, graph-height grip), else the default arrow.
    pub(super) fn update_eyedropper_cursor(&self) {
        let Some(win) = self.window.as_ref() else {
            return;
        };
        use winit::window::CursorIcon;
        let cursor = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map(|h| {
                if h.store.eyedropper_pending().is_some() {
                    CursorIcon::Crosshair
                } else if self.over_motion_split_divider(h) {
                    if h.view.center_split.is_vertical() {
                        CursorIcon::EwResize
                    } else {
                        CursorIcon::NsResize
                    }
                } else if let Some(icon) = ph2d_app_field3d::smoke::with_smoke(|s| {
                    ph2d_app_field3d::smoke::divider_cursor(s, self.last_pointer)
                })
                .flatten()
                {
                    // ⭐ A costura da divisão do canvas 3D (W93) — a mesma fonte que o arrasto lê.
                    icon
                } else if let Some(icon) = self.sculpt3d_seam_cursor() {
                    // ⭐ **A costura da divisão da ESCULTURA** (2026-09-08) — a mesma lei do vizinho
                    // acima, com a divisão da outra cena 3D. ⚠️ Ela sai da MESMA porta que o
                    // arrasto pergunta (`Sculpt3dScene::seam_grab`), senão a seta aparece um pixel
                    // ao lado de onde o gesto pega — o que se lê como *«às vezes não agarra»*.
                    icon
                } else if let Some(icon) = self.timeline_resize_cursor(h) {
                    icon
                } else if let Some(icon) =
                    self.dock_seam_cursor(self.last_pointer.0, self.last_pointer.1)
                {
                    // ⭐ A borda de uma coluna docada — a MESMA porta que o arrasto pergunta.
                    icon
                } else {
                    CursorIcon::Default
                }
            })
            .unwrap_or(CursorIcon::Default);
        win.set_cursor(cursor);
    }

    /// The double-arrow cursor for the timeline grab band under the pointer, if
    /// any. Resolves the last pointer through the hit index to a `TimelineSurface`
    /// hit — the same channel the drag uses, so the cursor and the gesture always
    /// agree on where the band is (mirror of `over_motion_split_divider`).
    pub(super) fn timeline_resize_cursor(
        &self,
        hero: &ph2d_editor_core::HeroScreen,
    ) -> Option<winit::window::CursorIcon> {
        use ph2d_editor_core::interaction::TimelineHitKind;
        use winit::window::CursorIcon;
        let (x, y) = self.last_pointer;
        let (_, kind) = hero
            .hit_index
            .hit(x, y)
            .and_then(|id| hero.store.timeline_surface_at_id(id))?;
        Some(match kind {
            // The names column widens sideways; the graph band grows downward.
            TimelineHitKind::LabelSplitter => CursorIcon::EwResize,
            TimelineHitKind::GraphResize => CursorIcon::NsResize,
            TimelineHitKind::ResizeEdge { edges } => resize_cursor_for_edges(edges),
            // The veil's duration grip resizes the composition sideways (Enio,
            // 2026-07-23: the ↔ over the ruler at the veil edge).
            TimelineHitKind::DurationHandle => CursorIcon::EwResize,
            _ => return None,
        })
    }

    /// Is the cursor over the Motion graph's draggable split divider? Resolves
    /// the last-pointer position through the hit index to a `GraphSurface` hit
    /// and checks its kind — the same channel the divider drag uses, so the
    /// cursor and the gesture agree on the grab band.
    pub(super) fn over_motion_split_divider(&self, hero: &ph2d_editor_core::HeroScreen) -> bool {
        let (x, y) = self.last_pointer;
        hero.hit_index
            .hit(x, y)
            .and_then(|id| hero.store.graph_surface_at_id(id))
            .is_some_and(|(_, kind)| {
                matches!(
                    kind,
                    ph2d_editor_core::interaction::GraphHitKind::SplitDivider
                )
            })
    }

    /// ADR-0108 Fase 1: booleana N-ária sobre as regiões fechadas SELECIONADAS
    /// (hotkeys U/I/D/X). Delega ao livre [`apply_vec_boolean`] com os refs
    /// decompostos — o mesmo caminho usado pelos botões Boolean do painel (drain
    /// do render_loop, onde `self.gfx` já está destruturado e o método não é
    /// chamável).
    pub(super) fn vec_boolean(&mut self, op: ph2d_vec_boolean::PathfinderOp) {
        if let Some(gfx) = self.gfx.as_mut() {
            let xf = ph2d_vec_entities::transform::build(&gfx.sim, &self.vec.entities);
            apply_vec_boolean(&mut gfx.vec_scene, &mut self.vec.pen, &xf, op);
        }
    }

    /// Arrow-key nudge: move the selection by a SCREEN delta (px), converted to
    /// world (honours zoom + orientation). Returns whether anything moved.
    ///
    /// (Até 2026-09-12 levava `record_undo`, que agrupava as auto-repetições da seta num passo da
    /// `History` do vetor. A pilha morreu sem leitor; o Ctrl+Z é o da fila global.)
    pub(crate) fn vec_nudge_selected(&mut self, dx_px: f64, dy_px: f64) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let base = gfx.camera.screen_to_world((0.0, 0.0), win);
        let moved = gfx
            .camera
            .screen_to_world((dx_px as f32, dy_px as f32), win);
        let (dx, dy) = ((moved[0] - base[0]) as f64, (moved[1] - base[1]) as f64);
        self.vec.pen.nudge(&mut gfx.vec_scene, dx, dy)
    }

    /// ADR-0108 Fase 1: Delete/Backspace no modo vetorial — prioriza apagar o
    /// VÉRTICE selecionado (edição de nó); sem vértice selecionado (ex.: resultado
    /// de booleana), apaga o PATH inteiro.
    pub(crate) fn vec_delete_selected_vertex_or_path(&mut self) -> bool {
        if self.vec.pen.selected_vert().is_some()
            && let Some(gfx) = self.gfx.as_mut()
            && apply_vec_delete_vertex(&mut gfx.vec_scene, &mut self.vec.pen)
        {
            eprintln!("[ph2d-vec] vértice apagado");
            return true;
        }
        self.vec_delete_selected()
    }

    /// ADR-0108 Fase 1: apaga o path selecionado (fallback do Delete sem vértice).
    /// World-space offset for a `px` screen-space diagonal shift (paste / dup
    /// placement), honouring the current zoom. `(0, 0)` when the gfx isn't ready.
    pub(super) fn vec_screen_offset(&self, px: f64) -> (f64, f64) {
        let Some(gfx) = self.gfx.as_ref() else {
            return (0.0, 0.0);
        };
        screen_offset_world(&gfx.camera, gfx.surface.size(), px)
    }

    /// **Um campo de entrada de TEXTO tem o foco do teclado?**
    ///
    /// Pergunta ao STORE, não ao painel: qualquer `TextInput` / `NumberInput` /
    /// `Combobox` conta, venha ele do rename da Hierarquia, de um chip numérico do
    /// Inspector ou de um campo do painel do vetor. ⚠️ O nome antigo era
    /// `text_entry_focused`, e ele **mentia desde o dia em que o bloco do
    /// Flip passou a chamá-lo** — isto nunca foi uma pergunta do Vector.
    ///
    /// Quem quer *"as minhas teclas estão vivas?"* pergunta a
    /// [`Self::vector_keys_live`] / [`Self::motion_keys_live`], não a esta —
    /// compor `tool_active && !text_entry_focused` em cada braço é a enumeração
    /// que o BUGS #25 documenta apodrecendo.
    pub(crate) fn text_entry_focused(&self) -> bool {
        let Some(h) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return false;
        };
        let Some(id) = h.store.focus_id() else {
            return false;
        };
        matches!(
            h.store.get(id),
            Some(
                ph2d_editor_core::InteractiveState::TextInput { .. }
                    | ph2d_editor_core::InteractiveState::NumberInput { .. }
                    | ph2d_editor_core::InteractiveState::Combobox { .. }
            )
        )
    }

    /// Vector Ctrl+C: copy the object selection into the in-app clipboard. O
    /// recorte leva os GRUPOS inteiramente selecionados junto (ver `VecScene::
    /// copy_paths`), então colar reconstrói a estrutura. No-op sem seleção.
    pub(super) fn vec_copy(&mut self) {
        let sel = self.vec.pen.selected_paths().to_vec();
        if sel.is_empty() {
            return;
        }
        if let Some(gfx) = self.gfx.as_ref() {
            let clip = gfx.vec_scene.copy_paths(&sel);
            if !clip.is_empty() {
                self.vec.clipboard = Some(clip);
            }
        }
    }

    /// Vector Ctrl+X: copy the object selection, then delete it.
    pub(super) fn vec_cut(&mut self) {
        self.vec_copy();
        self.vec_delete_selected();
    }

    /// Vector Ctrl+V: paste the clipboard, offset ~12 px (screen→world), e seleciona
    /// o resultado. Ctrl+Shift+V cola **no lugar** (sem deslocar). ONE undo step.
    pub(super) fn vec_paste(&mut self, in_place: bool) {
        let Some(clip) = self.vec.clipboard.clone() else {
            return;
        };
        let (dx, dy) = if in_place {
            (0.0, 0.0)
        } else {
            self.vec_screen_offset(PASTE_OFFSET_PX)
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let new_ids = gfx.vec_scene.paste_clip(&clip, dx, dy);
        if new_ids.is_empty() {
            return;
        }
        self.vec.pen.select_many(&new_ids);
    }

    /// A seleção de objeto que tocar `path` produz — o grupo inteiro, se houver.
    /// A árvore é a Hierarquia (ADR-0110), então quem sabe disso é o ECS.
    pub(crate) fn vec_object_selection_for(&self, path: u64) -> Vec<u64> {
        let Some(gfx) = self.gfx.as_ref() else {
            return vec![path];
        };
        ph2d_vec_entities::entities::object_selection_for(
            &gfx.sim,
            &gfx.vec_scene,
            &self.vec.entities,
            path,
        )
    }

    /// Ctrl+G / Ctrl+Shift+G: agrupa / desagrupa a seleção. O grupo é uma entidade
    /// comum, então ele aceita sprite e path vetorial no mesmo saco.
    pub(super) fn vec_group(&mut self, group: bool) {
        let sel: Vec<u64> = self
            .vec
            .pen
            .selected_paths()
            .iter()
            .filter_map(|id| self.vec.entities.get(id).copied())
            .collect();
        if sel.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else { return };
        let sim = &mut gfx.sim;
        // ⛔⛔ **A RECUSA FALA NA TELA, e não no terminal** (auditoria de 2026-09-06). As duas
        // recusas deste atalho saíam só por `eprintln!`, que o artista **não vê** — e o gémeo
        // deste gesto, o item *Group* do menu da Hierarquia, já respondia com toast. *Um gesto
        // que não faz nada e não diz porquê ensina que a feature está partida*, e foi assim que
        // um smoke desta linha mandou o dono agrupar um objecto só e ficar a olhar para o nada.
        if group {
            let name = tr_with(
                "shell.despacho_metodos_janela_e_vetor.group",
                &[("sel", &(sel.len()))],
            );
            if ph2d_vec_entities::entities::group_entities(sim, &sel, name).is_none() {
                gfx.toasts.push(ph2d_editor_core::Toast::warning(tr(
                    "shell.despacho_metodos_janela_e_vetor.select_two_or_more",
                )));
            }
        } else if ph2d_vec_entities::entities::ungroup_entities(sim, &sel) == 0 {
            gfx.toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.despacho_metodos_janela_e_vetor.that_selection_is_not",
            )));
        }
    }

    /// Vector Ctrl+D: duplicate the object selection (offset ~12 px) — o irmão de
    /// teclado do botão Arrange "Duplicate". Preserva os grupos, como o paste.
    pub(super) fn vec_duplicate_shortcut(&mut self) {
        let (dx, dy) = self.vec_screen_offset(PASTE_OFFSET_PX);
        if let Some(gfx) = self.gfx.as_mut() {
            apply_vec_duplicate(&mut gfx.vec_scene, &mut self.vec.pen, dx, dy);
        }
    }

    /// Apaga TODA a seleção de objeto (um grupo some inteiro) e limpa os grupos
    /// que ficaram sem membro. ONE undo step.
    pub(super) fn vec_delete_selected(&mut self) -> bool {
        let sel = self.vec.pen.selected_paths().to_vec();
        if sel.is_empty() {
            return false;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let mut any = false;
        for id in &sel {
            any |= gfx.vec_scene.remove_path(*id);
        }
        if !any {
            return false;
        }
        self.vec.pen.clear();
        eprintln!("[ph2d-vec] {} path(s) apagado(s)", sel.len());
        true
    }

    /// ADR-0108 cutover: is the Vector drawing tool the active tool? Gates the
    /// Pen input hooks (replaces the retired `PH2D_VEC_PEN` test flag).
    /// Põe a ORIGEM (o pivô) do path selecionado sob o cursor, sem mover a forma.
    /// `false` (e não consome o clique) se não há forma selecionada.
    pub(super) fn vec_set_origin_to_cursor(&mut self, x: f32, y: f32) -> bool {
        let Some(sel) = self.vec.pen.selected() else {
            return false;
        };
        let Some(&bits) = self.vec.entities.get(&sel) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let target = gfx.camera.screen_to_world((x, y), win);
        let moved = ph2d_vec_entities::transform::move_origin_to(
            &mut gfx.sim,
            &mut gfx.vec_scene,
            ph2d_ecs::Entity::from_bits(bits),
            sel,
            target,
        );
        if moved {
            self.title_dirty = true;
        }
        moved
    }
}

/// The double-arrow cursor for a panel-border grip, given its edge bitmask
/// (`TIMELINE_EDGE_*`; a corner sets two bits). Corners point along their own
/// diagonal: the top-left / bottom-right pair is `Nwse` (↖↘), the other `Nesw`.
pub(super) fn resize_cursor_for_edges(edges: u8) -> winit::window::CursorIcon {
    use ph2d_editor_core::interaction::{
        TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R, TIMELINE_EDGE_T,
    };
    use winit::window::CursorIcon;
    let (l, r) = (edges & TIMELINE_EDGE_L != 0, edges & TIMELINE_EDGE_R != 0);
    let (t, b) = (edges & TIMELINE_EDGE_T != 0, edges & TIMELINE_EDGE_B != 0);
    match (l, r, t, b) {
        (true, _, true, _) | (_, true, _, true) => CursorIcon::NwseResize,
        (_, true, true, _) | (true, _, _, true) => CursorIcon::NeswResize,
        (_, _, true, _) | (_, _, _, true) => CursorIcon::NsResize,
        _ => CursorIcon::EwResize,
    }
}
