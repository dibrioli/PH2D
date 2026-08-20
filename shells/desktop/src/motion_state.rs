//! `MotionState` — the shell-owned runtime aggregate for the Motion Nodes module
//! (Motion Nodes M0.T8). Held on `AppGfx.motion`; driven per frame by
//! `render_loop::motion_bridge` while the `motion` tool is active.
//!
//! Bundles the persistable document ([`MotionDoc`], with undo [`MotionHistory`])
//! with the runtime pieces that never persist: the **persistent** [`Cook`] (its
//! memo + `pre` feedback must survive across frames), the node [`NodeRegistry`]
//! (the `OpResolver`), the current sink node, and a reused `Vec<RenderInstance>`
//! lowering buffer (so the steady-state cook path is zero-alloc — gated by M0.T12).
//!
//! **No transport (W4.T7).** Motion used to keep a `MotionTransport` of its own
//! here, advanced by each frame's fixed steps, while the timeline ran
//! `ph2d_core::Playhead` — two clocks that each advanced themselves, and so two
//! clocks that could drift. The editor now has ONE: the bridge DERIVES the tick it
//! cooks from the playhead (`motion_bridge::motion_tick`), and the pump's own
//! `last_cooked_tick` is the only record of where the sim stands. Do not add a
//! tick back here.
//!
//! Document ≠ tool (ADR-0040): the `MotionTool` is a thin activation handle; all
//! the state lives here in the shell, mirroring `AppGfx.vec_scene`.

/// **A neve NÃO BOOTA mais** (Enio, 2026-08-07: *"tire a cena da cachoeira"*) — o editor de
/// Motion abre com a tela VAZIA, e o artista traz o que quiser pelo command-palette (`A`).
///
/// A cena continua aqui, `#[cfg(test)]`, porque ela tem CONSUMIDORES que não são o boot: o
/// censo de cobertura de GPU (`motion_gpu_coverage`) a chama de *"o único documento de nível
/// de artista"* e escolhe o próximo kernel medindo a fronteira de CPU DELA; o gate do
/// `motion.delay` usa os números do mar como fixture; e `motion_state_tests` prova o
/// save/load/relógio sobre um grafo RICO — sobre um documento vazio essas três provas ficam
/// vácuas. É o padrão do `warp_axis`/`serial_side`: o que perde o chamador de produção vira
/// **referência congelada**, não código morto e não um arquivo apagado.
#[cfg(test)]
#[path = "motion_demo_strobe.rs"]
mod strobe;

/// As cenas de GRUPO da conferência (doc 89, a segunda volta): o documento que cada uma
/// monta e a PROSA que ela imprime. ⚠️ O roteador continua a ser a ÚNICA lista de níveis —
/// este arquivo não tem `match` nenhum, de propósito (ver o cabeçalho dele).
#[path = "motion_state_demo_conferencia.rs"]
mod demo_conferencia;
/// As cenas de grupo da família ANIMADORES (folha 06) — o irmão do acima, cortado por
/// ASSUNTO quando aquele arquivo cruzou o teto de LOC. Também sem `match`.
#[path = "motion_state_demo_conferencia_animadores.rs"]
mod demo_conferencia_animadores;
#[path = "motion_state_demo_router.rs"]
mod demo_router;
/// ⚠️ **`pub(crate)` e não privada como as irmãs**, e por um consumidor real: as cenas do
/// COMPASSO (`=25`) e do GRITO (`=26`) são a fixture dos gates da fronteira de sinais, que
/// moram no `render_loop` (é lá que a tomada é lida). Uma cena existe para ser DIRIGIDA.
#[path = "motion_state_gpu_adsr_demo.rs"]
pub(crate) mod gpu_adsr_demo;
/// A cena da CENTELHA QUE ESTOURA (doc 89, folha 13), arquivo próprio pela mesma razão: ela
/// responde *"uma MORTE consegue dar à luz?"* — o `sim.replicate` da referência, que aqui é
/// uma fiação e não um nó.
#[path = "motion_state_gpu_death_demo.rs"]
mod gpu_death_demo;
/// The DEFORMER scene, its own file for the same reason: it answers "does a node
/// whose kernel needs one number about the WHOLE stream run on the device?",
/// which none of the per-element or neighbourhood scenes can.
#[path = "motion_state_gpu_demos_deform.rs"]
mod gpu_deform_demo;
#[path = "motion_state_gpu_demos.rs"]
mod gpu_demos;
/// The FIELD scenes (`field.*`, =17..=22 incl. the A1-gpu Curve contour), split
/// out at the HR-18 cap — a cohesive family like the deformers next door.
#[path = "motion_state_gpu_field_demos.rs"]
mod gpu_field_demos;

