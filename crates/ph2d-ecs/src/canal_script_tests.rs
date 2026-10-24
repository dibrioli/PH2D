//! A chave da família SCRIPT: o nome vai e volta pela tabela, nomes diferentes nunca partilham
//! canal, e o tipo diz as faixas.

use super::*;
use std::collections::BTreeSet;

/// ⛔ **Dois nomes nunca são o mesmo canal** — a tabela dá um índice por texto. (Mutação: `i:j`
/// fixos ⇒ colidem.)
#[test]
fn cada_nome_e_o_seu_canal_e_o_nome_le_se_da_chave() {
    let mut textos = TextosDosCanais::default();
    let mut vistas = BTreeSet::new();
    for nome in ["speed", "Speed", "jump", "cor", "posição", ""] {
        let k = chave(nome, Tipo::Number, 0, &mut textos).expect("faixa 0");
        assert!(vistas.insert(k), "'{nome}' colide");
        assert_eq!(nome_k(k, &textos), Some(nome));
    }
    assert_eq!(
        chave("speed", Tipo::Number, 0, &mut textos),
        chave("speed", Tipo::Number, 0, &mut textos),
        "o mesmo nome dá a mesma chave"
    );
}

fn nome_k(k: FieldKey, textos: &TextosDosCanais) -> Option<&str> {
    nome(k, textos)
}

#[test]
fn o_tipo_diz_as_faixas_e_o_info() {
    let mut textos = TextosDosCanais::default();
    let cor = chaves("tint", Tipo::Color, &mut textos);
    assert_eq!(cor.len(), 4);
    assert!(
        cor.iter().all(|&k| k.mesmo_grupo(cor[0])),
        "a cor é um grupo"
    );
    assert_eq!(chave("tint", Tipo::Color, 4, &mut textos), None);
    let txt = chave("modo", Tipo::Text, 0, &mut textos).expect("faixa 0");
    assert!(info(txt).is_some_and(|i| i.texto && i.discreto));
    let b = chave("on", Tipo::Bool, 0, &mut textos).expect("faixa 0");
    assert!(info(b).is_some_and(|i| i.discreto && !i.texto));
    assert!(!b.mesmo_grupo(txt));
    let outra = com_nome(b, valor_do_nome(txt));
    assert_eq!(
        nome(outra, &textos),
        Some("modo"),
        "o remapeamento troca só o nome"
    );
    assert_eq!(tipo(outra), Some((Tipo::Bool, 0)));
}
