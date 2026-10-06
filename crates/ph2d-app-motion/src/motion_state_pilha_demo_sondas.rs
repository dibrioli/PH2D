//! As SONDAS da `=114` aos milhares (doc 121 §9.19 (5)) — o cozimento, a prova dos impulsos da placa e o tique da
//! ponte. Irmã de `motion_state_pilha_demo_tests.rs` pelo tecto de LOC: lá vivem os gates, aqui as réguas.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// ⭐ doc 121 §9.19 (5) — **o custo do COZIMENTO da `=114` aos milhares**, por tique, com e sem `Collide`,
/// no mesmo processo (`docs/DevOps/MEDIR_VELOCIDADE.md`: intercaladas por tique, o mínimo e a mediana). É a
/// parte da CPU do `[frame] MOTION (cozer + separar)`: as duas taças com `lado × lado` peças cada, a taça a
/// crescer com a pilha (`medida_de`). Mede os tiques da queda e do contacto (o 2.º segundo de cena).
/// Ambiente: `PH2D_PILHA_LADOS=32,64,128`.
#[test]
#[ignore = "sonda de relógio"]
fn custo_do_cozimento_da_pilha() {
    let lados: Vec<f32> = std::env::var("PH2D_PILHA_LADOS").ok().map_or_else(
        || vec![32.0, 64.0, 128.0],
        |v| v.split(',').filter_map(|x| x.parse().ok()).collect(),
    );
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("PILHA load {}", carga().trim());
    for lado in lados {
        let mut cenas: Vec<(bool, MotionState, Vec<NodeId>)> = [true, false]
            .into_iter()
            .map(|colide| {
                let mut state = MotionState::new();
                let m = medida_de(Some(lado), colide);
                let sinks = build_com(&mut state.doc, &state.registry, &m).expect("a cena monta");
                crate::motion_shape_gen::publish(&mut state, 0.0);
                (colide, state, sinks)
            })
            .collect();
        let mut t = [Vec::new(), Vec::new()];
        let mut pecas = [0usize; 2];
        for k in 0..=120u32 {
            let tempo = f64::from(k) / 60.0;
            for (i, (_, state, sinks)) in cenas.iter_mut().enumerate() {
                let t0 = std::time::Instant::now();
                let mut n = 0;
                for sink in sinks.iter() {
                    let s = state
                        .pump
                        .cook
                        .cook(&state.doc.graph, &state.registry, *sink, tempo)
                        .expect("cozinha");
                    if let Some(Column::Vec2(v)) = s[0].as_stream().get("P") {
                        n += v.len();
                    }
                }
                // ⚠️ O tique AVANÇA a simulação (o `advance_tick`): sem ele o `cook` só lê o estado, e a
                // sonda media `0,02` ms por tique a `2 048` peças — a leitura, não a lei.
                state
                    .pump
                    .cook
                    .advance_tick(&state.doc.graph, &state.registry, tempo)
                    .expect("avanca");
                if k >= 60 {
                    t[i].push(t0.elapsed().as_secs_f64() * 1e3);
                }
                pecas[i] = n;
            }
        }
        for (i, (colide, _, _)) in cenas.iter().enumerate() {
            let mut o = t[i].clone();
            o.sort_by(f64::total_cmp);
            eprintln!(
                "PILHA lado {lado:>4} · {:>6} pecas · Collide {} · cozer por tique: min {:.2} ms · med {:.2} ms",
                pecas[i],
                if *colide { "ON " } else { "OFF" },
                o[0],
                o[o.len() / 2]
            );
        }
    }
    eprintln!("PILHA fim · load {}", carga().trim());
}

