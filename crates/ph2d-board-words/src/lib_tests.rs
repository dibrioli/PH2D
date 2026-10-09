//! As leis das palavras (`docs/MiroClone/05_facilitacao.md` §2 C7 e os três achados do oráculo §1.3).

use super::*;

fn keys(text: &str, lang: Lang) -> Vec<String> {
    keywords(text, lang).into_iter().map(|k| k.key).collect()
}

/// ⭐ Achado 1: o apóstrofo curvo dá a MESMA raiz que o recto (`team’s` = `team's` = `team`).
#[test]
fn a_curly_apostrophe_is_a_straight_one() {
    assert_eq!(keys("The team’s morale", Lang::English), ["team", "moral"]);
    assert_eq!(keys("The team's morale", Lang::English), ["team", "moral"]);
    assert_eq!(keys("Team", Lang::English), ["team"]);
}

/// ⭐ Achado 2: *case folding* completo (`ß`, a ligadura) e a chave sem acentos (`Café` = `CAFE`).
#[test]
fn case_and_accents_never_split_a_word() {
    assert_eq!(keys("STRASSE Straße", Lang::English), ["strass", "strass"]);
    assert_eq!(keys("ﬁle FILE", Lang::English), ["file", "file"]);
    assert_eq!(keys("Café CAFE", Lang::English), ["cafe", "cafe"]);
}

/// ⭐ Achado 3: as palavras vazias são as do Snowball — ⛔ a ISO apagava `need`, `work`, `test`, `new`.
#[test]
fn only_function_words_are_dropped() {
    assert_eq!(
        keys("We need new tests and the work is high", Lang::English),
        ["need", "new", "test", "work", "high"]
    );
    assert_eq!(keys("O tempo de trabalho é novo", Lang::Portuguese), ["temp", "trabalh", "nov"]);
}

/// Uma letra não é um assunto (o Snowball português NÃO lista o `é`, que ia agrupar tudo).
#[test]
fn a_single_letter_never_counts() {
    assert!(keys("é a e o x", Lang::Portuguese).is_empty());
    assert!(keys("v2.0", Lang::English).is_empty(), "`v2.0` tem UMA letra");
    assert_eq!(keys("b2b", Lang::English), ["b2b"], "duas letras ou mais contam, com algarismos");
}

/// ⭐ Os acentos ficam até à raiz: `informação` e `informações` são uma; sem acentos não seriam.
#[test]
fn accents_reach_the_stemmer() {
    let a = keys("Informação", Lang::Portuguese);
    assert_eq!(a, keys("informações", Lang::Portuguese));
    assert_ne!(
        stem(Lang::Portuguese, "informacao"),
        stem(Lang::Portuguese, "informacoes"),
        "a prova de que tirar os acentos ANTES partia a palavra"
    );
}

/// A forma escrita guarda o *case folding* com acentos (é o que o título mostra).
#[test]
fn the_surface_is_the_folded_word_with_accents() {
    let k = keywords("Reunião", Lang::Portuguese);
    assert_eq!(k[0].surface, "reunião");
    assert_eq!(k[0].key, "reunia", "a raiz da libstemmer é `reuniã`, sem o acento");
}

/// ⭐ A língua é a que tem mais palavras vazias; empate = inglês.
#[test]
fn the_language_is_the_one_with_more_function_words() {
    assert_eq!(detect(&["A equipa trabalha bem em pares", "Mais informações nas reuniões"]), Lang::Portuguese);
    assert_eq!(detect(&["The team is slow", "Builds are slow"]), Lang::English);
    assert_eq!(detect(&["Kanban", "Retro"]), Lang::English, "sem nenhuma, a da interface");
}
