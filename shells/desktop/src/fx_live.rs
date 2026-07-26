//! **O FX raster VIVO na shell** — o cozimento do [`ph2d_ecs::VecFilter`] (Blur/Glow/Drop Shadow).
//!
//! Irmão do [`crate::offset_live`]/[`crate::contour_live`], mas de OUTRA natureza: o offset produz
//! GEOMETRIA (uma `LiveGeometry` que o `dispatch` desenha), e o FX produz PIXELS (uma
//! [`ph2d_vec_render::FxImages`] que o `dispatch` injeta no z da forma). A costura é a do plano 24
//! §2: um FX raster não é um `PathEffect` (não é `VecPath -> VecPath`) nem uma `LiveGeometry` —
//! ele isola a forma na própria textura, borra/tinge, e recompõe.
//!
//! # O produtor: isolar → rasterizar → ler de volta → borrar → recompor
//!
//! Por forma filtrada, uma vez por frame (com memo):
//! 1. **onde** — [`ph2d_vec_render::path_screen_bounds`] dá o bbox de TELA da forma (honrando a
//!    geometria derivada e a pose, exatamente como o `dispatch` a desenha), expandido pela margem
//!    do borrão;
//! 2. **rasterizar** — [`ph2d_vec_render::draw_path_isolated`] desenha SÓ aquela forma num
//!    `VectorScene` scratch, transladada para a origem do scratch, e um [`VelloPass`] dedicado
//!    ([`FxLive::pass`], criado sob demanda) a rasteriza + lê de volta ([`VelloPass::render_and_readback`]);
//! 3. **borrar/tingir** — Gaussiana separável na CPU, em espaço PRÉ-MULTIPLICADO (o intermediate do
//!    Vello é premul — o blitter do [`VelloPass`] compõe com `PREMULTIPLIED_ALPHA_BLENDING`), e
//!    então o Glow/Drop Shadow trocam o RGB pela COR do efeito mantendo o alfa borrado;
//! 4. **recompor** — a [`ph2d_vec_render::FxImage`] entra no z da forma (`Below` para sombra/glow,
//!    `Replace` para o blur).
//!
//! O **raio é uma propriedade de MUNDO** (o `stdDev` em unidades da cena): a shell o converte para
//! px de tela pela escala da câmera, então dar zoom aumenta o borrão na tela — como deve ser.
//!
//! # O memo é MEDIDA, não otimização prematura
//!
//! O readback de GPU bloqueia a thread (o `map_async` + `poll`), e a Gaussiana na CPU é `O(área ·
//! kernel)`. Sem memo, uma forma filtrada PARADA pagaria os dois todo frame. A chave é o que de
//! facto determina os PIXELS: o `VecFilter`, o tamanho do scratch (`w×h`) e o `sigma_px` — a
//! POSIÇÃO na tela (o `rect`) é recomputada sempre (barata), então um PAN reusa a imagem e só um
//! ZOOM (que muda `w×h` e `sigma_px`) a re-cozinha. Vazio = nenhum FX (byte-idêntico ao pré-FX).

use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_ecs::{Entity, SimWorld, VecFilter};
use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_vec_render::{FxImage, FxImages, FxMode, LiveGeometry};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};
use ph2d_vector::{Affine, VectorScene};

use crate::vec_entities::VecEntityMap;

/// O maior lado de scratch que pedimos à GPU — o `maxTextureDimension2D` baseline do WebGPU
/// (8192). É um limite de RECURSO (a dimensão de textura que o dispositivo garante), não um teto de
/// gosto: uma forma cujo bbox+borrão em tela exceda isto (zoom gigante × raio gigante) é clampada e
/// a imagem sai cortada, o que é honesto e raro.
const MAX_FX_SIDE: u32 = 8192;

/// Uma entrada do memo: o que determina os PIXELS (spec + tamanho + sigma) e a imagem produzida.
struct MemoFx {
    spec: VecFilter,
    w: u32,
    h: u32,
    sig_bits: u32,
    rgba: Arc<Vec<u8>>,
    mode: FxMode,
}

/// O cozimento de todos os FX raster da cena. Runtime-only: o documento guarda a RELAÇÃO (o
/// `VecFilter` na entidade), e isto é o desenho derivado dela.
#[derive(Default)]
pub(crate) struct FxLive {
    live: FxImages,
    /// O rasterizador dedicado do FX — criado sob demanda no 1º FX da sessão (um `Renderer` do
    /// Vello é caro), depois reusado (o intermediate é redimensionado por forma).
    pass: Option<VelloPass>,
    memo: BTreeMap<VecPathId, MemoFx>,
}

