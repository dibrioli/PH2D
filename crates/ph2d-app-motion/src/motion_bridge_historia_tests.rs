//! ⭐ **A fita dita ao artista, pelo caminho do produto** (doc 132 §2): o quadro do produto
//! (`coze_o_quadro`) a tocar com um tecto pequeno publica à Timeline a faixa `[base, último]`; com o
//! relógio parado, arrastar o playhead para antes da base faz o cartão do `value.input` dizê-lo
//! (`readout::stamp`), e para dentro da faixa, não. Nas DUAS rotas: a CPU e a placa.

use super::*;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_eval_motion::Razao;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DT: f64 = 1.0 / 60.0;

/// `grid → output`, e um cartão `Input` chamado `e0` (uma das oito entradas que o jogo publica).
fn cena() -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (grid, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("liga");
    let e0 = g.add_node("value.input");
    g.set_text_param(e0, ph2d_node_value_input::NAME_KEY, "e0");
    m.doc.graph = g;
    m.sinks = vec![out];
    m.pump.define_o_razao_da_fita(Razao::new(16 << 10));
    (m, e0)
}

/// O quadro do PRODUTO no segundo do `playhead`, e o que o cartão de `no` diz a seguir.
fn quadro(
    m: &mut MotionState,
    gpu: &ph2d_gpu::GpuContext,
    playhead: &ph2d_core::Playhead,
    no: NodeId,
) -> Option<String> {
    quadro::coze_o_quadro(m, gpu, playhead, DT);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    readout::stamp(m, None, &mut snap);
    snap.nodes.iter().find(|n| n.id == no.0)?.readout.clone()
}

#[test]
fn o_cartao_das_entradas_avisa_ao_arrastar_para_antes_da_base() {
    let Some(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()
    else {
        eprintln!("sem adapter: o gate do quadro do produto nao corre aqui");
        return;
    };
    for placa in [false, true] {
        let (mut m, e0) = cena();
        m.gpu_enabled = placa;
        let mut playhead = ph2d_core::Playhead::new(DT);
        playhead.play();
        for k in 0..=300u64 {
            for e in 0..8u64 {
                #[expect(clippy::cast_precision_loss, reason = "um valor de teste")]
                let v = (k + e) as f32;
                m.entradas
                    .jogo
                    .set(&format!("e{e}"), Escritor::Script, Valor::Numero(v));
            }
            #[expect(clippy::cast_precision_loss, reason = "um tique")]
            playhead.seek(k as f64 * DT);
            quadro(&mut m, &gpu, &playhead, e0);
        }
        let rota = m.route_said.unwrap_or("");
        let (base, ultimo) = m.pump.fita().faixa().expect("gravou");
        assert!(
            base > 30 && ultimo == 300,
            "{rota}: dobrou? {base}..{ultimo}"
        );
        #[expect(clippy::cast_precision_loss, reason = "tiques")]
        let segundos = |t: u64| t as f64 * DT;
        assert_eq!(
            ph2d_panel_timeline::recorded_history(),
            Some((segundos(base), segundos(ultimo))),
            "{rota}: a faixa publicada é [base, último]"
        );
        let queixa = ph2d_i18n::tr_with(
            crate::motion_historia::CHAVE_DA_QUEIXA,
            &[("s", &format!("{:.2}", segundos(base)))],
        );
        playhead.pause();
        playhead.seek(segundos(base / 2));
        assert_eq!(
            quadro(&mut m, &gpu, &playhead, e0).as_deref(),
            Some(queixa.as_str()),
            "{rota}: antes da base o cartão diz que mostra a base"
        );
        playhead.seek(segundos(base + 10));
        assert_ne!(
            quadro(&mut m, &gpu, &playhead, e0).as_deref(),
            Some(queixa.as_str()),
            "{rota}: o CONTROLO — dentro da faixa o cartão não se queixa"
        );
        eprintln!("rota {rota}: base {base}, faixa até {ultimo}");
    }
}
