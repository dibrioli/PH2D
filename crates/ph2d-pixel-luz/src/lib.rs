//! ⭐ **O Pixel Sprite na luz 2D** (P14c-pixel 2, `docs/PixelArt/P14_plano.md` §4 L): a BANDA NORMAL
//! da faixa Pixel (a normal de `compose_normal` RODADA pela base da instância, desenhada com o MESMO
//! afim da cor), o `NaoRecebeLuz` por objecto (a sentinela [`SEM_LUZ`] na banda) e a camada de
//! oclusão ([`OclusaoDoPixel`], o alfa da cor no quadro mostrado). A família entrega cada objecto
//! ([`ObjetoPixel`]) já posto pela porta imagem → mundo (`ph2d_app_pixel::input`).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_ecs::Entity;
use ph2d_pixel_core::{FrameId, Sprite};
use ph2d_pixel_render::image_to_world;
use ph2d_vector::{Affine, ImageQuality, Point, StableImage, VectorScene};

/// Um Pixel Sprite como a luz o vê: o documento no quadro mostrado e o afim imagem → MUNDO (a porta
/// do pivô), mais a base local → mundo ([`base_da_instancia`]).
#[derive(Clone, Copy)]
pub struct ObjetoPixel<'a> {
    /// `Entity::to_bits` da entidade.
    pub bits: u64,
    pub sprite: &'a Sprite,
    pub quadro: FrameId,
    /// Muda a cada mudança visível do documento (a chave da imagem guardada).
    pub stamp: u64,
    pub img_mundo: Affine,
    pub lin: [f64; 4],
    /// `NaoRecebeLuz`.
    pub sem_luz: bool,
}

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
    for px in out.as_chunks_mut::<4>().0 {
        let (nx, ny, nz) = (dec(px[0]), dec(px[1]), dec(px[2]));
        let (mx, my) = (k * (d * nx - b * ny), k * (-c * nx + a * ny));
        let len = (mx * mx + my * my + nz * nz).sqrt().max(1e-9);
        px[0] = enc(mx / len);
        px[1] = enc(my / len);
        px[2] = enc(nz / len);
    }
    out
}

/// A assinatura de uma imagem normal já feita: versão do documento, quadro, base, «sem luz» e
/// «com o xadrez do Draw por baixo».
type Assinatura = (u64, u32, [u64; 4], bool, bool);

thread_local! {
    static NORMAIS: RefCell<BTreeMap<u64, (Assinatura, StableImage)>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// Algum objecto precisa da banda normal (tem uma camada Normal Map visível ou `sem_luz`)?
#[must_use]
pub fn precisa_da_banda(objetos: &[ObjetoPixel<'_>]) -> bool {
    objetos
        .iter()
        .any(|o| o.sem_luz || o.sprite.layers().iter().any(|l| l.normal && l.visible))
}

/// A imagem normal de `o` na banda (rodada para o mundo, ou a sentinela [`SEM_LUZ`]), guardada por
/// objecto enquanto a assinatura não muda.
#[must_use]
pub fn imagem_normal(o: &ObjetoPixel<'_>) -> Option<StableImage> {
    imagem_normal_de(o, false)
}

/// [`imagem_normal`] com `xadrez` = o editor pinta o xadrez de transparência por baixo do objecto
/// (o modo Draw): o texel de alfa 0 leva a sentinela [`SEM_LUZ`] OPACA — o xadrez fica fora da luz,
/// como o transparente fora do Draw (onde a cor é alfa 0 e a luz não tem o que mudar).
#[must_use]
pub fn imagem_normal_de(o: &ObjetoPixel<'_>, xadrez: bool) -> Option<StableImage> {
    let sig: Assinatura = (
        o.stamp,
        o.quadro.0,
        o.lin.map(f64::to_bits),
        o.sem_luz,
        xadrez,
    );
    NORMAIS.with(|n| {
        let mut n = n.borrow_mut();
        if let Some((s, img)) = n.get(&o.bits)
            && *s == sig
        {
            return Some(img.clone());
        }
        let mut px = ph2d_pixel_core::compose_normal(o.sprite, o.quadro);
        if o.sem_luz {
            for t in px.as_chunks_mut::<4>().0 {
                t[..3].copy_from_slice(&SEM_LUZ);
            }
        } else {
            px = normal_no_mundo(&px, o.lin);
        }
        if xadrez {
            for t in px.as_chunks_mut::<4>().0.iter_mut().filter(|t| t[3] == 0) {
                *t = [SEM_LUZ[0], SEM_LUZ[1], SEM_LUZ[2], 255];
            }
        }
        let (w, h) = (o.sprite.width(), o.sprite.height());
        let img = StableImage::from_rgba(Arc::new(px), w, h)?;
        n.insert(o.bits, (sig, img.clone()));
        Some(img)
    })
}

/// ⭐ **A banda normal** da faixa Pixel com `objetos` (de trás para a frente), com a câmera `cam`
/// (mundo → ecrã): o MESMO `cam · img_mundo` do desenho de cor. `None` = nenhum precisa dela (nada
/// é alocado). Os que não têm Normal Map entram de frente (tapam a normal de quem está atrás).
#[must_use]
pub fn banda_normal(objetos: &[ObjetoPixel<'_>], cam: Affine) -> Option<VectorScene> {
    banda_normal_no_draw(objetos, cam, None)
}

/// ⭐ [`banda_normal`] com `xadrez` = o objecto (`bits`) que o editor desenha em Draw, com o xadrez
/// de transparência por baixo da cor NA MESMA faixa: a colagem acenderia o xadrez (claro do lado da
/// lâmpada, a sombra do próprio sprite do outro). A banda nasce para ele mesmo sem Normal Map, e o
/// transparente dele vai fora da luz ([`imagem_normal_de`]).
#[must_use]
pub fn banda_normal_no_draw(
    objetos: &[ObjetoPixel<'_>],
    cam: Affine,
    xadrez: Option<u64>,
) -> Option<VectorScene> {
    let no_draw = |o: &ObjetoPixel<'_>| xadrez == Some(o.bits);
    if !precisa_da_banda(objetos) && !objetos.iter().any(no_draw) {
        return None;
    }
    let mut cena = VectorScene::new();
    for o in objetos {
        if let Some(img) = imagem_normal_de(o, no_draw(o)) {
            cena.draw_stable_image_transformed(&img, cam * o.img_mundo, ImageQuality::Low);
        }
    }
    Some(cena)
}

/// ⭐ **O Pixel Sprite que bloqueia** («outro» da luz): o alfa (e a cor, o albedo do rebote) da cor
/// composta no quadro mostrado, posto na camada da grade pela MESMA porta imagem → mundo.
pub struct OclusaoDoPixel<'a> {
    pub objetos: Vec<ObjetoPixel<'a>>,
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
        for o in self.objetos.iter().filter(|o| quem.contains(&o.bits)) {
            let img_mundo = o.img_mundo;
            if img_mundo.determinant().abs() < 1e-18 {
                continue;
            }
            let (sw, sh) = (o.sprite.width(), o.sprite.height());
            let mut cor = Vec::new();
            ph2d_pixel_core::compose_rgba(o.sprite, o.quadro, None, &mut cor);
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

#[cfg(test)]
#[path = "testes.rs"]
mod testes;

pub mod rampa;
pub use rampa::{BandaRampaDaFaixa, banda_rampa, banda_rampa_da_faixa};

#[cfg(test)]
#[path = "testes_rampa.rs"]
mod testes_rampa;
