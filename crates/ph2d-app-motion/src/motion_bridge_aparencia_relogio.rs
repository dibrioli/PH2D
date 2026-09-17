//! ⭐⭐⭐ **O RELÓGIO DO GRUPO DA APARÊNCIA** (ciclo 7, passo 5 —
//! [doc 112](../../../docs/Motion%20Nodes/112_ciclo_7_aparencia.md) §4-septies).
//!
//! ⚠️ **Os ciclos 5 e 6 mediram só a CPU de referência** (`motion_ciclo_preco::tabela`) e
//! escreveram por extenso que o relógio do DISPOSITIVO não tinha sonda nenhuma. Este grupo tem os
//! dez nós no dispositivo desde a W1d, então a pergunta que interessa é a da placa: esta sonda coze
//! a MESMA cadeia pelos dois motores, quadro a quadro, e imprime os dois relógios lado a lado.
//!
//! ⚠️⚠️ **Três coisas que uma tabela ingénua deste grupo mediria errado, e a sonda evita:**
//!
//! - **O ESTADO.** Três nós carregam memória por uma aresta `pre` (o rasto, o estroboscópio, o
//!   slit-scan), e o primeiro tique deles é a identidade por desenho — a frio mede-se um rasto sem
//!   cauda. ⇒ a cadeia passa pela **canalização do produto** (`plumbing::reconcile_after`, a mesma
//!   porta que o editor corre ao largar o nó; ⛔ nunca uma segunda cópia da regra) e cronometra-se
//!   o **regime**, depois de [`AQUECE`] tiques (a cauda mais longa que o despertar pede enche-se
//!   em `length × spacing` tiques).
//! - **O NEUTRO.** Metade do grupo nasce na identidade — cronometra-se o nó **acordado**
//!   ([`crate::motion_ciclo_preco::acordar`], a porta dos ciclos anteriores).
//! - **O PULSO.** O `motion.strobe` sem pulso nunca pisca; uma porta chamada `pulse` recebe um
//!   `pulse.beat` — lida do MANIFESTO, não do nome do nó.
//!
//! ⭐ **E a sonda confere-se a si mesma:** as linhas que a CPU e o dispositivo emitem têm de ser o
//! MESMO número (a paridade das colunas vive nos gates `gpu_cpu_parity_*`; aqui só a contagem, que
//! é o que um relógio sobre um stream diferente esconderia).
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture measure_the_fx_group_on_both_engines
//! ```
//! (`PH2D_LADO=1000` para o milhão; imprime o `/proc/loadavg` na primeira linha — CLAUDE.md §5.0.)

use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_render::SinkStyle;

const DT: f64 = 1.0 / 60.0;
/// Tiques antes de cronometrar. ⚠️ **A coluna das LINHAS é a prova de que chega:** acordado, o
/// rasto emite `n × 9` (a cabeça e oito ecos — medido a `24²`: `576 → 5 184`), e uma cauda a meio
/// de encher leria menos. Quem mudar os hints do rasto relê essa coluna antes de citar o relógio.
const AQUECE: u64 = 120;
/// Quadros cronometrados depois do aquecimento; cita-se a MEDIANA.
const AMOSTRAS: u64 = 9;
const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const TAMANHO: [f32; 2] = [0.4, 0.4];

fn liga(m: &mut MotionState, de: NodeId, dp: u16, para: NodeId, pp: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, dp),
            to: (para, pp),
            delayed: false,
        })
        .expect("as portas encaixam");
}

/// `grid lado² → oscillator → [nó] → output`. A fonte MEXE-SE (senão o memo da CPU responde e a
/// régua mede a tabela de hash — ciclo 1).
fn monta(m: &mut MotionState, lado: f32, no: Option<&str>) -> NodeId {
    let antes = m.doc.graph.clone();
    let grid = m.doc.graph.add_node("motion.grid".to_string());
    m.doc.graph.set_param(grid, "rows", lado);
    m.doc.graph.set_param(grid, "cols", lado);
    let osc = m.doc.graph.add_node("motion.oscillator".to_string());
    m.doc.graph.set_param(osc, "amplitude", 3.0);
    m.doc.graph.set_param(osc, "frequency", 0.7);
    liga(m, grid, 0, osc, 0);
    let out = m.doc.graph.add_node("motion.output".to_string());
    let Some(nome) = no else {
        liga(m, osc, 0, out, 0);
        return out;
    };
    let x = m.doc.graph.add_node(nome.to_string());
    crate::motion_ciclo_preco::acordar(m, x, nome);
    liga(m, osc, 0, x, 0);
    let porta_pulso = m
        .registry
        .resolve(NodeTypeId::of(nome))
        .and_then(|op| op.manifest().inputs.iter().position(|p| p.name == "pulse"));
    if let Some(p) = porta_pulso {
        let beat = m.doc.graph.add_node("pulse.beat".to_string());
        liga(m, osc, 0, beat, 0);
        liga(m, beat, 0, x, u16::try_from(p).expect("porta"));
    }
    liga(m, x, 0, out, 0);
    // ⭐ A canalização do PRODUTO: o `pre` do `state` que o editor arma ao largar o nó.
    crate::motion_bridge::plumbing::reconcile_after(&mut m.doc.graph, &m.registry, &antes);
    out
}

