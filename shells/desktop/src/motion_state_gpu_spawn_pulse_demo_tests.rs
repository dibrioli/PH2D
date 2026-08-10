//! Gates da cena **AS CINCO FONTES** (`PH2D_GPU_COOK_DEMO=24`) — a P0 da folha 12 do doc 89.
//!
//! O que estas medições protegem não é a aparência: é a afirmação de que **o pulso é o único
//! autor** da população (a taxa está em zero) e de que **cada baforada sai da boca que firou**.
//! As duas metades falham de formas diferentes — a primeira deixa a tela vazia, a segunda
//! colapsa cinco fontes numa —, então cada uma tem gate próprio.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

const FPS: f64 = 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

/// Roda a cena por `secs` e devolve, por tique, quantos elementos a zona segura.
fn run(secs: f64) -> Vec<usize> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_spawn_pulse_demo_document(&mut doc, &reg).expect("cena bem tipada");
    let sink = sinks[0];
    let mut cook = Cook::new();
    let mut counts = Vec::new();
    for k in 0..=((secs * FPS) as u64) {
        let t = k as f64 / FPS;
        let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
        counts.push(out[0].as_stream().count());
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("fecha o tique");
    }
    counts
}

/// **O pulso é o ÚNICO autor, e a cena cresce em DEGRAUS.**
///
/// Com `rate = 0` nada nasce entre as batidas, então a contagem fica parada na maioria dos
/// tiques e salta nos poucos em que o metrônomo dispara — a assinatura de um EVENTO, e o
/// oposto exato do fio contínuo que uma taxa desenha. ⚠️ A metade *"a maioria dos tiques é
/// quieta"* é o que distingue esta cena de uma que só *funciona*: sem ela o gate passaria
/// igualzinho sobre uma taxa comum.
///
/// ⚠️ **O QUE ESTE GATE MEDE É A VARIAÇÃO LÍQUIDA, e a distinção custou uma correção:** a
/// primeira versão afirmava *"toda batida dá exatamente `NOZZLES × BURST`"* sobre uma janela de
/// 1,6 s e ficava verde — mas 1,6 s é a própria `life`, então a janela **acabava antes da
/// primeira morte**. Medida a 4 s a sequência é `40,40,40,39,39,40,39,39`: o dígito que falta
/// não é um nascimento perdido, é um elemento que morreu no MESMO tique. A premissa fica
/// declarada em vez de herdada — e a afirmação forte (o salto CHEIO) vale só onde ela é
/// verdadeira, com a afirmação robusta (*saltos só nas batidas*) cobrindo o resto.
#[test]
fn a_cena_so_nasce_nas_batidas() {
    // A janela CURTA: antes de a primeira morte poder acontecer, o salto é a natalidade pura.
    let vida_curta = run(1.4);
    assert_eq!(
        vida_curta[0], 0,
        "o primeiro tique não tem dt: ninguém nasce"
    );
    let esperado = (NOZZLES * BURST) as usize;
    let saltos: Vec<usize> = vida_curta
        .windows(2)
        .filter(|w| w[1] > w[0])
        .map(|w| w[1] - w[0])
        .collect();
    assert!(
        !saltos.is_empty() && saltos.iter().all(|s| *s == esperado),
        "antes da primeira morte toda batida dá as {esperado} de uma vez: {saltos:?}"
    );

    // A janela LONGA: o que é invariante é ONDE os saltos caem — um a cada `PERIOD`, e nada
    // entre eles, por mais que a morte já esteja levando gente embora.
    let counts = run(4.0);
    let passo = (PERIOD as f64 * FPS).round() as usize;
    let tiques: Vec<usize> = counts
        .windows(2)
        .enumerate()
        .filter(|(_, w)| w[1] > w[0])
        .map(|(i, _)| i + 1)
        .collect();
    assert!(
        tiques.len() >= 6 && tiques.iter().all(|t| t % passo == 0),
        "nascimento só na batida (a cada {passo} tiques): {tiques:?}"
    );
    let quietos = counts.windows(2).filter(|w| w[1] == w[0]).count();
    assert!(
        quietos > counts.len() / 2,
        "com a taxa em ZERO a maioria dos tiques não gera nada ({quietos} de {})",
        counts.len() - 1
    );
}

