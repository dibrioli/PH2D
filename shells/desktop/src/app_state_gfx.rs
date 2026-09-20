//! **O `AppGfx`** — todo recurso que só existe depois do `resumed`, num agregado só para o quadro
//! poder partir os empréstimos campo a campo (o doc da struct diz porquê). Irmão de `app_state.rs`
//! pelo tecto de 600 LOC da shell.
//!
//! Corte mecânico: a struct saiu inteira, verbatim. O caminho `crate::AppGfx` não muda — o
//! `app_state.rs` re-exporta-a e o `main.rs` re-exporta o `app_state`.
//!
//! ⚠️ **Dois leitores de TEXTO liam os campos dela no `app_state.rs` e passaram a ler também
//! este ficheiro:** o censo de campos do `scripts/fecho-da-familia.py` (sem isso um
//! `gfx.<campo>` deixava de contar como âncora, e a régua do fecho passava a errar A FAVOR) e o
//! `the_authored_map_has_exactly_one_holder` (um segundo dono do Input Map escrito aqui passava
//! mudo).

use super::*;

/// Holds every initialized-after-`resumed` resource. Bundling them into
/// a single `Option<AppGfx>` lets us destructure into per-field `&mut`
/// borrows in `render_frame()` — split-borrowing through a method
/// chain on individual `Option<...>` fields would be awkward.
pub(crate) struct AppGfx {
    pub(crate) surface: SurfaceContext,
    pub(crate) renderer: SpriteRenderer,
    pub(crate) sim: SimWorld,
    pub(crate) present: PresentWorld,
    pub(crate) camera: Camera2d,
    /// **O destino do zoom da roda** (a UI viva no gesto mais repetido do app).
    ///
    /// ⚠️ Ele **não é uma segunda cópia do `camera.height_world`**: é um `Option` que só existe
    /// enquanto um gesto está em voo, semeado a partir do vivo no primeiro entalhe. Fora disso a
    /// câmera é de quem a escrever — o `View · All`, o load, as cenas de smoke. Ver
    /// [`crate::canvas_zoom`].
    pub(crate) canvas_zoom: crate::canvas_zoom::CanvasZoom,
    /// M6 — set when PNG fixtures loaded successfully; held so the
    /// AssetDb keeps `Arc<Asset>` alive for hot-reload follow-ups.
    pub(crate) asset_db: AssetDb,
    /// KTX2 Fase 2 (W2.T4): `LogicalTextureId → BTreeMap<TierIndex, AssetId>`
    /// — which cooked KTX2 artifact (in `asset_db`) backs a tier-agnostic
    /// logical texture for each platform tier. The cooked-texture loader
    /// pass (`cooked_texture_bridge`) reads this and the device tier to
    /// resolve and upload a `SpriteSource::CookedTexture` sprite's pixels.
    /// Empty until a cooked texture is loaded (e.g. the `PH2D_KTX2_SMOKE`
    /// harness).
    pub(crate) logical_texture_map: LogicalTextureMap,
    /// M6 — true when the atlas was composed from real PNG files (vs the
    /// procedural dummy fallback). Surfaced in window title.
    pub(crate) atlas_is_real: bool,
    /// M7 — Luau VM with placeholder script loaded. Per-frame gc_step
    /// keeps the GC budget warm; set/get bindings ready for follow-up
    /// gameplay work.
    pub(crate) script: Option<ScriptHost>,
    /// ⭐⭐⭐ **Os EMISSORES DE PARTÍCULAS a correr** (TOP-20 #18) — um por objecto com o
    /// componente, FORA do mundo: o que uma corrida de partículas guarda não é documento (a lei do
    /// `Spawned`), e registá-lo poria cada tique na pilha de `Ctrl+Z`.
    pub(crate) particles: ph2d_app_components::particles_bridge::ParticlesState,
    /// ⭐ **As BARRAS DE VIDA a correr** (plano 28, W4) — o rasto de cada barra, FORA do mundo pela
    /// mesma razão das partículas: o que escorre numa corrida não é documento.
    pub(crate) health_bars: ph2d_app_components::health_bar_bridge::HealthBarsState,
    /// M12 editor data layer + M11 widget paint pass.
    pub(crate) theme: Theme,
    pub(crate) zen: ZenMode,
    pub(crate) toasts: ToastQueue,
    /// Bars for work that outlives a frame (`ph2d_editor_core::progress`). Sibling of
    /// `toasts` in every way: ticked once per frame, painted into the same top-center
    /// column (under the toasts), and app-wide rather than per-feature — the AI Denoise
    /// is only the first caller, and batch LUFS / export / upscale have the same shape.
    ///
    /// It holds `Progress` handles, not the jobs: the typed `Job<T>` stays with whoever
    /// knows what to do with the result (e.g. `AudioSystem::ml_job`).
    pub(crate) jobs: JobQueue,
    /// Registered editor tools. Keys 1/2 switch active tool; the
    /// active tool's `build_panel()` is painted each frame as the
    /// FloatingPanel that shows in the bottom-center of the canvas.
    pub(crate) tools: ToolRegistry,
    /// 4-zone editor layout (ADR-0023 §3). Sized from window each
    /// resize; the M11 paint pass walks this to draw zone backdrops.
    pub(crate) layout: EditorLayout,
    /// M14.5: offscreen HDR (Rgba16Float) render target for the game
    /// world. Sprite + future light/particle/material passes write
    /// here; the tonemap pass reads here and writes to LDR. Recreated
    /// on resize.
    pub(crate) game_rt: GameRt,
    /// ⭐⭐ **O ACUMULADOR DO MUNDO** (ADR-0154 Fase 2) — onde as faixas de desenho se empilham na
    /// ordem que o ordenador decidiu. Só é usado quando a cena de facto INTERCALA vetor e sprite;
    /// sem isso o quadro corre o caminho de sempre, byte a byte.
    pub(crate) world_rt: ph2d_render::WorldRt,
    /// A colagem de uma faixa sobre o acumulador.
    pub(crate) band_blit: ph2d_render::BandBlit,
    /// As cenas do DOCUMENTO, uma por faixa de vetor — reusadas entre quadros (HR-3).
    ///
    /// ⚠️ Vazio ⇒ o documento foi codificado na cena do chrome, como sempre, e o presente não tem
    /// faixa de vetor nenhuma a desenhar.
    pub(crate) band_doc_scenes: Vec<ph2d_vector::VectorScene>,
    /// ⭐ **A grade ATRÁS dos objectos** (report do dono de 2026-09-24) — a cena que a
    /// `grid_layer::paint_behind` enche quando o `Behind` está ligado, e que o acumulador do mundo
    /// desenha logo depois do fundo. Reusada entre quadros (HR-3).
    pub(crate) grid_behind_scene: ph2d_vector::VectorScene,
    /// ⭐⭐⭐ **O VIDRO JATEADO** (2026-09-07) — o passe que borra o acumulador do mundo entre o
    /// desenho dele e o da receita aberta. Ver [`crate::render_loop::present_frost`].
    pub(crate) frost: ph2d_render::FrostPass,
    /// O DOCUMENTO sem a receita aberta, para o quadro que põe o vidro.
    ///
    /// ⚠️ **Cena própria e não a do chrome:** com o vidro, o documento tem de aterrar no
    /// acumulador ANTES do borrão, e o chrome depois dele. Vazia (e nunca desenhada) sem receita
    /// aberta, ou quando o quadro está intercalado — aí o documento já vive nas faixas.
    pub(crate) frost_doc_scene: ph2d_vector::VectorScene,
    /// A RECEITA sozinha — o que fica nítido acima do vidro.
    pub(crate) frost_front_scene: ph2d_vector::VectorScene,
    /// Há uma receita aberta NESTE quadro? Escrito pela codificação, lido pelo presente.
    ///
    /// ⚠️ **Um campo, e não uma segunda pergunta ao mundo:** quem sabe é a vista do vetor
    /// (`VecViewState::isolating`), que só existe dentro do bloco de codificação; o presente
    /// re-derivá-la seria a segunda resposta, e um quadro em que as duas discordassem desenharia
    /// a receita duas vezes ou nenhuma.
    pub(crate) frosting: bool,
    /// O compositor está a ler o acumulador (em vez da saída do tonemap)?
    ///
    /// ⚠️ **Ele guarda o `game_view` num bind group construído uma vez**, então trocar a fonte é um
    /// `rebind`. Esta bandeira faz o rebind acontecer **na mudança de modo**, e não por quadro.
    pub(crate) compositor_reads_world: bool,
    /// doc 67: the Motion module's own HDR glow pass. Owns a Motion-only
    /// `Rgba16Float` RT + the bloom chain. Runs only when the artist has
    /// authored bloom on the active Motion doc (`fx.bloom.enabled`); otherwise it
    /// is inert and the frame is byte-identical. Sized alongside `game_rt`.
    pub(crate) motion_fx: ph2d_render::MotionFx,
    /// M14.5: AgX tonemap pass — owns its own LDR output texture
    /// (`game_rt_ldr`). Sampled by the compositor as the "game layer"
    /// input. Identity LUT by default; swap in real AgX via
    /// `set_lut`.
    pub(crate) tonemap: Tonemap,
    /// M14.5: compositor that composes `tonemap.output_view()` (game)
    /// and `vello_pass.intermediate_view()` (UI chrome) onto the swap
    /// chain. Replaces the old `vello_pass.blitter` direct-to-surface
    /// blit so chrome and game live in fully isolated RTs.
    pub(crate) compositor: Compositor,
    /// Vello pipeline + intermediate texture for the widget paint
    /// pass. In M14.5 the intermediate is sampled by `compositor`
    /// (not blitted directly to the surface).
    pub(crate) vello_pass: VelloPass,
    /// Reused [`VectorScene`] — encoded fresh each frame; allocations
    /// pool inside Vello so this is cheap.
    pub(crate) vector_scene: VectorScene,
    /// ADR-0108 Fase 0: cena vetorial NOVA (modelo editor-first). Renderizada
    /// pela pipeline `ph2d-vec-render` no canvas, sob a mesma `vector_scene`
    /// Vello compartilhada. Fase 0 = cena-demo (prova o seam ponta-a-ponta);
    /// Fase 1 = dirigida pelas ferramentas de desenho.
    pub(crate) vec_scene: ph2d_vec_scene::VecScene,
    /// As **GUIAS** do documento (plano 25 §9) — as linhas que o artista arrasta da régua.
    ///
    /// Vivem AQUI, ao lado da geometria, porque viajam no [`crate::undo::ProjectState`]: é isso
    /// que lhes dá undo e save de graça, pelo mesmo diff que cobre o mundo, o vetor e o Flip.
    /// Não são entidades ECS de propósito — uma guia herdaria Hierarquia, gizmo, marquee e
    /// z-order, e cada um deles seria um comportamento a SUPRIMIR.
    pub(crate) guides: ph2d_guides::GuideSet,
    /// Os **ESTADOS de UI** (plano UI/UX W7) — idle/hover/press por objeto, e a pose que a cena
    /// tem entre dois deles.
    ///
    /// Aqui ao lado das guias e pela mesma razão: viajam no [`crate::undo::ProjectState`], então
    /// gravar um estado desfaz e salva de graça. ⚠️ E a chave é o `VecPathId` do hospedeiro,
    /// nunca a entidade — bits de entidade são id de ALOCAÇÃO e o undo respawna tudo com bits
    /// novos, o que perderia a tabela no primeiro Ctrl+Z.
    pub(crate) ui_states: ph2d_ui_state::StateSets,
    /// **As MÁQUINAS vivas** (plano UI/UX W7) — onde a cena está entre duas poses.
    ///
    /// ⚠️ Ao lado da tabela e **fora** do [`crate::undo::ProjectState`], porque respondem a
    /// perguntas diferentes: a tabela diz *onde as poses são* (documento, salva, desfaz), a
    /// máquina diz *onde a cena está agora* (vista). Salvá-la faria um projeto reabrir a meio
    /// caminho de uma transição.
    pub(crate) ui_machines: crate::render_loop::ui_state_bridge::UiMachines,
    /// ADR-0114: cena Flip (animação quadro-a-quadro). Documento puro
    /// (`ph2d-flip`); cada objeto vira uma entidade na Hierarquia via
    /// `FlipObjectRef`, ponte mantida por `flip_entities::sync`. Mirror do
    /// `vec_scene` — document ≠ tool. Guardada no `ProjectState` (undo/save).
    pub(crate) flip: ph2d_flip::FlipDoc,
    /// ADR-0114 W1: o pipeline wgpu que rasteriza o traço do Flip no `game_rt`
    /// (HDR), amostrado pelo playhead. Criado 1× (device + formato do game_rt).
    pub(crate) flip_render: ph2d_flip_render::FlipRenderer,
    /// ADR-0150 W1/M2: a cena 3D viva (malha + camera orbital + pipeline).
    /// `None` num run normal — so o smoke a cria, e sem ela toda porta do
    /// `ph2d_app_sculpt3d` devolve `false` e o frame 2D fica intocado.
    #[cfg(feature = "sculpt3d")]
    pub(crate) sculpt3d: Option<ph2d_app_sculpt3d::Sculpt3dScene>,
    /// **OS SPRITES QUE UMA FORMA ACENDE** (`docs/3D/02.2`, rota A), por bits de entidade.
    ///
    /// ⚠️ **Ele mora aqui, e NÃO dentro da cena 3D, porque um objeto assado sobrevive à escultura —
    /// e ao módulo.** Enquanto o mapa morasse na `Sculpt3dScene`, apagar a peça levaria os canais
    /// junto, e um binário sem a feature não teria onde pôr o que o arquivo trouxe. A promessa da
    /// rota A é exatamente essa: *a malha some do build, o objeto continua reluminável.*
    ///
    /// ⚠️ **NÃO é `cfg`-gated**, e é isso que torna a promessa verificável em vez de prosa.
    pub(crate) baked_forms:
        std::collections::BTreeMap<u64, ph2d_form_donation::baked_form::BakedForm>,
    /// **OS PASSES que ACENDEM um objeto assado** — um por LEI (a tinta do Painter e o OpenPBR da
    /// forma), cada um construído na primeira acendida DELE.
    ///
    /// ⚠️ Era um `Option<ImpastoLightPass>` até a lei da forma ganhar o passe de dispositivo
    /// (`docs/Render3d/15` §7). ⛔ Um segundo `Option` ao lado deste seria **quatro sítios a ter de
    /// concordar** (aqui, no `FrameGfx` e nas duas fases do quadro), e uma terceira lei a custar as
    /// quatro edições outra vez — ver o doc da [`ph2d_form_donation::baked_form::PassesDaLuz`].
    ///
    /// ⚠️ **Sem `Option` à volta, e isso NÃO é uma capacidade nova:** o vazio dele é o `Default`, e
    /// cada passe continua a nascer preguiçoso. Ver o [`crate::project_forget`] para porque ele não
    /// é limpo ao esquecer um projecto.
    pub(crate) baked_light: ph2d_form_donation::baked_form::PassesDaLuz,
    /// ADR-0114 W1 T1.7: as passagens de espaço-de-cor (resolve 16F→sRGB8 e blit
    /// sRGB8→16F) que ligam o rasterizador ao `LayerCompositor` do Painter. Criado
    /// 1× (device + formato do game_rt). O compositor em si é o `flip_composite`.
    pub(crate) flip_compose: ph2d_flip_render::FlipCompose,
    /// ADR-0114 W1 T1.7: a sessão de composição por-camada do Flip — o
    /// `LayerCompositor` 22-modos do Painter + o buffer dummy que satisfaz o
    /// filtro de tamanho do `ensure_slice` (as fatias reais entram por
    /// `inject_slice_from_texture`, então o dummy nunca é subido). Lazy: só nasce
    /// quando há camada Flip ativa (cena vazia = `None`, sem custo de GPU).
    pub(crate) flip_composite: Option<ph2d_app_flip::pass::FlipComposite>,
    /// Motion Nodes runtime state (M0.T8): document + transport + persistent
    /// `Cook` + node registry + reused instance buffer. Cooked per frame by
    /// `render_loop::motion_bridge` while the `motion` tool is active. Mirror of
    /// `vec_scene` — document ≠ tool (ADR-0040).
    pub(crate) motion: ph2d_app_motion::motion_state::MotionState,
    /// Global rigid-body physics (ADR-0131 W1). Owns the transient rapier
    /// world + entity↔handle map, driven at the `Playhead` tick by
    /// `render_loop::physics_bridge`. Derived from `RigidBody`/`Collider`
    /// components each frame; NOT serialized (rebuilt on load/undo).
    pub(crate) physics: ph2d_physics_ecs::PhysicsBridge,
    /// parley font + layout context (heavy state). Threaded through
    /// `PaintCtx` so future text passes don't re-load fonts.
    pub(crate) text_system: TextSystem,
    /// Hero screen (`02-editor-main` mockup) — populated by default;
    /// `None` only when `PH2D_M5_DEMO=1` selects the legacy
    /// 1000-sprite perf demo. Owns the [`WidgetStore`] + [`HitIndex`]
    /// so input pipeline (ADR-0024) can route pointer/key events
    /// through `dispatch_*`.
    pub(crate) hero_screen: Option<HeroScreen>,
    /// Per-frame arena for [`WidgetEvent`]s emitted by the hero
    /// dispatcher. Reset at end-of-frame.
    pub(crate) hero_arena: Bump,
    /// OS clipboard handle — used by Cmd+C/V/X. `None` when the OS
    /// rejected our request (rare; we just no-op those keys then).
    pub(crate) clipboard: Option<arboard::Clipboard>,
    /// M14.1 — cached `QueryState` pair for hierarchical transform
    /// propagation. Built once after `populate_sim`; used every frame
    /// inside the extract phase. The only way to iterate `&World`
    /// from inside `extract!`.
    pub(crate) prop_state: TransformPropagationState,
    /// ⭐ **A cache da captura incremental do desfazer** (ADR-0164 §2.7 / F2).
    ///
    /// ⚠️ **Vive aqui, ao lado do `prop_state`, porque tem a MESMA vida: a do mundo.** Ela guarda
    /// a linha de snapshot de cada objeto e o tick da última captura — pô-la fora daqui (num
    /// `static`, ou reconstruída por chamada) faria o desfazer voltar a custar o tamanho do mundo
    /// a cada quadro, que é exactamente o que esta fase existe para apagar.
    ///
    /// ⚠️ Quem a esquece é o RESTORE (`apply_project`): o undo repõe um mundo que ela não viu
    /// nascer, e continuar a comparar contra ela daria linhas limpas sobre bytes diferentes.
    pub(crate) undo_capture_cache: ph2d_ecs::scene::incremental::CaptureCache,
    /// ⭐ **A quem o `+` do Inspector abriu a paleta** (ADR-0166 / F3).
    ///
    /// ⚠️ Ele vive entre QUADROS: o clique abre, o artista escolhe um ou dez quadros depois, e o
    /// pick tem de saber a quem anexar. Re-derivar da seleção no momento do pick funcionaria hoje
    /// (o modal tem scrim), mas *"a seleção não pode mudar"* é um invariante que ninguém impõe —
    /// e o precedente da casa é guardar o contexto (a biblioteca do Motion guarda o `library_open`).
    pub(crate) component_palette_target: Option<u64>,
    /// M14.1 — pre-allocated DFS worklist for `propagate_transforms`.
    /// Capacity sized to `WorklistBuf::DEFAULT_CAPACITY` (8 192
    /// entities) — comfortably above `SPRITE_COUNT = 1000`. HR-3
    /// zero-alloc verified by `crates/ph2d-ecs/tests/propagate_no_alloc.rs`.
    pub(crate) worklist: WorklistBuf,
    /// W3.T3.8 reusable scratch + per-frame input buffer for the
    /// canonical sorting pipeline; threaded like `worklist` so the sort
    /// stays allocation-free after warm-up (HR-3).
    pub(crate) sort_scratch: ph2d_ecs::sort_key::SortScratch,
    pub(crate) sort_inputs: Vec<ph2d_ecs::sort_key::SortInput>,
    /// ⭐⭐ **A ordem TOTAL do quadro** (ADR-0154 Fase 2) — as duas famílias (sprite e forma
    /// vetorial) numa lista só, indexada pelo rank. Preenchida pelo `sim_extract` a partir do
    /// ordenador único; lida pelo presente, que a parte em faixas de desenho.
    ///
    /// ⚠️ Vive aqui pela razão do `sort_inputs` ao lado: ela é reusada e limpa por quadro, para o
    /// caminho quente ficar sem alocação depois do aquecimento (HR-3).
    pub(crate) frame_order: crate::draw_bands::FrameOrder,
    /// M14.4a live-bridge state. Present in the default editor mode
    /// (i.e. always unless `PH2D_M5_DEMO=1` switched to the legacy
    /// untouched.
    pub(crate) hero_live: Option<HeroLive>,
    /// M14.4c+M14.4d: next free atlas key for imported images.
    /// Starts at `FIRST_IMPORT_KEY` (= 16) so it sits past the
    /// demo's seeded HSV tile keys (0..15). With the Skyline atlas
    /// the key space is effectively unbounded (the underlying packer
    /// runs out of pixel space before `u32` does), so we just
    /// increment monotonically — no cycling, no overwrite. When
    /// the atlas does run out of room the regrow path
    /// (`insert_atlas_sprite_with_regrow`) uses `atlas_asset_map` to
    /// recover each existing region's source bytes from `asset_db`.
    pub(crate) next_import_cell: u32,
    /// **AS FOLHAS hand-packed da sessão**, por id estável (plano `docs/Sprite_projeto/17` §6).
    ///
    /// A folha é um documento AUTORADO — os pixels mudam quando o artista rearranja uma região —,
    /// e por isso a identidade é um `u32` estável e não o hash do conteúdo que os pixels próprios
    /// de um sprite usam. Os sprites apontam para uma região dela pelo `ph2d_ecs::SpriteSheetRef`.
    pub(crate) sheets: std::collections::BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
    /// `id da folha → texture_id` na sessão corrente.
    ///
    /// ⚠️ **Runtime puro, e é essa a divisão que este módulo inteiro existe para manter:** o
    /// `texture_id` é uma alocação da GPU que morre com o processo, então ele NUNCA entra no
    /// arquivo — quem viaja é o `sheets` acima, e este mapa é reconstruído no load.
    pub(crate) sheet_textures: std::collections::BTreeMap<u32, u32>,
    /// Próximo id livre de folha (mesmo contrato do `next_import_cell`: o load empurra-o para
    /// além dos ids do projeto, para um import novo não se sentar em cima de uma folha carregada).
    pub(crate) next_sheet_id: u32,
    /// Próximo `ph2d_ecs::PaintedDoc(u32)` livre — a identidade ESTÁVEL de um documento do Painter.
    ///
    /// Os bits de entidade (a chave do `doc_cache` do Painter) são id de ALOCAÇÃO do ECS: morrem no
    /// restore, que despawna tudo e recria. Este contador dá ao documento um nome que sobrevive ao
    /// arquivo — e, como a componente viaja no `WorldSnapshot`, ao undo também. Monotônico; o load o
    /// avança para além dos ids do projeto (mesmo contrato de `next_import_cell`).
    pub(crate) next_painted_doc: u32,
    /// Próximo `ph2d_ecs::BakedForm(u32)` livre — a identidade ESTÁVEL dos canais assados de uma
    /// malha. Mesmo contrato do `next_painted_doc`: monotônico, e o load o avança para além dos ids
    /// do projeto, senão um bake novo nesta sessão sobrescreveria o documento de um objeto carregado.
    pub(crate) next_baked_form: u32,
    /// M14.7 polish: atlas-key → AssetId map kept in sync with each
    /// import. Drives the regrow callback so doubling the atlas
    /// texture preserves every previously-imported sprite. BTreeMap
    /// per HR-5 / ADR-0022.
    pub(crate) atlas_asset_map: BTreeMap<u32, AssetId>,
    /// ⭐⭐ **A TAXONOMIA da biblioteca de assets** (plano 07, wave A3) — os catálogos e a que
    /// catálogo cada asset pertence.
    ///
    /// ⚠️ **Ela é do PROJECTO, e não da sessão** (ao contrário da `TextureLibrary`, que é memória
    /// de conteúdo): viaja no `.ph2dproj` **dentro do `ProjectState`**, pelo blob auto-versionado
    /// do [`crate::project_catalogs`] que o [`crate::project_library`] compõe.
    ///
    /// ⭐ **E por isso ela DESFAZ** (Enio, 2026-08-30). ⚠️ Quem a codifica é a
    /// [`AppGfx::library_cache`] ao lado — codificá-la por quadro custava até 28 % de um quadro.
    pub(crate) catalogs: ph2d_asset_index::CatalogTree,
    /// ⭐ A cache que impede a taxonomia de ser re-codificada por quadro — ver
    /// [`crate::project_library::LibraryCache`], com a tabela da medição.
    pub(crate) library_cache: crate::project_library::LibraryCache,
    /// ⭐⭐⭐ **A ÁRVORE DE TAGS do projecto** (TOP-20 #9) — a taxonomia que o artista escreve e a
    /// que os objectos pertencem por [`ph2d_ecs::Tags`].
    ///
    /// ⚠️ **Ela é do PROJECTO e DESFAZ**, como os catálogos: viaja no `.ph2dproj` dentro do
    /// `ProjectState` (o blob auto-versionado do [`ph2d_app_components::tags_doc`]), e apagar uma
    /// tag desfaz-se junto com a pertença que o gesto levou.
    pub(crate) tags: ph2d_tags::TagTree,
    /// ⭐ A cache que impede a árvore de ser re-codificada por quadro — a irmã exacta da
    /// [`AppGfx::library_cache`], e pela medição que aquela pagou.
    pub(crate) tags_cache: ph2d_app_components::tags_doc::TagsCache,
    /// ⭐⭐ **A RECUSA do último gesto sobre a árvore** — `(tag, frase)`, pintada NA LINHA dela.
    ///
    /// ⚠️ **Estado da shell e não valor derivado:** a lei devolve o `Result` no instante do gesto e
    /// o painel repinta dezenas de vezes depois disso. Sem esta memória, a frase apareceria um
    /// quadro e sumia — e o artista teria de repetir o gesto para saber porque ele falhou.
    ///
    /// ⚠️ A frase vem de `ph2d_tags::TagError::message`, ao lado da lei; a shell não a escreve.
    pub(crate) tags_problem: Option<(u64, String)>,
    /// M14.A: editor → SimWorld mutation pipeline. Populated at boot
    /// with the canonical Transform / Name / Visibility / RootOrder
    /// type registrations via `register_ecs_components`; future crates
    /// (`ph2d-render` for Sprite, `ph2d-script` for LuauScript) will
    /// extend it as their components join the live inspector.
    ///
    /// First real consumer is the Inspector's Transform editor: each
    /// commit pushes `EditorCommand::SetComponent` to
    /// [`Self::editor_queue`], which `apply_editor_commands` drains
    /// once per frame to write back to SimWorld via this registry.
    pub(crate) component_registry: ComponentRegistry,
    /// Editor command queue (Arc<Mutex<…>>-backed for multi-producer
    /// access). The Inspector's commit path is the only producer
    /// today; the shell drains and applies once per frame after the
    /// hero `apply_event` pass.
    pub(crate) editor_queue: EditorCommandQueue,
    /// Cached stable type id for `ph2d::ecs::Transform`. Lookup is
    /// blake3-of-name → first 8 bytes; cached so the per-commit push
    /// path doesn't re-hash. The matching registry entry was added
    /// via `register_ecs_components`.
    pub(crate) transform_type_id: u64,
    /// M14.D: same as `transform_type_id` for the `ph2d::ecs::Visibility`
    /// component. Cached at boot so the Inspector visibility checkbox
    /// commit doesn't re-hash on every toggle.
    pub(crate) visibility_type_id: u64,
    /// M14.E: same cache for `ph2d::ecs::Name`. Used when draining
    /// `pending_name_edit` from the Inspector's editable name field.
    pub(crate) name_type_id: u64,
    /// M14.C audit fix #8: cache for `ph2d::render::Sprite` so the
    /// Strategy switch commit goes through the canonical
    /// `EditorCommand::SetComponent` pipeline (parity with Transform /
    /// Visibility / Name). Loaded into the registry via
    /// `register_render_components` at boot.
    pub(crate) sprite_type_id: u64,
    /// Single-level image-edit undo — one TRANSACTION (covers a whole
    /// multi-sprite Apply) per slot. Cmd+Z (or TOOL_UNDO click) restores
    /// every sprite the transaction touched. The full editor undo system
    /// (a multi-transaction stack rooted in `EditorCommandQueue`) is
    /// M14.x scope.
    ///
    /// **Cross-sprite contract** (was the §1 audit CRITICAL): each
    /// per-sprite drain appends ONE `ImageEditSnapshot` to the in-flight
    /// `pending_entries` vec the orchestrator owns; once the multi-sprite
    /// loop completes, the orchestrator commits a single
    /// `ImageEditTransaction` here, releasing the previous transaction's
    /// pre-edit individual textures (if any). Undo pops the whole vec —
    /// restoring N sprites in one keypress — instead of just the last
    /// sprite the old single-slot design retained.
    pub(crate) image_edit_undo: Option<ImageEditTransaction>,
    /// ADR-0054 W0.T6 — image I/O importer registry populated at boot
    /// by `ph2d_imageio_registry_init::register_all_importers`. Held on
    /// `AppGfx` so future Open-file paths (W1+) can `find_for` a
    /// recognized format and decode straight to `DecodedImage`. Empty
    /// outside the format crates' coverage (W0: PNG only).
    #[allow(
        dead_code,
        reason = "W0.T6 stages the registry; W1+ wires Open into find_for"
    )]
    pub(crate) imageio_importers: ImporterRegistry,
    /// ADR-0054 W0.T6 — image I/O exporter registry populated at boot
    /// by `ph2d_imageio_registry_init::register_all_exporters`. Held
    /// for future Save-as paths (W1+).
    #[allow(
        dead_code,
        reason = "W0.T6 stages the registry; W1+ wires Save into find_for"
    )]
    pub(crate) imageio_exporters: ExporterRegistry,
}
