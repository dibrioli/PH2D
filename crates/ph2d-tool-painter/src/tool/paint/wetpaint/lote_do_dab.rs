//! **O lote do dab**: o que a rota apura uma vez e cada dab de [`super::pousa_o_dab`] lê (filho de [`super`], LOC cap).

use super::*;

/// O que [`super::dab_route`] apurou para o lote inteiro e cada dab lê (os nomes são os da rota).
///
/// DONO dos seus dados (N10 F2b): as imagens são `Arc` (clonar = contador), o resto é cópia — o lote pode
/// atravessar para o fio que segura o motor sem emprestar do `PainterTool`.
pub(super) struct LoteDoDab {
    pub(super) brush: BrushSpec,
    pub(super) erasing: bool,
    pub(super) opacidade_da_pressao: Option<f32>,
    pub(super) dual: Option<super::super::dual::RecorteNoWet>,
    pub(super) ratio: u8,
    pub(super) wet_tool: WetTool,
    pub(super) rampa: Option<super::super::watercolor_accum_cor::RampaDaAguada>,
    pub(super) padrao_vira_cor: bool,
    pub(super) grao_e_cor: bool,
    pub(super) shape_image: Option<super::super::brush_settings::BrushTextureImage>,
    pub(super) shape_ramp_lut: Option<std::sync::Arc<[f32]>>,
    pub(super) cerdas: bool,
    pub(super) marca_do_spray: bool,
    pub(super) filme_liso: bool,
    pub(super) fios: ph2d_wet_paint::bristles_fios::Fios,
    pub(super) relevo: ph2d_wet_paint::bristles_relevo::Relevo,
    pub(super) rebordo: f64,
    pub(super) lamina: bool,
    pub(super) passo_do_pigmento: f32,
    pub(super) capsulas: bool,
    pub(super) shape_active: bool,
    pub(super) grain_image: Option<super::super::brush_settings::BrushTextureImage>,
    pub(super) grain_active: bool,
    pub(super) groups: Vec<u32>,
    pub(super) canvas_wh: [f32; 2],
    pub(super) strength: f32,
    pub(super) fino: f32,
    pub(super) esmaece_fino: f64,
    pub(super) escala_tex: f32,
    pub(super) grao_e_o_papel: bool,
    pub(super) media_depois_do_portao: Option<f64>,
    pub(super) ponta_sob_o_grao: bool,
    pub(super) ganho_do_grao: f64,
    pub(super) sem_grao: Option<[f64; 2]>,
    pub(super) pigmento_da_marca: f32,
    pub(super) escala_cerdas: [f32; 2],
    pub(super) copia: Vec<u32>,
    pub(super) pontas: bool,
    pub(super) alem: f32,
    pub(super) esbate_e: f32,
    pub(super) virtual_: bool,
    pub(super) lote: super::tampa::Lote,
    pub(super) gota: bool,
}

impl LoteDoDab {
    /// A imagem do Shape como o motor a lê.
    pub(super) fn shape_mask(&self) -> Option<ph2d_painter_brush::ImageMask<'_>> {
        self.shape_image.as_ref().map(|i| i.as_mask())
    }
    /// A imagem do Grain como o motor a lê.
    pub(super) fn grain_mask(&self) -> Option<ph2d_painter_brush::ImageMask<'_>> {
        self.grain_image.as_ref().map(|i| i.as_mask())
    }
}

const _: () = {
    const fn envia<T: Send + 'static>() {}
    envia::<LoteDoDab>();
};
