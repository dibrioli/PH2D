//! As queries: compilam, a gramática cabe no ABI, e nenhuma captura ou predicado passa CALADO.

use ph2d_code_syntax::{
    CaptureUse, HIGHLIGHTS_SCM, capture_table, highlight_captures, unevaluated_predicates,
};
use std::collections::BTreeSet;

#[test]
fn abi_da_gramatica() {
    let lang: tree_sitter::Language = tree_sitter_luau::LANGUAGE.into();
    let v = lang.abi_version();
    assert!(
        (tree_sitter::MIN_COMPATIBLE_LANGUAGE_VERSION..=tree_sitter::LANGUAGE_VERSION).contains(&v),
        "ABI {v}"
    );
}

/// Controlo de cegueira: toda captura da query tem destino (pinta ou é ignorada de propósito), e
/// toda linha da tabela existe na query (uma linha morta esconderia uma captura renomeada).
#[test]
fn toda_captura_tem_destino_e_toda_linha_da_tabela_e_usada() {
    let in_query: BTreeSet<String> = highlight_captures().into_iter().collect();
    let in_table: BTreeSet<String> = capture_table()
        .iter()
        .map(|(n, _)| (*n).to_string())
        .collect();
    let sem_destino: Vec<_> = in_query.difference(&in_table).collect();
    let mortas: Vec<_> = in_table.difference(&in_query).collect();
    assert!(
        sem_destino.is_empty(),
        "capturas sem HighlightKind nem ignorada: {sem_destino:?}"
    );
    assert!(
        mortas.is_empty(),
        "linhas da tabela que a query não tem: {mortas:?}"
    );
    assert_eq!(
        in_table.len(),
        capture_table().len(),
        "captura repetida na tabela"
    );
    let ignoradas: Vec<_> = capture_table()
        .iter()
        .filter(|(_, u)| *u == CaptureUse::Ignore)
        .map(|(n, _)| *n)
        .collect();
    assert_eq!(ignoradas, ["spell", "_method"]);
}

/// `#lua-match?` e afins não são avaliados pelo tree-sitter puro: o padrão casaria SEM filtro.
#[test]
fn nenhum_predicado_fica_por_avaliar() {
    assert_eq!(unevaluated_predicates(), Vec::<String>::new());
}

/// A cópia é a do pacote, byte a byte, a menos das três trocas documentadas no cabeçalho.
#[test]
fn query_e_a_do_pacote_mais_a_traducao() {
    let marca = "; --- fim do cabeçalho ---\n";
    let at = HIGHLIGHTS_SCM
        .find(marca)
        .expect("cabeçalho sem a marca de fim");
    let corpo = &HIGHLIGHTS_SCM[at + marca.len()..];
    assert_eq!(corpo.matches("#match?").count(), 4);
    assert_eq!(corpo.matches("(type/identifier) @type\n").count(), 1);
    let desfeito = corpo
        .replace("#match?", "#lua-match?")
        .replace("(\\\\s?)", "(%s?)")
        .replace(
            "(type/identifier) @type\n",
            "(type\n  (identifier) @type)\n",
        );
    assert_eq!(desfeito, tree_sitter_luau::HIGHLIGHTS_QUERY);
}
