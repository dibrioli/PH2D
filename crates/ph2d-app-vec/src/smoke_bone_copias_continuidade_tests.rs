//! ⭐⭐⭐ **GATE — a tampa da junta NÃO SALTA com a pose** (A13, report do dono de 2026-10-06: *«a
//! parte externa da junta se deforma aos saltos»*). Sobre a `=6` presa UMA vez e dobrada a passos
//! de `0,1°`: na tampa de fora de cada junta (a menos de `0,6` dela, do lado oposto aos membros), a
//! mudança do DESENHO entre quadros vizinhos (Hausdorff) contra a da imagem EXACTA do contorno.
//!
//! A lei: `Δ desenho ≤ Δ verdade + CHAO`. A amostragem do produto é uniforme (não depende da pose),
//! e o chão é o maior excesso MEDIDO nas janelas e nas três varreduras inteiras (sonda
//! `diag_a_continuidade_por_lei`). ⛔ O CONTROLO é um orçamento que muda com a pose (o mecanismo do
//! refino pelo esticão posto, que saltava `0,041`/`0,144`): ele tem de passar do chão.

use super::super::a13::{dist_pl, fechados_de};
use super::*;

const R: f64 = 0.6;

/// As raízes dos esqueletos (os ossos sem pai).
fn raizes(sim: &SimWorld) -> Vec<Entity> {
    ph2d_skeleton_live::skin_live::bone_segments(sim)
        .into_iter()
        .filter_map(|(b, _, _)| Entity::try_from_bits(b))
        .filter(|e| sim.world().get::<ph2d_ecs::ChildOf>(*e).is_none())
        .collect()
}

/// As juntas da barra `k` e a bissectriz dos dois ossos (para o lado dos membros).
fn juntas(sim: &SimWorld, k: usize) -> Vec<([f64; 2], [f64; 2])> {
    let segs = ph2d_skeleton_live::skin_live::bone_segments(sim);
    let un = |v: [f64; 2]| {
        let l = v[0].hypot(v[1]).max(1e-12);
        [v[0] / l, v[1] / l]
    };
    let mut out = Vec::new();
    for (_, a, b) in &segs {
        if (k == 0) != (b[0] < 0.0) {
            continue;
        }
        if let Some((_, _, c)) = segs
            .iter()
            .find(|(_, a2, _)| (a2[0] - b[0]).hypot(a2[1] - b[1]) < 1e-6)
        {
            let (u1, u2) = (
                un([a[0] - b[0], a[1] - b[1]]),
                un([c[0] - b[0], c[1] - b[1]]),
            );
            out.push((*b, un([u1[0] + u2[0], u1[1] + u2[1]])));
        }
    }
    out
}

/// Hausdorff de `a` e `b` restrita à tampa da junta `(j, m)`.
fn na_tampa(a: &[Vec<[f64; 2]>], b: &[Vec<[f64; 2]>], (j, m): ([f64; 2], [f64; 2])) -> f64 {
    let tampa = |p: &[f64; 2]| {
        let d = [p[0] - j[0], p[1] - j[1]];
        d[0].hypot(d[1]) <= R && d[0] * m[0] + d[1] * m[1] < 0.0
    };
    let perto = |y: &[Vec<[f64; 2]>]| -> Vec<Vec<[f64; 2]>> {
        y.iter()
            .map(|l| {
                l.iter()
                    .copied()
                    .filter(|p| (p[0] - j[0]).hypot(p[1] - j[1]) <= R + 0.4)
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let lado = |x: &[Vec<[f64; 2]>], y: &[Vec<[f64; 2]>]| {
        let y = perto(y);
        x.iter()
            .flatten()
            .filter(|p| tampa(p))
            .map(|p| dist_pl(&y, *p))
            .fold(0.0, f64::max)
    };
    lado(a, b).max(lado(b, a))
}

/// O maior excesso medido do produto: `0,0013` nas três varreduras inteiras (`3 000` passos) e
/// `0,0000` nas janelas; o controlo salta `0,0214` nas janelas e `0,0224` nas varreduras.
const CHAO: f64 = 0.0015;

/// O pior `(excesso, pose, barra)` de `amostragem` nas janelas do gate.
fn pior_salto(amostragem: super::super::saltos::Amostragem) -> (f64, (f32, f32), usize) {
    let passo = |a: f32, b: f32| -> Vec<f32> {
        (0..=((b - a) * 10.0).round() as u16)
            .map(|i| a + 0.1 * f32::from(i))
            .collect()
    };
    let janelas: Vec<Vec<(f32, f32)>> = vec![
        passo(159.8, 160.6)
            .into_iter()
            .map(|g| (170.0, g))
            .collect(),
        passo(174.5, 175.0)
            .into_iter()
            .map(|g| (170.0, g))
            .collect(),
        passo(172.0, 173.0)
            .into_iter()
            .map(|g| (g, 170.0))
            .collect(),
    ];
    let mut pior = (0.0_f64, (0.0, 0.0), 0);
    for poses in &janelas {
        let (g1, g2) = poses[0];
        let (mut sim, _, st, ids) = cena(g1, g2);
        let rs = raizes(&sim);
        assert_eq!(rs.len(), 2, "a =6 tem dois esqueletos");
        let mut agora = (g1, g2);
        type Quadro = (Vec<Vec<[f64; 2]>>, Vec<Vec<[f64; 2]>>);
        let mut antes: Vec<Option<Quadro>> = vec![None, None];
        for &(a, b) in poses {
            for r in &rs {
                crate::smoke_bone_par::dobra_duas(&mut sim, *r, a - agora.0, b - agora.1);
            }
            agora = (a, b);
            for barra in 0..2 {
                let br = Barra::de(&sim, &st, ids[barra]);
                let verdade = br.imagem(128);
                let desenho = polilinhas(&fechados_de(&br.assa(amostragem((a, b)))), 64);
                if let Some((v0, d0)) = &antes[barra] {
                    for jm in juntas(&sim, barra) {
                        let excesso = na_tampa(&desenho, d0, jm) - na_tampa(&verdade, v0, jm);
                        if excesso > pior.0 {
                            pior = (excesso, (a, b), barra);
                        }
                    }
                }
                antes[barra] = Some((verdade, desenho));
            }
        }
    }
    pior
}

#[test]
fn a_tampa_da_junta_nao_salta_com_a_pose() {
    use super::super::saltos::{amostragem_do_produto, amostragem_que_muda_com_a_pose};
    let pior = pior_salto(amostragem_do_produto);
    let ctl = pior_salto(amostragem_que_muda_com_a_pose);
    println!(
        "  pior salto acima da verdade: {:.4} em {:?}, barra {} · controlo {:.4} em {:?}",
        pior.0, pior.1, pior.2, ctl.0, ctl.1
    );
    assert!(
        ctl.0 > 2.0 * CHAO,
        "o CONTROLO (orçamento que muda com a pose) salta só {:.4} — a régua deixou de ver saltos",
        ctl.0
    );
    assert!(
        pior.0 <= CHAO,
        "a tampa da junta salta {:.4} acima da verdade em {:?} (barra {}) — o conjunto de amostras \
         do bake mudou com a pose",
        pior.0,
        pior.1,
        pior.2
    );
}
