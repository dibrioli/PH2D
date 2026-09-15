//! Os gates do plano de deslize que **não** precisam do corpus do oráculo (esse
//! vive em `tests/it/o_corpus_do_oraculo.rs`).

use super::*;

/// A diagonal unitária — ⚠️ a constante do `core`, e não `0,7071` escrito à mão: o `clippy` recusa
/// o literal, e tem razão (um dígito a menos muda o ângulo que o gate julga medir).
const DIAG: f32 = core::f32::consts::FRAC_1_SQRT_2;

#[test]
fn sem_velocidade_nao_ha_plano() {
    assert!(first_step([0.0, 0.0], 1.0 / 60.0, 4).is_none());
    // ⚠️ E um `dt` impossível também não: um plano com orçamento infinito seria
    // um pedido de deslocamento infinito ao mundo.
    assert!(first_step([4.0, 0.0], 0.0, 4).is_none());
    assert!(first_step([4.0, 0.0], f32::NAN, 4).is_none());
}

#[test]
fn o_orcamento_do_primeiro_passo_e_a_distancia_do_tique() {
    let s = first_step([3.0, 4.0], 1.0 / 60.0, 4).expect("ha' velocidade");
    assert!(
        (s.budget - 5.0 / 60.0).abs() < 1.0e-6,
        "orcamento {:.6}",
        s.budget
    );
    assert!(
        (crate::len(s.dir) - 1.0).abs() < 1.0e-6,
        "a direccao vem normalizada"
    );
}

#[test]
fn quando_tudo_coube_nao_ha_deslize() {
    let s = first_step([4.0, 0.0], 1.0 / 60.0, 4).unwrap();
    assert!(next_step(s, s.budget, [-1.0, 0.0], 15.0).is_none());
}

#[test]
fn de_cabeca_contra_a_parede_ele_para() {
    // 0° de incidência: a cláusula 2 corta antes de qualquer tangente.
    let s = first_step([4.0, 0.0], 1.0 / 60.0, 4).unwrap();
    assert!(next_step(s, 0.0, [-1.0, 0.0], 15.0).is_none());
}

#[test]
fn a_tangente_conserva_o_resto_do_orcamento() {
    // ⭐ A cláusula 1, sozinha e sem mundo: o que sobra do orçamento viaja
    // INTEIRO para a direcção nova. É isto que a projecção não faz.
    let v = [4.0 * DIAG, 4.0 * DIAG];
    let s = first_step(v, 1.0 / 60.0, 4).unwrap();
    let seguinte = next_step(s, 0.0, [-1.0, 0.0], 15.0).expect("45° desliza");
    assert!(
        (seguinte.budget - s.budget).abs() < 1.0e-7,
        "o resto tinha de ser {:.7} e e' {:.7} — a projeccao daria {:.7}",
        s.budget,
        seguinte.budget,
        s.budget * DIAG
    );
    assert!(
        seguinte.dir[0].abs() < 1.0e-6,
        "a tangente nao tem componente normal"
    );
    assert!(
        seguinte.dir[1] > 0.0,
        "e aponta para o mesmo lado da parede"
    );
}

#[test]
fn uma_normal_que_nao_se_opoe_ao_movimento_nao_produz_deslize() {
    // A ponte normaliza o sinal antes de chamar; se falhar, a lei recusa em vez
    // de deslizar para dentro da parede.
    let s = first_step([4.0, 0.0], 1.0 / 60.0, 4).unwrap();
    assert!(next_step(s, 0.0, [1.0, 0.0], 15.0).is_none());
    assert!(next_step(s, 0.0, [0.0, 0.0], 15.0).is_none());
}

#[test]
fn o_tecto_desce_a_cada_deslize() {
    let s = first_step([4.0 * DIAG, 4.0 * DIAG], 1.0 / 60.0, 2).unwrap();
    assert_eq!(s.slides_left, 2);
    let a = next_step(s, 0.0, [-1.0, 0.0], 15.0).unwrap();
    assert_eq!(a.slides_left, 1);
    // ⚠️ A normal tem de ser OBLIQUA: depois do 1.º deslize a direcção é `(0,1)`,
    // e uma normal `(0,−1)` seria FRONTAL — a cláusula do limiar cortava antes de
    // o tecto chegar a falar, que foi como a 1.ª redacção deste gate rebentou.
    const OBLIQUA: [f32; 2] = [-DIAG, -DIAG];
    let b = next_step(a, 0.0, OBLIQUA, 15.0).unwrap();
    assert_eq!(b.slides_left, 0);
    assert!(
        next_step(b, 0.0, [-1.0, 0.0], 15.0).is_none(),
        "o tecto fecha"
    );
}

#[test]
fn um_resto_minusculo_nao_vale_um_passo() {
    let s = first_step([4.0, 0.0], 1.0 / 60.0, 4).unwrap();
    let quase_tudo = s.budget - RESTO_MINIMO * 0.5;
    assert!(next_step(s, quase_tudo, [0.0, -1.0], 15.0).is_none());
}
