//! ⭐ **O RELÓGIO DO GRUPO DO RITMO** (ciclo 17, passo 5 —
//! [doc 126](../../../docs/Motion%20Nodes/126_ciclo_17_o_ritmo.md) §5).
//!
//! A pergunta do ciclo: as famílias novas (`Sine` · `Expo` · `Elastic` · `Steps` · `Spring` ·
//! `Bezier`) e as ordens no espaço (`Grid` · `From Point`) custam na placa o mesmo que as de antes?
//! O `Spring` avalia a mola DUAS vezes (a normalização `x(t)/x(1)`) e o `Bezier` faz 24 bissecções
//! por linha — são os candidatos a caro. A sonda reusa a régua do ciclo 7
//! ([`super::aparencia_relogio`]): a MESMA cadeia pelos dois motores, intercalados quadro a
//! quadro, mínimo e mediana.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture measure_the_rhythm_group
//! ```
use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, NodeId};
use ph2d_render::SinkStyle;

const DT: f64 = 1.0 / 60.0;
const AQUECE: u64 = 20;
const AMOSTRAS: u64 = 31;
const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const TAMANHO: [f32; 2] = [0.4, 0.4];

/// Um caso: o nó, os params que o acordam, e se o valor vai por um `motion.drive`.
struct Caso {
    rotulo: &'static str,
    no: &'static str,
    params: &'static [(&'static str, f32)],
    via_drive: bool,
}

const ONDA: &[(&str, f32)] = &[("channel", 1.0), ("min", -0.8), ("max", 2.2)];

