//! **SONDA — o carimbo com `point_scale > 0` na placa** (aberto 1 do handoff LÍNGUA; doc 120 §8.3
//! W1(b).1 fez só `point_scale = 0`). A rota de ANTES (`A`: o kernel registado sem a variante, com a
//! recusa `point_scale <= 0`) contra a de HOJE (`B`), no MESMO processo: a rota medida pela porta do
//! produto e o quadro inteiro (`o_quadro` + a espera da placa) intercalado em blocos rodados, resumo =
//! o MÍNIMO ([MEDIR_VELOCIDADE](../../../docs/DevOps/MEDIR_VELOCIDADE.md)).
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib \
//!     sonda_o_carimbo_com_escala -- --ignored --nocapture --test-threads=1`
//! (`PH2D_SONDA_CENAS=132,137`, `PH2D_SONDA_BLOCO`, `PH2D_SONDA_RODADAS`).

use super::medida::{Cena, mede, monta_demo};
use crate::motion_state::MotionState;
use ph2d_nodegraph::gpu::{ApplicableFn, GpuKernel, KernelResolver};
use ph2d_nodegraph::node::NodeTypeId;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const CARIMBO: NodeTypeId = NodeTypeId::of("motion.duplicator");
/// As cenas que o handoff dá como caídas pelo carimbo, e as quatro fronteiras por confirmar.
const CENAS: &[&str] = &[
    "132", "136", "137", "139", "120", "111", "112", "119", "125", "141", "62", "131", "98", "134",
    "51",
];

static RECUSA_DE_HOJE: OnceLock<ApplicableFn> = OnceLock::new();

fn recusa_a_escala_do_ponto(p: &dyn Fn(&str) -> f32) -> bool {
    RECUSA_DE_HOJE.get().is_some_and(|f| f(p)) && p("point_scale") <= 0.0
}

/// A rota `A`: o kernel registado, sem variante, com a recusa de antes.
fn rota_de_antes(m: &mut MotionState) {
    let k = *m
        .registry
        .gpu_kernel(CARIMBO)
        .expect("o carimbo regista kernel");
    let _ = RECUSA_DE_HOJE.set(k.applicable.expect("o carimbo declara recusas"));
    m.registry.register_gpu_kernel(
        CARIMBO,
        GpuKernel {
            variant_by_param: None,
            applicable: Some(recusa_a_escala_do_ponto),
            ..k
        },
    );
}

/// Multiplica o lado de toda grelha da cena por `k` (cortado no tecto de lado): o MESMO grafo com
/// `k²` vezes os pontos, para medir o tecto e não o custo fixo do despacho.
fn amplia(m: &mut MotionState, k: f32) {
    let grelhas: Vec<_> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.grid")
        .map(|n| n.id)
        .collect();
    let teto = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
    for g in grelhas {
        for p in ["rows", "cols"] {
            let v = m
                .doc
                .graph
                .node_param_overrides(g)
                .and_then(|o| o.get(p).copied())
                .unwrap_or(3.0);
            m.doc.graph.set_param(g, p, (v * k).min(teto));
        }
    }
}

/// Porque é que cada carimbo que continua fronteira o é: os três params das recusas e o cartão
/// que alimenta os pontos.
fn porque_o_carimbo_fica(m: &mut MotionState, t: f64) -> Vec<String> {
    let plano = crate::motion_bridge::gpu::plano_do_produto(m, t);
    let g = &m.doc.graph;
    let tipo = |n| g.node(n).map(|i| i.type_name.clone()).unwrap_or_default();
    plano
        .boundaries
        .iter()
        .filter(|(n, _)| tipo(*n) == "motion.duplicator")
        .map(|(n, _)| {
            let o = g.node_param_overrides(*n);
            let p = |k: &str| o.and_then(|o| o.get(k).copied()).unwrap_or(0.0);
            let pontos = g.input_edge(*n, 1).map(|(s, _, pre)| (tipo(s), pre));
            let forma = g.input_edge(*n, 0).map(|(s, _, pre)| (tipo(s), pre));
            format!(
                "pick {} transfer {} point_scale {} · forma {forma:?} · pontos {pontos:?}",
                p("pick"),
                p("transfer"),
                p("point_scale")
            )
        })
        .collect()
}

