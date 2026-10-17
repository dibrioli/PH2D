//! **SONDA — as curas do custo fixo da rota híbrida** (doc 120 §8.3.2). A rota de antes (`A`, o
//! carimbo recusado), a de hoje (`B`), um gémeo de `B` (o controlo da régua e da paridade) e `B` com
//! cada variante do [`ph2d_gpu_cook::VariantesDoCozimento`] — todas no MESMO processo, em blocos
//! intercalados por ordem rodada, resumo = o MÍNIMO das médias de bloco, mediana ao lado
//! ([MEDIR_VELOCIDADE](../../../docs/DevOps/MEDIR_VELOCIDADE.md)). Antes da régua, um bloco com a
//! diagnose das faltas da cache ligada (fora da régua: guardar cada buffer custa µs); no fim, a
//! saída de cada variante contra a de `B`, ao bit.
//!
//! `__GL_SHADER_DISK_CACHE=0 PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion \
//!     --profile smoke --lib sonda_as_curas_do_custo_fixo -- --ignored --nocapture --test-threads=1`
//! (`PH2D_SONDA_CENAS=120,132`, `PH2D_SONDA_BLOCO`, `PH2D_SONDA_RODADAS`).

use super::carimbo_escala::{FASES, amplia, le_as_fases, numero, rota_de_antes};
use super::medida::monta_demo;
use crate::motion_state::MotionState;
use ph2d_gpu_cook::VariantesDoCozimento;
use std::time::Instant;

/// `(nome, rota de antes, variantes)`. O índice `1` é a régua da paridade e da diferença.
fn variantes() -> Vec<(&'static str, bool, VariantesDoCozimento)> {
    let v = |luts, pool, passe| VariantesDoCozimento {
        luts_por_conteudo: luts,
        pool_por_ordem: pool,
        um_passe: passe,
        diagnostica: false,
    };
    vec![
        ("A", true, v(false, false, false)),
        ("B", false, v(false, false, false)),
        ("B·gémeo", false, v(false, false, false)),
        ("B+1a", false, v(true, false, false)),
        ("B+1b", false, v(false, true, false)),
        ("B+1", false, v(true, true, false)),
        ("B+2", false, v(false, false, true)),
        ("B+1+2", false, v(true, true, true)),
    ]
}

/// O que se lê de volta para a paridade: as instâncias (em texto, que dá a mesma resposta que os
/// bits fora de `NaN`) e as palavras das formas.
fn saida(gpu: &ph2d_gpu::GpuContext, m: &MotionState) -> (Vec<String>, Vec<u32>) {
    let inst = m
        .gpu_cook
        .instances()
        .map(|i| {
            ph2d_gpu_cook::read_instances(gpu, i)
                .iter()
                .map(|r| format!("{r:?}"))
                .collect()
        })
        .unwrap_or_default();
    let formas = m
        .gpu_cook
        .formas()
        .map(|f| ph2d_gpu_cook::read_formas(gpu, f))
        .unwrap_or_default();
    (inst, formas)
}

