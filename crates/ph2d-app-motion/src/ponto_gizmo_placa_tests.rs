//! ⛔ **Bug #20: num grafo sem formas cozido pela PLACA os marcadores ficavam PARADOS.**
//!
//! As tomadas dos gizmos (`pump.tap_streams()`, a porta que os quatro leem: pontos, warp, colisor,
//! pivô) vinham do `cook_taps_only` — a CPU a cozer o sink sobre um estado que a bomba nunca
//! marchou. Sem estado (grelha → oscilador) dava certo; com estado (um `motion.trail`) o rasto não
//! tinha ecos e os marcadores ficavam na pose de repouso. Medido antes da cura: `72` pontos na CPU,
//! `9` na placa. Hoje elas são uma LEITURA da placa, um quadro atrás.
//!
//! Grafos SINTÉTICOS de propósito: as cenas de ciclo sem formas ganham formas (a `=118` ganhou).

use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};

fn liga(m: &mut MotionState, de: NodeId, para: NodeId, porta: u16, pre: bool) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, 0),
            to: (para, porta),
            delayed: pre,
        })
        .expect("liga");
}

/// grelha `lado × lado` → oscilador → (rasto com `pre`) → saída.
fn monta(m: &mut MotionState, lado: f32, com_rasto: bool) -> Vec<NodeId> {
    let g = m.doc.graph.add_node("motion.grid");
    m.doc.graph.set_param(g, "rows", lado);
    m.doc.graph.set_param(g, "cols", lado);
    let o = m.doc.graph.add_node("motion.oscillator");
    m.doc.graph.set_param(o, "amplitude", 40.0);
    m.doc.graph.set_param(o, "frequency", 0.5);
    liga(m, g, o, 0, false);
    let ultimo = if com_rasto {
        let t = m.doc.graph.add_node("motion.trail");
        liga(m, o, t, 0, false);
        liga(m, t, t, 1, true);
        t
    } else {
        o
    };
    let s = m.doc.graph.add_node("motion.output");
    liga(m, ultimo, s, 0, false);
    vec![s]
}

fn novo(lado: f32, com_rasto: bool, lei: bool) -> MotionState {
    let mut m = MotionState::new();
    m.pump.define_a_lei(lei);
    m.sinks = monta(&mut m, lado, com_rasto);
    m
}

fn quadro(m: &mut MotionState, placa: Option<&ph2d_gpu::GpuContext>, ph: &ph2d_core::Playhead) {
    crate::motion_bridge::quadro::o_quadro(
        m,
        placa,
        ph,
        1.0 / 60.0,
        crate::motion_bridge::quadro::MaoDoEditor::default(),
    );
}

/// O que cada quadro deu: a rota, os marcadores, e o `P` de cada tomada (a porta dos quatro gizmos).
struct Quadro {
    placa: bool,
    pontos: Vec<[f32; 2]>,
    tomadas: Vec<(NodeId, Vec<[f32; 2]>)>,
}

fn corre(placa: Option<&ph2d_gpu::GpuContext>, com_rasto: bool, quadros: u32) -> Vec<Quadro> {
    let mut m = novo(3.0, com_rasto, true);
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    ph.play();
    let mut out = Vec::new();
    for _ in 0..quadros {
        ph.advance_ticks(1);
        quadro(&mut m, placa, &ph);
        // O app apresentou o quadro: a placa acabou-o (a leitura sem espera recolhe-o no seguinte).
        if let Some(gpu) = placa {
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        }
        let pontos = super::resolve(&m, true, m.pump.a_lei())
            .map(|v| v.grupos.iter().flat_map(|g| g.pontos.clone()).collect())
            .unwrap_or_default();
        let tomadas = m
            .pump
            .tap_streams()
            .iter()
            .map(|(n, s)| (*n, super::posicoes_amostradas(s)))
            .collect();
        out.push(Quadro {
            placa: m.gpu_live,
            pontos,
            tomadas,
        });
    }
    out
}

fn dif(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return f32::INFINITY;
    }
    a.iter()
        .zip(b)
        .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
        .fold(0.0, f32::max)
}

