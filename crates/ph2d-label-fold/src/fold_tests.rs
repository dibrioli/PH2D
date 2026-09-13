//! Os gates da dobra — `docs/Components/08_plano_tags.md` §5.1 (a tabela) e §5.2 (gates 1 e 2).
//!
//! ⚠️ **As DUAS colunas são obrigatórias.** Uma dobra que colapsasse tudo em `""` passaria na
//! primeira; é a segunda — *o que NÃO pode colapsar* — que a apanha.

use super::fold;

/// ⭐⭐ **Os que TÊM de colapsar, e os que NÃO podem.**
///
/// **Mutações que devem sangrar:** tirar o *case folding* (`Inimigo` ≠ `inimigo`) · tirar a remoção
/// das marcas (`inímigo` ≠ `inimigo`) · trocar o *case folding* completo por `to_lowercase`
/// (`Straße` ≠ `STRASSE`) · tirar a normalização (`é` pré-composto ≠ `e` + U+0301) · tirar o colapso
/// de espaços · dobrar tudo para `""` (a coluna da direita).
#[test]
fn the_fold_collapses_case_and_accents_and_nothing_else() {
    let colapsam: &[&[&str]] = &[
        &["Inimigo", "inimigo", "INIMIGO", "inímigo", "ÍNIMIGO"],
        // Pré-composto (U+00E9) e decomposto (e + U+0301 COMBINING ACUTE ACCENT).
        &["caf\u{e9}", "cafe\u{301}", "CAFÉ", "cafe"],
        // ⚠️ É ESTA a linha que o `to_lowercase` reprova: `ß` só vira `ss` pelo *case folding*.
        &["Straße", "STRASSE", "strasse"],
        &["Inimigo  Voador", "inimigo voador", " Inimigo Voador "],
        &["Ação", "ACAO", "acao"],
    ];
    for grupo in colapsam {
        let chave = fold(grupo[0]);
        for &outro in grupo.iter().skip(1) {
            assert_eq!(
                fold(outro),
                chave,
                "{outro:?} tinha de dobrar igual a {:?}",
                grupo[0]
            );
        }
    }

    let distintos: &[(&str, &str)] = &[
        ("inimigo", "inimiga"),
        ("Enemy", "Enemies"),
        ("a b", "ab"),
        ("Boss", "Bass"),
    ];
    for (a, b) in distintos {
        assert_ne!(
            fold(a),
            fold(b),
            "{a:?} e {b:?} NAO podem virar a mesma tag"
        );
    }
    assert!(!fold("Inimigo").is_empty(), "a chave nao pode ser vazia");
}

/// ⭐ **Dobrar duas vezes é dobrar uma**, e a forma de normalização da ENTRADA não importa.
///
/// ⚠️ Sem isto uma chave guardada e uma recalculada divergiam — e a árvore compara as duas.
#[test]
fn the_fold_is_idempotent_and_normalisation_blind() {
    for s in [
        "Inimigo Voador",
        "Straße",
        "cafe\u{301}",
        "ÇÃO",
        "Ǆemal",
        "İstanbul",
    ] {
        let uma = fold(s);
        assert_eq!(fold(&uma), uma, "dobrar {s:?} duas vezes mudou a chave");
    }
    assert_eq!(
        fold("\u{1e09}"),
        fold("c\u{327}\u{301}"),
        "NFC e NFD dao a mesma chave"
    );
}
