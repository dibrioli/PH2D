//! **OS DOIS `MAX_DT` QUE FICARAM POR MEDIR** — a dívida que o doc
//! [91](../../../docs/Motion%20Nodes/91_os_tetos_que_ninguem_mediu.md) §5.4 regista:
//!
//! > ⏳ *"`motion.boids` e `motion.wave` continuam com `0,1` por medir. Eles copiaram o número
//! > sem derivação, como os dois que este bloco curou."*
//!
//! ⚠️⚠️ **E a primeira coisa que a medição diz é que eles NÃO SÃO O MESMO PROBLEMA** — o doc
//! trata-os como um item só, e o código diz outra coisa:
//!
//! | nó | a assinatura do passo | o `dt` chega lá? |
//! |---|---|---|
//! | `motion.boids` | `step(pos, vel, accel, w, …, **dt**, p)` | **SIM** — é um grampo de estabilidade a sério |
//! | `motion.wave` | `step(h, h_prev, drive, p)` | **NÃO** — o passo do leapfrog é FIXO |
//!
//! No `motion.wave` o `dt` aparece em **uma** linha do arquivo inteiro (`if dt < 1e-6`), e o
//! grampo superior não pode moldar coisa nenhuma: qualquer valor acima de `1e-6` dá exactamente
//! o mesmo passo. ⇒ **um dos dois é um teto; o outro é uma constante inerte.**
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_boids_and_wave_ceilings -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// A maior distância de uma peça à origem — a régua de *"isto explodiu?"*.
fn excursion(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(p)) => p
            .iter()
            .map(|q| (q[0] * q[0] + q[1] * q[1]).sqrt())
            .fold(0.0f32, f32::max),
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// `motion.boids` — o teto de VERDADE
// ---------------------------------------------------------------------------

/// Um bando com o laço de estado fechado, puxado por um alvo.
fn flock(g: &mut Graph, seek: f32) -> NodeId {
    let b = g.add_node("motion.boids");
    g.set_param(b, "count", 64.0);
    g.set_param(b, "radius", 3.0);
    // ⚠️ **O `seek` é a mola desta malha**: ele é a única força cuja magnitude depende da
    // POSIÇÃO, e é uma malha fechada que diverge — a mesma escolha do `integrator_ceilings`
    // (uma força constante acelera em linha recta, o que é grande mas não é instável).
    g.set_param(b, "seek", seek);
    g.set_param(b, "max_speed", 100.0);
    g.set_param(b, "max_force", 100.0);
    // ⚠️ A porta de estado é a **2**: `0`/`1` são o `target_x`/`target_y`.
    wire(g, b, 0, b, 2, true);
    b
}

/// Corre `ticks` com um passo de relógio de `dt` e devolve a maior excursão vista.
fn run_boids(reg: &NodeRegistry, seek: f32, dt: f64, ticks: usize) -> f32 {
    let mut g = Graph::new();
    let b = flock(&mut g, seek);
    g.validate(reg).expect("bem-tipado");
    let mut cook = Cook::new();
    let mut worst = 0.0f32;
    for k in 0..ticks {
        let t = k as f64 * dt;
        let out = cook.cook(&g, reg, b, t).expect("coza");
        worst = worst.max(excursion(out[0].as_stream()));
        cook.advance_tick(&g, reg, t).expect("avanca");
    }
    worst
}

/// A distância MÉDIA ao vizinho mais próximo — a régua de *"isto ainda é um bando?"*.
///
/// ⚠️ **A excursão não serve para este nó.** O `max_speed`/`max_force` limitam o passo por
/// construção, então nada aqui diverge — a tabela de excursão mostra o grampo a funcionar e
/// **nenhuma coluna a explodir**. O que um `dt` grande de facto estraga é outra coisa: um
/// pássaro que anda mais do que o próprio RAIO DE PERCEPÇÃO num tique **atravessa** a
/// vizinhança a que era suposto reagir, e as três regras de Reynolds passam a ler um mundo em
/// que ele nunca esteve.
fn nearest_neighbour(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    let Some(Column::Vec2(p)) = s.get("P") else {
        return 0.0;
    };
    if p.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    for (i, a) in p.iter().enumerate() {
        let mut best = f32::MAX;
        for (j, b) in p.iter().enumerate() {
            if i != j {
                best = best.min(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt());
            }
        }
        total += best;
    }
    total / p.len() as f32
}

