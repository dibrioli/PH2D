//! **A marca do artista no documento do Painter** (W9, `docs/Macros/07_plano_W9.md`): quem mostra uma
//! macro em cena dá ao Painter a marca da imagem ligada ([`PainterTool::poe_a_marca`]) e lê-a de volta
//! ([`PainterTool::marca_do_artista`]); a história escreve-a ([`crate::undo::marca_viva`]).

use super::PainterTool;
use crate::macro_track::marca::MarcaDoArtista;
use crate::undo::marca_viva::MarcaViva;
use ph2d_painter_brush::{BrushSpec, Dab, capsula};

impl PainterTool {
    /// A marca da imagem ligada (`None` = a imagem não tem macro: nada a marcar).
    pub fn poe_a_marca(&mut self, m: Option<MarcaDoArtista>) {
        self.undo.marca.poe(m);
    }

    /// A marca da imagem ligada, com o pincel do passo em curso.
    #[must_use]
    pub fn marca_do_artista(&self) -> Option<&MarcaDoArtista> {
        self.undo.marca.marca.as_ref()
    }

    /// **A marca que um ficheiro guarda** (W14): a do artista e, se a água da gravação ainda corre, o
    /// estado em que a imagem gravada a fica — reaberta, a água não corre mais.
    #[must_use]
    pub fn marca_a_gravar(&self) -> Option<MarcaDoArtista> {
        let mut m = self.marca_do_artista()?.clone();
        if let Some(c) = self.wet_agua_da_gravacao_viva() {
            m.para_a_agua(c);
        }
        Some(m)
    }

    /// O pincel cobre cada dab de `dabs`, com o alcance `reach` da ponta (`na_agua` = entram na água).
    pub(crate) fn marca_os_dabs(&mut self, dabs: &[Dab], reach: f32, na_agua: bool) {
        if !self.undo.marca.ativa() {
            return;
        }
        for d in dabs {
            let r = d.radius_px * reach + 0.5;
            self.undo.marca.disco(d.center[0], d.center[1], r, na_agua);
        }
    }
}

/// A força com que `dabs` do pincel `spec` cobrem cada texel ([`crate::macro_track::forca`]): o perfil da
/// ponta × a cobertura × o Flow, cada sub-dab da cápsula; `uniao` = os dabs compõem-se (Accumulate
/// ligado). Só numa ponta que é o perfil redondo sozinho — uma forma ou um grão recortam-na por píxel e
/// uma ponta achatada é elipse: o perfil mentiria para cima (aí vale só o que mudou).
pub(crate) fn marca_a_forca(m: &mut MarcaViva, spec: &BrushSpec, dabs: &[Dab], uniao: bool) {
    let Some(tamanho) = m.marca.as_ref().map(MarcaDoArtista::tamanho) else {
        return;
    };
    if spec.shape.is_active() || spec.texture.is_active() || !spec.footprint_deform().is_identity()
    {
        return;
    }
    let flow = spec.flow.clamp(0.0, 1.0);
    for d in dabs {
        for s in capsula::sub_dabs(d, capsula::sub_passo_px(d.radius_px)) {
            let (c, r) = (s.coverage.clamp(0.0, 1.0) * flow, s.radius_px);
            if c > 0.0 && r > 0.0 {
                let a = |dist: f32| c * spec.falloff_weight(dist / r);
                m.forca
                    .deposita(tamanho, (s.center[0], s.center[1], r), uniao, a);
            }
        }
    }
}
