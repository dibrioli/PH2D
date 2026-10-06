//! **Os controlos do pincel que DEPENDEM de outro** — e a dica que o painel mostra enquanto eles não
//! agem (decisão do dono, 2026-10-04: *«deixá-los esmaecidos»*; a tabela medida é o doc 45 §2.3).
//!
//! ⭐ **Cada lei é a MESMA que torna o controlo inerte**, perguntada à porta que o motor usa — o
//! `space_attenuation_reaches` que o `space_overlap_factor` pergunta, o `stroke_cover_wanted` que o
//! carimbo pergunta, o `Taper::is_active` que o traço pergunta. Uma segunda opinião sobre «este
//! controlo age?» seria a linha esmaecida que mente. O gate é o censo: esmaecida ⇔ a sonda mede inerte.

use super::PainterTool;
use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_painter_brush::{BrushSpec, MAX_TEX_PARAMS, param_inerte};

/// Um controlo cujo efeito depende de outro — com os ids que o painel pinta para ele e a dica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dependente {
    TaperTip,
    TaperOpacity,
    Accumulate,
    SpaceAttenuation,
    JitterUnit,
    DashLength,
    PaperRoughness,
    Paper,
    GrainDaAquarela,
    GrainDaAgua,
    Pull,
    DepthSourceGrain,
    EraseDaAgua,
    ToothDoDigital,
}

impl Dependente {
    pub const TODOS: [Dependente; 14] = [
        Dependente::TaperTip,
        Dependente::TaperOpacity,
        Dependente::Accumulate,
        Dependente::SpaceAttenuation,
        Dependente::JitterUnit,
        Dependente::DashLength,
        Dependente::PaperRoughness,
        Dependente::Paper,
        Dependente::GrainDaAquarela,
        Dependente::GrainDaAgua,
        Dependente::Pull,
        Dependente::DepthSourceGrain,
        Dependente::EraseDaAgua,
        Dependente::ToothDoDigital,
    ];

    /// Os ids que o painel pinta para este controlo.
    #[must_use]
    pub fn ids(self) -> &'static [NodeId] {
        match self {
            Dependente::TaperTip => &[ids::PAINTER_TAPER_TIP_START],
            Dependente::TaperOpacity => &[ids::PAINTER_TAPER_OPACITY],
            Dependente::Accumulate => &[ids::PAINTER_BRUSH_ACCUMULATE],
            Dependente::SpaceAttenuation => &[ids::PAINTER_BRUSH_SPACE_ATTEN],
            Dependente::JitterUnit => &[ids::PAINTER_BRUSH_JITTER_UNIT],
            Dependente::DashLength => &[
                ids::PAINTER_BRUSH_DASH_LENGTH,
                ids::PAINTER_BRUSH_DASH_LENGTH_CHIP,
            ],
            Dependente::PaperRoughness => &[ids::PAINTER_SUBSTRATE_ROUGHNESS],
            Dependente::Paper => &[ids::PAINTER_WATERCOLOR_PAPER_KIND],
            Dependente::GrainDaAquarela | Dependente::GrainDaAgua => &GRAIN,
            Dependente::Pull => &[ids::PAINTER_WATERCOLOR_PULL],
            Dependente::DepthSourceGrain => &[ids::PAINTER_IMPASTO_SOURCE_GRAIN],
            Dependente::EraseDaAgua => &[ids::PAINTER_WETPAINT_ERASE],
            Dependente::ToothDoDigital => &[ids::PAINTER_WATERCOLOR_PAPER_DEPTH],
        }
    }

    /// A chave da dica (`ph2d-i18n`): o que LIGA o controlo.
    #[must_use]
    pub fn dica(self) -> &'static str {
        match self {
            Dependente::TaperTip | Dependente::TaperOpacity => {
                "panel.painter_layers.inerte.taper_length"
            }
            Dependente::Accumulate => "panel.painter_layers.inerte.strength_below_full",
            Dependente::SpaceAttenuation => "panel.painter_layers.inerte.accumulate_on",
            Dependente::JitterUnit => "panel.painter_layers.inerte.jitter_above_zero",
            Dependente::DashLength => "panel.painter_layers.inerte.dash_ratio_below_full",
            Dependente::PaperRoughness => "panel.painter_layers.inerte.paper_and_relief",
            Dependente::Paper => "panel.painter_layers.inerte.relief_above_zero",
            Dependente::GrainDaAquarela => "panel.painter_layers.inerte.same_as_paper_off",
            Dependente::GrainDaAgua => "panel.painter_layers.inerte.wet_paint_tool",
            Dependente::Pull => "panel.painter_layers.inerte.charge_below_full",
            Dependente::DepthSourceGrain => "panel.painter_layers.inerte.grain_chosen",
            Dependente::EraseDaAgua => "panel.painter_layers.inerte.eraser_on",
            Dependente::ToothDoDigital => "panel.painter_layers.inerte.relief_and_no_grain",
        }
    }

    fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

