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
    // A ponta de PÍXEL (L18) carimba a pegada binária: o grão, a Shape e as N imagens não a recortam.
    ForaDaPontaDePixel,
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
    BordaSemTilt,
    SemPapel,
    QuandoDoRandomize,
    CerdasDaAgua,
    FollowSemNada,
    ContinuoSemPasso,
    CorDaTexturaSemLeitor,
    TexturaSemForma,
    CurvaSemOpacidade,
    CanetaDePe,
    PegadaDaAutomatica,
    CurvaDoTamanhoSemPressao,
    MaoSemPressao,
    MarcaSemSpray,
    OpacidadeNaAguada,
    CorDaMaoSemLeitor,
    // O fim do traço (L5) e o traço como uma camada (L8), no Digital (`traco_plano`); o modo sobre o vazio (L7).
    PontaDaCaudaSemCauda,
    PatamarSemFallOff,
    LarguraSemOrla,
    VazioSemModoDeCor,
    PlanoForaDoTraco,
    FallOffDaAgua,
    // A profundidade do grão por dab (L9): o mínimo e o controlo só leem o que o jitter faz variar.
    ProfundidadeSemJitter,
    // O dual brush (L6) recorta o traço composto como CAMADA (`traco_plano`, o Digital) ou o pigmento que pousa (o
    // Wet): a aguada e o Impasto não; e só num traço livre que pinta.
    DualForaDoDigital,
    DualForaDoTraco,
}

impl Dependente {
    pub const TODOS: [Dependente; 40] = [
        Dependente::ForaDaPontaDePixel,
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
        Dependente::BordaSemTilt,
        Dependente::SemPapel,
        Dependente::QuandoDoRandomize,
        Dependente::CerdasDaAgua,
        Dependente::FollowSemNada,
        Dependente::ContinuoSemPasso,
        Dependente::CorDaTexturaSemLeitor,
        Dependente::TexturaSemForma,
        Dependente::CurvaSemOpacidade,
        Dependente::CanetaDePe,
        Dependente::PegadaDaAutomatica,
        Dependente::CurvaDoTamanhoSemPressao,
        Dependente::MaoSemPressao,
        Dependente::MarcaSemSpray,
        Dependente::OpacidadeNaAguada,
        Dependente::CorDaMaoSemLeitor,
        Dependente::PontaDaCaudaSemCauda,
        Dependente::PatamarSemFallOff,
        Dependente::LarguraSemOrla,
        Dependente::VazioSemModoDeCor,
        Dependente::PlanoForaDoTraco,
        Dependente::FallOffDaAgua,
        Dependente::ProfundidadeSemJitter,
        Dependente::DualForaDoDigital,
        Dependente::DualForaDoTraco,
    ];

