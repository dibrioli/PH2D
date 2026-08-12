//! **QUANTO UM SUBSTEP COMPRA** (doc 89, folha 13 — o P1 `sim.zone` / SUBSTEPS).
//!
//! A folha diz que substeps é **omissão P1** citando o `Substeps` do DOP Network do Houdini,
//! o `Time Step` do Cavalry e as *Simulation Stages* do Niagara, e nomeia por que a cadeia
//! não existe: *"encadear `sim.step` duas vezes é no-op exato (o 1º escreve `sim_t =
//! playhead`, o 2º lê `dt = 0`) — e não há outro lugar para pôr um 2º passe: o cook roda o
//! interior UMA vez por tick."*
//!
//! **Antes de construir, meça o que o item compra** (CLAUDE.md §0). E o interior inteiro
//! pode ser sub-passado HOJE, sem tocar no motor, porque **o `dt` do `sim.step` sai do
//! playhead** (`dt = clamp(playhead − sim_t, 0, MAX_DT)`, `sim-step:315`): cozinhar o mesmo
//! tique em `N` playheads intermediários dá a cada passe `dt = 1/(60N)` e roda **forças,
//! integração e colisão** em cada um — que é exatamente o que a referência chama de substep.
//! Esta sonda faz isso e imprime a tabela; nenhuma linha de produto depende dela.
//!
//! ⚠️ **A fixture tem de ISOLAR o mecanismo:** `force.wind` é `Effect::Temporal` porque as
//! rajadas amostram o playhead, então com `gust ≠ 0` cada substep veria **outra força** e a
//! tabela misturaria *"integrei mais fino"* com *"soprei diferente"*. `gust = 0` deixa a
//! força constante e a única variável é o tamanho do passo.
//!
//! ⚠️ **E o oráculo é o TÚNEL, não a profundidade:** um `Plane` é um semi-espaço INFINITO —
//! quem cai dentro dele é empurrado de volta por mais fundo que entre, então ele não pode
//! ser atravessado e não mede nada sobre substeps. Quem mede é o **Disc**, um obstáculo
//! FINITO: um elemento rápido o bastante pula de um lado ao outro dentro de um tique e nunca
//! vê o contato.
//!
//! Rodar: `cargo test -p ph2d-node-registry-init --release --test measure_substeps -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn connect(g: &mut Graph, from: NodeId, from_port: u16, to: NodeId, to_port: u16) {
    g.connect(Edge {
        from: (from, from_port),
        to: (to, to_port),
        delayed: false,
    })
    .expect("edge");
}

/// Um elemento solto em `(0, start_y)`, caindo por vento constante sobre um **Disc** de
/// raio `r` na origem. O laço tem a forma que o motor gerencia: `pre` da zona para o
/// PRIMEIRO nó do corpo, volta ao `state` por aresta normal.
fn falling_world(
    g: &mut Graph,
    start_y: f32,
    gravity: f32,
    disc_r: f32,
    max_speed: f32,
) -> (NodeId, NodeId) {
    let src = g.add_node("motion.grid");
    g.set_param(src, "rows", 1.0);
    g.set_param(src, "cols", 1.0);
    g.set_param(src, "gap_x", 1.0);
    g.set_param(src, "gap_y", 1.0);

    // A grade nasce na origem; um `motion.transform` a sobe para a altura de queda.
    // ⚠️ O param é `offset_y` — a 1ª versão desta sonda escreveu `translate_y`, que o
    // `set_param` aceita em silêncio (o `validate` recusaria, o `cook` não o chama), e a
    // tabela inteira saiu com o elemento nascendo DENTRO do disco: quatro colunas
    // idênticas dizendo "PEGOU" sobre um mundo que nunca caiu.
    let lift = g.add_node("motion.transform");
    g.set_param(lift, "offset_y", start_y);
    connect(g, src, 0, lift, 0);

    let zone = g.add_node("sim.zone");
    connect(g, lift, 0, zone, 0);

    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 270.0); // para baixo
    g.set_param(wind, "strength", gravity);
    g.set_param(wind, "gust", 0.0); // o CONTROLE: força constante, um passe é igual ao outro
    g.set_param(wind, "gust_freq", 0.0);

    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 0.0);
    g.set_param(step, "max_speed", max_speed);
    g.set_param(step, "min_speed", 0.0);

    let coll = g.add_node("sim.collide");
    g.set_param(coll, "shape", 1.0); // Disc — o obstáculo FINITO
    g.set_param(coll, "center_x", 0.0);
    g.set_param(coll, "center_y", 0.0);
    g.set_param(coll, "radius", disc_r);
    g.set_param(coll, "restitution", 0.0);
    g.set_param(coll, "friction", 1.0);

    g.connect(Edge {
        from: (zone, 0),
        to: (wind, 0),
        delayed: true,
    })
    .expect("a entrada de estado que o motor gerencia");
    connect(g, wind, 0, step, 0);
    connect(g, step, 0, coll, 0);
    connect(g, coll, 0, zone, 1);
    (zone, coll)
}