struct Linha {
    /// As linhas que a CPU emitiu no último quadro.
    cpu_linhas: usize,
    /// As que o dispositivo emitiu — `None` se o planeador não reivindicou a cadeia.
    disp_linhas: Option<usize>,
    passes: usize,
    cpu_ms: f64,
    disp_ms: Option<f64>,
    tem_pre: bool,
}

fn mediana(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn mede(gpu: &GpuContext, lado: f32, no: Option<&str>) -> Linha {
    let mut m = MotionState::new();
    let sink = monta(&mut m, lado, no);
    let tem_pre = m.doc.graph.edges().iter().any(|e| e.delayed);
    let plano = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink);
    let passes = plano.dispatching_stages(&m.registry);
    let (mut disp_ms, mut disp_linhas) = (None, None);
    if plano.is_fully_gpu() {
        let mut gc = GpuCook::new();
        let mut ms = Vec::new();
        for f in 0..AQUECE + AMOSTRAS {
            let t0 = std::time::Instant::now();
            gc.cook(
                gpu,
                &m.doc.graph,
                &m.registry,
                &m.registry,
                &plano,
                &[],
                CookClock {
                    playhead: f as f64 * DT,
                    tick: Some(f),
                },
                UV,
                TAMANHO,
                SinkStyle::PLAIN,
            )
            .expect("o dispositivo coze a cadeia que o planeador reivindicou");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            if f >= AQUECE {
                ms.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
            disp_linhas = Some(gc.instances().map_or(0, |b| b.len() as usize));
        }
        disp_ms = Some(mediana(ms));
    }
    let mut cook = Cook::new();
    let (mut ms, mut cpu_linhas) = (Vec::new(), 0usize);
    for f in 0..AQUECE + AMOSTRAS {
        let t = f as f64 * DT;
        let t0 = std::time::Instant::now();
        let saida = cook
            .cook(&m.doc.graph, &m.registry, sink, t)
            .expect("a CPU coze a cadeia");
        if f >= AQUECE {
            ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        }
        cpu_linhas = saida[0].as_stream().count();
        cook.advance_tick(&m.doc.graph, &m.registry, t)
            .expect("tique");
    }
    Linha {
        cpu_linhas,
        disp_linhas,
        passes,
        cpu_ms: mediana(ms),
        disp_ms,
        tem_pre,
    }
}

#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE, com adaptador e com a máquina calma"]
fn measure_the_fx_group_on_both_engines() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adaptador — esta sonda mede a placa e nao tem versao de CPU");
    };
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let base = mede(&gpu, lado, None);
    eprintln!(
        "  grid {lado}² -> oscillator -> X -> output · {} objectos · regime = mediana de {AMOSTRAS} \
         quadros depois de {AQUECE} tiques",
        base.cpu_linhas
    );
    eprintln!(
        "\n  {:<20} │ {:>10} │ {:>6} │ {:>8} │ {:>8} │ {:>6} │ {:>9}",
        "nó", "linhas", "passes", "disp ms", "CPU ms", "CPU/d", "ns/linha"
    );
    eprintln!(
        "  ---------------------|------------|--------|----------|----------|--------|----------"
    );
    let imprime = |rotulo: &str, l: &Linha| {
        let linhas = match l.disp_linhas {
            Some(d) if d == l.cpu_linhas => format!("{d}"),
            Some(d) => format!("⚠️{}≠{d}", l.cpu_linhas),
            None => format!("{}", l.cpu_linhas),
        };
        let (disp, razao, ns) = match l.disp_ms {
            Some(d) => (
                format!("{d:.2}"),
                format!("{:.1}×", l.cpu_ms / d.max(1e-9)),
                format!("{:.2}", d * 1e6 / l.disp_linhas.unwrap_or(1).max(1) as f64),
            ),
            None => ("⛔ CPU".to_string(), "—".to_string(), "—".to_string()),
        };
        eprintln!(
            "  {rotulo:<20} │ {linhas:>10} │ {:>6} │ {disp:>8} │ {:>8.2} │ {razao:>6} │ {ns:>9}{}",
            l.passes,
            l.cpu_ms,
            if l.tem_pre { "  (pre)" } else { "" }
        );
    };
    imprime("(sem X — a base)", &base);
    let grupo = crate::motion_aparencia_probe::grupo();
    assert!(grupo.len() >= 10, "piso de populacao: {grupo:?}");
    for no in grupo {
        let l = mede(&gpu, lado, Some(no));
        assert!(l.cpu_linhas > 0, "`{no}` nao emitiu linha nenhuma");
        imprime(no, &l);
    }
    eprintln!(
        "\n  (linhas = as que a CPU e o dispositivo emitiram no ultimo quadro — um ⚠️ e' um \
         desacordo de CONTAGEM entre os dois motores.\n   passes = estagios que despacham um \
         passe (sem os passa-tudo). (pre) = a cadeia tem a realimentacao do produto.\n   Um \
         quadro de 60 fps tem 16,67 ms.)\n"
    );
}
