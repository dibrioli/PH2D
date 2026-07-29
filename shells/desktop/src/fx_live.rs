//! **A PILHA de FX raster VIVA na shell — 100% RESIDENTE NA GPU** (`ph2d_ecs::VecFilter`, plano 24).
//!
//! Irmão do [`crate::offset_live`]/[`crate::contour_live`], mas de OUTRA natureza: o offset produz
//! GEOMETRIA (uma `LiveGeometry` que o `dispatch` desenha), e o FX produz PIXELS. A costura do
//! plano 24 §2: um FX raster não é `PathEffect` (não é `VecPath -> VecPath`) nem `LiveGeometry` —
//! ele isola a forma na própria textura, roda a pilha e recompõe.
//!
//! # Por que GPU-resident (Enio: "tudo é para o game em runtime, total performance")
//!
//! O 1º corte foi CPU-first (render → **readback GPU→CPU** → Gaussiana na CPU → **re-upload**) — um
//! padrão de PREVIEW de editor. Em runtime a forma filtrada ANIMA, então esse roundtrip roda por
//! frame por forma: readback bloqueia o pipeline e o re-upload cresce sem fim (o Vello cacheia
//! imagem por id de Blob, e um Blob novo por frame vaza o atlas). Não há "total performance" com
//! CPU no caminho.
//!
//! Agora é **tudo na placa**:
//! 1. **todas** as formas que erraram o memo são rasterizadas num [`VelloPass`] scratch, **numa
//!    passagem só**, cada uma na célula que o [`crate::fx_atlas`] lhe deu (renderer próprio, **sem
//!    readback**);
//! 2. o [`FxStackPass`] roda a PILHA (op₁ → op₂ → … → resolve) na GPU para cada forma, lendo a
//!    célula dela, para uma textura de SAÍDA;
//! 3. essa textura é registrada no renderer PRINCIPAL por um **id ESTÁVEL**
//!    ([`VelloPass::register_texture`]); re-cozinhar escreve NA MESMA textura, então o Vello reusa o
//!    slot do atlas (zero churn de id, zero upload de CPU) e o `dispatch` a desenha no z da forma.
//!
//! # Por que UM render, e não `n`
//!
//! ⚠️ Um render do Vello custa **~0,12 ms antes de desenhar coisa alguma** — ele corre a cadeia de
//! binning/tiling inteira e submete, seja qual for o conteúdo. Com um render por forma, uma cena
//! de 32 formas filtradas gastava **4,0 ms só em raster**; a MESMA área de arte numa passagem só
//! custa **0,39 ms** (medido na RTX: `ph2d-render/tests/fx_scene_scale_cost.rs`). Num editor isso
//! é invisível — o artista mexe numa forma de cada vez —, mas o eixo é o do Enio: *"performance em
//! tempo real em runtime para games"*, e num jogo as formas filtradas **animam**, logo erram o
//! memo **todas, todo frame**. O que multiplica é o fixo, não o filtro.
//!
//! O memo (por *pilha resolvida em pixels* + tamanho) vira OTIMIZAÇÃO — pula o re-cook quando nada
//! muda —, não requisito de correção: mesmo re-cozinhando todo frame (cena inteira animada), é UM
//! render + `2n+1` passes por forma. Vazio = nenhum FX (byte-idêntico ao mundo pré-FX).
//!
//! # A conversão MUNDO → PIXEL mora aqui, e só aqui
//!
//! O componente fala MUNDO (resolution-crisp: dar zoom aumenta o borrão na tela, como deve). O
//! passe fala PIXEL. A câmera é conhecida por esta função e por mais ninguém — uma segunda
//! conversão noutro sítio seria um segundo sítio a errá-la.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecFilter};
use ph2d_gpu::GpuContext;
use ph2d_render::{FxOpGpu, FxStackPass, VelloPass, make_output_texture, stack_reach};
use ph2d_vec_render::{FxImage, FxImages, LiveGeometry};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};
use ph2d_vector::{Affine, Color, ImageData, Rect, StableImage, VectorScene};

use crate::vec_entities::VecEntityMap;

/// O maior lado de scratch/saída que pedimos à GPU — o `maxTextureDimension2D` baseline do WebGPU
/// (8192). Limite de RECURSO (a dimensão de textura garantida), não de gosto.
const MAX_FX_SIDE: u32 = 8192;

