//! Os gates da [`super::Lens`] — a ida-e-volta e a alternância.

use super::Lens;

/// ⭐ **O índice e a variante são a MESMA coisa nos dois sentidos.**
///
/// ⚠️ **A metade da VOLTA é a que impede um chip morto:** um selector segmentado guarda a posição,
/// e uma lista que produzisse duas variantes com o mesmo índice deixaria uma delas inalcançável
/// pelo dedo — com lei, com gates, e sem ninguém lhe chegar.
#[test]
fn o_indice_e_a_variante_vao_e_voltam() {
    let mut vistos = std::collections::BTreeSet::new();
    for l in Lens::ALL {
        assert_eq!(Lens::from_index(l.index()), l, "a volta de {l:?}");
        assert!(vistos.insert(l.index()), "{l:?} repete um indice");
    }
    assert_eq!(vistos.len(), Lens::ALL.len());
}

/// ⚠️ **Fora da faixa devolve o valor de FÁBRICA e nunca entra em pânico** — um ficheiro gravado
/// por uma versão com três lentes é lido por uma com duas, e a resposta honesta é a de fábrica.
#[test]
fn um_indice_de_fora_cai_no_valor_de_fabrica() {
    assert_eq!(Lens::from_index(99), Lens::default());
    assert_eq!(Lens::default(), Lens::Perspective);
}

/// ⭐⭐ **A tecla alterna e VOLTA** — e com duas lentes `other` é uma involução.
///
/// ⚠️ **A segunda metade é a que a nota da porta promete:** ela é derivada de `ALL`, logo com uma
/// terceira lente ela deixa de ser involução — e este gate passa a reprovar, com a premissa a
/// morrer à vista no diff. *Uma alternância escrita como `match` de dois braços continuaria a
/// compilar e saltaria a terceira em silêncio.*
#[test]
fn a_outra_lente_percorre_a_lista_inteira() {
    assert_eq!(Lens::Perspective.other(), Lens::Ortho);
    assert_eq!(Lens::Ortho.other(), Lens::Perspective);
    // Partindo de qualquer uma, `ALL.len()` passos fecham o ciclo — a propriedade que sobrevive a
    // uma variante nova.
    for l in Lens::ALL {
        let mut cursor = l;
        for _ in 0..Lens::ALL.len() {
            cursor = cursor.other();
        }
        assert_eq!(cursor, l, "o ciclo nao fecha a partir de {l:?}");
    }
}

/// ⚠️ **Cada lente tem a própria chave** — duas variantes a partilhar um rótulo são duas fileiras
/// indistinguíveis na tela.
#[test]
fn cada_lente_tem_a_propria_chave() {
    let chaves: std::collections::BTreeSet<_> = Lens::ALL.iter().map(|l| l.label_key()).collect();
    assert_eq!(chaves.len(), Lens::ALL.len());
    for l in Lens::ALL {
        assert!(
            l.label_key().starts_with("panel.sculpt3d.lens."),
            "{l:?} nao vem da tabela do painel"
        );
    }
}