/// As cenas da CONFERÊNCIA DOS NÓS (doc 89) — as quatro metades que só o olho
/// julga, `PH2D_GPU_COOK_DEMO=32..35`.
#[path = "motion_state_conferencia_demos.rs"]
mod conferencia_demos;
/// A cena da ARITMETICA (`=41`) — o grupo A da conferencia (cinco nos irmaos do
/// dominio de VALOR), irmao pelo mesmo teto de LOC.
#[path = "motion_state_conferencia_demos_arith.rs"]
mod conferencia_demos_arith;

/// A cena do AUDIO (`=40`), irmao pelo mesmo motivo — e porque ela escreve a
/// propria fixture em disco (nao ha asset de audio no repo).
#[path = "motion_state_conferencia_demos_audio.rs"]
mod conferencia_demos_audio;
/// A cena da DIRECAO (`=38`) mora num irmao: o pai bate no teto de LOC da shell.
#[path = "motion_state_conferencia_demos_direction.rs"]
mod conferencia_demos_direction;
#[path = "motion_state_conferencia_demos_stats.rs"]
mod conferencia_demos_stats;
#[path = "motion_state_conferencia_demos_table_seed.rs"]
mod conferencia_demos_table_seed;

// O grupo E — a comparação e o nome que não resolve (cena `=45`).
// ⚠️ `pub(crate)` porque o gate do badge da cena mora no `render_loop` (ver o doc
// do `build_compare_demo_document`).
#[path = "motion_state_conferencia_demos_compare.rs"]
pub(crate) mod conferencia_demos_compare;
// O grupo F — o ENVELOPE: que forma tem uma coisa que acende e apaga (cena `=46`).
#[path = "motion_state_conferencia_demos_envelope.rs"]
pub(crate) mod conferencia_demos_envelope;

#[path = "motion_state_conferencia_demos_velocity.rs"]
pub(crate) mod conferencia_demos_velocity;

#[path = "motion_state_conferencia_demos_collide.rs"]
pub(crate) mod conferencia_demos_collide;
#[path = "motion_state_conferencia_demos_pin.rs"]
pub(crate) mod conferencia_demos_pin;
#[path = "motion_state_conferencia_demos_proximity.rs"]
pub(crate) mod conferencia_demos_proximity;
#[path = "motion_state_conferencia_demos_rate.rs"]
pub(crate) mod conferencia_demos_rate;

#[path = "motion_state_conferencia_demos_octave.rs"]
pub(crate) mod conferencia_demos_octave;

#[path = "motion_state_conferencia_demos_shape.rs"]
pub(crate) mod conferencia_demos_shape;

#[path = "motion_state_conferencia_demos_axes.rs"]
pub(crate) mod conferencia_demos_axes;
#[path = "motion_state_conferencia_demos_clock.rs"]
pub(crate) mod conferencia_demos_clock;
#[path = "motion_state_conferencia_demos_column.rs"]
pub(crate) mod conferencia_demos_column;
#[path = "motion_state_conferencia_demos_field_space.rs"]
pub(crate) mod conferencia_demos_field_space;
#[path = "motion_state_conferencia_demos_join.rs"]
pub(crate) mod conferencia_demos_join;
#[path = "motion_state_conferencia_demos_sortkey.rs"]
pub(crate) mod conferencia_demos_sortkey;

#[path = "motion_state_conferencia_demos_space.rs"]
pub(crate) mod conferencia_demos_space;
#[path = "motion_state_conferencia_demos_substep.rs"]
pub(crate) mod conferencia_demos_substep;
#[path = "motion_state_conferencia_demos_taper.rs"]
pub(crate) mod conferencia_demos_taper;

#[path = "motion_state_conferencia_demos_cursor.rs"]
pub(crate) mod conferencia_demos_cursor;

#[path = "motion_state_conferencia_demos_field.rs"]
pub(crate) mod conferencia_demos_field;