/// Os recursos de GPU PERSISTENTES de uma forma filtrada: a textura de saída (o resultado da
/// pilha) e o handle [`ImageData`] estável que a referencia no renderer principal. `tex` fica viva
/// para o re-cook e para o Vello copiá-la no render (DEPOIS do recook).
struct PathFx {
    /// O handle registrado no renderer principal (id de Blob estável).
    image: ImageData,
    /// A textura de saída (a pilha escreve aqui; o Vello copia daqui). Clone = handle, mesma tex.
    tex: wgpu::Texture,
    w: u32,
    h: u32,
    /// A pilha JÁ RESOLVIDA em pixels — a chave do memo. Guardá-la resolvida (e não o componente)
    /// é o que faz o zoom invalidar sozinho: a mesma pilha noutro zoom é outra lista.
    ops: Vec<FxOpGpu>,
}

/// **O que uma forma filtrada precisa neste frame**, resolvido pela 1ª varredura do `recook`.
///
/// Existe porque a decisão do ATLAS é sobre a CENA: só depois de conhecer o tamanho de todas as
/// formas que erraram o memo é que se sabe em quantos renders elas cabem. Sem isto o laço teria de
/// resolver cada forma duas vezes — e a 2ª resposta é a que poderia divergir.
struct Job {
    id: VecPathId,
    /// A pilha JÁ RESOLVIDA em pixels de tela.
    ops: Vec<FxOpGpu>,
    /// O canto do scratch desta forma, em pixels de tela (a caixa dela mais a margem da pilha).
    ex0: f64,
    ey0: f64,
    w: u32,
    h: u32,
}

/// O cozimento de todos os FX raster da cena, GPU-resident. Runtime-only: o documento guarda a
/// RELAÇÃO (o `VecFilter` na entidade), e isto é o desenho derivado dela.
#[derive(Default)]
pub(crate) struct FxLive {
    live: FxImages,
    /// Renderer dedicado que rasteriza a forma ISOLADA (separado do principal para não pisar no
    /// intermediate da UI). Criado sob demanda no 1º FX da sessão.
    scratch: Option<VelloPass>,
    /// O passe da pilha. Build-once.
    stack: Option<FxStackPass>,
    /// O despejo de pixels (`PH2D_FX_DUMP`) — a sonda que só o app pode dar.
    dump: crate::fx_dump::FxDump,
    /// Os recursos persistentes por forma (textura de saída + handle estável).
    paths: BTreeMap<VecPathId, PathFx>,
    /// Handles a desregistrar do renderer no próximo recook (o `forget` não tem `vello_pass` em mãos).
    pending_unregister: Vec<ImageData>,
    /// Contador só de diagnóstico (`PH2D_FX_PERF`).
    dbg_frames: u64,
}

impl FxLive {
    /// As imagens de FX deste frame — o que o [`ph2d_vec_render::dispatch`] injeta no z das formas.
    /// Vazio = nenhum FX na cena (o desenho é o de sempre, byte-idêntico ao mundo pré-FX).
    pub(crate) fn images(&self) -> &FxImages {
        &self.live
    }

