//! As leis do desvio, cada uma com a régua que a reprova.

use crate::v2::{V2, len, sub};
use crate::{Agent, Crowd, Params, Walls};

const DT: f64 = 1.0 / 60.0;

fn params() -> Params {
    Params {
        time_horizon: 1.0,
        time_horizon_walls: 1.0,
        neighbor_dist: None,
        max_neighbors: None,
        side_bias: 0.0,
    }
}

fn agente(pos: V2, pref: V2) -> Agent {
    Agent {
        pos,
        vel: pref,
        pref,
        radius: 0.5,
        max_speed: 2.0,
        avoids: true,
    }
}

#[test]
fn sozinho_anda_a_velocidade_que_quer_cortada_a_maxima() {
    let c = Crowd::new(vec![agente([0.0, 0.0], [1.5, 0.0])], params());
    assert_eq!(c.velocity(0, None, DT), [1.5, 0.0]);
    let c = Crowd::new(vec![agente([0.0, 0.0], [3.0, 4.0])], params());
    let v = c.velocity(0, None, DT);
    assert!((len(v) - 2.0).abs() < 1e-12, "{v:?}");
}

#[test]
fn frente_a_frente_os_dois_desviam_em_espelho() {
    // Ligeiramente desencontrados (um frente a frente EXACTO é o empate que o Godot não desfaz).
    let a = agente([0.0, 0.0], [2.0, 0.0]);
    let b = agente([3.0, 0.1], [-2.0, 0.0]);
    let c = Crowd::new(vec![a, b], params());
    let va = c.velocity(0, None, DT);
    let vb = c.velocity(1, None, DT);
    assert!(va[1] < -1e-3, "o de baixo desvia para baixo: {va:?}");
    assert!(vb[1] > 1e-3, "o de cima desvia para cima: {vb:?}");
    // O recíproco: cada um faz METADE ⇒ os desvios laterais são simétricos.
    assert!((va[1] + vb[1]).abs() < 1e-9, "{va:?} {vb:?}");
}

#[test]
fn quem_nao_desvia_obriga_o_outro_a_fazer_o_desvio_inteiro() {
    let a = agente([0.0, 0.0], [2.0, 0.0]);
    let mut b = agente([3.0, 0.1], [-2.0, 0.0]);
    let metade = Crowd::new(vec![a, b], params()).velocity(0, None, DT);
    b.avoids = false;
    let c = Crowd::new(vec![a, b], params());
    let inteiro = c.velocity(0, None, DT);
    assert_eq!(c.velocity(1, None, DT), b.vel, "o corpo que não desvia não é resolvido");
    assert!(inteiro[1] < metade[1] - 1e-3, "{inteiro:?} contra {metade:?}");
}

#[test]
/// A VISTA de um agente (os vizinhos dele, por ordem total) não depende da ordem da fotografia — só
/// [`Crowd::solve_all`] a usa, e de propósito.
fn a_vista_de_um_agente_nao_depende_da_ordem_da_fotografia() {
    let base = vec![
        agente([0.0, 0.0], [2.0, 0.0]),
        agente([3.0, 0.1], [-2.0, 0.0]),
        agente([1.5, -1.5], [0.0, 2.0]),
        agente([1.4, 1.6], [0.0, -2.0]),
    ];
    let c = Crowd::new(base.clone(), params());
    let v: Vec<V2> = (0..4).map(|i| c.velocity(i, None, DT)).collect();
    let perm = [2, 0, 3, 1];
    let c2 = Crowd::new(perm.iter().map(|&k| base[k]).collect(), params());
    for (pos, &k) in perm.iter().enumerate() {
        assert_eq!(c2.velocity(pos, None, DT), v[k], "agente {k}");
    }
}

#[test]
fn o_alcance_sem_perda_nao_muda_resposta_nenhuma() {
    // Um vizinho logo para lá do alcance: com o alcance infinito a resposta é a MESMA, bit a bit.
    let a = agente([0.0, 0.0], [2.0, 0.0]);
    let mut b = agente([0.0, 0.0], [-2.0, 0.3]);
    let alcance = crate::crowd::lossless_range(&a, &b, 1.0);
    b.pos = [alcance * 1.0001, 0.0];
    let p = params();
    let sem = Crowd::new(vec![a, b], p).velocity(0, None, DT);
    let infinito = Crowd::new(
        vec![a, b],
        Params {
            neighbor_dist: Some(1e9),
            ..p
        },
    )
    .velocity(0, None, DT);
    assert_eq!(sem, infinito);
    // CONTROLO: perto, o vizinho muda a resposta (senão a régua acima não media nada).
    b.pos = [2.0, 0.0];
    let perto = Crowd::new(vec![a, b], p).velocity(0, None, DT);
    assert_ne!(perto, [2.0, 0.0]);
}

/// Um quadrado andável de `0..10`, como a malha o daria (anti-horário, andável à esquerda).
fn sala() -> Walls {
    let v = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    Walls::from_walkable_walls(&v, &[(0, 1), (1, 2), (2, 3), (3, 0)])
}

#[test]
fn a_parede_da_malha_nunca_e_atravessada_no_horizonte() {
    let w = sala();
    // Encostado à parede da direita, a querer sair por ela a fundo.
    for (pos, pref) in [
        ([9.5, 5.0], [2.0, 0.0]),
        ([9.9, 9.9], [2.0, 2.0]),
        ([0.2, 5.0], [-2.0, 0.5]),
    ] {
        let mut a = agente(pos, pref);
        a.vel = pref;
        let c = Crowd::new(vec![a], params());
        let v = c.velocity(0, Some((&w, 0.0)), DT);
        let fim = [pos[0] + v[0] * 1.0, pos[1] + v[1] * 1.0];
        assert!(
            (-1e-9..=10.0 + 1e-9).contains(&fim[0]) && (-1e-9..=10.0 + 1e-9).contains(&fim[1]),
            "de {pos:?} a {pref:?}: {v:?} sai para {fim:?}"
        );
    }
    // CONTROLO: sem as paredes, o mesmo pedido sai.
    let a = agente([9.5, 5.0], [2.0, 0.0]);
    let v = Crowd::new(vec![a], params()).velocity(0, None, DT);
    assert!(9.5 + v[0] > 10.0);
}

#[test]
fn ao_longo_da_parede_anda_a_velocidade_inteira() {
    // Rente à parede de baixo, a andar paralelo a ela: nada a cortar.
    let w = sala();
    let a = agente([5.0, 0.3], [2.0, 0.0]);
    let v = Crowd::new(vec![a], params()).velocity(0, Some((&w, 0.0)), DT);
    assert!(len(sub(v, [2.0, 0.0])) < 1e-9, "{v:?}");
}

#[test]
fn em_sequencia_cada_um_ve_a_velocidade_nova_dos_anteriores() {
    let base = vec![agente([0.0, 0.0], [2.0, 0.0]), agente([3.0, 0.1], [-2.0, 0.0])];
    let mut c = Crowd::new(base.clone(), params());
    let v = c.solve_all(|_| None, DT);
    // O 1.º resolve sobre a fotografia; o 2.º com a velocidade NOVA do 1.º.
    assert_eq!(v[0], Crowd::new(base.clone(), params()).velocity(0, None, DT));
    let mut vista = base.clone();
    vista[0].vel = v[0];
    assert_eq!(v[1], Crowd::new(vista, params()).velocity(1, None, DT));
    // CONTROLO: a velocidade nova do 1.º mudou, logo a régua de cima distingue as duas leituras.
    assert_ne!(v[0], base[0].vel);
    assert_ne!(v[1], Crowd::new(base, params()).velocity(1, None, DT));
}
