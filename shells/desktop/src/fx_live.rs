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
//! 1. a forma isolada é rasterizada num [`VelloPass`] scratch (renderer próprio, **sem readback**);
//! 2. o [`FxStackPass`] roda a PILHA (op₁ → op₂ → … → resolve) na GPU, para uma textura de SAÍDA;
//! 3. essa textura é registrada no renderer PRINCIPAL por um **id ESTÁVEL**
//!    ([`VelloPass::register_texture`]); re-cozinhar escreve NA MESMA textura, então o Vello reusa o
//!    slot do atlas (zero churn de id, zero upload de CPU) e o `dispatch` a desenha no z da forma.
//!
//! O memo (por *pilha resolvida em pixels* + tamanho) vira OTIMIZAÇÃO — pula o re-cook quando nada
//! muda —, não requisito de correção: mesmo re-cozinhando todo frame (forma animada), é um render
//! + `2n+1` passes na GPU. Vazio = nenhum FX (byte-idêntico ao mundo pré-FX).
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
use ph2d_vector::{Affine, Color, ImageData, StableImage, VectorScene};

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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        sim: &SimWorld,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &LiveGeometry,
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
        let mut misses = 0usize;
        let mut seen: Vec<VecPathId> = Vec::new();

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
            seen.push(path.id);

            // O memo é otimização: re-coza só quando os PIXELS mudam.
            let hit = self
                .paths
                .get(&path.id)
                .is_some_and(|p| p.ops == ops && p.w == w && p.h == h);
            if !hit {
                misses += 1;
                if !self.recook_one(
                    gpu,
                    surface_format,
                    vello_pass,
                    scene,
                    xforms,
                    live,
                    camera,
                    path.id,
                    &ops,
                    ex0,
                    ey0,
                    w,
                    h,
                ) {
                    continue;
                }
            }

            let Some(pfx) = self.paths.get(&path.id) else {
                continue;
            };
            let rect = (ex0, ey0, ex0 + f64::from(pfx.w), ey0 + f64::from(pfx.h));
            self.live.insert(
                path.id,
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
            .filter(|id| !seen.contains(id))
            .copied()
            .collect();
        for id in dead {
            if let Some(pfx) = self.paths.remove(&id) {
                self.pending_unregister.push(pfx.image);
            }
        }

        if perf && (misses > 0 || self.dbg_frames.is_multiple_of(120)) {
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            eprintln!(
                "[fx-perf] {} pilha(s), {misses} re-cozida(s), recook {ms:.3} ms",
                self.live.len()
            );
        }
        self.dbg_frames = self.dbg_frames.wrapping_add(1);
    }

    /// Cozinha UMA forma: rasteriza isolada no scratch, roda a pilha na GPU para a textura de
    /// saída (realoca + re-registra se o tamanho mudou), e atualiza o `PathFx`. `false` só em
    /// falha de recurso (pula a forma neste frame).
    #[allow(clippy::too_many_arguments)]
    fn recook_one(
        &mut self,
        gpu: &GpuContext,
        surface_format: wgpu::TextureFormat,
        vello_pass: &mut VelloPass,
        scene: &VecScene,
        xforms: &VecXforms,
        live: &LiveGeometry,
        camera: Affine,
        id: VecPathId,
        ops: &[FxOpGpu],
        ex0: f64,
        ey0: f64,
        w: u32,
        h: u32,
    ) -> bool {
        // 1. Rasteriza a forma isolada no scratch (sem readback), na MESMA escala da tela (câmera),
        //    transladada para a origem do scratch (`-ex0,-ey0`).
        let mut scratch_scene = VectorScene::new();
        ph2d_vec_render::draw_path_isolated(
            scene,
            xforms,
            live,
            id,
            camera,
            Affine::translate((-ex0, -ey0)),
            &mut scratch_scene,
        );
        let scratch = match self.scratch.as_mut() {
            Some(p) => p,
            None => match VelloPass::new(gpu, surface_format, (w, h)) {
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
            (w, h),
            Color::TRANSPARENT,
            false,
        ) {
            eprintln!("[fx] scratch render: {e}");
            return false;
        }

        // 2. (Re)aloca a textura de saída por forma e mantém o id ESTÁVEL no renderer principal.
        let need_alloc = !matches!(self.paths.get(&id), Some(p) if p.w == w && p.h == h);
        if need_alloc {
            let tex = make_output_texture(gpu, w, h);
            match self.paths.get_mut(&id) {
                // Resize: RE-registra (id novo com as dims certas) e agenda o desregistro do
                // antigo. ⚠️ `override_image` só troca a textura e NÃO atualiza width/height da
                // `ImageData` — copiar com as dims velhas de uma textura de tamanho novo ESTOURA
                // (o Vello avisa isso no doc; foi o "panic ao abrir" pós-resize). Resize é raro
                // ⇒ churn de id mínimo; o re-cook (comum) não re-registra.
                Some(pfx) => {
                    let fresh = vello_pass.register_texture(tex.clone());
                    let old = std::mem::replace(&mut pfx.image, fresh);
                    self.pending_unregister.push(old);
                    pfx.tex = tex;
                    pfx.w = w;
                    pfx.h = h;
                }
                // Forma nova: registra a textura e ganha um id estável.
                None => {
                    let image = vello_pass.register_texture(tex.clone());
                    self.paths.insert(
                        id,
                        PathFx {
                            image,
                            tex,
                            w,
                            h,
                            ops: ops.to_vec(),
                        },
                    );
                }
            }
        }

        // 3. A PILHA na GPU: scratch intermediate (premul) → op₁ → … → textura de saída (reta).
        let src = self
            .scratch
            .as_ref()
            .expect("scratch ensured")
            .intermediate_texture();
        let pfx = self.paths.get_mut(&id).expect("path ensured");
        pfx.ops.clear();
        pfx.ops.extend_from_slice(ops);
        // A SILHUETA em segmentos, no MESMO transform com que a forma foi rasterizada no scratch
        // — é ela que dá ao campo de distância o pé exato da fronteira. Vazia (forma com traço,
        // ou complexa demais) = o campo cai no caminho do raster, que é pior mas nunca trava.
        let geom = ph2d_vec_render::silhouette_segments(
            scene,
            xforms,
            live,
            id,
            camera,
            Affine::translate((-ex0, -ey0)),
        );
        let stack = self.stack.get_or_insert_with(|| FxStackPass::new(gpu));
        stack.run(gpu, src, &pfx.tex, w, h, ops, &geom);
        true
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
            FxOpGpu {
                kind: o.kind,
                sigma_px: (o.radius * cam_scale).max(0.0),
                offset_px: [
                    (a * ox + c * oy).round() as i32,
                    (b * ox + d * oy).round() as i32,
                ],
                tint: o.color,
                opacity: o.opacity,
                mode: o.mode,
            }
        })
        .collect()
}

