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

mod do_pincel; // `PainterTool::inercias` — filho por LOC

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
    CorDoGraoDaAgua,
    LeiDoGraoNaAguada,
    TexturaSemForma,
    CurvaSemOpacidade,
    CanetaDePe,
    PegadaDaAutomatica,
    CurvaDoTamanhoSemPressao,
    MaoSemPressao,
    MarcaSemSpray,
    OpacidadeNaAguada,
    CorDaMaoSemLeitor,
    // O fim do traço (L5), re-pintado no plano do traço (`traco_plano`, o Digital); o modo sobre o vazio (L7).
    PontaDaCaudaSemCauda,
    PatamarSemFallOff,
    VazioSemModoDeCor,
    PlanoForaDoTraco,
    FallOffDaAgua,
    // A profundidade do grão por dab (L9): o mínimo e o controlo só leem o que o jitter faz variar.
    ProfundidadeSemJitter,
    // O dual brush (L6) recorta o traço composto como CAMADA (`traco_plano`, o Digital) ou o pigmento que pousa (o
    // Wet): a aguada e o Impasto não; e só num traço livre que pinta, apaga, borra, esfrega ou clona (`dual_ferramenta`).
    DualForaDoDigital,
    DualForaDoTraco,
    // O atraso da velocidade (J4) só filtra a velocidade que uma lei da velocidade lê.
    AtrasoSemVelocidade,
    // A pressão leve e a velocidade só TIRAM dureza (`Dynamics::dureza_da_mao`): com a Hardness 0 não há o que tirar.
    MaciezSemDureza,
    // O Wash é a lei do Accumulate DESLIGADO (`stroke_cover::cover_max`).
    WashComAccumulate,
    // O material do Real Oil no Tuning (Depth, Gloss, Paper Press): sem o Real Oil a célula é NEUTRA
    // (`grid/oleo_celula.rs::do_pincel`) e o relevo do papel na tinta nem se soma (`oleo_real.rs`).
    MaterialSemOleo,
}

impl Dependente {
    pub const TODOS: [Dependente; 44] = [
        Dependente::ForaDaPontaDePixel,
        Dependente::TaperTip,
        Dependente::TaperOpacity,
        Dependente::Accumulate,
        Dependente::SpaceAttenuation,
        Dependente::JitterUnit,
        Dependente::DashLength,
        Dependente::PaperRoughness,
        Dependente::Paper,
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
        Dependente::CorDoGraoDaAgua,
        Dependente::LeiDoGraoNaAguada,
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
        Dependente::VazioSemModoDeCor,
        Dependente::PlanoForaDoTraco,
        Dependente::FallOffDaAgua,
        Dependente::ProfundidadeSemJitter,
        Dependente::DualForaDoDigital,
        Dependente::DualForaDoTraco,
        Dependente::AtrasoSemVelocidade,
        Dependente::MaciezSemDureza,
        Dependente::WashComAccumulate,
        Dependente::MaterialSemOleo,
    ];

