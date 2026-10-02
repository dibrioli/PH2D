//! As leis puras da ronda: avançar pelos pontos alcançados, fechada dá voltas, aberta vai e volta.

use super::*;

const QUADRADO: [V2; 4] = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];

#[test]
fn fechada_da_voltas() {
    let mut r = Ronda::default();
    for (em, esperado) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
        avanca(&mut r, &QUADRADO, true, QUADRADO[em], 0.1);
        assert_eq!(r.ponto, esperado);
    }
}

#[test]
fn aberta_vai_e_volta() {
    let mut r = Ronda::default();
    let mut vistos = Vec::new();
    for _ in 0..7 {
        let em = QUADRADO[r.ponto as usize];
        avanca(&mut r, &QUADRADO, false, em, 0.1);
        vistos.push(r.ponto);
    }
    assert_eq!(vistos, vec![1, 2, 3, 2, 1, 0, 1]);
}

#[test]
fn longe_do_ponto_nao_avanca_e_pontos_juntos_avancam_todos() {
    let mut r = Ronda::default();
    avanca(&mut r, &QUADRADO, true, [2.0, 2.0], 0.1);
    assert_eq!(r.ponto, 0, "o CONTROLO: longe, fica");
    // Três pontos no mesmo sítio: um tique alcança-os todos.
    let juntos = [[0.0, 0.0], [0.0, 0.05], [0.05, 0.0], [5.0, 5.0]];
    avanca(&mut r, &juntos, true, [0.0, 0.0], 0.1);
    assert_eq!(r.ponto, 3);
}

#[test]
fn junta_se_pelo_mais_perto() {
    assert_eq!(mais_perto_de(&QUADRADO, [3.9, 3.5]), 2);
    assert_eq!(mais_perto_de(&QUADRADO, [-1.0, 0.1]), 0);
}