#[path = "motion_state_conferencia_demos_drizzle.rs"]
pub(crate) mod conferencia_demos_drizzle;

#[path = "motion_state_conferencia_demos_deform.rs"]
pub(crate) mod conferencia_demos_deform;
#[path = "motion_state_conferencia_demos_text.rs"]
mod conferencia_demos_text;
#[path = "motion_state_conferencia_demos_time.rs"]
mod conferencia_demos_time;
#[path = "motion_state_conferencia_demos_wave.rs"]
pub(crate) mod conferencia_demos_wave;
#[path = "motion_state_conferencia_demos_weight.rs"]
pub(crate) mod conferencia_demos_weight;
/// A cena da MARCA DO IMPACTO (doc 89, folha 13), arquivo próprio pela mesma razão: ela
/// responde *"um nó a jusante consegue saber que houve uma COLISÃO?"* — o passo mexe em `P` e
/// `vel` a cada tique, então até existir a coluna `hit` a pergunta não era exprimível.
#[path = "motion_state_gpu_hit_demo.rs"]
mod gpu_hit_demo;
/// The NEIGHBOURHOOD scenes (ADR-0140), split out for the same reason: they
/// answer the interacting-sim question none of the throughput scenes can.
#[path = "motion_state_gpu_neighbour_demos.rs"]
mod gpu_neighbour_demos;
/// The panel scene, split out at the HR-18 cap — see the module's own note on
/// why the seam is the QUESTION each scene answers, not the line count.
#[path = "motion_state_gpu_panel_demo.rs"]
mod gpu_panel_demo;
/// A cena do PORTÃO ESPACIAL (doc 89, folha 12), arquivo próprio pela mesma razão: ela
/// responde *"um CAMPO consegue decidir quem escuta um EVENTO?"*, que nenhuma das outras faz.
#[path = "motion_state_gpu_pulse_demo.rs"]
mod gpu_pulse_demo;
/// A cena do RAIO DA PARTÍCULA (doc 89, folha 13), arquivo próprio pela mesma razão: ela
/// responde *"o que colide tem TAMANHO?"* mostrando as duas metades lado a lado — o colisor
/// de PONTO, que afunda cada sprite pela própria metade, e o que POUSA.
#[path = "motion_state_gpu_radius_demo.rs"]
mod gpu_radius_demo;
/// A cena da CALHA (doc 89, folha 13), arquivo próprio pela mesma razão: ela responde *"um
/// plano do colisor consegue ser outra coisa que HORIZONTAL?"* — a rampa TRANSPORTA (o que
/// uma cadeia de chãos, que só sabe construir uma escada, não alcança) e a parede PARA.
#[path = "motion_state_gpu_ramp_demo.rs"]
mod gpu_ramp_demo;
/// A cena das CINCO FONTES (doc 89, folha 12), arquivo próprio pela mesma razão: ela responde
/// *"um EVENTO consegue decidir o que passa a EXISTIR?"*, que é a outra metade da pergunta —
/// a `=23` gateia quem escuta, esta gateia quem nasce.
#[path = "motion_state_gpu_spawn_pulse_demo.rs"]
mod gpu_spawn_pulse_demo;
/// A cena do TETO DE VELOCIDADE (doc 89, folha 13), arquivo próprio pela mesma razão: ela
/// responde *"uma operação por-ELEMENTO sobre a velocidade é exprimível?"* — a folha mediu que
/// nenhuma cadeia do catálogo escreve `vel` por elemento, e um atrator forte é onde isso dói.
#[path = "motion_state_gpu_speed_demo.rs"]
mod gpu_speed_demo;
/// The Lloyd/JFA scene (ADR-0139), its own file for the same reason: it answers
/// "does a node whose cook is a multi-pass ALGORITHM run on the device?".
#[path = "motion_state_gpu_voronoi_demo.rs"]
mod gpu_voronoi_demo;
/// The sim-zone scene (ADR-0135), its own file for the same reason: it answers
/// "does the state-loop CONTAINER run on the device?", which none of the others do.
#[path = "motion_state_gpu_zone_demo.rs"]
mod gpu_zone_demo;