    /// Re-coza o FX de cada forma filtrada, na GPU. Chamado uma vez por frame, DEPOIS do `sync` e
    /// dos outros `recook` (o FX honra a geometria DERIVADA via `live`). `vello_pass` é o renderer
    /// PRINCIPAL (o que desenha `vector_scene`) — os handles de FX têm de ser registrados NELE, ou
    /// o Vello entra em pânico ao desenhá-los.
    ///
    /// ⚠️ **O frame tem DUAS varreduras, e é a divisão que paga a cena:** a primeira resolve o que
    /// cada forma precisa (a pilha em pixels, a caixa, o tamanho do scratch) e decide o memo; a
    /// segunda rasteriza **todas as que erraram numa passagem só**, num ATLAS. Antes era um render
    /// do Vello por forma, e um render custa **~0,12 ms antes de desenhar coisa alguma** — numa
    /// cena de jogo, onde as formas filtradas animam e erram o memo todas, era esse fixo que
    /// multiplicava (medido: 32 formas = 4,0 ms só de raster, contra 0,39 ms para a MESMA área
    /// numa passagem).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &LiveGeometry,
        sil: &LiveGeometry,
        camera: Affine,
        gpu: &GpuContext,
        surface_format: wgpu::TextureFormat,
        vello_pass: &mut VelloPass,
    ) {
        // Desregistra o que morreu (filtro removido, projeto trocado) antes de qualquer coisa.
        for image in self.pending_unregister.drain(..) {
            vello_pass.unregister_texture(image);
        }
        self.live.clear();

        let perf = std::env::var("PH2D_FX_PERF").is_ok();
        let t0 = std::time::Instant::now();

        // 1ª varredura — o que cada forma precisa, e quem errou o memo.
        let mut jobs: Vec<Job> = Vec::new();
        let mut miss: Vec<usize> = Vec::new();
        for path in scene.paths() {
            let Some(filter) = spec_of(sim, map, path.id) else {
                continue;
            };
            let ops = resolve_ops(&filter, camera);
            if ops.is_empty() {
                continue;
            }
            let Some((x0, y0, x1, y1)) =
                ph2d_vec_render::path_screen_bounds(scene, xforms, live, path.id, camera)
            else {
                continue;
            };
            // A margem é da PILHA INTEIRA (as reaches somam ao longo dela) e assimétrica (uma
            // sombra longa para a direita não paga textura à esquerda). Porta única no passe.
            let (ml, mt, mr, mb) = stack_reach(&ops);
            let ex0 = (x0 - f64::from(ml)).floor();
            let ey0 = (y0 - f64::from(mt)).floor();
            let w = (((x1 + f64::from(mr)).ceil() - ex0).max(1.0) as u32).min(MAX_FX_SIDE);
            let h = (((y1 + f64::from(mb)).ceil() - ey0).max(1.0) as u32).min(MAX_FX_SIDE);
            // O memo é otimização: re-coza só quando os PIXELS mudam.
            let hit = self
                .paths
                .get(&path.id)
                .is_some_and(|p| p.ops == ops && p.w == w && p.h == h);
            if !hit {
                miss.push(jobs.len());
            }
            jobs.push(Job {
                id: path.id,
                ops,
                ex0,
                ey0,
                w,
                h,
            });
        }

        // 2ª varredura — o ATLAS. Zero formas a re-cozinhar = zero renders: uma cena parada não
        // paga nada, que é a outra metade do que o memo sempre prometeu.
        let mut renders = 0usize;
        if !miss.is_empty() {
            let sizes: Vec<(u32, u32)> = miss.iter().map(|&i| (jobs[i].w, jobs[i].h)).collect();
            for batch in crate::fx_atlas::pack(&sizes, MAX_FX_SIDE) {
                if self.cook_batch(
                    gpu,
                    surface_format,
                    vello_pass,
                    scene,
                    xforms,
                    live,
                    sil,
                    camera,
                    &jobs,
                    &miss,
                    &batch,
                ) {
                    renders += 1;
                }
            }
        }

        // O que o `dispatch` injeta no z de cada forma.
        //
        // ⚠️ A textura só é publicada se ela for do TAMANHO que este frame pediu — uma falha de
        // recurso a meio do lote deixaria para trás a textura da era anterior, e desenhá-la seria
        // mostrar a forma com a caixa errada em vez de a deixar sem filtro por um frame.
        for job in &jobs {
            let Some(pfx) = self
                .paths
                .get(&job.id)
                .filter(|p| p.w == job.w && p.h == job.h)
            else {
                continue;
            };
            let rect = (
                job.ex0,
                job.ey0,
                job.ex0 + f64::from(pfx.w),
                job.ey0 + f64::from(pfx.h),
            );
            self.live.insert(
                job.id,
                FxImage {
                    // Clone do handle estável = MESMO id de Blob (o slot de atlas do Vello).
                    image: StableImage::from_image_data(pfx.image.clone()),
                    rect,
                },
            );
        }

        // Formas que perderam o filtro: agenda o desregistro (a textura sai do atlas).
        let dead: Vec<VecPathId> = self
            .paths
            .keys()
            .filter(|id| !jobs.iter().any(|j| j.id == **id))
            .copied()
            .collect();
        for id in dead {
            if let Some(pfx) = self.paths.remove(&id) {
                self.pending_unregister.push(pfx.image);
            }
        }

        if perf && (renders > 0 || self.dbg_frames.is_multiple_of(120)) {
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            eprintln!(
                "[fx-perf] {} pilha(s), {} re-cozida(s) em {renders} render(es), recook {ms:.3} ms",
                self.live.len(),
                miss.len()
            );
        }
        self.dbg_frames = self.dbg_frames.wrapping_add(1);
    }

    /// Cozinha um LOTE: **um** render do Vello com todas as formas dele, cada uma na célula que o
    /// empacotador lhe deu, e depois a pilha de cada uma lendo a própria célula.
    ///
    /// Devolve se o render aconteceu (`false` = falha de recurso; as formas do lote ficam sem
    /// filtro neste frame, e o próximo tenta de novo — elas continuam a errar o memo).
    #[allow(clippy::too_many_arguments)]
    fn cook_batch(
        &mut self,
        gpu: &GpuContext,
        surface_format: wgpu::TextureFormat,
        vello_pass: &mut VelloPass,
        scene: &VecScene,
        xforms: &VecXforms,
        live: &LiveGeometry,
        sil: &LiveGeometry,
        camera: Affine,
        jobs: &[Job],
        miss: &[usize],
        batch: &crate::fx_atlas::Batch,
    ) -> bool {
        // 1. UMA cena com as formas do lote, cada uma transladada para a origem da CÉLULA dela.
        //    (`-ex0,-ey0` põe a forma na origem do scratch dela; `+org` põe o scratch na célula.)
        let mut scratch_scene = VectorScene::new();
        for cell in &batch.cells {
            let job = &jobs[miss[cell.index]];
            // ⚠️ **A célula é RECORTADA, e isso repõe um limite que já existia.** Com um render por
            // forma, a textura tinha exactamente o tamanho do scratch dela — arte que passasse da
            // caixa (um traço mais largo do que os limites calculados, uma junta miter comprida)
            // era descartada pelo rasterizador, na borda. Partilhando a textura, essa mesma arte
            // cairia **dentro da célula da vizinha**, e o resultado ainda pareceria arte. O clip
            // não é otimização: é o que mantém o desenho igual ao de antes.
            scratch_scene.push_clip(&Rect::new(
                f64::from(cell.org[0]),
                f64::from(cell.org[1]),
                f64::from(cell.org[0]) + f64::from(job.w),
                f64::from(cell.org[1]) + f64::from(job.h),
            ));
            ph2d_vec_render::draw_path_isolated(
                scene,
                xforms,
                live,
                job.id,
                camera,
                Affine::translate((
                    f64::from(cell.org[0]) - job.ex0,
                    f64::from(cell.org[1]) - job.ey0,
                )),
                &mut scratch_scene,
            );
            scratch_scene.pop_layer();
        }

        // 2. UM render — o número que esta wave existe para levar a um.
        let scratch = match self.scratch.as_mut() {
            Some(p) => p,
            None => match VelloPass::new(gpu, surface_format, (batch.w, batch.h)) {
                Ok(p) => self.scratch.insert(p),
                Err(e) => {
                    eprintln!("[fx] scratch VelloPass::new: {e}");
                    return false;
                }
            },
        };
        if let Err(e) = scratch.render_to_intermediate(
            gpu,
            scratch_scene.inner(),
            (batch.w, batch.h),
            Color::TRANSPARENT,
            false,
        ) {
            eprintln!("[fx] scratch render: {e}");
            return false;
        }

        // 3. As texturas de SAÍDA, por forma. Numa varredura própria porque ela **registra** no
        //    renderer principal (`&mut vello_pass`), e é a única parte do lote que o toca.
        for cell in &batch.cells {
            let job = &jobs[miss[cell.index]];
            self.ensure_output(gpu, vello_pass, job);
        }

        // 4. A PILHA de cada forma, lendo a CÉLULA dela do atlas. Tudo a jusante do ingest fala em
        //    coordenadas locais da forma — inclusive os segmentos de silhueta, que por isso
        //    continuam a ser construídos com `-ex0,-ey0` e nada sabem do atlas.
        let Self {
            scratch,
            stack,
            paths,
            dump,
            ..
        } = self;
        let src = scratch
            .as_ref()
            .expect("scratch ensured")
            .intermediate_texture();
        let stack = stack.get_or_insert_with(|| FxStackPass::new(gpu));
        for cell in &batch.cells {
            let job = &jobs[miss[cell.index]];
            let Some(pfx) = paths.get_mut(&job.id) else {
                continue;
            };
            pfx.ops.clear();
            pfx.ops.extend_from_slice(&job.ops);
            let geom = ph2d_vec_render::silhouette_segments(
                scene,
                xforms,
                live,
                sil,
                job.id,
                camera,
                Affine::translate((-job.ex0, -job.ey0)),
            );
            if std::env::var("PH2D_FX_DIAG").is_ok() {
                eprintln!(
                    "[fx-diag] path {:?}: {} segmentos, celula {}x{} em ({},{}) do atlas {}x{}",
                    job.id,
                    geom.len(),
                    job.w,
                    job.h,
                    cell.org[0],
                    cell.org[1],
                    batch.w,
                    batch.h
                );
            }
            let org = [
                i32::try_from(cell.org[0]).unwrap_or(0),
                i32::try_from(cell.org[1]).unwrap_or(0),
            ];
            stack.run_from(gpu, src, org, &pfx.tex, job.w, job.h, &job.ops, &geom);
            dump.maybe(gpu, job.id, &pfx.tex, job.w, job.h, &job.ops, &geom);
        }
        true
    }

    /// (Re)aloca a textura de saída de uma forma e mantém o id ESTÁVEL no renderer principal.
    fn ensure_output(&mut self, gpu: &GpuContext, vello_pass: &mut VelloPass, job: &Job) {
        if matches!(self.paths.get(&job.id), Some(p) if p.w == job.w && p.h == job.h) {
            return;
        }
        let tex = make_output_texture(gpu, job.w, job.h);
        match self.paths.get_mut(&job.id) {
            // Resize: RE-registra (id novo com as dims certas) e agenda o desregistro do antigo.
            // ⚠️ `override_image` só troca a textura e NÃO atualiza width/height da `ImageData` —
            // copiar com as dims velhas de uma textura de tamanho novo ESTOURA (o Vello avisa isso
            // no doc; foi o "panic ao abrir" pós-resize). Resize é raro ⇒ churn de id mínimo; o
            // re-cook (comum) não re-registra.
            Some(pfx) => {
                let fresh = vello_pass.register_texture(tex.clone());
                let old = std::mem::replace(&mut pfx.image, fresh);
                self.pending_unregister.push(old);
                pfx.tex = tex;
                pfx.w = job.w;
                pfx.h = job.h;
            }
            // Forma nova: registra a textura e ganha um id estável.
            None => {
                let image = vello_pass.register_texture(tex.clone());
                self.paths.insert(
                    job.id,
                    PathFx {
                        image,
                        tex,
                        w: job.w,
                        h: job.h,
                        ops: job.ops.clone(),
                    },
                );
            }
        }
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira, e os `VecPathId`
    /// são reciclados. Os handles vão para a fila de desregistro (o próximo recook os solta do
    /// atlas, pois é lá que há `vello_pass`).
    pub(crate) fn forget(&mut self) {
        self.live.clear();
        for (_, pfx) in std::mem::take(&mut self.paths) {
            self.pending_unregister.push(pfx.image);
        }
    }
}

