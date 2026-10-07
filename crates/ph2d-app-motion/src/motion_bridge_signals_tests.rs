//! **Gates do lado de Motion da fronteira** — o que a ponte lê das tomadas e o que ela recusa
//! a publicar.
//!
//! Eles dirigem o pump pelas MESMAS portas de marcha que o `dispatch` usa — as DUAS, uma por
//! rota de cook ([`Rota`]) — e leem o livro-razão UMA vez no fim, como o quadro faz. É isso que
//! os separa de um espelho: o `MotionState` nasce sem janela, então o caminho de produto inteiro
//! — cook, tomada, nome, contagem — cabe num teste de unidade.
//!
//! ⚠️ **A porta única `advance_or_scrub_with_taps_scoped` NÃO existe mais**, e a razão está no
//! defeito que ela causou: a tomada era argumento de UMA das marchas, e o produto tem duas. Hoje
//! ela é estado da bomba (`set_taps`) e o resultado é carimbado num livro-razão por TIQUE.
//!
//! ⚠️ **O que NÃO cabe aqui é a LEI do relógio** (*só publica tocando para a frente*): ela mora
//! no `dispatch`, que exige o `HeroScreen` vivo. Quem a afirma é o par
//! `clock_forward::tests` (a pergunta) + o arch-gate `the_graph_only_shouts_while_the_clock_plays_forward`
//! (o lugar onde ela é feita).

use super::{collect_signals, signal_nodes};
use crate::motion_state::gpu_adsr_demo::{
    COMPASSO, DIVIDE_BY, SIDE, TIC, build_gpu_adsr_demo_document, build_gpu_signal_demo_document,
};
use crate::motion_state::{MotionSignalOut, MotionState};
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::NodeId;

const FPS: f64 = 60.0;

type Scene = fn(&mut MotionDoc, &NodeRegistry) -> Option<Vec<NodeId>>;

/// **Por qual PORTA a bomba é marchada** — e a fixture tem as duas porque o produto tem as duas.
///
/// ⚠️ Esta é a metade que faltava, e o preço foi um smoke: a cena `=26` planeja **híbrida**
/// (medido: `boundaries = [5, 4]`, 4 estágios de despacho), então o quadro real marcha por
/// [`Rota::Hibrida`] e devolve `Handled` — enquanto TODO gate desta suíte dirigia a
/// [`Rota::Cpu`]. O grafo cozinhava, desenhava e não gritava nada, com quatro gates verdes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Rota {
    /// A bomba renderiza os sinks (GPU off, ou plano que não reivindica trabalho útil).
    Cpu,
    /// O prefixo de CPU cozinha até as fronteiras do plano; o sufixo é da GPU.
    Hibrida,
}

/// Roda `secs` de cena pela porta indicada e devolve **tudo o que ela gritou**.
///
/// ⚠️ **A leitura acontece UMA vez, no fim**, como no shell: o livro-razão (`tap_fires`) é
/// carimbado por qualquer marcha, então quem lê não precisa saber que rota o quadro tomou.
fn shouts_of(
    scene: Scene,
    secs: f64,
    rota: Rota,
    rename: impl Fn(&mut MotionState),
) -> Vec<MotionSignalOut> {
    let mut motion = MotionState::new();
    motion.sinks = scene(&mut motion.doc, &motion.registry).expect("a cena é bem tipada");
    rename(&mut motion);
    motion.signal_taps = signal_nodes(&motion.doc.graph);
    motion.pump.set_taps(&motion.signal_taps);
    motion.pump.clear_tap_fires();
    // As fronteiras do plano — o que a rota híbrida entrega à bomba.
    let boundaries: Vec<NodeId> = {
        let plan = ph2d_gpu_cook::plan(
            &motion.doc.graph,
            &motion.registry,
            &motion.registry,
            motion.sinks[0],
        );
        plan.boundaries.iter().map(|(n, _)| *n).collect()
    };
    let scopes = TimeScopes::new();
    for tick in 0..=((secs * FPS) as u64) {
        match rota {
            Rota::Cpu => motion.pump.advance_or_scrub_scoped(
                &motion.doc.graph,
                &motion.registry,
                &motion.sinks,
                tick,
                |t| t as f64 / FPS,
                motion.default_uv_rect,
                motion.default_size,
                &scopes,
            ),
            Rota::Hibrida => motion.pump.advance_or_scrub_to_nodes_scoped(
                &motion.doc.graph,
                &motion.registry,
                &boundaries,
                tick,
                |t| t as f64 / FPS,
                &scopes,
            ),
        };
    }
    collect_signals(&mut motion);
    std::mem::take(&mut motion.signals_out)
}

