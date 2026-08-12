//! **ONDE O `motion.spring` DEIXA DE HONRAR O NÚMERO** (doc 89, folha 03 — a linha 69).
//!
//! A folha marca `tension`/`friction` sem `ParamHardMax` e nota que **o próprio código** já
//! nomeia o vão: a const `MAX_STEPS` traz o comentário *"at the UI's max tension (60) the
//! adaptive count is 4, so 64 only guards absurd hand-authored overrides"*. Um `ParamHardMax`
//! diz exactamente onde o *absurdo* começa (doc 88 B2) — e **a lei deste repo é que um teto
//! digitável não pode passar do que o kernel HONRA** (a cicatriz do `lattice` 400 e do
//! `kaleidoscope` 256: uma caixa que aceita 5.000 sobre um clamp de 400 **aceita e mente**).
//!
//! O mecanismo está na aritmética do passo adaptativo:
//!
//! ```text
//!   ideal = sqrt(STABLE / tension)          STABLE = 0,05
//!   steps = ceil(dt / ideal).clamp(1, 64)   MAX_DT = 0,1
//! ```
//!
//! ⇒ `steps` satura quando `0,1 · sqrt(tension/0,05) > 64`, isto é `tension > 20.480`. Acima
//! disso o `sub_dt` **para de encolher** e o Euler explícito diverge; o guard de NaN do `eval`
//! então repõe o elemento no alvo, ou seja **a mola SALTA em silêncio**.
//!
//! ⚠️ Isso é uma DERIVAÇÃO. Esta sonda MEDE — varre a tensão pela porta do produto (o `Cook`
//! do registry, o mesmo caminho do artista) e reporta onde a trajetória deixa de assentar.
//! Se os dois números concordarem, são duas testemunhas; se não, quem manda é o relógio.
//!
//! Rodar: `cargo test -p ph2d-node-registry-init --release --test measure_spring_ceiling -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// O alvo que a mola persegue: um degrau em Y, longe da origem.
const TARGET_Y: f32 = 100.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId, to_port: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, to_port),
        delayed,
    })
    .expect("edge");
}

/// grid (1 linha, deslocada para o alvo) → spring, com o `pre` self-loop que o editor plumba.
///
/// ⚠️ **O self-loop é escrito à MÃO**: o editor o liga ao SOLTAR o nó, e um documento montado
/// por `add_node` não o ganha — sem ele o `state` chega vazio todo tique e a mola nunca integra
/// (a armadilha que o `motion.boids` já pagou, medindo 3,2 ns por agente por nunca dar um passo).
fn spring_scene(tension: f32, friction: f32) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let step = g.add_node("motion.transform");
    g.set_param(step, "offset_y", 0.0); // o DEGRAU: a marcha o levanta a meio caminho
    let spring = g.add_node("motion.spring");
    g.set_param(spring, "channel", 1.0); // Y
    g.set_param(spring, "tension", tension);
    g.set_param(spring, "friction", friction);
    wire(&mut g, seed, step, 0, false);
    wire(&mut g, step, spring, 0, false);
    wire(&mut g, spring, spring, 1, true); // out --pre--> state
    (g, spring, step)
}

fn py(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][1],
        _ => f32::NAN,
    }
}

/// Marcha `frames` tiques com passo `dt` e devolve (o Y final, o maior |Y| visto).
///
/// ⚠️ **O ORÁCULO é a DIVERGÊNCIA, não a distância ao alvo** — e a 1ª versão desta sonda
/// errava aqui: ela chamava de insana toda mola cujo Y final não estivesse a 1,0 do alvo, e
/// uma mola pouco amortecida **legitimamente** ainda oscila depois de 240 tiques. Pior, o
/// limite de pico ficou em `2×`: a resposta ao DEGRAU de uma mola sem atrito passa em
/// **exactamente 2×** o alvo — o oráculo condenava a física correta.
///
/// Um passo explícito que diverge cresce sem limite, então `10×` separa as duas coisas com
/// folga de uma ordem de grandeza; e o não-finito é o outro braço (o guard de NaN do `eval`
/// repõe o elemento no alvo, então **o assentamento APAGA a prova** — o pico é quem a guarda).
fn march(
    g: &mut Graph,
    reg: &NodeRegistry,
    spring: NodeId,
    step: NodeId,
    frames: u64,
    dt: f64,
) -> (f32, f32, f32) {
    let mut cook = Cook::new();
    let mut peak = 0.0f32;
    let mut last = f32::NAN;
    // O Y no tique logo APÓS o degrau — o terceiro braço do oráculo (ver `snapped`).
    let mut just_after = f32::NAN;
    for t in 0..frames {
        // O DEGRAU: o alvo salta para `TARGET_Y` no tique 10. Antes disso o elemento nasce NO
        // alvo e fica (`fresh id: stays at its target`), que é por que a 1ª fixture desta sonda
        // media 0,000 em toda a varredura — ela não continha o fenômeno.
        if t == 10 {
            g.set_param(step, "offset_y", TARGET_Y);
        }
        let playhead = t as f64 * dt;
        cook.advance_tick(g, reg, playhead).expect("tick");
        let out = cook.cook(g, reg, spring, playhead).expect("cook");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida da mola e um stream")
        };
        last = py(s);
        if last.is_finite() {
            peak = peak.max(last.abs());
        } else {
            peak = f32::INFINITY;
        }
        if t == 11 {
            just_after = last;
        }
    }
    (last, peak, just_after)
}

