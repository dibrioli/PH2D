//! ⭐⭐⭐ **O PREÇO DA COSTURA DE UMA FONTE** (ciclo 8, passo 2 —
//! [doc 113](../../../docs/Motion%20Nodes/113_ciclo_8_fontes_e_dados.md) §3).
//!
//! ⚠️ **Uma fonte é o PRIMEIRO nó de um grafo.** No ciclo 7 a aparência era o último e puxava a
//! cadeia inteira para a CPU; aqui a costura cai NA fonte, e o resto fica na placa — então o preço
//! não é «a cadeia na CPU», é **o que atravessa por quadro**: cozer a fonte (o memo responde quando
//! ela não mudou) e ENVIAR o stream dela para a placa (a costura envia sempre —
//! `ph2d_gpu_cook::GpuCook::cook`, *«one upload per boundary node»*).
//!
//! ⇒ a sonda corre `fonte → scale → output` pela **ponte do produto** (`cook_gpu`, a mesma que o
//! quadro chama, com o `publish_all` das membranas antes), e põe ao lado a MESMA contagem numa
//! `motion.grid` (a cadeia inteira na placa) — o controlo que separa o preço da costura do preço
//! das linhas.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib --release -- --ignored --nocapture measure_the_source_seam
//! ```

use crate::motion_state::MotionState;
use ph2d_gpu::GpuContext;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::{Edge, NodeId};

const DT: f64 = 1.0 / 60.0;
const AQUECE: u64 = 20;
const AMOSTRAS: u64 = 15;

#[derive(Clone, Copy)]
enum Fonte {
    /// A cadeia inteira na placa — o controlo.
    Grade,
    /// `source.table` sobre um ficheiro de `n` linhas.
    Tabela,
}

fn liga(m: &mut MotionState, de: NodeId, para: NodeId) {
    m.doc
        .graph
        .connect(Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .expect("liga");
}

/// Um CSV de `n` linhas, escrito uma vez por tamanho.
fn tabela(n: usize) -> String {
    let dir = std::env::temp_dir().join("ph2d-fontes-costura");
    std::fs::create_dir_all(&dir).expect("pasta");
    let p = dir.join(format!("linhas_{n}.csv"));
    if !p.exists() {
        let mut s = String::with_capacity(n * 16 + 32);
        s.push_str("nome,valor,peso\n");
        for i in 0..n {
            s.push_str(&format!("r{i},{},{}\n", i % 97, (i % 13) as f32 * 0.5));
        }
        std::fs::write(&p, s).expect("escreve");
    }
    p.to_string_lossy().into_owned()
}

struct Linha {
    /// O que a MEMBRANA custa: publicar as fontes externas do quadro (`publish_all`).
    publicar: f64,
    /// O que o COZIMENTO custa: o prefixo na CPU, o envio da costura e os passes.
    cozer: f64,
    linhas: usize,
    rota: &'static str,
}

fn mede(gpu: &GpuContext, fonte: Fonte, n: usize) -> Linha {
    let mut m = MotionState::new();
    let head = match fonte {
        Fonte::Grade => {
            let g = m.doc.graph.add_node("motion.grid".to_string());
            #[expect(clippy::cast_precision_loss, reason = "lado de uma grade")]
            let lado = (n as f64).sqrt().round() as f32;
            m.doc.graph.set_param(g, "rows", lado);
            m.doc.graph.set_param(g, "cols", lado);
            g
        }
        Fonte::Tabela => {
            let t = m.doc.graph.add_node("source.table".to_string());
            m.doc
                .graph
                .set_text_param(t, ph2d_node_source_table::FILE_KEY, tabela(n));
            t
        }
    };
    let s = m.doc.graph.add_node("motion.scale".to_string());
    let o = m.doc.graph.add_node("motion.output".to_string());
    liga(&mut m, head, s);
    liga(&mut m, s, o);
    m.sinks = vec![o];
    let scopes = TimeScopes::new();
    let (mut pub_ms, mut cozer_ms) = (Vec::new(), Vec::new());
    for f in 0..AQUECE + AMOSTRAS {
        // ⚠️ **Os dois relógios separados, e é o achado que eles separam:** a membrana volta a
        // publicar a fonte a cada quadro (e o `set_external` HASHA o conteúdo para a revisão), e o
        // cozimento volta a ENVIAR a costura. São duas repetições diferentes de trabalho sobre a
        // mesma coisa parada, e um número só não diz qual.
        let t0 = std::time::Instant::now();
        crate::motion_externals::publish_all(&mut m, f as f64 * DT);
        let t1 = std::time::Instant::now();
        let _ = super::super::gpu::cook_gpu(&mut m, gpu, f, DT, &scopes);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        if f >= AQUECE {
            pub_ms.push((t1 - t0).as_secs_f64() * 1000.0);
            cozer_ms.push(t1.elapsed().as_secs_f64() * 1000.0);
        }
    }
    pub_ms.sort_by(f64::total_cmp);
    cozer_ms.sort_by(f64::total_cmp);
    Linha {
        publicar: pub_ms[pub_ms.len() / 2],
        cozer: cozer_ms[cozer_ms.len() / 2],
        linhas: m.gpu_cook.instances().map_or(0, |b| b.len() as usize),
        rota: m.route_said.unwrap_or("?"),
    }
}

#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE, com adaptador e com a máquina calma"]
fn measure_the_source_seam() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        panic!("sem adaptador — esta sonda mede a placa");
    };
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "\n  fonte    │         n │ linhas na placa │ publicar │ cozer+enviar │ quadro │ rota"
    );
    for n in [10_000usize, 100_000, 1_000_000] {
        for (rotulo, f) in [("grade", Fonte::Grade), ("tabela", Fonte::Tabela)] {
            let l = mede(&gpu, f, n);
            eprintln!(
                "  {rotulo:<8} │ {n:>9} │ {:>15} │ {:>8.2} │ {:>12.2} │ {:>6.2} │ {}",
                l.linhas,
                l.publicar,
                l.cozer,
                l.publicar + l.cozer,
                l.rota
            );
        }
    }
    eprintln!();
}
