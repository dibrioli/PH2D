//! Canvas painting input (SHELL-only): Primary Down/Move/Up → the active [`PainterTool`]
//! (`CanvasPaintTool::on_canvas_pointer`, ADR-0040 Am. 3), screen → image px by the FULL sprite
//! affine; the downcast is the ADR-0040 §3 exception. Pressure/tilt come from `ph2d_pen::porta::tinta`
//! (ADR-0189); the `Up` closes with the last contact's pressure (the pen's `Up` carries `0`).

use std::cell::Cell;

use ph2d_app_painter::fila_da_caneta::Evento;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_tool_painter::PainterTool;

use crate::App;

use ph2d_app_painter::canvas_down::canvas_down_accepts;
/// A lei vive em [`ph2d_app_painter::shape_grab`]; re-exportada para `painter_curve_input.rs`.
pub(crate) use ph2d_app_painter::shape_grab::shape_grab_tol_from_affine;

/// A malha sob o cursor, porta ÚNICA da pincelada e do passeio; o raio é o de ANTES da composição.
pub(crate) fn malha_sob_o_cursor(
    present: &mut ph2d_ecs::World,
    bits: u64,
    world: [f32; 2],
    starting: bool,
    raio_px: f32,
    iw: u32,
    ih: u32,
) -> ph2d_render::MeshUv {
    let footprint_uv = [raio_px / iw as f32, raio_px / ih as f32];
    ph2d_render::mesh_uv(present, bits, world, starting, footprint_uv)
}

/// A deformação local da malha (identidade fora), lida pela tinta e pelo anel: a ÚNICA `MeshWarp` → `CanvasWarp`.
pub(crate) fn warp_da_malha(malha: ph2d_render::MeshUv) -> ph2d_tool_painter::CanvasWarp {
    match malha {
        ph2d_render::MeshUv::Use { warp, .. } => ph2d_tool_painter::CanvasWarp {
            linear: warp.linear,
            curve: warp.curve,
        },
        _ => ph2d_tool_painter::CanvasWarp::rest(),
    }
}

thread_local! {
    /// `true` between a consumed painter Down and its Up (or the lost Up: `perde_a_mao`).
    static STROKE_ACTIVE: Cell<bool> = const { Cell::new(false) };
    /// A pressão do último contacto do traço aberto — a do `Up`.
    static STROKE_PRESSURE: Cell<f32> = const { Cell::new(1.0) };
    /// A ponta de borracha ligou a borracha do Painter neste traço: o `Up` desliga-a.
    static PEN_ERASER: Cell<bool> = const { Cell::new(false) };
}

/// Para os gates da porta da caneta (`input_drop::caneta`): abre um traço e lê a pressão dele.
#[cfg(test)]
pub(crate) fn traco_de_teste(abre: bool) -> f32 {
    STROKE_ACTIVE.with(|s| s.set(abre));
    STROKE_PRESSURE.with(Cell::get)
}

#[cfg(test)]
pub(crate) fn traco_esta_aberto() -> bool {
    STROKE_ACTIVE.with(Cell::get)
}

impl App {
    /// The active tool as [`PainterTool`] (ADR-0040 §3 downcast); `None` when not the Painter.
    pub(crate) fn painter_tool_mut(&mut self) -> Option<&mut PainterTool> {
        self.gfx
            .as_mut()?
            .tools
            .active_mut()?
            .as_any_mut()
            .downcast_mut::<PainterTool>()
    }

    /// Primary Down → [`PointerPhase::Down`]; `true` (consumes) iff a stroke started.
    pub(crate) fn painter_canvas_down(&mut self, px: f32, py: f32, pressure: f32) -> bool {
        // A press over the CHROME is its widgets'; an open stroke keeps painting over it (Move).
        if crate::chrome_hit::pointer_over_chrome(self.gfx.as_ref(), px, py) {
            return false;
        }
        // A ponta de BORRACHA liga a borracha pelo botão (a macro grava-o), só neste traço.
        let mut borracha = false;
        if ph2d_pen::porta::tinta().borracha()
            && let Some(p) = self.painter_tool_mut()
            && !p.brush_eraser()
        {
            p.toggle_brush_eraser();
            borracha = true;
        }
        let started = self.deliver_canvas_pointer(px, py, pressure, PointerPhase::Down);
        if started {
            STROKE_ACTIVE.with(|s| s.set(true));
            STROKE_PRESSURE.with(|s| s.set(pressure));
        }
        PEN_ERASER.with(|e| e.set(borracha));
        if !started {
            self.pen_eraser_end();
        }
        started
    }

    fn pen_eraser_end(&mut self) {
        if PEN_ERASER.with(|e| e.replace(false))
            && let Some(p) = self.painter_tool_mut()
        {
            p.toggle_brush_eraser();
        }
    }