    /// Os ids que o painel pinta para este controlo.
    #[must_use]
    pub fn ids(self) -> &'static [NodeId] {
        match self {
            Dependente::ForaDaPontaDePixel => &FORA_DA_PONTA_DE_PIXEL,
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
            Dependente::BordaSemTilt => &[ids::PAINTER_WETPAINT_HEAVY_EDGE],
            Dependente::SemPapel => &[
                ids::PAINTER_WATERCOLOR_PAPER_DEPTH,
                ids::PAINTER_WETPAINT_PAPER_VISUAL,
            ],
            Dependente::QuandoDoRandomize => &ids::PAINTER_BRUSH_COLOR_JITTER_SCOPES,
            Dependente::CerdasDaAgua => &CERDAS,
            Dependente::FollowSemNada => &[ids::PAINTER_SHAPE_FOLLOW],
            Dependente::ContinuoSemPasso => &[ids::PAINTER_BRUSH_CONTINUOUS_SPACING],
            Dependente::CorDaTexturaSemLeitor => &[ids::PAINTER_SHAPE_PER_LAYER_COLOR],
            Dependente::TexturaSemForma => &[
                ids::PAINTER_SHAPE_TEXTURE_COLOR,
                ids::PAINTER_SHAPE_TEXTURE_OPACITY,
            ],
            // O cartão Pen (L2), pela ordem de `LINHAS_DA_MAO`.
            Dependente::CurvaSemOpacidade => &[ids::PAINTER_PEN_FIELDS[3]],
            Dependente::CanetaDePe => &[
                ids::PAINTER_PEN_FIELDS[9],
                ids::PAINTER_PEN_FIELDS[10],
                ids::PAINTER_PEN_FIELDS[14],
            ],
            Dependente::PegadaDaAutomatica => {
                &[ids::PAINTER_PEN_FIELDS[11], ids::PAINTER_PEN_FIELDS[12]]
            }
            Dependente::CurvaDoTamanhoSemPressao => &[ids::PAINTER_PEN_FIELDS[1]],
            // As linhas da MARCA (L4), pela ordem de `LINHAS_DA_MARCA`: a contagem sorteada e a partícula são
            // do spray; a opacidade, de um meio que leia a do dab. (Os espelhos, a redondeza e o ângulo do traço NÃO
            // esmaecem: ver `o_censo_so_encolhe`, a razão do Jitter Rotate.)
            Dependente::MarcaSemSpray => {
                &[ids::PAINTER_MARCA_FIELDS[1], ids::PAINTER_MARCA_FIELDS[7]]
            }
            // …e o FALL-OFF (L5), que é a cobertura do dab ao longo do traço: a aguada também não o lê.
            Dependente::OpacidadeNaAguada => &[
                ids::PAINTER_MARCA_FIELDS[0],
                ids::PAINTER_TAPER_FALL_OFF,
                ids::PAINTER_TAPER_FALL_OFF_START,
            ],
            Dependente::PontaDaCaudaSemCauda => &[ids::PAINTER_TAPER_TIP_END],
            Dependente::FallOffDaAgua => &[
                ids::PAINTER_TAPER_FALL_OFF,
                ids::PAINTER_TAPER_FALL_OFF_START,
            ],
            Dependente::PatamarSemFallOff => &[ids::PAINTER_TAPER_FALL_OFF_START],
            Dependente::LarguraSemOrla => &[ids::PAINTER_TRACO_FIELDS[3]],
            Dependente::VazioSemModoDeCor => &[ids::PAINTER_TRACO_FIELDS[4]],
            Dependente::PlanoForaDoTraco => &[
                ids::PAINTER_TRACO_FIELDS[0],
                ids::PAINTER_TRACO_FIELDS[1],
                ids::PAINTER_TRACO_FIELDS[2],
                ids::PAINTER_TRACO_FIELDS[3],
                ids::PAINTER_TAPER_END,
                ids::PAINTER_TAPER_TIP_END,
            ],
            // A cor pela mão e a secundária (L11): a cor do dab — que a rampa e a Texture Color substituem.
            Dependente::CorDaMaoSemLeitor => &[
                ids::PAINTER_MARCA_FIELDS[8],
                ids::PAINTER_MARCA_FIELDS[9],
                ids::PAINTER_MARCA_FIELDS[10],
                ids::PAINTER_MARCA_FIELDS[11],
                ids::PAINTER_MARCA_FIELDS[12],
                ids::PAINTER_MARCA_SECONDARY,
            ],
            // A ponta (L9), pela ordem de `LINHAS_DA_PONTA`: Minimum depth e Depth control.
            Dependente::ProfundidadeSemJitter => {
                &[ids::PAINTER_PONTA_FIELDS[2], ids::PAINTER_PONTA_FIELDS[4]]
            }
            Dependente::DualForaDoDigital | Dependente::DualForaDoTraco => &DUAL,
            Dependente::MaoSemPressao => &[
                ids::PAINTER_PEN_FIELDS[4],
                ids::PAINTER_PEN_FIELDS[5],
                ids::PAINTER_PEN_FIELDS[6],
            ],
        }
    }

    /// A chave da dica (`ph2d-i18n`): o que LIGA o controlo.
    #[must_use]
    pub fn dica(self) -> &'static str {
        match self {
            Dependente::ForaDaPontaDePixel => "panel.painter_ponta.inerte.pixel_tip",
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
            Dependente::BordaSemTilt => "panel.painter_layers.inerte.tilt_on",
            Dependente::SemPapel => "panel.painter_layers.inerte.paper_chosen",
            Dependente::QuandoDoRandomize => "panel.painter_layers.inerte.randomize_above_zero",
            Dependente::CerdasDaAgua => "panel.painter_layers.inerte.bristles_no_grain",
            Dependente::FollowSemNada => "panel.painter_layers.inerte.follow_nothing_to_turn",
            Dependente::ContinuoSemPasso => "panel.painter_layers.inerte.method_lays_a_path",
            Dependente::CorDaTexturaSemLeitor => "panel.painter_layers.inerte.texture_colors_media",
            Dependente::TexturaSemForma => {
                "panel.painter_layers.inerte.texture_color_needs_a_shape"
            }
            Dependente::CurvaSemOpacidade => "panel.painter_layers.inerte.pressure_opacity_on",
            Dependente::CanetaDePe => "panel.painter_layers.inerte.tilt_law_on",
            Dependente::PegadaDaAutomatica => "panel.painter_layers.inerte.shape_automatic_off",
            Dependente::CurvaDoTamanhoSemPressao => "panel.painter_layers.inerte.pressure_size_on",
            Dependente::MaoSemPressao => "panel.painter_layers.inerte.pressure_on",
            Dependente::MarcaSemSpray => "panel.painter_layers.inerte.spray_count_above_one",
            Dependente::OpacidadeNaAguada => "panel.painter_layers.inerte.wash_ignores_dab_opacity",
            Dependente::CorDaMaoSemLeitor => "panel.painter_layers.inerte.color_owned_by_ramp",
            Dependente::PontaDaCaudaSemCauda => "panel.painter_layers.inerte.taper_end_length",
            Dependente::PatamarSemFallOff => "panel.painter_layers.inerte.fall_off_length",
            Dependente::LarguraSemOrla => "panel.painter_layers.inerte.edges_above_zero",
            Dependente::VazioSemModoDeCor => "panel.painter_layers.inerte.color_blend_mode",
            Dependente::PlanoForaDoTraco => "panel.painter_layers.inerte.freehand_paint_stroke",
            Dependente::FallOffDaAgua => "panel.painter_layers.inerte.wet_paint_tool",
            Dependente::ProfundidadeSemJitter => {
                "panel.painter_layers.inerte.depth_jitter_above_zero"
            }
            Dependente::DualForaDoDigital => "panel.painter_ponta.inerte.dual_medium",
            Dependente::DualForaDoTraco => "panel.painter_layers.inerte.freehand_paint_stroke",
        }
    }

    /// Este id é uma linha deste controlo — os de [`Self::ids`] e, na cor da textura, as linhas «Layer N
    /// Color» de uma Shape de várias camadas (ids de fábrica, o 26 C16).
    #[must_use]
    pub fn cobre(self, id: NodeId) -> bool {
        self.ids().contains(&id)
            || (self == Dependente::CorDaTexturaSemLeitor
                && (0..super::shape_layers::MAX_SHAPE_LAYERS as u8).any(|i| {
                    id == ids::painter_shape_layer_color_check_id(i)
                        || id == ids::painter_shape_layer_color_swatch_id(i)
                }))
    }

    /// **Porque está inerte** — o que decide se a linha esmaecida ainda aceita o clique.
    #[must_use]
    pub fn natureza(self) -> Natureza {
        match self {
            Dependente::CorDaTexturaSemLeitor
            | Dependente::OpacidadeNaAguada
            | Dependente::DualForaDoDigital => Natureza::MeioNaoLe,
            _ => Natureza::DependeDeOutro,
        }
    }

    fn bit(self) -> u64 {
        1 << (self as u64)
    }
}