impl FxLive {
    /// As imagens de FX deste frame — o que o [`ph2d_vec_render::dispatch`] injeta no z das formas.
    /// Vazio = nenhum FX na cena (o desenho é o de sempre, byte-idêntico ao mundo pré-FX).
    pub(crate) fn images(&self) -> &FxImages {
        &self.live
    }

    /// Re-coze o FX de cada forma filtrada. Chamado uma vez por frame, DEPOIS do `sync` (senão uma
    /// forma recém-criada ainda não tem entidade e o `VecFilter` dela não seria encontrado) e
    /// DEPOIS dos outros `recook` (o FX honra a geometria DERIVADA via `live`).
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
    ) {
        self.live.clear();
        // A escala da câmera: o raio de MUNDO vira px de tela por ela (zoom aumenta o borrão).
        let [a, b, c, d, _, _] = camera.as_coeffs();
        let cam_scale = ((a * a + b * b).sqrt() + (c * c + d * d).sqrt()) as f32 * 0.5;

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

            let hit = self.memo.get(&path.id).is_some_and(|m| {
                m.spec == spec && m.w == w && m.h == h && m.sig_bits == sig_bits
            });
            if !hit {
                // Rasteriza a forma isolada num scratch local (a Scene do Vello é um buffer de
                // comandos — barata de criar por forma), transladada para a origem do scratch.
                let mut scratch = VectorScene::new();
                ph2d_vec_render::draw_path_isolated(
                    scene,
                    xforms,
                    live,
                    path.id,
                    camera,
                    Affine::translate((-ex0, -ey0)),
                    &mut scratch,
                );
                let pass = match self.pass.as_mut() {
                    Some(p) => p,
                    None => match VelloPass::new(gpu, surface_format, (w, h)) {
                        Ok(p) => {
                            self.pass = Some(p);
                            self.pass.as_mut().unwrap()
                        }
                        Err(e) => {
                            eprintln!("[fx] VelloPass::new: {e}");
                            continue;
                        }
                    },
                };
                let readback = match pass.render_and_readback(gpu, scratch.inner(), (w, h)) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[fx] render_and_readback: {e}");
                        continue;
                    }
                };
                let (rgba, mode) = process_fx(&readback, w, h, sigma_px, &spec);
                self.memo.insert(
                    path.id,
                    MemoFx {
                        spec,
                        w,
                        h,
                        sig_bits,
                        rgba,
                        mode,
                    },
                );
            }

            let Some(m) = self.memo.get(&path.id) else {
                continue;
            };
            // O deslocamento da Drop Shadow é de MUNDO ⇒ sobe pela PARTE LINEAR da câmera (sem a
            // translação): é um vetor, não um ponto.
            let (sox, soy) = if spec.displaces() {
                let (ox, oy) = (f64::from(spec.offset[0]), f64::from(spec.offset[1]));
                (a * ox + c * oy, b * ox + d * oy)
            } else {
                (0.0, 0.0)
            };
            let rect = (
                ex0 + sox,
                ey0 + soy,
                ex0 + f64::from(m.w) + sox,
                ey0 + f64::from(m.h) + soy,
            );
            self.live.insert(
                path.id,
                FxImage {
                    rgba: m.rgba.clone(),
                    width: m.w,
                    height: m.h,
                    rect,
                    mode: m.mode,
                },
            );
        }
        // O memo não pode sobreviver ao componente: uma forma que perdeu o filtro (remove, Ctrl+Z)
        // manteria a imagem velha.
        self.memo.retain(|id, _| self.live.contains_key(id));
    }

    /// Esquece tudo — o load de projeto e o restore de undo trocam a cena inteira debaixo do
    /// cozimento, e os `VecPathId` são reciclados entre documentos (a lei do `offset_live::forget`).
    pub(crate) fn forget(&mut self) {
        self.live.clear();
        self.memo.clear();
    }
}

/// O filtro de `id`, se houver. Porta única: o cozimento e o publish para o painel perguntam AQUI.
pub(crate) fn spec_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecFilter> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecFilter>(Entity::from_bits(bits))
        .copied()
}

