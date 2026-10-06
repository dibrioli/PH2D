//! Os gates do mundo de contacto (doc 121 §9.20) — pela porta [`super::passo`].

use super::*;
use ph2d_contact::obstaculo::{self, FormaFixa};
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column, Stream};

const DT: f32 = 1.0 / 60.0;

/// Uma caixa `0,22 × 0,22` por posição, todas com `id` e relógio em `t`.
fn caixas(p: &[[f32; 2]], t: f32) -> Stream {
    let n = p.len();
    #[expect(clippy::cast_precision_loss, reason = "um id pequeno")]
    let ids = (0..n).map(|i| i as f32).collect();
    Stream::new(n)
        .with("P", Column::Vec2(p.to_vec()))
        .with("vel", Column::Vec2(vec![[0.0, 0.0]; n]))
        .with("id", Column::Scalar(ids))
        .with("sim_t", Column::Scalar(vec![t; n]))
        .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![[0.11, 0.11]; n]))
}

/// O que um passo devolve: posições, velocidades, ângulos e o feito.
type Passo = (Vec<[f32; 2]>, Vec<[f32; 2]>, Vec<f32>, Option<Feito>);

/// Um passo com a gravidade `4` já na velocidade (a metade do `sim.step`), devolvendo o estado.
fn um(mundo: &mut Option<Mundo>, s: &Stream, t: f32) -> Passo {
    let n = s.count();
    let (Some(Column::Vec2(p0)), Some(Column::Vec2(v0))) = (s.get("P"), s.get("vel")) else {
        panic!("sem P/vel");
    };
    let rot0 = match s.get("rot") {
        Some(Column::Scalar(r)) => r.clone(),
        _ => vec![0.0; n],
    };
    let mut v: Vec<[f32; 2]> = v0.iter().map(|v| [v[0], v[1] - 4.0 * DT]).collect();
    let mut p = p0.clone();
    let (mut rot, mut spin) = (
        rot0.clone(),
        match s.get("spin") {
            Some(Column::Scalar(w)) => w.clone(),
            _ => vec![0.0; n],
        },
    );
    let sim_t = match s.get("sim_t") {
        Some(Column::Scalar(x)) => Some(x.clone()),
        _ => None,
    };
    let dt: Vec<f32> = vec![if sim_t.is_some() { DT } else { 0.0 }; n];
    let feito = passo(
        mundo,
        &Pedido {
            state: s,
            pesos: &vec![1.0; n],
            dt: &dt,
            sim_t: sim_t.as_deref(),
            playhead: t,
        },
        &mut Estado {
            antes: p0,
            rot_antes: &rot0,
            p: &mut p,
            vel: &mut v,
            rot: &mut rot,
            spin: &mut spin,
        },
    );
    (p, v, rot, feito)
}

/// Marcha `tiques` passos de uma pilha sobre um chão declarado, e devolve as posições.
fn marcha(mundo: &mut Option<Mundo>, mut s: Stream, t0: f32, tiques: u32) -> Stream {
    for k in 1..=tiques {
        #[expect(clippy::cast_precision_loss, reason = "um índice de tique")]
        let t = t0 + k as f32 * DT;
        let mut chao = s.clone();
        obstaculo::declara(
            &mut chao,
            7,
            FormaFixa::Plano {
                normal: [0.0, 1.0],
                altura: 0.0,
            },
            0.6,
            &vec![0.0; s.count()],
        );
        let (p, v, rot, _) = um(mundo, &chao, t);
        let n = s.count();
        s = s
            .with("P", Column::Vec2(p))
            .with("vel", Column::Vec2(v))
            .with("rot", Column::Scalar(rot))
            .with("sim_t", Column::Scalar(vec![t; n]));
    }
    s
}

fn bits(s: &Stream) -> Vec<u32> {
    match s.get("P") {
        Some(Column::Vec2(p)) => p
            .iter()
            .flat_map(|x| [x[0].to_bits(), x[1].to_bits()])
            .collect(),
        _ => Vec::new(),
    }
}

fn pilha() -> Stream {
    caixas(
        &(0..12)
            .map(|i| [((i % 3) as f32 - 1.0) * 0.23, 0.2 + (i / 3) as f32 * 0.3])
            .collect::<Vec<_>>(),
        0.0,
    )
}

/// ⭐⭐ **Duas corridas iguais dão os MESMOS bits** — o `enhanced-determinism` e a ordem do stream.
#[test]
fn two_runs_give_the_same_bits() {
    let (mut a, mut b) = (None, None);
    let (x, y) = (
        marcha(&mut a, pilha(), 0.0, 120),
        marcha(&mut b, pilha(), 0.0, 120),
    );
    assert_eq!(bits(&x), bits(&y));
    assert_eq!(a.as_ref().map(Mundo::passos), Some(120));
}

/// ⭐⭐⭐ **Uma CÓPIA do mundo continua com os bits do original** — é o que o ponto de recuo guarda,
/// e é isto que faz um recuo seguido de Play dar a 1.ª passagem (o gate do pump está na `=114`).
#[test]
fn a_copy_of_the_world_continues_with_the_bits_of_the_original() {
    let mut original = None;
    let meio = marcha(&mut original, pilha(), 0.0, 60);
    let mut copia = original.clone();
    let x = marcha(&mut original, meio.clone(), 1.0, 60);
    let y = marcha(&mut copia, meio, 1.0, 60);
    assert_eq!(bits(&x), bits(&y));
}

