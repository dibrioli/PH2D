//! ⭐ GEF 19 E4 — **o Flip na ordem da Hierarchy**: o que o passe do Flip lê num quadro, montado uma
//! vez por passe do presente e desenhado por FAIXA da ordem total (`ph2d_render::draw_bands`). A
//! faixa logo acima da última de sprites vai no `game_rt` do pipeline inteiro; as outras, cada uma
//! no `game_rt` limpo, com tonemap e colagem próprios (`present_bands` do shell).
//!
//! Os fantasmas e o *peek* são chrome de AUTORIA: só com a tool Flip, nunca na vista da câmera, e
//! o peek nunca no play.

use crate::state::FlipState;
use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipObjectId};
use ph2d_flip_render::{FlipCompose, FlipRenderer};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, GameRt};
use ph2d_vec_scene::Xform;

/// As peças de um passe do Flip neste quadro.
pub struct Quadro<'a> {
    pub flip: &'a FlipDoc,
    pub flip_render: &'a mut FlipRenderer,
    pub flip_compose: &'a mut FlipCompose,
    pub flip_composite: &'a mut Option<crate::pass::FlipComposite>,
    pub state: &'a FlipState,
    pub playhead: &'a Playhead,
    /// O afim de cada objecto (`ph2d_flip_entities::transform::build`).
    pub models: Vec<(FlipObjectId, Xform)>,
    /// A vista da câmera (A6): mostra o jogo — sem fantasmas, sem peek, sem traço em curso.
    pub na_vista: bool,
    pub camera: &'a Camera2d,
    pub window: WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    pub gpu: &'a GpuContext,
}

impl<'a> Quadro<'a> {
    /// Monta o passe deste quadro; o afim de cada objecto lê-se do ECS aqui (ADR-0111).
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        flip: &'a FlipDoc,
        (flip_render, flip_compose, flip_composite): (
            &'a mut FlipRenderer,
            &'a mut FlipCompose,
            &'a mut Option<crate::pass::FlipComposite>,
        ),
        state: &'a FlipState,
        playhead: &'a Playhead,
        sim: &ph2d_ecs::SimWorld,
        na_vista: bool,
        camera: &'a Camera2d,
        (window, scene_viewport): (WindowSize, Option<[f32; 4]>),
        gpu: &'a GpuContext,
    ) -> Self {
        Self {
            flip,
            flip_render,
            flip_compose,
            flip_composite,
            state,
            playhead,
            models: ph2d_flip_entities::transform::build(sim, &state.entities),
            na_vista,
            camera,
            window,
            scene_viewport,
            gpu,
        }
    }

    /// Desenha os objectos `ids` (documento; de trás para a frente) por cima do que o `game_rt` tem.
    pub fn desenha(&mut self, ids: &[FlipObjectId], game_rt: &GameRt) {
        if ids.is_empty() {
            return;
        }
        let st = self.state;
        let autoria = st.active && !self.na_vista;
        let peek = st.peek.filter(|_| autoria && !self.playhead.is_playing());
        let ghosts = (autoria && peek.is_none()).then(|| crate::pass_ghosts::GhostSources {
            selected: st.strip.selected_keys(),
            pinned: st.strip.pinned_keys(),
            trace: Some(&st.strip.trace),
        });
        let preview = st.preview_do_quadro.as_ref().filter(|_| !self.na_vista);
        let (t, span) = (
            std::time::Instant::now(),
            ph2d_gpu::pass_profiler::span_begin(&self.gpu.device, &self.gpu.queue, "flip.faixa"),
        );
        crate::pass::render(
            self.flip,
            self.flip_render,
            self.flip_compose,
            self.flip_composite,
            preview,
            st.target,
            &self.models,
            Some(ids),
            self.playhead,
            ghosts,
            peek,
            game_rt,
            self.camera,
            self.window,
            self.scene_viewport,
            self.gpu,
        );
        if let Some(span) = span {
            ph2d_gpu::pass_profiler::span_end(&self.gpu.device, &self.gpu.queue, span);
        }
        let ns = u64::try_from(t.elapsed().as_nanos()).unwrap_or(u64::MAX);
        ph2d_gpu::sonda::nota_cpu("flip.faixa", ns);
    }

    /// Desenha a faixa Flip `band` da ordem total por cima do que o `game_rt` tem (GEF 19 E4).
    pub fn desenha_a_faixa(
        &mut self,
        ordem: &ph2d_render::draw_bands::FrameOrder,
        band: ph2d_render::draw_bands::Band,
        game_rt: &GameRt,
    ) {
        self.desenha(&Self::da_faixa(&ordem.flip_ids_in(band)), game_rt);
    }

    /// Os objectos de uma faixa: os ids que a ordem total registou (`FrameOrder::flip_ids_in`).
    #[must_use]
    pub fn da_faixa(ids: &[u64]) -> Vec<FlipObjectId> {
        ids.iter().map(|&i| FlipObjectId(i)).collect()
    }

    /// Os objectos sem entidade na Hierarchy — fora da ordem total, desenhados no passe principal,
    /// por cima (o invariante «um objecto ⟺ uma entidade» do `flip_entities::sync` diz que não há).
    #[must_use]
    pub fn sem_entidade(&self) -> Vec<FlipObjectId> {
        self.flip
            .objects()
            .iter()
            .map(|o| o.id)
            .filter(|id| !self.state.entities.contains_key(id))
            .collect()
    }
}
