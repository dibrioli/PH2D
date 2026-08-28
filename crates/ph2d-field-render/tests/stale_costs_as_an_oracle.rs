//! ⭐⭐⭐ **O CUSTO DO QUADRO ANTERIOR SERVE DE ESTIMADOR?** (W96)
//!
//! # A pergunta, e porque ela é a que falta
//!
//! O oráculo do escalonamento (`measure_what_a_perfect_tile_schedule_would_buy`) diz que a ordem
//! **perfeita** compra `1,14×` a 32 threads com o ladrilho a `24` (era `1,76×` a `64`). Ele usa os
//! custos **verdadeiros**, medidos depois do facto — não é um estimador, é o tecto.
//!
//! ⛔ **A minha primeira tentativa de estimador foi RECUSADA por medição** (§89): a profundidade da
//! peça sob o ladrilho está **anti-correlacionada** com o custo — o caro é o da silhueta, não o do
//! meio. Mas há um estimador que ali não existia: **o custo do MESMO ladrilho no quadro anterior**.
//! Num arrasto, a câmera anda `~2°` por quadro, e um ladrilho caro continua caro.
//!
//! ⚠️ **A régua não é a correlação, é o ESCALONAMENTO**: o que interessa é o que a ordem do quadro
//! `k` faria ao quadro `k+1`, comparada com a ordem perfeita **dele**. *Uma correlação alta não
//! promete um escalonamento bom, e é o escalonamento que se paga.*
//!
//! ```text
//! cargo test -p ph2d-field-render --profile ci-test --test stale_costs_as_an_oracle -- --ignored --nocapture
//! ```

use ph2d_field::{FieldDoc, FillRule, Node, NodeId, NodeKind, Primitive, Profile, Xform};
use ph2d_field_render::Orbit;
use std::sync::atomic::Ordering::Relaxed;

fn circulo(n: usize) -> FieldDoc {
    let c: Vec<[f32; 2]> = (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * (i as f64) / (n as f64);
            [(0.6 * a.cos()) as f32, (0.6 * a.sin()) as f32]
        })
        .collect();
    FieldDoc::new(
        vec![Node {
            xform: Xform::IDENTITY,
            kind: NodeKind::Leaf(Primitive::Extrude {
                profile: Profile::new(vec![c], FillRule::NonZero, 1e-4).expect("perfil"),
                half_height: 0.4,
                round: 0.06,
            }),
            mods: Vec::new(),
        }],
        NodeId(0),
    )
    .expect("extrusão")
}

/// O tempo de parede de um escalonamento guloso: `n` obreiros, as tarefas na ordem dada.
fn makespan(custos: &[u64], threads: usize) -> u64 {
    let mut carga = vec![0u64; threads];
    for c in custos {
        let i = carga
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| **v)
            .map_or(0, |(i, _)| i);
        carga[i] += c;
    }
    carga.into_iter().max().unwrap_or(0)
}

/// Os custos por ladrilho de um quadro, na ordem natural, com o índice do ladrilho.
fn custos_de(
    doc: &FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    graus: f64,
) -> Vec<(usize, u64)> {
    let cam = Orbit {
        rotation: Orbit::from_yaw_pitch(0.72 + graus.to_radians() as f32, 0.52).rotation,
        ..Orbit::default()
    };
    ph2d_field_render::TILE_COSTS.lock().expect("mutex").clear();
    ph2d_field_render::RECORD_TILE_COSTS.store(true, Relaxed);
    let g = ph2d_field_render::trace_tiled_for_test(
        doc,
        reg,
        &cam,
        640,
        360,
        ph2d_field_render::tile_for_test(),
        ph2d_field_render::slabs_for_test(),
        false,
        false,
    )
    .expect("traçado");
    ph2d_field_render::RECORD_TILE_COSTS.store(false, Relaxed);
    assert!(g.hits() > 1000, "a peça saiu vazia — a sonda não mede nada");
    ph2d_field_render::TILE_COSTS.lock().expect("mutex").clone()
}

#[test]
#[ignore = "sonda; roda com --ignored --nocapture"]
fn measure_what_the_previous_frames_costs_would_buy() {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let doc = circulo(168);
    for passo in [1.0f64, 2.0, 4.0] {
        let antes = custos_de(&doc, &reg, 0.0);
        let agora = custos_de(&doc, &reg, passo);
        // ⚠️ Os ladrilhos são os MESMOS índices nos dois quadros (a grelha é da tela, não da peça),
        // mas um ladrilho pode não aparecer num deles — casa-se por índice e falta = custo zero.
        // ⚠️ `BTreeMap`, nunca `HashMap` — a espinha do determinismo desta casa (HR-5 / ADR-0022),
        // e o lint estrutural apanha-o mesmo numa sonda.
        let mut velho = std::collections::BTreeMap::new();
        for (i, c) in &antes {
            velho.insert(*i, *c);
        }
        let total: u64 = agora.iter().map(|(_, c)| *c).sum();
        // Ordem natural (a de hoje).
        let natural: Vec<u64> = agora.iter().map(|(_, c)| *c).collect();
        // Ordem por custo VERDADEIRO deste quadro — o tecto.
        let mut perfeita = natural.clone();
        perfeita.sort_unstable_by(|a, b| b.cmp(a));
        // Ordem pelo custo do quadro ANTERIOR — o estimador.
        let mut estimada: Vec<(u64, u64)> = agora
            .iter()
            .map(|(i, c)| (velho.get(i).copied().unwrap_or(0), *c))
            .collect();
        estimada.sort_unstable_by_key(|(velho, _)| std::cmp::Reverse(*velho));
        let estimada: Vec<u64> = estimada.into_iter().map(|(_, c)| c).collect();
        println!(
            "--- arrasto de {passo}° por quadro · {} ladrilhos ---",
            agora.len()
        );
        println!(
            "threads | ideal | natural | ESTIMADA (quadro anterior) | perfeita | natural/ideal | estimada/ideal"
        );
        for t in [8usize, 16, 32] {
            let ideal = (total as f64 / t as f64).ceil() as u64;
            let (n, e, p) = (
                makespan(&natural, t),
                makespan(&estimada, t),
                makespan(&perfeita, t),
            );
            println!(
                "{t:7} | {ideal:9} | {n:9} | {e:9} | {p:9} | {:8.2}x | {:8.2}x",
                n as f64 / ideal as f64,
                e as f64 / ideal as f64
            );
        }
    }
}