/// As duas espécies de linha esmaecida (prompt 35).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Natureza {
    /// Age quando o artista mudar OUTRO controlo do mesmo meio: continua editável, ele prepara o valor
    /// (o `active = False` do Blender).
    DependeDeOutro,
    /// Este meio nunca o lê, mude-se o que se mudar nele: não aceita clique (o Color inativo do Bug #51).
    MeioNaoLe,
}

/// O Grain inteiro: o menu, os seis parâmetros e a lei dele (L9: o modo, Minimum depth, Depth jitter, Depth control).
const GRAIN: [NodeId; 11] = [
    ids::PAINTER_BRUSH_TEXTURE_KIND,
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[0],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[1],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[2],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[3],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[4],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[5],
    ids::PAINTER_PONTA_FIELDS[1],
    ids::PAINTER_PONTA_FIELDS[2],
    ids::PAINTER_PONTA_FIELDS[3],
    ids::PAINTER_PONTA_FIELDS[4],
];

/// As cerdas do Shape: os quatro sliders e a transformação (Angle, Offset, Size) — no Wet Paint elas SÃO a
/// ponta (o `texv`). ⛔ O Follow NÃO: ele roda o referencial do DAB inteiro, e com um Grain armado roda o
/// Grain (medido pelo censo, 13/10).
const CERDAS: [NodeId; 9] = [
    ids::PAINTER_SHAPE_PARAMS[0],
    ids::PAINTER_SHAPE_PARAMS[1],
    ids::PAINTER_SHAPE_PARAMS[2],
    ids::PAINTER_SHAPE_PARAMS[3],
    ids::PAINTER_SHAPE_ANGLE,
    ids::PAINTER_SHAPE_OFFSET_X,
    ids::PAINTER_SHAPE_OFFSET_Y,
    ids::PAINTER_SHAPE_SIZE_X,
    ids::PAINTER_SHAPE_SIZE_Y,
];

