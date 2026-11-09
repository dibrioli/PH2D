//! ⭐ **O Pixel Sprite na luz 2D** (P14c-pixel 2, `docs/PixelArt/P14_plano.md` §4 L): a BANDA NORMAL
//! da faixa Pixel (a normal de `compose_normal` RODADA pela base da instância, desenhada com o MESMO
//! afim da cor), o `NaoRecebeLuz` por objecto (a sentinela [`SEM_LUZ`] na banda) e a camada de
//! oclusão ([`OclusaoDoPixel`], o alfa da cor no `quadro_mostrado`).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_ecs::Entity;
use ph2d_pixel_render::image_to_world;
use ph2d_vector::{Affine, ImageQuality, Point, StableImage, VectorScene};

use crate::PixelApp;
use crate::input::{key_of, origem_px, to_view};

/// A cor da banda normal de um objecto com `NaoRecebeLuz`: `b = 0` (z = −1) nunca sai de uma normal
/// do hemisfério da frente (`b ≥ 128`); a colagem lê `b < ¼` como «fora da luz».
pub const SEM_LUZ: [u8; 3] = [128, 128, 0];

/// A parte linear `[a, b, c, d]` (kurbo: `x' = a·x + c·y`) do LOCAL → MUNDO da entidade (Y para
/// cima nos dois): rotação, inclinação e escala, sem o `ppm` nem a inversão do Y da imagem.
#[must_use]
pub fn base_da_instancia(world: &ph2d_ecs::World, e: Entity) -> Option<[f64; 4]> {
    let tr = ph2d_ecs::world_transform(world, e)?;
    let img = image_to_world(
        (0.0, 0.0),
        1.0,
        (0.0, 0.0),
        f64::from(tr.rotation),
        (f64::from(tr.scale.x), f64::from(tr.scale.y)),
        (f64::from(tr.skew_x), f64::from(tr.skew_y)),
    );
    let [a, b, c, d, _, _] = img.as_coeffs();
    // `image_to_world` = L · diag(1, −1) com ppm 1 ⇒ L = as colunas com a segunda trocada de sinal.
    Some([a, b, -c, -d])
}

