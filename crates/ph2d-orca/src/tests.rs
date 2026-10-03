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
        ignores: None,
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
    assert_eq!(
        c.velocity(1, None, DT),
        b.vel,
        "o corpo que não desvia não é resolvido"
    );
    assert!(
        inteiro[1] < metade[1] - 1e-3,
        "{inteiro:?} contra {metade:?}"
    );
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
    let base = vec![
        agente([0.0, 0.0], [2.0, 0.0]),
        agente([3.0, 0.1], [-2.0, 0.0]),
    ];
    let mut c = Crowd::new(base.clone(), params());
    let v = c.solve_all(|_| None, DT);
    // O 1.º resolve sobre a fotografia; o 2.º com a velocidade NOVA do 1.º.
    assert_eq!(
        v[0],
        Crowd::new(base.clone(), params()).velocity(0, None, DT)
    );
    let mut vista = base.clone();
    vista[0].vel = v[0];
    assert_eq!(v[1], Crowd::new(vista, params()).velocity(1, None, DT));
    // CONTROLO: a velocidade nova do 1.º mudou, logo a régua de cima distingue as duas leituras.
    assert_ne!(v[0], base[0].vel);
    assert_ne!(v[1], Crowd::new(base, params()).velocity(1, None, DT));
}

#[test]
fn o_alvo_ignorado_nao_desvia_ninguem() {
    let a = agente([0.0, 0.0], [2.0, 0.0]);
    let mut b = agente([3.0, 0.1], [-2.0, 0.0]);
    b.avoids = false;
    let mut a_ignora = a;
    a_ignora.ignores = Some(1);
    assert_eq!(
        Crowd::new(vec![a_ignora, b], params()).velocity(0, None, DT),
        [2.0, 0.0]
    );
    // CONTROLO: sem o «ignora», o mesmo corpo desvia-o.
    assert_ne!(
        Crowd::new(vec![a, b], params()).velocity(0, None, DT),
        [2.0, 0.0]
    );
}

/// ⭐ **Passa pela DIREITA** — no empate exacto, o que anda para `+x` sai para `−y` (a direita dele
/// com o `y` para cima), e o outro em espelho. Sem esta régua a preferência podia trocar de lado sem
/// que o banco de cenários o visse (passar pela esquerda também desfaz o empate).
///
/// ⚠️ **A `4,5 m`, no regime do CORTE do cone:** é aí que o empate PRENDE (o semi-plano é
/// perpendicular ao caminho e a resposta é só travar) — a velocidade relativa `4` cai no círculo do
/// corte, a `|4 − 4,5| = 0,5 < R/τ = 1` do centro. Perto, no regime das PERNAS, o próprio ORCA escolhe
/// um lado (a 1.ª redacção pôs os dois a `2 m` e o CONTROLO saiu do eixo sem peso nenhum); a `6 m`
/// ninguém está ainda em rota de colisão no horizonte (a 2.ª).
#[test]
fn no_empate_cada_um_sai_pela_sua_direita() {
    let mut a = agente([0.0, 0.0], [2.0, 0.0]);
    let mut b = agente([4.5, 0.0], [-2.0, 0.0]);
    a.vel = [2.0, 0.0];
    b.vel = [-2.0, 0.0];
    let p = Params {
        side_bias: crate::SIDE_BIAS,
        ..params()
    };
    let mut c = Crowd::new(vec![a, b], p);
    let v = c.solve_all(|_| None, DT);
    assert!(v[0][1] < -1e-3, "o que vai para +x sai para −y: {:?}", v[0]);
    assert!(v[1][1] > 1e-3, "o que vai para −x sai para +y: {:?}", v[1]);
    // CONTROLO: sem o peso, nenhum sai do eixo (o empate do Godot).
    let mut c = Crowd::new(vec![a, b], params());
    let v = c.solve_all(|_| None, DT);
    assert_eq!((v[0][1], v[1][1]), (0.0, 0.0), "{v:?}");
}

/// ⭐ **O tecto de vizinhos corta pelos MAIS PERTO** — vinte numa fila, e só os `MAX_NEIGHBORS` mais
/// perto entram, por ordem. Sem esta régua tirar o tecto não mudava teste nenhum (nenhuma cena do
/// banco tem mais de oito).
#[test]
fn o_tecto_guarda_os_mais_perto() {
    let mut todos = vec![agente([0.0, 0.0], [0.0, 0.0])];
    for k in 1..=20 {
        todos.push(agente([0.7 * k as f64, 0.0], [0.0, 0.0]));
    }
    let c = Crowd::new(
        todos,
        Params {
            max_neighbors: Some(crate::MAX_NEIGHBORS),
            ..params()
        },
    );
    let mut nb = Vec::new();
    c.neighbors(0, &mut nb);
    let esperado: Vec<u32> = (1..=crate::MAX_NEIGHBORS as u32).collect();
    assert_eq!(nb, esperado);
}

