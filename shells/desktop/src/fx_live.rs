//! **O FX raster VIVO na shell — 100% RESIDENTE NA GPU** (o `ph2d_ecs::VecFilter`, plano 24).
//!
//! Irmão do [`crate::offset_live`]/[`crate::contour_live`], mas de OUTRA natureza: o offset produz
//! GEOMETRIA (uma `LiveGeometry` que o `dispatch` desenha), e o FX produz PIXELS. A costura do
//! plano 24 §2: um FX raster não é `PathEffect` (não é `VecPath -> VecPath`) nem `LiveGeometry` —
//! ele isola a forma na própria textura, borra e recompõe.
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
//! 2. o [`FxBlurPass`] borra + tinge na GPU (Gaussiana separável) para uma textura de SAÍDA;
//! 3. essa textura é registrada no renderer PRINCIPAL por um **id ESTÁVEL** ([`VelloPass::register_texture`]);
//!    re-borrar escreve NA MESMA textura, então o Vello reusa o slot do atlas (zero churn de id, zero
//!    upload de CPU) e o `dispatch` a desenha no z da forma.
//!
//! O memo (por `spec + w + h + sigma`) vira OTIMIZAÇÃO — pula o re-blur quando nada muda —, não
//! requisito de correção: mesmo re-borrando todo frame (forma animada), é um render + 2 passes na
//! GPU, barato. Vazio = nenhum FX (byte-idêntico ao mundo pré-FX).

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecFilter};
use ph2d_gpu::GpuContext;
use ph2d_render::{FxBlurPass, VelloPass, make_output_texture};
use ph2d_vec_render::{FxImage, FxImages, FxMode, LiveGeometry};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};
use ph2d_vector::{Affine, Color, ImageData, StableImage, VectorScene};

use crate::vec_entities::VecEntityMap;

/// O maior lado de scratch/saída que pedimos à GPU — o `maxTextureDimension2D` baseline do WebGPU
/// (8192). Limite de RECURSO (a dimensão de textura garantida), não de gosto.
const MAX_FX_SIDE: u32 = 8192;

/// Os recursos de GPU PERSISTENTES de uma forma filtrada: a textura de saída (borrada) e o handle
/// [`ImageData`] estável que a referencia no renderer principal. `tex` fica viva para o re-blur e
/// para o Vello copiá-la no render (DEPOIS do recook).
struct PathFx {
    /// O handle registrado no renderer principal (id de Blob estável).
    image: ImageData,
    /// A textura de saída (o FX escreve aqui; o Vello copia daqui). Clone = handle, mesma GPU tex.
    tex: wgpu::Texture,
    w: u32,
    h: u32,
    sig_bits: u32,
    spec: VecFilter,
    mode: FxMode,
}

/// O cozimento de todos os FX raster da cena, GPU-resident. Runtime-only: o documento guarda a
/// RELAÇÃO (o `VecFilter` na entidade), e isto é o desenho derivado dela.
#[derive(Default)]
pub(crate) struct FxLive {
    live: FxImages,
    /// Renderer dedicado que rasteriza a forma ISOLADA (separado do principal para não pisar no
    /// intermediate da UI). Criado sob demanda no 1º FX da sessão.
    scratch: Option<VelloPass>,
    /// O passe de borrão de GPU. Build-once.
    blur: Option<FxBlurPass>,
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

        let [a, b, c, d, _, _] = camera.as_coeffs();
        let cam_scale = ((a * a + b * b).sqrt() + (c * c + d * d).sqrt()) as f32 * 0.5;

        let perf = std::env::var("PH2D_FX_PERF").is_ok();
        let t0 = std::time::Instant::now();
        let mut misses = 0usize;
        let mut seen: Vec<VecPathId> = Vec::new();

        for path in scene.paths() {
            let Some(spec) = spec_of(sim, map, path.id) else {
                continue;
            };
            let Some((x0, y0, x1, y1)) =
                ph2d_vec_render::path_screen_bounds(scene, xforms, live, path.id, camera)
            else {
                continue;
            };
            let sigma_px = (spec.radius * cam_scale).max(0.0);
            let margin = (3.0 * sigma_px as f64).ceil();
            let ex0 = (x0 - margin).floor();
            let ey0 = (y0 - margin).floor();
            let w = (((x1 + margin).ceil() - ex0).max(1.0) as u32).min(MAX_FX_SIDE);
            let h = (((y1 + margin).ceil() - ey0).max(1.0) as u32).min(MAX_FX_SIDE);
            let sig_bits = sigma_px.to_bits();
            seen.push(path.id);

            // O memo é otimização: re-borra só quando os PIXELS mudam.
            let cached = self.paths.get(&path.id);
            let hit = cached
                .is_some_and(|p| p.spec == spec && p.w == w && p.h == h && p.sig_bits == sig_bits);

            if !hit {
                misses += 1;
                if !self.recook_one(gpu, surface_format, vello_pass, scene, xforms, live, camera, path.id, spec, ex0, ey0, w, h, sigma_px, sig_bits) {
                    continue;
                }
            }

            let Some(pfx) = self.paths.get(&path.id) else {
                continue;
            };
            // A Drop Shadow desloca no COMPOSITE (vetor de mundo pela parte linear da câmera).
            let (sox, soy) = if spec.displaces() {
                let (ox, oy) = (f64::from(spec.offset[0]), f64::from(spec.offset[1]));
                (a * ox + c * oy, b * ox + d * oy)
            } else {
                (0.0, 0.0)
            };
            let rect = (
                ex0 + sox,
                ey0 + soy,
                ex0 + f64::from(pfx.w) + sox,
                ey0 + f64::from(pfx.h) + soy,
            );
            self.live.insert(
                path.id,
                FxImage {
                    // Clone do handle estável = MESMO id de Blob (o slot de atlas do Vello).
                    image: StableImage::from_image_data(pfx.image.clone()),
                    rect,
                    mode: pfx.mode,
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
                "[fx-perf] {} filtro(s), {misses} re-borrado(s), recook {ms:.3} ms",
                self.live.len()
            );
        }
        self.dbg_frames = self.dbg_frames.wrapping_add(1);
    }

