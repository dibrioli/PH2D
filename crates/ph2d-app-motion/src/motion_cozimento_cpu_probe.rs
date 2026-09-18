//! ⭐⭐⭐ **O QUE UM COZIMENTO DE CENTENAS DE OBJECTOS CUSTA NA CPU** — o número que faltava à
//! decisão do §10.4 do doc 115.
//!
//! # A pergunta, e porque ela NÃO precisa da placa
//!
//! A cerca da W1 derruba o **cozimento inteiro** para a CPU quando um objecto traz colisor. A
//! decisão do §10.4 — *os objectos trazem sempre a forma, ou só quando um botão o diz* — depende
//! de uma coisa só: **uma cena de centenas de objectos coze confortavelmente na CPU?** Se sim, a
//! cerca a disparar é inofensiva àquela população e a forma pode ser sempre declarada.
//!
//! ⚠️ **A COMPARAÇÃO com a placa seria outra pergunta** (quanto mais rápida ela é), e essa pediria
//! a placa em exclusão. *A decisão não a precisa* — precisa de saber se o lado lento chega, e o
//! lado lento mede-se sozinho.
//!
//! # ⛔⛔ E o número que este ficheiro substitui era uma EXTRAPOLAÇÃO
//!
//! O §10.3 do doc 115 estimava `0,023 ms` a 500 objectos, dividindo os `195,9 ms` que o doc 98
//! mede a **4,19 M** — *seis ordens de grandeza acima*, e um cozimento tem custos fixos que uma
//! divisão linear apaga. A extrapolação estava declarada como extrapolação; isto é a medição.

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// O orçamento: um quadro a 60 Hz.
const QUADRO_MS: f64 = 16.67;

/// O passo de tempo de um tique.
const DT: f64 = 1.0 / 60.0;

fn no(g: &mut Graph, tipo: &str, x: f32) -> NodeId {
    let n = g.add_node(tipo.to_string());
    g.set_pos(n, Pos { x, y: 0.0 });
    n
}

