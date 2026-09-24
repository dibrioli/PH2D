//! ⭐⭐⭐ **A tomada do gizmo das posições NÃO recozinha na CPU o que a placa já desenhou** (ciclo
//! 12, [doc 120 §8.5](../../../docs/Motion%20Nodes/120_ciclo_12_os_tectos_confortaveis.md)).
//!
//! Com o carimbo na placa, o Motion no app **não ficava mais barato**: o
//! [`crate::ponto_gizmo::taps_for`] pedia TODOS os sinks, e numa rota de dispositivo uma tomada
//! obriga a CPU a cozinhar a cadeia inteira até ao sink — na escada dos tectos, a simulação toda,
//! `2,4 ms` por quadro na RTX a `32 768` partículas, para um gizmo que **não é desenhado** (o sink
//! tem aparência, logo quem se vê são as peças).
//!
//! Os gates vivem na ponte porque a metade que importa pede um quadro conduzido pela PLACA, e a
//! porta que o conduz ([`super::gpu::cook_gpu`]) é da ponte.

use crate::motion_state::MotionState;
use crate::ponto_gizmo::{a_placa_desenhou, taps_filtrados, taps_for};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

fn cols(nomes: &[&str]) -> Vec<String> {
    nomes.iter().map(|s| (*s).to_owned()).collect()
}

/// **A pergunta é o VEREDITO DO DISPOSITIVO, coluna a coluna** — o ladrilho e a geometria viva
/// desenham, uma corrente só de posições não, e um sink que a placa não encenou pede a tomada.
#[test]
fn a_placa_desenhou_e_o_veredito_do_dispositivo() {
    assert!(
        !a_placa_desenhou(None),
        "um sink que a placa nao encenou tem de pedir a tomada, como sempre pediu"
    );
    assert!(
        !a_placa_desenhou(Some(&cols(&["P", "Index", "Count"]))),
        "uma corrente so de posicoes nao e' desenhada pela placa — o gizmo precisa dela"
    );
    assert!(
        a_placa_desenhou(Some(&cols(&["P", "uv_rect", "size"]))),
        "o ladrilho de uma `source.object` desenha — o gizmo cala-se"
    );
    assert!(
        a_placa_desenhou(Some(&cols(&["P", "geometry_id"]))),
        "a placa le a geometria pela EXISTENCIA da coluna (a divergencia nomeada do veredito) — \
         e o gizmo tem de dizer o mesmo que os pixeis dela"
    );
}

/// **A lei, pura:** o sink que a placa desenhou sai da lista, os outros ficam, e com a lei do
/// dono desligada não se pede nada. ⚠️ O CONTROLO é o predicado que responde sempre NÃO — é o
/// comportamento de antes do ciclo 12, e ele tem de devolver TODOS os sinks.
#[test]
fn a_tomada_so_e_pedida_onde_a_placa_nao_desenhou() {
    let sinks = [NodeId(3), NodeId(7), NodeId(9)];
    assert_eq!(
        taps_filtrados(&sinks, true, |s| s == NodeId(7)),
        vec![NodeId(3), NodeId(9)],
        "o sink que a placa desenhou nao e' recozinhado na CPU"
    );
    assert_eq!(
        taps_filtrados(&sinks, true, |_| false),
        sinks.to_vec(),
        "CONTROLO: sem nada desenhado pela placa, pedem-se todos"
    );
    assert!(
        taps_filtrados(&sinks, false, |_| false).is_empty(),
        "com a lei do dono desligada o gizmo nao e' desenhado — nao cobra cozimento"
    );
}

fn liga(g: &mut Graph, a: NodeId, porta: u16, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, porta),
        delayed: false,
    })
    .expect("ligar");
}

/// Conduz dois quadros na placa; `None` sem adaptador.
fn na_placa(motion: &mut MotionState) -> Option<ph2d_gpu::GpuContext> {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("sem adaptador de GPU — a saltar");
        return None;
    };
    motion.gpu_enabled = true;
    let scopes = Default::default();
    for tick in 0..2 {
        assert!(
            matches!(
                super::gpu::cook_gpu(motion, &gpu, tick, 1.0 / 60.0, &scopes),
                super::gpu::GpuOutcome::Handled
            ),
            "o documento tem de cozer na PLACA, senao este gate mede a rota da CPU"
        );
    }
    assert!(motion.gpu_live, "a placa conduziu o quadro");
    Some(gpu)
}

/// ⭐⭐⭐ **A costura inteira, com a placa real:** um carimbo de um objecto do atlas sobre uma
/// grelha, cozido no dispositivo, deixa o sink FORA das tomadas — e o CONTROLO, a mesma grelha
/// sem aparência nenhuma, deixa-o DENTRO (o gizmo precisa das posições dela).
///
/// ⚠️ Sem a primeira metade, a mutação que devolve `motion.sinks.clone()` no `taps_for` ficava
/// verde em todos os gates puros acima — eles não sabem se o produto os chama.
#[test]
#[ignore = "precisa de um adaptador de GPU — corra com --ignored e PH2D_GPU=1"]
fn a_placa_que_desenhou_o_sink_nao_pede_a_tomada() {
    // O que a placa desenha: `source.object` → carimbo ← grelha.
    let mut m = MotionState::new();
    m.pump.cook.set_external(
        "Particle",
        Stream::new(1)
            .with("texture_id", Column::Scalar(vec![0.0]))
            .with("uv_rect", Column::Vec4(vec![[0.0, 0.0, 1.0, 1.0]]))
            .with("size", Column::Vec2(vec![[0.08, 0.08]])),
    );
    let g = &mut m.doc.graph;
    let obj = g.add_node("source.object");
    g.set_text_param(obj, "object", "Particle");
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    liga(g, obj, 0, dup);
    liga(g, grid, 1, dup);
    liga(g, dup, 0, out);
    for (i, n) in [obj, grid, dup, out].into_iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "quatro nos")]
        g.set_pos(
            n,
            Pos {
                x: i as f32 * 200.0,
                y: 0.0,
            },
        );
    }
    m.doc.graph.validate(&m.registry).expect("bem tipado");
    m.sinks = vec![out];
    let Some(_gpu) = na_placa(&mut m) else {
        return;
    };
    assert!(
        a_placa_desenhou(m.gpu_cook.shape().columns(out)),
        "o sink carimbado tem de chegar da placa com o ladrilho — colunas: {:?}",
        m.gpu_cook.shape().columns(out)
    );
    assert!(
        !taps_for(&m, true).contains(&out),
        "a placa desenhou este sink: pedir a tomada recozinha a cadeia inteira na CPU"
    );

    // O CONTROLO: uma grelha sem aparência nenhuma — a placa não a desenha e o gizmo precisa dela.
    let mut c = MotionState::new();
    let g = &mut c.doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    let out = g.add_node("motion.output");
    liga(g, grid, 0, out);
    g.set_pos(grid, Pos { x: 0.0, y: 0.0 });
    g.set_pos(out, Pos { x: 200.0, y: 0.0 });
    c.sinks = vec![out];
    let Some(_gpu) = na_placa(&mut c) else {
        return;
    };
    assert!(
        taps_for(&c, true).contains(&out),
        "CONTROLO: uma corrente so de posicoes continua a pedir a tomada"
    );
}
