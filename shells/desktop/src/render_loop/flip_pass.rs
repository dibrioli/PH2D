//! ADR-0113 W1 T1.4/T1.7 — o passe que põe o Flip na tela, **compondo camada a
//! camada** pelo compositor 22-modos do Painter.
//!
//! Roda no present phase, logo APÓS o passe de sprites e ANTES do tonemap: o
//! artwork do Flip compõe no mesmo `game_rt` (HDR) com a MESMA câmera e é
//! tonemapeado junto. Cada camada visível (com desenho ativo no playhead) é
//! rasterizada isolada, resolvida pra uma fatia straight sRGB8 e **injetada**
//! (GPU→GPU) no `LayerCompositor`; o compositor aplica blend/opacity por-camada
//! (idêntico ao Painter) e a saída é blitada de volta no `game_rt`.
//!
//! **Por que passar pelo compositor 8-bit sRGB:** o artwork do Flip é linha SDR
//! (`Rgba` ∈ [0,1]) — o round-trip 8-bit é imperceptível — e o ganho é a
//! consistência dura: uma camada Multiply do Flip compõe EXATAMENTE como uma
//! camada Multiply do Painter (mesma matemática de blend, sem reimplementação).

use ph2d_core::Playhead;
use ph2d_flip::FlipDoc;
use ph2d_flip_render::{CameraRaw, FlipCompose, FlipGpuData, FlipRenderer, pack_drawing};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::layer_compositor::{
    LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_render::{Camera2d, GameRt};

/// A sessão de composição por-camada do Flip: o `LayerCompositor` do Painter +
/// o buffer dummy. As fatias reais entram por `inject_slice_from_texture`
/// (GPU→GPU); o dummy só existe para satisfazer o filtro de tamanho do
/// `ensure_slice` (`rgba8.len() == w·h·4`) — como a versão injetada bate com a
/// que o provider reporta, o dummy **nunca é subido** pra GPU.
pub(crate) struct FlipComposite {
    compositor: LayerCompositor,
    dummy: Vec<u8>,
}

impl FlipComposite {
    fn new(gpu: &GpuContext) -> Self {
        Self {
            compositor: LayerCompositor::new(gpu),
            dummy: Vec::new(),
        }
    }

    /// Garante o dummy com `w·h·4` bytes (zerado; redimensiona no resize).
    fn ensure_dummy(&mut self, w: u32, h: u32) {
        let need = (w as usize) * (h as usize) * 4;
        if self.dummy.len() != need {
            self.dummy.resize(need, 0);
        }
    }
}

/// Provider que devolve o dummy para QUALQUER chave, na versão `0` — a mesma que
/// o `inject` grava, então `ensure_slice` acha "limpo" e não sobe o dummy.
struct DummyProvider<'a> {
    pixels: &'a [u8],
}

impl LayerPixelProvider for DummyProvider<'_> {
    fn layer_pixels(&self, _key: u64) -> Option<LayerPixels<'_>> {
        Some(LayerPixels {
            version: 0,
            rgba8: self.pixels,
        })
    }
}

/// Uma camada pronta pra compor: a chave estável, blend/opacity e o desenho já
/// empacotado (a fatia é rasterizada no `stage_layer`).
struct StagedLayer {
    key: u64,
    blend: u8,
    opacity: f32,
    data: FlipGpuData,
}