/// ⭐ **Pela placa os marcadores ANDAM e são os da CPU, no máximo um quadro atrás** — com e sem estado, e a
/// porta das tomadas (que o warp, o colisor e o pivô leem) igual à da CPU nó a nó.
/// PISO: o quadro é da placa, e na CPU os marcadores andam (senão «igual» seria vácuo). CONTROLO:
/// o rasto tem ecos na CPU (mais pontos que sem rasto) — é o estado que a cura tem de trazer.
#[test]
#[ignore = "placa: precisa de GPU"]
fn os_marcadores_andam_na_placa_como_na_cpu() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter — este gate mede a rota do device");
    const N: u32 = 40;
    let mut pontos_por_caso = Vec::new();
    for com_rasto in [false, true] {
        let cpu = corre(None, com_rasto, N);
        let pla = corre(Some(&gpu), com_rasto, N);
        let caso = if com_rasto { "com rasto" } else { "sem estado" };
        assert!(
            dif(&cpu[10].pontos, &cpu[30].pontos) > 1.0,
            "PISO {caso}: na CPU os marcadores andam"
        );
        pontos_por_caso.push(cpu[30].pontos.len());
        for q in 2..N as usize {
            assert!(pla[q].placa, "PISO {caso}: o quadro {q} é da placa");
            // A leitura da placa vem um quadro atrás (`k = 1`); `k = 0` é o mesmo quadro.
            let igual = |k: usize| {
                let c = &cpu[q - k];
                dif(&pla[q].pontos, &c.pontos) < 1e-2
                    && pla[q].tomadas.len() == c.tomadas.len()
                    && pla[q]
                        .tomadas
                        .iter()
                        .zip(&c.tomadas)
                        .all(|((n, p), (m, x))| n == m && dif(p, x) < 1e-2)
            };
            assert!(
                igual(1) || igual(0),
                "{caso}, quadro {q}: os marcadores da placa ({} pontos) não são os da CPU \
                 ({} pontos), nem um quadro atrás",
                pla[q].pontos.len(),
                cpu[q - 1].pontos.len()
            );
        }
    }
    assert!(
        pontos_por_caso[1] > pontos_por_caso[0],
        "CONTROLO: o rasto tem ecos na CPU ({pontos_por_caso:?})"
    );
}

/// **O PREÇO da leitura** (MEDIR_VELOCIDADE.md): a parede de CPU de um quadro, três variantes no
/// MESMO processo, intercaladas por blocos rodados, resumo = o MÍNIMO (mediana ao lado):
/// `leitura` (a cura: os marcadores lidos da placa) · `sem gizmo` (a lei desligada, nada pedido) ·
/// `cpu antiga` (sem gizmo + o `cook_taps_only` do sink, o que a rota pagava antes).
/// Correr com `--profile smoke`.
#[test]
#[ignore = "sonda de placa: mede, não decide"]
fn sonda_o_preco_da_leitura_dos_marcadores() {
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("adapter");
    let lado = 64.0; // 4 096 por eco, 8 ecos: 32 768 linhas no sink
    let mut v = [
        novo(lado, true, true),
        novo(lado, true, false),
        novo(lado, true, false),
    ];
    let nomes = ["leitura", "sem gizmo", "cpu antiga"];
    let mut ph = [(); 3].map(|()| {
        let mut p = ph2d_core::Playhead::new(1.0 / 60.0);
        p.play();
        p
    });
    let mut tempos: [Vec<f64>; 3] = Default::default();
    let um = |k: usize, m: &mut MotionState, ph: &mut ph2d_core::Playhead| -> f64 {
        ph.advance_ticks(1);
        let t0 = std::time::Instant::now();
        quadro(m, Some(&gpu), ph);
        if k == 2 {
            let sinks = m.sinks.clone();
            m.pump.set_taps(&sinks);
            let (g, r) = (&m.doc.graph, &m.registry);
            m.pump
                .cook_taps_only(g, r, ph.time(), &ph2d_nodegraph::cook::TimeScopes::new());
        }
        let dt = t0.elapsed().as_secs_f64() * 1e3;
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        dt
    };
    for k in 0..3 {
        for _ in 0..30 {
            um(k, &mut v[k], &mut ph[k]);
        }
    }
    eprintln!("loadavg {:?}", std::fs::read_to_string("/proc/loadavg"));
    for r in 0..9 {
        for j in 0..3 {
            let k = (r + j) % 3;
            let mut bloco = Vec::new();
            for _ in 0..20 {
                bloco.push(um(k, &mut v[k], &mut ph[k]));
            }
            bloco.sort_by(f64::total_cmp);
            tempos[k].push(bloco[bloco.len() / 2]);
        }
    }
    let linhas: usize = v[0].pump.tap_streams().iter().map(|(_, s)| s.count()).sum();
    for k in 0..3 {
        let mut t = tempos[k].clone();
        t.sort_by(f64::total_cmp);
        eprintln!(
            "PRECO {}: min {:.3} ms · mediana {:.3} ms (por quadro, mediana de cada bloco)",
            nomes[k],
            t[0],
            t[t.len() / 2]
        );
    }
    eprintln!(
        "PRECO linhas lidas por quadro: {linhas} (sink {:?})",
        v[0].gpu_cook.node_count(v[0].sinks[0])
    );
    eprintln!("loadavg {:?}", std::fs::read_to_string("/proc/loadavg"));
    assert!(v[0].gpu_live && v[1].gpu_live && v[2].gpu_live);
}