use gpu_deform_demo::{
    build_gpu_deform_demo_document, build_gpu_deform_organism_demo_document,
    build_gpu_four_point_warp_demo_document, build_gpu_kaleidoscope_demo_document,
    build_gpu_spherize_demo_document,
};
use gpu_demos::{
    build_gpu_demo_document, build_gpu_emitter_demo_document, build_gpu_hybrid_demo_document,
    build_gpu_sea_demo_document, build_gpu_sim_demo_document,
};
use gpu_field_demos::{
    build_gpu_field_box_demo_document, build_gpu_field_combine_demo_document,
    build_gpu_field_curve_demo_document, build_gpu_field_index_range_demo_document,
    build_gpu_field_radial_sweep_demo_document, build_gpu_field_remap_demo_document,
};
use gpu_neighbour_demos::{
    build_gpu_boids_demo_document, build_gpu_collide_demo_document, build_gpu_sweep_demo_document,
};
use gpu_panel_demo::build_gpu_panel_demo_document;
use gpu_pulse_demo::build_gpu_pulse_gate_demo_document;
use gpu_voronoi_demo::build_gpu_voronoi_demo_document;
use gpu_zone_demo::build_gpu_zone_demo_document;

use ph2d_eval_motion::MotionCookPump;
use ph2d_motion_doc::{MotionDoc, MotionHistory};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::format::ParseError;
use ph2d_nodegraph::graph::NodeId;

/// Um `pulse.signal` que disparou num tique deste quadro.
///
/// ⚠️ **Isto NÃO é um `ph2d_runtime::Signal`, e a distância é o desenho:** o grafo não conhece
/// a outbox e não chama ninguém (ADR-0075) — ele deixa um fato aqui, e quem o transforma em
/// sinal é o shell, que já é o dono da saída e já drena as outras duas fontes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MotionSignalOut {
    /// O nome autorado no text param do nó.
    pub(crate) name: String,
    /// O tique fixo do cook em que ele disparou.
    pub(crate) tick: u64,
    /// Quantas LINHAS dispararam nesse tique — o que o colapso por-quadro descartaria.
    pub(crate) rows: usize,
}

