//! **O CENSO DO BUG #14** — irmã por RESPONSABILIDADE (HR-18) das do [`super::census`]: aquelas
//! medem o que as cenas AUTORAM; esta mede o que uma FAMÍLIA de cenas (carimbo a jusante de uma
//! zona de simulação) entrega ao passe das formas, pela porta do produto (doc 122 §1).

use super::*;

/// ⭐ **O CENSO DO BUG #14** (doc 122 §1) — toda cena com um carimbo (`motion.duplicator`) cujo
/// cone a montante tem um `sim.zone`, cozida pela porta do produto nas duas rotas: quantas cópias
/// de forma o quadro entrega ao passe das formas, e o raio da MENOR em pixels da câmera do
/// produto (`48 px` por unidade). Sonda, não gate: imprime a tabela do doc.
#[test]
#[ignore = "sonda — `PH2D_GPU=1 … -- --ignored --nocapture`"]
fn census_stamps_downstream_of_a_zone() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        return;
    };
    let mut achadas = 0;
    for level in 1..=MAX_DEMO_LEVEL {
        let mut probe = MotionState::new();
        let _ =
            crate::motion_demo_legend::monta(&level.to_string(), &mut probe.doc, &probe.registry);
        let g = &probe.doc.graph;
        let tipo = |id| g.node(id).map(|n| n.type_name.as_str()).unwrap_or("");
        let carimbos: Vec<_> = g
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.duplicator")
            .filter(|n| {
                // O cone a montante, pelas arestas (as atrasadas incluídas).
                let mut visto = std::collections::BTreeSet::new();
                let mut fila = vec![n.id];
                while let Some(x) = fila.pop() {
                    for e in g.edges().iter().filter(|e| e.to.0 == x) {
                        if visto.insert(e.from.0) {
                            fila.push(e.from.0);
                        }
                    }
                }
                visto.iter().any(|&u| tipo(u) == "sim.zone")
            })
            .map(|n| n.id)
            .collect();
        if carimbos.is_empty() {
            continue;
        }
        achadas += 1;
        let mut linha = format!("={level:<4} carimbos {}", carimbos.len());
        for placa in [false, true] {
            let mut m = MotionState::new();
            m.gpu_enabled = placa;
            m.sinks =
                crate::motion_demo_legend::monta(&level.to_string(), &mut m.doc, &m.registry).0;
            let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
            let mut copias = Vec::new();
            let mut menor = f32::INFINITY;
            for q in [1u64, 30, 90] {
                playhead.seek(q as f64 / 60.0);
                crate::motion_bridge::quadro::coze_o_quadro(&mut m, &gpu, &playhead, 1.0 / 60.0);
                match m.gpu_cook.formas().filter(|_| m.gpu_live) {
                    Some(f) => {
                        let w = ph2d_gpu_cook::read_formas(&gpu, f);
                        copias.push(w.len() / 16);
                        for c in w.chunks(16) {
                            menor = menor.min(f32::from_bits(c[2]).min(f32::from_bits(c[3])));
                        }
                    }
                    None => {
                        copias.push(m.pump.vector_instances.len());
                        for v in &m.pump.vector_instances {
                            menor = menor.min(v.size[0].min(v.size[1]));
                        }
                    }
                }
            }
            linha += &format!(
                " | {} {copias:?} menor {:.1} px",
                if placa { "placa" } else { "CPU" },
                menor * 48.0
            );
        }
        eprintln!("{linha}");
    }
    eprintln!("{achadas} cenas com carimbo a jusante de uma zona");
}
