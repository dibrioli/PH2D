//! **A migração dos presets para os controlos da E10** por FUNÇÃO (`docs/Painter/Arquitetura/F3_controlos.md` §3 e
//! §9): um preset sem a marca `controlosE10` ganha Dilution e Volume exactos (a mesma `φ` e o mesmo `h` da receita
//! da E4), o Medium pela FAMÍLIA (§3.2) e o Body do `levelingOwnStroke`. Corre DEPOIS da E4 ([`migra_a_receita`], que mora aqui por LOC).

use super::super::WetKnobs;
use ph2d_brush_presets::Parametros;
use ph2d_wet_paint::tuning::Knob;

/// A marca da E10 no mapa `engine` (`1`): o preset já traz Dilution, Volume, Medium e Body.
pub(in crate::tool::paint) const CHAVE_CONTROLOS: &str = "controlosE10";

/// O Body que dá o MESMO traço isolado que a parede `levelingOwnStroke = 1` dava a um pincel SEM meio (Wet Liner,
/// Wet Wash, Gouache Opaque); no óleo o meio já dá a cedência (Body 0). Medido: `E10a2_meio_e_corpo.md`.
pub(in crate::tool::paint) const BODY_DO_LEVELING: f64 =
    ph2d_wet_paint::painter::receita::BODY_DA_PAREDE;

/// A chave da parede binária de antes da E10 (`Knob::LevelingOwnStroke`, fora do registo): só a migração a lê.
pub(in crate::tool::paint) const CHAVE_PAREDE: &str = "levelingOwnStroke";

/// **O meio por família** (F3 §3.2, dono 01/11 Q5): `familia` = a pasta do meio (`Oil`, `Pastel`…), `grupo` = a
/// subpasta. Óleo e Express Oil `1`; Acrylic (o Fan Brush) `0,5`; os pastéis de óleo e cera (`Pastel/Oil`,
/// `Pastel/Scrape`) `1`; o resto `0` (água, tintas, lápis, e as ferramentas Blend/Smear, que não pousam tinta).
#[must_use]
pub(in crate::tool::paint) fn meio_da_familia(familia: &str, grupo: &str) -> f64 {
    ph2d_wet_paint::painter::receita::meio_da_familia(familia, grupo)
}

/// A família de `prm`: a `library_folder` (o exportado, ou quem aplica a partir da biblioteca), senão a dos
/// grupos de fábrica que só uma família tem — um grupo desconhecido dá `""` (meio `0`).
// A ponte (`painter_bridge_presets.rs::aplicar`) põe a `library_folder` da biblioteca antes de aplicar (E10c2).
#[must_use]
pub(in crate::tool::paint) fn familia_do_preset(prm: &Parametros) -> &str {
    if let Some(f) = prm.library_folder.as_deref() {
        return f;
    }
    let pinta = prm.paint_mode.as_deref().is_none_or(|m| m == "Paint");
    match prm.group.as_str() {
        "Dabber" | "Knife" | "Metallic" if pinta => "Oil",
        "Flat" if pinta => "Oil",
        "Round" if pinta => "ExpressOil",
        "Fan" if pinta => "Acrylic",
        "Oil" | "Scrape" | "Dry" if pinta => "Pastel",
        _ => "",
    }
}

/// A água e a Opacity do preset, ou as de agora quando ele não as fixa (a mesma regra na E4 e na E10).
fn agua_e_opacidade(prm: &Parametros, agua: f64, opacidade: f64) -> (f64, f64) {
    let agua = prm
        .volumes
        .water
        .filter(|w| w.is_finite())
        .map_or(agua, |w| w.clamp(0.0, 1.0));
    let opacidade = prm
        .volumes
        .opacity
        .filter(|o| o.is_finite())
        .map_or(opacidade, |o| o.clamp(0.0, 1.0));
    (agua, opacidade)
}

