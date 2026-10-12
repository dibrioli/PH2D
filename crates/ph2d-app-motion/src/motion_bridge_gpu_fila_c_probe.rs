//! **A FILA C** (doc 131 §4) — a sonda única de velocidade: variantes no MESMO processo, blocos
//! intercalados com a ordem rodada, o MÍNIMO, a carga ao lado (`docs/DevOps/MEDIR_VELOCIDADE.md`).
//!
//!   PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib -- \
//!     --ignored --nocapture fila_c

use super::cook_gpu;
use super::mistura_tests::{coze, estado_de, marcha_a_cpu};
use crate::motion_state::MotionState;
use ph2d_eval_motion::BlendWith;
use ph2d_gpu::GpuContext;

const DT: f64 = 1.0 / 60.0;

fn tipos_das_fronteiras(m: &MotionState) -> Vec<String> {
    m.pump
        .boundary_streams()
        .iter()
        .filter_map(|(n, _)| m.doc.graph.node(*n).map(|i| i.type_name.clone()))
        .collect()
}

fn cena(nivel: &str) -> MotionState {
    let mut m = MotionState::new();
    crate::motion_demo_legend::monta(nivel, &mut m.doc, &m.registry);
    m.sinks = super::super::remove::output_nodes(&m.doc.graph);
    m
}

/// O diagnóstico do C.8: que nós a rota híbrida coze na CPU (as fronteiras), por cena.
#[test]
#[ignore = "sonda: placa"]
fn fila_c_quem_e_fronteira() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adapter");
    let (mut estrelas, _, _) = estado_de(32.0, true, 0, BlendWith::Copies, false, false);
    marcha_a_cpu(&mut estrelas, 1);
    let _ = coze(&mut estrelas, &gpu, 2);
    eprintln!(
        "estrelas com wiggle: {:?} · fronteiras {:?}",
        estrelas.route_said,
        tipos_das_fronteiras(&estrelas)
    );
    for nivel in ["134", "135", "132"] {
        let mut m = cena(nivel);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        for t in 1..=3 {
            crate::motion_externals::publish_all(&mut m, t as f64 * DT);
            let _ = cook_gpu(&mut m, &gpu, t, DT, &scopes);
        }
        eprintln!(
            "={nivel}: {:?} · fronteiras {:?}",
            m.route_said,
            tipos_das_fronteiras(&m)
        );
    }
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

/// O MÍNIMO por chamada de cada variante: `blocos` blocos, a ordem rodada a cada bloco, `n` chamadas
/// por variante por bloco (MEDIR_VELOCIDADE.md).
fn intercalado(variantes: &mut [&mut dyn FnMut()], blocos: usize, n: u32) -> Vec<f64> {
    let k = variantes.len();
    let mut min = vec![f64::MAX; k];
    for b in 0..blocos {
        for j in 0..k {
            let v = (b + j) % k;
            let t = std::time::Instant::now();
            for _ in 0..n {
                (variantes[v])();
            }
            min[v] = min[v].min(t.elapsed().as_secs_f64() * 1e3 / f64::from(n));
        }
    }
    min
}