/// **Arma** (ou remove) o filtro de cada caminho de `ids`. `None` remove o componente — um
/// documento não acumula relações que não desenham nada. Devolve quantas entidades mudaram.
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
/// arrasto de um slider (Radius / Offset / Opacity) ou a cor do picker. Caminho sem filtro é
/// ignorado (o slider só existe quando há filtro). Espelho de `contour_live::edit`.
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
/// seria um clique que não muda um pixel (a lição do `Add Contour`). Blur/Glow nascem com raio; a
/// Drop Shadow nasce deslocada e a meia-opacidade (a sombra padrão).
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

/// Converte a leitura de GPU (premul sRGB) na imagem de FX pronta para o `draw_image_rgba` (alfa
/// RETO). Blur = a forma borrada, des-premultiplicada; Glow/Drop Shadow = a COR do efeito com o
/// alfa borrado da silhueta (a cor da forma é descartada — só a cobertura importa).
fn process_fx(readback: &[u8], w: u32, h: u32, sigma_px: f32, spec: &VecFilter) -> (Arc<Vec<u8>>, FxMode) {
    let n = (w as usize) * (h as usize);
    // premul sRGB u8 -> f32 [0,1]
    let mut buf = vec![0.0f32; n * 4];
    for (dst, &src) in buf.iter_mut().zip(readback.iter()) {
        *dst = f32::from(src) / 255.0;
    }
    let blurred = blur_premul(&buf, w as usize, h as usize, sigma_px);
    let op = spec.opacity.clamp(0.0, 1.0);
    let mut out = vec![0u8; n * 4];

    if spec.tints() {
        let tr = spec.color[0].clamp(0.0, 1.0);
        let tg = spec.color[1].clamp(0.0, 1.0);
        let tb = spec.color[2].clamp(0.0, 1.0);
        let ta = spec.color[3].clamp(0.0, 1.0);
        for i in 0..n {
            // O alfa borrado é a silhueta; a cor é a do efeito, RETA (o draw_image premultiplica).
            let a = blurred[i * 4 + 3] * ta * op;
            out[i * 4] = to_u8(tr);
            out[i * 4 + 1] = to_u8(tg);
            out[i * 4 + 2] = to_u8(tb);
            out[i * 4 + 3] = to_u8(a);
        }
        (Arc::new(out), FxMode::Below)
    } else {
        for i in 0..n {
            let a = blurred[i * 4 + 3];
            // des-premultiplica: o draw_image espera RGBA reto.
            let inv = if a > 1e-4 { 1.0 / a } else { 0.0 };
            out[i * 4] = to_u8(blurred[i * 4] * inv);
            out[i * 4 + 1] = to_u8(blurred[i * 4 + 1] * inv);
            out[i * 4 + 2] = to_u8(blurred[i * 4 + 2] * inv);
            out[i * 4 + 3] = to_u8(a * op);
        }
        (Arc::new(out), FxMode::Replace)
    }
}

#[inline]
fn to_u8(x: f32) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Gaussiana separável em espaço PRÉ-MULTIPLICADO (RGB e alfa juntos — é a forma correta de filtrar
/// uma imagem com transparência). `sigma <= 0.01` ⇒ cópia (borrão nulo). Bordas por clamp (a
/// margem já é transparente).
fn blur_premul(src: &[f32], w: usize, h: usize, sigma: f32) -> Vec<f32> {
    if sigma <= 0.01 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let kernel = gaussian_kernel(sigma);
    let r = (kernel.len() / 2) as i32;
    let mut tmp = vec![0.0f32; src.len()];
    // horizontal
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (ki, &wt) in kernel.iter().enumerate() {
                let sx = (x as i32 + ki as i32 - r).clamp(0, w as i32 - 1) as usize;
                let o = (y * w + sx) * 4;
                for (c, a) in acc.iter_mut().enumerate() {
                    *a += src[o + c] * wt;
                }
            }
            let o = (y * w + x) * 4;
            tmp[o..o + 4].copy_from_slice(&acc);
        }
    }
    // vertical
    let mut out = vec![0.0f32; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 4];
            for (ki, &wt) in kernel.iter().enumerate() {
                let sy = (y as i32 + ki as i32 - r).clamp(0, h as i32 - 1) as usize;
                let o = (sy * w + x) * 4;
                for (c, a) in acc.iter_mut().enumerate() {
                    *a += tmp[o + c] * wt;
                }
            }
            let o = (y * w + x) * 4;
            out[o..o + 4].copy_from_slice(&acc);
        }
    }
    out
}