/// **A migração da E4** (ADR-0197 §1.1): um preset da lei velha pousa o MESMO pigmento e a MESMA água por dab
/// (à pressão de referência), agora como receita (`pigmentPerDab`) × quantidade (`intensity`).
pub(in crate::tool::paint) fn migra_a_receita(
    k: &mut WetKnobs,
    prm: &Parametros,
    agua: f64,
    opacidade: f64,
) {
    let (agua, opacidade) = agua_e_opacidade(prm, agua, opacidade);
    let (pig, carga) = ph2d_wet_paint::painter::receita::migra_da_lei_velha(
        k.get(Knob::PigmentPerDab),
        k.get(Knob::WaterPerDab),
        agua,
        opacidade,
    );
    k.set(Knob::PigmentPerDab, pig);
    k.set(Knob::Intensity, carga);
}

/// **`migra_para_controlos`** sobre os knobs já migrados pela E4: `agua` e `opacidade` = as do preset (ou as de
/// agora quando ele não as fixa, a mesma regra da E4). Escreve Dilution, Volume, Medium e Body.
pub(in crate::tool::paint) fn migra_para_controlos(
    k: &mut WetKnobs,
    prm: &Parametros,
    agua: f64,
    opacidade: f64,
) {
    let (agua, opacidade) = agua_e_opacidade(prm, agua, opacidade);
    let (diluicao, volume) = ph2d_wet_paint::painter::receita::migra_para_controlos(
        k.get(Knob::PigmentPerDab),
        k.get(Knob::WaterPerDab),
        agua,
        k.get(Knob::Intensity),
        opacidade,
    );
    k.set(Knob::Dilution, diluicao);
    k.set(Knob::Volume, volume);
    let meio = meio_da_familia(familia_do_preset(prm), &prm.group);
    k.set(Knob::Medium, meio);
    // A parede saiu do registo (E10c2): a chave do preset lê-se pelo NOME, só aqui.
    let parede = prm.engine.get(CHAVE_PAREDE).is_some_and(|v| *v >= 0.5) && meio == 0.0;
    k.set(Knob::Body, if parede { BODY_DO_LEVELING } else { 0.0 });
}

// ── A faixa das miniaturas das ferramentas da água (`preview.rs`), aqui por LOC ──
/// O pigmento por dab da faixa no Wet — o da Acrylic Round de fábrica (a tinta cheia que as ferramentas movem).
pub(in crate::tool::paint) const PIGMENTO_DA_FAIXA: f64 = 500.0;
/// …e a água dela (o volume Water da Acrylic Round: a tinta de corpo, não a aguada).
pub(in crate::tool::paint) const AGUA_DA_FAIXA: f64 = 0.05;

/// Os knobs da faixa: os do pincel SEM preset com o pigmento e a água da Acrylic Round, pela receita da E4
/// (ADR-0197 §1.1) — a faixa pousa o MESMO pigmento e a MESMA água por dab que na lei velha (`PIGMENTO_DA_FAIXA` ×
/// a Opacity, a água de `AGUA_DA_FAIXA`), como a Acrylic Round migrada ([`migra_a_receita`]).
pub(in crate::tool::paint) fn knobs_da_faixa(opacidade: f64) -> WetKnobs {
    let mut k = WetKnobs::DEFAULT;
    k.water = AGUA_DA_FAIXA;
    let (pig, carga) = ph2d_wet_paint::painter::receita::migra_da_lei_velha(
        PIGMENTO_DA_FAIXA,
        k.get(Knob::WaterPerDab),
        AGUA_DA_FAIXA,
        opacidade,
    );
    k.set(Knob::PigmentPerDab, pig);
    k.set(Knob::Intensity, carga);
    // E10: a receita lê Dilution e Volume — os mesmos `φ` e `h` (a Opacity dobrada no Volume).
    let (d, v) = ph2d_wet_paint::painter::receita::migra_para_controlos(
        pig,
        k.get(Knob::WaterPerDab),
        AGUA_DA_FAIXA,
        carga,
        opacidade,
    );
    k.set(Knob::Dilution, d);
    k.set(Knob::Volume, v);
    k
}