/// **O SALTO SILENCIOSO** — o modo de falha que nem o pico guarda.
///
/// ⚠️ Achado MEDINDO, não previsto: a varredura do canto deu 1.638.400 e 2 M **divergindo** e
/// 4 M *"sadia"*, com pico exactamente `100,000`. Não-monotônico é assinatura de oráculo
/// enganado — e a causa é que numa tensão absurda o passo estoura **dentro do primeiro
/// sub-passo**, o guard de NaN do `eval` repõe `x` no alvo e `v` em zero, e isso se repete a
/// cada tique: a mola fica pregada no alvo, finita e imóvel. O pico nunca vê nada.
///
/// O discriminante é o TEMPO: uma mola de verdade leva tiques para chegar; uma pregada pelo
/// guard já está lá no tique seguinte ao degrau. É a diferença entre *seguir* e *teletransportar*.
fn snapped(just_after: f32) -> bool {
    (just_after - TARGET_Y).abs() < TARGET_Y * 0.01
}

#[test]
#[ignore = "probe: prints the measured ceiling, asserts nothing"]
fn measure_where_the_spring_stops_honouring_its_numbers() {
    let reg = registry();

    // ⚠️ **Os DOIS dt, e o pior é que decide.** O `eval` clampa `dt` em `MAX_DT = 0,1`, ou
    // seja um quadro perdido ou um salto de régua entrega LEGITIMAMENTE 0,1 ao integrador.
    // Um teto que só vale a 60 fps é um teto que depende da MÁQUINA: a mola do artista
    // explodiria na primeira engasgada, e nada na tela diria por quê.
    let clocks: [(&str, f64); 2] = [("60 fps (1/60)", 1.0 / 60.0), ("PIOR (MAX_DT=0,1)", 0.1)];

    for (name, dt) in clocks {
        println!("\n===== relogio: {name} =====");

        println!("\n-- TENSION (friction = 1,5) --");
        println!(
            "{:>12}  {:>10}  {:>16}  {:>10}",
            "tension", "Y final", "pico |Y|", "diverge?"
        );
        for &tension in &[
            8.0,
            60.0,
            500.0,
            5_000.0,
            20_000.0,
            20_480.0,
            21_000.0,
            40_000.0,
            100_000.0,
            1_000_000.0,
        ] {
            let (mut g, spring, step) = spring_scene(tension, 1.5);
            let (last, peak, ja) = march(&mut g, &reg, spring, step, 600, dt);
            println!(
                "{tension:>12.0}  {last:>10.3}  {peak:>16.3}  {:>10}",
                verdict(peak, ja)
            );
        }

        // ⚠️ **O CANTO é a pergunta honesta:** o teto de um knob tem de valer com o OUTRO no
        // teto DELE. Medir a tension só com friction 1,5 responde sobre um documento que o
        // artista não é obrigado a autorar.
        println!("\n-- TENSION com o FRICTION no proprio teto (20) --");
        println!(
            "{:>12}  {:>10}  {:>16}  {:>10}",
            "tension", "Y final", "pico |Y|", "diverge?"
        );
        for &tension in &[
            20_480.0,
            100_000.0,
            400_000.0,
            800_000.0,
            1_600_000.0,
            1_638_400.0,
            2_000_000.0,
            4_000_000.0,
        ] {
            let (mut g, spring, step) = spring_scene(tension, 20.0);
            let (last, peak, ja) = march(&mut g, &reg, spring, step, 600, dt);
            println!(
                "{tension:>12.0}  {last:>10.3}  {peak:>16.3}  {:>10}",
                verdict(peak, ja)
            );
        }

        println!("\n-- FRICTION, por TENSION (o teto de um e funcao do outro) --");
        println!(
            "{:>10}  {:>12}  {:>16}  {:>10}",
            "tension", "friction", "pico |Y|", "diverge?"
        );
        for &tension in &[0.1, 8.0, 60.0, 20_480.0] {
            for &friction in &[1.5, 20.0, 21.0, 40.0, 80.0, 120.0, 200.0, 1_280.0, 1_400.0] {
                let (mut g, spring, step) = spring_scene(tension, friction);
                let (_, peak, ja) = march(&mut g, &reg, spring, step, 600, dt);
                println!(
                    "{tension:>10.1}  {friction:>12.0}  {peak:>16.3}  {:>10}",
                    verdict(peak, ja)
                );
            }
        }
    }
    println!();
}

/// Diverge = não-finito, ou pico uma ordem de grandeza além do que a física admite.
/// A resposta ao degrau de uma mola SEM atrito passa em exactamente `2×` o alvo; `10×` deixa
/// uma ordem inteira de folga entre *oscila muito* e *explode*.
fn diverged(peak: f32) -> bool {
    !peak.is_finite() || peak > TARGET_Y * 10.0
}

/// O veredito completo: explodiu, saltou (o guard escondeu), ou está sadia.
fn verdict(peak: f32, just_after: f32) -> &'static str {
    if diverged(peak) {
        "EXPLODE"
    } else if snapped(just_after) {
        "SALTA"
    } else {
        "-"
    }
}
