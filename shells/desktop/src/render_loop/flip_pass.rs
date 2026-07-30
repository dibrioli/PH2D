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

use super::flip_pass_cache::TessCache;
use crate::flip_transform::art_to_world;
use ph2d_core::Playhead;
use ph2d_flip::{FlipDoc, FlipDrawing, FlipObjectId, LayerId};
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
pub(crate) struct FlipComposite {
    compositor: LayerCompositor,
    dummy: Vec<u8>,
    /// Geometria empacotada por (objeto, desenho), camera-INDEPENDENTE — sobrevive
    /// a pan/zoom e a *holds*. Validada por hash de conteúdo.
    tess: TessCache,
    /// **A frescura do Pass A** (doc 12 §22.3): o que já foi rasterizado em cada fatia. É o que faz
    /// arte commitada e fantasma de onion custarem **zero** enquanto ninguém mexe neles.
    stage: super::flip_pass_stage::StageMemo,
}

impl FlipComposite {
    fn new(gpu: &GpuContext) -> Self {
        Self {
            compositor: LayerCompositor::new(gpu),
            dummy: Vec::new(),
            tess: TessCache::default(),
            stage: super::flip_pass_stage::StageMemo::default(),
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
/// (`active_layer`) para compor pelo blend/opacity dela em tempo real. No-op se
/// não há camada ativa NEM preview (cena vazia = o default sem `PH2D_FLIP_DEMO`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn render(
    flip: &FlipDoc,
    flip_render: &mut FlipRenderer,
    flip_compose: &mut FlipCompose,
    flip_composite: &mut Option<FlipComposite>,
    preview: Option<&FlipGpuData>,
    active_layer: Option<LayerId>,
    models: &[(FlipObjectId, Xform)],
    playhead: &Playhead,
    // Ghost Frames: `Some(chaves selecionadas)` = a tool Flip está ativa (fantasmas
    // ligados, sujeitos aos gates do objeto/camada e ao "some no play"); `None` =
    // outra tool no comando — a cena Flip aparece limpa, sem fantasma.
    ghosts: Option<super::flip_pass_ghosts::GhostSources<'_>>,
    // O PEEK (Shift & Trace fatia 2): `Some` = uma folha vizinha na mão (a camada
    // ATIVA amostra o desenho anterior/atual/seguinte). O shell passa `ghosts: None`
    // junto — o flip é uma folha na mão, não uma pilha translúcida.
    peek: Option<crate::flip_peek::PeekDir>,
    game_rt: &GameRt,
    camera: &Camera2d,
    window: WindowSize,
    gpu: &GpuContext,
) {
    // O preview é atribuído à camada ativa (dobrado na fatia dela). `unfolded` só
    // sobra quando a camada-alvo é invisível/irresolvível — aí cai no overlay
    // Normal (o usuário nunca desenha às cegas). Os fantasmas entram na MESMA lista,
    // cada um como uma fatia logo abaixo da sua camada.
    let (layers, unfolded) =
        collect_layers(flip, playhead, preview, active_layer, models, ghosts, peek);
    if layers.is_empty() && unfolded.is_none() {
        return;
    }
    let (w, h) = (window.width.max(1), window.height.max(1));
    let cam = camera_raw(camera, window);

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

/// **O PERCURSO É O DEFAULT** (doc 12 §22) — `PH2D_FLIP_NEW_ENGINE=0` é a ESCAPE para o
/// rasterizador que shipava.
///
/// A inversão é a decisão do padrão-ouro, e o que a sustenta é a hierarquia das leis, não uma
/// preferência: a lei do percurso (`τ = ∫ f(dn) ds`, `α = 1 − exp(−τ)`) é o **limite contínuo** que
/// os dab buffers de GIMP/Krita/Procreate — e o do nosso próprio Painter — aproximam por soma
/// finita, e o rasterizador (união global + eleição por depth) não está na família. Medido contra o
/// depósito do Painter, o pico na ponta: raster **+129/+131/+175** contra percurso
/// **−12/−17/−46** (durezas 0,2/0,4/0,7).
fn new_engine_armed() -> bool {
    static ARMED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ARMED.get_or_init(|| walk_from_env(std::env::var("PH2D_FLIP_NEW_ENGINE").ok().as_deref()))
}

/// A política do interruptor, **PURA** — quem decide *qual motor* a partir do que o ambiente diz.
///
/// ⚠️ **Ela existe separada porque o default não era testável:** o [`new_engine_armed`] é um
/// `OnceLock` sobre uma variável de processo, então nenhum teste consegue exercitá-lo duas vezes no
/// mesmo binário, e um default que ninguém pode afirmar é um default que a próxima edição inverte
/// em silêncio.
///
/// ⚠️ **Só o desligamento EXPLÍCITO volta ao raster** — ausente, vazio ou irreconhecível dá o
/// percurso. Isso é deliberado: um erro de digitação na escape (`=flase`) falha **para o default**,
/// nunca para um terceiro comportamento.
fn walk_from_env(v: Option<&str>) -> bool {
    !matches!(v, Some("0" | "false" | "off"))
}

/// Compõe as camadas ativas (blend/opacity por-camada) e blita no `game_rt`.
#[allow(clippy::too_many_arguments)]
fn composite_layers(
    layers: &[LayerRef<'_>],
    flip_render: &mut FlipRenderer,
    flip_compose: &mut FlipCompose,
    flip_composite: &mut Option<FlipComposite>,
    game_rt: &GameRt,
    cam: &CameraRaw,
    // O centro (pan) da câmera-viewport em MUNDO — a paralaxe multiplano lerpa a
    // translação de cada camada entre ele e a origem do objeto por `depth`.
    cam_center: [f32; 2],
    (w, h): (u32, u32),
    gpu: &GpuContext,
) {
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

    let comp = flip_composite.get_or_insert_with(|| FlipComposite::new(gpu));
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
            let fp = super::flip_pass_stage::fingerprint(
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
                return;
            }
            stage.record(l.key, fp);
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
    if std::env::var_os("PH2D_FLIP_STATS").is_some() {
        let (staged, skipped) = comp.stage.stats();
        eprintln!("[ph2d-flip] pass A: {staged} rasterizada(s), {skipped} pulada(s) neste frame");
    }
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
                view: game_rt.view(),
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

/// As camadas ativas AGORA, na ordem de composição (objeto por objeto; dentro de
/// cada objeto, de baixo p/ cima = ordem do slice; só visíveis). Cada objeto
/// amostra pelo SEU FPS. NÃO empacota — isso é sob demanda no cache (`ensure_tess`),
/// pra troca de quadro barata.
///
/// O `preview` (traço em curso) é **dobrado na camada-alvo** — a `active_layer` do
/// 1º objeto (fallback: topo, igual ao bake) — para compor pelo blend/opacity dela
/// em tempo real, na posição z certa (inclusive uma camada ainda VAZIA, sintetizada
/// aqui). Devolve `(camadas, preview_não_atribuído)`: o 2º é `Some` só quando a
/// camada-alvo é oculta/inexistente (cai no overlay Normal — nunca desenhar às cegas).
fn collect_layers<'a>(
    flip: &'a FlipDoc,
    playhead: &Playhead,
    preview: Option<&'a FlipGpuData>,
    active_layer: Option<LayerId>,
    models: &[(FlipObjectId, Xform)],
    ghosts: Option<super::flip_pass_ghosts::GhostSources<'_>>,
    peek: Option<crate::flip_peek::PeekDir>,
) -> (Vec<LayerRef<'a>>, Option<&'a FlipGpuData>) {
    // Camada-alvo do preview: a ativa do 1º objeto (se ainda existe) ou o topo —
    // exatamente o fallback que o `bake_stroke` usa. `None` sem preview.
    let target: Option<(u64, LayerId)> = preview.and_then(|_| {
        let obj = flip.objects().first()?;
        let lid = active_layer
            .filter(|id| obj.layer(*id).is_some())
            .or_else(|| obj.layers().last().map(|l| l.id))?;
        Some((obj.id.0, lid))
    });

    // A camada que o PEEK retima — a MESMA resolução do alvo do preview (a ativa do
    // 1º objeto, ou o topo): é a camada que o animador está folheando.
    let peek_target: Option<(u64, LayerId)> = peek.and_then(|_| {
        let obj = flip.objects().first()?;
        let lid = active_layer
            .filter(|id| obj.layer(*id).is_some())
            .or_else(|| obj.layers().last().map(|l| l.id))?;
        Some((obj.id.0, lid))
    });

    let mut out = Vec::new();
    let mut unfolded = preview; // vira None assim que o preview é dobrado
    for obj in flip.objects() {
        let frame = obj.frame_at(playhead);
        // O afim LOCAL→mundo do objeto (ADR-0111). Ausente = identidade (o objeto
        // nunca foi movido; a geometria ainda É mundo).
        let model = models
            .iter()
            .find(|(id, _)| *id == obj.id)
            .map_or(Xform::IDENTITY, |(_, x)| *x);
        for layer in obj.layers() {
            if !layer.visible {
                continue; // oculta não contribui (preview numa oculta cai no fallback)
            }
            let this_preview = if target == Some((obj.id.0, layer.id)) {
                unfolded.take()
            } else {
                None
            };
            // O PEEK: a camada-alvo amostra a folha VIZINHA (âncora = chave ativa;
            // `flip_peek::peek_frame`). As outras ficam no quadro — são o contexto
            // sobre o qual se folheia. Sem peek, `sample == frame` e nada muda.
            let sample = match peek {
                Some(dir) if peek_target == Some((obj.id.0, layer.id)) => {
                    crate::flip_peek::peek_frame(layer, frame, dir)
                }
                _ => frame,
            };
            // **Pelo CICLO** (`drawing_at_cycled`), não pelo caminho cru: é aqui que
            // Loop/Ping-Pong existem. Amostrar cru fazia o último desenho segurar para
            // sempre e os ciclos não faziam NADA (o bug do 1º corte).
            let did = layer.drawing_at_cycled(sample);
            let drawing = did.and_then(|d| obj.drawing(d));
            let has_geo = drawing.is_some_and(|d| !d.strokes.is_empty());

            // Os fantasmas DESTA camada entram ANTES dela na pilha (mais para o
            // fundo) — e DEPOIS de todas as camadas de baixo. É isto que faz o
            // fantasma da camada de cima aparecer POR CIMA da arte da camada de
            // baixo: ele é uma fatia da pilha, não um passe por baixo de tudo.
            // Blend Normal + opacity 1: o fade/opacidade já estão no alpha do tint
            // (aplicá-los de novo pelo compositor os elevaria ao quadrado, e um
            // blend Multiply da camada tingiria o fantasma com a arte de baixo).
            if let Some(sources) = ghosts {
                // No quadro-FONTE (o do ciclo): os vizinhos do desenho que está NA
                // TELA. No quadro cru, um Loop na 2ª volta não teria vizinho nenhum.
                let src = layer.source_frame(frame);
                for g in super::flip_pass_ghosts::collect(obj, layer, src, playhead, sources) {
                    out.push(LayerRef {
                        key: ghost_key(obj.id.0, layer.id.0, g.delta),
                        blend: ph2d_flip::BlendMode::default().to_u8(),
                        opacity: 1.0,
                        cache_key: (obj.id.0, g.drawing_id),
                        drawing: Some(g.drawing),
                        preview: None,
                        // **Cada fantasma na POSE DA SUA chave** (W7.2): o fantasma
                        // mostra onde o desenho ESTAVA — e "onde" inclui o lugar, não
                        // só a forma. Herdar a pose do quadro corrente empilharia todos
                        // os fantasmas em cima da arte de agora. O `shift` é o Shift &
                        // Trace (a folha deslizada) — identidade fora do modo.
                        model: crate::flip_transform::art_to_world_traced(
                            &model,
                            layer.frame_pose(g.key),
                            g.shift,
                        ),
                        ghost: Some((g.tint, g.alpha)),
                        // O fantasma paralaxa NO plano da própria camada (um ghost de
                        // fundo mora no fundo), senão ele descolaria da arte que sombreia.
                        depth: layer.depth,
                    });
                }
            }

            if !has_geo && this_preview.is_none() {
                continue; // sem geometria e sem preview aqui = não compõe
            }
            out.push(LayerRef {
                key: layer_key(obj.id.0, layer.id.0),
                blend: layer.blend.to_u8(),
                opacity: layer.opacity,
                cache_key: (obj.id.0, did.map_or(u32::MAX, |d| d.0)),
                drawing: if has_geo { drawing } else { None },
                preview: this_preview,
                // A pose da chave EXIBIDA — amostrada pelo mesmo mapa do desenho
                // (`offset_at_cycled`), senão a arte da 2ª volta de um Loop sairia no
                // lugar de um quadro que não existe.
                //
                // O PREVIEW (o traço vivo) entra nesta MESMA fatia e por isso recebe a
                // mesma pose — e está certo: ele já foi convertido para o espaço do
                // DESENHO pelo funil de entrada (`flip_active_world_to_local`, que desfaz
                // a pose da chave ativa). As duas pontas usam a mesma transform, que é a
                // única forma de o preview não folgar do traço assado.
                model: art_to_world(&model, layer.pose_at_cycled(sample)),
                ghost: None,
                depth: layer.depth,
            });
        }
    }
    (out, unfolded)
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

#[path = "flip_pass_camera.rs"]
mod camera;
use camera::{camera_raw, fold_model, parallax_model};

#[cfg(test)]
#[path = "flip_pass_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "flip_multiplane_tests.rs"]
mod multiplane_tests;
