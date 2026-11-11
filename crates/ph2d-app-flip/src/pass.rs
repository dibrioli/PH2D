//! ADR-0114 W1 T1.4/T1.7/T1.8 — o passe que põe o Flip na tela, **compondo
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

use crate::pass_cache::TessCache;
use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipDrawing, FlipObjectId, FlipTarget, LayerId};
use ph2d_flip_entities::transform::art_to_world;
use ph2d_flip_render::engine::new_engine_armed;
use ph2d_flip_render::{CameraRaw, FlipCompose, FlipGpuData, FlipRenderer};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::layer_compositor::{
    LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_render::{Camera2d, GameRt};
use ph2d_vec_scene::Xform;

/// A sessão de composição por-camada do Flip: o `LayerCompositor` do Painter, o
/// buffer dummy e o cache de tesselação. As fatias reais entram por
/// `inject_slice_from_texture` (GPU→GPU); o dummy só existe para satisfazer o
/// filtro de tamanho do `ensure_slice` (`rgba8.len() == w·h·4`) — como a versão
/// injetada bate com a que o provider reporta, o dummy **nunca é subido**.
pub struct FlipComposite {
    compositor: LayerCompositor,
    dummy: Vec<u8>,
    /// Geometria empacotada por (objeto, desenho), camera-INDEPENDENTE — sobrevive
    /// a pan/zoom e a *holds*. Validada por hash de conteúdo.
    tess: TessCache,
    /// **A frescura do Pass A** (doc 12 §22.3): o que já foi rasterizado em cada fatia. É o que faz
    /// arte commitada e fantasma de onion custarem **zero** enquanto ninguém mexe neles.
    stage: crate::pass_stage::StageMemo,
    /// ⭐ W5e: o rasterizador da camada da LUZ 2D ([`luz`]) — próprio, para a profundidade do tamanho
    /// da grade não disputar a do ecrã.
    luz: Option<FlipRenderer>,
}

/// As camadas activas AGORA (ficheiro irmão, tecto de LOC).
#[path = "pass_collect.rs"]
mod collect;
use collect::collect_layers;

/// ⭐ W5e — o traço do Flip na luz 2D (ficheiro irmão).
#[path = "pass_luz.rs"]
pub mod luz;

impl FlipComposite {
    fn new(gpu: &GpuContext) -> Self {
        Self {
            compositor: ph2d_flip_render::compositor_do_flip(gpu),
            dummy: Vec::new(),
            tess: TessCache::default(),
            stage: crate::pass_stage::StageMemo::default(),
            luz: None,
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
            dirty: None, // flip composites whole layers, never incrementally
            dirty_parts: &[],
        })
    }
}

/// Uma camada pronta pra compor: a chave estável do compositor, blend/opacity, a
/// chave do cache de tesselação e o desenho-fonte (empacotado sob demanda).
///
/// `drawing` é `None` numa camada que carrega SÓ o preview ao vivo (1º traço numa
/// camada ainda vazia); `preview`, quando presente, é o traço em curso **dobrado
/// nesta fatia** — compõe pelo blend/opacity DESTA camada em tempo real (idêntico
/// ao bake). Só a camada-alvo do preview recebe `preview: Some`.
struct LayerRef<'a> {
    key: u64,
    /// O objeto Flip (`FlipObjectId.0`) — a luz recebida é dele (W5j, `NaoRecebeLuz`).
    obj: u64,
    blend: u8,
    opacity: f32,
    cache_key: (u64, u32),
    drawing: Option<&'a FlipDrawing>,
    preview: Option<&'a FlipGpuData>,
    /// O afim LOCAL→mundo do objeto desta camada (ADR-0111): o gizmo move/gira/
    /// escala via `Transform`, e o render o dobra no `world_to_clip`. Identidade
    /// para um objeto não-transformado (caminho comum) — sem custo.
    model: Xform,
    /// **Esta fatia é um Ghost Frame** (`(rgb do tint, alpha)`): a arte sai como
    /// silhueta recolorida e esmaecida. `None` = a arte real do quadro.
    ///
    /// O fantasma é uma FATIA DA PILHA, não um passe por baixo de tudo: ele entra
    /// logo ABAIXO da própria camada, então a arte das camadas de baixo NÃO o cobre
    /// (era o bug do 1º corte — o fundo opaco comia o fantasma da camada de cima) e
    /// a arte do quadro atual, essa sim, cai por cima dele.
    ghost: Option<([f32; 3], f32)>,
    /// **Profundidade multiplano** (2.5D, ADR-0114 §Decisão 3) — a fração com que
    /// esta camada acompanha a câmera (`1.0` = flat, o comum). `< 1.0` desloca a
    /// translação do `model` por `parallax_model` para dar paralaxe ao panhar.
    depth: f32,
}

