//! **O ROTEIRO de uma cena de smoke, com forma CONSTANTE.**
//!
//! # Por que isto é uma porta e não um `eprintln!` por cena
//!
//! Cada cena escrevia o próprio muro de `eprintln!`, e o resultado divergiu em três eixos ao
//! mesmo tempo: umas em português com acento, outras num ASCII mutilado (`esta'`, `a'`, `voce`);
//! umas com `**markdown**` que o terminal não renderiza e imprime como asteriscos; e **todas sem
//! linha em branco entre os passos**, o que transforma nove passos num bloco só.
//!
//! ⚠️ **E o defeito que decide a legibilidade foi MEDIDO, não achado:** as linhas da cena dos
//! tokens tinham **81-82 caracteres** de conteúdo mais a indentação, então **quebravam sozinhas
//! num terminal de 80 colunas** — e uma linha que quebra perde a indentação, o que faz a
//! continuação parecer um passo novo. O muro não era só prosa densa: era *wrap*.
//!
//! Aqui a forma é decidida **uma vez**: o passo tem um VERBO (o que se vai fazer) e um corpo (o
//! que tem de acontecer), há uma linha em branco entre passos, e a largura cabe em 80 colunas.
//!
//! # A largura NÃO é gosto — ela nomeia o recurso
//!
//! `MAX_COLS` é 80 porque 80 é a largura default de um terminal POSIX. O corpo é indentado em
//! [`BODY_INDENT`], então o texto útil tem `MAX_BODY` colunas. O gate afirma isso sobre as
//! cenas que passam por esta porta — um roteiro que não cabe é um roteiro que se lê torto.

/// A indentação do corpo de um passo (`  1. ` tem 5 colunas). É o único número que o IMPRESSOR
/// precisa saber — o teto de largura é assunto do gate, e mora com ele.
pub(crate) const BODY_INDENT: usize = 5;

/// A largura de um terminal POSIX default. Não é preferência: é o recurso.
///
/// ⚠️ `cfg(test)` porque **quem a consome é o gate**, não o impressor: o `script` obedece a
/// largura escrevendo linhas que cabem, e é o gate que verifica. Deixá-la no build de produto
/// seria uma constante sem leitor — a forma exata de const que apodrece.
#[cfg(test)]
pub(crate) const MAX_COLS: usize = 80;
/// Quantas colunas sobram para o texto de uma linha de corpo.
#[cfg(test)]
pub(crate) const MAX_BODY: usize = MAX_COLS - BODY_INDENT;

/// Um passo do roteiro: **o que fazer** (o verbo) e **o que tem de acontecer** (o corpo).
///
/// ⚠️ O verbo é obrigatório e curto de propósito. Um passo cujo título é uma frase inteira volta
/// a ser o muro que esta porta existe para desfazer — o artista varre os títulos para se
/// localizar e só lê o corpo do passo em que está.
pub(crate) struct Step {
    /// O título do passo, em caixa alta (`CRIAR`, `O ELO`, `O CONTROLE`).
    pub verb: &'static str,
    /// O corpo, uma linha por elemento. Cada uma cabe na largura que o gate afirma.
    pub lines: &'static [&'static str],
}

/// Imprime o roteiro com a forma que esta porta decide.
///
/// `tag` é o prefixo da cena (o mesmo do resto do log dela) e `intro` é a frase que diz o que o
/// artista precisa ter em mãos antes do passo 1 — a premissa, não o primeiro passo.
pub(crate) fn script(tag: &str, intro: &str, steps: &[Step]) {
    eprintln!();
    eprintln!("[{tag}] ROTEIRO ({} passos) — {intro}", steps.len());
    for (i, s) in steps.iter().enumerate() {
        eprintln!();
        eprintln!("  {}. {}", i + 1, s.verb);
        for l in s.lines {
            eprintln!("{:BODY_INDENT$}{l}", "");
        }
    }
    eprintln!();
}

/// **Toda linha cabe no terminal** — o defeito medido que originou esta porta.
///
/// ⚠️ A contagem é de **caracteres**, não de bytes: `está` tem 4 colunas e 5 bytes, e medir bytes
/// reprovaria texto correto em português exatamente onde ele é mais legível.
#[cfg(test)]
pub(crate) fn assert_fits(tag: &str, steps: &[Step]) {
    for (i, s) in steps.iter().enumerate() {
        assert!(
            s.verb.chars().count() <= MAX_BODY,
            "[{tag}] passo {}: o verbo tem {} colunas",
            i + 1,
            s.verb.chars().count()
        );
        assert!(!s.lines.is_empty(), "[{tag}] passo {}: corpo vazio", i + 1);
        for l in s.lines {
            let cols = l.chars().count();
            assert!(
                cols <= MAX_BODY,
                "[{tag}] passo {}: a linha tem {cols} colunas (o teto e' {MAX_BODY}); ela quebra \
                 num terminal de {MAX_COLS} e a continuacao perde a indentacao:\n{l}",
                i + 1
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O gate de largura de facto morde** (controle positivo).
    #[test]
    fn the_width_gate_catches_a_line_that_would_wrap() {
        const LONG: &str = "esta linha tem mais de setenta e cinco colunas de proposito, para provar que o gate \
             de largura de facto morde quando alguem escreve um paragrafo numa linha so";
        assert!(
            LONG.chars().count() > MAX_BODY,
            "a fixture nao contem o fenomeno"
        );
        let steps = [Step {
            verb: "CONTROLE",
            lines: &[LONG],
        }];
        let caught = std::panic::catch_unwind(|| assert_fits("t", &steps)).is_err();
        assert!(caught, "o gate de largura nao mordeu uma linha que quebra");
    }

    /// **Uma linha acentuada e curta PASSA** — a metade que prova que ele conta COLUNAS.
    ///
    /// ⚠️ Sem ela, um gate que medisse BYTES ficaria verde e reprovaria português correto.
    #[test]
    fn an_accented_line_is_measured_in_columns_not_bytes() {
        const ACCENTED: &str = "a régua está à direita, e a métrica não é a memória: colunas.";
        assert!(
            ACCENTED.len() > ACCENTED.chars().count(),
            "sem acento a fixture nao prova nada"
        );
        assert_fits(
            "t",
            &[Step {
                verb: "ACENTO",
                lines: &[ACCENTED],
            }],
        );
    }
}
