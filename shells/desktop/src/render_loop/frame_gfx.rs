//! **O EMPRÉSTIMO DO `gfx` DO QUADRO, num sítio só** (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! O `run_render_frame` desestruturava o `AppGfx` inteiro uma vez, e o corpo inteiro do quadro
//! corria sobre os campos. Partido em FASES, cada fase re-deriva os campos de que precisa com
//! `let FrameGfx { .., .. } = FrameGfx::of(gfx);` — e este tipo é quem guarda o padrão EXAUSTIVO:
//! um campo novo do `AppGfx` é erro de compilação no [`FrameGfx::of`] até alguém dizer se o quadro
//! o lê (um nome) ou não (`_`, com o porquê), e nenhuma fase o esquece atrás de um `..`.
//!
//! ⚠️ Zero custo por quadro: são referências para os campos, sem alocação nem cópia.

use std::collections::BTreeMap;

use bumpalo::Bump;
use ph2d_asset::{AssetDb, AssetId, LogicalTextureMap};
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{PresentWorld, TransformPropagationState, WorklistBuf};
use ph2d_editor_core::{HeroScreen, JobQueue, ToastQueue, ToolRegistry, ZenMode};
use ph2d_gpu::SurfaceContext;
use ph2d_imageio::ExporterRegistry;
use ph2d_render::{Camera2d, Compositor, GameRt, Tonemap, VelloPass};
use ph2d_script::ScriptHost;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

// Os tipos que o `mod.rs` já importa (o `AppGfx`, o `SimWorld`, o `SpriteRenderer`, o `HeroLive`…).
use super::*;

/// Os campos do `AppGfx` que o quadro lê, emprestados para uma fase (ver o cabeçalho do módulo).
pub(super) struct FrameGfx<'a> {
    pub(super) doc_guides: &'a mut ph2d_guides::GuideSet,
    pub(super) ui_states: &'a mut ph2d_ui_state::StateSets,
    pub(super) ui_machines: &'a mut crate::render_loop::ui_state_bridge::UiMachines,
    #[cfg(feature = "sculpt3d")]
    pub(super) sculpt3d: &'a mut Option<ph2d_app_sculpt3d::Sculpt3dScene>,
    pub(super) baked_forms:
        &'a mut std::collections::BTreeMap<u64, ph2d_form_donation::baked_form::BakedForm>,
    pub(super) baked_light: &'a mut Option<ph2d_render::ImpastoLightPass>,
    pub(super) next_baked_form: &'a mut u32,
    pub(super) surface: &'a mut SurfaceContext,
    pub(super) renderer: &'a mut SpriteRenderer,
    pub(super) sim: &'a mut SimWorld,
    pub(super) present: &'a mut PresentWorld,
    pub(super) camera: &'a mut Camera2d,
    pub(super) canvas_zoom: &'a mut crate::canvas_zoom::CanvasZoom,
    pub(super) asset_db: &'a mut AssetDb,
    pub(super) script: &'a mut Option<ScriptHost>,
    /// ⭐ Os emissores de partículas a correr (TOP-20 #18).
    pub(super) particles: &'a mut ph2d_app_components::particles_bridge::ParticlesState,
    pub(super) theme: &'a mut Theme,
    pub(super) zen: &'a mut ZenMode,
    pub(super) toasts: &'a mut ToastQueue,
    pub(super) jobs: &'a mut JobQueue,
    pub(super) tools: &'a mut ToolRegistry,
    pub(super) layout: &'a mut EditorLayout,
    pub(super) game_rt: &'a mut GameRt,
    pub(super) motion_fx: &'a mut ph2d_render::MotionFx,
    pub(super) tonemap: &'a mut Tonemap,
    pub(super) compositor: &'a mut Compositor,
    pub(super) vello_pass: &'a mut VelloPass,
    pub(super) vector_scene: &'a mut VectorScene,
    pub(super) vec_scene: &'a mut ph2d_vec_scene::VecScene,
    pub(super) flip: &'a mut ph2d_flip::FlipDoc,
    pub(super) text_system: &'a mut TextSystem,
    pub(super) hero_screen: &'a mut Option<HeroScreen>,
    pub(super) hero_arena: &'a mut Bump,
    pub(super) prop_state: &'a mut TransformPropagationState,
    pub(super) worklist: &'a mut WorklistBuf,
    pub(super) sort_scratch: &'a mut ph2d_ecs::sort_key::SortScratch,
    pub(super) sort_inputs: &'a mut Vec<ph2d_ecs::sort_key::SortInput>,
    pub(super) frame_order: &'a mut crate::draw_bands::FrameOrder,
    pub(super) world_rt: &'a mut ph2d_render::WorldRt,
    pub(super) band_doc_scenes: &'a mut Vec<ph2d_vector::VectorScene>,
    pub(super) compositor_reads_world: &'a mut bool,
    pub(super) hero_live: &'a mut Option<HeroLive>,
    pub(super) next_import_cell: &'a mut u32,
    pub(super) sheets: &'a mut std::collections::BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
    pub(super) sheet_textures: &'a mut std::collections::BTreeMap<u32, u32>,
    pub(super) next_sheet_id: &'a mut u32,
    pub(super) atlas_asset_map: &'a mut BTreeMap<u32, AssetId>,
    pub(super) asset_catalogs: &'a mut ph2d_asset_index::CatalogTree,
    /// ⭐⭐ **A ÁRVORE DE TAGS do projecto** (TOP-20 #9) — lida pela resolução do `SignalActions`
    /// (um alvo por tag pergunta quem pertence à subárvore) e, a partir da W3, pelo painel.
    pub(super) tags: &'a mut ph2d_tags::TagTree,
    /// ⭐⭐ **A RECUSA do último gesto do painel *Tags*** — ver o campo homónimo do `AppGfx`. Ela
    /// vem junto com a árvore porque quem a escreve é quem a muta, e quem a lê é quem a publica.
    pub(super) tags_problem: &'a mut Option<(u64, String)>,
    pub(super) logical_texture_map: &'a mut LogicalTextureMap,
    pub(super) component_registry: &'a mut ComponentRegistry,
    pub(super) editor_queue: &'a mut EditorCommandQueue,
    pub(super) transform_type_id: &'a mut u64,
    pub(super) visibility_type_id: &'a mut u64,
    pub(super) name_type_id: &'a mut u64,
    pub(super) sprite_type_id: &'a mut u64,
    pub(super) image_edit_undo: &'a mut Option<ImageEditTransaction>,
    pub(super) imageio_exporters: &'a mut ExporterRegistry,
    pub(super) motion: &'a mut ph2d_app_motion::motion_state::MotionState,
    pub(super) physics: &'a mut ph2d_physics_ecs::PhysicsBridge,
    pub(super) component_palette_target: &'a mut Option<u64>,
    pub(super) frost_doc_scene: &'a mut ph2d_vector::VectorScene,
    pub(super) frost_front_scene: &'a mut ph2d_vector::VectorScene,
    pub(super) frosting: &'a mut bool,
}