/// ⭐ **Uma cena de `n` objectos carimbados** — a forma que a ordem do dono cria: uma grelha de
/// pontos e um duplicador a estampar em cada um.
///
/// ⚠️ **É a rota do PRODUTO e não uma soma de partes:** o que a cerca derruba é o documento
/// inteiro, logo o que se mede é o documento inteiro.
fn cena(n: f32) -> (Graph, NodeId) {
    let mut g = Graph::default();
    // ⛔⛔ **A 1.ª redacção desta cena cozia o NADA, e as duas causas são silenciosas:**
    // `set_param(grade, "count", …)` — o `motion.grid` tem `rows`/`cols`, e *um `set_param` com
    // um nome que o nó não tem NÃO falha* — e o fio ia à porta `0` do duplicador, que é a
    // **forma** e não os pontos. A sonda lia `0,000 ms` a 5 000 objectos e o teste acabava em
    // `0,00 s`. ⇒ o piso de população abaixo é o que torna isto impossível outra vez.
    let lado = n.sqrt().ceil();
    let grade = no(&mut g, "motion.grid", 0.0);
    g.set_param(grade, "rows", lado);
    g.set_param(grade, "cols", lado);
    // A FORMA que é estampada em cada ponto — um objecto da cena é um template como este.
    let forma = no(&mut g, "motion.grid", 0.0);
    g.set_param(forma, "rows", 1.0);
    g.set_param(forma, "cols", 1.0);
    let dup = no(&mut g, "motion.duplicator", 200.0);
    let out = no(&mut g, "motion.output", 400.0);
    for (de, porta) in [(forma, 0u16), (grade, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .expect("entrada do duplicador");
    }
    g.connect(Edge {
        from: (dup, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("duplicador → saída");
    (g, out)
}

/// ⚠️ **O PISO DE POPULAÇÃO** — quantas peças o documento de facto emite.
///
/// *Sem isto a sonda lê zero e o zero lê-se como «é grátis».* Ela é a única testemunha de que o
/// relógio mediu alguma coisa.
fn pecas(m: &MotionState, n: f32) -> usize {
    let (g, sink) = cena(n);
    let mut cook = Cook::new();
    cook.cook(&g, &m.registry, sink, 0.0)
        .map_or(0, |v| v[0].as_stream().count())
}

/// A mediana do relógio de um quadro em regime, em ms — a mesma forma do relógio do grupo
/// (doc 114 §7): mediana de nove depois de aquecer, para um pico de escalonador não decidir a
/// tabela.
///
/// ⛔⛔⛔ **`mexe` é a diferença entre duas respostas verdadeiras, e sem ele a sonda mente.**
/// A 1.ª redacção lia **`0,001 ms` PLANO** de `100` a `5 041` peças — e um custo que não cresce
/// com a população não é um custo, é um **memo**. Este documento é `Effect::Pure`, logo com nada
/// a mudar o cozedor devolve o resultado guardado e o relógio mede a consulta ao cache.
///
/// ⭐ **As duas leituras interessam e são cenas diferentes:**
/// - `mexe = false` — a cena **parada**. É o que o app de facto paga quando ninguém toca em nada,
///   e é honesto que seja quase zero.
/// - `mexe = true` — **alguma coisa mudou** (o artista arrasta um knob, ou um nó anima). É o
///   cozimento a sério, e é este que a decisão do §10.4 precisa: *o pior caso é o que decide se a
///   cerca a disparar magoa.*
fn relogio(m: &MotionState, n: f32, mexe: bool) -> f64 {
    let (mut g, sink) = cena(n);
    let grade = g
        .nodes()
        .iter()
        .find(|x| x.type_name == "motion.grid")
        .map(|x| x.id)
        .expect("a cena tem uma grelha");
    let mut cook = Cook::new();
    let mut t = 0.0f64;
    let mut k = 0.0f32;
    let mexer = |g: &mut Graph, k: &mut f32| {
        if mexe {
            *k += 1.0;
            // Invalida o memo pela porta do produto: um param mudou, como num arrasto.
            g.set_param(grade, "gap_x", 1.0 + *k * 1e-3);
        }
    };
    for _ in 0..60 {
        mexer(&mut g, &mut k);
        let _ = cook.cook(&g, &m.registry, sink, t);
        let _ = cook.advance_tick(&g, &m.registry, t);
        t += DT;
    }
    let mut ms = Vec::with_capacity(9);
    for _ in 0..9 {
        mexer(&mut g, &mut k);
        let agora = std::time::Instant::now();
        let _ = cook.cook(&g, &m.registry, sink, t);
        ms.push(agora.elapsed().as_secs_f64() * 1e3);
        let _ = cook.advance_tick(&g, &m.registry, t);
        t += DT;
    }
    ms.sort_by(f64::total_cmp);
    ms[ms.len() / 2]
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "?".to_string())
}

/// A SONDA do §10.4. ⚠️ **Só vale numa máquina calma** — imprime o `load` ao lado.
///
/// ```text
/// cargo test -p ph2d-app-motion --release cozimento_cpu -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medição, não gate"]
fn cozimento_cpu_da_populacao_do_dono() {
    let m = MotionState::new();
    eprintln!("\n  ═══ O COZIMENTO NA CPU, À POPULAÇÃO DO DONO (doc 115 §10.4) ═══\n");
    eprintln!(
        "  Uma grelha de `n` pontos com um duplicador a estampar em cada um — o documento\n  \
         INTEIRO, que é o que a cerca da W1 derruba. Mediana de 9 depois de 60 tiques a aquecer.\n  \
         Um quadro tem {QUADRO_MS} ms.\n"
    );
    eprintln!(
        "  {:<9} │ {:>7} │ {:>13} │ {:>15} │ {:>12}",
        "pedidos", "PEÇAS", "cena PARADA", "algo MUDOU", "% de um quadro"
    );
    eprintln!("  ----------|---------|---------------|-----------------|-------------");
    for n in [100.0f32, 250.0, 500.0, 1000.0, 5000.0] {
        let p = pecas(&m, n);
        assert!(
            p >= (n as usize) / 2,
            "o documento emitiu {p} peças para {n} pedidas — a sonda está a medir o NADA"
        );
        let parada = relogio(&m, n, false);
        let mudou = relogio(&m, n, true);
        eprintln!(
            "  {n:<9.0} │ {p:>7} │ {parada:>10.3} ms │ {mudou:>12.3} ms │ {:>11.2}%",
            mudou / QUADRO_MS * 100.0
        );
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}