/// O Grain inteiro: o menu e os seis parâmetros.
const GRAIN: [NodeId; 7] = [
    ids::PAINTER_BRUSH_TEXTURE_KIND,
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[0],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[1],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[2],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[3],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[4],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[5],
];

/// A dica do parâmetro de um padrão (Grain ou Shape) que está inerte.
pub const DICA_DO_PARAMETRO: &str = "panel.painter_layers.inerte.detail_above_minimum";

/// O que está inerte AGORA — viaja no instantâneo do pincel ([`super::BrushSettings::inercias`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inercias {
    dependentes: u32,
    /// Bit `k` = o parâmetro `k` do Grain está inerte ([`param_inerte`]).
    grain_params: u8,
    /// Bit `k` = o parâmetro `k` da Shape está inerte.
    shape_params: u8,
}

impl Inercias {
    /// Nenhum controlo inerte — o instantâneo de recurso do painel antes da 1.ª publicação.
    pub const NENHUMA: Inercias = Inercias {
        dependentes: 0,
        grain_params: 0,
        shape_params: 0,
    };

    #[must_use]
    pub fn inerte(self, d: Dependente) -> bool {
        self.dependentes & d.bit() != 0
    }

    /// **A dica de um id pintado, se ele está inerte** — a pergunta única do painel.
    #[must_use]
    pub fn dica_de(self, id: NodeId) -> Option<&'static str> {
        if let Some(d) = Dependente::TODOS
            .into_iter()
            .find(|d| self.inerte(*d) && d.ids().contains(&id))
        {
            return Some(d.dica());
        }
        let slot_inerte = |mask: u8, ids: &[NodeId]| {
            ids.iter()
                .enumerate()
                .any(|(k, i)| *i == id && mask & (1 << k) != 0)
        };
        (slot_inerte(self.grain_params, &ids::PAINTER_BRUSH_TEXTURE_PARAMS)
            || slot_inerte(self.shape_params, &ids::PAINTER_SHAPE_PARAMS))
        .then_some(DICA_DO_PARAMETRO)
    }
}

fn params_inertes(t: &ph2d_painter_brush::TextureSettings) -> u8 {
    (0..MAX_TEX_PARAMS)
        .filter(|&k| param_inerte(t.kind, &t.params, k))
        .fold(0u8, |m, k| m | (1 << k))
}

impl PainterTool {
    /// O que está inerte com o pincel de AGORA — cada linha pela lei que o motor usa.
    pub(super) fn inercias(&self) -> Inercias {
        let b = &self.paint.brush;
        let aquarela = self.watercolor_render_active();
        let agua = self.paint.wetpaint.armed;
        let leis = [
            (Dependente::TaperTip, !b.taper.is_active()),
            (Dependente::TaperOpacity, !b.taper.is_active()),
            // O Accumulate decide DUAS coisas: o tecto do Strength (`stroke_cover_wanted`) e se o
            // Space Attenuation ligado chega ao carimbo (`space_attenuation_reaches` com ele ligado).
            (
                Dependente::Accumulate,
                !self.stroke_cover_wanted(&BrushSpec {
                    accumulate: false,
                    ..*b
                }) && !(b.space_attenuation
                    && BrushSpec {
                        accumulate: true,
                        ..*b
                    }
                    .space_attenuation_reaches()),
            ),
            (Dependente::SpaceAttenuation, !b.space_attenuation_reaches()),
            (Dependente::JitterUnit, !b.jitter_unit_matters()),
            (Dependente::DashLength, !b.dash_length_matters()),
            (Dependente::PaperRoughness, self.substrate().is_none()),
            (
                Dependente::Paper,
                !aquarela && !agua && self.paint.substrate_depth <= 0.0,
            ),
            (
                Dependente::GrainDaAquarela,
                aquarela && b.granulation_use_paper,
            ),
            // Na água só o depósito (Paint) e a borracha recebem o Grain: Blend, Smear, Wet, Dry e
            // Blow ignoram-no de propósito (`wetpaint/dab_route.rs`).
            (
                Dependente::GrainDaAgua,
                agua && !self.paint.eraser && self.paint.wetpaint.tool != super::WetTool::Paint,
            ),
            (Dependente::Pull, aquarela && !self.wet_mixer_active()),
            (Dependente::DepthSourceGrain, !b.texture.is_active()),
            (Dependente::EraseDaAgua, agua && !self.paint.eraser),
            // No Digital o Tooth é a profundidade do papel feito Grain (`papel_como_grain`).
            (
                Dependente::ToothDoDigital,
                matches!(self.paint_media(), super::media::PaintMedia::Digital)
                    && !self.papel_pode_ser_grain(),
            ),
        ];
        Inercias {
            dependentes: leis
                .into_iter()
                .filter(|(_, inerte)| *inerte)
                .fold(0, |m, (d, _)| m | d.bit()),
            grain_params: params_inertes(&b.texture),
            shape_params: params_inertes(&b.shape),
        }
    }
}