    /// CursorMoved over an open stroke → [`PointerPhase::Move`]; re-stamp methods are BUFFERED (latest
    /// wins, [`Self::flush_pending_painter_move`]); incremental ones are QUEUED (every event, B27.1).
    pub(crate) fn painter_canvas_move(&mut self, px: f32, py: f32, pressure: f32) -> bool {
        if !STROKE_ACTIVE.with(Cell::get) {
            // HOVER: feeds the brush ring's orientation; consumes nothing.
            self.deliver_canvas_hover(px, py);
            return false;
        }
        if self.painter_coalesces_motion() {
            self.pending_painter_move = Some((px, py, pressure));
            return true; // delivered at the frame flush
        }
        // A mid-stroke method flip must not leave a stale buffered move for the flush.
        self.pending_painter_move = None;
        STROKE_PRESSURE.with(|s| s.set(pressure));
        ph2d_app_painter::paint_perf::stamp_pointer();
        ph2d_app_painter::fila_da_caneta::enfileira(self.evento_vivo(px, py, pressure));
        true
    }

    /// Pressão, tilt e modificadores como a entrega os leria AGORA.
    fn evento_vivo(&self, px: f32, py: f32, pressure: f32) -> Evento {
        let m = self.modifiers;
        let mods = [m.shift_key(), m.control_key() || m.super_key(), m.alt_key()];
        Evento::novo((px, py), pressure, ph2d_pen::porta::tinta().tilt(), mods)
    }

    /// BARREIRA (todo `WindowEvent` ≠ `CursorMoved`, o `Up`) e topo do quadro: cada Move enfileirado, na
    /// ordem; um recusado abandona o traço (`false`). Abaixo dela, o `window_event` é a tabela pura (PR 9b).
    pub(crate) fn drena_a_caneta(&mut self) -> bool {
        let _ = self.painter_tool_mut().map(|p| p.abre_lote()); // B27.2: os Moves do quadro num lote
        let fila = ph2d_app_painter::fila_da_caneta::drena;
        let d = fila(|e| self.entrega(e, PointerPhase::Move, None, false));
        let us = d.us + self.painter_tool_mut().map_or(0, |p| p.fecha_lote());
        self.paint_stamp_us_this_frame = self.paint_stamp_us_this_frame.saturating_add(us);
        self.paint_stamps_this_frame = self.paint_stamps_this_frame.saturating_add(d.entregues);
        STROKE_ACTIVE.with(|s| s.set(s.get() && d.vivo)); // morto = Painter/seleção já não estão
        d.vivo
    }

    /// The Painter is in a method whose Moves may be coalesced to one delivery per frame.
    fn painter_coalesces_motion(&mut self) -> bool {
        self.painter_tool_mut()
            .is_some_and(|p| p.coalesces_canvas_motion())
    }

