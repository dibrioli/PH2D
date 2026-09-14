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

use ph2d_i18n::tr;
use std::collections::BTreeMap;
use std::sync::Arc;

use bumpalo::Bump;
use ph2d_asset::{AssetDb, LogicalTextureMap};
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components, stable_type_id};
use ph2d_ecs::scene::{EditorCommandQueue, HierarchySnapshot, HierarchyWalkState};
use ph2d_ecs::{PresentWorld, SimWorld, TransformPropagationState, WorklistBuf};
use ph2d_editor_core::{
    HeroScreen, JobQueue, Layout as EditorLayout, NodeId, Toast, ToastQueue, ToolRegistry, ZenMode,
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

#[path = "init_subsystems.rs"]
mod subsystems; // os subsistemas do arranque, por ordem (LOC cap: sibling module)

/// ⭐ **O registo de componentes do produto** — as quatro famílias que o app conhece.
///
/// ⚠️ **É função com nome porque o censo da F3 tem de perguntar ao MESMO registo que o app usa**
/// (ADR-0166): *"todo componente que a paleta oferece, o registo sabe construir"* é uma afirmação
/// sobre este objeto, e um segundo `ComponentRegistry::new()` montado à mão dentro do teste seria
/// uma segunda lista a envelhecer — o gate ficaria verde sobre um registo que ninguém executa.
///
/// ⚠️ Um componente que falte aqui é **descartado em silêncio** pelo `WorldSnapshot` (undo + save),
/// e o sintoma é o objeto perder a feature ao desfazer.
pub(crate) fn build_component_registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    // M14.C audit fix #8: register Sprite alongside the ecs components so the Strategy switch can
    // flow through `EditorCommand::SetComponent` instead of direct world mutation.
    ph2d_render::register_render_components(&mut reg);
    // ADR-0131 W1: RigidBody/Collider — without this the WorldSnapshot (undo + save) silently
    // drops them.
    ph2d_physics_ecs::register_physics_components(&mut reg);
    // ADR-0161 — o objeto de modelagem 3D. Sem esta linha o WorldSnapshot descarta o componente EM
    // SILENCIO, e o sintoma é o objeto sumir ao desfazer. `field3d_snapshot_tests` prova os dois
    // lados disso.
    ph2d_field_ecs::register_field_components(&mut reg);
    // ⭐⭐⭐ O ESQUELETO (2026-09-06) — o osso e a pele. Eles viveram dentro do `ph2d-ecs` até virarem
    // MÓDULO: ele serve vector, raster, 3D e Flip, e um componente por mídia dentro da fundação a
    // faria crescer uma vez por cliente. Sem esta linha o WorldSnapshot descarta-os EM SILÊNCIO, e
    // o sintoma é o personagem perder o esqueleto ao desfazer.
    ph2d_skeleton_ecs::register_skeleton_components(&mut reg);
    reg
}

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
        .with_title(tr("shell.init.ph2d_editor"))
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

    let (asset_db, mut logical_texture_map, atlas_is_real, motion_default_uv, renderer) =
        subsystems::boot_assets_and_renderer(handler, &surface);

    let (hero_live_enabled, sim, present, prop_state, worklist, hero_live) =
        subsystems::boot_sim_world(handler, &asset_db, &mut logical_texture_map);
    let camera = Camera2d::default();

    let script = subsystems::boot_script_host(handler);

    let (theme, zen, jobs, toasts, tools, layout, vello_pass) =
        subsystems::boot_editor_layer(handler, &surface, size);

    let (game_rt, world_rt, band_blit, frost, motion_fx, tonemap, compositor) =
        subsystems::boot_rt_pipeline(handler, &surface, size, &vello_pass);
    let vector_scene = VectorScene::new();
    let text_system = TextSystem::new();

    let hero_screen = boot_hero_screen(handler, hero_live_enabled, theme);

    let (imageio_importers, imageio_exporters) = subsystems::boot_imageio_registries(handler);

    let vec_scene = subsystems::boot_vec_scene();

    // ADR-0114 W1: rasterizador do traço do Flip, no formato HDR do game_rt.
    // Criado ANTES do literal (o `surface` é movido pra dentro dele).
    let flip_render =
        ph2d_flip_render::FlipRenderer::new(&surface.gpu().device, ph2d_render::GameRt::FORMAT);
    // W1 T1.7: as passagens resolve/blit da composição por-camada (mesmo formato).
    let flip_compose =
        ph2d_flip_render::FlipCompose::new(&surface.gpu().device, ph2d_render::GameRt::FORMAT);

    let gfx = AppGfx {
        // ADR-0150 W1/M2: cena 3D nasce VAZIA — o smoke a arma no 1o frame.
        #[cfg(feature = "sculpt3d")]
        sculpt3d: None,
        // Os objetos que uma forma acende (`docs/3D/02.2`). Nascem vazios e NÃO são `cfg`-gated: um
        // projeto salvo pode trazer objetos assados para um binário sem o módulo 3D.
        baked_forms: std::collections::BTreeMap::new(),
        baked_light: None,
        surface,
        renderer,
        sim,
        present,
        camera,
        canvas_zoom: crate::canvas_zoom::CanvasZoom::default(),
        asset_db,
        logical_texture_map,
        atlas_is_real,
        script,
        theme,
        zen,
        toasts,
        jobs,
        tools,
        layout,
        game_rt,
        world_rt,
        band_blit,
        frost,
        frost_doc_scene: ph2d_vector::VectorScene::new(),
        frost_front_scene: ph2d_vector::VectorScene::new(),
        frosting: false,
        band_doc_scenes: Vec::new(),
        compositor_reads_world: false,
        motion_fx,
        tonemap,
        compositor,
        vello_pass,
        vector_scene,
        // ADR-0108 Fase 0: `vec_scene` escolhido logo acima (smiley ou grade N).
        vec_scene,
        guides: ph2d_guides::GuideSet::default(),
        ui_states: ph2d_ui_state::StateSets::default(),
        ui_machines: Default::default(),
        // ADR-0114: cena Flip + demo ready-to-smoke (a tool do W2 cria objetos
        // interativamente; aqui um objeto animado pra abrir e ver na hora).
        flip: ph2d_app_flip::demo::demo_scene(),
        // ADR-0114 W1: rasterizador do traço, no formato HDR do game_rt (criado
        // logo acima, antes do literal — `surface` já foi movido aqui).
        flip_render,
        // W1 T1.7: passagens de espaço-de-cor da composição; o compositor é lazy.
        flip_compose,
        flip_composite: None,
        // Motion Nodes M0.T8: boot state = default grid→transform→clone vertical
        // + full node registry + paused transport (cooked per frame by the bridge).
        // Its instances sample one opaque atlas tile (computed above) so the raw
        // M0 output renders as clean solid quads.
        motion: {
            let mut m = ph2d_app_motion::motion_state::MotionState::new();
            m.default_uv_rect = motion_default_uv;
            m
        },
        physics: ph2d_physics_ecs::PhysicsBridge::new(),
        text_system,
        hero_screen,
        hero_arena: Bump::with_capacity(4096),
        clipboard: arboard::Clipboard::new()
            .map_err(|e| eprintln!("[ph2d] clipboard init failed: {e}"))
            .ok(),
        prop_state,
        undo_capture_cache: ph2d_ecs::scene::incremental::CaptureCache::new(),
        component_palette_target: None,
        worklist,
        sort_scratch: ph2d_ecs::sort_key::SortScratch::new(),
        sort_inputs: Vec::new(),
        frame_order: crate::draw_bands::FrameOrder::default(),
        hero_live,
        next_import_cell: ph2d_render::FIRST_IMPORT_KEY,
        sheets: std::collections::BTreeMap::new(),
        sheet_textures: std::collections::BTreeMap::new(),
        next_sheet_id: 0,
        next_painted_doc: 1, // 0 fica livre como "nenhum"
        next_baked_form: 1,  // idem: 0 fica livre como "nenhum"
        atlas_asset_map: BTreeMap::new(),
        catalogs: ph2d_asset_index::CatalogTree::new(),
        library_cache: crate::project_library::LibraryCache::default(),
        tags: ph2d_tags::TagTree::new(),
        tags_cache: ph2d_app_components::tags_doc::TagsCache::default(),
        tags_problem: None,
        component_registry: build_component_registry(),
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

/// **Os registos de painéis e de ferramentas, e o HeroScreen** — com as preferências, as paletas e
/// a arrumação do artista instaladas antes do primeiro quadro.
///
/// ⚠️ Saiu do [`build_initial_state`] pelo tecto de 200 LOC por função, verbatim, e FICA neste
/// ficheiro de propósito: o gate `the_registry_is_installed_before_the_hero` lê aqui a ordem
/// `install_registry(` → `HeroScreen::new(`.
fn boot_hero_screen(
    handler: &LoggingHandler,
    hero_live_enabled: bool,
    theme: ph2d_tokens::Theme,
) -> Option<HeroScreen> {
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
    // ⚠️ **O registry de TOOLS tem de estar instalado ANTES do primeiro `HeroScreen::new`** —
    // pela MESMA razão que o de painéis, logo acima, e este bloco vivia 50 linhas abaixo do hero.
    //
    // O `topbar::populate()` (que corre dentro do `HeroScreen::new`) dá `InteractiveState` a cada
    // pill da fila de Image Tools **percorrendo a fila derivada do registry**. Sem registry
    // instalado ela cai no fallback de três (trim · make_square · bgremoval) e **os outros oito
    // pills nascem sem estado: pintados, hit-registered e MORTOS sob o rato** (Enio, 2026-08-19:
    // *"padding, Color equalization, Rasterize, Upscale e Painter não estão funcionando"*).
    //
    // ⚠️ Foi uma regressão que eu introduzi ao curar exatamente este defeito para UM tool: troquei
    // a lista escrita à mão por uma derivada, sem reparar que a derivação corria antes da fonte
    // existir. *Uma lista derivada de algo que ainda não existe é uma lista vazia com cara de
    // correta.* O gate `the_tool_registry_is_installed_before_the_hero` fixa esta ordem.
    //
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
    ph2d_editor_core::install_registry(registry);
    println!(
        "[{:>6}ms] PR 8: tool registry built ({} manifests, installed in editor)",
        handler.elapsed_ms(),
        manifest_count,
    );

    let hero_screen = if hero_screen_enabled {
        let mut hero = HeroScreen::new(NodeId(1)).theme(theme);
        // Cross-session palettes: restore the named-palette set saved last run (the picker was just
        // seeded with the default by `pre_populate`; replace it when a save exists).
        let saved = crate::palette_persist::load();
        if !saved.is_empty() {
            let palettes = saved
                .into_iter()
                .map(
                    |(name, colors)| ph2d_editor_core::interaction::NamedPalette {
                        name,
                        swatches: colors
                            .iter()
                            .map(|c| ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]))
                            .collect(),
                    },
                )
                .collect();
            hero.store
                .blender_set_palettes(ph2d_editor_core::ids::INSP_BLENDER_PICKER, palettes);
            hero.store
                .sync_blender_palette_name_buffer(ph2d_editor_core::ids::INSP_BLENDER_PICKER);
        }
        // Preferências de utilizador (`~/.ph2d/prefs.txt`): o carácter da UI viva + o reduced
        // motion, escolhidos no pill Settings → Motion. ⚠️ Instaladas ANTES do primeiro quadro —
        // instalar depois deixaria a primeira animação correr no carácter errado, e é justamente o
        // primeiro quadro que o artista vê. Ficheiro ausente ⇒ os defaults, que são os de hoje.
        let prefs = crate::prefs::load();
        hero.motion.set_character(prefs.character);
        hero.motion.set_reduced_motion(prefs.reduced_motion);
        hero.ui_sound = prefs.ui_sound;
        // ⭐⭐ **A ARRUMAÇÃO do artista** (`~/.ph2d/layout.txt`, decisão D4): que painel está em
        // que encaixe e a largura das colunas. ⚠️ Antes do primeiro quadro, pela mesma razão das
        // preferências — instalar depois faria o primeiro quadro desenhar a arrumação de omissão e
        // saltar para a do artista no seguinte.
        crate::layout_persist::install_saved(&mut hero, &crate::layout_persist::load());
        Some(hero)
    } else {
        None
    };

    let _ = hero_screen_enabled; // explicitly mark consumed
    let _ = Lifecycle::Foreground; // exercise import; lifecycle hook fires from caller.
    hero_screen
}
