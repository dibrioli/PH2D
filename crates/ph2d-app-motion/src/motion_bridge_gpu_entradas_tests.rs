//! ⭐ **As entradas publicadas pela rota da PLACA** (ciclo 19, doc 128 §3.3): a fita também vale
//! quando a simulação corre no dispositivo — o valor de cada tique chega ao uniform do param
//! dirigido, e voltar a fita repete-o (a rota `FullyGpu` rebobina no dispositivo e coze o condutor
//! de cada tique na CPU, com as entradas da fita).

use crate::motion_state::MotionState;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_nodegraph::graph::{Edge, NodeId};

/// `grid 3×3 → Integrate ⇄ Vortex`, com a força do `Vortex` dirigida pela entrada `forca`.
fn cena() -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    m.doc = ph2d_motion_doc::MotionDoc::new();
    let g = &mut m.doc.graph;
    let grid = g.add_node("motion.grid".to_string());
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 3.0);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    let ig = g.add_node("motion.integrate".to_string());
    let out = g.add_node("motion.output".to_string());
    let vx = g.add_node("force.vortex".to_string());
    g.set_param(vx, "radius", 6.0);
    for (a, b, d) in [
        ((grid, 0), (ig, 0), false),
        ((ig, 0), (vx, 0), true),
        ((vx, 0), (ig, 1), false),
        ((ig, 0), (out, 0), false),
    ] {
        g.connect(Edge {
            from: a,
            to: b,
            delayed: d,
        })
        .expect("liga");
    }
    let forca = g.add_node("value.input".to_string());
    g.set_text_param(forca, "name", "forca");
    g.drive_param(vx, "strength", (forca, 0)).expect("conduz");
    m.doc.graph.validate(&m.registry).expect("valida");
    m.sinks = vec![out];
    (m, ig)
}

#[test]
fn the_device_route_replays_the_published_inputs_on_a_scrub() {
    let Some(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    else {
        eprintln!("sem adapter: o gate da placa nao corre aqui");
        return;
    };
    let (mut m, ig) = cena();
    m.gpu_cook.retain_streams_for_debug(true);
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let mut quadro = |m: &mut MotionState, t: u64| -> Vec<[f32; 2]> {
        crate::motion_entradas::publica(m);
        let o = super::super::gpu::cook_gpu(m, &gpu, t, 1.0 / 60.0, &scopes);
        assert!(
            matches!(o, super::super::gpu::GpuOutcome::Handled) && m.gpu_live,
            "a cena tem de correr na placa: {:?}",
            m.route_said
        );
        m.gpu_cook
            .read_column_vec2(&gpu, ig, "P")
            .expect("le a placa")
    };
    let mut visto = Vec::new();
    for t in 0..=90u64 {
        let f = if t < 40 { 0.0 } else { 20.0 };
        m.entradas
            .jogo
            .set("forca", Escritor::Script, Valor::Numero(f));
        visto.push(quadro(&mut m, t));
    }
    assert_ne!(visto[30], visto[80], "CONTROLO: a força mexeu a simulação");
    // O jogo continua a publicar o valor de AGORA; com o relógio PARADO, arrastar a fita (para
    // trás e para a frente) repete o da época.
    m.pump.define_a_tocar(false);
    for alvo in [60u64, 35, 75, 45] {
        let p = quadro(&mut m, alvo);
        let pior = p
            .iter()
            .zip(&visto[alvo as usize])
            .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
            .fold(0.0f32, f32::max);
        assert!(pior < 1e-5, "scrub para {alvo} na placa: diverge {pior}");
    }
}