/// O bloco Dual brush (L6): o modo, as linhas da 2.ª ponta (pela ordem de `LINHAS_DA_PONTA`) e «Dual image».
const DUAL: [NodeId; 13] = [
    ids::PAINTER_PONTA_FIELDS[14],
    ids::PAINTER_PONTA_FIELDS[15],
    ids::PAINTER_PONTA_FIELDS[16],
    ids::PAINTER_PONTA_FIELDS[17],
    ids::PAINTER_PONTA_FIELDS[18],
    ids::PAINTER_PONTA_FIELDS[19],
    ids::PAINTER_PONTA_FIELDS[20],
    ids::PAINTER_PONTA_FIELDS[21],
    ids::PAINTER_PONTA_FIELDS[22],
    ids::PAINTER_PONTA_FIELDS[23],
    ids::PAINTER_PONTA_FIELDS[24],
    ids::PAINTER_PONTA_FIELDS[25],
    ids::PAINTER_DUAL_IMAGE,
];

/// O que a ponta de PÍXEL (L18) não lê: o Grain (o menu, os parâmetros e a lei da L9) e a ponta de N imagens (L19) —
/// todos recortariam o píxel em fracções. ⚠️ A Grain Depth e a fonte da Shape NÃO entram: o censo do esmaecido
/// (`a_linha_esmaecida_e_a_que_nao_age`) mede-as inertes noutros meios sem lei que o diga (a Watercolor não lê a
/// Depth; o Wet não muda com a fonte da Shape no traço do censo) — listá-las aqui pô-las-ia nessa régua sem a lei.
const FORA_DA_PONTA_DE_PIXEL: [NodeId; 21] = [
    ids::PAINTER_BRUSH_TEXTURE_KIND,
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[0],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[1],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[2],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[3],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[4],
    ids::PAINTER_BRUSH_TEXTURE_PARAMS[5],
    ids::PAINTER_PONTA_FIELDS[1],
    ids::PAINTER_PONTA_FIELDS[2],
    ids::PAINTER_PONTA_FIELDS[3],
    ids::PAINTER_PONTA_FIELDS[4],
    ids::PAINTER_PONTA_FIELDS[5],
    ids::PAINTER_PONTA_FIELDS[6],
    ids::PAINTER_PONTA_FIELDS[7],
    ids::PAINTER_PONTA_FIELDS[8],
    ids::PAINTER_PONTA_FIELDS[9],
    ids::PAINTER_PONTA_FIELDS[10],
    ids::PAINTER_PONTA_FIELDS[11],
    ids::PAINTER_PONTA_FIELDS[12],
    ids::PAINTER_PONTA_FIELDS[13],
    ids::PAINTER_SHAPE_HOSE_ADD,
];