/// Corre o bando e devolve a distância média ao vizinho no fim.
fn cohesion(reg: &NodeRegistry, speed: f32, radius: f32, dt: f64, ticks: usize, seed: f32) -> f32 {
    let mut g = Graph::new();
    let b = g.add_node("motion.boids");
    g.set_param(b, "count", 64.0);
    g.set_param(b, "radius", radius);
    g.set_param(b, "seed", seed);
    g.set_param(b, "seek", 1.0);
    g.set_param(b, "cohesion", 1.0);
    g.set_param(b, "max_speed", speed);
    wire(&mut g, b, 0, b, 2, true);
    g.validate(reg).expect("bem-tipado");
    let mut cook = Cook::new();
    let mut last = 0.0f32;
    for k in 0..ticks {
        let t = k as f64 * dt;
        let out = cook.cook(&g, reg, b, t).expect("coza");
        if k == ticks - 1 {
            last = nearest_neighbour(out[0].as_stream());
        }
        cook.advance_tick(&g, reg, t).expect("avanca");
    }
    last
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn how_far_can_a_bird_step_before_it_stops_flocking() {
    let reg = registry();
    let radius = 2.0f32; // o default
    eprintln!(
        "\n[boids] a distancia media ao VIZINHO ao fim de 200 tiques, com raio de percepcao {radius}\n"
    );
    eprintln!(
        "  {:>8}  {:>10}  {:>12}  {:>10}  {:>16}",
        "max_speed", "dt", "passo/raio", "mediana", "espalhamento"
    );
    // ⚠️⚠️ **SEMENTES, e é a correcção que a auditoria de 2026-08-27 forçou.** A 1.ª versão
    // desta sonda fazia **uma** corrida por célula, e um bando é caótico: três corridas
    // fisicamente idênticas (~`0,1` por tique) deram `0,653`, `0,763` e `0,863` — **17% de
    // espalhamento**, causado por perturbações de `2,4e-8` (o `playhead` é `f32`, e em
    // `dt = 0,1` o grampo morde em 12 de 20 tiques, tirando no máximo `2,4e-8` de cada vez).
    // ⇒ **o `1,284 → 0,763` que o doc-comment do `MAX_DT` publicava a três decimais era da
    // ordem do próprio ruído.** *Um número de sistema caótico sem barra de dispersão não é uma
    // medição — é uma amostra.*
    const SEEDS: [f32; 5] = [1.0, 2.0, 3.0, 4.0, 5.0];
    for speed in [4.0f32, 20.0, 100.0] {
        for dt in [1.0 / 60.0, 0.1, 0.25, 0.5] {
            #[expect(clippy::cast_possible_truncation, reason = "a razao e' de leitura")]
            let ratio = speed * dt as f32 / radius;
            let mut v: Vec<f32> = SEEDS
                .iter()
                .map(|s| cohesion(&reg, speed, radius, dt, 200, *s))
                .collect();
            v.sort_by(f32::total_cmp);
            let (lo, med, hi) = (v[0], v[v.len() / 2], v[v.len() - 1]);
            eprintln!(
                "  {speed:>8.0}  {dt:>10.4}  {ratio:>12.2}  {med:>10.3}  {:>16}",
                format!("{lo:.3}..{hi:.3}")
            );
        }
    }
    eprintln!(
        "\n  LEITURA: a coluna `passo/raio` e' `max_speed·dt / radius` -- quanto de uma
  vizinhanca um passaro atravessa num tique. ⚠️ Acima de `1` ele SALTA por cima da
  vizinhanca a que devia reagir.

  ⛔⛔ ESTA TABELA NAO PODE REFUTAR NADA ACIMA DO GRAMPO. O `MAX_DT` esta' DENTRO do
  sistema medido -- o `dt` so' chega ao passo depois de `clamp(0, MAX_DT)` --, entao
  com `max_speed = 20` ou `100` as linhas de `0,25` e `0,5` sao a linha de `0,1`
  REPETIDA. Uma hipotese sobre o que acontece acima do grampo precisa de uma corrida
  com o grampo LEVANTADO, e ela nao esta' aqui.
  ⚠️ E compare medianas com o ESPALHAMENTO ao lado: uma diferenca menor que a largura
  da coluna `espalhamento` nao e' um efeito, e' a mesma corrida outra vez."
    );
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_is_the_boids_max_dt_a_ceiling_of() {
    let reg = registry();
    eprintln!("\n[boids] a maior excursao em 120 tiques — o bando nasce num raio de 3\n");
    eprint!("  {:>10}", "seek\\dt");
    let dts = [1.0 / 120.0, 1.0 / 60.0, 1.0 / 30.0, 0.05, 0.1, 0.2, 0.5];
    for d in dts {
        eprint!("  {d:>8.4}");
    }
    eprintln!();
    for seek in [1.0f32, 4.0, 16.0, 64.0] {
        eprint!("  {seek:>10.0}");
        for d in dts {
            let e = run_boids(&reg, seek, d, 120);
            if e.is_finite() && e < 1e6 {
                eprint!("  {e:>8.2}");
            } else {
                eprint!("  {:>8}", "EXPLODE");
            }
        }
        eprintln!();
    }
    eprintln!(
        "\n  LEITURA: o `MAX_DT` grampa o `dt` que chega ao passo, entao a coluna em que a
  excursao dispara e' o teto de VERDADE deste no'. ⚠️ O grampo de hoje e' `0,1` -- se a
  linha ja' estiver ruim ali, o numero e' um palpite e nao um teto."
    );
}

// ---------------------------------------------------------------------------
// `motion.wave` — a constante INERTE
// ---------------------------------------------------------------------------

/// Um tanque com a fonte do centro, cozido com saltos de relógio de tamanhos diferentes.
fn run_wave(reg: &NodeRegistry, jump: f64, ticks: usize) -> f32 {
    let mut g = Graph::new();
    let w = g.add_node("motion.wave");
    for (k, v) in [
        ("rows", 15.0),
        ("cols", 15.0),
        ("speed", 0.4),
        ("damping", 0.0),
    ] {
        g.set_param(w, k, v);
    }
    // ⚠️⚠️ **A fonte tem de ser CONSTANTE, e a 1.ª versão desta sonda usou um `value.lfo`.**
    // Com saltos de relógio que são múltiplos do período dele, a senoide é amostrada sempre na
    // MESMA fase — e nas linhas grandes ela caía em zero, dando `max |h| = 0,000000`. Isso
    // media o ALIASING da fixtura, não o grampo. Um LFO de amplitude zero é a constante mais
    // barata do catálogo.
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "amplitude", 0.0);
    g.set_param(lfo, "offset", 1.0);
    wire(&mut g, lfo, 0, w, 0, false);
    wire(&mut g, w, 0, w, 1, true);
    g.validate(reg).expect("bem-tipado");
    let mut cook = Cook::new();
    let mut last = 0.0f32;
    for k in 0..ticks {
        let t = k as f64 * jump;
        let out = cook.cook(&g, reg, w, t).expect("coza");
        if k == ticks - 1 {
            let s = out[0].as_stream();
            last = match s.get("wave_h") {
                Some(Column::Scalar(h)) => h.iter().map(|x| x.abs()).fold(0.0f32, f32::max),
                _ => 0.0,
            };
        }
        cook.advance_tick(&g, reg, t).expect("avanca");
    }
    last
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_the_waves_max_dt_shape_anything_at_all() {
    let reg = registry();
    eprintln!("\n[wave] o maior |h| ao fim de 60 tiques, variando o SALTO de relogio\n");
    eprintln!(
        "  {:>10}  {:>12}  vs o grampo antigo (0,1)",
        "salto (s)", "max |h|"
    );
    for jump in [1.0 / 60.0, 0.05, 0.1, 0.5, 2.0, 30.0] {
        let h = run_wave(&reg, jump, 60);
        let rel = if jump > 0.1 { "ACIMA" } else { "abaixo" };
        eprintln!("  {jump:>10.4}  {h:>12.6}  {rel}");
    }
    eprintln!(
        "\n  LEITURA: se as linhas ACIMA do grampo derem o MESMO numero das de baixo, o
  `MAX_DT` do `motion.wave` nao molda passo nenhum -- o passo do leapfrog e' FIXO e o
  `dt` so' responde a pergunta *passou tempo?*. Nesse caso ele nao e' um teto por medir: e' uma
  constante que afirma estabilidade e nao participa dela."
    );
}
