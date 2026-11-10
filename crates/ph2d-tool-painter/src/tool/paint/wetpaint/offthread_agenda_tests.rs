//! O agendamento da imagem sobre um relógio SIMULADO (T1-42): o laço do worker (`offthread_laco::worker_loop`) a
//! contar a dívida como ele, com passo e cópia de custo fixo, e a regra pura a decidir em cada consulta.

use super::super::super::laco::MAX_BEHIND_S;
use super::*;

/// A UI pede no máximo uma imagem por quadro (uma no ar; o espelho volta antes do próximo passo).
const QUADRO_S: f64 = 1.0 / 60.0;
const DURANTE_S: f64 = 3.0;

struct Corrida {
    /// Inícios das cópias, s.
    inicios: Vec<f64>,
    passos: u64,
}

fn corre(passo_ms: f64, copia_ms: Option<f64>) -> Corrida {
    let passo = passo_ms / 1000.0;
    let mut agenda = Agenda::default();
    let (mut t, mut last, mut acc) = (0.0f64, 0.0f64, 0.0f64);
    let mut c = Corrida {
        inicios: Vec::new(),
        passos: 0,
    };
    let mut serve = |t: &mut f64, acc: f64, inicios: &mut Vec<f64>| {
        let Some(ms) = copia_ms else { return };
        let pedido = inicios.last().is_none_or(|&i| *t - i >= QUADRO_S);
        let m = Momento {
            agora_s: *t,
            acc,
            sim_corre: true,
        };
        if pedido && agenda.deve_servir(m) {
            inicios.push(*t);
            *t += ms / 1000.0;
            agenda.copiou(ms, *t);
        }
    };
    while t < DURANTE_S {
        acc = (acc + (t - last)).min(MAX_BEHIND_S);
        last = t;
        if acc < STEP_S {
            serve(&mut t, acc, &mut c.inicios);
            t += 0.004;
            continue;
        }
        t += passo;
        acc = (acc + (t - last)).min(MAX_BEHIND_S);
        last = t;
        acc -= STEP_S;
        c.passos += 1;
        serve(&mut t, acc, &mut c.inicios);
    }
    c
}

/// A tabela do T1-42: passo da água × custo da cópia (2 = estreita, 6,4/32 = faixas, 55 = Full a UHD, 130 = Full a
/// 4K na integrada). O intervalo entre imagens ≤ `cópia / FRACCAO_DA_COPIA` + um passo (o worker só serve em passo
/// fechado) — com a cópia estreita (≤ 20 ms) ≥ 1 imagem por 100 ms + um passo — e a água abranda ≤ a fracção.
#[test]
fn a_agua_no_tecto_recebe_imagem_a_tempo() {
    let mut falhas = Vec::new();
    for passo in [20.0, 24.0, 26.0, 45.0, 60.0] {
        let sem = corre(passo, None).passos as f64;
        for copia in [2.0, 6.4, 32.0, 55.0, 130.0] {
            let c = corre(passo, Some(copia));
            let mut maior = c.inicios.first().copied().unwrap_or(DURANTE_S);
            for w in c.inicios.windows(2) {
                maior = maior.max(w[1] - w[0]);
            }
            maior = maior.max(DURANTE_S - c.inicios.last().copied().unwrap_or(0.0));
            let tecto_ms = (copia / FRACCAO_DA_COPIA).max(QUADRO_S * 1000.0) + passo + copia;
            let agua = c.passos as f64 / sem;
            // A 1.ª cópia é de graça (sem anterior): a borda da janela vale uma cópia e um passo.
            let borda = copia / 1000.0 / DURANTE_S + 1.0 / sem;
            if maior * 1000.0 > tecto_ms + 1e-6 || agua < 1.0 - FRACCAO_DA_COPIA - borda {
                falhas.push(format!(
                    "passo {passo} cópia {copia}: {} imagens em {DURANTE_S} s, maior intervalo {:.0} ms \
                     (tecto {tecto_ms:.0}), água a {agua:.2}×",
                    c.inicios.len(),
                    maior * 1000.0,
                ));
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Sem passo a correr (a água parou), a imagem corre sempre: não há água a atrasar.
#[test]
fn a_agua_parada_e_servida_sempre() {
    let mut a = Agenda::default();
    a.copiou(130.0, 0.13);
    let m = Momento {
        agora_s: 0.14,
        acc: MAX_BEHIND_S,
        sim_corre: false,
    };
    assert!(a.deve_servir(m));
}
