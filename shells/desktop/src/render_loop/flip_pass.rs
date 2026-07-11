//! ADR-0113 W1 T1.4/T1.7/T1.8 — o passe que põe o Flip na tela, **compondo
//! camada a camada** pelo compositor 22-modos do Painter, com a tesselação
//! **cacheada por desenho** (troca de quadro barata).
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
//!
//! **T1.8 — troca de quadro barata:** a geometria empacotada (`FlipGpuData`) é
//! camera-INDEPENDENTE (mundo; a câmera entra só no shader), então é cacheada
//! por (objeto, desenho) e validada por hash de conteúdo. Num *hold* (o mesmo
//! desenho segurando N quadros) ou num pan/zoom, a tesselação (ear-clipping +
//! layout) NÃO re-roda — só o render GPU com a câmera nova. `PH2D_FLIP_STATS=1`
//! loga packs vs hits por frame.

use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipDrawing};
use ph2d_flip_render::{CameraRaw, FlipCompose, FlipGpuData, FlipRenderer, pack_drawing};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::layer_compositor::{
    LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_render::{Camera2d, GameRt};
use std::collections::HashMap;

/// A sessão de composição por-camada do Flip: o `LayerCompositor` do Painter, o
/// buffer dummy e o cache de tesselação. As fatias reais entram por
/// `inject_slice_from_texture` (GPU→GPU); o dummy só existe para satisfazer o
/// filtro de tamanho do `ensure_slice` (`rgba8.len() == w·h·4`) — como a versão
/// injetada bate com a que o provider reporta, o dummy **nunca é subido**.
pub(crate) struct FlipComposite {
    compositor: LayerCompositor,
    dummy: Vec<u8>,
    /// Geometria empacotada por (objeto, desenho), camera-INDEPENDENTE — sobrevive
    /// a pan/zoom e a *holds*. Validada por hash de conteúdo.
    tess: TessCache,
}

impl FlipComposite {
    fn new(gpu: &GpuContext) -> Self {
        Self {
            compositor: LayerCompositor::new(gpu),
            dummy: Vec::new(),
            tess: TessCache::default(),
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

/// Cache de tesselação por (objeto, desenho) + instrumentação (por frame). Puro
/// CPU (sem GPU) — testável headless. A troca de quadro barata de T1.8 é isto:
/// num *hold*/pan/zoom, `ensure` só recomputa o hash (barato) e reusa a
/// geometria; a tesselação (ear-clipping + layout) NÃO re-roda.
#[derive(Default)]
struct TessCache {
    map: HashMap<(u64, u32), CachedTess>,
    packs: u32,
    hits: u32,
}

struct CachedTess {
    hash: u64,
    data: FlipGpuData,
}

impl TessCache {
    fn reset_stats(&mut self) {
        self.packs = 0;
        self.hits = 0;
    }

    /// Garante que `key` está tesselado e VÁLIDO para o `drawing` atual:
    /// re-empacota só se ausente ou se o conteúdo mudou (hash diverge — pega
    /// edição do W2 e reuso posicional de `DrawingId` após compactação).
    fn ensure(&mut self, key: (u64, u32), drawing: &FlipDrawing) {
        let hash = drawing_hash(drawing);
        match self.map.get(&key) {
            Some(c) if c.hash == hash => self.hits += 1,
            _ => {
                self.packs += 1;
                self.map.insert(
                    key,
                    CachedTess {
                        hash,
                        data: pack_drawing(drawing),
                    },
                );
            }
        }
    }

    fn get(&self, key: &(u64, u32)) -> Option<&FlipGpuData> {
        self.map.get(key).map(|c| &c.data)
    }

    fn log(&self) {
        if std::env::var_os("PH2D_FLIP_STATS").is_some() {
            eprintln!(
                "[ph2d-flip] tess: {} pack(s), {} hit(s) neste frame",
                self.packs, self.hits
            );
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

/// Uma camada pronta pra compor: a chave estável do compositor, blend/opacity, a
/// chave do cache de tesselação e o desenho-fonte (empacotado sob demanda).
struct LayerRef<'a> {
    key: u64,
    blend: u8,
    opacity: f32,
    cache_key: (u64, u32),
    drawing: &'a FlipDrawing,
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
    comp.tess.reset_stats();

    // Passe 1: garante a tesselação de cada camada (cache-hit num hold/pan/zoom).
    for l in &layers {
        comp.tess.ensure(l.cache_key, l.drawing);
    }

    // Passe 2: rasteriza + injeta cada camada. O straight scratch é reusado: a
    // cópia do `inject` (submissão k) roda antes do próximo `stage_layer`
    // sobrescrevê-lo (submissão k+1) — garantido pela ordem da fila.
    {
        let FlipComposite {
            compositor, tess, ..
        } = comp;
        for l in &layers {
            let data = tess.get(&l.cache_key).expect("garantido no passe 1");
            let slice =
                flip_compose.stage_layer(&gpu.device, &gpu.queue, flip_render, &cam, data, (w, h));
            if let Err(e) =
                compositor.inject_slice_from_texture(gpu, &ops, l.key, slice, w, h, (0, 0, w, h), 0)
            {
                eprintln!("[ph2d-flip] inject falhou: {e}");
                return;
            }
        }
    }

    // Passe 3: compõe (blend/opacity por-camada) e blita a saída no game_rt.
    {
        let FlipComposite {
            compositor, dummy, ..
        } = comp;
        let provider = DummyProvider { pixels: dummy };
        if let Err(e) = compositor.composite(gpu, &ops, &provider, w, h, Region::full(w, h)) {
            eprintln!("[ph2d-flip] composite falhou: {e}");
            return;
        }
        if let Some(out) = compositor.output_texture() {
            flip_compose.blit(&gpu.device, &gpu.queue, out, game_rt.view());
        }
    }
    comp.tess.log();
}

/// As camadas ativas AGORA, na ordem de composição (objeto por objeto; dentro de
/// cada objeto, de baixo p/ cima = ordem do slice; só visíveis, com desenho não
/// vazio). Cada objeto amostra pelo SEU FPS. NÃO empacota — isso é sob demanda
/// no cache (`ensure_tess`), pra troca de quadro barata.
fn collect_layers<'a>(flip: &'a FlipDoc, playhead: &Playhead) -> Vec<LayerRef<'a>> {
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
            if drawing.strokes.is_empty() {
                continue; // camada transparente não contribui em nenhum modo
            }
            out.push(LayerRef {
                key: layer_key(obj.id.0, layer.id.0),
                blend: layer.blend.to_u8(),
                opacity: layer.opacity,
                cache_key: (obj.id.0, did.0),
                drawing,
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
    object_id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (layer_id as u64)
}

/// FNV-1a (transcendental-free, HR-5) de um passo.
#[inline]
fn fnv(h: u64, x: u32) -> u64 {
    (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3)
}

/// Hash de conteúdo de um desenho — cobre TUDO que `pack_drawing` lê (posições,
/// larguras, opacidades, cores, fechamento, cap, hardness, material, fill). Se
/// o desenho muda (edição do W2) ou um `DrawingId` é reusado após compactação, o
/// hash diverge e o cache re-empacota. Muito mais barato que a tesselação.
fn drawing_hash(d: &FlipDrawing) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for s in &d.strokes {
        for p in s.positions() {
            h = fnv(h, p.x.to_bits());
            h = fnv(h, p.y.to_bits());
        }
        for w in s.widths() {
            h = fnv(h, w.to_bits());
        }
        for o in s.opacities() {
            h = fnv(h, o.to_bits());
        }
        for c in s.colors() {
            for k in c.0 {
                h = fnv(h, k.to_bits());
            }
        }
        h = fnv(h, u32::from(s.closed));
        h = fnv(h, s.cap.0 as u32);
        h = fnv(h, s.cap.1 as u32);
        h = fnv(h, s.hardness.to_bits());
        h = fnv(h, s.material.0);
        match &s.fill {
            Some(f) => {
                h = fnv(h, 1);
                for k in f.color.0 {
                    h = fnv(h, k.to_bits());
                }
                h = fnv(h, f.opacity.to_bits());
            }
            None => h = fnv(h, 0),
        }
        // separa strokes adjacentes (evita colisão por concatenação).
        h = fnv(h, 0xFFFF_FFFF);
    }
    fnv(h, d.strokes.len() as u32)
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

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_flip::{FlipStroke, Point, Rgba};

    fn line(a: Vec2, b: Vec2) -> FlipDrawing {
        let mut s = FlipStroke::new();
        for p in [a, b] {
            s.push_point(Point {
                pos: p,
                width: 0.1,
                opacity: 1.0,
                color: Rgba::new(1.0, 0.0, 0.0, 1.0),
            });
        }
        let mut d = FlipDrawing::new();
        d.strokes.push(s);
        d
    }

    /// T1.8: um *hold* (o MESMO desenho segurando N quadros) NÃO re-tessela — a
    /// segunda amostra do mesmo `(key, drawing)` é cache-hit (`packs` não sobe).
    #[test]
    fn hold_reuses_tessellation_zero_repacks() {
        let mut c = TessCache::default();
        let d = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let key = (7, 3);

        c.reset_stats();
        c.ensure(key, &d); // 1º quadro do hold: miss → pack
        assert_eq!((c.packs, c.hits), (1, 0));

        // Quadros seguintes do MESMO hold: cache-hit, zero re-tesselação.
        for _ in 0..12 {
            c.reset_stats();
            c.ensure(key, &d);
            assert_eq!((c.packs, c.hits), (0, 1), "hold não pode re-tesselar");
        }
        assert!(c.get(&key).is_some(), "a geometria fica residente");
    }

    /// Trocar para um desenho de CONTEÚDO diferente (ainda que sob a mesma
    /// `cache_key`, ex.: reuso posicional de `DrawingId` / edição do W2)
    /// re-tessela — o hash diverge.
    #[test]
    fn content_change_forces_repack() {
        let mut c = TessCache::default();
        let key = (1, 0);
        let a = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));
        let b = line(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0)); // ponto diferente

        c.ensure(key, &a);
        let after_a = c.packs;
        c.ensure(key, &b); // mesmo slot, conteúdo diferente → re-pack
        assert_eq!(c.packs, after_a + 1, "conteúdo mudou deve re-tesselar");
    }

    /// O hash é estável para conteúdo idêntico e sensível a uma mudança mínima.
    #[test]
    fn hash_is_stable_and_sensitive() {
        let a = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let a2 = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let moved = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0001));
        assert_eq!(drawing_hash(&a), drawing_hash(&a2), "idêntico → mesmo hash");
        assert_ne!(
            drawing_hash(&a),
            drawing_hash(&moved),
            "mudança mínima → hash diferente"
        );
    }
}