fn numero(var: &str, omissao: usize) -> usize {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(omissao)
}

#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_o_carimbo_com_escala() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let (bloco, rodadas) = (
        numero("PH2D_SONDA_BLOCO", 20),
        numero("PH2D_SONDA_RODADAS", 7),
    );
    let so: Option<Vec<String>> = std::env::var("PH2D_SONDA_CENAS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect());
    let escalas: Vec<f32> = std::env::var("PH2D_SONDA_ESCALAS")
        .unwrap_or_else(|_| "1,4,11".into())
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "CARIMBO placa «{}» · bloco {bloco} · {rodadas} rodadas · load {}",
        gpu.adapter.get_info().name,
        carga().trim()
    );
    let dt = 1.0 / 60.0;
    let quadro = |m: &mut MotionState, ph: &mut ph2d_core::Playhead| -> Duration {
        ph.advance_ticks(1);
        let t0 = Instant::now();
        crate::motion_bridge::quadro::o_quadro(
            m,
            Some(&gpu),
            ph,
            dt,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        t0.elapsed()
    };
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    for &n in CENAS {
        if so.as_ref().is_some_and(|f| !f.iter().any(|x| x == n)) {
            continue;
        }
        let cena = Cena::Demo(n.to_string());
        let (Some(a), Some(b)) = (mede(&gpu, &cena, rota_de_antes), mede(&gpu, &cena, |_| {}))
        else {
            eprintln!("={n}: a cena não existe");
            continue;
        };
        eprintln!(
            "={n} ROTA\n  A {:?} · fronteiras {:?}\n    no processador {:?}\n  B {:?} · fronteiras {:?}\n    no processador {:?}",
            a.razoes, a.fronteiras, a.no_processador, b.razoes, b.fronteiras, b.no_processador
        );
        for &k in &escalas {
            // O quadro inteiro, A e B lado a lado no MESMO relógio da cena.
            let mut va = monta_demo(n).expect("a cena existe");
            rota_de_antes(&mut va);
            amplia(&mut va, k);
            let mut vb = monta_demo(n).expect("a cena existe");
            amplia(&mut vb, k);
            let mut ph = [ph2d_core::Playhead::new(dt), ph2d_core::Playhead::new(dt)];
            for p in &mut ph {
                p.play();
            }
            let (pa, pb) = ph.split_at_mut(1);
            for _ in 0..30 {
                quadro(&mut va, &mut pa[0]);
                quadro(&mut vb, &mut pb[0]);
            }
            if k == 1.0 {
                for r in porque_o_carimbo_fica(&mut vb, pb[0].time()) {
                    eprintln!("  B carimbo fronteira: {r}");
                }
            }
            let (mut ta, mut tb) = (Vec::new(), Vec::new());
            for r in 0..rodadas {
                let bloco_de = |m: &mut MotionState, p: &mut ph2d_core::Playhead| {
                    (0..bloco).map(|_| quadro(m, p)).sum::<Duration>() / bloco as u32
                };
                if r % 2 == 0 {
                    ta.push(bloco_de(&mut va, &mut pa[0]));
                    tb.push(bloco_de(&mut vb, &mut pb[0]));
                } else {
                    tb.push(bloco_de(&mut vb, &mut pb[0]));
                    ta.push(bloco_de(&mut va, &mut pa[0]));
                }
            }
            let resumo = |v: &mut Vec<Duration>| {
                v.sort();
                (ms(v[0]), ms(v[v.len() / 2]))
            };
            let ((a_min, a_med), (b_min, b_med)) = (resumo(&mut ta), resumo(&mut tb));
            let elementos = vb.gpu_cook.instances().map_or(0, |i| i.len());
            eprintln!(
                "={n} ×{k} QUADRO  A min {a_min:.3} ms (med {a_med:.3}) · B min {b_min:.3} ms (med {b_med:.3}) · B/A {:.3} · {elementos} instâncias na placa (B)",
                b_min / a_min
            );
        }
    }
    eprintln!("load no fim {}", carga().trim());
}
