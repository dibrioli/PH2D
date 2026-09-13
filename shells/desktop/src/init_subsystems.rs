//! **Os SUBSISTEMAS do arranque** — irmão de `init.rs` pelos dois tectos da shell (HR-18: 600 LOC
//! por ficheiro, 200 por função).
//!
//! O `build_initial_state` nasceu como o corpo verbatim do antigo `resumed()`, e o cabeçalho do
//! `init.rs` já prometia partir esse corpo em sub-funções por subsistema: é este corte, feito
//! mecanicamente e pela MESMA ordem do arranque. Cada bloco saiu inteiro para uma função que
//! devolve o que o arranque lê dele. Mudaram duas linhas, e por força do corte: o `mut` de um mapa
//! que o bloco de assets deixou de mutar, e os dois empréstimos que o smoke do KTX2 recebe e que
//! passaram a chegar já emprestados.
//!
//! ⚠️ **O bloco dos REGISTOS e do HERO ficou no `init.rs`** (`boot_hero_screen`): o gate
//! `the_registry_is_installed_before_the_hero` lê a ORDEM das duas chamadas naquele ficheiro.

use super::*;

/// **O banco de assets, o atlas e o renderer de sprites** — o atlas compõe-se dos PNG do disco (ou
/// do procedural de recurso), reserva o ladrilho branco do Motion e entra no renderer.
pub(super) fn boot_assets_and_renderer(
    handler: &LoggingHandler,
    surface: &SurfaceContext,
) -> (AssetDb, LogicalTextureMap, bool, [f32; 4], SpriteRenderer) {
    // M6: try to compose the atlas from real PNG files on disk.
    // Auto-generates 16 procedural fixtures on first launch so the
    // demo is self-contained (no committed binary fixtures). Any
    // failure logs and falls back to the M5 procedural dummy —
    // the shell must boot regardless of asset-pipeline issues.
    let asset_db = AssetDb::new();
    // KTX2 Fase 2 (W2.T4): logical-texture → per-tier cooked AssetId map.
    // Empty until a cooked texture is loaded (e.g. the PH2D_KTX2_SMOKE
    // harness below, or a future scene/import path).
    let logical_texture_map = LogicalTextureMap::new();
    let assets_dir = integration::demo_assets_dir();
    let (mut atlas, atlas_is_real) =
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
    // Motion Nodes M0: the raw default document has no framing node yet, so its
    // instances carry no `uv_rect` column and fall back to this rect. It must be
    // the reserved opaque WHITE tile: the shader multiplies `tint` by the texel,
    // so any other tile silently stains the authored colour (the demo's tile 0 is
    // saturated red -- a red->blue gradient came out red->maroon). Whole-atlas
    // fallback only if the insert fails. Read before the atlas moves into the
    // renderer below.
    let motion_default_uv = atlas
        .insert_white_tile(surface.gpu())
        .unwrap_or([0.0, 0.0, 1.0, 1.0]);
    // M14.5: sprite pipeline now targets the offscreen HDR game RT
    // (Rgba16Float) instead of the swap chain. The tonemap +
    // compositor passes carry pixels through to the surface.
    let renderer = SpriteRenderer::new(
        surface.gpu().clone(),
        GameRt::FORMAT,
        atlas,
        SPRITE_COUNT.next_power_of_two(),
    );
    (
        asset_db,
        logical_texture_map,
        atlas_is_real,
        motion_default_uv,
        renderer,
    )
}