    /// Os ids que o painel pinta para este controlo.
    #[must_use]
    pub fn ids(self) -> &'static [NodeId] {
        match self {
            Dependente::ForaDaPontaDePixel => &FORA_DA_PONTA_DE_PIXEL,
            Dependente::TaperTip => &[ids::PAINTER_TAPER_TIP_START],
            Dependente::TaperOpacity => &[ids::PAINTER_TAPER_OPACITY],
            Dependente::Accumulate => &[ids::PAINTER_BRUSH_ACCUMULATE],
            Dependente::WashComAccumulate => &[ids::PAINTER_BRUSH_WASH],
            Dependente::SpaceAttenuation => &[ids::PAINTER_BRUSH_SPACE_ATTEN],
            Dependente::JitterUnit => &[ids::PAINTER_BRUSH_JITTER_UNIT],
            Dependente::DashLength => &[
                ids::PAINTER_BRUSH_DASH_LENGTH,
                ids::PAINTER_BRUSH_DASH_LENGTH_CHIP,
            ],
            Dependente::PaperRoughness => &[ids::PAINTER_SUBSTRATE_ROUGHNESS],
            Dependente::Paper => &[ids::PAINTER_WATERCOLOR_PAPER_KIND],
            Dependente::GrainDaAgua => &GRAIN,
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
            Dependente::CorDoGraoDaAgua => &[ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE],
            Dependente::LeiDoGraoNaAguada => &[
                ids::PAINTER_PONTA_FIELDS[1],
                ids::PAINTER_PONTA_FIELDS[2],
                ids::PAINTER_PONTA_FIELDS[3],
                ids::PAINTER_PONTA_FIELDS[4],
            ],
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
            Dependente::VazioSemModoDeCor => &[ids::PAINTER_BRUSH_BLEND_ON_EMPTY],
            Dependente::PlanoForaDoTraco => &[ids::PAINTER_TAPER_END, ids::PAINTER_TAPER_TIP_END],
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
            // Min/Max pressure, Smoothing e a curva em S: o caminho da pressão que todo leitor dela lê.
            Dependente::MaoSemPressao => &[
                ids::PAINTER_PEN_FIELDS[4],
                ids::PAINTER_PEN_FIELDS[5],
                ids::PAINTER_PEN_FIELDS[6],
                ids::PAINTER_PEN_FIELDS[18],
            ],
            // Speed softness e a da pressão, pela ordem de `LINHAS_DA_MAO`.
            Dependente::MaciezSemDureza => {
                &[ids::PAINTER_PEN_FIELDS[21], ids::PAINTER_PEN_FIELDS[22]]
            }
            // «Speed lag», pela ordem de `LINHAS_DA_MAO`.
            Dependente::AtrasoSemVelocidade => &[ids::PAINTER_PEN_FIELDS[20]],
            Dependente::MaterialSemOleo => material_do_oleo(),
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
            Dependente::CorDoGraoDaAgua => "panel.painter_layers.inerte.paint_tool_only",
            Dependente::LeiDoGraoNaAguada => "panel.painter_ponta.inerte.wash_grain_law",
            Dependente::TexturaSemForma => {
                "panel.painter_layers.inerte.texture_color_needs_a_shape"
            }
            Dependente::CurvaSemOpacidade => "panel.painter_layers.inerte.pressure_opacity_on",
            Dependente::CanetaDePe => "panel.painter_layers.inerte.tilt_law_on",
            Dependente::PegadaDaAutomatica => "panel.painter_layers.inerte.shape_automatic_off",
            Dependente::CurvaDoTamanhoSemPressao => "panel.painter_layers.inerte.pressure_size_on",
            Dependente::MaoSemPressao => "panel.painter_layers.inerte.pressure_on",
            Dependente::MaciezSemDureza => "panel.painter_layers.inerte.hardness_above_zero",
            Dependente::WashComAccumulate => "panel.painter_layers.inerte.accumulate_off",
            Dependente::MarcaSemSpray => "panel.painter_layers.inerte.spray_count_above_one",
            Dependente::OpacidadeNaAguada => "panel.painter_layers.inerte.wash_ignores_dab_opacity",
            Dependente::CorDaMaoSemLeitor => "panel.painter_layers.inerte.color_owned_by_ramp",
            Dependente::PontaDaCaudaSemCauda => "panel.painter_layers.inerte.taper_end_length",
            Dependente::PatamarSemFallOff => "panel.painter_layers.inerte.fall_off_length",
            Dependente::VazioSemModoDeCor => "panel.painter_layers.inerte.color_blend_mode",
            Dependente::PlanoForaDoTraco => "panel.painter_layers.inerte.freehand_paint_stroke",
            Dependente::FallOffDaAgua => "panel.painter_layers.inerte.wet_paint_tool",
            Dependente::ProfundidadeSemJitter => {
                "panel.painter_layers.inerte.depth_jitter_above_zero"
            }
            Dependente::DualForaDoDigital => "panel.painter_ponta.inerte.dual_medium",
            Dependente::DualForaDoTraco => "panel.painter_layers.inerte.freehand_paint_stroke",
            Dependente::AtrasoSemVelocidade => "panel.painter_layers.inerte.speed_on",
            Dependente::MaterialSemOleo => "panel.wet_tuning.inerte.real_oil_on",
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

/// As linhas do Tuning que só o Real Oil lê: o slider e o chip de cada (os ids do Tuning hasheiam-se em execução).
fn material_do_oleo() -> &'static [NodeId] {
    static IDS: std::sync::OnceLock<Vec<NodeId>> = std::sync::OnceLock::new();
    IDS.get_or_init(|| {
        MATERIAL_DO_OLEO
            .iter()
            .flat_map(|k| [ids::wet_tuning_slider_id(k), ids::wet_tuning_chip_id(k)])
            .collect()
    })
}

/// As chaves do Tuning que só o Real Oil lê ([`Dependente::MaterialSemOleo`]).
pub const MATERIAL_DO_OLEO: [&str; 3] = ["oilDepth", "oilGloss", "paperPress"];

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
