//! ⭐⭐⭐ **OS NOMES QUE AS CAMADAS DE AJUSTE PINTAM — da crate de efeitos para a tabela de strings.**
//!
//! A crate de efeitos devolve, para cada slot, um rótulo em inglês (`adjustment_slider_params` e as
//! irmãs) — e até 2026-09-16 o painel pintava-o **cru**: `Shad Amt`, `High Wid`, `Preserve Lum.`,
//! abreviados para caber numa coluna de `44 px`. ⛔ A régua lexical do painel não os via, porque o
//! literal mora noutra crate: *um texto pintado que nasce do outro lado de uma fronteira é invisível
//! a um censo por crate.*
//!
//! ⇒ aqui o rótulo da crate é um **identificador** (o braço de um `match`), e o que se pinta é a
//! chave. ⚠️ **O `match` é POR FAMÍLIA** porque o mesmo rótulo diz coisas diferentes: `Red` é um
//! canal no misturador e um *desvio* na aberração cromática; `Contrast` é o contraste geral e o de
//! meios-tons nas sombras/luzes; `Tint` é o interruptor e a quantidade no preto-e-branco.
//!
//! O gate `cada_rotulo_de_ajuste_tem_nome_na_tabela` varre todas as espécies (com o tinte ligado, e
//! cada saída do misturador) e exige uma chave para cada rótulo, e nomes distintos dentro da pilha.

use ph2d_i18n::tr;
use ph2d_tool_painter::{AdjustmentKind, AdjustmentParams};

/// O nome a pintar para uma chave encontrada — ou o rótulo da crate, se a tabela não o conhece.
///
/// ⚠️ **O rótulo cru é a DEGRADAÇÃO, não o caminho:** o gate exige que o `None` nunca aconteça, e
/// um nome em inglês abreviado ainda diz mais ao artista do que uma linha sem nome.
pub(crate) fn pintar(chave: Option<&'static str>, fonte: &'static str) -> &'static str {
    debug_assert!(
        chave.is_some(),
        "o rotulo {fonte:?} da crate de efeitos nao tem chave em `adjust_nomes`"
    );
    chave.map_or(fonte, tr)
}

/// A chave do nome da barra `fonte` de uma pilha genérica (e da do preto-e-branco).
#[must_use]
pub fn chave_da_barra(params: &AdjustmentParams, fonte: &str) -> Option<&'static str> {
    match params {
        AdjustmentParams::ChromaticAberration(_) => desvio(fonte),
        AdjustmentParams::ShadowsHighlights(_) => sombras_e_luzes(fonte),
        AdjustmentParams::BlackAndWhite(_) => preto_e_branco(fonte),
        AdjustmentParams::Levels(_) => niveis(fonte),
        AdjustmentParams::Threshold(_) => limiar(fonte),
        _ => comum(fonte),
    }
}

/// A chave de uma barra do misturador de canais.
#[must_use]
pub fn chave_do_misturador(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Const" => "panel.painter_layers.adjust.constant",
        _ => return canal(fonte),
    })
}

/// A chave de uma barra da cor selectiva.
#[must_use]
pub fn chave_da_seletiva(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Cyan" => "panel.painter_layers.adjust.cyan",
        "Mag" => "panel.painter_layers.adjust.magenta",
        "Yel" => "panel.painter_layers.adjust.yellow",
        "Blk" => "panel.painter_layers.adjust.black",
        _ => return None,
    })
}

/// A chave de uma barra da paragem do mapa de gradiente.
#[must_use]
pub fn chave_do_gradiente(fonte: &str) -> Option<&'static str> {
    canal(fonte)
}

/// A chave de um interruptor.
#[must_use]
pub fn chave_do_interruptor(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Preserve Lum." => "panel.painter_layers.adjust.preserve_luminosity",
        "Monochrome" => "panel.painter_layers.adjust.monochrome",
        "Tint" => "panel.painter_layers.adjust.tint",
        _ => return None,
    })
}

/// A chave de uma opção do grupo segmentado.
#[must_use]
pub fn chave_do_segmento(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Shadows" => "panel.painter_layers.adjust.shadows",
        "Midtones" => "panel.painter_layers.adjust.midtones",
        "Highlights" => "panel.painter_layers.adjust.highlights",
        "Linear" => "panel.painter_layers.adjust.linear",
        "Smooth" => "panel.painter_layers.adjust.smooth",
        "Relative" => "panel.painter_layers.adjust.relative",
        "Absolute" => "panel.painter_layers.adjust.absolute",
        "Gaussian" => "panel.painter_layers.adjust.gaussian",
        "Uniform" => "panel.painter_layers.adjust.uniform",
        "Dot" => "panel.painter_layers.adjust.dot",
        "Line" => "panel.painter_layers.adjust.line",
        "Circle" => "panel.painter_layers.adjust.circle",
        _ => return None,
    })
}