#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_as_curas_do_custo_fixo() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let (bloco, rodadas) = (
        numero("PH2D_SONDA_BLOCO", 20),
        numero("PH2D_SONDA_RODADAS", 7),
    );
    let so: Option<Vec<String>> = std::env::var("PH2D_SONDA_CENAS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect());
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "CURAS placa «{}» · bloco {bloco} · {rodadas} rodadas · load {}",
        gpu.adapter.get_info().name,
        carga().trim()
    );
    let dt = 1.0 / 60.0;
    let cam = crate::motion_shape_placa::gpu_tests::camara();
    let quadro = |m: &mut MotionState,
                  ph: &mut ph2d_core::Playhead,
                  placa: &mut crate::motion_shape_placa::PlacaDeFormas| {
        ph.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            m,
            Some(&gpu),
            ph,
            dt,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
        let t = Instant::now();
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let espera = t.elapsed();
        let t = Instant::now();
        let _ = crate::motion_shape_placa::decide_o_quadro(placa, &gpu, m, (true, true), cam);
        le_as_fases(m, espera, t.elapsed())
    };
    let casos: &[(&str, f32)] = &[
        ("120", 1.0),
        ("132", 1.0),
        ("119", 1.0),
        ("136", 1.0),
        ("112", 1.0),
        ("125", 1.0),
        ("111", 1.0),
        ("132", 11.0),
        ("136", 11.0),
    ];
    let vs = variantes();
    let nv = vs.len();
    let i_quadro = FASES.iter().position(|f| *f == "quadro").expect("a fase quadro");
    for &(n, k) in casos {
        if so.as_ref().is_some_and(|f| !f.iter().any(|x| x == n)) {
            continue;
        }
        if monta_demo(n).is_none() {
            eprintln!("CURAS ={n}: a cena não existe");
            continue;
        }
        let mut ms: Vec<MotionState> = vs
            .iter()
            .map(|(_, antes, v)| {
                let mut m = monta_demo(n).expect("a cena existe");
                if *antes {
                    rota_de_antes(&mut m);
                }
                amplia(&mut m, k);
                m.gpu_cook.usa_variantes(*v);
                m
            })
            .collect();
        let mut phs: Vec<ph2d_core::Playhead> = (0..nv)
            .map(|_| {
                let mut p = ph2d_core::Playhead::new(dt);
                p.play();
                p
            })
            .collect();
        let mut placas: Vec<crate::motion_shape_placa::PlacaDeFormas> =
            (0..nv).map(|_| Default::default()).collect();
        for _ in 0..30 {
            for i in 0..nv {
                quadro(&mut ms[i], &mut phs[i], &mut placas[i]);
            }
        }
        // A diagnose das faltas, fora da régua: o último de 5 quadros com ela ligada.
        for (i, (nome, _, v)) in vs.iter().enumerate() {
            ms[i].gpu_cook.usa_variantes(VariantesDoCozimento {
                diagnostica: true,
                ..*v
            });
            for _ in 0..5 {
                quadro(&mut ms[i], &mut phs[i], &mut placas[i]);
            }
            let c = ms[i].fases.cozimento;
            eprintln!(
                "CURAS ={n} ×{k} {nome} rota «{}» · por quadro: {} despachos de kernel, {} passes · grupos {} criados ({} com buffer novo, {} por combinação nova) / {} da cache · stream-op {} criados / {} da cache · {} LUT criadas · {} buffers alocados",
                ms[i].route_said.unwrap_or("?"),
                c.despachos,
                c.passes,
                c.grupos_criados,
                c.faltas_por_buffer_novo,
                c.faltas_por_combinacao,
                c.grupos_reaproveitados,
                c.grupos_de_stream_op_criados,
                c.grupos_de_stream_op_reaproveitados,
                c.luts_criadas,
                c.buffers_alocados
            );
            ms[i].gpu_cook.usa_variantes(*v);
        }
        let mut blocos: Vec<Vec<[f64; 21]>> = vec![Vec::new(); nv];
        for r in 0..rodadas {
            for j in 0..nv {
                let i = (j + r) % nv;
                let mut s = [0.0f64; 21];
                for _ in 0..bloco {
                    let (f, _) = quadro(&mut ms[i], &mut phs[i], &mut placas[i]);
                    for (a, b) in s.iter_mut().zip(f) {
                        *a += b / bloco as f64;
                    }
                }
                blocos[i].push(s);
            }
        }
        let resumo = |i: usize, j: usize| {
            let mut v: Vec<f64> = blocos[i].iter().map(|b| b[j]).collect();
            v.sort_by(f64::total_cmp);
            (v[0], v[v.len() / 2])
        };
        let (qa, _) = resumo(0, i_quadro);
        let (qb, _) = resumo(1, i_quadro);
        let lacuna = qb - qa;
        let regua = saida(&gpu, &ms[1]);
        for (i, (nome, _, _)) in vs.iter().enumerate() {
            let (q, med) = resumo(i, i_quadro);
            let paridade = if i == 0 {
                "—".to_string()
            } else {
                let s = saida(&gpu, &ms[i]);
                let difs = s.0.iter().zip(&regua.0).filter(|(a, b)| a != b).count()
                    + s.1.iter().zip(&regua.1).filter(|(a, b)| a != b).count();
                let tamanho = s.0.len() == regua.0.len() && s.1.len() == regua.1.len();
                if difs == 0 && tamanho {
                    format!("ao bit ({} instâncias, {} palavras)", s.0.len(), s.1.len())
                } else {
                    format!("DIFERE: {difs} (tamanhos iguais: {tamanho})")
                }
            };
            let fases: Vec<String> = ["o_quadro", "espera", "encode", "estagios", "finish", "submit"]
                .iter()
                .map(|f| {
                    let j = FASES.iter().position(|x| x == f).expect("fase");
                    let (a, b) = resumo(i, j);
                    format!("{f} {a:.1} ({b:.1})")
                })
                .collect();
            eprintln!(
                "CURAS ={n} ×{k} {nome} · quadro {q:.1} µs ({med:.1}) · B−variante {:+.1} µs = {:.0} % da lacuna B−A ({lacuna:+.1}) · {} · {paridade}",
                qb - q,
                if lacuna.abs() > f64::EPSILON { 100.0 * (qb - q) / lacuna } else { 0.0 },
                fases.join(" · ")
            );
        }
    }
    eprintln!("CURAS load no fim {}", carga().trim());
}