    /// Cozinha UMA forma: rasteriza isolada no scratch, borra na GPU para a textura de saída
    /// (realoca + re-registra se o tamanho mudou), e atualiza o `PathFx`. `false` só em falha de
    /// recurso (pula a forma neste frame).
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
        spec: VecFilter,
        ex0: f64,
        ey0: f64,
        w: u32,
        h: u32,
        sigma_px: f32,
        sig_bits: u32,
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
                // ⇒ churn de id mínimo; o re-blur (comum) não re-registra.
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
                            sig_bits,
                            spec,
                            mode: fx_mode(spec),
                        },
                    );
                }
            }
        }

        // 3. Borra + tinge na GPU: scratch intermediate (premul) -> textura de saída (reta).
        let src = self.scratch.as_ref().expect("scratch ensured").intermediate_texture();
        let pfx = self.paths.get_mut(&id).expect("path ensured");
        pfx.sig_bits = sig_bits;
        pfx.spec = spec;
        pfx.mode = fx_mode(spec);
        let blur = self.blur.get_or_insert_with(|| FxBlurPass::new(gpu));
        blur.run(
            gpu,
            src,
            &pfx.tex,
            w,
            h,
            sigma_px,
            spec.kind,
            spec.color,
            spec.opacity,
        );
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

/// O modo de composição do filtro: Blur SUBSTITUI a forma; Glow/Drop Shadow entram ABAIXO dela.
fn fx_mode(spec: VecFilter) -> FxMode {
    if spec.tints() {
        FxMode::Below
    } else {
        FxMode::Replace
    }
}

/// O filtro de `id`, se houver. Porta única: o cozimento e o publish para o painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecFilter> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecFilter>(Entity::from_bits(bits))
        .copied()
}

/// **Arma** (ou remove) o filtro de cada caminho de `ids`. `None` remove o componente. Devolve
/// quantas entidades mudaram.
pub(crate) fn set_filter(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    ids: &[VecPathId],
    want: Option<VecFilter>,
) -> usize {
    let mut n = 0;
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let cur = sim.world().get::<VecFilter>(e).copied();
        if cur == want {
            continue;
        }
        let Ok(mut em) = sim.world_mut().get_entity_mut(e) else {
            continue;
        };
        match want {
            Some(v) => {
                em.insert(v);
            }
            None => {
                em.remove::<VecFilter>();
            }
        }
        n += 1;
    }
    n
}

/// **Edita** um campo do filtro de cada caminho de `ids` que JÁ tenha um (read-modify-write) — o
/// arrasto de um slider ou a cor do picker. Espelho de `contour_live::edit`.
pub(crate) fn edit(sim: &mut SimWorld, map: &VecEntityMap, ids: &[VecPathId], f: impl Fn(&mut VecFilter)) {
    for id in ids {
        let Some(&bits) = map.get(id) else { continue };
        let e = Entity::from_bits(bits);
        let Some(mut cur) = sim.world().get::<VecFilter>(e).copied() else {
            continue;
        };
        f(&mut cur);
        if let Ok(mut em) = sim.world_mut().get_entity_mut(e) {
            em.insert(cur);
        }
    }
}

/// O filtro que um chip de tipo recém-clicado deve armar, com defaults VISÍVEIS — armar no neutro
/// seria um clique que não muda um pixel.
pub(crate) fn default_for(kind: u8) -> VecFilter {
    match kind {
        VecFilter::GLOW => VecFilter {
            kind,
            radius: 0.18,
            offset: [0.0, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            opacity: 1.0,
        },
        VecFilter::DROP_SHADOW => VecFilter {
            kind,
            radius: 0.1,
            offset: [0.12, -0.12],
            color: [0.0, 0.0, 0.0, 1.0],
            opacity: 0.6,
        },
        _ => VecFilter {
            kind: VecFilter::BLUR,
            radius: 0.12,
            offset: [0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
            opacity: 1.0,
        },
    }
}
