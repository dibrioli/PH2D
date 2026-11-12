//! O tom de cada selo de linha da Hierarquia (saiu de `row.rs` para o tecto de LOC do painel).

use ph2d_editor_core::widget::TagTone;

/// **O TOM de cada selo de linha.** Porta única: o selo é um código, e o tom é o que o olho lê
/// antes de decifrar as três letras.
///
/// ⚠️ Ela saiu de dentro do `paint_hierarchy_row` em 2026-08-22, quando os selos do papel booleano
/// empurraram aquela função para fora do teto de LOC. ⛔ **Split, nunca allowlist** — e o número
/// do allowlist desceu junto, que é a única direção em que ele pode andar.
pub(crate) fn badge_tone(badge: &str) -> TagTone {
    match badge {
        "PRF" | "SPR" => TagTone::Accent,
        "OUT" | "LGT" => TagTone::Warn,
        "CAM" => TagTone::Success,
        "TRG" => TagTone::Danger,
        // O PAPEL de uma forma dentro de uma booleana viva (2026-08-22). O tom separa **o que
        // acrescenta** do **o que tira** — a leitura que o olho faz ao percorrer a receita.
        //
        // ⚠️ `UNI` cai no Neutral por já existir acima com outro significado, e ⛔ **não se muda o
        // tom daquele** para acomodar este: seria repintar um selo de outra família.
        "SUB" => TagTone::Warn,
        "INT" | "EXC" => TagTone::Accent,
        // A BASE não tem verbo e a RECEITA é do grupo inteiro: nenhum dos dois é escolha daquela
        // linha, e o neutro é o que os separa dos que são.
        _ => TagTone::Neutral,
    }
}

/// ⛔⛔ **O SELO É PINTADO **E** É A CHAVE DO TOM** — o gate que nomeia essa dupla vida.
#[cfg(test)]
mod badge_tests {
    use super::badge_tone;
    use ph2d_editor_core::widget::TagTone;

    /// ⚠️⚠️ **[`badge_tone`] casa contra o TEXTO PINTADO**, e desde 2026-09-16 esse texto vem da
    /// tabela de strings (`app.vec.*`, `shell.*` — HR-15). As duas coisas convivem porque a
    /// tabela devolve exactamente o mesmo código; **o dia em que uma língua traduzir `SUB` o selo
    /// perde a cor em silêncio**, e é esse dia que este teste apanha.
    ///
    /// ⇒ a cura, quando ele reprovar, **não é reverter a tabela**: é o selo viajar como ID até
    /// aqui (a lição *identidade ≠ exibição*), e o tom ser escolhido pelo id. Enquanto o app for
    /// inglês-only, isto é uma cerca executável em vez de uma nota que envelhece.
    #[test]
    fn the_badge_tone_still_matches_what_the_table_paints() {
        for (chave, tom) in [
            ("shell.hero_bridge.prf", TagTone::Accent),
            ("panel.hierarchy.badge.entity", TagTone::Neutral),
            ("app.vec.bool_shape.sub", TagTone::Warn),
            ("app.vec.bool_shape.int", TagTone::Accent),
            ("app.vec.bool_shape.exc", TagTone::Accent),
        ] {
            let pintado = ph2d_i18n::tr(chave);
            assert_eq!(
                badge_tone(pintado),
                tom,
                "o selo `{chave}` pinta {pintado:?} e a tabela de TOM não o conhece — ela casa \
                 contra o texto, logo traduzir um selo apaga-lhe a cor. A cura é o selo viajar \
                 como ID até esta função, nunca reverter a tabela para um literal."
            );
        }
    }
}