    /// A **hover** in image px: [`Self::deliver_canvas_pointer`] without gesture state; never consumes.
    fn deliver_canvas_hover(&mut self, px: f32, py: f32) {
        let Some(gfx) = self.gfx.as_mut() else { return };
        if !gfx
            .tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("painter"))
        {
            return;
        }
        let Some(bits) = gfx.hero_screen.as_ref().and_then(|h| h.gizmo.selection) else {
            return;
        };
        let entity = ph2d_ecs::Entity::from_bits(bits);
        // ⚠️ Pose de MUNDO — vide o doc do `sprite_image_to_screen_affine`.
        let (Some(tr), Some(sprite)) = (
            ph2d_ecs::world_transform(gfx.sim.world(), entity),
            ph2d_sprite_screen::sprite_mostrada(gfx.sim.world(), entity),
        ) else {
            return;
        };
        // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula.
        let sprite_grid = gfx.sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
        let window_size = gfx.scene_window();
        let camera = gfx.camera;
        let Some((iw, ih, raio_px)) = gfx
            .tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .map(|p| {
                let (w, h) = p.canvas_size();
                (w, h, p.dab_footprint_px())
            })
        else {
            return;
        };
        if iw == 0 || ih == 0 {
            return;
        }
        // O empréstimo do Painter larga-se aqui: a malha precisa do mundo de PRESENTE do mesmo `gfx`.
        let world = camera.screen_to_world((px, py), window_size);
        let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
            iw,
            ih,
            tr,
            sprite,
            sprite_grid,
            &camera,
            window_size,
        );
        let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
        // O passeio pergunta à malha como a pincelada; `starting = false`: nunca abre um traço.
        let malha =
            malha_sob_o_cursor(gfx.present.world_mut(), bits, world, false, raio_px, iw, ih);
        let Some(painter) = self.painter_tool_mut() else {
            return;
        };
        painter.set_canvas_warp(warp_da_malha(malha));
        let (hx, hy) = match malha {
            // A UV de repouso ALI — a mesma resposta que a pincelada usa.
            ph2d_render::MeshUv::Use { u, v, .. } => (u * iw as f32, v * ih as f32),
            _ => (img.x as f32, img.y as f32),
        };
        painter.on_canvas_hover([hx, hy]);
    }

    /// Deliver the buffered painter Move, if any — once per frame and before a pen-up.
    pub(crate) fn flush_pending_painter_move(&mut self) {
        let vivo = self.drena_a_caneta();
        let Some((px, py, pressure)) = self.pending_painter_move.take().filter(|_| vivo) else {
            return; // nothing buffered — queued stamps are timed in `drena_a_caneta`
        };
        let t0 = std::time::Instant::now();
        STROKE_PRESSURE.with(|s| s.set(pressure));
        let delivered = self.deliver_canvas_pointer(px, py, pressure, PointerPhase::Move);
        self.paint_stamp_us_this_frame = self
            .paint_stamp_us_this_frame
            .saturating_add(t0.elapsed().as_micros() as u64);
        if delivered {
            self.paint_stamps_this_frame = self.paint_stamps_this_frame.saturating_add(1);
        } else {
            STROKE_ACTIVE.with(|s| s.set(false));
        }
    }

    /// Primary Up: close any open painter stroke. No-op when not painting.
    pub(crate) fn painter_canvas_up(&mut self) {
        if STROKE_ACTIVE.with(Cell::get) {
            // The buffered final position is stamped BEFORE the commit.
            self.flush_pending_painter_move();
            STROKE_ACTIVE.with(|s| s.set(false));
            let (px, py) = self.last_pointer;
            let pressure = STROKE_PRESSURE.with(Cell::get);
            self.deliver_canvas_pointer(px, py, pressure, PointerPhase::Up);
        }
        self.pen_eraser_end();
    }

    /// One [`CanvasPointer`] in image px: the tool's `consumed`, or `false` when not delivered.
    pub(crate) fn deliver_canvas_pointer(
        &mut self,
        px: f32,
        py: f32,
        pressure: f32,
        phase: PointerPhase,
    ) -> bool {
        self.deliver_canvas_pointer_colordrop(px, py, pressure, phase, None)
    }

    /// [`Self::deliver_canvas_pointer`] for the C&F drag (`colordrop` = the mode to return to).
    pub(crate) fn deliver_canvas_pointer_colordrop(
        &mut self,
        px: f32,
        py: f32,
        pressure: f32,
        phase: PointerPhase,
        colordrop: Option<&str>,
    ) -> bool {
        self.entrega(self.evento_vivo(px, py, pressure), phase, colordrop, true)
    }

    /// A entrega de um [`Evento`]; `chega` = carimbar a chegada aqui (a fila carimbou-a no handler).
    fn entrega(&mut self, e: Evento, phase: PointerPhase, cd: Option<&str>, chega: bool) -> bool {
        let ((px, py), pressure, colordrop) = (e.pos, e.pressure, cd);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let painter_active = gfx
            .tools
            .active()
            .map(|t| t.id() == ph2d_editor_core::ToolId::new("painter"))
            .unwrap_or(false);
        if !painter_active {
            return false;
        }
        let Some(hero) = gfx.hero_screen.as_ref() else {
            return false;
        };
        let Some(bits) = hero.gizmo.selection else {
            return false;
        };
        // Pose de MUNDO: com a local, uma sprite FILHA recusaria toda pincelada.
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let (Some(tr), Some(sprite)) = (
            ph2d_ecs::world_transform(gfx.sim.world(), entity),
            ph2d_sprite_screen::sprite_mostrada(gfx.sim.world(), entity),
        ) else {
            return false;
        };
        // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula.
        let sprite_grid = gfx.sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
        let window_size = gfx.scene_window();
        // A arte dobrada manda no ponteiro (lei em `ph2d_render::mesh_uv`); `Quad` = o afim de sempre.
        let world = gfx.camera.screen_to_world((px, py), window_size);
        let Some(tool) = gfx.tools.active_mut() else {
            return false;
        };
        let Some(painter) = tool.as_any_mut().downcast_mut::<PainterTool>() else {
            return false;
        };
        let (iw, ih) = painter.canvas_size();
        if iw == 0 || ih == 0 {
            return false;
        }
        let raio_px = painter.dab_footprint_px();
        let malha = malha_sob_o_cursor(
            gfx.present.world_mut(),
            bits,
            world,
            phase == PointerPhase::Down,
            raio_px,
            iw,
            ih,
        );
        if malha == ph2d_render::MeshUv::Refuse {
            // Um traço não NASCE sobre o quad de repouso de uma arte que se desenha dobrada.
            return false;
        }
        // `u`/`v` is the image fraction, NOT clamped (a Repeat-Image neighbour lands outside `[0, 1]`).
        let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
            iw,
            ih,
            tr,
            sprite,
            sprite_grid,
            &gfx.camera,
            window_size,
        );
        let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
        // A FORMA do dab também é da malha (`ph2d_painter_brush::canvas_warp`).
        painter.set_canvas_warp(warp_da_malha(malha));
        let (u, v) = match malha {
            ph2d_render::MeshUv::Use { u: mu, v: mv, .. } => (mu, mv), // a UV de repouso ALI
            _ => (
                (img.x / f64::from(iw)) as f32,
                (img.y / f64::from(ih)) as f32,
            ),
        };
        // Only a Down is gated by the footprint (Move/Up reach an open stroke past the edge). Repeat +
        // Tiling extends it to the 3×3 grid (coordinate UN-wrapped); a live Deform grants a grab margin
        // so its corner squares win over the sprite gizmo.
        let tol = shape_grab_tol_from_affine(&affine);
        painter.set_shape_grab_tol_px(tol);
        let margin_uv = {
            // Fill has no gizmo: the ColorDrop's Down is judged as Fill would judge it.
            let m = if colordrop.is_some() {
                0.0
            } else {
                painter.deform_gizmo_grab_margin_px()
            };
            [m / iw as f32, m / ih as f32]
        };
        let repeat = painter.repeat_image();
        let tiling = painter.brush_tiling();
        let in_x = canvas_down_accepts(u, repeat && tiling[0], margin_uv[0]);
        let in_y = canvas_down_accepts(v, repeat && tiling[1], margin_uv[1]);
        if phase == PointerPhase::Down && !(in_x && in_y) {
            return false;
        }
        if let Some(volta) = colordrop
            && phase == PointerPhase::Down
        {
            painter.begin_colordrop_fill(volta);
        }
        let ev = CanvasPointer {
            pos: [u * iw as f32, v * ih as f32],
            pressure,
            tilt: e.tilt,
            phase,
        };
        super::painter_canvas_mods::forward(painter, e.shift, e.ctrl, e.alt);
        // Grid snap (only when ON) for shape point placement: nearest grid node in WORLD → image px.
        let grid_snap = if gfx
            .hero_screen
            .as_ref()
            .is_some_and(|h| h.grid.snap_state.snap_enabled)
        {
            let world = gfx.camera.screen_to_world((px, py), window_size);
            let snapped = gfx
                .hero_screen
                .as_mut()
                .expect("hero present")
                .grid
                .snap_state
                .snap_world(world, [0.0, 0.0]);
            let (sx, sy) = gfx.camera.world_to_screen(snapped, window_size);
            let gimg = affine.inverse() * ph2d_vector::Point::new(f64::from(sx), f64::from(sy));
            Some([gimg.x as f32, gimg.y as f32])
        } else {
            None
        };
        painter.set_grid_snap(grid_snap);
        // Carimbo da chegada DEPOIS das guardas (um Down recusado nunca vira pixel) e o custo da
        // entrega, que corre fora do `run_render_frame`.
        chega.then(ph2d_app_painter::paint_perf::stamp_pointer);
        let t0 = ph2d_app_painter::paint_perf::on().then(std::time::Instant::now);
        let phase = ph2d_app_painter::paint_perf::InputPhase::of(ev.phase);
        let consumed = painter.on_canvas_pointer(ev);
        if let Some(t0) = t0 {
            let ms = t0.elapsed().as_secs_f64() as f32 * 1e3;
            ph2d_app_painter::paint_perf::record_input(ms, phase);
        }
        consumed
    }

    /// Enter: commit the in-progress shape. `true` (consumes) iff a session was open.
    pub(crate) fn painter_shape_commit(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        // A caneta da seleção vem ANTES do Offset: é a sessão mais interna.
        if painter.selection_pen_live() {
            painter.selection_pen_commit();
            return true;
        }
        // A live Selection Offset takes Enter as its Apply.
        if painter.selection_offset_engaged() {
            painter.selection_offset_apply();
            return true;
        }
        painter.commit_open_shape()
    }

    /// Right-click while a Line is being drawn ends point creation; `true` (no context menu) iff it did.
    pub(crate) fn painter_line_finish_points(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        painter.line_finish_points()
    }

    /// Esc: discard the in-progress shape (revert the preview). Returns `true` (consuming) iff a
    /// session was open.
    pub(crate) fn painter_shape_cancel(&mut self) -> bool {
        let Some(painter) = self.painter_tool_mut() else {
            return false;
        };
        // A caneta (sessão mais interna) descarta primeiro, sem gastar undo.
        if painter.selection_pen_cancel() {
            return true;
        }
        painter.cancel_open_shape()
    }
}