/// ⭐ **C.8 — DE ONDE VEM O PISO da rota híbrida** (~0,55 ms a 1 024 estrelas, comum às três rotas da
/// sonda do B6). As estrelas com `wiggle`: a única fronteira da CPU é o `source.shape` (uma linha).
/// Variantes no MESMO processo: o cozimento da placa com a espera · só a marcha do prefixo (a CPU)
/// · o cozimento sem esperar (só a submissão) · uma submissão VAZIA com espera (o piso do
/// dispositivo) · a marcha inteira na CPU (a rota da CPU, sem desenho).
#[test]
#[ignore = "sonda: placa, --profile smoke"]
fn fila_c_o_piso_hibrido() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adapter");
    let espera = |gpu: &GpuContext| {
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    };
    println!("\n=== C.8 · o piso híbrido · carga {} ===", carga());
    for lado in [32.0f32, 128.0] {
        let (mut dev, _, _) = estado_de(lado, true, 0, BlendWith::Copies, false, false);
        marcha_a_cpu(&mut dev, 1);
        let (mut so_dev, _, _) = estado_de(lado, true, 0, BlendWith::Copies, false, false);
        marcha_a_cpu(&mut so_dev, 1);
        let (mut pre, _, _) = estado_de(lado, true, 0, BlendWith::Copies, false, false);
        let (mut cpu, _, _) = estado_de(lado, true, 0, BlendWith::Copies, false, false);
        cpu.gpu_enabled = false;
        assert!(coze(&mut dev, &gpu, 2), "{:?}", dev.route_said);
        let fronteiras: Vec<_> = dev
            .pump
            .boundary_streams()
            .iter()
            .map(|(n, _)| *n)
            .collect();
        let scopes = ph2d_node_motion_time_remap::time_scopes(&cpu.doc.graph, &cpu.registry);
        let (mut t0, mut t1, mut t2, mut t3) = (3u64, 3u64, 3u64, 3u64);
        let mut a = || {
            assert!(coze(&mut dev, &gpu, t0));
            espera(&gpu);
            t0 += 1;
        };
        let mut b = || {
            let (g, r) = (&pre.doc.graph, &pre.registry);
            pre.pump.advance_or_scrub_to_nodes_scoped(
                g,
                r,
                &fronteiras,
                t1,
                |t| t as f64 * DT,
                &scopes,
            );
            t1 += 1;
        };
        let mut c = || {
            assert!(coze(&mut so_dev, &gpu, t2));
            t2 += 1;
        };
        let mut d = || {
            let enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            gpu.queue.submit(Some(enc.finish()));
            espera(&gpu);
        };
        let mut e = || {
            crate::motion_externals::publish_all(&mut cpu, t3 as f64 * DT);
            cpu.pump.advance_or_scrub_scoped(
                &cpu.doc.graph,
                &cpu.registry,
                &cpu.sinks,
                t3,
                |t| t as f64 * DT,
                cpu.default_uv_rect,
                cpu.default_size,
                &scopes,
            );
            t3 += 1;
        };
        for _ in 0..5 {
            a();
            b();
            c();
            d();
            e();
        }
        espera(&gpu);
        let m = intercalado(&mut [&mut a, &mut b, &mut c, &mut d, &mut e], 12, 10);
        println!(
            "  {:>6} estrelas: placa+espera {:.3} · prefixo (CPU) {:.3} · placa sem espera {:.3} · submissão vazia+espera {:.3} · CPU inteira {:.3} ms",
            (lado * lado) as u32,
            m[0],
            m[1],
            m[2],
            m[3],
            m[4]
        );
    }
}