impl<'a> FrameGfx<'a> {
    /// O destructure exaustivo do `AppGfx` — o único do quadro.
    pub(super) fn of(gfx: &'a mut AppGfx) -> Self {
        let AppGfx {
            // As guias do documento: lidas para desenhar e para alimentar o snap.
            guides: doc_guides,
            // Os ESTADOS de UI (plano UI/UX W7) e as MÁQUINAS que os mostram. Os dois viajam
            // juntos por toda a autoria: gravar lê o mundo e escreve a tabela, mostrar lê a
            // tabela e pede à máquina, e a máquina escreve o mundo de volta.
            ui_states,
            ui_machines,
            // A cena 3D é DESENHADA no `present`; aqui ela é lida por um assunto só — o bake do
            // objeto misto (`docs/3D/02.2`), que precisa do mundo e do renderizador ao lado dela.
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            // Os objetos que uma forma acende. NAO sao `cfg`-gated: a re-acendida deles roda sem o
            // modulo 3D no build, que e' a promessa da rota A (`docs/3D/02.2`).
            baked_forms,
            baked_light,
            next_baked_form,
            surface,
            renderer,
            sim,
            present,
            camera,
            canvas_zoom,
            asset_db,
            atlas_is_real: _,
            script,
            particles,
            theme,
            zen,
            toasts,
            jobs,
            tools,
            layout,
            game_rt,
            motion_fx,
            tonemap,
            compositor,
            vello_pass,
            vector_scene,
            vec_scene,
            // ADR-0114: cena Flip. A ponte objeto↔entidade é reconciliada todo
            // frame (abaixo, ao lado do vetor). O RENDER é no present phase.
            flip,
            // ADR-0114 W1: o rasterizador + a composição são usados no present.rs.
            flip_render: _,
            flip_compose: _,
            flip_composite: _,
            text_system,
            hero_screen,
            hero_arena,
            clipboard: _,
            prop_state,
            worklist,
            sort_scratch,
            sort_inputs,
            frame_order,
            // ADR-0154 Fase 2 — o acumulador e a colagem são do PRESENTE; aqui só se enche o
            // `band_doc_scenes` (a codificação do documento por faixa) porque é aqui que as
            // entradas do `dispatch` existem.
            world_rt,
            band_blit: _,
            band_doc_scenes,
            compositor_reads_world,
            hero_live,
            next_import_cell,
            // A identidade estável do documento pintado é carimbada no SAVE (`project_painter`), que é
            // o único momento em que ela precisa existir — o loop de render não a lê.
            next_painted_doc: _,
            // As folhas hand-packed: o loop de render não as lê. O que ele desenha é o COZIDO
            // (`Sprite.source` + `region_rect`), que o import e o load já escreveram — é
            // exatamente essa separação fonte-≠-cozido que faz o extract não mudar uma linha.
            sheets,
            // ⚠️ Eram `_` até o BAKE existir: assar cria uma folha e a textura dela, então os dois
            // mapas e o contador passam a ser ESCRITOS aqui. O comentário acima continua verdade
            // para o desenho — o extract lê o cozido —, e é a autoria que precisa deles.
            sheet_textures,
            next_sheet_id,
            atlas_asset_map,
            // ⭐⭐ A TAXONOMIA da biblioteca (wave A3) — publicada ao painel e mutada pelos verbos.
            catalogs: asset_catalogs,
            // ⭐⭐ A ÁRVORE DE TAGS (TOP-20 #9) — ver o campo.
            tags,
            // ⭐⭐ A recusa do último gesto do painel — a metade que sobrevive ao quadro.
            tags_problem,
            // ⚠️ **CONSIDERADA e deixada de fora**, pela razão exacta da `library_cache` abaixo: a
            // cache das tags é lida no `capture_project_state`, que corre depois deste bloco.
            tags_cache: _,
            logical_texture_map,
            component_registry,
            editor_queue,
            transform_type_id,
            visibility_type_id,
            name_type_id,
            sprite_type_id,
            image_edit_undo,
            // ADR-0054 W0.T6: registries held but not yet consumed
            // inside the render loop — W1 wires Open/Save user paths
            // through `imageio_importers.find_for(...)`.
            imageio_importers: _,
            imageio_exporters,
            // Motion Nodes: cooked per frame by `motion_bridge` (M0.T10) into its
            // reused instance buffer while the `motion` tool is active.
            motion,
            // Global rigid physics: stepped per frame by `physics_bridge`
            // (ADR-0131 W1) — reads RigidBody/Collider, writes Transform.
            physics,
            // ⚠️ A cache da captura incremental (F2) **não é do quadro** — ela é lida e escrita
            // pelo `post_frame_undo`, que corre depois disto. Listada por nome porque este padrão
            // é exaustivo de propósito: um campo novo tem de ser CONSIDERADO aqui, não ignorado
            // por um `..` que nunca mais ninguém relê.
            undo_capture_cache: _,
            // O alvo do `+` do Inspector (F3) — usado no fim deste mesmo quadro, ver abaixo.
            component_palette_target,
            // ⚠️ **CONSIDERADO e deixado de fora**: a cache da biblioteca é lida no
            // `capture_project_state`, que corre depois deste bloco e pega o `gfx` inteiro. Aqui
            // ela não tem consumidor — e o `_` diz isso, ao contrário de um binding nomeado que
            // parece um esquecimento.
            library_cache: _,
            // ⭐⭐⭐ **O VIDRO JATEADO** (2026-09-07) — o passe é do presente; aqui escrevem-se as
            // duas cenas que ele separa e o interruptor do quadro.
            frost: _,
            frost_doc_scene,
            frost_front_scene,
            frosting,
        } = gfx;
        Self {
            doc_guides,
            ui_states,
            ui_machines,
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            baked_forms,
            baked_light,
            next_baked_form,
            surface,
            renderer,
            sim,
            present,
            camera,
            canvas_zoom,
            asset_db,
            script,
            particles,
            theme,
            zen,
            toasts,
            jobs,
            tools,
            layout,
            game_rt,
            motion_fx,
            tonemap,
            compositor,
            vello_pass,
            vector_scene,
            vec_scene,
            flip,
            text_system,
            hero_screen,
            hero_arena,
            prop_state,
            worklist,
            sort_scratch,
            sort_inputs,
            frame_order,
            world_rt,
            band_doc_scenes,
            compositor_reads_world,
            hero_live,
            next_import_cell,
            sheets,
            sheet_textures,
            next_sheet_id,
            atlas_asset_map,
            asset_catalogs,
            tags,
            tags_problem,
            logical_texture_map,
            component_registry,
            editor_queue,
            transform_type_id,
            visibility_type_id,
            name_type_id,
            sprite_type_id,
            image_edit_undo,
            imageio_exporters,
            motion,
            physics,
            component_palette_target,
            frost_doc_scene,
            frost_front_scene,
            frosting,
        }
    }
}
