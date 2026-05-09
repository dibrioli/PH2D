#![forbid(unsafe_code)]
#![allow(deprecated)] // egui 0.34 still ships these aliases; refactor to Panel::left etc. is a follow-up
#![allow(dead_code)]
//! Desktop shell — winit 0.30 + wgpu + ECS + sprite render + egui.
//!
//! Run with: `cargo run -p ph2d-host-desktop` (or `./play`).
//!
//! Layered subsystems:
//! - **M5** SpriteRenderer + 1000-sprite Vogel spiral with bouncing
//! - **M6** AssetDb loads 16 procedural PNG fixtures from
//!   `assets/sprites/` (auto-generated on first launch)
//! - **M7** ScriptHost with placeholder Luau script; per-frame gc_step
//! - **M8** gilrs gamepad → ph2d_input::InputState → ScriptHost's
//!   ph2d.input snapshot (Luau readable each frame)
//! - **M12** editor data layer (ToolRegistry / ZenMode / ToastQueue)
//!   rendered via **egui** — replaces the previous custom Vello widget
//!   paint per ADR-0024 (egui pivot for velocity).
//!
//! Editor demo:
//!   - Left side bar shows tool palette: Brush + Move
//!   - Click a tool → its panel renders on the bottom with real
//!     widgets (sliders/toggles/radio) bound to the tool's model.
//!   - Tab toggles Zen mode (hides editor chrome)
//!   - M flips theme (Dark/Light) — egui Visuals follow ph2d-tokens

mod integration;

use ph2d_asset::AssetDb;
use ph2d_core::{FixedStep, Vec2, install_panic_hook, panic};
use ph2d_ecs::{Component, PresentWorld, SimComponent, SimWorld};
use ph2d_editor::{
    BrushTool, MoveTool, PanelControl, PanelEvent, Toast, ToastQueue, ToolId, ToolRegistry, ZenMode,
};
use ph2d_gpu::{AcquireError, GpuContext, SurfaceContext};
use ph2d_host::{
    CloseAction, HostHandler, KeyEvent, KeyKind, Lifecycle, Modifiers, PlatformHost, PointerEvent,
    PointerKind, PointerSource, WindowSize,
};
use ph2d_input::InputState;
use ph2d_render::{Camera2d, RenderInstance, Sprite, SpriteRenderer, TextureAtlas};
use ph2d_script::ScriptHost;
use ph2d_tokens::Theme;
use std::cell::Cell;
use std::sync::Arc;
use std::time::Instant;

mod gilrs_adapter;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent as WinitKeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Window, WindowId};

const SPRITE_COUNT: u32 = 1000;
const WORLD_HALF: f32 = 5.0;

#[derive(Component, Copy, Clone, Debug)]
struct Position(Vec2);
impl SimComponent for Position {}

#[derive(Component, Copy, Clone, Debug)]
struct Velocity(Vec2);
impl SimComponent for Velocity {}

struct WinitHost {
    window: Arc<Window>,
    scale: Cell<f32>,
}

