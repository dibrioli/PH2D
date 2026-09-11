//! **A MEDIÇÃO DO GRUPO «ARRANJO»** — o passo 5 do ciclo 1
//! ([doc 104](../../../../docs/Motion%20Nodes/104_ciclo_1_arranjo.md)), e a lei §2.1 da
//! dinâmica: *todo nó de um ciclo diz onde corre e porquê*.
//!
//! Para cada um dos dez nós que põem objectos na tela: quantos elementos emite, se a cadeia
//! `nó → output` fica inteira no **device**, quantos passes despacha, e o que custa na CPU.
//!
//! `cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_arranjo_group`

use crate::motion::motion_state::MotionState;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::Edge;

/// Os dez nós do ciclo, com o param de CONTAGEM de cada um (o que se sobe para medir a escala)
/// e o valor a que se mede. `None` = a contagem não é um param (vem da geometria).
const GRUPO: [(&str, Option<(&str, f32)>); 10] = [
    ("motion.grid", Some(("rows", 100.0))),
    ("motion.scatter", Some(("count", 10_000.0))),
    ("motion.distribute_radial", Some(("count", 10_000.0))),
    ("motion.fibonacci", Some(("count", 10_000.0))),
    ("motion.lattice", Some(("rows", 100.0))),
    ("motion.voronoi", Some(("count", 2_000.0))),
    ("motion.distribute_poisson", None),
    ("motion.distribute_curve", Some(("count", 10_000.0))),
    ("motion.path", Some(("count", 10_000.0))),
    ("motion.clone", Some(("count", 1_000.0))),
];

#[test]
#[ignore = "medicao"]
fn measure_the_arranjo_group() {
    eprintln!(
        "  load: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    eprintln!(
        "  {:<26} │ {:>9} │ {:>9} │ {:>6} │ {:>9} │ recusa",
        "nó", "elementos", "device", "passes", "CPU ms"
    );
    for (nome, contagem) in GRUPO {
        let mut m = MotionState::new();
        let src = m.doc.graph.add_node(nome.to_string());
        if let Some((p, v)) = contagem {
            m.doc.graph.set_param(src, p, v);
            // Uma grelha conta linhas × colunas: subir só as linhas mede outra coisa.
            if p == "rows" {
                m.doc.graph.set_param(src, "cols", v);
            }
        }
        let out = m.doc.graph.add_node("motion.output".to_string());
        // ⚠️ **O `clone` e o `path` MULTIPLICAM uma entrada** — sem ela emitem zero, e medir
        // zero mede o caminho vazio. Alimenta-se-lhes uma grelha pequena, que é o que um
        // artista põe lá.
        use ph2d_nodegraph::cook::OpResolver;
        let leva_entrada = m
            .registry
            .resolve(m.doc.graph.node(src).expect("no'").type_id())
            .is_some_and(|op| !op.manifest().inputs.is_empty());
        if leva_entrada {
            let feed = m.doc.graph.add_node("motion.grid".to_string());
            m.doc.graph.set_param(feed, "rows", 10.0);
            m.doc.graph.set_param(feed, "cols", 10.0);
            let _ = m.doc.graph.connect(Edge {
                from: (feed, 0),
                to: (src, 0),
                delayed: false,
            });
        }
        let _ = m.doc.graph.connect(Edge {
            from: (src, 0),
            to: (out, 0),
            delayed: false,
        });
        let plano = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, out);
        // ⛔⛔ **UM `Cook` NOVO POR CORRIDA.** A 1.ª versão desta sonda cozinhava duas vezes no
        // mesmo `Cook` e cronometrava a segunda: estes nós são `Effect::Pure`, o memo responde,
        // e as dez linhas leram **0,00 ms**. *A régua media o memo, que é exactamente o que o
        // produto quer que ela não meça.*
        let mut n = 0usize;
        let mut melhor = f64::MAX;
        for _ in 0..5 {
            let mut cook = Cook::new();
            let t = std::time::Instant::now();
            let r = cook.cook(&m.doc.graph, &m.registry, out, 0.5);
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1000.0);
            n = r.map_or(0, |v| v.first().map_or(0, |c| c.as_stream().count()));
        }
        let ms = melhor;
        let recusa = if plano.is_fully_gpu() {
            String::new()
        } else {
            format!("{} fronteira(s)", plano.boundaries.len())
        };
        eprintln!(
            "  {nome:<26} │ {n:>9} │ {:>9} │ {:>6} │ {ms:>9.2} │ {recusa}",
            if plano.is_fully_gpu() { "SIM" } else { "nao" },
            plano.dispatching_stages(&m.registry),
        );
    }
    eprintln!(
        "  (device = a cadeia `no' -> output` fica inteira na placa; 4,19 M em 3,85 ms e' o tecto medido)"
    );
}
