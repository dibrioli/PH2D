//! Subsystem initialization for the desktop shell.
//!
//! PR 9c of `docs/Migracao/2026-05-convention-by-discovery.md`:
//! `resumed()` in `main.rs` used to inline ~260 LOC of boot work
//! (window creation, GPU init, atlas load, sim populate, script host,
//! editor stack, hero screen). That kept `main.rs` growing every time
//! a subsystem joined — anti-pattern §15 + HR-18 cap pressure.
//!
//! This module hosts the boot pipeline as one `pub(crate) fn
//! build_initial_state(...) -> (Arc<Window>, WinitHost, AppGfx)`. The
//! body is the verbatim former content of `resumed()` (no behaviour
//! change — just a code move), so smoke parity is byte-for-byte. PR
//! 9c.next decomposes the body into per-subsystem sub-fns; this commit
//! is the safe atomic move.

use std::collections::BTreeMap;
use std::sync::Arc;

use bumpalo::Bump;
use ph2d_asset::{AssetDb, LogicalTextureMap};
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components, stable_type_id};
use ph2d_ecs::scene::{EditorCommandQueue, HierarchySnapshot, HierarchyWalkState};
use ph2d_ecs::{PresentWorld, SimWorld, TransformPropagationState, WorklistBuf};
use ph2d_editor::{
    HeroScreen, Layout as EditorLayout, NodeId, Toast, ToastQueue, ToolRegistry, ZenMode,
};
use ph2d_gpu::{GpuContext, SurfaceContext};
use ph2d_host::{Lifecycle, PlatformHost};
use ph2d_imageio::{ExporterRegistry, ImporterRegistry};
use ph2d_render::{Camera2d, Compositor, GameRt, SpriteRenderer, TextureAtlas, Tonemap, VelloPass};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use crate::hero_bridge;
use crate::integration;
use crate::theme::parse_theme_env;
use crate::winit_host::{LoggingHandler, WinitHost};
use crate::{AppGfx, HeroLive, SPRITE_COUNT};