/// Runtime state for the Motion Nodes editor. One instance on `AppGfx`.
pub(crate) struct MotionState {
    /// The persistable document (the graph is the only part that cooks).
    pub(crate) doc: MotionDoc,
    /// Snapshot undo/redo of `doc`. The graph-edit intents push onto it (M1
    /// Phase 1b: connect / disconnect / add / delete / drag), and Ctrl+Z/Y drive
    /// [`MotionHistory::undo`]/[`redo`] from the shell (Phase 1b-3).
    pub(crate) history: MotionHistory,
    /// Per-frame cook driver (persistent [`Cook`] + reused instance buffer). Its
    /// [`MotionCookPump::pump`] re-cooks only on a dirty frame, so a paused frame
    /// is zero-alloc (M0.T12). The rendered slice is `pump.instances`.
    pub(crate) pump: MotionCookPump,
    /// Registered node ops (the `OpResolver` the cook resolves against).
    pub(crate) registry: NodeRegistry,
    /// Terminal nodes whose output streams are lowered to instances — every
    /// `motion.output` node in the document, in node-id order. Several sinks
    /// compose into one draw, so a document *can* hold independent scenes without
    /// a stream-merging node; the current boot document uses a single scene.
    /// Empty until a well-typed Output node exists.
    pub(crate) sinks: Vec<NodeId>,
    /// Os nós `pulse.signal` do documento — as TOMADAS que a marcha de tiques lê
    /// para saber que um pulso nomeado disparou. Recomputadas do grafo a cada
    /// quadro, ao lado de `sinks`, pelo mesmo motivo (curam-se sozinhas).
    pub(crate) signal_taps: Vec<NodeId>,
    /// O que as tomadas gritaram nos tiques deste quadro, drenado pelo shell.
    ///
    /// ⚠️ **Acumulado DENTRO do laço de tiques devidos, nunca depois dele.** O
    /// `tap_streams` do pump é limpo a cada cook, então um quadro que deve dois
    /// tiques deixa só o último — e a perda seria SILENCIOSA.
    pub(crate) signals_out: Vec<MotionSignalOut>,
    /// `atlas_uv` fallback for instances whose stream carries no `uv_rect`
    /// column (no framing node yet). Set from the composed atlas at init to a
    /// single opaque tile, so instances render as clean solid quads instead of a
    /// whole-atlas thumbnail. `[0,0,1,1]` (whole atlas) until the shell overrides.
    pub(crate) default_uv_rect: [f32; 4],
    /// `size` fallback for instances whose stream carries no `size` column.
    ///
    /// **It is the IDENTITY, and it may not be anything else** (`SIZE_IDENTITY`):
    /// every node that materializes `size` builds it from unit scale, so a
    /// fallback that disagrees makes those nodes resize the scene just by being
    /// dropped (doc 39 — it was `0.4`, so a `motion.scale` at `amount = 1` scaled
    /// every quad by 2.5×). A document that wants small quads says so, with a
    /// `motion.scale` — it does not get it from a number hidden in the shell.
    pub(crate) default_size: [f32; 2],
    /// F2 probe: the node whose output the editor is reading, and the ring of its
    /// most recent readings (oldest first). UI-only — it never touches the cook.
    pub(crate) probe: Option<NodeId>,
    pub(crate) probe_ring: Vec<f32>,
    /// F3 flow: last frame's digest of each cooked node's output, keyed by `NodeId.0`. A node
    /// whose digest MOVED has data running down its wires, and the panel marches them
    /// (TouchDesigner's animated wire). UI-only — the cook never reads it.
    pub(crate) flow_digest: std::collections::BTreeMap<u32, u64>,
    /// **The subgraph the editor is standing IN** (doc 57): `None` = the root canvas.
    /// Navigation, so it is not in the document (not serialized, not undoable) — but
    /// it is not in the PANEL either: an undo that unmakes the group you are inside
    /// has to be able to put you back on solid ground, and only the shell sees the
    /// undo (`subgraph::clamp_level`, every frame).
    pub(crate) level: Option<u32>,
    /// GPU/M5 Fase 1 (ADR-0126): the GPU-resident sequencer. Only driven when
    /// [`Self::gpu_enabled`]; persistent so its pipeline caches + buffer pool
    /// survive across frames (the GPU twin of the pump's reused buffer).
    pub(crate) gpu_cook: ph2d_gpu_cook::GpuCook,
    /// Did THIS frame's instances come from the GPU path? When `true`, present
    /// binds `gpu_cook.instances()` directly (zero readback) and
    /// `pump.instances` is stale — set/cleared by the bridge every frame.
    pub(crate) gpu_live: bool,
    /// The GPU cook path — **ON by default** since the editor learned to read a
    /// GPU-resident frame (the tap feeds the readouts/probe/digest, Fase 4);
    /// `PH2D_GPU_COOK=0` opts back out. The CPU pump remains the CANONICAL
    /// path either way (replay-hash, parity oracles — ADR-0126).
    pub(crate) gpu_enabled: bool,
    /// **This frame's GPU tap** — the same `BTreeMap<NodeId, Stream>` the graph
    /// panel's readouts read (`readout::take_tap`), stashed so the PARAMS panel
    /// reads a GPU-cooked frame through the SAME door. On a GPU frame the CPU
    /// memo (`pump.cook`) is empty, so `build_params_snapshot`'s driven-value and
    /// the `value.attribute` column picker fall back to this (48-row subsample per
    /// staged node). `None` on a CPU-driven frame (the memo holds the real thing)
    /// and one frame behind — exactly as the memo is (`readout::stamp`'s note).
    pub(crate) gpu_tap: Option<std::collections::BTreeMap<NodeId, ph2d_nodegraph::attr::Stream>>,
    /// **The graph clipboard** (Ctrl+C / Ctrl+V), `None` until the first copy. It
    /// lives HERE, not in [`MotionDoc`], because a clipboard is not document state:
    /// an undo must not empty it, entering a group must not lose it, and it can
    /// outlive the nodes it snapshotted. See [`GraphClip`].
    pub(crate) clip: Option<GraphClip>,
    /// **Add Node palette handshake.** `open_library` is the TRANSIENT trigger the `OpenLibrary` intent
    /// sets (spawn + wire context); the bridge — which owns the editor `WidgetStore` — takes it, filters
    /// the model to the compatible types and opens the full-screen palette. `library_open` then PERSISTS
    /// while the palette is up so the routed pick lands where the artist opened it AND carries the wire
    /// context ([`ph2d_panel_motion_graph::library_pick`] turns it into add / smart-connect / splice).
    /// Both are runtime-only (never serialized).
    pub(crate) open_library: Option<LibraryOpen>,
    pub(crate) library_open: Option<LibraryOpen>,
    /// doc 86 §2 (A2): the vector→tile bake cache (+ its scratch `VelloPass`). A
    /// named vector shape a `source.object` brings in is rasterized once into a
    /// tile here; the membrane publishes the result. Cached by content, filled at
    /// the fx phase (`motion_bridge::bake_objects`), read by `publish_objects`.
    pub(crate) object_bake: crate::motion_object_bake::ObjectBake,
    /// doc 86 §2 A3: named Flip objects baked to tiles (the same `BakedTile` output
    /// as `object_bake`, driven through the Flip raster + compositor).
    pub(crate) flip_object_bake: crate::motion_flip_bake::FlipObjectBake,
    /// ADR-0154: content-addressed store of `source.shape` geometry. The publish
    /// pass interns each shape's `VecPath` here (keyed by its content) and the
    /// instance carries the handle in its `geometry_id` column; the present encode
    /// looks it up to draw the shape live. Kept across frames — a static shape
    /// builds once.
    pub(crate) shape_store: crate::render_loop::motion_shape_gen::VecPathStore,
    /// As analises de audio vivas (`audio.bands`) — a FFT roda AQUI, uma vez por
    /// `(arquivo, analise)`, e nunca dentro do cook (doc 63 §6).
    pub(crate) band_cache: crate::render_loop::motion_audio_gen::BandCache,
    /// **The node-help system on/off** (ADR-0155, Enio 2026-08-04). The ONE flag the
    /// setup diagnoser rides: the auto-heal, the ⚠ inert badges and the advisories all
    /// read it (`motion_bridge_heal`), so turning it off makes the graph stop offering to
    /// fix anything — the artist's freedom, and the release valve for the `falloff`
    /// family where a missed CPU-only consumer would otherwise show a stray advisory.
    /// ON by default; a session preference (not serialized — runtime UX, like `gpu_live`).
    pub(crate) node_help_enabled: bool,
}