/// A chave do nome de uma ESPÉCIE, como o menu «+ Adjustment» a lista.
///
/// ⚠️ Exaustivo de propósito: uma espécie nova é erro de compilação AQUI.
#[must_use]
pub fn chave_da_especie(especie: AdjustmentKind) -> &'static str {
    match especie {
        AdjustmentKind::HueSaturationBrightness => {
            "panel.painter_layers.adjust.kind.hue_saturation"
        }
        AdjustmentKind::ColorBalance => "panel.painter_layers.adjust.kind.color_balance",
        AdjustmentKind::Curves => "panel.painter_layers.adjust.kind.curves",
        AdjustmentKind::GradientMap => "panel.painter_layers.adjust.kind.gradient_map",
        AdjustmentKind::BrightnessContrast => {
            "panel.painter_layers.adjust.kind.brightness_contrast"
        }
        AdjustmentKind::GaussianBlur => "panel.painter_layers.adjust.kind.gaussian_blur",
        AdjustmentKind::MotionBlur => "panel.painter_layers.adjust.kind.motion_blur",
        AdjustmentKind::Bloom => "panel.painter_layers.adjust.kind.bloom",
        AdjustmentKind::Noise => "panel.painter_layers.adjust.kind.noise",
        AdjustmentKind::Sharpen => "panel.painter_layers.adjust.kind.sharpen",
        AdjustmentKind::Halftone => "panel.painter_layers.adjust.kind.halftone",
        AdjustmentKind::ChromaticAberration => {
            "panel.painter_layers.adjust.kind.chromatic_aberration"
        }
        AdjustmentKind::Vibrance => "panel.painter_layers.adjust.kind.vibrance",
        AdjustmentKind::ColorLookupLut => "panel.painter_layers.adjust.kind.color_lookup",
        AdjustmentKind::PhotoFilter => "panel.painter_layers.adjust.kind.photo_filter",
        AdjustmentKind::Posterize => "panel.painter_layers.adjust.kind.posterize",
        AdjustmentKind::Threshold => "panel.painter_layers.adjust.kind.threshold",
        AdjustmentKind::Invert => "panel.painter_layers.adjust.kind.invert",
        AdjustmentKind::Levels => "panel.painter_layers.adjust.kind.levels",
        AdjustmentKind::SelectiveColor => "panel.painter_layers.adjust.kind.selective_color",
        AdjustmentKind::ChannelMixer => "panel.painter_layers.adjust.kind.channel_mixer",
        AdjustmentKind::Exposure => "panel.painter_layers.adjust.kind.exposure",
        AdjustmentKind::ShadowsHighlights => "panel.painter_layers.adjust.kind.shadows_highlights",
        AdjustmentKind::BlackAndWhite => "panel.painter_layers.adjust.kind.black_white",
    }
}

/// Os três canais de cor, com o nome do canal.
fn canal(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Red" => "panel.painter_layers.adjust.red",
        "Green" => "panel.painter_layers.adjust.green",
        "Blue" => "panel.painter_layers.adjust.blue",
        _ => return None,
    })
}

/// Os três canais da aberração cromática — ali cada um é um DESVIO.
fn desvio(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Red" => "panel.painter_layers.adjust.red_shift",
        "Green" => "panel.painter_layers.adjust.green_shift",
        "Blue" => "panel.painter_layers.adjust.blue_shift",
        _ => return None,
    })
}

fn sombras_e_luzes(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Shad Amt" => "panel.painter_layers.adjust.shadows_amount",
        "Shad Wid" => "panel.painter_layers.adjust.shadows_width",
        "Shad Rad" => "panel.painter_layers.adjust.shadows_radius",
        "High Amt" => "panel.painter_layers.adjust.highlights_amount",
        "High Wid" => "panel.painter_layers.adjust.highlights_width",
        "High Rad" => "panel.painter_layers.adjust.highlights_radius",
        "Color" => "panel.painter_layers.adjust.color_correction",
        "Contrast" => "panel.painter_layers.adjust.midtone_contrast",
        _ => return None,
    })
}

fn preto_e_branco(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Reds" => "panel.painter_layers.adjust.reds",
        "Yellows" => "panel.painter_layers.adjust.yellows",
        "Greens" => "panel.painter_layers.adjust.greens",
        "Cyans" => "panel.painter_layers.adjust.cyans",
        "Blues" => "panel.painter_layers.adjust.blues",
        "Magentas" => "panel.painter_layers.adjust.magentas",
        "Tint" => "panel.painter_layers.adjust.tint_amount",
        _ => return comum(fonte),
    })
}

fn niveis(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Black" => "panel.painter_layers.adjust.input_black",
        "White" => "panel.painter_layers.adjust.input_white",
        "Out Lo" => "panel.painter_layers.adjust.output_black",
        "Out Hi" => "panel.painter_layers.adjust.output_white",
        _ => return comum(fonte),
    })
}

fn limiar(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Level" => "panel.painter_layers.adjust.threshold",
        _ => return None,
    })
}

/// O vocabulário partilhado pelas pilhas genéricas.
fn comum(fonte: &str) -> Option<&'static str> {
    Some(match fonte {
        "Hue" => "panel.painter_layers.adjust.hue",
        "Sat" => "panel.painter_layers.adjust.saturation",
        "Bright" => "panel.painter_layers.adjust.brightness",
        "Contrast" => "panel.painter_layers.adjust.contrast",
        "Expo" => "panel.painter_layers.adjust.exposure",
        "Offset" => "panel.painter_layers.adjust.offset",
        "Gamma" => "panel.painter_layers.adjust.gamma",
        "Vib" => "panel.painter_layers.adjust.vibrance",
        "Levels" => "panel.painter_layers.adjust.levels",
        "Temp" => "panel.painter_layers.adjust.temperature",
        "Density" => "panel.painter_layers.adjust.density",
        "C/R" => "panel.painter_layers.adjust.cyan_red",
        "M/G" => "panel.painter_layers.adjust.magenta_green",
        "Y/B" => "panel.painter_layers.adjust.yellow_blue",
        "Radius" => "panel.painter_layers.adjust.radius",
        "Distance" => "panel.painter_layers.adjust.distance",
        "Angle" => "panel.painter_layers.adjust.angle",
        "Amount" => "panel.painter_layers.adjust.amount",
        "Dot Size" => "panel.painter_layers.adjust.dot_size",
        "Look" => "panel.painter_layers.adjust.look",
        "Threshold" => "panel.painter_layers.adjust.threshold",
        "Intensity" => "panel.painter_layers.adjust.intensity",
        "Falloff" => "panel.painter_layers.adjust.falloff",
        _ => return None,
    })
}
