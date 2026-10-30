//! **O «Textured Paper» do New Image** (dono 30/10): a imagem nasce com o cartão Paper do dono
//! (foto `docs/Painter/Pinceis/pranchas/J9_smoke_dono_papel_texturizado.png`).

use super::PainterTool;
use ph2d_painter_brush::TextureKind;

/// O cartão Paper de uma imagem nascida «Textured Paper».
pub struct PapelPreset {
    /// A Color do papel (sRGB8).
    pub cor: [u8; 3],
    pub kind: TextureKind,
    pub relief: f32,
    pub roughness: f32,
    pub angle_deg: f32,
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub tooth: f32,
    pub smooth_edges: f32,
    /// Os params do padrão (Cold Press: Contrast, Brightness).
    pub params: [f32; 2],
}

/// Os valores do dono (30/10): Cold Press bege `#E6D9CC`.
pub const PAPEL_TEXTURIZADO: PapelPreset = PapelPreset {
    cor: [0xE6, 0xD9, 0xCC], // LITERAL-COLOR-OK: a cor do papel escolhida pelo dono (dado, não UI)
    kind: TextureKind::PaperCold,
    relief: 0.5,
    roughness: 0.5,
    angle_deg: 0.0,
    offset: [0.0, 0.0],
    size: [1.0, 1.0],
    tooth: 1.0,
    smooth_edges: 0.0,
    params: [0.5, 0.5],
};

impl PainterTool {
    /// A imagem `entity` acabou de nascer pelo *New Image…* com «Textured Paper»: quando ela vira
    /// documento, o papel dela é o [`PAPEL_TEXTURIZADO`].
    pub fn agenda_papel_texturizado(&mut self, entity: u64) {
        self.papel_texturizado_para = Some(entity);
    }

    /// Chamada quando `entity` abre como documento novo: serve o papel agendado, se é dela.
    pub(crate) fn serve_papel_texturizado(&mut self, entity: u64) {
        if self.papel_texturizado_para != Some(entity) || self.papel.is_none() {
            return;
        }
        self.papel_texturizado_para = None;
        self.aplica_papel_preset(&PAPEL_TEXTURIZADO);
    }

    /// Escreve o cartão Paper inteiro pelas portas dos controlos dele.
    fn aplica_papel_preset(&mut self, p: &PapelPreset) {
        let [r, g, b] = p.cor;
        self.set_paper_color_rgb8(r, g, b);
        self.papel = Some(p.cor);
        self.set_brush_paper_kind(p.kind.to_u8());
        self.set_substrate_depth(p.relief);
        self.set_substrate_roughness(p.roughness);
        self.set_brush_paper_angle(p.angle_deg);
        for axis in 0..2 {
            self.set_brush_paper_offset(axis, p.offset[axis]);
            self.set_brush_paper_size(axis, p.size[axis]);
        }
        self.set_brush_paper_depth(p.tooth);
        for (slot, v) in p.params.iter().enumerate() {
            self.set_brush_paper_param(slot, *v);
        }
        self.set_wet_smooth_edges(p.smooth_edges);
        self.invalidate_composite();
    }
}