/// **O mundo da simulação** — o modo (hero vivo, smoke de física ou demo M5), o smoke do KTX2, e
/// o que a propagação e a Hierarquia viva precisam desde o primeiro quadro.
pub(super) fn boot_sim_world(
    handler: &LoggingHandler,
    asset_db: &AssetDb,
    logical_texture_map: &mut LogicalTextureMap,
) -> (
    bool,
    SimWorld,
    PresentWorld,
    TransformPropagationState,
    WorklistBuf,
    Option<HeroLive>,
) {
    // Mode gate inverted 2026-05-14: hero live is the **default**
    // user-facing experience. `PH2D_M5_DEMO=1` opts into the legacy
    // M5 perf-validation demo (1000-sprite Vogel spiral, no editor
    // chrome) — kept reachable for HR-4 frame-budget validation,
    // 100k-sprite stress tests, and future bench work without
    // forcing users through it on first launch.
    let m5_demo_enabled = std::env::var("PH2D_M5_DEMO").as_deref() == Ok("1");
    let hero_live_enabled = !m5_demo_enabled;
    let mut sim = SimWorld::new();
    // The physics smoke OWNS the scene: no demo sprites, so the hierarchy
    // shows only the simulation (Enio 2026-07-18). Spawning them here just to
    // despawn them later would leave a frame of debris in the hierarchy.
    // ⚠️ O smoke de instância entra na MESMA condição, e não numa segunda: ele também é uma
    // cena de física, e a razão é a mesma — sprites de demonstração na Hierarquia ao lado de
    // três instâncias é exatamente o ruído que torna o smoke ilegível.
    // ⚠️ **Pergunta-se à CRATE, não à variável.** Desde a Fase B o dono do `PH2D_PHYSICS_SMOKE` é
    // a `ph2d-app-physics` (é ela que o declara no `FAMILY` e que responde por ele). Reler a env
    // aqui punha DUAS respostas à mesma pergunta, que é a forma que este repo já pagou várias
    // vezes: elas divergem no dia em que só uma for afinada. O `PH2D_INSTANCE_SMOKE` fica cru
    // porque não tem família — ninguém o declarou ainda.
    let physics_smoke = ph2d_app_physics::smoke::armed_scene().is_some()
        || std::env::var_os("PH2D_INSTANCE_SMOKE").is_some();
    if physics_smoke {
        println!(
            "[{:>6}ms] physics smoke: empty scene (only the physics bodies)",
            handler.elapsed_ms()
        );
    } else if hero_live_enabled {
        // **A cena nasce VAZIA** (Enio 2026-07-17: *"vamos retirar os grupos de sprites de
        // teste do hierarchy para ficarmos apenas com nossos objetos flip"*).
        //
        // O `populate_sim_live` da M14.4a semeava 8 entidades falsas (`group_01/02` +
        // `sprite_001..008`) só para a Hierarquia ter linhas legíveis quando ela ainda não
        // tinha conteúdo real. Hoje tem — objetos Flip, formas vetoriais, sprites
        // importados, camadas do Painter —, então o andaime virou RUÍDO: o artista abre o
        // app e a árvore já vem suja de coisa que ele não criou. Mesmo destino dos
        // scaffolds de debug da timeline, aposentados quando a autoria real os tornou
        // obsoletos.
        println!(
            "[{:>6}ms] live hero mode (cena vazia; a Hierarquia mostra o que VOCE criar)",
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
    crate::ktx2_smoke::spawn_if_enabled(&mut sim, asset_db, logical_texture_map);
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
        let z_walk_state = HierarchyWalkState::new(sim.world_mut());
        Some(HeroLive {
            bridge: hero_bridge::EntityNodeMap::new(),
            walk_state,
            walk_scratch: Vec::with_capacity(64),
            snapshot: HierarchySnapshot::new(),
            z_walk_state,
            z_walk_scratch: Vec::with_capacity(64),
            z_snapshot: HierarchySnapshot::new(),
        })
    } else {
        None
    };
    (
        hero_live_enabled,
        sim,
        present,
        prop_state,
        worklist,
        hero_live,
    )
}

/// **O ScriptHost** (M7) — a falha não é fatal: o arranque segue sem scripting.
pub(super) fn boot_script_host(handler: &LoggingHandler) -> Option<ph2d_script::ScriptHost> {
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
    script
}

/// **A camada de dados do editor e o passe Vello** (M12 + M11) — tema, zen, fila de jobs, toasts,
/// ferramentas, layout das zonas e o passe que pinta os widgets.
pub(super) fn boot_editor_layer(
    handler: &LoggingHandler,
    surface: &SurfaceContext,
    size: ph2d_host::WindowSize,
) -> (
    ph2d_tokens::Theme,
    ZenMode,
    JobQueue,
    ToastQueue,
    ToolRegistry,
    EditorLayout,
    VelloPass,
) {
    // M12 + M11: editor data layer + Vello widget paint pass.
    // ZenMode/ToastQueue/ToolRegistry model state, Layout computes
    // the 4 zones, VelloPass renders all widgets onto the surface
    // AFTER the sprite pass.
    let theme = parse_theme_env();
    eprintln!("[ph2d] theme = {}", theme.id());
    let zen = ZenMode::new();
    let jobs = JobQueue::new();
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
    (theme, zen, jobs, toasts, tools, layout, vello_pass)
}

/// **A cadeia de render targets** (M14.5) — o game RT, o acumulador do mundo e a colagem de faixa,
/// o vidro jateado, o brilho do Motion, o tonemap e o compositor.
pub(super) fn boot_rt_pipeline(
    handler: &LoggingHandler,
    surface: &SurfaceContext,
    size: ph2d_host::WindowSize,
    vello_pass: &VelloPass,
) -> (
    GameRt,
    ph2d_render::WorldRt,
    ph2d_render::BandBlit,
    ph2d_render::FrostPass,
    ph2d_render::MotionFx,
    Tonemap,
    Compositor,
) {
    // M14.5: viewport / RT pipeline construction. game_rt → tonemap
    // → compositor (which also reads vello_pass intermediate). The
    // sample views are extracted here at boot; rebound on resize
    // alongside game_rt/tonemap output recreation.
    let game_rt = GameRt::new(surface.gpu(), (size.width, size.height));
    // ADR-0154 Fase 2: o acumulador do mundo + a colagem de faixa. Inertes enquanto a cena não
    // intercalar vetor e sprite.
    let world_rt = ph2d_render::WorldRt::new(surface.gpu(), (size.width, size.height));
    let band_blit = ph2d_render::BandBlit::new(surface.gpu(), ph2d_render::WorldRt::FORMAT);
    // ⭐⭐⭐ O vidro jateado do *Edit Prefab*: ele borra o acumulador, então nasce no formato DELE.
    let frost = ph2d_render::FrostPass::new(surface.gpu(), ph2d_render::WorldRt::FORMAT);
    // doc 67: the Motion module's own HDR glow pass, sized to the surface like
    // game_rt. Inert until the artist authors bloom on the active Motion doc.
    let motion_fx = ph2d_render::MotionFx::new(surface.gpu(), (size.width, size.height));
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
    (
        game_rt, world_rt, band_blit, frost, motion_fx, tonemap, compositor,
    )
}

/// **Os registos de importação e exportação de imagem** (ADR-0054 W0.T6).
pub(super) fn boot_imageio_registries(
    handler: &LoggingHandler,
) -> (ImporterRegistry, ExporterRegistry) {
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
    (imageio_importers, imageio_exporters)
}

/// **A cena vetorial de arranque** (ADR-0108 Fase 0) — vazia por omissão; a grade do spike de
/// escala ou o canvas limpo do Pen só com as variáveis de ambiente deles.
pub(super) fn boot_vec_scene() -> ph2d_vec_scene::VecScene {
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
    vec_scene
}