/// ⭐ doc 121 §9.19 (5) — **a PROVA do modelo do dispositivo**: a pilha da DIREITA (com `Collide`) no instante
/// `2,6` s — a distância mediana ao vizinho, a velocidade média (entre os dois últimos tiques) e a sobreposição
/// mais funda (o `contato` de cada par) — com os impulsos do produto (Gauss–Seidel) ou por Jacobi
/// (`PH2D_CONTACT_JACOBI=<iterações>`, lido uma vez por processo: uma corrida por variante).
/// Ambiente: `PH2D_PILHA_LADOS=5,32`.
#[test]
#[ignore = "sonda da prova"]
fn prova_dos_impulsos_da_placa() {
    let lados: Vec<f32> = std::env::var("PH2D_PILHA_LADOS").ok().map_or_else(
        || vec![5.0, 32.0],
        |v| v.split(',').filter_map(|x| x.parse().ok()).collect(),
    );
    let lei = std::env::var("PH2D_CONTACT_JACOBI").unwrap_or_else(|_| "GS".to_owned());
    for lado in lados {
        let mut state = MotionState::new();
        let m = medida_de(Some(lado), true);
        let sinks = build_com(&mut state.doc, &state.registry, &m).expect("a cena monta");
        crate::motion_shape_gen::publish(&mut state, 0.0);
        // O instante (`PH2D_PROVA_TIQUE`, tiques de `1/60` s): `156` (`2,6` s) na pilha pequena; a de `1 024`
        // só assenta no fim da pausa (`210`, `3,5` s).
        let ultimo: u32 = std::env::var("PH2D_PROVA_TIQUE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(156);
        let (mut antes, mut fim, mut fluxo) = (Vec::new(), Vec::new(), None);
        for k in 0..=ultimo {
            let t = f64::from(k) / 60.0;
            let s = state
                .pump
                .cook
                .cook(&state.doc.graph, &state.registry, sinks[1], t)
                .expect("cozinha")[0]
                .as_stream()
                .clone();
            if let Some(Column::Vec2(v)) = s.get("P") {
                if k + 1 == ultimo {
                    antes = v.clone();
                }
                if k == ultimo {
                    fim = v.clone();
                    fluxo = Some(s.clone());
                }
            }
            state
                .pump
                .cook
                .advance_tick(&state.doc.graph, &state.registry, t)
                .expect("avanca");
        }
        let s = fluxo.expect("o fluxo do fim");
        let col = ph2d_contact::colisores(&s).expect("a direita declara colisor");
        let mut funda = 0.0_f32;
        for i in 0..fim.len() {
            for j in (i + 1)..fim.len() {
                if let (Some(a), Some(b)) = (col[i], col[j])
                    && let Some(c) = ph2d_contact::contato(&a, fim[i], &b, fim[j], true)
                {
                    funda = funda.max(c.penetracao);
                }
            }
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de pecas")]
        let vel = fim
            .iter()
            .zip(&antes)
            .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]) * 60.0)
            .sum::<f32>()
            / fim.len() as f32;
        eprintln!(
            "PROVA lei {lei} · lado {lado} ({} pecas): vizinho mediano {:.5} · velocidade media {:.5} u/s · sobreposicao mais funda {:.5} ({:.2} % do lado)",
            fim.len(),
            super::tests::vizinho_mediano(&fim),
            vel,
            funda,
            funda / (2.0 * LADO) * 100.0
        );
    }
}

/// ⭐ doc 121 §9.19 (5) — **o tique do APP na pilha**: o `advance_or_scrub_scoped` que a ponte corre por quadro
/// (o cozimento, a simulação e o passe de acabamento do sink), `480` tiques (dois recomeços da queda), com o passe
/// e sem ele, intercalados por tique. Imprime a mediana, a média, o máximo e os tiques `> 5×` a mediana.
/// Ambiente: `PH2D_PILHA_LADOS=32,64`.
#[test]
#[ignore = "sonda de relógio"]
fn custo_do_tique_do_app_na_pilha() {
    let lados: Vec<f32> = std::env::var("PH2D_PILHA_LADOS").ok().map_or_else(
        || vec![32.0, 64.0],
        |v| v.split(',').filter_map(|x| x.parse().ok()).collect(),
    );
    for lado in lados {
        let mut cenas: Vec<(bool, MotionState, Vec<NodeId>)> = [true, false]
            .into_iter()
            .map(|passe| {
                let mut state = MotionState::new();
                let sinks = build_com(
                    &mut state.doc,
                    &state.registry,
                    &medida_de(Some(lado), true),
                )
                .expect("a cena monta");
                crate::motion_shape_gen::publish(&mut state, 0.0);
                (passe, state, sinks)
            })
            .collect();
        let mut t: [Vec<(u64, f64)>; 2] = Default::default();
        for tick in 0..480_u64 {
            for (i, (passe, state, sinks)) in cenas.iter_mut().enumerate() {
                let scopes =
                    ph2d_node_motion_time_remap::time_scopes(&state.doc.graph, &state.registry);
                let t0 = std::time::Instant::now();
                state.pump.set_separa_o_desenho(*passe);
                state.pump.advance_or_scrub_scoped(
                    &state.doc.graph,
                    &state.registry,
                    sinks,
                    tick,
                    |k| k as f64 / 60.0,
                    state.default_uv_rect,
                    state.default_size,
                    &scopes,
                );
                t[i].push((tick, t0.elapsed().as_secs_f64() * 1e3));
            }
        }
        for (i, (passe, _, _)) in cenas.iter().enumerate() {
            let mut o: Vec<f64> = t[i].iter().map(|x| x.1).collect();
            o.sort_by(f64::total_cmp);
            let med = o[o.len() / 2];
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de tiques")]
            let media = o.iter().sum::<f64>() / o.len() as f64;
            let picos: Vec<String> = t[i]
                .iter()
                .filter(|x| x.1 > 5.0 * med)
                .map(|x| format!("{}:{:.1}", x.0, x.1))
                .collect();
            eprintln!(
                "TIQUE lado {lado} · passe {} · mediana {med:.2} ms · media {media:.2} ms · max {:.2} ms · picos {:?}",
                if *passe { "ON " } else { "OFF" },
                o[o.len() - 1],
                picos
            );
        }
    }
}