/// A dica do parâmetro de um padrão (Grain ou Shape) que está inerte.
pub const DICA_DO_PARAMETRO: &str = "panel.painter_layers.inerte.detail_above_minimum";

/// O que está inerte AGORA — viaja no instantâneo do pincel ([`super::BrushSettings::inercias`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inercias {
    dependentes: u64,
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

    /// **O clique neste id é recusado** — ele está inerte por um meio que nunca o lê
    /// ([`Natureza::MeioNaoLe`]). A porta dos eventos do painel pergunta-o antes de rotear.
    #[must_use]
    pub fn recusa_clique(self, id: NodeId) -> bool {
        Dependente::TODOS
            .into_iter()
            .any(|d| self.inerte(d) && d.natureza() == Natureza::MeioNaoLe && d.cobre(id))
    }

    /// **A dica de um id pintado, se ele está inerte** — a pergunta única do painel.
    #[must_use]
    pub fn dica_de(self, id: NodeId) -> Option<&'static str> {
        if let Some(d) = Dependente::TODOS
            .into_iter()
            .find(|d| self.inerte(*d) && d.cobre(id))
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
    pub(crate) fn inercias(&self) -> Inercias {
        let b = &self.paint.brush;
        let aquarela = self.watercolor_render_active();
        let agua = self.paint.wetpaint.armed;
        // O Follow roda a pegada INTEIRA do dab (`BrushSpec::dab_rotor`): sem textura no Shape, sem ponta achatada
        // e sem um Grain preso ao dab, a ponta redonda não tem ângulo para mostrar.
        let sem_lado = !b.shape_silhouette_active(self.paint.shape_image.is_some())
            && b.shape.kind != ph2d_painter_brush::TextureKind::Bristles
            && !b.pegada_anisotropica()
            && !(b.grain_samples(self.paint.texture_image.is_some())
                && matches!(
                    b.texture.mapping,
                    ph2d_painter_brush::TextureMapping::ViewPlane
                        | ph2d_painter_brush::TextureMapping::Random
                ))
            // A profundidade do grão POR DAB (L9) leva o dab à rota por dab (`DabTilt::grao`, `any_tilted`), que roda a
            // pegada pelo rotor: a tinta muda pelo arredondamento (a razão do Jitter Rotate, `o_censo_so_encolhe`). O
            // Impasto não: a altura e a cor dele não rodam a pegada redonda (medido pelo gate do esmaecido, 20/10).
            && !(b.texture.lei.varia_por_dab()
                && b.texture.is_active()
                && !matches!(self.paint_media(), super::media::PaintMedia::Impasto));
        let leis = [
            // A ponta de píxel (L18) é binária: o grão, a Shape e as N imagens tornariam o píxel fraccionário.
            (Dependente::ForaDaPontaDePixel, self.pixel_vivo(b)),
            (Dependente::TaperTip, !b.taper.is_active()),
            (Dependente::TaperOpacity, !b.taper.is_active()),
            // O Accumulate decide DUAS coisas: o tecto do Strength (`stroke_cover_wanted`) e se o
            // Space Attenuation ligado chega ao carimbo (`space_attenuation_reaches` com ele ligado).
            (
                Dependente::Accumulate,
                !self.cobertura_tapada(&BrushSpec {
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
            // A Heavy Edge só age com a tela inclinada (`Sim::sedimenta` multiplica `|g|`).
            (
                Dependente::BordaSemTilt,
                agua && !(self.paint.wetpaint.tilt_on && self.paint.wetpaint.tilt_ring > 0),
            ),
            // Sem Kind o Wet Paint pinta sem desenho de papel (dono 10/10): o Tooth não tem relevo para
            // esculpir e o Texture on Paint não tem papel para imprimir.
            (Dependente::SemPapel, agua && !b.paper.is_active()),
            // «Every Dab / Every Stroke» escolhe QUANDO o Randomize Color sorteia — sem Hue/Sat/Value não
            // há sorteio (`jitter::colour_draw`).
            (Dependente::QuandoDoRandomize, !b.has_colour_jitter_amount()),
            // As cerdas do Shape entram no motor como o `texv` (`wetpaint/dab_route.rs`): um Grain armado
            // toma o lugar delas (W2.4), e as ferramentas da água que não depositam não as lêem.
            (
                Dependente::CerdasDaAgua,
                agua && b.shape.kind == ph2d_painter_brush::TextureKind::Bristles
                    && (b.grain_samples(self.paint.texture_image.is_some())
                        || (!self.paint.eraser
                            && self.paint.wetpaint.tool != super::WetTool::Paint)),
            ),
            // O Follow roda a pegada INTEIRA do dab (`BrushSpec::dab_rotor`): sem textura no Shape, sem ponta
            // achatada e sem um Grain preso ao dab, a ponta redonda não tem ângulo para mostrar.
            (Dependente::FollowSemNada, sem_lado),
            // A Texture Color colore a textura do Shape (prompt 38): sem textura não há o que colorir.
            (
                Dependente::TexturaSemForma,
                !b.shape_silhouette_active(self.paint.shape_image.is_some()),
            ),
            // A cápsula é uma lei da PONTA, em todo meio: não age num método sem Spacing (Dots,
            // Airbrush) nem sem caminho (Drag Dot, Anchored, Grid Stamp), num Spacing já tão fino quanto
            // o sub-passo, nem — na água — num passo que não passa de uma célula do motor
            // (`wetpaint/dab_route.rs`). Só a lêem as rotas do carimbo (Paint, a borracha, a Máscara) e a
            // água: Smear, Blur, Clone, Inpaint, Liquify, Sculpt e Knife têm kernel próprio (medido,
            // `sonda_p40_modos`: 0 texels de diferença com a caixa).
            (Dependente::ContinuoSemPasso, {
                let le = matches!(
                    self.paint.paint_mode,
                    super::PaintMode::Paint | super::PaintMode::Mask | super::PaintMode::WetPaint
                );
                // O raio do traço: na água o do MOTOR (a lei da largura, `dab_spec.rs`). ⛔ Não o
                // `stroke_spec()`: ele pergunta ao painel e voltava aqui (estouro de pilha, medido).
                let raio = match (agua, &self.paint.wetpaint.largura) {
                    (true, Some(lei)) => lei.raio_do_motor(b.radius_px),
                    _ => b.radius_px,
                };
                let s = BrushSpec {
                    continuous_spacing: true,
                    radius_px: raio,
                    ..*b
                };
                !le || !s.lays_capsules()
                    || (agua && s.dab_spacing_px() <= f32::from(self.paint.wetpaint.grid_ratio))
            }),
            // A cor da textura só a pintam as rotas do carimbo (`stamp_route.rs`, `is_color_mode`): a água
            // deposita a cor do pincel pela silhueta (`wet_owns_the_dabs`) e a aguada reconstrói-se da cor
            // do pincel (`watercolor_render_active`) — as duas saem antes dessas rotas (26 C1).
            (Dependente::CorDaTexturaSemLeitor, agua || aquarela),
            // O cartão Pen (L2), pela lei que o traço lê (`dinamica_do_traco`, `inclinacao_do_pincel`): a
            // curva da opacidade só com a pressão na opacidade; os ângulos e o atraso só com a caneta a mudar
            // o carimbo. (O grão alisado e a rotação NÃO esmaecem: ver `o_censo_so_encolhe`.)
            (
                Dependente::CurvaSemOpacidade,
                !self.dinamica_do_traco().strength_pressure,
            ),
            (Dependente::CanetaDePe, {
                let t = self.inclinacao_do_pincel();
                !t.active() && t.opacity <= 0.0
            }),
            // A Watercolor com a Shape «Automatic» não lê a pegada deitada (`inclinacao_dos_antigos`).
            (
                Dependente::PegadaDaAutomatica,
                aquarela && b.watercolor_shape_auto,
            ),
            // A contagem sorteada e a partícula só agem com spray (`DabJitter::particula`, `marca::conta`).
            (
                Dependente::MarcaSemSpray,
                b.spray_count <= 1 || !b.stroke_method.allows_jitter(),
            ),
            // A aguada não lê a cobertura do dab (`WASH_DEPOSIT_PEAK`, a mesma razão do cartão Pen).
            (Dependente::OpacidadeNaAguada, aquarela),
            // O fim do traço (L5): a ponta da cauda sem cauda (ou sem o plano que a re-pinta), o patamar sem a
            // carga que acaba; o plano (L8): a largura sem orla; e o modo sobre o vazio sem um modo de cor (L7).
            (
                Dependente::PontaDaCaudaSemCauda,
                !b.taper.end_active() || self.plano_recusado().is_some(),
            ),
            (Dependente::PatamarSemFallOff, !b.taper.fall_active()),
            // Na água a carga que acaba é o pigmento que o PINCEL pousa (`dab_route`): a borracha, Smear, Blend,
            // Wet, Dry e Blow não pousam pigmento.
            (
                Dependente::FallOffDaAgua,
                agua && (self.paint.eraser || self.paint.wetpaint.tool != super::WetTool::Paint),
            ),
            (
                Dependente::LarguraSemOrla,
                !b.traco.tem_orla() || self.plano_recusado().is_some(),
            ),
            (Dependente::VazioSemModoDeCor, !b.blend.e_de_cor()),
            (
                Dependente::PlanoForaDoTraco,
                self.plano_recusado() == Some(super::traco_plano::SemPlano::ForaDoTraco),
            ),
            // A cor pela mão muda a cor do DAB: uma Color Ramp dona da cor (no carimbo e na aguada) ou a Texture
            // Color de uma Shape (só nas rotas do carimbo, `stamp_route.rs`) pintam a delas; na água só o depósito
            // (Paint) pousa a cor do dab — Smear, Blend, Wet, Dry e Blow não (a lei do `GrainDaAgua`).
            (
                Dependente::CorDaMaoSemLeitor,
                (agua && !self.paint.eraser && self.paint.wetpaint.tool != super::WetTool::Paint)
                    || (!agua
                        && (self.color_ramp_owner(b.texture.is_active())
                            != super::ramp_lut::RampLutOwner::None
                            || (!aquarela
                                && self.paint.shape_layers.is_color_mode()
                                && b.shape_silhouette_active(self.paint.shape_image.is_some())))),
            ),
            // Sem pressão no tamanho (nem na opacidade) a curva dele (e o piso, o tecto, a suavização da mão)
            // não têm canal que leia a pressão. Só nos meios antigos: no Wet a pressão chega também ao fluido.
            (Dependente::CurvaDoTamanhoSemPressao, {
                let d = self.dinamica_do_traco();
                !agua && (!d.size_pressure || d.size_min >= 1.0)
            }),
            // A profundidade por dab é `max(min, 1 − jitter·(1 − x))` (`GrainLaw::profundidade_do_dab`): com jitter 0
            // nenhum dab varia (`varia_por_dab`) e o mínimo e o controlo (pressão/sorteio) não têm o que decidir.
            (
                Dependente::ProfundidadeSemJitter,
                !b.texture.lei.varia_por_dab(),
            ),
            // O dual brush (L6): a MESMA porta que arma a 2.ª ponta (`dual_recusado`: o plano no Digital, o pincel
            // que pousa no Wet).
            (
                Dependente::DualForaDoDigital,
                self.dual_recusado() == Some(super::traco_plano::SemPlano::ForaDoDigital),
            ),
            (
                Dependente::DualForaDoTraco,
                self.dual_recusado() == Some(super::traco_plano::SemPlano::ForaDoTraco),
            ),
            (Dependente::MaoSemPressao, {
                let d = self.dinamica_do_traco();
                !agua && (!d.size_pressure || d.size_min >= 1.0) && !d.strength_pressure
            }),
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
