//! **GATES DO MOTOR DE FLUXO** — a peça que a quantização inteira assenta em cima.
//!
//! ⚠️ **Estes gates existem porque um erro AQUI passa como `Ok`.** Medido em
//! 2026-08-20: os nós com excesso estavam pendurados no *sorvedouro* em vez de na
//! *fonte*. O fluxo fechava, `solve` devolvia `Ok`, e a resposta satisfazia a
//! conservação de **outro** problema — a única coisa que acusou foi a força
//! bruta, quatro camadas acima. Um motor com uma resposta conhecida é mais barato
//! que uma bissecção.

use ph2d_quantize::mcf::{Mcf, McfError};

/// A conservação medida diretamente: entrada menos saída, por nó.
fn balance(mcf: &Mcf, nodes: usize, arcs: &[(usize, usize, usize)]) -> Vec<i64> {
    let mut bal = vec![0i64; nodes];
    for &(from, to, id) in arcs {
        let f = mcf.flow(id);
        bal[to] += f;
        bal[from] -= f;
    }
    bal
}

#[test]
fn a_chain_routes_the_demand_and_pays_for_every_hop() {
    let mut m = Mcf::new(3);
    let a = m.arc(0, 1, 10, 1.0);
    let b = m.arc(1, 2, 10, 1.0);
    m.demand(0, -5);
    m.demand(2, 5);
    assert_eq!(m.solve(usize::MAX), Ok(10.0));
    assert_eq!(m.flow(a), 5);
    assert_eq!(m.flow(b), 5);
    assert_eq!(balance(&m, 3, &[(0, 1, a), (1, 2, b)]), vec![-5, 0, 5]);
}

#[test]
fn the_cheaper_of_two_parallel_routes_wins() {
    let mut m = Mcf::new(4);
    // Duas rotas de 0 a 3: uma cara (custo 5 por salto) e uma barata (1).
    let up1 = m.arc(0, 1, 3, 5.0);
    let up2 = m.arc(1, 3, 3, 5.0);
    let dn1 = m.arc(0, 2, 2, 1.0);
    let dn2 = m.arc(2, 3, 2, 1.0);
    m.demand(0, -4);
    m.demand(3, 4);
    // 2 pela barata (4) + 2 pela cara (20) = 24.
    assert_eq!(m.solve(usize::MAX), Ok(24.0));
    assert_eq!(m.flow(dn1), 2, "a barata satura primeiro");
    assert_eq!(m.flow(up1), 2);
    assert_eq!(
        balance(&m, 4, &[(0, 1, up1), (1, 3, up2), (0, 2, dn1), (2, 3, dn2)]),
        vec![-4, 0, 0, 4]
    );
}

#[test]
fn a_negative_cost_arc_is_used_to_the_hilt_and_still_conserves() {
    // ⚠️ O caso que a pré-saturação existe para tratar: um arco cujo custo é
    // negativo tem de ser preenchido até ao teto, e o desequilíbrio que isso
    // cria tem de ser drenado — **sem** desfazer o ganho.
    let mut m = Mcf::new(3);
    let good = m.arc(0, 1, 4, -2.0);
    let back = m.arc(1, 0, 4, 1.0);
    let out = m.arc(0, 2, 10, 0.0);
    m.demand(0, -3);
    m.demand(2, 3);
    // 4 unidades pelo arco de -2 (−8), 4 de volta a +1 (+4), 3 para o sorvedouro.
    assert_eq!(m.solve(usize::MAX), Ok(-4.0));
    assert_eq!(m.flow(good), 4);
    assert_eq!(m.flow(back), 4);
    assert_eq!(m.flow(out), 3);
    assert_eq!(
        balance(&m, 3, &[(0, 1, good), (1, 0, back), (0, 2, out)]),
        vec![-3, 0, 3]
    );
}

#[test]
fn a_demand_with_no_capacity_is_refused_not_rounded() {
    let mut m = Mcf::new(2);
    m.arc(0, 1, 2, 1.0);
    m.demand(0, -5);
    m.demand(1, 5);
    assert_eq!(
        m.solve(usize::MAX),
        Err(McfError::Infeasible { missing: 3 })
    );
}

#[test]
fn a_node_with_surplus_sends_it_away_instead_of_doubling_it() {
    // ⭐ **O gate exato do bug de 2026-08-20.** O arco `0 -> 1` tem custo
    // negativo, logo é pré-saturado com 4; mas o nó 1 só deve ficar com 1. As
    // outras 3 têm de SAIR do nó 1. Com fonte e sorvedouro trocados, o motor
    // acrescentava mais 3 em vez de as tirar — e devolvia `Ok`.
    let mut m = Mcf::new(3);
    let inflow = m.arc(0, 1, 4, -1.0);
    let drain = m.arc(1, 2, 10, 1.0);
    m.demand(0, -4);
    m.demand(1, 1);
    m.demand(2, 3);
    assert_eq!(m.solve(usize::MAX), Ok(-4.0 + 3.0));
    assert_eq!(m.flow(inflow), 4);
    assert_eq!(m.flow(drain), 3, "o excedente SAI do no 1");
    assert_eq!(
        balance(&m, 3, &[(0, 1, inflow), (1, 2, drain)]),
        vec![-4, 1, 3],
        "a conservacao tem de bater a demanda em TODO no"
    );
}
