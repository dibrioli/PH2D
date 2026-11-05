//! O [`crate::material::Material`] do pincel e a referência do Grain aplicado (filho de [`super`], LOC cap).

use super::*;

impl BrushSpec {
    /// This brush's [`crate::material::Material`] — what its paint IS, as opposed to what shape it
    /// leaves. The one place the four fields are read together, so a knob added later cannot be
    /// forgotten by half the pipeline.
    #[must_use]
    pub fn material(&self) -> crate::material::Material {
        crate::material::Material {
            shine: self.impasto_shine,
            roughness: self.impasto_roughness,
            metallic: self.impasto_metallic,
            wax: self.impasto_wax,
            wax_color: self.impasto_wax_color,
        }
    }
}

/// A referência da quantidade do Grain ([`BrushSpec::grao_aplicado`]): o estado do pincel, viaja com ele.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GraoAplicado {
    /// O Grain do preset (`kind` `None` = o pincel foi aplicado sem Grain).
    pub grao: TextureSettings,
    /// A média dele, congelada quando já não há como amostrá-lo: um Grain de imagem cuja imagem saiu do slot.
    pub media: Option<f32>,
}

impl BrushSpec {
    /// O Grain de agora já não é o aplicado? Mexer nos parâmetros do MESMO Grain não é trocá-lo.
    #[must_use]
    pub fn grao_trocado(&self) -> bool {
        self.grao_aplicado.media.is_some() || self.grao_aplicado.grao.kind != self.texture.kind
    }
}
