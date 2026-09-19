//! ⭐⭐⭐ **A DISPOSIÇÃO de um grupo segmentado** — quantas peças por fileira, e que largura tem
//! cada uma.
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super::segmented_adaptive`]** (corte de 2026-09-19, imposto
//! pelo tecto de 500 LOC do widget): aquele é o widget TIPADO (a lista de opções, a a11y, o
//! `selected`); este é a **lei da disposição**, que o pintor de chrome
//! ([`super::panel_chrome::paint_segmented_group_adaptive`]) e o medidor de altura consomem.
//! ⛔ *A cura de um tecto é o corte, nunca uma entrada no `FILE_OVERAGE_OK`* — e o corte é honesto:
//! o que o widget É e como ele se ARRUMA crescem por motivos diferentes.
//!
//! ⚠️⚠️ **As três leis daqui são lidas pelo PINTOR e pelo MEDIDOR**, e isso é estrutural e não
//! arrumação: *um contentor medido por uma regra e preenchido por outra é como a secção seguinte
//! pinta por cima destes botões e lhes mata o alvo*.

use super::panel_chrome::segmented_gap;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, TypeToken};

/// Each label's natural width — the text plus the canonical breathing room. The paint side and the
/// measure side must agree to the pixel, so they both come here.
pub(crate) fn segmented_natural_widths(labels: &[&str], text_system: &mut TextSystem) -> Vec<f32> {
    let font_size = TypeToken::Sm.px();
    let pad_inside = Spacing::Lg.px() * 2.0;
    labels
        .iter()
        .map(|label| text_system.layout(label, font_size, f32::INFINITY).width() + pad_inside)
        .collect()
}

/// **How the group wraps: how many buttons on each row.** Greedy flow — fill a row with as many as fit
/// at their natural widths, then start the next.
///
/// ⚠️ This replaced an **END-demotion** rule (fit a prefix in the top row, then give every leftover a
/// full-width row of its own). The two agree wherever there are 0 or 1 leftovers, which is every group
/// of two to four options in the app — so the difference was invisible until a list of **ten** arrived
/// (the Impasto TOOL list) and rendered as three across the top and seven stacked one per line, each
/// stretched edge to edge. Enio, 2026-07-19: *"deve organizar os botões como na primeira linha de
/// botões, quantos couberem por linha e não um por linha."*
///
/// ⚠️ **Both the painter and the measurer call this**, and that is structural rather than tidy: they used
/// to implement the wrap twice, and a container measured by one rule and filled by another is how the
/// next section quietly paints over these buttons and kills their hit targets
/// (`seam_impasto_rig.rs::no_impasto_widget_loses_its_hit_to_the_section_below`).
///
/// A label wider than the whole row still gets a row — never an empty one, or the walk would not
/// terminate.
pub(crate) fn segmented_row_counts(rect_w: f32, widths: &[f32]) -> Vec<usize> {
    let gap = segmented_gap();
    let mut rows = Vec::new();
    let mut i = 0;
    while i < widths.len() {
        let mut n = 0usize;
        let mut used = 0.0f32;
        while i + n < widths.len() {
            let extra = widths[i + n] + if n > 0 { gap } else { 0.0 };
            if n > 0 && used + extra > rect_w {
                break;
            }
            used += extra;
            n += 1;
        }
        let n = n.max(1);
        rows.push(n);
        i += n;
    }
    rows
}

/// ⭐⭐⭐ **As larguras DENTRO de uma fileira: cada peça leva o que a PALAVRA dela pede, e a folga
/// reparte-se por igual.**
///
/// ⛔⛔ **Nasceu de um vermelho medido em 2026-09-19**, quando a varredura de elisões passou a
/// pintar o Inspector com um documento na mão. O grupo tinha **duas leis para uma disposição**: a
/// que QUEBRA ([`segmented_row_counts`]) mede cada rótulo, e a que PINTAVA dividia a fileira em
/// partes **iguais** ⇒ *uma fileira podia caber inteira e ainda assim cortar a peça mais larga*.
/// Medido: o `Direction Override` do Inspector (`Inherit · Fwd · Rev · PP · PP Rev`) dava
/// `34,4 px` a cada uma das cinco e pintava `Inh…` e `PP…`, com a soma das larguras naturais a
/// caber na coluna com folga.
///
/// ⚠️ **É a MESMA família que o doc do [`segmented_row_counts`] já nomeia** (*«um contentor medido
/// por uma regra e preenchido por outra»*), aqui na dimensão horizontal em vez da vertical — e o
/// `block_cells_of`, a porta que recebe larguras dadas, existe desde 2026-09-07 e este pintor não
/// a chamava.
///
/// **A lei:**
/// - cabe ⇒ cada peça leva a sua largura natural **mais** `folga/n` (a fileira continua a encher a
///   coluna: uma direita esfarrapada leria-se como um grupo incompleto);
/// - não cabe ⇒ todas encolhem **na mesma proporção**, e aí a elisão é a resposta certa — falta
///   coluna, não disposição.
///
/// ⭐ Com rótulos de larguras iguais ela devolve o que a divisão igual devolvia, **ao pixel** — é
/// isso que mantém intacto todo grupo do app cujas palavras já eram do mesmo tamanho.
///
/// ⚠️ As larguras são **arredondadas para baixo** e a última peça come o resto, exactamente como o
/// [`segment_rects`](super::segment_rects) faz: sem isso `n` peças fraccionárias deixam uma costura
/// de sub-pixel entre duas quinas que deviam formar uma linha recta.
pub(crate) fn segmented_row_widths(rect_w: f32, naturals: &[f32]) -> Vec<f32> {
    let n = naturals.len();
    if n == 0 {
        return Vec::new();
    }
    let gaps = segmented_gap() * (n - 1) as f32;
    let disponivel = (rect_w - gaps).max(n as f32);
    let soma: f32 = naturals.iter().sum();
    let mut larguras: Vec<f32> = if soma <= disponivel {
        let folga = (disponivel - soma) / n as f32;
        naturals.iter().map(|w| (w + folga).floor()).collect()
    } else {
        let escala = disponivel / soma;
        naturals.iter().map(|w| (w * escala).floor()).collect()
    };
    // A última come o resto — a fileira acaba **exactamente** na borda da coluna.
    let usado: f32 = larguras.iter().take(n - 1).sum();
    larguras[n - 1] = (disponivel - usado).max(1.0);
    larguras
}

#[cfg(test)]
mod tests {

    /// ⭐⭐⭐ **UMA FILEIRA QUE CABE NUNCA CORTA A PEÇA MAIS LARGA.**
    ///
    /// ⛔⛔ **Este é o defeito que a varredura de elisões apanhou em 2026-09-19**, e ele não é um
    /// caso extremo: a fileira **cabia** e a divisão em partes IGUAIS cortava na mesma. *Uma média
    /// não é um máximo* — com larguras desiguais, `rect_w / n` pode ser menor do que UMA das peças
    /// precisa mesmo quando a SOMA delas cabe de sobra. Foi o que pôs `Inh…` e `PP…` no
    /// `Direction Override` do Inspector, com `34,4 px` para cada uma das cinco.
    ///
    /// ⛔ **E a metade que quase faltou é a DEGENERADA:** com rótulos de larguras iguais a lei tem
    /// de devolver o que a divisão igual devolvia, **ao pixel** — senão isto move todo grupo do app
    /// cujas palavras já eram do mesmo tamanho, e o diff passa a ser ilegível.
    ///
    /// (Mutação: devolver `vec![each; n]` ⇒ a 1.ª metade acusa a peça larga, RED.)
    #[test]
    fn uma_fileira_que_cabe_nunca_corta_a_peca_mais_larga() {
        let naturais = [64.0_f32, 44.0, 44.0, 38.0, 60.0];
        let coluna = 300.0_f32;
        let w = super::segmented_row_widths(coluna, &naturais);
        assert_eq!(w.len(), naturais.len());
        for (i, (&dado, &pede)) in w.iter().zip(naturais.iter()).enumerate() {
            assert!(
                dado >= pede,
                "a peça {i} pede {pede} px e recebeu {dado} — e a fileira CABIA (soma {} em {coluna})",
                naturais.iter().sum::<f32>()
            );
        }
        // A fileira enche a coluna: uma direita esfarrapada lê-se como um grupo incompleto.
        let gaps = super::segmented_gap() * (naturais.len() - 1) as f32;
        let soma: f32 = w.iter().sum();
        assert!(
            (soma + gaps - coluna).abs() < 1e-3,
            "a fileira ocupa {soma} + {gaps} de traço numa coluna de {coluna}"
        );
        // ⭐ A DEGENERADA: palavras do mesmo tamanho devolvem a divisão igual, ao pixel.
        let iguais = [50.0_f32; 4];
        let w = super::segmented_row_widths(200.0, &iguais);
        let esperado = ((200.0 - super::segmented_gap() * 3.0) / 4.0).floor();
        for (i, &dado) in w.iter().take(3).enumerate() {
            assert!(
                (dado - esperado).abs() < 1e-3,
                "peça {i} de quatro iguais: {dado} contra os {esperado} da divisão igual"
            );
        }
    }

    /// ⭐⭐ **E quando ela NÃO cabe, todas encolhem na mesma proporção.**
    ///
    /// ⚠️ Ali a elisão é a resposta certa — falta COLUNA, não disposição —, e o que esta régua
    /// afirma é que nenhuma peça é sacrificada para salvar as vizinhas.
    #[test]
    fn quando_nao_cabe_todas_encolhem_por_igual() {
        let naturais = [100.0_f32, 50.0, 50.0];
        let w = super::segmented_row_widths(100.0, &naturais);
        let razao: Vec<f32> = w.iter().zip(naturais).map(|(d, n)| d / n).collect();
        assert!(
            (razao[0] - razao[1]).abs() < 0.05 && (razao[1] - razao[2]).abs() < 0.05,
            "as razões divergem: {razao:?} — alguém está a pagar pelos outros"
        );
    }
}
