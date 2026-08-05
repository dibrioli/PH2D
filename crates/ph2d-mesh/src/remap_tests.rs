//! Os gates da renumeração — o plano PURO, antes de qualquer estrutura ser
//! tocada. É aqui que a propriedade *"aplicar a sequência compacta a lista"* é
//! afirmada contra um oráculo que não conhece o algoritmo.

use super::*;

/// Aplica a sequência a uma lista, como todo consumidor tem de fazer.
fn apply<T: Copy>(items: &mut Vec<T>, moves: &[(u32, u32)], len: usize) {
    for &(from, to) in moves {
        items[to as usize] = items[from as usize];
    }
    items.truncate(len);
}

/// **REMOVER O ÚLTIMO NÃO MOVE NINGUÉM** — e é por isso que
/// [`Remap::moves_nothing`] não quer dizer *"nada morreu"*.
///
/// O gate existe porque a confusão entre as duas frases é a que faz um consumidor
/// pular o `truncate` e continuar lendo um item que já não é de ninguém.
#[test]
fn removing_the_last_item_moves_nobody() {
    let r = Remap::plan(&[3], 4, &[], 0);
    assert!(r.face_moves.is_empty(), "ninguém tinha para onde ir");
    assert_eq!(r.faces, 3, "mas a lista encolheu");
    assert!(r.moves_nothing());
}

/// **UM SOBREVIVENTE PODE MUDAR DE CASA DUAS VEZES**, e é isso que torna a
/// renumeração uma SEQUÊNCIA em vez de uma tabela.
///
/// Com os mortos `[3, 8]` numa lista de 10: o 8 vaga primeiro e recebe o 9;
/// depois o 3 vaga e recebe o que agora está em 8 — que é o 9 original. Uma
/// tabela `de → para` guardaria `9 → 8` e `8 → 3` como fatos independentes, e
/// quem os lesse em qualquer ordem acertaria um e erraria o outro.
#[test]
fn a_survivor_can_change_house_twice() {
    let r = Remap::plan(&[3, 8], 10, &[], 0);
    assert_eq!(r.face_moves, vec![(9, 8), (8, 3)]);
    assert_eq!(r.faces, 8);
}

/// **APLICAR A SEQUÊNCIA COMPACTA A LISTA** — o oráculo é o `retain`, que não
/// sabe nada sobre trocas.
///
/// ⚠️ **O oráculo compara CONJUNTOS e não a ordem**, e a escolha é honesta: a
/// ordem dentro do vetor de faces é arbitrária (quem dá sentido a um índice é
/// quem o cita), então pinar a permutação seria pinar o algoritmo em vez da
/// propriedade. O que tem de valer é que ninguém vivo some e nenhum morto fica.
#[test]
fn applying_the_sequence_keeps_exactly_the_survivors() {
    for dead in [
        vec![],
        vec![0u32],
        vec![9],
        vec![3, 8],
        vec![0, 1, 2],
        vec![0, 4, 5, 9],
        vec![1, 3, 5, 7, 9],
    ] {
        let mut items: Vec<u32> = (0..10).collect();
        let r = Remap::plan(&dead, 10, &[], 0);
        apply(&mut items, &r.face_moves, r.faces);
        let mut got = items.clone();
        got.sort_unstable();
        let want: Vec<u32> = (0..10).filter(|v| !dead.contains(v)).collect();
        assert_eq!(got, want, "os mortos {dead:?} deixaram a lista errada");
        assert_eq!(items.len(), 10 - dead.len());
    }
}

/// **TUDO MORRE** — o caso degenerado, que existe porque um `end -= 1` a mais
/// nele estoura em vez de devolver uma lista vazia.
#[test]
fn everything_can_die() {
    let dead: Vec<u32> = (0..5).collect();
    let r = Remap::plan(&dead, 5, &dead, 5);
    let mut items: Vec<u32> = (0..5).collect();
    apply(&mut items, &r.face_moves, r.faces);
    assert!(items.is_empty());
    assert_eq!(r.verts, 0);
}

/// **ENCADEAR DUAS RODADAS É A MESMA COISA QUE UMA SÓ**, e é isso que autoriza
/// um dab a fazer três passes e entregar UM canal ao traço.
#[test]
fn chaining_two_rounds_compacts_like_one_sequence() {
    let mut items: Vec<u32> = (0..12).collect();
    let mut oracle = items.clone();

    let first = Remap::plan(&[2, 7], 12, &[], 0);
    apply(&mut oracle, &first.face_moves, first.faces);
    let second = Remap::plan(&[0, 5], 10, &[], 0);
    apply(&mut oracle, &second.face_moves, second.faces);

    let mut chained = first;
    chained.then(second);
    apply(&mut items, &chained.face_moves, chained.faces);
    assert_eq!(items, oracle, "a corrente descreveu outra compactação");
}