/// ⚠️ **Num vértice onde a fronteira se toca a si própria, fica a continuação de MENOR índice** — a
/// ordem é a da malha, igual nos três sistemas (a prova de mutação da W6 achou-a sem régua).
#[test]
fn no_vertice_que_se_toca_fica_a_continuacao_de_menor_indice() {
    // Duas voltas que partilham o vértice 0 (um laço em oito).
    let verts: Vec<V2> = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [-1.0, 0.0],
        [-1.0, -1.0],
    ];
    let walls = [(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0)];
    let w = Walls::from_walkable_walls(&verts, &walls);
    // A entrada 0 vai de 1 a 0; a seguinte começa em 0 — há duas (as entradas 2 e 5): fica a 2.
    assert_eq!(w.next(0), 2);
    // A entrada 2 acaba... a anterior da entrada 2 (que começa em 0) acaba em 0: as entradas 0 e 3.
    assert_eq!(w.prev(2), 0);
}

/// (W9) As PAREDES sozinhas sem ponto em comum (um agente entalado entre duas): o 2D esvazia numa
/// parede, e o 3D começa ANTES do primeiro vizinho. Partia num `[n_walls..i]` ao contrário — achado
/// pela sonda `medir_replaneio` da ponte, com 200 agentes.
#[test]
fn entalado_entre_duas_paredes_o_3d_nao_parte() {
    use crate::lines::Line;
    use crate::lp::{Regime, solve};
    let linhas = [
        // vy ≥ 1, e vy ≤ −1: as duas paredes excluem-se.
        Line {
            point: [0.0, 1.0],
            dir: [1.0, 0.0],
        },
        Line {
            point: [0.0, -1.0],
            dir: [-1.0, 0.0],
        },
        // Um vizinho à direita.
        Line {
            point: [1.0, 0.0],
            dir: [0.0, 1.0],
        },
    ];
    let (v, regime) = solve(&linhas, 2, 2.0, [0.5, 0.0]);
    assert_eq!(regime, Regime::Dense);
    assert!(
        v[0].is_finite() && v[1].is_finite() && len(v) <= 2.0 + 1e-9,
        "{v:?}"
    );
}

/// (W9) A GRELHA das paredes devolve, ao bit, o que a varredura inteira devolve — os mesmos índices
/// pela mesma ordem —, em paredes ao calhas (polígonos e segmentos soltos, arestas compridas que
/// atravessam muitas células), com posições dentro e FORA da caixa e alcances de zero a infinito.
#[test]
fn a_grelha_das_paredes_da_o_mesmo_que_a_varredura_inteira() {
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    let mut r = || {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    let mut polys: Vec<Vec<V2>> = Vec::new();
    for k in 0..300 {
        let c = [r() * 100.0, r() * 100.0];
        if k % 10 == 0 {
            // Um segmento solto, às vezes comprido.
            polys.push(vec![
                c,
                [c[0] + (r() - 0.5) * 60.0, c[1] + (r() - 0.5) * 60.0],
            ]);
        } else {
            let (hx, hy) = (0.2 + r() * 2.0, 0.2 + r() * 2.0);
            polys.push(vec![
                [c[0] - hx, c[1] - hy],
                [c[0] + hx, c[1] - hy],
                [c[0] + hx, c[1] + hy],
                [c[0] - hx, c[1] + hy],
            ]);
        }
    }
    let w = Walls::from_polygons(&polys);
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let mut com_algum = 0;
    for k in 0..2_000 {
        let pos = [r() * 140.0 - 20.0, r() * 140.0 - 20.0];
        let range = match k % 5 {
            0 => 0.0,
            1 => f64::INFINITY,
            _ => r() * 12.0,
        };
        w.near(pos, range, &mut a);
        w.near_todas(pos, range, &mut b);
        assert_eq!(a, b, "pos {pos:?}, alcance {range}");
        com_algum += usize::from(!a.is_empty());
    }
    // A fixtura contém o fenómeno: a maioria das perguntas acha paredes.
    assert!(com_algum > 1_000, "{com_algum} de 2 000 acharam paredes");
}