impl WinitHost {
    fn new(window: Arc<Window>) -> Self {
        let scale = window.scale_factor() as f32;
        Self {
            window,
            scale: Cell::new(scale),
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }
}

impl PlatformHost for WinitHost {
    fn request_redraw(&self) {
        self.window.request_redraw();
    }
    fn window_size(&self) -> WindowSize {
        let size = self.window.inner_size();
        WindowSize::new(size.width, size.height)
    }
    fn scale_factor(&self) -> f32 {
        self.scale.get()
    }
}

struct LoggingHandler {
    started_at: Instant,
}
impl LoggingHandler {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
    fn elapsed_ms(&self) -> u128 {
        self.started_at.elapsed().as_millis()
    }
}
impl HostHandler for LoggingHandler {
    fn on_resize(&mut self, size: WindowSize, scale_factor: f32) {
        println!(
            "[{:>6}ms] resize: {}x{} @ {:.2}x scale",
            self.elapsed_ms(),
            size.width,
            size.height,
            scale_factor
        );
    }
    fn on_lifecycle(&mut self, kind: Lifecycle) {
        println!("[{:>6}ms] lifecycle: {:?}", self.elapsed_ms(), kind);
    }
    fn on_pointer(&mut self, _event: PointerEvent) {}
    fn on_key(&mut self, _event: KeyEvent) {}
    fn on_close_request(&mut self) -> CloseAction {
        println!("[{:>6}ms] close requested → Close", self.elapsed_ms());
        CloseAction::Close
    }
}

/// Egui plumbing: context + winit translator + wgpu renderer.
/// Created once `resumed` brings up the surface.
struct EguiSystem {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EguiSystem {
    fn new(window: &Window, surface: &SurfaceContext) -> Self {
        let ctx = egui::Context::default();
        let viewport_id = ctx.viewport_id();
        let state = egui_winit::State::new(
            ctx.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let renderer = egui_wgpu::Renderer::new(
            &surface.gpu().device,
            surface.format(),
            egui_wgpu::RendererOptions::default(),
        );
        Self {
            ctx,
            state,
            renderer,
        }
    }
}

struct AppGfx {
    surface: SurfaceContext,
    renderer: SpriteRenderer,
    sim: SimWorld,
    present: PresentWorld,
    camera: Camera2d,
    asset_db: AssetDb,
    atlas_is_real: bool,
    script: Option<ScriptHost>,
    theme: Theme,
    zen: ZenMode,
    toasts: ToastQueue,
    tools: ToolRegistry,
    egui: EguiSystem,
}

struct App {
    window: Option<Arc<Window>>,
    host: Option<WinitHost>,
    gfx: Option<AppGfx>,
    handler: LoggingHandler,
    fixed_step: FixedStep,
    last_frame: Instant,
    pending_resize: Option<WindowSize>,
    modifiers: ModifiersState,
    last_pointer: (f32, f32),
    /// gilrs context (M8). `None` if init failed (e.g. CI sandbox).
    gilrs: Option<gilrs::Gilrs>,
    input: InputState,
}

impl App {
    fn new() -> Self {
        let gilrs = match gilrs::Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("gilrs init failed (continuing without gamepad): {e}");
                None
            }
        };
        Self {
            window: None,
            host: None,
            gfx: None,
            handler: LoggingHandler::new(),
            fixed_step: FixedStep::default(),
            last_frame: Instant::now(),
            pending_resize: None,
            modifiers: ModifiersState::default(),
            last_pointer: (0.0, 0.0),
            gilrs,
            input: InputState::new(),
        }
    }

    fn convert_modifiers(state: ModifiersState) -> Modifiers {
        Modifiers {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            meta: state.super_key(),
        }
    }

    fn timestamp_ns() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }

    /// M12 demo controls (key path; egui handles its own widget input).
    /// Tab → ZenMode toggle. M → flip theme.
    fn handle_editor_key(&mut self, code: KeyCode) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        match code {
            KeyCode::Tab if gfx.zen.try_toggle() => {
                let msg = if gfx.zen.is_active() {
                    "Zen mode ON"
                } else {
                    "Zen mode OFF"
                };
                gfx.toasts.push(Toast::info(msg));
            }
            KeyCode::KeyM => {
                gfx.theme = gfx.theme.toggle();
                gfx.toasts
                    .push(Toast::info(format!("Theme → {:?}", gfx.theme)));
            }
            KeyCode::Digit1 if gfx.tools.set_active(&ToolId::new("brush")) => {
                gfx.toasts.push(Toast::info("Tool → Brush"));
            }
            KeyCode::Digit2 if gfx.tools.set_active(&ToolId::new("move")) => {
                gfx.toasts.push(Toast::info("Tool → Move"));
            }
            _ => {}
        }
    }

    /// Snapshot `InputState` into the ScriptHost's `ph2d.input` table
    /// (M8 → Luau loop). Per the M8 ph2d_input_resolves test convention.
    fn push_input_to_script(&self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let Some(host) = gfx.script.as_ref() else {
            return;
        };
        host.clear_input();
        for button in self.input.gamepad.iter_held() {
            let key = format!("gamepad.held.{}", button.as_lua_key());
            host.provide_input(&key, 1.0);
        }
        for (axis, value) in self.input.gamepad.iter_axes() {
            let key = format!("gamepad.axis.{}", axis.as_lua_key());
            host.provide_input(&key, value as f64);
        }
    }

    fn try_load_real_atlas(
        gpu: &GpuContext,
        asset_db: &AssetDb,
        dir: &std::path::Path,
    ) -> Result<TextureAtlas, String> {
        let created = integration::ensure_demo_assets_exist(dir)
            .map_err(|e| format!("ensure_demo_assets_exist({}): {e}", dir.display()))?;
        if created > 0 {
            println!(
                "M6: generated {created} demo PNG fixtures in {}",
                dir.display()
            );
        }
        let ids = integration::load_demo_assets(asset_db, dir)?;
        let rgba = integration::compose_atlas_rgba(asset_db, &ids)?;
        Ok(TextureAtlas::from_rgba8(
            gpu,
            integration::ATLAS_PX,
            integration::ATLAS_PX,
            &rgba,
        ))
    }

    fn populate_sim(sim: &mut SimWorld) {
        for i in 0..SPRITE_COUNT {
            let f = i as f32;
            let angle = f * 2.399_963_2;
            let r = (f / SPRITE_COUNT as f32).sqrt() * (WORLD_HALF - 0.5);
            let pos = Vec2::new(r * angle.cos(), r * angle.sin());
            let vx = ((f * 12.9898).sin() * 43758.547).fract() * 3.0 - 1.5;
            let vy = ((f * 78.233).sin() * 12345.678).fract() * 3.0 - 1.5;
            sim.world_mut().spawn((
                Position(pos),
                Velocity(Vec2::new(vx, vy)),
                Sprite {
                    atlas_index: i % 16,
                    size: [0.18, 0.18],
                    tint: [1.0, 1.0, 1.0, 1.0],
                },
            ));
        }
    }

    /// Pump every queued gilrs event into the [`InputState`].
    fn pump_gamepad(&mut self) {
        let Some(g) = self.gilrs.as_mut() else {
            return;
        };
        self.input.begin_frame();
        while let Some(gilrs::Event { event, time: _, .. }) = g.next_event() {
            match event {
                gilrs::EventType::Connected | gilrs::EventType::Disconnected => {}
                _ => {
                    if let Some(translated) = gilrs_adapter::translate(event) {
                        self.input.apply_event(translated);
                    }
                }
            }
        }
    }

    fn render_frame(&mut self) {
        self.pump_gamepad();
        self.push_input_to_script();

        let Some(host) = self.host.as_ref() else {
            return;
        };
        let window = host.window();
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };

        // Apply coalesced resize once per frame.
        if let Some(size) = self.pending_resize.take() {
            gfx.surface.resize(size);
            self.handler.on_resize(size, host.scale_factor());
        }

        // M7 per-frame GC step (cheap; keeps Luau heap warm).
        if let Some(host) = gfx.script.as_ref() {
            let _ = host.gc_step();
        }

        // Drive fixed-step.
        let now = Instant::now();
        let wall_dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        let report = self.fixed_step.advance(wall_dt);
        if report.dropped_secs > 0.0 {
            eprintln!(
                "warn: dropped {:.3}s of sim time (max_substeps cap)",
                report.dropped_secs
            );
        }
        panic::set_frame_id(self.fixed_step.tick_count());

        // Sim tick.
        let dt = self.fixed_step.fixed_dt() as f32;
        {
            let mut q = gfx
                .sim
                .world_mut()
                .query::<(&mut Position, &mut Velocity)>();
            for (mut pos, mut vel) in q.iter_mut(gfx.sim.world_mut()) {
                let mut p = pos.0;
                let mut v = vel.0;
                p += v * dt;
                if p.x.abs() > WORLD_HALF {
                    v.x = -v.x;
                    p.x = p.x.clamp(-WORLD_HALF, WORLD_HALF);
                }
                if p.y.abs() > WORLD_HALF {
                    v.y = -v.y;
                    p.y = p.y.clamp(-WORLD_HALF, WORLD_HALF);
                }
                pos.0 = p;
                vel.0 = v;
            }
        }

        // Extract sim → present.
        let atlas = gfx.renderer.atlas();
        let mut sim_q = gfx.sim.world_mut().query::<(&Position, &Sprite)>();
        ph2d_ecs::extract!(gfx.sim => gfx.present, |sim_w, present_w| {
            present_w.clear_entities();
            for (pos, spr) in sim_q.iter(sim_w) {
                present_w.spawn(RenderInstance {
                    world_pos: [pos.0.x, pos.0.y],
                    size: spr.size,
                    atlas_uv: atlas.dummy_uv(spr.atlas_index),
                    tint: spr.tint,
                });
            }
        });

        // Per-frame editor ticks.
        gfx.zen.tick();
        gfx.toasts.tick();

        // Animated background tint.
        let t = self.fixed_step.tick_count() as f64 * self.fixed_step.fixed_dt();
        let r = (t.sin() * 0.05 + 0.05).clamp(0.0, 1.0);
        let g = ((t + 2.094).sin() * 0.05 + 0.05).clamp(0.0, 1.0);
        let b = ((t + 4.188).sin() * 0.05 + 0.05).clamp(0.0, 1.0);

        // Build the egui frame BEFORE acquiring the surface so an
        // Occluded/Timeout doesn't waste the encoder.
        let raw_input = gfx.egui.state.take_egui_input(window);
        let panel_event_cell: std::cell::RefCell<Option<PanelEvent>> =
            std::cell::RefCell::new(None);
        let active_label = gfx
            .tools
            .active()
            .map(|t| t.label().to_string())
            .unwrap_or_default();
        let active_id = gfx.tools.active().map(|t| t.id());
        let mut next_active: Option<ToolId> = None;
        let zen_active = gfx.zen.is_active();
        let theme = gfx.theme;
        apply_theme(&gfx.egui.ctx, theme);
        let active_panel = gfx.tools.active().map(|t| t.build_panel());
        let toast_msgs: Vec<(String, ph2d_editor::ToastSeverity)> = gfx
            .toasts
            .iter()
            .map(|t| (t.message.clone(), t.severity))
            .collect();

        let full_output = gfx.egui.ctx.run(raw_input, |ctx| {
            if zen_active {
                return; // hide all chrome
            }
            // Tool palette in a left side panel.
            egui::SidePanel::left("tool_palette")
                .exact_width(80.0)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.heading("Tools");
                    ui.add_space(8.0);
                    if ui
                        .selectable_label(active_label == "Brush", "Brush")
                        .clicked()
                    {
                        next_active = Some(ToolId::new("brush"));
                    }
                    if ui
                        .selectable_label(active_label == "Move", "Move")
                        .clicked()
                    {
                        next_active = Some(ToolId::new("move"));
                    }
                });

            // Active tool's panel at the bottom.
            if let Some(panel) = &active_panel {
                egui::TopBottomPanel::bottom("tool_panel")
                    .resizable(false)
                    .min_height(120.0)
                    .show(ctx, |ui| {
                        ui.heading(&panel.title);
                        for tab in &panel.tabs {
                            let _ = ui.selectable_label(tab.active, &tab.label);
                        }
                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            for ctrl in &panel.controls {
                                ui.vertical(|ui| {
                                    ui.label(ctrl.label());
                                    match ctrl {
                                        PanelControl::Slider(s) => {
                                            let mut v = s.value;
                                            if ui
                                                .add(
                                                    egui::Slider::new(&mut v, 0.0..=1.0)
                                                        .show_value(true),
                                                )
                                                .changed()
                                            {
                                                *panel_event_cell.borrow_mut() =
                                                    Some(PanelEvent::SetValue(s.id, v as f64));
                                            }
                                        }
                                        PanelControl::Toggle(t) => {
                                            let mut on = t.on;
                                            if ui.checkbox(&mut on, "").changed() {
                                                *panel_event_cell.borrow_mut() =
                                                    Some(PanelEvent::Toggle(t.id, on));
                                            }
                                        }
                                        PanelControl::RadioGroup(g) => {
                                            for opt in &g.options {
                                                let selected =
                                                    g.selected.as_ref() == Some(&opt.value);
                                                if ui.radio(selected, &opt.label).clicked() {
                                                    *panel_event_cell.borrow_mut() =
                                                        Some(PanelEvent::SelectOption(
                                                            g.id,
                                                            opt.value.clone(),
                                                        ));
                                                }
                                            }
                                        }
                                        PanelControl::ColorSwatch(s) => {
                                            let mut rgba = s.rgba;
                                            let mut srgba = [rgba[0], rgba[1], rgba[2], rgba[3]];
                                            if ui
                                                .color_edit_button_srgba_unmultiplied(&mut srgba)
                                                .changed()
                                            {
                                                rgba = srgba;
                                                let _ = rgba; // ColorSwatch event surface is just Click for now
                                                *panel_event_cell.borrow_mut() =
                                                    Some(PanelEvent::Click(s.id));
                                            }
                                        }
                                        PanelControl::Action(a) => {
                                            if ui.button(&a.label).clicked() {
                                                // Action click — no PanelEvent variant for it yet
                                            }
                                        }
                                    }
                                });
                            }
                        });
                    });
            }

            // Toasts as a top floating area.
            egui::Area::new(egui::Id::new("toasts"))
                .anchor(egui::Align2::CENTER_TOP, [0.0, 16.0])
                .show(ctx, |ui| {
                    for (msg, sev) in &toast_msgs {
                        ui.colored_label(severity_color(*sev), msg);
                    }
                });
        });

        let _ = active_id;

        // Acquire surface + sprite pass + egui pass.
        let window_size = gfx.surface.size();
        match gfx.surface.acquire_frame() {
            Ok(frame) => {
                gfx.renderer.render(
                    &frame,
                    &mut gfx.present,
                    &gfx.camera,
                    window_size,
                    wgpu::Color { r, g, b, a: 1.0 },
                );

                // egui pass: paint chrome on top.
                let pixels_per_point = window.scale_factor() as f32;
                gfx.egui
                    .state
                    .handle_platform_output(window, full_output.platform_output);
                let paint_jobs = gfx
                    .egui
                    .ctx
                    .tessellate(full_output.shapes, pixels_per_point);
                let screen = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [window_size.width, window_size.height],
                    pixels_per_point,
                };
                for (id, image_delta) in &full_output.textures_delta.set {
                    gfx.egui.renderer.update_texture(
                        &gfx.surface.gpu().device,
                        &gfx.surface.gpu().queue,
                        *id,
                        image_delta,
                    );
                }
                let mut encoder = gfx.surface.gpu().device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("egui encoder"),
                    },
                );
                gfx.egui.renderer.update_buffers(
                    &gfx.surface.gpu().device,
                    &gfx.surface.gpu().queue,
                    &mut encoder,
                    &paint_jobs,
                    &screen,
                );
                {
                    let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("egui pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: frame.view(),
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    gfx.egui
                        .renderer
                        .render(&mut pass.forget_lifetime(), &paint_jobs, &screen);
                }
                for id in &full_output.textures_delta.free {
                    gfx.egui.renderer.free_texture(id);
                }
                gfx.surface.gpu().queue.submit([encoder.finish()]);
            }
            Err(AcquireError::AwaitingReconfigure) => {
                gfx.surface.reconfigure_after_lost();
            }
            Err(AcquireError::Occluded) => {}
            Err(AcquireError::Timeout) => {}
            Err(AcquireError::Other(s)) => {
                eprintln!("acquire_frame other error: {s}");
            }
        }

        // Apply panel event from this frame's egui ui.
        if let Some(event) = panel_event_cell.into_inner()
            && let Some(active) = gfx.tools.active_mut()
        {
            active.handle_panel_event(event);
        }
        // Tool switch from palette click.
        if let Some(id) = next_active
            && gfx.tools.set_active(&id)
        {
            gfx.toasts.push(Toast::info(format!("Tool → {:?}", id.0)));
        }

        host.request_redraw();
    }
}

