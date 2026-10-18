//! **A PORTA LIVRE da luz do Impasto** (W3; 05 §2): [`super::PainterTool::apply_impasto_light`] é método do
//! estado do Impasto (lê os campos e o rig dele); o Wet Paint acende o relevo dos fios com a MESMA lei — o
//! `Rig::shade` do rig do artista, o corpo [`super::paint_body`]/[`super::gloss_body`] e o
//! [`super::light_pixel`] — sobre um texel qualquer, sem que o corpo do passo do Impasto mude.

use super::super::impasto_shade::{MatShade, Rig};
use ph2d_painter_brush::material::Material;

/// O rig do artista resolvido para a tinta neutra (o Wet Paint não tem material).
pub(in crate::tool::paint) struct LuzLivre {
    rig: Rig,
    mat: MatShade,
}

impl LuzLivre {
    /// `None` com todas as lâmpadas apagadas (a tela volta sem luz, ao byte).
    pub(in crate::tool::paint) fn nova(rig: &super::super::impasto_rig::LightRig) -> Option<Self> {
        let rig = Rig::new(rig)?;
        let mat = rig.resolve(Material::NEUTRAL.to_bytes());
        Some(Self { rig, mat })
    }

    /// Acende o texel `px` (RGBA sRGB8 direito) com o gradiente da altura `dhx, dhy` (unidades do campo do
    /// Impasto por texel, [`super::DEPTH_UNIT_PX`]) e a cobertura `cover` (`0..1`). Plano = intacto, ao byte.
    pub(in crate::tool::paint) fn acende(&self, dhx: f32, dhy: f32, cover: f32, px: &mut [u8]) {
        let albedo = [
            f32::from(px[0]) / 255.0,
            f32::from(px[1]) / 255.0,
            f32::from(px[2]) / 255.0,
        ];
        let (mul, add) = self.rig.shade(
            &self.mat,
            super::paint_body(cover),
            super::gloss_body(cover),
            dhx,
            dhy,
            albedo,
        );
        if mul == [1.0; 3] && add == [0.0; 3] {
            return;
        }
        for c in 0..3 {
            px[c] = (super::light_pixel(albedo[c], mul[c], add[c]) * 255.0 + 0.5) as u8;
        }
    }
}
