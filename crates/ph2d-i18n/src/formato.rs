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
        let named = after
            .find('}')
            .and_then(|close| peca(&after[..close], args).map(|texto| (close, texto)));
        match named {
            Some((close, value)) => {
                out.push_str(&value);
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

/// O que um marcador vale: `{nome}` é o valor; `{nome?entity|entities}` é a forma do substantivo para
/// esse valor (a 1.ª só para `1`). A frase inteira fica na tabela: `"{n} {n?entity|entities}"`.
fn peca(marcador: &str, args: &[(&str, &dyn std::fmt::Display)]) -> Option<String> {
    let (nome, formas) = match marcador.split_once('?') {
        Some((nome, formas)) => (nome, formas.split_once('|')),
        None => (marcador, None),
    };
    let valor = args.iter().find(|(n, _)| *n == nome)?.1.to_string();
    Some(match formas {
        Some((um, outros)) => if valor == "1" { um } else { outros }.to_owned(),
        None if marcador.contains('?') => return None,
        None => valor,
    })
}

#[cfg(test)]
mod tests {
    use super::peca;

    /// *Mutação: `valor == "1"` trocado ⇒ «1 entities», a foto do dono na Hierarquia.*
    #[test]
    fn a_count_of_one_takes_the_singular_and_any_other_the_plural() {
        let forma = |n: u32| peca("n?entity|entities", &[("n", &n)]).unwrap();
        assert_eq!(forma(1), "entity");
        assert_eq!(forma(0), "entities");
        assert_eq!(forma(11), "entities");
        assert_eq!(peca("n", &[("n", &7)]).unwrap(), "7");
        assert_eq!(peca("x?a|b", &[("n", &1)]), None, "sem valor fica escrito");
        assert_eq!(
            peca("n?sem_barra", &[("n", &1)]),
            None,
            "mal escrito fica escrito"
        );
    }

    /// Nenhuma frase da tabela conta um substantivo sem dizer o singular, nem com o `(s)` de formulário.
    /// *Mutação: voltar uma frase a `{n} objects` ⇒ vermelho a dizer qual.*
    #[test]
    fn no_sentence_counts_a_noun_without_its_singular() {
        const NOMES: [&str; 20] = [
            "entities",
            "components",
            "objects",
            "frames",
            "sprites",
            "cards",
            "votes",
            "pieces",
            "layers",
            "clips",
            "markers",
            "items",
            "results",
            "errors",
            "tracks",
            "steps",
            "bodies",
            "tags",
            "points",
            "groups",
        ];
        // Só com dois ou mais, por construção: o caso de 1 tem outra frase ou outra chave.
        const SEMPRE_VARIOS: [&str; 7] = [
            "panel.vector.bar.name.many",
            // `{n}` é o tecto de peças do traço (`MAX_PIECES`, centenas).
            "stay under {n} pieces",
            "keeps the value of {count} objects",
            "{count} objects are named",
            "{n} objects selected",
            "Merged {n_sources} sprites",
            // `joint_editor.drop.many` só com `n > 1`; o de 1 é `joint_editor.drop.one`.
            "these {n} objects become",
        ];
        // `\u{2014} cards` é um travessão seguido de um substantivo: não conta nada.
        let conta_em = |l: &str, nome: &str| {
            l.match_indices(&format!("}} {nome}")).any(|(i, _)| {
                l[..i]
                    .rfind('{')
                    .is_some_and(|abre| !l[..abre].ends_with("\\u"))
            })
        };
        let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let mut pilha = vec![std::path::PathBuf::from(raiz)];
        let mut culpados = Vec::new();
        while let Some(d) = pilha.pop() {
            for e in std::fs::read_dir(d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    pilha.push(p);
                    continue;
                }
                if p.file_name()
                    .is_some_and(|n| n == "formato.rs" || n == "pseudo.rs")
                {
                    continue;
                }
                let texto = std::fs::read_to_string(&p).unwrap();
                for l in texto.lines().filter(|l| !l.trim_start().starts_with("//")) {
                    let conta = NOMES.iter().any(|n| conta_em(l, n));
                    let formulario = l.contains("(s)") && l.contains('{');
                    if (conta || formulario) && !SEMPRE_VARIOS.iter().any(|k| l.contains(k)) {
                        culpados.push(l.trim().to_owned());
                    }
                }
            }
        }
        assert!(culpados.is_empty(), "{culpados:#?}");
    }
}