/// **What opened the Add-Node palette** — the spawn point plus the wire context of the gesture, so the
/// pick becomes the right edit (plain add, smart-connect, or splice). Carried on the handshake because
/// the panel that gestured and the store that holds the palette live in different structs.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LibraryOpen {
    /// Graph-space point the picked node lands at.
    pub spawn: (f32, f32),
    /// The output socket a wire was dragged FROM and dropped on empty canvas (smart-connect).
    pub connect_from: Option<(u32, u16)>,
    /// The wire (target `(to_node, to_port)`) an R-press landed ON (splice into it).
    pub splice: Option<(u32, u16)>,
    /// For smart-connect, the node types that output can feed — the palette shows only these. Empty
    /// for the unfiltered cases (whole catalog). Only read at open time (never at pick time).
    pub compatible: Vec<&'static str>,
}

/// O CLIPBOARD do grafo — o que uma cópia carrega. Irmão pelo cap de 600 LOC (HR-18),
/// pelo mesmo corte por assunto do `fixture` abaixo.
#[path = "motion_state_clip.rs"]
mod clip;
/// Re-exportado porque quem os nomeia os nomeia por `motion_state::` — mover o arquivo
/// não pode mover o caminho de quem chama (a mesma nota do `build_default_document`).
pub(crate) use clip::{ClipEdge, ClipNode, ClipSubgraph, GraphClip};

/// **Is the GPU cook path on?** — the policy, as a pure function of the env var,
/// so it can be gated without a test mutating the process environment.
///
/// ON unless explicitly switched off. See the call site for why the default
/// flipped and why flipping it is safe.
pub(crate) fn gpu_enabled_from_env(var: Option<&str>) -> bool {
    !matches!(var, Some("0"))
}