/// Build the editor's initial state. Called once from
/// [`ApplicationHandler::resumed`] on the first frame; never re-runs.
///
/// Returns the window handle, the `WinitHost` wrapper, and the
/// fully-populated `AppGfx`. The caller stores these on `App` and
/// invokes lifecycle hooks (`on_lifecycle(Foreground)`, `on_resize`).
pub(crate) fn build_initial_state(
    handler: &LoggingHandler,
    event_loop: &ActiveEventLoop,
) -> (Arc<Window>, WinitHost, AppGfx) {
    let attrs = Window::default_attributes()
        .with_title("PH2D — editor")
        .with_inner_size(LogicalSize::new(1024, 768));
    let window = Arc::new(
        event_loop
            .create_window(attrs)
            .expect("create_window must succeed"),
    );
    // Enable IME so dead-key + composition sequences (PT-BR
    // accents `á`, `ç`, `ñ`, …) reach us via `WindowEvent::Ime`.
    // Without this the macOS text-input service swallows the
    // dead-key keystroke and `KeyEvent::text` arrives empty.
    window.set_ime_allowed(true);
    let host = WinitHost::new(window.clone());
    let size = host.window_size();

    let instance = GpuContext::default_instance();
    let raw_surface = instance
        .create_surface(window.clone())
        .expect("create_surface");
    let gpu = GpuContext::new(instance, Some(&raw_surface)).expect("GpuContext::new");
    // GPU pass profiler (PH2D_FLUID_PROFILE=1): per-pass GPU EXECUTION timings —
    // the `[fluid]`/`[frame]` CPU timers can't see where the GPU itself spends
    // the frame. Inert without the env var / TIMESTAMP_QUERY.
    ph2d_gpu::pass_profiler::init(&gpu.device, &gpu.queue);
    let surface = SurfaceContext::new(gpu, raw_surface, size).expect("SurfaceContext::new");

    // M6: try to compose the atlas from real PNG files on disk.
    // Auto-generates 16 procedural fixtures on first launch so the
    // demo is self-contained (no committed binary fixtures). Any
    // failure logs and falls back to the M5 procedural dummy —
    // the shell must boot regardless of asset-pipeline issues.
    let asset_db = AssetDb::new();
    // KTX2 Fase 2 (W2.T4): logical-texture → per-tier cooked AssetId map.
    // Empty until a cooked texture is loaded (e.g. the PH2D_KTX2_SMOKE
    // harness below, or a future scene/import path).
    let mut logical_texture_map = LogicalTextureMap::new();
    let assets_dir = integration::demo_assets_dir();
    let (atlas, atlas_is_real) =
        match crate::atlas_loader::load_atlas(surface.gpu(), &asset_db, &assets_dir) {
            Ok(atlas) => {
                println!(
                    "[{:>6}ms] M6: real atlas composed from {} ({} assets cached)",
                    handler.elapsed_ms(),
                    assets_dir.display(),
                    asset_db.len_assets()
                );
                (atlas, true)
            }
            Err(e) => {
                eprintln!(
                    "[{:>6}ms] M6 fallback to dummy atlas: {e}",
                    handler.elapsed_ms()
                );
                (TextureAtlas::dummy(surface.gpu()), false)
            }
        };
    // M14.5: sprite pipeline now targets the offscreen HDR game RT
    // (Rgba16Float) instead of the swap chain. The tonemap +
    // compositor passes carry pixels through to the surface.
    let renderer = SpriteRenderer::new(
        surface.gpu().clone(),
        GameRt::FORMAT,
        atlas,
        SPRITE_COUNT.next_power_of_two(),
    );

    // Mode gate inverted 2026-05-14: hero live is the **default**
    // user-facing experience. `PH2D_M5_DEMO=1` opts into the legacy
    // M5 perf-validation demo (1000-sprite Vogel spiral, no editor
    // chrome) — kept reachable for HR-4 frame-budget validation,
    // 100k-sprite stress tests, and future bench work without
    // forcing users through it on first launch.
    let m5_demo_enabled = std::env::var("PH2D_M5_DEMO").as_deref() == Ok("1");
    let hero_live_enabled = !m5_demo_enabled;
    let mut sim = SimWorld::new();
    if hero_live_enabled {
        // M14.4a: spawn a small named set so the hierarchy
        // panel renders readable rows. The 1000-sprite Vogel
        // spiral demo is fixture-only; live mode is for the
        // editor's hierarchy/inspector pipeline.
        crate::sim_populate::populate_sim_live(&mut sim);
        println!(
            "[{:>6}ms] live hero mode (8 named entities; \
             hierarchy panel binds to ECS)",
            handler.elapsed_ms()
        );
    } else {
        crate::sim_populate::populate_sim(&mut sim);
        println!(
            "[{:>6}ms] M5 demo mode (PH2D_M5_DEMO=1; 1000-sprite \
             Vogel spiral, no editor chrome)",
            handler.elapsed_ms()
        );
    }
    // W2.T4 end-to-end smoke (PH2D_KTX2_SMOKE=1): cook an RGBA8 KTX2 in
    // memory, register it, and spawn a `SpriteSource::CookedTexture` sprite so
    // the loader path renders it. No-op unless the env var is set.
    crate::ktx2_smoke::spawn_if_enabled(&mut sim, &asset_db, &mut logical_texture_map);
    let present = PresentWorld::new();
    // ADR-0025 M14.1: build the cached propagation queries AFTER
    // populate_sim so bevy_ecs has already seen the Transform
    // archetype. QueryState::new on `&mut World` is fine here
    // (one-shot at boot); inside the extract phase the queries
    // iterate via `&World` only.
    let prop_state = TransformPropagationState::new(sim.world_mut());
    let worklist = WorklistBuf::new();
    let hero_live = if hero_live_enabled {
        let walk_state = HierarchyWalkState::new(sim.world_mut());
        Some(HeroLive {
            bridge: hero_bridge::EntityNodeMap::new(),
            walk_state,
            walk_scratch: Vec::with_capacity(64),
            snapshot: HierarchySnapshot::new(),
        })
    } else {
        None
    };
    let camera = Camera2d::default();

    // M7: ScriptHost. Failure here is also non-fatal (script is
    // a placeholder; full sim-driving lands in M12+ editor panel).
    let script = match integration::init_script_host() {
        Ok(host) => {
            println!(
                "[{:>6}ms] M7: ScriptHost initialized (placeholder script loaded)",
                handler.elapsed_ms()
            );
            Some(host)
        }
        Err(e) => {
            eprintln!(
                "[{:>6}ms] M7 ScriptHost failed: {e} — continuing without scripting",
                handler.elapsed_ms()
            );
            None
        }
    };

    // M12 + M11: editor data layer + Vello widget paint pass.
    // ZenMode/ToastQueue/ToolRegistry model state, Layout computes
    // the 4 zones, VelloPass renders all widgets onto the surface
    // AFTER the sprite pass.
    let theme = parse_theme_env();
    eprintln!("[ph2d] theme = {}", theme.id());
    let zen = ZenMode::new();
    let mut toasts = ToastQueue::new();
    toasts.push(Toast::success("Editor data layer wired (M12)"));
    toasts.push(Toast::info("Press 1=Brush, 2=Move, 3=Bg Removal, Tab=Zen"));
    // All modal tools are registered by codegen (ADR-0040 T-close):
    // `ph2d-tool-sync` generates `register_all_tools` from the scan of
    // `crates/ph2d-tool-*` (pub fn make). Adding a tool = drop a crate +
    // run the sync — zero edit here. `activate_default` selects the boot
    // tool data-drivenly (`Tool::is_default` = Brush), not registration order.
    let mut tools = ToolRegistry::new();
    ph2d_tool_registry_init::register_all_tools(&mut tools);
    tools.activate_default();
    let layout = EditorLayout::new(size.width as f32, size.height as f32);
    let vello_pass =
        match VelloPass::new(surface.gpu(), surface.format(), (size.width, size.height)) {
            Ok(p) => {
                println!(
                    "[{:>6}ms] M11: VelloPass initialized ({}×{} intermediate)",
                    handler.elapsed_ms(),
                    size.width,
                    size.height
                );
                p
            }
            Err(e) => {
                // Pass init failure is fatal here — the demo's whole
                // point is showing the editor over the canvas.
                panic!("VelloPass::new failed: {e}");
            }
        };

    // M14.5: viewport / RT pipeline construction. game_rt → tonemap
    // → compositor (which also reads vello_pass intermediate). The
    // sample views are extracted here at boot; rebound on resize
    // alongside game_rt/tonemap output recreation.
    let game_rt = GameRt::new(surface.gpu(), (size.width, size.height));
    let tonemap = Tonemap::new(
        surface.gpu(),
        game_rt
            .texture()
            .create_view(&wgpu::TextureViewDescriptor::default()),
        (size.width, size.height),
    );
    let compositor = Compositor::new(
        surface.gpu(),
        surface.format(),
        tonemap
            .output_texture()
            .create_view(&wgpu::TextureViewDescriptor::default()),
        vello_pass
            .intermediate_texture()
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    println!(
        "[{:>6}ms] M14.5: RT pipeline ready (game_rt Rgba16Float HDR + AgX tonemap + compositor)",
        handler.elapsed_ms()
    );
    let vector_scene = VectorScene::new();
    let text_system = TextSystem::new();

    // Hero screen (TopBar / LeftRail / Hierarchy / Inspector /
    // BottomHUD) is always-on in the default mode and disabled
    // in the M5 demo path. The legacy `PH2D_HERO_SCREEN=1` env
    // var is kept as a no-op alias — anyone with it in their
    // shell rc still gets the editor instead of an error.
    let hero_screen_enabled = hero_live_enabled;
    // Wave 8 Phase 1 — install the panel registry BEFORE the first
    // `HeroScreen::new` call. `register_all_panels` honors the
    // `panel-*` cargo features on `ph2d-panel-registry-init`, so
    // `--no-default-features --features panel-inspector` (etc.)
    // produces a binary with exactly the selected panels at runtime.
    // Idempotent — re-entry is a no-op.
    if hero_screen_enabled {
        let _ = ph2d_panel_registry_init::register_all_panels();
    }
    let hero_screen = if hero_screen_enabled {
        let mut hero = HeroScreen::new(NodeId(1)).theme(theme);
        // Cross-session palettes: restore the named-palette set saved last run (the picker was just
        // seeded with the default by `pre_populate`; replace it when a save exists).
        let saved = crate::palette_persist::load();
        if !saved.is_empty() {
            let palettes = saved
                .into_iter()
                .map(|(name, colors)| ph2d_editor::interaction::NamedPalette {
                    name,
                    swatches: colors
                        .iter()
                        .map(|c| ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]))
                        .collect(),
                })
                .collect();
            hero.store
                .blender_set_palettes(ph2d_editor::ids::INSP_BLENDER_PICKER, palettes);
            hero.store
                .sync_blender_palette_name_buffer(ph2d_editor::ids::INSP_BLENDER_PICKER);
        }
        Some(hero)
    } else {
        None
    };

    let _ = hero_screen_enabled; // explicitly mark consumed
    let _ = Lifecycle::Foreground; // exercise import; lifecycle hook fires from caller.

    // PR 8 of the convention-by-discovery migration: build the tool
    // registry at boot. `register_all` adds every manifest declared
    // in `ph2d-tool-registry-init`'s append-only list; `build()`
    // detects id duplicates + NodeId hash collisions + sorts
    // deterministically per HR-5. Held on `AppGfx` for PR 9 (generic
    // dispatcher) and chrome derivation follow-ups.
    let mut registry = ph2d_tool_registry::Registry::default();
    ph2d_tool_registry_init::register_all(&mut registry);
    registry
        .build()
        .expect("registry build must succeed at boot");
    let manifest_count = registry.manifests().len();
    // Wave 2 PR 11.4: hand the built registry to `ph2d-editor` so the
    // hero painters can derive chrome (Image Tools action row, future
    // TopBar clusters) from manifests instead of hardcoded lists.
    // `install_registry` returns true on first install; subsequent
    // calls from re-init paths in tests get false and silently drop
    // the second registry (safe — the manifests are identical).
    ph2d_editor::install_registry(registry);
    println!(
        "[{:>6}ms] PR 8: tool registry built ({} manifests, installed in editor)",
        handler.elapsed_ms(),
        manifest_count,
    );

    // ADR-0054 W0.T6: image I/O registries populated at boot. Same
    // "drop a crate, zero central edit" mechanism as the tool registry
    // above — `ph2d-imageio-sync` regenerates the bodies from a scan of
    // `crates/ph2d-imageio-*`. W0.T5 wires PNG only; W1+ adds JPEG /
    // WebP / GIF / .ph2d-native; W2+ adds PSD/ORA/TIFF/APNG; W3+ adds
    // EXR/AVIF/JXL/HDR/SVG.
    let mut imageio_importers = ImporterRegistry::new();
    let mut imageio_exporters = ExporterRegistry::new();
    ph2d_imageio_registry_init::register_all_importers(&mut imageio_importers);
    ph2d_imageio_registry_init::register_all_exporters(&mut imageio_exporters);
    println!(
        "[{:>6}ms] ADR-0054 W0.T6: imageio registries built ({} importer(s), {} exporter(s))",
        handler.elapsed_ms(),
        imageio_importers.len(),
        imageio_exporters.len(),
    );

    // ADR-0108 Fase 0: cena-demo prova o seam; `PH2D_VEC_DEMO_N=<n>` troca para a
    // grade de N blobs do spike de escala (kill-criterion §5). Loga a escolha no
    // terminal — diagnóstico infalível de qual caminho rodou.
    let pen_on =
        std::env::var("PH2D_VEC_PEN").is_ok_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
    let vec_scene = match std::env::var("PH2D_VEC_DEMO_N")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
    {
        Some(n) if n > 0 => {
            eprintln!("[ph2d-vec] Fase 0 spike: demo_grid N={n}");
            ph2d_vec_scene::VecScene::demo_grid(n)
        }
        // Pen ligado → canvas vazio, pra o desenho aparecer sem o smiley junto.
        _ if pen_on => {
            eprintln!(
                "[ph2d-vec] Fase 1.1: Pen ATIVO — canvas vazio; clique desenha, \
                 botão direito finaliza o traço"
            );
            ph2d_vec_scene::VecScene::new()
        }
        // Default (sem flag): cena VAZIA — a feature é 100% flag-gated, o app
        // normal não mostra nada da pipeline vetorial nova.
        _ => ph2d_vec_scene::VecScene::new(),
    };

    let gfx = AppGfx {
        surface,
        renderer,
        sim,
        present,
        camera,
        asset_db,
        logical_texture_map,
        atlas_is_real,
        script,
        theme,
        zen,
        toasts,
        tools,
        layout,
        game_rt,
        tonemap,
        compositor,
        vello_pass,
        vector_scene,
        // ADR-0108 Fase 0: `vec_scene` escolhido logo acima (smiley ou grade N).
        vec_scene,
        // Motion Nodes M0.T8: boot state = default grid→transform→clone vertical
        // + full node registry + paused transport (cooked per frame by the bridge).
        motion: crate::motion_state::MotionState::new(),
        text_system,
        hero_screen,
        hero_arena: Bump::with_capacity(4096),
        clipboard: arboard::Clipboard::new()
            .map_err(|e| eprintln!("[ph2d] clipboard init failed: {e}"))
            .ok(),
        prop_state,
        worklist,
        sort_scratch: ph2d_ecs::sort_key::SortScratch::new(),
        sort_inputs: Vec::new(),
        hero_live,
        next_import_cell: ph2d_render::FIRST_IMPORT_KEY,
        atlas_asset_map: BTreeMap::new(),
        component_registry: {
            let mut reg = ComponentRegistry::new();
            register_ecs_components(&mut reg);
            // M14.C audit fix #8: register Sprite alongside the
            // ecs components so the Strategy switch can flow
            // through `EditorCommand::SetComponent` instead of
            // direct world mutation.
            ph2d_render::register_render_components(&mut reg);
            reg
        },
        editor_queue: EditorCommandQueue::new(),
        transform_type_id: stable_type_id("ph2d::ecs::Transform"),
        visibility_type_id: stable_type_id("ph2d::ecs::Visibility"),
        name_type_id: stable_type_id("ph2d::ecs::Name"),
        sprite_type_id: stable_type_id("ph2d::render::Sprite"),
        image_edit_undo: None,
        imageio_importers,
        imageio_exporters,
    };

    (window, host, gfx)
}
