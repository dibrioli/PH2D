//! Gates do **GRUPO** (doc 89 folha 08, linha 34: *"ordena DENTRO de grupos"*).
//!
//! A lei tem três metades, e cada uma tem aqui o seu gate: o grupo é a chave PRIMÁRIA, o
//! `descending` e o `shift` valem **dentro** de cada grupo, e a porta desligada é a
//! permutação de sempre — ao bit, não «parecida».

use super::{KEY_X, keys, permutation};

const O: [f32; 2] = [0.0, 0.0];

/// Quatro peças em `x = 0..3`, para a chave `X` ser a própria posição.
fn row(n: usize) -> Vec<[f32; 2]> {
    (0..n).map(|i| [i as f32, 0.0]).collect()
}

fn perm_x(p: &[[f32; 2]], groups: &[f32], descending: bool, shift: i64) -> Vec<usize> {
    permutation(&keys(p, KEY_X, O, 0, 0.0, &[]), groups, descending, shift)
}

/// ⭐ **O CONTROLE.** A porta desligada tem de dar exactamente a permutação que este nó
/// sempre deu — e o gate compara com a coluna CONSTANTE, que é a outra leitura de *"um
/// grupo só"*: se as duas discordassem, o default estaria a escolher entre elas em silêncio.
#[test]
fn an_unconnected_group_port_is_the_permutation_that_always_shipped() {
    let p = row(7);
    for descending in [false, true] {
        for shift in [-9i64, -1, 0, 1, 3, 7, 12] {
            let solto = perm_x(&p, &[], descending, shift);
            assert_eq!(
                solto,
                perm_x(&p, &[0.0; 7], descending, shift),
                "coluna de zeros = porta desligada (desc {descending}, shift {shift})"
            );
            assert_eq!(
                solto,
                perm_x(&p, &[4.25; 7], descending, shift),
                "e um grupo CONSTANTE qualquer da' o mesmo — o valor nao e' a ordem"
            );
            assert_eq!(
                solto,
                perm_x(&p, &[9.0], descending, shift),
                "e a coluna de UM valor (a difusao) tambem"
            );
        }
    }
}

/// O grupo lidera e a chave ordena dentro dele: com grupos `1 0 1 0` e a chave a crescer
/// com a posição, saem primeiro os do grupo `0`, cada bloco ordenado por si.
#[test]
fn the_group_leads_and_the_key_sorts_inside_it() {
    let p = row(4);
    let g = [1.0, 0.0, 1.0, 0.0];
    assert_eq!(
        perm_x(&p, &g, false, 0),
        vec![1, 3, 0, 2],
        "grupo 0 (pecas 1 e 3) antes do grupo 1 (pecas 0 e 2), cada um por `x`"
    );
}

/// ⚠️ **O `descending` inverte DENTRO do grupo e nunca atravessa a fronteira** — inverter a
/// lista toda moveria peças entre grupos, que é a negação da lei.
#[test]
fn descending_reverses_inside_the_group_never_across_it() {
    let p = row(6);
    let g = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let out = perm_x(&p, &g, true, 0);
    assert_eq!(out, vec![2, 1, 0, 5, 4, 3]);
    // A metade que o `assert_eq` acima não diz por si: cada bloco continua a ser o mesmo
    // conjunto de peças. É esta afirmação que morre se alguém inverter a lista inteira.
    let (a, b) = out.split_at(3);
    assert!(a.iter().all(|&i| g[i] == 0.0), "o 1.o bloco e' o grupo 0");
    assert!(b.iter().all(|&i| g[i] == 1.0), "o 2.o bloco e' o grupo 1");
}

/// O `shift` roda dentro de cada grupo, e grupos de tamanhos diferentes rodam cada um pelo
/// SEU tamanho — uma rotação sobre a lista toda daria outra coisa em ambos.
#[test]
fn the_shift_rotates_inside_each_group_by_its_own_length() {
    let p = row(5);
    // Grupo 0 com três peças (0,1,2) e grupo 1 com duas (3,4).
    let g = [0.0, 0.0, 0.0, 1.0, 1.0];
    assert_eq!(perm_x(&p, &g, false, 1), vec![1, 2, 0, 4, 3]);
    // ⚠️ `shift = 3` é a identidade no grupo de TRÊS e uma troca no de DOIS — é isto que
    // prova que o módulo é o do grupo e não o da lista.
    assert_eq!(perm_x(&p, &g, false, 3), vec![0, 1, 2, 4, 3]);
}

/// Empates nas DUAS chaves mantêm a ordem de chegada — o `sort_by` é estável, e é essa
/// estabilidade que faz *"reordenar os pontos não re-distribui as formas"* continuar verdade
/// a jusante.
#[test]
fn a_tie_in_both_keys_keeps_the_order_it_arrived_in() {
    let p = vec![[0.0, 0.0]; 4];
    let g = [1.0, 0.0, 1.0, 0.0];
    assert_eq!(perm_x(&p, &g, false, 0), vec![1, 3, 0, 2]);
}

/// Uma coluna de grupo mais CURTA que a lista: quem falta cai no grupo `0`. Sem isto, um
/// `value.*` que devolvesse menos linhas faria o nó indexar fora e escolher entre entrar em
/// pânico ou inventar — e inventar em silêncio é o que esta casa não faz.
#[test]
fn a_short_group_column_puts_the_missing_ones_in_group_zero() {
    let p = row(4);
    let g = [5.0, 5.0]; // só duas das quatro
    assert_eq!(
        perm_x(&p, &g, false, 0),
        vec![2, 3, 0, 1],
        "as pecas 2 e 3 caem no grupo 0 e vem primeiro"
    );
}