/// Compõe o Flip amostrado em `playhead` no `game_rt`, mais o **preview ao vivo**
/// do traço em curso (`preview`, se houver), **dobrado na fatia da camada ativa**
/// do desenho em edição (`target`) para compor pelo blend/opacity dela em tempo real. No-op se
/// não há camada ativa NEM preview (cena vazia = o default sem `PH2D_FLIP_DEMO`).
#[allow(clippy::too_many_arguments)]
pub fn render(
    flip: &FlipDoc,
    flip_render: &mut FlipRenderer,
    flip_compose: &mut FlipCompose,
    flip_composite: &mut Option<FlipComposite>,
    preview: Option<&FlipGpuData>,
    target: FlipTarget,
    models: &[(FlipObjectId, Xform)],
    // ⭐ GEF 19 E4: `Some(ids)` = só estes objectos, nesta ordem (de trás para a frente) — uma faixa da
    // ordem total; `None` = todos, na ordem do documento.
    objects: Option<&[FlipObjectId]>,
    playhead: &Playhead,
    // Ghost Frames: `Some(chaves selecionadas)` = a tool Flip está ativa (fantasmas
    // ligados, sujeitos aos gates do objeto/camada e ao "some no play"); `None` =
    // outra tool no comando — a cena Flip aparece limpa, sem fantasma.
    ghosts: Option<crate::pass_ghosts::GhostSources<'_>>,
    // O PEEK (Shift & Trace fatia 2): `Some` = uma folha vizinha na mão (a camada
    // ATIVA amostra o desenho anterior/atual/seguinte). O shell passa `ghosts: None`
    // junto — o flip é uma folha na mão, não uma pilha translúcida.
    peek: Option<crate::peek::PeekDir>,
    game_rt: &GameRt,
    camera: &Camera2d,
    window: WindowSize,
    // O sub-retângulo da CENA (`CenterSplit::scene_viewport`), o MESMO que o passe de sprites
    // recebe em `present.rs` — sem ele este passe projetava a janela cheia e a arte escorregava
    // sob o pan (ver [`camera_scene`]). `None` fora do split = byte-idêntico.
    scene_viewport: Option<[f32; 4]>,
    gpu: &GpuContext,
) {
    // O preview é atribuído à camada ativa (dobrado na fatia dela). `unfolded` só
    // sobra quando a camada-alvo é invisível/irresolvível — aí cai no overlay
    // Normal (o usuário nunca desenha às cegas). Os fantasmas entram na MESMA lista,
    // cada um como uma fatia logo abaixo da sua camada.
    let (layers, unfolded) = so_estes(
        collect_layers(flip, playhead, preview, target, models, ghosts, peek),
        flip,
        target,
        objects,
    );
    if layers.is_empty() && unfolded.is_none() {
        return;
    }
    // ⚠️ As dims do ALVO (os intermédios e o blit final são de alvo inteiro); a CÂMERA é que
    // passa pelo sub-retângulo da cena.
    let (w, h) = (window.width.max(1), window.height.max(1));
    let cam = camera_scene(camera, window, scene_viewport);

    // O MOTOR do traço ([doc 12](../../../../docs/Flip/12_novo_motor_pesquisa.md)) — o percurso é o
    // DEFAULT, e `PH2D_FLIP_NEW_ENGINE=0` é a escape. **O shell é o único interruptor**: a crate não
    // lê o ambiente, senão a escolha moraria em dois lugares e o gate de um seria verde sobre o
    // outro.
    //
    // ⚠️ Lido UMA vez (`OnceLock`) — um `var()` por frame é syscall por frame, e pior: um
    // interruptor que muda no meio de uma sessão faria o A/B do smoke depender de *quando* o
    // artista olhou.
    flip_compose.set_walk_engine(&gpu.device, new_engine_armed());

    if !layers.is_empty() {
        composite_layers(
            &layers,
            flip_render,
            flip_compose,
            flip_composite,
            game_rt,
            &cam,
            camera.center,
            (w, h),
            gpu,
        );
    }

    // Fallback: preview não-atribuído (camada-alvo oculta/inexistente) — rasteriza
    // DIRETO por cima do composite (premult-over = Normal), transitório. Reusa o
    // `flip_render` (os buffers dele já foram consumidos pela composição acima).
    if let Some(pv) = unfolded {
        draw_overlay(flip_render, pv, &cam, game_rt, (w, h), gpu);
    }
}