/// **Que controle da pilha um id de painel endereça.** Os ids da seção são derivados por LINHA
/// (hashes de nome), então não há aritmética que os inverta: decodifica-se varrendo o teto.
///
/// Porta única de propósito — a ponte tem TRÊS sítios que perguntam *"este id é da pilha?"* (o
/// comando, o valor e o alvo do picker), e três varreduras escritas à mão divergiriam na primeira
/// linha nova.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FilterHit {
    /// "Add \<tipo\>" — põe um degrau novo no fim da pilha.
    Add(u8),
    /// ✕ — apaga a linha (a última apaga o componente).
    Remove(usize),
    /// ↑ / ↓ — a ORDEM é a feature.
    Up(usize),
    Down(usize),
    /// 👁 — desarma sem apagar.
    Hide(usize),
    /// A swatch de cor da linha (abre o picker OKLCH partilhado).
    Color(usize),
    /// O chip de MODO da linha (a LEI do degrau).
    Mode(usize, u8),
    /// Os sliders.
    Radius(usize),
    OffX(usize),
    OffY(usize),
    Opacity(usize),
}

/// Decodifica um id de painel para o controle da pilha que ele endereça.
pub(crate) fn hit_of(id: ph2d_editor::NodeId) -> Option<FilterHit> {
    use ph2d_editor::ids as vid;
    for k in 0..vid::MAX_FILTER_KINDS {
        if id == vid::filter_add_id(k) {
            #[allow(clippy::cast_possible_truncation)]
            return Some(FilterHit::Add(k as u8));
        }
    }
    for r in 0..vid::MAX_FILTER_ROWS {
        for m in 0..vid::MAX_FILTER_MODES {
            if id == vid::filter_mode_id(r, m) {
                #[allow(clippy::cast_possible_truncation)]
                return Some(FilterHit::Mode(r, m as u8));
            }
        }
        let hit = if id == vid::filter_remove_id(r) {
            FilterHit::Remove(r)
        } else if id == vid::filter_up_id(r) {
            FilterHit::Up(r)
        } else if id == vid::filter_down_id(r) {
            FilterHit::Down(r)
        } else if id == vid::filter_hide_id(r) {
            FilterHit::Hide(r)
        } else if id == vid::filter_color_id(r) {
            FilterHit::Color(r)
        } else if id == vid::filter_radius_id(r) {
            FilterHit::Radius(r)
        } else if id == vid::filter_offx_id(r) {
            FilterHit::OffX(r)
        } else if id == vid::filter_offy_id(r) {
            FilterHit::OffY(r)
        } else if id == vid::filter_opacity_id(r) {
            FilterHit::Opacity(r)
        } else {
            continue;
        };
        return Some(hit);
    }
    None
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

#[cfg(test)]
#[path = "fx_live_tests.rs"]
mod tests;