/// **Cada baforada sai da BOCA que firou** — o oráculo de olho da cena, medido.
///
/// O modo de falha natural (todo recém-nascido colhendo a linha 0) colapsaria as cinco fontes
/// numa só, e é exatamente isso que este gate recusa: as posições dos recém-nascidos do
/// primeiro instante têm de cobrir as CINCO abscissas distintas do template.
#[test]
fn cada_baforada_sai_da_boca_que_firou() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_spawn_pulse_demo_document(&mut doc, &reg).expect("cena bem tipada");
    let sink = sinks[0];
    let mut cook = Cook::new();
    let mut bocas: Vec<i64> = Vec::new();
    // Uma janela de um segundo cobre DUAS batidas — a primeira pode cair fora de um trecho
    // curto, e um gate que não contém o fenômeno não mede nada.
    for k in 0..=60u64 {
        let t = k as f64 / FPS;
        let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
        let s = out[0].as_stream();
        if s.count() > 0
            && bocas.is_empty()
            && let Some(Column::Vec2(p)) = s.get("P")
        {
            // Milésimos: a queda já começou, então as abscissas são as das bocas mas os
            // valores não são literais — o que se conta é quantas DISTINTAS existem.
            bocas = p.iter().map(|v| (v[0] * 1000.0).round() as i64).collect();
            bocas.sort_unstable();
            bocas.dedup();
        }
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("fecha o tique");
    }
    assert_eq!(
        bocas.len(),
        NOZZLES as usize,
        "as cinco bocas dão à luz, não uma: abscissas distintas {bocas:?}"
    );
}

/// **A porta desconectada é o mundo de antes** — o CONTROLE, e sem ele os dois gates acima
/// passariam sobre um `sim.spawn` que desse à luz por conta própria.
#[test]
fn sem_o_fio_do_pulso_a_cena_fica_vazia() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_spawn_pulse_demo_document(&mut doc, &reg).expect("cena bem tipada");
    let sink = sinks[0];
    // Acha o `sim.spawn` e solta a porta 1 (o `expect` é o CONTROLE do controle: se a cena
    // deixasse de fiar a porta, este gate ficaria verde sobre uma ablação que não aconteceu).
    let spawn = doc
        .graph
        .nodes()
        .iter()
        .position(|n| n.type_name == "sim.spawn")
        .map(|i| ph2d_nodegraph::graph::NodeId(i as u32))
        .expect("a cena tem um sim.spawn");
    doc.graph
        .disconnect(spawn, 1)
        .expect("a cena fia a porta `pulse`");

    let mut cook = Cook::new();
    for k in 0..=96u64 {
        let t = k as f64 / FPS;
        let out = cook.cook(&doc.graph, &reg, sink, t).expect("cozinha");
        assert_eq!(
            out[0].as_stream().count(),
            0,
            "taxa ZERO e nenhum pulso: a cena não pode inventar população (tique {k})"
        );
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("fecha o tique");
    }
}

/// Sonda: o retrato da cena em números, para a mensagem de smoke **medir** em vez de estimar.
///
/// `cargo test -p ph2d-host-desktop --bins probe_cinco_fontes -- --ignored --nocapture`
#[test]
#[ignore = "sonda de diagnóstico: imprime, não afirma"]
fn probe_cinco_fontes() {
    let counts = run(4.0);
    let saltos: Vec<(usize, usize)> = counts
        .windows(2)
        .enumerate()
        .filter(|(_, w)| w[1] > w[0])
        .map(|(i, w)| (i + 1, w[1] - w[0]))
        .collect();
    let quietos = counts.windows(2).filter(|w| w[1] == w[0]).count();
    println!("[cinco fontes] bocas {NOZZLES} x burst {BURST} a cada {PERIOD}s");
    println!("  batidas em 4 s: {} -> tiques {:?}", saltos.len(), {
        let t: Vec<usize> = saltos.iter().map(|(t, _)| *t).collect();
        t
    });
    println!(
        "  nasce por batida: {:?} (esperado {})",
        saltos.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
        (NOZZLES * BURST) as usize
    );
    println!(
        "  populacao: pico {} - final {} - tiques QUIETOS {} de {}",
        counts.iter().max().unwrap(),
        counts.last().unwrap(),
        quietos,
        counts.len() - 1
    );
}