/// ⭐ A normal da imagem (rgb = `n·½+½`, Y para cima no sprite) levada aos eixos do MUNDO pela base
/// `lin` ([`base_da_instancia`]): `n_xy ← L⁻ᵀ·n_xy·√|det L|` (a inversa transposta, sem a escala
/// uniforme), renormalizada com o `n_z`. Rodar 90° leva `+x` a `+y`; espelhar (`det < 0`) vira-a.
#[must_use]
pub fn normal_no_mundo(rgba: &[u8], lin: [f64; 4]) -> Vec<u8> {
    let [a, b, c, d] = lin;
    let det = a * d - b * c;
    let mut out = rgba.to_vec();
    if det.abs() < 1e-12 {
        return out;
    }
    let k = det.abs().sqrt() / det;
    let dec = |v: u8| f64::from(v) / 255.0 * 2.0 - 1.0;
    let enc = |v: f64| ((v * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
    for px in out.chunks_exact_mut(4) {
        let (nx, ny, nz) = (dec(px[0]), dec(px[1]), dec(px[2]));
        let (mx, my) = (k * (d * nx - b * ny), k * (-c * nx + a * ny));
        let len = (mx * mx + my * my + nz * nz).sqrt().max(1e-9);
        px[0] = enc(mx / len);
        px[1] = enc(my / len);
        px[2] = enc(nz / len);
    }
    out
}

/// A assinatura de uma imagem normal já feita: versão do documento, quadro, base e «sem luz».
type Assinatura = (u64, u32, [u64; 4], bool);

thread_local! {
    static NORMAIS: RefCell<BTreeMap<u64, (Assinatura, StableImage)>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// Algum Pixel Sprite de `ids` precisa da banda normal (tem uma camada Normal Map ou `sem_luz`)?
#[must_use]
pub fn precisa_da_banda(
    app: &PixelApp,
    world: &ph2d_ecs::World,
    ids: &[u64],
    sem_luz: &[Entity],
) -> bool {
    ids.iter().any(|&bits| {
        let Some(doc) = key_of(world, bits).and_then(|k| app.docs.get(&k)) else {
            return false;
        };
        sem_luz.iter().any(|e| e.to_bits() == bits)
            || doc.sprite.layers().iter().any(|l| l.normal && l.visible)
    })
}

/// ⭐ **A banda normal** da faixa Pixel com os objectos `ids` (de trás para a frente), com a câmera
/// `cam` — o MESMO afim do desenho de cor ([`to_view`]). `None` = nenhum precisa dela (nada é
/// alocado). Os que não têm Normal Map entram de frente (tapam a normal de quem está atrás).
#[must_use]
pub fn banda_normal(
    app: &PixelApp,
    world: &ph2d_ecs::World,
    cam: Affine,
    ppm: f32,
    ids: &[u64],
    sem_luz: &[Entity],
) -> Option<VectorScene> {
    if !precisa_da_banda(app, world, ids, sem_luz) {
        return None;
    }
    let mut cena = VectorScene::new();
    for &bits in ids {
        let (Some(key), Some(e)) = (key_of(world, bits), Entity::try_from_bits(bits)) else {
            continue;
        };
        if ph2d_ecs::visibility::is_hidden_in_tree(world, e) {
            continue;
        }
        let Some(doc) = app.docs.get(&key) else {
            continue;
        };
        let (Some(m), Some(lin)) = (
            to_view(world, bits, doc, cam, ppm),
            base_da_instancia(world, e),
        ) else {
            continue;
        };
        let quadro = ph2d_pixel_cena::quadro_mostrado(world, e, &doc.sprite, doc.frame);
        let fora = sem_luz.iter().any(|s| s.to_bits() == bits);
        let sig: Assinatura = (doc.stamp, quadro.0, lin.map(f64::to_bits), fora);
        let (w, h) = (doc.sprite.width(), doc.sprite.height());
        let img = NORMAIS.with(|n| {
            let mut n = n.borrow_mut();
            if let Some((s, img)) = n.get(&bits)
                && *s == sig
            {
                return Some(img.clone());
            }
            let mut px = ph2d_pixel_core::compose_normal(&doc.sprite, quadro);
            if fora {
                for t in px.chunks_exact_mut(4) {
                    t[..3].copy_from_slice(&SEM_LUZ);
                }
            } else {
                px = normal_no_mundo(&px, lin);
            }
            let img = StableImage::from_rgba(Arc::new(px), w, h)?;
            n.insert(bits, (sig, img.clone()));
            Some(img)
        });
        if let Some(img) = img {
            cena.draw_stable_image_transformed(&img, m, ImageQuality::Low);
        }
    }
    Some(cena)
}

/// ⭐ **O Pixel Sprite que bloqueia** («outro» da luz): o alfa (e a cor, o albedo do rebote) da cor
/// composta no `quadro_mostrado`, posto na camada da grade pela MESMA porta imagem → mundo.
pub struct OclusaoDoPixel<'a> {
    pub app: &'a PixelApp,
    pub world: &'a ph2d_ecs::World,
    pub ppm: f32,
}

fn linear(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb(v: f64) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let e = if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (e * 255.0).round() as u8
}

impl OclusaoDoPixel<'_> {
    /// Os bytes (`FORMATO_DA_OCLUSAO`: sRGB, premultiplicado; linha 0 = `y1`) da camada `tamanho`
    /// sobre a janela `mundo` com os Pixel Sprites de `quem`; `None` = nenhum desenhado.
    #[must_use]
    pub fn rasterizar(&self, (w, h): (u32, u32), mundo: [f64; 4], quem: &[u64]) -> Option<Vec<u8>> {
        let [x0, y0, x1, y1] = mundo;
        let (pw, ph) = ((x1 - x0) / f64::from(w), (y1 - y0) / f64::from(h));
        let mut px = vec![0_u8; (w as usize) * (h as usize) * 4];
        let mut desenhou = false;
        for &bits in quem {
            let (Some(key), Some(e)) = (key_of(self.world, bits), Entity::try_from_bits(bits))
            else {
                continue;
            };
            let Some(doc) = self.app.docs.get(&key) else {
                continue;
            };
            let Some(tr) = ph2d_ecs::world_transform(self.world, e) else {
                continue;
            };
            let img_mundo = image_to_world(
                origem_px(self.world, e, doc),
                f64::from(self.ppm),
                (f64::from(tr.translation.x), f64::from(tr.translation.y)),
                f64::from(tr.rotation),
                (f64::from(tr.scale.x), f64::from(tr.scale.y)),
                (f64::from(tr.skew_x), f64::from(tr.skew_y)),
            );
            if img_mundo.determinant().abs() < 1e-18 {
                continue;
            }
            let (sw, sh) = (doc.sprite.width(), doc.sprite.height());
            let quadro = ph2d_pixel_cena::quadro_mostrado(self.world, e, &doc.sprite, doc.frame);
            let mut cor = Vec::new();
            ph2d_pixel_core::compose_rgba(&doc.sprite, quadro, None, &mut cor);
            // A caixa do sprite na camada (os quatro cantos), para não varrer a grade inteira.
            let cantos = [
                (0.0, 0.0),
                (f64::from(sw), 0.0),
                (0.0, f64::from(sh)),
                (f64::from(sw), f64::from(sh)),
            ]
            .map(|(x, y)| img_mundo * Point::new(x, y));
            let (mut cx0, mut cx1, mut cy0, mut cy1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
            for p in cantos {
                (cx0, cx1, cy0, cy1) = (cx0.min(p.x), cx1.max(p.x), cy0.min(p.y), cy1.max(p.y));
            }
            let i0 = ((cx0 - x0) / pw).floor().max(0.0) as u32;
            let i1 = (((cx1 - x0) / pw).ceil().max(0.0) as u32).min(w);
            let j0 = ((y1 - cy1) / ph).floor().max(0.0) as u32;
            let j1 = (((y1 - cy0) / ph).ceil().max(0.0) as u32).min(h);
            let inv = img_mundo.inverse();
            for j in j0..j1 {
                for i in i0..i1 {
                    let p = Point::new(
                        x0 + (f64::from(i) + 0.5) * pw,
                        y1 - (f64::from(j) + 0.5) * ph,
                    );
                    let q = inv * p;
                    let (tx, ty) = (q.x.floor(), q.y.floor());
                    if tx < 0.0 || ty < 0.0 || tx >= f64::from(sw) || ty >= f64::from(sh) {
                        continue;
                    }
                    let t = ((ty as usize) * (sw as usize) + tx as usize) * 4;
                    let Some(c) = cor.get(t..t + 4) else {
                        continue;
                    };
                    if c[3] == 0 {
                        continue;
                    }
                    let a = f64::from(c[3]) / 255.0;
                    let k = ((j as usize) * (w as usize) + i as usize) * 4;
                    for canal in 0..3 {
                        px[k + canal] = srgb(linear(c[canal]) * a);
                    }
                    px[k + 3] = c[3];
                    desenhou = true;
                }
            }
        }
        desenhou.then_some(px)
    }
}

impl ph2d_light2d_gpu::CamadaDeOclusao for OclusaoDoPixel<'_> {
    fn desenhar(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        alvo: &wgpu::TextureView,
        tamanho: (u32, u32),
        mundo: [f64; 4],
        quem: &[u64],
    ) -> bool {
        let Some(px) = self.rasterizar(tamanho, mundo, quem) else {
            return false;
        };
        gpu.queue.write_texture(
            alvo.texture().as_image_copy(),
            &px,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(tamanho.0 * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: tamanho.0,
                height: tamanho.1,
                depth_or_array_layers: 1,
            },
        );
        true
    }
}

/// A banda normal de cada faixa do quadro (índice = o da cena da faixa) e a textura onde ela pousa,
/// alocada só quando alguma faixa a tem.
#[derive(Default)]
pub struct NormaisDasFaixas {
    pub cenas: Vec<Option<VectorScene>>,
    textura: Option<(wgpu::Texture, wgpu::TextureView, (u32, u32))>,
}

impl NormaisDasFaixas {
    /// Alguma faixa deste quadro tem banda normal (⇒ `CenaDeLuz.com_normal` do que a luz grava)?
    #[must_use]
    pub fn alguma(&self) -> bool {
        self.cenas.iter().any(Option::is_some)
    }

    /// Os bytes da textura da banda (0 = nada alocado: a guarda G6).
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.textura
            .as_ref()
            .map_or(0, |(_, _, (w, h))| u64::from(*w) * u64::from(*h) * 4)
    }

    /// ⭐ Cola a faixa `i` (`cena`, a cor) sobre `alvo`: com banda normal, ela vai primeiro ao
    /// intermediário do Vello e é copiada para a textura da banda; a cor depois, pela colagem com
    /// a normal. Sem ela, a colagem de sempre.
    pub fn colar(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        vello: &mut ph2d_render::VelloPass,
        colagem: &ph2d_render::BandBlit,
        alvo: &wgpu::TextureView,
        (i, cena): (usize, &VectorScene),
        tamanho: (u32, u32),
    ) {
        let transparente = ph2d_vector::Color::TRANSPARENT;
        let normal = self.cenas.get_mut(i).and_then(Option::take);
        let vista = normal.as_ref().map(|n| {
            if let Err(e) = vello.render_to_intermediate(gpu, n.inner(), tamanho, transparente) {
                eprintln!("[zorder] banda normal falhou: {e}");
            }
            let (tex, vista) = self.textura_de(gpu, tamanho, vello.intermediate_texture().format());
            let mut enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ph2d-app-pixel banda normal"),
                });
            enc.copy_texture_to_texture(
                vello.intermediate_texture().as_image_copy(),
                tex.as_image_copy(),
                wgpu::Extent3d {
                    width: tamanho.0,
                    height: tamanho.1,
                    depth_or_array_layers: 1,
                },
            );
            gpu.queue.submit([enc.finish()]);
            vista
        });
        if let Some(n) = normal {
            self.cenas[i] = Some(n);
        }
        if let Err(e) = vello.render_to_intermediate(gpu, cena.inner(), tamanho, transparente) {
            eprintln!("[zorder] faixa de vetor falhou: {e}");
        }
        let fonte = ph2d_render::BandSource::Vector;
        match vista {
            Some(n) => {
                colagem.blit_com_normal(gpu, alvo, vello.intermediate_view(), &n, fonte, None)
            }
            None => colagem.blit(gpu, alvo, vello.intermediate_view(), fonte),
        }
        if !self.alguma() {
            self.textura = None;
        }
    }

    fn textura_de(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        tamanho: (u32, u32),
        formato: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        if self
            .textura
            .as_ref()
            .is_none_or(|(t, _, s)| *s != tamanho || t.format() != formato)
        {
            let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ph2d-app-pixel banda normal"),
                size: wgpu::Extent3d {
                    width: tamanho.0,
                    height: tamanho.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: formato,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.textura = Some((tex, vista, tamanho));
        }
        let (t, v, _) = self.textura.as_ref().expect("acabada de criar");
        (t.clone(), v.clone())
    }
}

#[cfg(test)]
#[path = "luz_tests.rs"]
mod testes;
