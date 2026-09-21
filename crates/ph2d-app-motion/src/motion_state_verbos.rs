//! ⭐ **O que o [`MotionState`] FAZ** — o bloco `impl` dele, cortado por RESPONSABILIDADE do
//! ficheiro que diz o que ele É e que cena existe.
//!
//! ⚠️ **Um `impl` inerente move-se sem mover CAMINHO nenhum**: ele é encontrado a partir do TIPO e
//! não do módulo em que está escrito, logo nem um `use` do resto da crate muda. *É a única espécie
//! de corte com raio de explosão zero, e foi por isso que foi esta a escolhida* — o tecto pedia
//! `7` linhas (`707` contra `700`) e a lei da casa manda cortar pelo excesso que a catraca IMPRIME,
//! não pelo maior bloco à vista.
//!
//! ⛔ **O tecto estava vermelho ANTES desta linha lhe tocar** (`705`), e ninguém o tinha visto: ele
//! vive em `ph2d-editor-core/tests/it/`, que um portão de fecho que só corre as crates EDITADAS
//! nunca alcança. É a mesma cegueira que o `CLAUDE.md` §5 já regista cinco vezes.

use super::*;

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
    pub fn new() -> Self {
        let mut registry = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut registry)
            .expect("motion node registry builds");
        let mut doc = MotionDoc::new();
        let sinks = demo_router::demo_sinks(&mut doc, &registry);
        Self {
            doc,
            history: MotionHistory::new(),
            pump: Self::pump_do_motion(),
            registry,
            selected_shape: FormaEscolhida::Nada,
            sinks,
            signal_taps: Vec::new(),
            signals_out: Vec::new(),
            // Whole-atlas until the shell wires a real tile (init.rs). Headless
            // callers / tests keep this default.
            card_sections: std::collections::BTreeMap::new(),
            default_uv_rect: [0.0, 0.0, 1.0, 1.0],
            // The SAME unit scale every node assumes when it materializes `size`.
            default_size: ph2d_nodegraph::attr::SIZE_IDENTITY,
            probe: None,
            probe_ring: Vec::new(),
            flow_digest: std::collections::BTreeMap::new(),
            level: None,
            gpu_cook: ph2d_gpu_cook::GpuCook::new(),
            gpu_live: false,
            route_said: None,
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
            shape_bake: crate::motion_shape_bake::ShapeBake::default(),
            flip_object_bake: crate::motion_flip_bake::FlipObjectBake::default(),
            // ADR-0154: empty until the publish pass interns a `source.shape`.
            shape_store: crate::motion_shape_gen::VecPathStore::default(),
            collider_drag: None,
            lsystem_memo: crate::motion_lsystem_gen::PlantMemo::default(),
            band_cache: crate::motion_audio_gen::BandCache::default(),
            table_cache: crate::motion_table_gen::TableCache::default(),
            // ADR-0155: the node-help system is ON by default; the toolbar chip toggles it.
            node_help_enabled: true,
            ui_now: 0.0,
            piscada: None,
        }
    }

    /// **Install a saved document** (the project's Ctrl+O path) — parse the canonical text
    /// and replace the current one, runtime and all.
    pub fn load_text(&mut self, text: &str) -> Result<(), ParseError> {
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
    /// ⭐⭐⭐ **A BOMBA DO MOTION — a única que nasce com a LEI DO DONO ligada.**
    ///
    /// A [`MotionCookPump`] nasce NEUTRA de propósito (ver o campo `so_com_forma` dela): o emissor de
    /// partículas constrói uma bomba própria, e uma partícula é uma posição pura — com a lei ligada
    /// por omissão ela era cortada e **o emissor desaparecia do produto**, medido em 11 gates
    /// vermelhos que só a varredura impactada viu.
    ///
    /// ⇒ *a lei é do módulo que o artista edita*, e é aqui que ela entra. ⚠️ **Os DOIS sítios que
    /// constroem a bomba do Motion passam por esta porta** — o arranque e o `install` de um ficheiro
    /// —, senão carregar um documento repunha o neutro.
    fn pump_do_motion() -> MotionCookPump {
        let mut p = MotionCookPump::new();
        p.define_a_lei(ph2d_eval_motion::so_com_forma_por_ordem());
        p
    }

    fn install(&mut self, doc: MotionDoc) {
        self.doc = doc;
        // ⚠️⚠️ **A LEI VIAJA, o estado da simulação não.** A bomba é substituída de propósito
        // (ela guarda os flocos que estão no ar), mas a lei do dono é CONFIGURAÇÃO — carregar um
        // ficheiro não pode ligá-la nem desligá-la. Sem esta linha, um `load` repõe o valor de
        // omissão e um arnês que a tivesse desligado passa a medir outro programa a meio.
        let lei = self.pump.a_lei();
        self.pump = Self::pump_do_motion();
        self.pump.define_a_lei(lei);
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