/// **A pilha AUTORADA resolvida em pixels de tela.** Os degraus desligados caem aqui (a pilha os
/// SALTA, como a de geometria salta um `FxEntry` desarmado), então uma pilha toda desligada devolve
/// vazio e a forma sai nua.
///
/// ⚠️ O deslocamento é arredondado a pixel INTEIRO — o passe amostra o halo por `textureLoad`, e
/// posição sub-pixel numa sombra não é algo que se veja (a textura já é alinhada ao pixel da tela).
#[must_use]
pub(crate) fn resolve_ops(filter: &VecFilter, camera: Affine) -> Vec<FxOpGpu> {
    let [a, b, c, d, _, _] = camera.as_coeffs();
    let cam_scale = ((a * a + b * b).sqrt() + (c * c + d * d).sqrt()) as f32 * 0.5;
    filter
        .ops
        .iter()
        .filter(|o| o.is_active())
        .map(|o| {
            let (ox, oy) = (f64::from(o.offset[0]), f64::from(o.offset[1]));
            // A rampa do Gradient Map, ordenada e empacotada pela porta única do componente — o
            // shader assume ordenado, e ordenar aqui à mão seria a segunda resposta que diverge.
            let (stops, stop_pos, stop_count) = o.ramp_for_device();
            FxOpGpu {
                kind: o.kind,
                sigma_px: (o.radius * cam_scale).max(0.0),
                offset_px: [
                    (a * ox + c * oy).round() as i32,
                    (b * ox + d * oy).round() as i32,
                ],
                tint: o.color,
                // A SEGUNDA ponta da rampa, pelo MESMO caminho da primeira: também não atravessa a
                // câmara (uma cor não é um comprimento), pela razão que os três do ajuste já dão
                // mais abaixo.
                tint_b: o.color_b,
                opacity: o.opacity,
                mode: o.mode,
                // ⚠️ **`blend_code`, não `blend`** — é a metade de HONRAR da porta única
                // `FxOp::takes_blend` (a de OFERECER é do painel). Um arquivo cujo degrau carrega
                // uma lei de um tipo que deixou de a tomar desenharia uma mistura que a UI não
                // mostra; aqui ela vira Normal, e o dispositivo nunca vê um número órfão.
                blend: o.blend_code(),
                stops,
                stop_pos,
                stop_count,
                // O tamanho das ondulações atravessa a mesma conversão do raio — é ela que torna
                // o padrão zoom-invariante (o `noise_p` do shader divide por este número, e o
                // numerador também escala com o zoom, então ele cancela).
                noise_scale_px: (o.scale * cam_scale).max(0.0),
                // ⚠️ **`detail_clamped`, não `detail`** — a metade de HONRAR da porta única, como
                // o `blend_code`: um arquivo com detalhe 0 (ou 200) desenharia um laço vazio (ou
                // caro) que a UI não oferece.
                detail: o.detail_clamped(),
                seed: o.seed,
                // O crescimento atravessa a MESMA conversão do raio (mundo → pixels de tela pelo
                // zoom): engordar 0,06 tem de engordar a mesma fração da forma em qualquer escala.
                // ⚠️ **Sem `max(0.0)`** — aqui o sinal É a operação.
                grow_px: o.grow * cam_scale,
                // ⚠️ **Os três do ajuste NÃO atravessam a câmara, e é a diferença que importa:**
                // eles não são comprimentos. Uma matiz é um ÂNGULO e a saturação/brilho são
                // frações — dar zoom não pode mudar a cor de nada. Multiplicá-los pelo
                // `cam_scale` seria o mesmo erro que dividir o raio por ele.
                hue: o.hue,
                sat: o.sat,
                bright: o.bright,
            }
        })
        .collect()
}