fn shouts(secs: f64) -> Vec<MotionSignalOut> {
    shouts_of(build_gpu_signal_demo_document, secs, Rota::Cpu, |_| {})
}

/// **Um grito carrega o NOME, o TIQUE e quantas linhas dispararam.** As três coisas que um
/// consumidor não pode redescobrir sozinho: ele não vê o grafo, não sabe em que tique o cook
/// está, e — o mais fácil de perder — não sabe se aquilo foi um ponto ou a grade inteira.
#[test]
fn um_grito_traz_o_nome_o_tique_e_quantas_linhas() {
    let out = shouts(2.0);
    assert!(!out.is_empty(), "dois segundos de compasso gritam");
    let rows = (SIDE * SIDE) as usize;
    for s in &out {
        assert!(
            s.name == TIC || s.name == COMPASSO,
            "só os dois nomes autorados saem, veio {:?}",
            s.name
        );
        assert!(s.tick <= 120, "o tique é o do cook, veio {}", s.tick);
        assert_eq!(
            s.rows, rows,
            "⚠️ a grade inteira dispara junta: UM evento com {rows} linhas, não {rows} eventos"
        );
    }
    eprintln!(
        "gritos em 2 s: {} | linhas por grito: {rows} | primeiro: {:?}@{}",
        out.len(),
        out[0].name,
        out[0].tick
    );
}

/// **O terminal conta a MESMA razão que o olho** — entre dois `compasso` cabem exatamente
/// `DIVIDE_BY` `tic`.
///
/// É o gate que faz da cena `=26` um instrumento em vez de uma demonstração: o número que o
/// artista lê no log é o número que ele conta na tela, e um `carry` mal ligado move os dois.
#[test]
fn entre_dois_compassos_cabem_quatro_tics() {
    let out = shouts(6.0);
    let at = |name: &str| -> Vec<u64> {
        out.iter()
            .filter(|s| s.name == name)
            .map(|s| s.tick)
            .collect()
    };
    let (tic, bar) = (at(TIC), at(COMPASSO));
    assert!(
        bar.len() >= 3,
        "6 s dão 3+ compassos para ter 2+ intervalos, medido {} ({bar:?})",
        bar.len()
    );
    for w in bar.windows(2) {
        let n = tic.iter().filter(|k| **k >= w[0] && **k < w[1]).count();
        assert_eq!(
            n, DIVIDE_BY as usize,
            "entre os compassos em {} e {} cabem {DIVIDE_BY} tics, contados {n} ({tic:?})",
            w[0], w[1]
        );
    }
    eprintln!("em 6 s: {} tics · {} compassos", tic.len(), bar.len());
}

/// **Uma tomada SEM nome fica calada, e a irmã continua gritando.**
///
/// Não é validação: um `pulse.signal` acabado de soltar no grafo tem o campo vazio, e um sinal
/// anônimo não é endereçável por consumidor nenhum. A metade que importa é a segunda — apagar
/// um nome não pode emudecer o resto do documento.
#[test]
fn uma_tomada_sem_nome_nao_grita_e_a_irma_continua() {
    let out = shouts_of(build_gpu_signal_demo_document, 6.0, Rota::Cpu, |motion| {
        let tap = signal_nodes(&motion.doc.graph)[0];
        motion
            .doc
            .graph
            .set_text_param(tap, ph2d_node_pulse_signal::NAME_KEY, "   ");
        // ⚠️ Espaços, não vazio: o campo que o artista de facto deixa quando apaga o texto.
    });
    let names: std::collections::BTreeSet<&str> = out.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names.len(),
        1,
        "uma das duas emudeceu e a outra ficou, veio {names:?}"
    );
    assert!(!out.is_empty(), "a irmã nomeada continua gritando");
}

/// **Sem tomadas o mundo é o de antes** — a cena `=25` não tem `pulse.signal`, então a lista de
/// tomadas é vazia e nada é lido nem guardado. É o CONTROLE: sem ele, um gate que só afirma
/// *"a `=26` grita"* não distingue *o nó funciona* de *toda cena grita*.
#[test]
fn a_cena_sem_tomadas_nao_grita_nada() {
    let mut motion = MotionState::new();
    motion.sinks =
        build_gpu_adsr_demo_document(&mut motion.doc, &motion.registry).expect("cena bem tipada");
    assert!(
        signal_nodes(&motion.doc.graph).is_empty(),
        "a cena do compasso não tem tomada nenhuma"
    );
    assert!(shouts_of(build_gpu_adsr_demo_document, 2.0, Rota::Cpu, |_| {}).is_empty());
}