impl MotionState {
    /// Build the boot state: register every node op, e **abrir com a TELA VAZIA**.
    ///
    /// ⚠️ **O documento de boot não existe mais** (Enio, 2026-08-07: *"tire a cena da
    /// cachoeira"*). O editor abria com a neve caindo no mar — um sistema de partículas de
    /// 21 nós que o artista tinha de apagar antes de fazer qualquer coisa. Ele agora traz o
    /// que quiser pelo command-palette (`A`); as cenas de demonstração seguem acessíveis por
    /// `PH2D_GPU_COOK_DEMO`, e a neve segue viva como fixture dos gates
    /// ([`build_default_document`]). Todas as cenas anteriores (o rig da Cavalry, as de sim,
    /// as de deformer, as cadeias de value/pulse e M3/M4) já tinham saído pelo mesmo motivo e
    /// vivem no git — *um documento de boot é uma demo, não um arquivo morto*, e agora nem
    /// demo ele é. Transporte pausado no tick 0 (a ponte dá auto-play).
    pub(crate) fn new() -> Self {
        let mut registry = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut registry)
            .expect("motion node registry builds");
        let mut doc = MotionDoc::new();
        let sinks = demo_router::demo_sinks(&mut doc, &registry);
        Self {
            doc,
            history: MotionHistory::new(),
            pump: MotionCookPump::new(),
            registry,
            sinks,
            signal_taps: Vec::new(),
            signals_out: Vec::new(),
            // Whole-atlas until the shell wires a real tile (init.rs). Headless
            // callers / tests keep this default.
            default_uv_rect: [0.0, 0.0, 1.0, 1.0],
            // The SAME unit scale every node assumes when it materializes `size`.
            default_size: ph2d_nodegraph::attr::SIZE_IDENTITY,
            probe: None,
            probe_ring: Vec::new(),
            flow_digest: std::collections::BTreeMap::new(),
            level: None,
            gpu_cook: ph2d_gpu_cook::GpuCook::new(),
            gpu_live: false,
            // **ON by default** (GPU/M5, 2026-07-18). It was opt-in for one
            // reason and the reason is gone: a GPU-resident cook does not feed
            // the CPU memo, so the graph panel's readouts, postage stamps, wire
            // march and probe all went blank exactly on the documents worth
            // watching. `GpuCook::tap` now answers them for a measured +0,075 ms
            // (`bounded_readback_cost_probe`), and the panel reads the device.
            //
            // Turning it on is not a claim that every document runs there:
            // `gpu_route` still recuses a multi-sink or time-scoped document
            // whole, and `plan` recuses any chain with an uncovered node in it.
            // Those all fall through to the CPU pump exactly as before — which is
            // why the flip is safe and why it is a DEFAULT rather than a
            // requirement.
            //
            // `PH2D_GPU_COOK=0` forces the CPU pump: an escape for bisecting a
            // suspected device-path bug against the canonical path, which stays
            // the CPU's (ADR-0126 — the replay-hash never runs on a GPU).
            gpu_enabled: gpu_enabled_from_env(std::env::var("PH2D_GPU_COOK").ok().as_deref()),
            // Filled each active frame by the bridge from the GPU tap (`None` until
            // then, and on every CPU-cooked frame).
            gpu_tap: None,
            // Nothing copied yet — the first Ctrl+C fills it.
            clip: None,
            // No palette open until the `A` key asks.
            open_library: None,
            library_open: None,
            // doc 86 §2 (A2): the vector→tile bake cache, empty until the fx
            // phase bakes a named vector a `source.object` brings in.
            object_bake: crate::motion_object_bake::ObjectBake::default(),
            flip_object_bake: crate::motion_flip_bake::FlipObjectBake::default(),
            // ADR-0154: empty until the publish pass interns a `source.shape`.
            shape_store: crate::render_loop::motion_shape_gen::VecPathStore::default(),
            band_cache: crate::render_loop::motion_audio_gen::BandCache::default(),
            // ADR-0155: the node-help system is ON by default; the toolbar chip toggles it.
            node_help_enabled: true,
        }
    }

    /// **Install a saved document** (the project's Ctrl+O path) — parse the canonical text
    /// and replace the current one, runtime and all.
    pub(crate) fn load_text(&mut self, text: &str) -> Result<(), ParseError> {
        let doc = MotionDoc::from_text(text)?;
        self.install(doc);
        Ok(())
    }

    /// Adopt `doc`, **discarding every runtime trace of the one before it**.
    ///
    /// The document is the only thing a project stores; everything else here is derived. But
    /// "derived" is not the same as "harmless", because the runtime is keyed by NODE ID — and
    /// node ids are small integers that the next document reuses for entirely different nodes:
    ///
    /// - the **`Cook` is the simulation's living state**, not a cache — it holds the flakes
    ///   that are in the air. The pump is therefore replaced OUTRIGHT, not merely
    ///   `mark_dirty`'d (which invalidates the scrub cache but keeps the memo and the `pre`
    ///   feedback). A fresh pump says what it means.
    /// - the **clock** goes back to 0 — but that is now the CALLER's job, not ours (W4.T7):
    ///   Motion keeps no transport of its own, and the editor's ONE `Playhead` is not a
    ///   field we own. `App::project_load_from` rewinds it the moment a file is accepted, and
    ///   both halves of that are gated: the rewind itself in `project::tests`, and what it buys
    ///   us in `a_clock_that_was_not_rewound_opens_the_document_mid_scene`. The reason stands:
    ///   a playhead at t=40s into a graph that has never been cooked is not a resumption, it is
    ///   a lie about a simulation that never ran.
    /// - **undo** belongs to the document that was edited, not to the file that replaced it.
    /// - the **probe**, the **flow digests**, the panel's **selection** and the **level the
    ///   editor is standing in** (doc 57) all name things by id. A stale selection is the
    ///   sharpest of them: the params panel would happily edit whichever node inherited the
    ///   number. A stale LEVEL is the strangest — the new document's group `2` is not the room
    ///   you were in, and you would be looking at a canvas you never opened.
    ///
    /// `sinks` is the exception that proves the rule — the bridge recomputes it from the graph
    /// every frame, so it heals itself; it is cleared anyway so a headless caller between the
    /// load and the first pump never reads the old graph's outputs.
    fn install(&mut self, doc: MotionDoc) {
        self.doc = doc;
        self.pump = MotionCookPump::new();
        self.history = MotionHistory::new();
        self.sinks.clear();
        self.probe = None;
        self.probe_ring.clear();
        self.flow_digest.clear();
        self.level = None;
        // The GPU path re-plans against the new graph next frame; until then
        // its instance buffer describes the OLD document, so it must not draw.
        self.gpu_live = false;
        // …and last frame's tap samples the OLD graph's nodes — drop it so the
        // params panel never reads a stranger's columns for one frame.
        self.gpu_tap = None;
        // …and its SIMULATION state (last tick's `pre` columns, keyed by node id)
        // is the old document's — a new graph that reuses those ids for a `pre`
        // source would read a stranger's flakes-in-the-air. Forget it, exactly as
        // the pump above was replaced (ADR-0130 D7: a document change invalidates
        // the sim); the sim re-bakes from the seed under the new graph next frame.
        self.gpu_cook.forget_state();
        #[cfg(feature = "panel-motion-graph")]
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    }
}