fn pos_y(s: &Stream) -> Option<f32> {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => Some(v[0][1]),
        _ => None,
    }
}

/// Corre `ticks` quadros a 60 fps com `substeps` passes por tique, devolvendo `(y final,
/// PRIMEIRO y visto)`. Com `substeps = 1` é literalmente o laço do pump de hoje.
///
/// ⚠️ O primeiro y é o **controle**, e ele não existe no tique 0: a zona só emite `init`
/// nesse cook, o corpo lê o `pre` VAZIO e a saída do colisor não tem `P` nenhum.
fn run(g: &Graph, reg: &NodeRegistry, out: NodeId, ticks: usize, substeps: usize) -> (f32, f32) {
    let mut cook = Cook::new();
    let mut last = f32::NAN;
    let mut first = f32::NAN;
    for k in 0..ticks {
        for j in 0..substeps {
            let t = (k as f64 + j as f64 / substeps as f64) / 60.0;
            let s = cook.cook(g, reg, out, t).expect("cooks")[0]
                .as_stream()
                .clone();
            if let Some(y) = pos_y(&s) {
                last = y;
                if first.is_nan() {
                    first = y;
                }
            }
            cook.advance_tick(g, reg, t).expect("advances");
        }
    }
    (last, first)
}

#[test]
#[ignore = "measurement probe; run with --ignored --nocapture"]
fn measure_what_a_substep_buys() {
    let reg = registry();

    // O CONTROLE, antes de qualquer coluna: o mundo de fato começa lá em cima?
    // (Um elemento que nasce DENTRO do disco reporta "PEGOU" em toda linha da tabela — foi
    // o que a 1ª corrida desta sonda imprimiu, com um param escrito errado.)
    {
        let mut g = Graph::new();
        let (_zone, out) = falling_world(&mut g, 80.0, 200.0, 1.0, 0.0);
        let (_, y0) = run(&g, &reg, out, 3, 1);
        println!("CONTROLE: 1o y visto = {y0:.3} (tem de ser ~80, nunca ~1)");
        assert!(
            y0 > 70.0,
            "a fixture nao contem o fenomeno: o elemento nao cai"
        );
    }

    println!("\n=== O DISC É ATRAVESSADO? (queda livre sobre obstáculo finito, r = 1) ===");
    println!("start_y  g    max_speed  substeps   y_final    veredito");
    for &(start_y, gravity, cap) in &[
        (40.0f32, 200.0f32, 0.0f32),
        (80.0, 200.0, 0.0),
        // A coluna que pergunta se o item DISSOLVE: o teto de velocidade que shipou hoje
        // mantém o passo por tique abaixo do diâmetro do obstáculo — ele já basta?
        (80.0, 200.0, 100.0),
    ] {
        for &n in &[1usize, 2, 4, 8] {
            let mut g = Graph::new();
            let (_zone, out) = falling_world(&mut g, start_y, gravity, 1.0, cap);
            let (yf, _) = run(&g, &reg, out, 120, n);
            // Pousar SOBRE o disco = repousar perto de +r; atravessar = seguir descendo.
            let caught = yf > -1.0;
            println!(
                "{start_y:6.0}  {gravity:4.0}  {cap:8.0}   {n:5}     {yf:8.3}     {}",
                if caught { "PEGOU" } else { "ATRAVESSOU" }
            );
        }
    }

    // A OUTRA metade do que a referência compra, e a que um teto de velocidade não dá:
    // FIDELIDADE. Queda livre sem obstáculo, contra a resposta analítica.
    println!("\n=== FIDELIDADE (queda livre, 30 tiques, g = 200, sem colisor) ===");
    let ticks = 30usize;
    let t = ticks as f32 / 60.0;
    let exact = 80.0 - 0.5 * 200.0 * t * t;
    // ⚠️ O resíduo contra o analítico NÃO é erro do integrador e não vai a zero: a zona
    // emite só o `init` no 1º cook (o corpo lê um `pre` vazio) e um recém-nascido tem
    // `dt = 0` no primeiro `sim.step` por LEI (`sim-step:26`) — duas partidas atrasadas, que
    // encolhem em tempo de parede conforme o passe afina. Quem mede o que o substep muda é
    // a coluna Δ contra a linha anterior.
    println!("substeps    y(30)     vs analitico   Δ vs linha acima");
    let mut prev: Option<f32> = None;
    for &n in &[1usize, 2, 4, 8, 16] {
        let mut g = Graph::new();
        // O disco fica LONGE (raio minúsculo na origem) para não interferir na queda.
        let (_zone, out) = falling_world(&mut g, 80.0, 200.0, 0.001, 0.0);
        let (yf, _) = run(&g, &reg, out, ticks, n);
        match prev {
            Some(p) => println!(
                "{n:5}     {yf:9.4}   {:+8.4}      {:+8.4}",
                yf - exact,
                yf - p
            ),
            None => println!("{n:5}     {yf:9.4}   {:+8.4}             —", yf - exact),
        }
        prev = Some(yf);
    }
    println!(
        "analitico  {exact:9.4}   (queda ideal de {:.1} unidades)",
        80.0 - exact
    );
}