/// **AS DUAS ROTAS GRITAM O MESMO — o gate que faltava.**
///
/// A cena `=26` planeja **híbrida**, então o quadro real nunca passou pela porta de sinks: o
/// `pulse.signal` cozinhava, a tela animava e o terminal ficava mudo, com quatro gates verdes
/// nesta suíte. A causa não era a leitura nem a lei — era a TOMADA ser argumento de UMA das
/// portas de marcha, e o produto ter duas.
///
/// ⚠️ Ele compara as duas listas INTEIRAS (nome e tique), e não uma contagem: a rota híbrida
/// cozinha até as fronteiras do plano, então nada garante *a priori* que uma tomada a montante
/// veja o mesmo tique — e é exatamente isso que o gate existe para afirmar.
#[test]
fn as_duas_rotas_de_cook_gritam_a_mesma_coisa() {
    let cpu = shouts_of(build_gpu_signal_demo_document, 4.0, Rota::Cpu, |_| {});
    let hibrida = shouts_of(build_gpu_signal_demo_document, 4.0, Rota::Hibrida, |_| {});
    assert!(
        !hibrida.is_empty(),
        "a rota HÍBRIDA é a que o quadro real toma nesta cena — se ela cala, o produto cala"
    );
    let resumo = |v: &[MotionSignalOut]| -> Vec<(String, u64)> {
        v.iter().map(|s| (s.name.clone(), s.tick)).collect()
    };
    assert_eq!(
        resumo(&cpu),
        resumo(&hibrida),
        "a rota não pode mudar o que o grafo grita"
    );
}

/// **A cena `=26` é INTEIRA na placa desde 07/10** (doc 110 §14.2) — a premissa dos gates acima,
/// medida em vez de suposta. ⛔ Até lá ela era híbrida POR ACASO: o `carry` do `pulse.counter`
/// derrubava o contador para a CPU, e a marcha da bomba gritava. O `pulse.signal` tem kernel desde
/// a W2 do ciclo 6, logo «um grafo com sinal nunca é 100 % placa» deixou de ser verdade sem
/// ninguém o dizer. ⇒ o grito de um documento inteiro na placa vem da marcha do CONE das tomadas
/// (zero fronteiras), e é isso que o irmão acima compara com a CPU.
#[test]
fn a_cena_do_grito_e_inteira_na_placa_e_as_tomadas_marcham() {
    let mut motion = MotionState::new();
    motion.sinks = build_gpu_signal_demo_document(&mut motion.doc, &motion.registry)
        .expect("a cena é bem tipada");
    let plan = ph2d_gpu_cook::plan(
        &motion.doc.graph,
        &motion.registry,
        &motion.registry,
        motion.sinks[0],
    );
    assert!(plan.is_fully_gpu(), "fronteiras: {:?}", plan.boundaries);
    assert!(
        !signal_nodes(&motion.doc.graph).is_empty(),
        "e ela tem tomada de sinal — senão o irmão compara dois silêncios"
    );
}

/// ⭐⭐ **PELA PONTE, COM PLACA: a `=26` inteira no dispositivo grita o que a CPU grita** — o
/// quadro do produto (`coze_o_quadro`), quatro segundos, a lista inteira (nome e tique).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_ponte_na_placa_grita_o_que_a_cpu_grita() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        return;
    };
    let cpu = shouts_of(build_gpu_signal_demo_document, 4.0, Rota::Cpu, |_| {});
    let mut motion = MotionState::new();
    motion.sinks = build_gpu_signal_demo_document(&mut motion.doc, &motion.registry)
        .expect("a cena é bem tipada");
    motion.signal_taps = signal_nodes(&motion.doc.graph);
    let mut playhead = ph2d_core::Playhead::new(1.0 / FPS);
    let mut placa = Vec::new();
    for tick in 0..=((4.0 * FPS) as u64) {
        playhead.seek(tick as f64 / FPS);
        motion.pump.set_taps(&motion.signal_taps.clone());
        motion.pump.clear_tap_fires();
        motion.signals_out.clear();
        crate::motion_bridge::quadro::coze_o_quadro(&mut motion, &gpu, &playhead, 1.0 / FPS);
        assert_eq!(
            motion.route_said,
            Some("device: o plano inteiro (fully-GPU)"),
            "a cena tem de correr INTEIRA na placa"
        );
        collect_signals(&mut motion);
        placa.extend(std::mem::take(&mut motion.signals_out));
    }
    let resumo = |v: &[MotionSignalOut]| -> Vec<(String, u64)> {
        v.iter().map(|s| (s.name.clone(), s.tick)).collect()
    };
    assert!(!cpu.is_empty());
    assert_eq!(
        resumo(&cpu),
        resumo(&placa),
        "a placa não pode mudar o que o grafo grita"
    );
}