/// `Some(ids)`: fica só com as camadas dos objectos `ids`, na ordem deles (estável dentro de cada
/// objecto), e o preview sem dobra só se o alvo for um deles. `None`: tudo como veio.
fn so_estes<'a>(
    (mut layers, unfolded): (Vec<LayerRef<'a>>, Option<&'a FlipGpuData>),
    flip: &FlipDoc,
    target: FlipTarget,
    objects: Option<&[FlipObjectId]>,
) -> (Vec<LayerRef<'a>>, Option<&'a FlipGpuData>) {
    let Some(ids) = objects else {
        return (layers, unfolded);
    };
    let pos = |o: u64| ids.iter().position(|i| i.0 == o);
    layers.retain(|l| pos(l.obj).is_some());
    layers.sort_by_key(|l| pos(l.obj));
    let alvo = target.resolve(flip).map(|(o, _)| o);
    (
        layers,
        unfolded.filter(|_| alvo.is_some_and(|o| ids.contains(&o))),
    )
}

/// Compõe as camadas ativas (blend/opacity por-camada) e blita no `game_rt` — em trechos com a luz de
/// cada um (W5j).
#[allow(clippy::too_many_arguments)]
fn composite_layers(
    layers: &[LayerRef<'_>],
    flip_render: &mut FlipRenderer,
    flip_compose: &mut FlipCompose,
    flip_composite: &mut Option<FlipComposite>,
    game_rt: &GameRt,
    cam: &CameraRaw,
    cam_center: [f32; 2],
    (w, h): (u32, u32),
    gpu: &GpuContext,
) {
    let comp = flip_composite.get_or_insert_with(|| FlipComposite::new(gpu));
    if let Some(ops) = stage_slices(
        layers,
        flip_render,
        flip_compose,
        comp,
        cam,
        cam_center,
        (w, h),
        gpu,
    ) {
        composite_lit(layers, &ops, flip_compose, comp, game_rt, (w, h), gpu);
    }
    comp.tess.log();
    if std::env::var_os("PH2D_FLIP_STATS").is_some() {
        let (staged, skipped) = comp.stage.stats();
        eprintln!("[ph2d-flip] pass A: {staged} rasterizada(s), {skipped} pulada(s) neste frame");
    }
}

/// Os passes 1–2: a tesselação e a fatia de cada camada no compositor. Devolve a op-list (bottom-to-top),
/// ou `None` se uma injecção falhou (dito no terminal).
#[allow(clippy::too_many_arguments)]
fn stage_slices(
    layers: &[LayerRef<'_>],
    flip_render: &mut FlipRenderer,
    flip_compose: &mut FlipCompose,
    comp: &mut FlipComposite,
    cam: &CameraRaw,
    // O centro (pan) da câmera-viewport em MUNDO — a paralaxe multiplano lerpa a
    // translação de cada camada entre ele e a origem do objeto por `depth`.
    cam_center: [f32; 2],
    (w, h): (u32, u32),
    gpu: &GpuContext,
) -> Option<Vec<LayerOp>> {
    // A op-list (bottom-to-top = ordem das camadas), construída antes do loop
    // porque o `inject`/`composite` dimensionam o array de fatias por ela.
    let ops: Vec<LayerOp> = layers
        .iter()
        .map(|l| LayerOp::Layer {
            mask: None,
            clipping: false,
            key: l.key,
            blend_mode: l.blend,
            opacity: l.opacity,
        })
        .collect();

    comp.ensure_dummy(w, h);
    comp.tess.reset_stats();
    comp.stage.reset_stats();
    // QUAL motor produziu os pixels — entra na impressão digital da frescura (constante no processo
    // hoje, e explícito para o dia em que o interruptor virar dinâmico).
    let walk = new_engine_armed();

    // Passe 1: garante a tesselação das camadas COM desenho (cache-hit num
    // hold/pan/zoom). Uma camada só-preview (`drawing: None`) não cacheia.
    for l in layers {
        if let Some(d) = l.drawing {
            comp.tess.ensure(l.cache_key, d);
        }
    }

    // Passe 2: rasteriza + injeta cada camada. O straight scratch é reusado: a
    // cópia do `inject` (submissão k) roda antes do próximo `stage_layer`
    // sobrescrevê-lo (submissão k+1) — garantido pela ordem da fila.
    {
        let FlipComposite {
            compositor,
            tess,
            stage,
            ..
        } = comp;
        for l in layers {
            // Camada-alvo do preview: dobra o traço em curso na geometria dela
            // (clone só desta camada, só enquanto desenha) → compõe pelo blend/
            // opacity dela em tempo real. As demais reusam o cache direto.
            let merged;
            let data: &FlipGpuData = if let Some(pv) = l.preview {
                let mut base = if l.drawing.is_some() {
                    tess.get(&l.cache_key).cloned().unwrap_or_default()
                } else {
                    FlipGpuData::default()
                };
                base.append(pv);
                merged = base;
                &merged
            } else {
                tess.get(&l.cache_key).expect("garantido no passe 1")
            };
            // A geometria é LOCAL; o `model` do objeto entra pelo `world_to_clip`
            // (e a espessura pela escala). A **paralaxe multiplano** (2.5D, ADR-0114
            // §Decisão 3) desloca a translação do `model` por `depth` ANTES do fold —
            // a MESMA porta que a arte assada e o preview desta camada usam, senão o
            // esboço vivo descolaria. `depth == 1.0` devolve o model intacto ⇒ o caminho
            // comum (todas as camadas flat) fica byte-idêntico.
            // Numa fatia de FANTASMA, a mesma câmera carrega o tint (a cobertura é a
            // mesma do desenho; só a cor e o alpha mudam).
            let model = parallax_model(&l.model, cam_center, l.depth);
            let mut layer_cam = if model.is_identity() {
                *cam
            } else {
                fold_model(cam, &model)
            };
            if let Some((rgb, a)) = l.ghost {
                layer_cam = layer_cam.with_ghost_tint(rgb, a);
            }
            // ⭐ **A FRESCURA** (doc 12 §22.3): o Pass A rasterizava por camada, por frame, SEMPRE —
            // então arte commitada e fantasma de onion pagavam o preço inteiro para redesenhar
            // pixels idênticos. Aqui a pergunta é feita ANTES do trabalho, e ela tem duas metades:
            // a nossa impressão digital e a palavra do compositor (`has_slice`), porque a fatia pode
            // ter sido despejada pelo LRU ou limpa por um rebuild — e um memo sozinho mandaria
            // compor arte velha nesses dois casos, sem nada parecer quebrado.
            let fp = crate::pass_stage::fingerprint(
                tess.hash(&l.cache_key).unwrap_or(0),
                l.preview,
                &layer_cam,
                (w, h),
                walk,
            );
            if !stage.needs_stage(l.key, fp, compositor.has_slice(l.key)) {
                continue;
            }
            let slice = flip_compose.stage_layer(
                &gpu.device,
                &gpu.queue,
                flip_render,
                &layer_cam,
                data,
                (w, h),
            );
            // ⚠️ **A `version` do inject fica em `0`, e NÃO é a impressão digital.** O
            // `DummyProvider` reporta versão 0 para qualquer chave, então o `ensure_slice` do
            // compositor só acha a fatia "limpa" — e não sobe o dummy TRANSPARENTE por cima da arte
            // — enquanto os dois números batem. A frescura mora no nosso memo justamente por isso.
            if let Err(e) =
                compositor.inject_slice_from_texture(gpu, &ops, l.key, slice, w, h, (0, 0, w, h), 0)
            {
                eprintln!("[ph2d-flip] inject falhou: {e}");
                return None;
            }
            stage.record(l.key, fp);
        }
    }

    Some(ops)
}

/// O passe 3 do ECRÃ: compõe (blend/opacity por-camada) e blita a saída no `game_rt`.
fn composite_lit(
    layers: &[LayerRef<'_>],
    ops: &[LayerOp],
    flip_compose: &mut FlipCompose,
    comp: &mut FlipComposite,
    game_rt: &GameRt,
    (w, h): (u32, u32),
    gpu: &GpuContext,
) {
    {
        let FlipComposite {
            compositor, dummy, ..
        } = comp;
        let provider = DummyProvider { pixels: dummy };
        // ⭐ W5j: com objetos que NÃO recebem a luz acesa, cada trecho seguido de camadas com a
        // mesma resposta compõe-se à parte (as outras a opacidade 0) e blita-se com a luz dele.
        // Sem ninguém de fora: um trecho só, o de sempre.
        for (recebe, trecho) in trechos(layers, flip_compose) {
            let so: Vec<LayerOp>;
            let ops_do_trecho = if trecho == (0..layers.len()) {
                ops
            } else {
                so = ops
                    .iter()
                    .enumerate()
                    .map(|(k, &op)| match op {
                        LayerOp::Layer {
                            mask,
                            clipping,
                            key,
                            blend_mode,
                            ..
                        } if !trecho.contains(&k) => LayerOp::Layer {
                            mask,
                            clipping,
                            key,
                            blend_mode,
                            opacity: 0.0,
                        },
                        o => o,
                    })
                    .collect();
                &so
            };
            if let Err(e) =
                compositor.composite(gpu, ops_do_trecho, &provider, w, h, Region::full(w, h))
            {
                eprintln!("[ph2d-flip] composite falhou: {e}");
                return;
            }
            if let Some(out) = compositor.output_texture() {
                flip_compose.blit_com(&gpu.device, &gpu.queue, out, game_rt.view(), recebe);
            }
        }
    }
}

/// ⭐ W5j — os trechos seguidos de camadas cujo objeto dá a MESMA resposta a «recebe a luz?», com
/// essa resposta. Ninguém de fora (ou luz apagada) ⇒ um trecho só, todas as camadas.
fn trechos(layers: &[LayerRef<'_>], compose: &FlipCompose) -> Vec<(bool, std::ops::Range<usize>)> {
    if !compose.ha_quem_nao_recebe() {
        return vec![(true, 0..layers.len())];
    }
    let mut out: Vec<(bool, std::ops::Range<usize>)> = Vec::new();
    for (k, l) in layers.iter().enumerate() {
        let recebe = compose.recebe(l.obj);
        match out.last_mut() {
            Some((r, t)) if *r == recebe => t.end = k + 1,
            _ => out.push((recebe, k..k + 1)),
        }
    }
    out
}

/// Rasteriza `data` DIRETO no `game_rt` (premult-over, `LoadOp::Load`) com o depth
/// próprio do Flip — o overlay do preview ao vivo. Uma passagem simples (sem o
/// compositor 22-modos): o traço em curso é sempre Normal por cima.
pub(super) fn draw_overlay(
    flip_render: &mut FlipRenderer,
    data: &FlipGpuData,
    cam: &CameraRaw,
    game_rt: &GameRt,
    (w, h): (u32, u32),
    gpu: &GpuContext,
) {
    rasterizar_em(flip_render, data, cam, game_rt.view(), (w, h), gpu);
}

/// Rasteriza `data` em `alvo` (premult-over, `LoadOp::Load`) com o depth próprio do `flip_render` —
/// o overlay do preview e a camada da luz 2D ([`luz`]).
pub(crate) fn rasterizar_em(
    flip_render: &mut FlipRenderer,
    data: &FlipGpuData,
    cam: &CameraRaw,
    alvo: &wgpu::TextureView,
    (w, h): (u32, u32),
    gpu: &GpuContext,
) {
    if data.is_empty() {
        return;
    }
    flip_render.upload(&gpu.device, &gpu.queue, cam, data);
    flip_render.ensure_depth(&gpu.device, (w, h));
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-flip preview overlay"),
        });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-flip preview overlay pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: alvo,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: flip_render.depth_view().map(|v| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: v,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        flip_render.draw(&mut pass);
    }
    gpu.queue.submit([enc.finish()]);
}

/// Chave do compositor para o fantasma `delta` da camada — distinta da chave da
/// própria camada e das dos outros fantasmas dela (senão duas fatias dividiriam a
/// mesma textura e uma sobrescreveria a outra).
fn ghost_key(object_id: u64, layer_id: u32, delta: i32) -> u64 {
    layer_key(object_id, layer_id).wrapping_mul(0xD6E8_FEB8_6659_FD93) ^ (delta as u32 as u64)
}

/// Chave estável do compositor por (objeto, camada) — determinística e distinta
/// entre objetos (evita colisão quando dois objetos têm `LayerId` iguais). O
/// compositor cacheia fatias por chave, então a estabilidade frame-a-frame dá
/// reuso; a mistura é transcendental-free (HR-5).
fn layer_key(object_id: u64, layer_id: u32) -> u64 {
    object_id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (layer_id as u64)
}

// A CÂMERA do passe mora na `ph2d-flip-render` (A1, 12/09): a mesma da assadura do Motion.
use ph2d_flip_render::camera::{camera_scene, fold_model, parallax_model};
// O passe já não chama a versão SEM sub-retângulo (ela sobrevive dentro do `camera_scene`, no ramo
// `None`), mas o `multiplane_tests` endereça-a por `super::camera_raw`.
#[cfg(test)]
use ph2d_flip_render::camera::camera_raw;

#[cfg(test)]
#[path = "pass_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "multiplane_tests.rs"]
mod multiplane_tests;

/// ⭐ O quadro ACTUAL de um objecto, rasterizado fora do ecrã (o «Send to board…», W15).
#[path = "pass_raster.rs"]
pub mod raster;