/// ⭐ **C.12 — O ENGASGO DA TROCA DE ROTA** (CICLO_20 §9: `0,115 ms` por tique re-simulado, uma vez por
/// troca). A `=135` pelo quadro do PRODUTO (`coze_o_quadro`): o quadro em que a rota SAI da placa
/// contra um quadro da CPU sem troca. Cada bloco: placa → (cronometrado) CPU → placa.
#[test]
#[ignore = "sonda: placa, --profile smoke"]
fn fila_c_a_troca_de_rota() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adapter");
    println!(
        "\n=== C.12 · a troca de rota (=135) · carga {} ===",
        carga()
    );
    let novo = |placa: bool| {
        let mut m = cena("135");
        m.gpu_enabled = placa;
        m
    };
    let (mut troca, mut fixa) = (novo(true), novo(false));
    let mut ph_t = ph2d_core::Playhead::new(DT);
    let mut ph_f = ph2d_core::Playhead::new(DT);
    let (mut kt, mut kf) = (1u64, 1u64);
    let quadro = |m: &mut MotionState, ph: &mut ph2d_core::Playhead, k: &mut u64| {
        ph.seek(*k as f64 * DT);
        crate::motion_bridge::quadro::coze_o_quadro(m, &gpu, ph, DT);
        *k += 1;
    };
    for _ in 0..30 {
        quadro(&mut troca, &mut ph_t, &mut kt);
        quadro(&mut fixa, &mut ph_f, &mut kf);
    }
    let rota_placa = troca.route_said.unwrap_or("");
    let (mut min_troca, mut min_fixa) = (f64::MAX, f64::MAX);
    let mut rota_da_troca = "";
    for b in 0..24 {
        for v in [(b % 2), 1 - (b % 2)] {
            if v == 0 {
                troca.gpu_enabled = false;
                let t = std::time::Instant::now();
                quadro(&mut troca, &mut ph_t, &mut kt);
                min_troca = min_troca.min(t.elapsed().as_secs_f64() * 1e3);
                rota_da_troca = troca.route_said.unwrap_or("");
                troca.gpu_enabled = true;
                for _ in 0..4 {
                    quadro(&mut troca, &mut ph_t, &mut kt);
                }
            } else {
                let t = std::time::Instant::now();
                quadro(&mut fixa, &mut ph_f, &mut kf);
                min_fixa = min_fixa.min(t.elapsed().as_secs_f64() * 1e3);
            }
        }
    }
    println!(
        "  quadro que sai da placa {min_troca:.3} ms · quadro da CPU sem troca {min_fixa:.3} ms · engasgo {:.3} ms ({:.1} % de um quadro a 60 Hz)\n  rotas: placa {rota_placa:?} · na troca {rota_da_troca:?}",
        min_troca - min_fixa,
        (min_troca - min_fixa) / (1000.0 / 60.0) * 100.0
    );
}

/// ⭐ **C.11 — A FITA por entrada** (doc 128 §7: *«com MUITOS objectos a mexer são minutos — por
/// medir»*): `K` entradas a mudar em todo tique, os bytes por tique da fita e quanto tempo de história
/// cabe no orçamento `FITA_BYTES`.
#[test]
#[ignore = "sonda: CPU"]
fn fila_c_a_fita_por_entrada() {
    use ph2d_ecs::motion_inputs::{Escritor, Valor};
    println!(
        "\n=== C.11 · a fita por entrada (orçamento {} MiB) ===",
        ph2d_eval_motion::FITA_BYTES >> 20
    );
    for k in [3usize, 30, 300] {
        let mut m = MotionState::new();
        let g = &mut m.doc.graph;
        let grade = g.add_node("motion.grid");
        let out = g.add_node("motion.output");
        let _ = g.connect(ph2d_nodegraph::graph::Edge {
            from: (grade, 0),
            to: (out, 0),
            delayed: false,
        });
        for e in 0..k {
            let n = g.add_node("value.input");
            g.set_text_param(n, ph2d_node_value_input::NAME_KEY, format!("e{e}"));
        }
        m.sinks = vec![out];
        m.pump.define_a_lei(true);
        let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
        const TIQUES: u64 = 600;
        for t in 0..TIQUES {
            for e in 0..k {
                #[expect(clippy::cast_precision_loss, reason = "um valor de teste")]
                m.entradas.jogo.set(
                    &format!("e{e}"),
                    Escritor::Script,
                    Valor::Numero((t + e as u64) as f32),
                );
            }
            crate::motion_externals::publish_all(&mut m, t as f64 * DT);
            m.pump.advance_or_scrub_scoped(
                &m.doc.graph,
                &m.registry,
                &m.sinks,
                t,
                |t| t as f64 * DT,
                m.default_uv_rect,
                m.default_size,
                &scopes,
            );
        }
        let bytes = m.pump.fita().bytes();
        #[expect(clippy::cast_precision_loss, reason = "uma razão")]
        let por_tique = bytes as f64 / TIQUES as f64;
        #[expect(clippy::cast_precision_loss, reason = "uma razão")]
        let minutos = ph2d_eval_motion::FITA_BYTES as f64 / por_tique / 60.0 / 60.0;
        println!(
            "  {k:>4} entradas a mudar: {por_tique:.1} B/tique · {minutos:.1} min de história a 60 Hz"
        );
    }
}
