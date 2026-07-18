//! **DIAGNÓSTICO: de que lado o contorno erra?**
//!
//! Não é um gate — é a medição que decidiu o desenho da dilatação, guardada porque a
//! conclusão dela é contraintuitiva e cara de re-descobrir.
//!
//! `cargo test -p ph2d-flip-fill --test probe_offsets -- --nocapture`
//!
//! # O que ela mostra
//!
//! O erro de vetorização do contorno (marching squares → `expand_under_ink` → RDP →
//! alisamento) é **DE UM LADO SÓ**: o contorno cai sistematicamente *dentro* do eixo,
//! quase nunca fora. Medido em 6 combinações de espessura e tremor, `s` (o desvio com
//! sinal, positivo = aquém do eixo) vai de +0,007 a +0,875, com **zero pontos negativos**
//! em 5 delas e 3 em 99 na sexta, de magnitude 0,08.
//!
//! # Por que isso importa
//!
//! 1. **O sinal da compensação quase nunca dispara.** `w + 2s` e `w + 2·|s|` produzem
//!    números byte-idênticos nestas fixtures — o sinal é uma guarda de correção (e os 3
//!    pontos negativos existem), não a fonte do ganho. Quem melhora o encaixe é a
//!    **magnitude por ponto**, e é honesto dizer isso em vez de vender o sinal.
//! 2. **Ela corrige um veredito de 2026-07-18.** A compensação por ponto foi
//!    implementada, medida, julgada *pior* que uma margem uniforme e revertida sem
//!    shipar. O veredito estava errado, e a causa foi a MÉTRICA: mediu-se a *mediana da
//!    compensação* (0,0178 contra 0,005) — o tamanho do próprio remédio — em vez do
//!    defeito visível. Uma compensação maior não é um resultado pior: ela é maior porque
//!    o erro que ela cobre é maior. Medida no pixel, a mesma ideia corta a sub-cobertura
//!    de 158 para 138 amostras **sem mexer no transbordo**.
//!
//! > **Lição:** um número que sobe quando o remédio age não pode ser o critério de o
//! > remédio estar funcionando. Meça o SINTOMA (o que se vê na tela), nunca a dose.

use ph2d_core::Vec2;
use ph2d_flip_fill::{FillMode, FillParams, fill_at, nearest_on_axis};

#[test]
fn probe_offset_distribution() {
    println!("espessura | tremor | pontos | s_min | s_max | s_medio | NEGATIVOS");
    for width in [8.0f32, 16.0, 32.0] {
        for tremor in [0.0f32, 4.0] {
            let n = 200;
            let pts: Vec<Vec2> = (0..n)
                .map(|i| {
                    let t = i as f32 / n as f32 * std::f32::consts::TAU;
                    let h = ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
                    let rr = 110.0 + h * tremor;
                    Vec2::new(160.0 + rr * t.cos(), 160.0 + rr * t.sin())
                })
                .collect();
            let lines = vec![(pts.clone(), vec![width * 0.5; n], true)];
            let res = fill_at(
                &lines,
                Vec2::new(160.0, 160.0),
                FillParams {
                    precision: 1.6,
                    gap_reach: 0.0,
                    grow: 0,
                    trap_px: 0.0,
                    mode: FillMode::Paint,
                },
            )
            .expect("a arte preenche");

            // A normal externa aqui é RADIAL (o anel é ~circular e o centro é conhecido),
            // então esta sonda não depende do `outward_normals` que ela avalia.
            let mut neg = 0usize;
            let (mut mn, mut mx, mut sum) = (f32::MAX, f32::MIN, 0.0f32);
            for p in &res.outer {
                let (_, q) = nearest_on_axis(&lines, *p).expect("ha linha");
                let rad = Vec2::new(p.x - 160.0, p.y - 160.0);
                let l = (rad.x * rad.x + rad.y * rad.y).sqrt().max(1e-6);
                let s = (q.x - p.x) * (rad.x / l) + (q.y - p.y) * (rad.y / l);
                if s < -0.01 {
                    neg += 1;
                }
                mn = mn.min(s);
                mx = mx.max(s);
                sum += s;
            }
            println!(
                "{width:>9} | {tremor:>6} | {:>6} | {mn:>5.3} | {mx:>5.3} | {:>7.3} | {neg:>9}",
                res.outer.len(),
                sum / res.outer.len() as f32
            );
        }
    }
}
