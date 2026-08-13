//! **A CHAVE: o índice ou o `id`** (doc 89 folha 15, o P0).
//!
//! ⚠️ **O defeito era silencioso, e é isso que o torna P0.** Com a chave no
//! índice, um `motion.sort` ou um `motion.cull` a montante reordena o conjunto e
//! **todo valor aleatório troca de dono** — nada na tela diz que a variação mudou
//! de elemento, e o artista lê a cena inteira a re-embaralhar como se fosse o
//! ruído a mudar.

use super::*;

/// Um `id` por elemento, na ordem dada.
fn ids(v: &[f32]) -> Vec<f32> {
    v.to_vec()
}

/// **Com a chave no ÍNDICE o nó é o que sempre foi, AO BIT** — inclusive com uma
/// coluna `id` presente, que ele tem de IGNORAR.
#[test]
fn keying_by_index_is_the_node_that_always_shipped_to_the_bit() {
    let id = ids(&[7.0, 3.0, 11.0, 0.0, 5.0]);
    for mode in [FieldMode::Index, FieldMode::Ramp, FieldMode::Random] {
        let bare = field(5, mode, 9, KeyBy::Index, None);
        let with_ids = field(5, mode, 9, KeyBy::Index, Some(&id));
        for (a, b) in bare.iter().zip(&with_ids) {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "a chave no indice ignora a coluna `id` ({mode:?})"
            );
        }
    }
}

/// **REORDENAR O CONJUNTO NÃO RE-SORTEIA OS VALORES** — a propriedade inteira.
///
/// O mesmo conjunto, embaralhado: com a chave no `id` cada elemento leva o seu
/// valor consigo; com a chave no índice ele fica onde estava e passa a pertencer
/// a outro. ⚠️ O segundo é o **CONTROLE**, e sem ele o gate ficaria verde sobre um
/// nó que devolvesse uma constante.
#[test]
fn sorting_the_set_does_not_re_deal_the_values_when_keyed_by_id() {
    let before = ids(&[10.0, 20.0, 30.0, 40.0]);
    let after = ids(&[30.0, 10.0, 40.0, 20.0]); // o mesmo conjunto, noutra ordem
    let a = field(4, FieldMode::Random, 5, KeyBy::Id, Some(&before));
    let b = field(4, FieldMode::Random, 5, KeyBy::Id, Some(&after));
    for (k, id) in after.iter().enumerate() {
        let was = before.iter().position(|q| q == id).expect("mesmo conjunto");
        assert_eq!(
            b[k].to_bits(),
            a[was].to_bits(),
            "o elemento de id {id} tem de levar o valor DELE para a posicao {k}"
        );
    }
    // O CONTROLE: pela posição, os valores ficam parados e trocam de dono.
    let c = field(4, FieldMode::Random, 5, KeyBy::Index, Some(&after));
    assert_eq!(
        c,
        field(4, FieldMode::Random, 5, KeyBy::Index, Some(&before)),
        "pelo indice o valor e da POSICAO -- e por isso a cena re-embaralha"
    );
    assert!(
        c.iter().zip(&b).any(|(x, y)| (x - y).abs() > 1e-6),
        "e as duas chaves TEM de diferir, senao este gate nao mede nada"
    );
}

/// **Sem coluna `id`, a chave cai no ÍNDICE — não em zero.**
///
/// ⚠️ É a armadilha do caminho de GPU escrita como gate: a `identity` de um
/// binding ausente é zero, e zero como chave daria a TODO elemento o mesmo valor
/// aleatório — um campo de variação que não varia. Uma grade não carrega `id`, e
/// é o caso mais comum que existe.
#[test]
fn with_no_id_column_the_key_falls_back_to_the_index_not_to_zero() {
    let v = field(6, FieldMode::Random, 3, KeyBy::Id, None);
    assert_eq!(
        v,
        field(6, FieldMode::Random, 3, KeyBy::Index, None),
        "sem `id` as duas chaves coincidem"
    );
    let all_same = v.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-9);
    assert!(!all_same, "e o campo VARIA: {v:?}");
}

/// **O modo Index passa a emitir a IDENTIDADE quando a chave é o `id`** — que é a
/// distinção que o Blender faz entre os nós *Index* e *ID*.
#[test]
fn the_index_mode_emits_the_identity_when_keyed_by_id() {
    let id = ids(&[7.0, 3.0, 11.0]);
    assert_eq!(field(3, FieldMode::Index, 0, KeyBy::Id, Some(&id)), id);
    assert_eq!(
        field(3, FieldMode::Index, 0, KeyBy::Index, Some(&id)),
        vec![0.0, 1.0, 2.0],
        "e pelo indice continua a ser a POSICAO"
    );
}

/// **O `Ramp` ignora a chave, e é decisão** — uma rampa é *onde você está na
/// lista*. Este gate existe para que trocá-la custe reconferir o motivo (e o
/// `ParamGate` esconde o seletor nesse modo, então não há knob morto).
#[test]
fn the_ramp_is_positional_by_definition_and_ignores_the_key() {
    let id = ids(&[100.0, 3.0, 42.0, 7.0]);
    assert_eq!(
        field(4, FieldMode::Ramp, 0, KeyBy::Id, Some(&id)),
        field(4, FieldMode::Ramp, 0, KeyBy::Index, None),
        "`id / (n-1)` sobre ids esparsos nem fica em [0,1] nem e uma rampa"
    );
}
