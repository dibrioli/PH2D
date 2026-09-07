//! ⭐⭐⭐ **A LEI DO APELIDO** — irmã do [`crate::color`], que estava no tecto de LOC.
//!
//! Um apelido não tem valor: ele tem **pai**. Este ficheiro é a tabela dessa relação e a razão
//! dela; a fábrica em `color.rs` resolve através dela.

use crate::ColorToken;

/// ⭐⭐⭐ **O slot GERAL de que este token nasce, quando ele é um APELIDO.**
///
/// Os 16 `timeline-*` são apelidos: cada um resolve exactamente para um slot geral, em todos
/// os temas. Isso era, até 2026-09-07, uma **coincidência medida** — 57 valores escritos à mão
/// no `tokens.json` (16 no `forge`, 16 no `sunstone`, 16 no `blueprint`, 9 no `workshop`) que
/// um gate comparava par a par. Hoje é **construção**: não há valor a escrever, logo não há
/// valor que possa divergir por engano.
///
/// # ⛔⛔ Porque os NOMES ficam, e a ordem era «fundir»
///
/// O dono mandou fundir os apelidos (2026-09-07) sobre uma nota de 30/08 que prometia
/// *«83 → 67 slots por tema, zero pixels»*. ⚠️ **A medição partiu a ordem em duas metades que a
/// nota não separava:**
///
/// - **os VALORES** eram duplicação a sério — 57 cópias mantidas à mão, e é isso que esta
///   função apaga;
/// - **os NOMES** não são duplicação: cada um é uma linha REGULÁVEL no painel *Tokens* e um
///   destino de vínculo no editor vetorial. Apagá-los tirava ao artista 16 controlos que ele
///   tem hoje, e transformava *«a playhead passa a ter cor própria»* de uma edição de token em
///   sete linhas de código.
///
/// ⭐ **E o estado da arte, que o dono mandou consultar, diz o mesmo:** o manual do Blender
/// (CC-BY-SA) descreve o tema como *«as cores de cada editor podem ser definidas
/// separadamente»*, com uma secção por editor; o `theme_modern.cpp` do Godot (MIT) escreve
/// cor por CONTROLO (`font_color` de `Tree`, de `Button`, …). *As duas referências mantêm
/// tokens de componente e expõem-nos — o que elas não fazem é escrever o valor deles à mão.*
///
/// ⚠️ **E metade da ordem já estava feita sem ninguém saber:** a família moderna deriva tudo
/// desde a wave 1, logo ali os 16 nunca tiveram valor próprio. A nota de 30/08 descrevia um
/// mundo que a wave 1 já tinha mudado — §0.0: *quem move o número que tornava algo
/// inalcançável tem de reconferir a nota*.
#[must_use]
pub(crate) const fn parent_of(t: ColorToken) -> Option<ColorToken> {
    Some(match t {
        ColorToken::TimelineCurve
        | ColorToken::TimelineHandle
        | ColorToken::TimelineKeySelected
        | ColorToken::TimelineLoopBrace
        | ColorToken::TimelinePlayhead
        | ColorToken::TimelineSummaryRing => ColorToken::Accent,
        ColorToken::TimelineHandleLine | ColorToken::TimelineLoopRegion => ColorToken::AccentSoft,
        ColorToken::TimelineRowAlt | ColorToken::TimelineRulerBg => ColorToken::Bg2,
        ColorToken::TimelineMarker | ColorToken::TimelineSummaryKey => ColorToken::Warn,
        ColorToken::TimelineKeyActive => ColorToken::AccentPress,
        ColorToken::TimelineMissing => ColorToken::Danger,
        ColorToken::TimelineKey => ColorToken::Text1,
        ColorToken::TimelineRulerTick => ColorToken::Text3,
        _ => return None,
    })
}
