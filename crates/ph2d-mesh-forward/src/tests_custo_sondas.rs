//! ⭐ **O instrumento do CUSTO das capturas de reflexo** — o quadro inteiro a `1920 × 1080`, com e sem
//! capturas, ALTERNADOS (a máquina é partilhada: só a diferença dentro da mesma rodada vale):
//!
//! - **girar** — a câmara anda e a cena não: as capturas ficam como estão; o custo é a leitura no pixel;
//! - **arrastar** — uma peça anda a cada quadro: TODAS as capturas se refazem (as vizinhas dela mudam).
//!
//! As peças do oráculo (`3`) e grelhas de `8` e `16` (as mesmas malhas noutras poses), todas metal sobre
//! o chão. O quadro inclui a leitura de volta (a mesma dos dois lados).
//!
//! Corra (placa de exclusão, `load < 5`):
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-mesh-forward --lib
//!  instrumento_custo_sondas -- --ignored --nocapture`

use std::time::Instant;

use crate::tests::{ID, cena};
use crate::tests_chao_tapa::{PECAS, metal};
use crate::tests_contacto::camera_do_blender;
use crate::tests_reflexo::desenhista;
use crate::{Forward, Instancia};

const RODADAS: usize = 7;
const QUADROS: usize = 12;

/// `n` peças: as do oráculo e, a partir da 4.ª, as mesmas malhas numa grelha à volta.
fn pecas(n: usize) -> Vec<Instancia> {
    (0..n)
        .map(|k| {
            let mut m = ID;
            let (c, r, _) = PECAS[k % 3];
            let anel = (k / 3) as f32;
            let a = k as f32 * 2.399;
            let off = if k < 3 {
                [0.0; 2]
            } else {
                [
                    1.3 * (1.0 + 0.3 * anel) * a.cos(),
                    1.3 * (1.0 + 0.3 * anel) * a.sin(),
                ]
            };
            m[3][..3].copy_from_slice(&[c[0] + off[0], r, c[2] + off[1]]);
            if !PECAS[k % 3].2 {
                m[3][1] = c[1];
            }
            Instancia {
                malha: (k % 3) as u64 + 1,
                modelo: m,
            }
        })
        .collect()
}

/// A mediana do quadro (ms) de `QUADROS` quadros: a câmara gira (`arrasta = false`) ou a peça `0` anda.
fn mede(fw: &mut Forward, objs: &mut [Instancia], arrasta: bool) -> f64 {
    let mats = [metal(0.0), metal(0.2), metal(0.5)];
    let mut v = Vec::with_capacity(QUADROS);
    for q in 0..QUADROS {
        let a = q as f32 * 0.05;
        let de = if arrasta {
            [0.6, 0.35, 1.0]
        } else {
            [0.6 * a.cos() - a.sin(), 0.35, 0.6 * a.sin() + a.cos()]
        };
        if arrasta {
            objs[0].modelo[3][0] = PECAS[0].0[0] + 0.01 * q as f32;
        }
        let mut c = cena(objs, &mats, camera_do_blender(de, [0.0, 0.2, 0.15], 1.6));
        c.tamanho = (1920, 1080);
        c.chao = Some(0.0);
        c.caixa_tan = Some(0.47);
        let t0 = Instant::now();
        let _ = fw.quadro(&c);
        v.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    v.sort_by(f64::total_cmp);
    v[QUADROS / 2]
}

#[test]
#[ignore = "instrumento: precisa de aparelho"]
fn instrumento_custo_sondas() {
    let Some(mut fw) = desenhista() else {
        eprintln!("sem aparelho");
        return;
    };
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load {}", carga.split_whitespace().next().unwrap_or("?"));
    for n in [3usize, 8, 16] {
        let mut objs = pecas(n);
        for arrasta in [false, true] {
            let (mut sem, mut com) = (Vec::new(), Vec::new());
            for _ in 0..RODADAS {
                fw.liga_reflexos(false);
                sem.push(mede(&mut fw, &mut objs, arrasta));
                fw.liga_reflexos(true);
                com.push(mede(&mut fw, &mut objs, arrasta));
            }
            let med = |v: &mut Vec<f64>| {
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            let (s, c) = (med(&mut sem), med(&mut com));
            // Ao arrastar, a parte de cada passe: sem as vizinhas nas faces, e sem o pós-processamento.
            let mut partes = String::new();
            if arrasta {
                for (bit, nome) in [
                    (1u8, "sem desenhar as faces"),
                    (2, "sem o octaedro e o pré-filtro"),
                ] {
                    crate::gpu::sondas_passes::PULA
                        .store(bit, std::sync::atomic::Ordering::Relaxed);
                    let mut v: Vec<f64> = (0..RODADAS)
                        .map(|_| mede(&mut fw, &mut objs, true))
                        .collect();
                    crate::gpu::sondas_passes::PULA.store(0, std::sync::atomic::Ordering::Relaxed);
                    let x = med(&mut v);
                    partes.push_str(&format!(" · {nome} {x:.2} ms"));
                }
            }
            eprintln!(
                "{n:>2} peças · {}: sem capturas {s:.2} ms · com {c:.2} ms · {:+.2} ms{partes}",
                if arrasta {
                    "arrastar (refaz todas)"
                } else {
                    "girar (guardadas)"
                },
                c - s
            );
        }
    }
}