/// ⭐⭐ **Um stream que NÃO continua o mundo dá um mundo NOVO** — um recomeço (`Loop`) chega sem
/// `sim_t`, e a 2.ª volta tem de dar os bits da 1.ª.
#[test]
fn a_stream_that_does_not_continue_the_world_starts_a_new_one() {
    let mut m = None;
    let primeira = marcha(&mut m, pilha(), 0.0, 90);
    let passos_da_1a = m.as_ref().map(Mundo::passos);
    // O recomeço: o estado de partida, sem relógio, com o mundo velho na memória.
    let recomeco = pilha();
    let mut sem_relogio = Stream::new(recomeco.count());
    for (k, c) in recomeco.columns() {
        if k != "sim_t" {
            sem_relogio.set(k.clone(), c.clone());
        }
    }
    let _ = um(&mut m, &sem_relogio, 10.0);
    assert_eq!(
        m.as_ref().map(Mundo::passos),
        Some(0),
        "o recomeço nasce um mundo novo"
    );
    let segunda = marcha(&mut m, pilha(), 0.0, 90);
    assert_eq!(passos_da_1a, Some(90));
    assert_eq!(bits(&primeira), bits(&segunda));
}

/// ⭐ **O obstáculo declarado entra, devolve recibo, e SAI quando deixa de ser declarado.**
#[test]
fn a_declared_obstacle_enters_returns_a_receipt_and_leaves() {
    let mut m = None;
    let mut s = caixas(&[[0.0, 0.2]], 0.0);
    obstaculo::declara(
        &mut s,
        3,
        FormaFixa::Plano {
            normal: [0.0, 1.0],
            altura: 0.0,
        },
        0.5,
        &[0.0],
    );
    let (_, _, _, feito) = um(&mut m, &s, DT);
    assert_eq!(feito.map(|f| f.recibos), Some(vec![3]));
    let sem = caixas(&[[0.0, 0.2]], DT);
    let (_, _, _, feito) = um(&mut m, &sem, 2.0 * DT);
    assert_eq!(feito.map(|f| f.recibos), Some(Vec::new()));
}

/// ⭐⭐ **Uma caixa assenta NO obstáculo** (o plano em `y = 0`): a peça não o atravessa.
#[test]
fn a_box_rests_on_the_declared_floor() {
    let mut m = None;
    let fim = marcha(&mut m, caixas(&[[0.0, 0.5]], 0.0), 0.0, 120);
    let Some(Column::Vec2(p)) = fim.get("P") else {
        panic!()
    };
    assert!((p[0][1] - 0.11).abs() < 0.006, "pousada pela face: {p:?}");
}

/// ⭐⭐ **O ângulo sai ACUMULADO** — o rapier dá `(−π, π]`, e uma caixa a girar duas voltas tem de
/// sair com `720°`, não com o resto.
#[test]
fn the_angle_comes_out_accumulated() {
    let mut m = None;
    let mut s = caixas(&[[0.0, 5.0]], 0.0).with("spin", Column::Scalar(vec![720.0]));
    for k in 1..=60 {
        #[expect(clippy::cast_precision_loss, reason = "um índice de tique")]
        let t = k as f32 * DT;
        let (p, _, rot, _) = um(&mut m, &s, t);
        s = s
            .with("P", Column::Vec2(p))
            .with("vel", Column::Vec2(vec![[0.0, 0.0]]))
            .with("rot", Column::Scalar(rot))
            .with("sim_t", Column::Scalar(vec![t]));
    }
    let Some(Column::Scalar(r)) = s.get("rot") else {
        panic!()
    };
    assert!((r[0] - 720.0).abs() < 1.0, "duas voltas: {r:?}");
}

/// ⭐ **Quem nasce agora (sem relógio próprio) não se move neste tique** — entra DEPOIS do passo,
/// como sem o mundo.
#[test]
fn a_newborn_does_not_move_on_its_first_tick() {
    let mut m = None;
    let _ = marcha(&mut m, caixas(&[[0.0, 1.0]], 0.0), 0.0, 5);
    let t = 5.0 * DT;
    let s = caixas(&[[0.0, 1.0], [2.0, 3.0]], t);
    let n = 2;
    let (p0, rot0) = (vec![[0.0_f32, 1.0], [2.0, 3.0]], vec![0.0_f32; n]);
    let (mut p, mut v, mut rot, mut spin) = (
        p0.clone(),
        vec![[0.0_f32, -1.0]; n],
        rot0.clone(),
        vec![0.0; n],
    );
    let feito = passo(
        &mut m,
        &Pedido {
            state: &s,
            pesos: &[1.0, 1.0],
            dt: &[DT, 0.0],
            sim_t: Some(&[t, t]),
            playhead: t + DT,
        },
        &mut Estado {
            antes: &p0,
            rot_antes: &rot0,
            p: &mut p,
            vel: &mut v,
            rot: &mut rot,
            spin: &mut spin,
        },
    )
    .expect("há colisor");
    assert_eq!(feito.movidas, vec![true, false]);
    assert_eq!(p[1], [2.0, 3.0]);
    assert_eq!(m.as_ref().map(Mundo::pecas), Some(2));
}
