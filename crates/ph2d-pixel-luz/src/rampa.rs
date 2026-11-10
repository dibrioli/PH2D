//! ⭐ **A banda de RAMPA** da faixa Pixel (P13 L, `docs/PixelArt/P13_plano.md` §4.L), irmã da
//! [`crate::banda_normal`]: cada texel de rampa de um objecto com «Light: Palette» sai
//! `(rampa, degrau, 0, 255)`, com a rampa numerada na [`LutDaLuz`] da FAIXA (as rampas de cada
//! objecto, de trás para a frente, umas a seguir às outras). Opaco fora de rampa sai com `a = 254`
//! (tapa quem está atrás e a colagem lê a lei de hoje); a colagem só lê `a == 255`.

use std::sync::Arc;

use ph2d_pixel_rampa::luz::LutDaLuz;
use ph2d_vector::{Affine, ImageQuality, StableImage, VectorScene};

use crate::ObjetoPixel;

/// O alfa do texel opaco que NÃO é de rampa (tapa, mas não troca de degrau).
pub const FORA_DA_RAMPA: u8 = 254;

/// As linhas que a LUT aguenta (o id da rampa é um byte).
pub const MAX_RAMPAS_POR_FAIXA: usize = 256;

/// A imagem da banda de rampa de `o`, com as rampas a começar na linha `base` da LUT.
#[must_use]
pub fn imagem_rampa(o: &ObjetoPixel<'_>, lut: &LutDaLuz, base: usize) -> Option<StableImage> {
    let px = texeis_rampa(o, lut, base);
    StableImage::from_rgba(Arc::new(px), o.sprite.width(), o.sprite.height())
}

/// Os texels RGBA8 de [`imagem_rampa`].
#[must_use]
pub fn texeis_rampa(o: &ObjetoPixel<'_>, lut: &LutDaLuz, base: usize) -> Vec<u8> {
    let mut px = Vec::new();
    ph2d_pixel_core::compose_rgba(o.sprite, o.quadro, None, &mut px);
    for t in px.as_chunks_mut::<4>().0 {
        *t = match (t[3], lut.degrau_de(*t)) {
            (0, _) => [0; 4],
            (_, Some((s, p))) if base + s < MAX_RAMPAS_POR_FAIXA => {
                [(base + s) as u8, p as u8, 0, 255]
            }
            _ => [0, 0, 0, FORA_DA_RAMPA],
        };
    }
    px
}

/// ⭐ A banda de rampa e a LUT da faixa com `objetos` (de trás para a frente; `paleta[i]` = o
/// objecto `i` acende na rampa), com o MESMO `cam · img_mundo` da cor. `None` = nenhum objecto com
/// «Light: Palette» tem rampas (nada é alocado: o custo das cenas sem ele continua zero).
#[must_use]
pub fn banda_rampa(
    objetos: &[ObjetoPixel<'_>],
    paleta: &[bool],
    cam: Affine,
) -> Option<(VectorScene, LutDaLuz)> {
    let luts: Vec<Option<LutDaLuz>> = objetos
        .iter()
        .zip(paleta)
        .map(|(o, &p)| {
            let lut = LutDaLuz::da_paleta(o.sprite.palette(), o.sprite.ramps());
            (p && !lut.rampas.is_empty()).then_some(lut)
        })
        .collect();
    if luts.iter().all(Option::is_none) {
        return None;
    }
    let mut cena = VectorScene::new();
    let mut faixa = LutDaLuz::default();
    for (o, lut) in objetos.iter().zip(&luts) {
        let vazia = LutDaLuz::default();
        let lut = lut.as_ref().unwrap_or(&vazia);
        if let Some(img) = imagem_rampa(o, lut, faixa.rampas.len()) {
            cena.draw_stable_image_transformed(&img, cam * o.img_mundo, ImageQuality::Low);
        }
        faixa.rampas.extend(lut.rampas.iter().cloned());
    }
    faixa.rampas.truncate(MAX_RAMPAS_POR_FAIXA);
    Some((cena, faixa))
}