const GRUPO: &[Caso] = &[
    Caso {
        rotulo: "stagger Linear",
        no: "motion.stagger",
        params: &[("ease_curve", 0.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Quad",
        no: "motion.stagger",
        params: &[("ease_curve", 1.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Bounce",
        no: "motion.stagger",
        params: &[("ease_curve", 7.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Sine",
        no: "motion.stagger",
        params: &[("ease_curve", 9.0), ("ease_dir", 2.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Expo",
        no: "motion.stagger",
        params: &[("ease_curve", 10.0), ("ease_dir", 1.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Elastic",
        no: "motion.stagger",
        params: &[("ease_curve", 11.0), ("ease_dir", 1.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Steps",
        no: "motion.stagger",
        params: &[("ease_curve", 12.0), ("steps", 6.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Spring",
        no: "motion.stagger",
        params: &[("ease_curve", 13.0), ("damping", 0.25)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Bezier",
        no: "motion.stagger",
        params: &[("ease_curve", 14.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger Grid",
        no: "motion.stagger",
        params: &[("order", 3.0), ("grid_cols", 181.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "stagger From Point",
        no: "motion.stagger",
        params: &[("order", 4.0), ("reach", 30.0)],
        via_drive: false,
    },
    Caso {
        rotulo: "time delay Index",
        no: "value.time",
        params: &[("delay", 1.5)],
        via_drive: true,
    },
    Caso {
        rotulo: "time delay Point+Spr",
        no: "value.time",
        params: &[
            ("delay", 1.5),
            ("order", 4.0),
            ("reach", 30.0),
            ("ease_curve", 13.0),
        ],
        via_drive: true,
    },
    // ⭐ Ciclo 18 (doc 127 §5.4): o gradiente do Tint nos espaços perceptuais (vermelho → azul,
    // o par que atravessa a gama no OKLCH longo — o pior caso do mapeamento).
    Caso {
        rotulo: "tint Linear",
        no: "motion.tint",
        params: &[
            ("mode", 1.0),
            ("g", 0.0),
            ("b", 0.0),
            ("b2", 1.0),
            ("space", 0.0),
        ],
        via_drive: false,
    },
    Caso {
        rotulo: "tint OKLab",
        no: "motion.tint",
        params: &[
            ("mode", 1.0),
            ("g", 0.0),
            ("b", 0.0),
            ("b2", 1.0),
            ("space", 1.0),
        ],
        via_drive: false,
    },
    Caso {
        rotulo: "tint OKLCH longo",
        no: "motion.tint",
        params: &[
            ("mode", 1.0),
            ("g", 0.0),
            ("b", 0.0),
            ("b2", 1.0),
            ("space", 3.0),
        ],
        via_drive: false,
    },
];

fn liga(m: &mut MotionState, de: NodeId, para: NodeId, porta: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, 0),
            to: (para, porta),
            delayed: false,
        })
        .expect("liga");
}

/// `grid lado² → oscillator → [caso] → output`; a fonte MEXE-SE (o memo da CPU não responde).
fn monta(m: &mut MotionState, lado: f32, caso: Option<&Caso>) -> NodeId {
    let grid = m.doc.graph.add_node("motion.grid".to_string());
    m.doc.graph.set_param(grid, "rows", lado);
    m.doc.graph.set_param(grid, "cols", lado);
    let osc = m.doc.graph.add_node("motion.oscillator".to_string());
    m.doc.graph.set_param(osc, "amplitude", 3.0);
    m.doc.graph.set_param(osc, "frequency", 0.7);
    liga(m, grid, osc, 0);
    let out = m.doc.graph.add_node("motion.output".to_string());
    let Some(c) = caso else {
        liga(m, osc, out, 0);
        return out;
    };
    let x = m.doc.graph.add_node(c.no.to_string());
    if c.no == "motion.stagger" {
        for &(k, v) in ONDA {
            m.doc.graph.set_param(x, k, v);
        }
    }
    for &(k, v) in c.params {
        m.doc.graph.set_param(x, k, v);
    }
    liga(m, osc, x, 0);
    if c.via_drive {
        let drive = m.doc.graph.add_node("motion.drive".to_string());
        m.doc.graph.set_param(drive, "channel", 1.0);
        liga(m, osc, drive, 0);
        liga(m, x, drive, 1);
        liga(m, drive, out, 0);
    } else {
        liga(m, x, out, 0);
    }
    out
}

/// `(linhas, disp min, disp mediana, CPU min, CPU mediana)` — `None` no dispositivo se o planeador
/// não reivindicou a cadeia inteira.
fn mede(
    gpu: &GpuContext,
    lado: f32,
    caso: Option<&Caso>,
) -> (usize, Option<(f64, f64)>, (f64, f64)) {
    let mut m = MotionState::new();
    let sink = monta(&mut m, lado, caso);
    let plano = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink);
    let mut gc = plano.is_fully_gpu().then(GpuCook::new);
    let mut cook = Cook::new();
    let (mut d, mut c, mut n) = (Vec::new(), Vec::new(), 0usize);
    for f in 0..AQUECE + AMOSTRAS {
        let t = f as f64 * DT;
        if let Some(gc) = gc.as_mut() {
            let t0 = std::time::Instant::now();
            gc.cook(
                gpu,
                &m.doc.graph,
                &m.registry,
                &m.registry,
                &plano,
                &[],
                CookClock {
                    playhead: t,
                    tick: Some(f),
                },
                UV,
                TAMANHO,
                SinkStyle::PLAIN,
            )
            .expect("o dispositivo coze");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            if f >= AQUECE {
                d.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
        }
        let t0 = std::time::Instant::now();
        let saida = cook
            .cook(&m.doc.graph, &m.registry, sink, t)
            .expect("a CPU coze");
        if f >= AQUECE {
            c.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        n = saida[0].as_stream().count();
        cook.advance_tick(&m.doc.graph, &m.registry, t)
            .expect("tique");
    }
    let resumo = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        (v[0], v[v.len() / 2])
    };
    (n, gc.is_some().then(|| resumo(d)), resumo(c))
}

#[test]
#[ignore = "sonda: medição — corra à mão, em RELEASE, com adaptador"]
fn measure_the_rhythm_group() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adaptador — esta sonda mede a placa");
    };
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32);
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!("  grid {lado}² -> oscillator -> X -> output · ms = minimo / mediana\n");
    eprintln!(
        "  {:<22} │ {:>8} │ {:>15} │ {:>15} │ {:>8}",
        "caso", "linhas", "disp ms", "CPU ms", "ns/linha"
    );
    let imprime = |r: &str, (n, disp, cpu): (usize, Option<(f64, f64)>, (f64, f64))| {
        let (ds, ns) = match disp {
            Some((a, b)) => (
                format!("{a:.3} / {b:.3}"),
                format!("{:.2}", a * 1e6 / n.max(1) as f64),
            ),
            None => ("⛔ CPU".to_string(), "—".to_string()),
        };
        eprintln!(
            "  {r:<22} │ {n:>8} │ {ds:>15} │ {:>15} │ {ns:>8}",
            format!("{:.3} / {:.3}", cpu.0, cpu.1)
        );
    };
    imprime("(a base, sem X)", mede(&gpu, lado, None));
    for c in GRUPO {
        let medida = mede(&gpu, lado, Some(c));
        assert!(medida.1.is_some(), "`{}` caiu para a CPU", c.rotulo);
        imprime(c.rotulo, medida);
    }
}