/// A pilha de `id`, se houver. Porta única: o cozimento e o publish para o painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecFilter> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecFilter>(Entity::from_bits(bits))
        .cloned()
}

/// **Escreve** (ou remove) a pilha de cada caminho de `ids`. Uma pilha VAZIA remove o componente —
/// a lei do `VecOffset`: um documento não acumula relações inertes que não desenham nada. Devolve
/// quantas entidades mudaram.
pub(crate) fn set_filter(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    want: Option<VecFilter>,
) -> usize {
    let want = want.filter(|f| !f.ops.is_empty());
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let cur = sim.world().get::<VecFilter>(e).cloned();
        if cur == want {
            continue;
        }
        let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        match &want {
            Some(v) => {
                em.insert(v.clone());
            }
            None => {
                em.remove::<VecFilter>();
            }
        }
        n += 1;
    }
    n
}

/// **Edita** a pilha de cada caminho de `ids` que JÁ tenha uma (read-modify-write) — o arrasto de
/// um slider, a cor do picker, o ✕ de uma linha. Espelho de `contour_live::edit`.
///
/// Se a edição esvaziar a pilha, o componente é REMOVIDO (a mesma lei do `set_filter`, perguntada
/// no mesmo lugar: quem remove a última linha não deixa um componente inerte para trás).
pub(crate) fn edit(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    f: impl Fn(&mut VecFilter),
) {
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let Some(mut cur) = sim.world().get::<VecFilter>(e).cloned() else {
            continue;
        };
        f(&mut cur);
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            if cur.ops.is_empty() {
                em.remove::<VecFilter>();
            } else {
                em.insert(cur);
            }
        }
    }
}

/// A tradução id→controle mora no irmão [`crate::fx_live_hit`] (teto de LOC), e é re-exportada
/// aqui porque `fx_live::hit_of` é a porta que a ponte chama: mover a função não pode mover o
/// caminho de quem a usa.
pub(crate) use crate::fx_live_hit::{FilterHit, colour_bytes, colour_target, hit_of};

#[cfg(test)]
#[path = "fx_live_tests.rs"]
mod tests;