/// **A fixture de nível de artista** — a neve, e a porta única que a instala.
///
/// Irmão pelo cap de 600 LOC do shell (HR-18), cortado por ASSUNTO: o que sobra aqui é o
/// que o app FAZ com um documento; o que saiu é o documento que só os gates constroem.
#[cfg(test)]
#[path = "motion_state_fixture.rs"]
mod fixture;
/// Re-exportado para os quatro módulos de teste que já o chamavam por `super::` — mover o
/// arquivo não pode mover o caminho de quem chama.
#[cfg(test)]
pub(crate) use fixture::build_default_document;

#[cfg(test)]
#[path = "motion_state_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "motion_state_gpu_tests.rs"]
mod gpu_tests;

#[cfg(test)]
#[path = "motion_state_gpu_field_tests.rs"]
mod gpu_field_tests;

#[cfg(test)]
#[path = "motion_state_gpu_neighbour_tests.rs"]
mod gpu_neighbour_tests;

#[cfg(test)]
#[path = "motion_state_gpu_pulse_demo_tests.rs"]
mod gpu_pulse_demo_tests;

#[cfg(test)]
#[path = "motion_state_gpu_voronoi_tests.rs"]
mod gpu_voronoi_tests;

#[cfg(test)]
#[path = "motion_gpu_coverage.rs"]
mod gpu_coverage;

/// A sonda do TETO DE CONTAGEM (doc 88 A1) — irmã do censo de cobertura acima: os dois varrem o
/// registry do produto e imprimem uma tabela, um sobre o que a GPU alcança e o outro sobre onde
/// uma contagem começa a custar.
#[cfg(test)]
#[path = "motion_count_ceiling_tests.rs"]
mod count_ceiling_tests;

#[cfg(test)]
#[path = "motion_delay_gate_tests.rs"]
mod delay_gate_tests;