/// Kernel Gaussiano normalizado, raio `ceil(3·sigma)` (suporte de 99,7%).
fn gaussian_kernel(sigma: f32) -> Vec<f32> {
    let r = (3.0 * sigma).ceil().max(1.0) as i32;
    let two_s2 = 2.0 * sigma * sigma;
    let mut k: Vec<f32> = (-r..=r)
        .map(|i| (-((i * i) as f32) / two_s2).exp())
        .collect();
    let sum: f32 = k.iter().sum();
    if sum > 0.0 {
        for w in &mut k {
            *w /= sum;
        }
    }
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um degrau opaco→transparente vira uma RAMPA monótona centrada na fronteira, e a largura da
    /// rampa cresce com o `sigma`. É a propriedade que separa um borrão de um simples corte de alfa
    /// (a queixa que o produto existe para responder).
    #[test]
    fn a_step_edge_becomes_a_monotone_ramp_that_widens_with_sigma() {
        // 64×1 premul: metade esquerda branca opaca, metade direita transparente.
        let w = 64usize;
        let mut src = vec![0.0f32; w * 4];
        for x in 0..w / 2 {
            src[x * 4..x * 4 + 4].copy_from_slice(&[1.0, 1.0, 1.0, 1.0]);
        }
        let alpha = |buf: &[f32]| -> Vec<f32> { (0..w).map(|x| buf[x * 4 + 3]).collect() };
        let ramp_width = |a: &[f32]| -> usize {
            // Nº de amostras entre 10% e 90% de cobertura (a "rampa").
            a.iter().filter(|&&v| v > 0.1 && v < 0.9).count()
        };

        let narrow = alpha(&blur_premul(&src, w, 1, 2.0));
        let wide = alpha(&blur_premul(&src, w, 1, 6.0));

        // Monótona não-crescente da esquerda (opaca) para a direita (transparente).
        for a in [&narrow, &wide] {
            for pair in a.windows(2) {
                assert!(pair[1] <= pair[0] + 1e-4, "ramp não é monótona: {a:?}");
            }
            // Centrada: ~0.5 na fronteira (índice 31/32).
            assert!((a[w / 2 - 1] - 0.5).abs() < 0.12, "fronteira não está em ~0.5");
        }
        // Sigma maior ⇒ rampa mais larga (a propriedade que o borrão É).
        assert!(
            ramp_width(&wide) > ramp_width(&narrow),
            "sigma 6 ({}) deveria alargar mais que sigma 2 ({})",
            ramp_width(&wide),
            ramp_width(&narrow),
        );
        // Sigma nulo ⇒ cópia byte-idêntica (borrão inerte).
        assert_eq!(blur_premul(&src, w, 1, 0.0), src);
    }

    /// O Glow/Drop Shadow descartam a COR da forma e pintam a do efeito com o alfa borrado da
    /// silhueta; o Blur preserva a cor (des-premultiplicada). É a diferença de MODO que o
    /// [`FxMode`] carrega.
    #[test]
    fn tint_paints_the_effect_colour_blur_keeps_the_shape_colour() {
        // 8×8 premul: um bloco central branco opaco (a forma é branca).
        let (w, h) = (8u32, 8u32);
        let mut rb = vec![0u8; (w * h * 4) as usize];
        for y in 2..6 {
            for x in 2..6 {
                let o = ((y * w + x) * 4) as usize;
                rb[o..o + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
        let glow = VecFilter {
            kind: VecFilter::GLOW,
            radius: 0.0,
            offset: [0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0], // vermelho
            opacity: 1.0,
        };
        let (rgba, mode) = process_fx(&rb, w, h, 0.0, &glow);
        assert_eq!(mode, FxMode::Below);
        // No miolo coberto: cor do EFEITO (vermelho), não a branca da forma.
        let o = ((3 * w + 3) * 4) as usize;
        assert_eq!(&rgba[o..o + 3], &[255, 0, 0], "glow não pintou a cor do efeito");
        assert!(rgba[o + 3] > 200, "glow perdeu a cobertura da silhueta");

        let blur = VecFilter {
            kind: VecFilter::BLUR,
            radius: 0.0,
            offset: [0.0, 0.0],
            color: [1.0, 0.0, 0.0, 1.0],
            opacity: 1.0,
        };
        let (rgba, mode) = process_fx(&rb, w, h, 0.0, &blur);
        assert_eq!(mode, FxMode::Replace);
        // O blur preserva a cor da forma (branca), ignorando a cor do filtro.
        assert_eq!(&rgba[o..o + 3], &[255, 255, 255], "blur não preservou a cor da forma");
    }
}