/// Compõe o Flip amostrado em `playhead` no `game_rt`. No-op se não há camada
/// ativa (cena vazia = o default sem `PH2D_FLIP_DEMO`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    flip: &FlipDoc,
    flip_render: &mut FlipRenderer,
    flip_compose: &mut FlipCompose,
    flip_composite: &mut Option<FlipComposite>,
    playhead: &Playhead,
    game_rt: &GameRt,
    camera: &Camera2d,
    window: WindowSize,
    gpu: &GpuContext,
) {
    let layers = collect_layers(flip, playhead);
    if layers.is_empty() {
        return;
    }
    let (w, h) = (window.width.max(1), window.height.max(1));
    let cam = camera_raw(camera, window);

    // A op-list (bottom-to-top = ordem das camadas), construída antes do loop
    // porque o `inject`/`composite` dimensionam o array de fatias por ela.
    let ops: Vec<LayerOp> = layers
        .iter()
        .map(|l| LayerOp::Layer {
            key: l.key,
            blend_mode: l.blend,
            opacity: l.opacity,
        })
        .collect();

    let comp = flip_composite.get_or_insert_with(|| FlipComposite::new(gpu));
    comp.ensure_dummy(w, h);

    // Rasteriza + injeta cada camada. O straight scratch é reusado: a cópia do
    // `inject` (submissão k) roda antes do próximo `stage_layer` sobrescrevê-lo
    // (submissão k+1) — garantido pela ordem da fila.
    for l in &layers {
        let slice = flip_compose.stage_layer(&gpu.device, &gpu.queue, flip_render, &cam, &l.data, (w, h));
        if let Err(e) =
            comp.compositor
                .inject_slice_from_texture(gpu, &ops, l.key, slice, w, h, (0, 0, w, h), 0)
        {
            // Falha aqui deixaria a composição incompleta; aborta o passe (o
            // frame fica sem o Flip, sem corromper o game_rt).
            eprintln!("[ph2d-flip] inject falhou: {e}");
            return;
        }
    }

    // Compõe (blend/opacity por-camada) e blita a saída no game_rt.
    {
        let FlipComposite { compositor, dummy } = comp;
        let provider = DummyProvider { pixels: dummy };
        if let Err(e) = compositor.composite(gpu, &ops, &provider, w, h, Region::full(w, h)) {
            eprintln!("[ph2d-flip] composite falhou: {e}");
            return;
        }
        if let Some(out) = compositor.output_texture() {
            flip_compose.blit(&gpu.device, &gpu.queue, out, game_rt.view());
        }
    }
}

/// As camadas ativas AGORA, na ordem de composição (objeto por objeto; dentro de
/// cada objeto, de baixo p/ cima = ordem do slice; só visíveis, com desenho não
/// vazio). Cada objeto amostra pelo SEU FPS.
fn collect_layers(flip: &FlipDoc, playhead: &Playhead) -> Vec<StagedLayer> {
    let mut out = Vec::new();
    for obj in flip.objects() {
        let frame = obj.frame_at(playhead);
        for layer in obj.layers() {
            if !layer.visible {
                continue;
            }
            let Some(did) = layer.drawing_at(frame) else {
                continue;
            };
            let Some(drawing) = obj.drawing(did) else {
                continue;
            };
            let data = pack_drawing(drawing);
            if data.is_empty() {
                continue; // camada transparente não contribui em nenhum modo
            }
            out.push(StagedLayer {
                key: layer_key(obj.id.0, layer.id.0),
                blend: layer.blend.to_u8(),
                opacity: layer.opacity,
                data,
            });
        }
    }
    out
}

/// Chave estável do compositor por (objeto, camada) — determinística e distinta
/// entre objetos (evita colisão quando dois objetos têm `LayerId` iguais). O
/// compositor cacheia fatias por chave, então a estabilidade frame-a-frame dá
/// reuso; a mistura é transcendental-free (HR-5).
fn layer_key(object_id: u64, layer_id: u32) -> u64 {
    object_id
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (layer_id as u64)
}

/// Converte a `Camera2d` (mundo→clip ortográfico) no uniform do passe. O
/// `view_proj` é o MESMO afim que os sprites usam; `px_per_world` = altura da
/// janela / altura de mundo (pixels isotrópicos), o zoom que vira a espessura.
fn camera_raw(camera: &Camera2d, window: WindowSize) -> CameraRaw {
    let vp = camera.view_proj(window).to_cols_array_2d();
    let px_per_world = window.height.max(1) as f32 / camera.height_world.max(f32::EPSILON);
    CameraRaw::new(
        vp,
        [window.width as f32, window.height as f32],
        px_per_world,
    )
}
