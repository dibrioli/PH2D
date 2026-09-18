//! ⭐⭐ **UMA FRASE COM PEÇAS VINDAS DO CÓDIGO** — a substituição de marcadores.
//!
//! ⚠️ **Irmão do `lib.rs` por ASSUNTO**, cortado dele em 2026-09-17 (tecto de 700 LOC): o pai
//! é o encaminhador mais a tabela dos `panel.*`, e ler um MODELO é outra lei.
//!
//! ⭐ **E ela mora ao lado da lei que a tem de respeitar:** o [`crate::pseudo::deforma`] copia
//! `{nome}` verbatim exactamente porque esta função o vai procurar a seguir, e as duas partilham
//! a tolerância ao `{` sem par. *Duas leituras do mesmo modelo escritas em ficheiros distantes
//! divergem no dia em que uma delas ganhar um caso.*

use crate::{Idioma, idioma, tr_em};

/// **Uma frase com PEÇAS vindas do código** — `tr_with("panel.hierarchy.count", &[("n", "12")])`
/// sobre `"{n} entities"` dá `"12 entities"`.
///
/// ⭐ **A frase inteira mora na tabela, com os marcadores nomeados**, e o código só entrega os
/// valores. É a forma que o Fluent tem (`{ $n } entities`) e a única que sobrevive a uma segunda
/// língua: colar `format!("{n} {}", tr("…entities"))` fixa a ORDEM das palavras no código, e há
/// línguas em que o número vem depois do nome.
///
/// ⚠️ Um marcador sem valor fica ESCRITO (`{n}`), de propósito — como a chave desconhecida do [`tr`],
/// o erro tem de se ver na tela.
///
/// ⛔ **Uma passagem só sobre o MODELO** (2026-09-16): substituir marcador a marcador deixava um
/// VALOR com cara de marcador ser reescrito pelo seguinte — um prefab chamado `{follows}`. Os valores
/// são texto do artista e saem verbatim (gate `a_value_never_becomes_a_marker`).
pub fn tr_with(key: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
    tr_with_em(idioma(), key, args)
}

/// O mesmo, com o idioma DADO — irmã do [`crate::tr_em`], e pela mesma razão: um gate mede o
/// idioma de teste sem escrever no ambiente de um processo que corre a suíte em paralelo.
///
/// ⭐ **É aqui que a lei do [`crate::pseudo`] se cobra:** deformado, o modelo tem de continuar a
/// deixar esta função achar `{nome}` e trocá-lo pelo valor — *e o valor sai verbatim, na língua
/// nenhuma, porque é texto do artista*.
#[must_use]
pub fn tr_with_em(idioma: Idioma, key: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
    let template = tr_em(idioma, key);
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let named = after.find('}').and_then(|close| {
            let name = &after[..close];
            args.iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| (close, *v))
        });
        match named {
            Some((close, value)) => {
                out.push_str(&value.to_string());
                rest = &after[close + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}
