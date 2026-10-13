//! ⭐ **A fita dita ao artista, pelo caminho do produto** (doc 132 §2): o quadro do produto
//! (`coze_o_quadro`) a tocar com um tecto pequeno publica à Timeline a faixa `[base, último]`; com o
//! relógio parado, arrastar o playhead para antes da base faz o cartão do `value.input` dizê-lo
//! (`readout::stamp`), para antes do PRIMEIRO tique gravado diz que mostra o publicado agora, e
//! para dentro da faixa, não. Nas DUAS rotas: a CPU (sempre, sem adapter) e a placa (onde há).

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

/// O quadro do PRODUTO no segundo do `playhead` — na placa se `gpu` a dá, senão na rota da CPU
/// sem adapter nenhum — e o que o cartão de `no` diz a seguir.
fn quadro(
    m: &mut MotionState,
    gpu: Option<&ph2d_gpu::GpuContext>,
    playhead: &ph2d_core::Playhead,
    no: NodeId,
) -> Option<String> {
    match gpu {
        Some(gpu) => quadro::coze_o_quadro(m, gpu, playhead, DT),
        None => quadro::coze_o_quadro_sem_placa(m, playhead, DT),
    }
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    readout::stamp(m, None, &mut snap);
    snap.nodes.iter().find(|n| n.id == no.0)?.readout.clone()
}

#[expect(clippy::cast_precision_loss, reason = "tiques")]
fn segundos(t: u64) -> f64 {
    t as f64 * DT
}

/// Uma rota: toca de `de` a `de + 300` com as oito entradas a mudar em todo tique, pára, e arrasta
/// o playhead para antes do primeiro tique gravado, para antes da base e para dentro da faixa.
fn uma_rota(gpu: Option<&ph2d_gpu::GpuContext>) {
    const DE: u64 = 60;
    let (mut m, e0) = cena();
    m.gpu_enabled = gpu.is_some();
    let mut playhead = ph2d_core::Playhead::new(DT);
    playhead.play();
    for k in DE..=DE + 300 {
        for e in 0..8u64 {
            #[expect(clippy::cast_precision_loss, reason = "um valor de teste")]
            let v = (k + e) as f32;
            m.entradas
                .jogo
                .set(&format!("e{e}"), Escritor::Script, Valor::Numero(v));
        }
        playhead.seek(segundos(k));
        quadro(&mut m, gpu, &playhead, e0);
    }
    let rota = m.route_said.unwrap_or("");
    assert_eq!(
        m.gpu_live,
        gpu.is_some(),
        "{rota}: a rota pedida é a que cozeu"
    );
    let (base, ultimo) = m.pump.fita().faixa().expect("gravou");
    assert!(
        base > DE + 30 && ultimo == DE + 300,
        "{rota}: dobrou? {base}..{ultimo}"
    );
    assert_eq!(
        ph2d_panel_timeline::recorded_history(),
        Some((segundos(base), segundos(ultimo))),
        "{rota}: a faixa publicada é [base, último]"
    );
    let dobrada = ph2d_i18n::tr_with(
        crate::motion_historia::CHAVE_DA_QUEIXA,
        &[("s", &format!("{:.2}", segundos(base)))],
    );
    let nao_gravada = ph2d_i18n::tr(crate::motion_historia::CHAVE_NAO_GRAVADA);
    playhead.pause();
    playhead.seek(segundos(base - 5));
    assert_eq!(
        quadro(&mut m, gpu, &playhead, e0).as_deref(),
        Some(dobrada.as_str()),
        "{rota}: antes da base o cartão diz que mostra a base"
    );
    // Antes do PRIMEIRO tique gravado o replay usa o publicado agora — e o cartão di-lo, não a base.
    playhead.seek(segundos(DE / 2));
    assert_eq!(
        quadro(&mut m, gpu, &playhead, e0).as_deref(),
        Some(nao_gravada),
        "{rota}: antes do primeiro tique gravado o cartão diz que mostra o publicado agora"
    );
    assert_eq!(
        m.pump.fita().faixa(),
        Some((base, ultimo)),
        "{rota}: arrastar para fora da fita não a grava (nem a esquece)"
    );
    playhead.seek(segundos(base + 10));
    let dentro = quadro(&mut m, gpu, &playhead, e0);
    assert!(
        dentro.as_deref() != Some(dobrada.as_str()) && dentro.as_deref() != Some(nao_gravada),
        "{rota}: o CONTROLO — dentro da faixa o cartão não se queixa ({dentro:?})"
    );
    eprintln!("rota {rota}: base {base}, faixa até {ultimo}");
}

/// A rota da CPU corre SEMPRE (não precisa de placa); a da placa, onde há adapter.
#[test]
fn o_cartao_das_entradas_avisa_ao_arrastar_para_antes_da_base() {
    uma_rota(None);
    match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
        Ok(gpu) => uma_rota(Some(&gpu)),
        Err(_) => eprintln!("sem adapter: so a rota da CPU correu"),
    }
}