/// Map a ph2d-tokens Theme onto egui's Visuals so the editor follows
/// our token system (a future PR can expand this to map every
/// `ColorToken` onto egui's `Style`).
fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    ctx.set_visuals(visuals);
}

fn severity_color(sev: ph2d_editor::ToastSeverity) -> egui::Color32 {
    use ph2d_editor::ToastSeverity::*;
    match sev {
        Info => egui::Color32::from_rgb(0x29, 0xB6, 0xF6),
        Success => egui::Color32::from_rgb(0x4C, 0xAF, 0x50),
        Warning => egui::Color32::from_rgb(0xFF, 0xB3, 0x00),
        ErrorState => egui::Color32::from_rgb(0xEF, 0x53, 0x50),
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("PH2D — desktop shell (egui)")
            .with_inner_size(winit::dpi::LogicalSize::new(1024, 768));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("create_window must succeed"),
        );
        let host = WinitHost::new(window.clone());
        let size = host.window_size();
        let scale = host.scale_factor();

        let instance = GpuContext::default_instance();
        let raw_surface = instance
            .create_surface(window.clone())
            .expect("create_surface");
        let gpu = GpuContext::new(instance, Some(&raw_surface)).expect("GpuContext::new");
        let surface = SurfaceContext::new(gpu, raw_surface, size).expect("SurfaceContext::new");

        let asset_db = AssetDb::new();
        let assets_dir = integration::demo_assets_dir();
        let (atlas, atlas_is_real) =
            match Self::try_load_real_atlas(surface.gpu(), &asset_db, &assets_dir) {
                Ok(atlas) => {
                    println!(
                        "[{:>6}ms] M6: real atlas composed ({} assets cached)",
                        self.handler.elapsed_ms(),
                        asset_db.len_assets()
                    );
                    (atlas, true)
                }
                Err(e) => {
                    eprintln!(
                        "[{:>6}ms] M6 fallback to dummy atlas: {e}",
                        self.handler.elapsed_ms()
                    );
                    (TextureAtlas::dummy(surface.gpu()), false)
                }
            };
        let renderer = SpriteRenderer::new(
            surface.gpu().clone(),
            surface.format(),
            atlas,
            SPRITE_COUNT.next_power_of_two(),
        );

        let mut sim = SimWorld::new();
        Self::populate_sim(&mut sim);
        let present = PresentWorld::new();
        let camera = Camera2d::default();

        let script = match integration::init_script_host() {
            Ok(host) => {
                println!(
                    "[{:>6}ms] M7: ScriptHost initialized",
                    self.handler.elapsed_ms()
                );
                Some(host)
            }
            Err(e) => {
                eprintln!(
                    "[{:>6}ms] M7 ScriptHost failed: {e}",
                    self.handler.elapsed_ms()
                );
                None
            }
        };

        let theme = Theme::Dark;
        let zen = ZenMode::new();
        let mut toasts = ToastQueue::new();
        toasts.push(Toast::success("egui pivot live"));
        toasts.push(Toast::info("Press 1=Brush, 2=Move, Tab=Zen, M=Theme"));
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(BrushTool::default()));
        tools.register(Box::new(MoveTool::default()));
        let egui = EguiSystem::new(&window, &surface);
        println!(
            "[{:>6}ms] egui: initialized ({}×{})",
            self.handler.elapsed_ms(),
            size.width,
            size.height
        );

        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(AppGfx {
            surface,
            renderer,
            sim,
            present,
            camera,
            asset_db,
            atlas_is_real,
            script,
            theme,
            zen,
            toasts,
            tools,
            egui,
        });
        let _ = atlas_is_real;
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        if let Some(host) = self.host.as_ref() {
            host.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // Forward to egui-winit FIRST so it can claim events we don't
        // need to also see (text input, hover, etc.).
        if let (Some(host), Some(gfx)) = (self.host.as_ref(), self.gfx.as_mut()) {
            let _ = gfx.egui.state.on_window_event(host.window(), &event);
        }

        match event {
            WindowEvent::CloseRequested => match self.handler.on_close_request() {
                CloseAction::Close => {
                    self.handler.on_lifecycle(Lifecycle::WillTerminate);
                    event_loop.exit();
                }
                CloseAction::Cancel => {}
            },

            WindowEvent::Resized(size) => {
                self.pending_resize = Some(WindowSize::new(size.width, size.height));
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(host) = &self.host {
                    host.scale.set(scale_factor as f32);
                    if let Some(gfx) = self.gfx.as_ref() {
                        self.pending_resize = Some(gfx.surface.size());
                    }
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.last_pointer = (position.x as f32, position.y as f32);
            }

            WindowEvent::MouseInput { state, .. } => {
                let kind = match state {
                    ElementState::Pressed => PointerKind::Down,
                    ElementState::Released => PointerKind::Up,
                };
                self.handler.on_pointer(PointerEvent {
                    x: self.last_pointer.0,
                    y: self.last_pointer.1,
                    pressure: 1.0,
                    kind,
                    source: PointerSource::Mouse,
                    timestamp_ns: Self::timestamp_ns(),
                });
            }

            WindowEvent::KeyboardInput {
                event:
                    WinitKeyEvent {
                        physical_key,
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                let keycode = match physical_key {
                    winit::keyboard::PhysicalKey::Code(code) => code as u32,
                    winit::keyboard::PhysicalKey::Unidentified(_) => 0,
                };
                let kind = match (state, repeat) {
                    (ElementState::Pressed, false) => KeyKind::Down,
                    (ElementState::Pressed, true) => KeyKind::Repeat,
                    (ElementState::Released, _) => KeyKind::Up,
                };
                self.handler.on_key(KeyEvent {
                    keycode,
                    modifiers: Self::convert_modifiers(self.modifiers),
                    kind,
                    timestamp_ns: Self::timestamp_ns(),
                });
                if matches!((state, repeat), (ElementState::Pressed, false))
                    && let PhysicalKey::Code(code) = physical_key
                {
                    self.handle_editor_key(code);
                }
            }

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }
}

fn main() {
    install_panic_hook();
    let event_loop = EventLoop::new().expect("create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    println!("PH2D desktop shell starting (egui pivot)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}
