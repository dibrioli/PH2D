//! **DE QUE É O TETO DE UM PASSO DE INTEGRAÇÃO** — a medição que os dois `MAX_DT` nunca tiveram
//! (bloco Z, doc [91](../../../docs/Motion%20Nodes/91_os_tetos_que_ninguem_mediu.md); células das
//! folhas 13 e 17 da conferência).
//!
//! ## O que estava escrito, e o que ninguém tinha medido
//!
//! Há **duas** constantes no catálogo com o mesmo nome, o mesmo papel e valores diferentes:
//!
//! | nó | `MAX_DT` | o que o doc-comment dele afirma |
//! |---|---|---|
//! | `motion.integrate` | `0,100` | *"guards a pathological playhead jump from becoming one giant unstable step"* |
//! | `sim.step` | `0,050` | *"otherwise arrives as one enormous `dt` and the sim explodes"* |
//!
//! As duas afirmam **estabilidade**, nenhuma traz medição, e elas **discordam por 2×** sem que
//! nada diga porquê. Um artista com uma zona de simulação ao lado de um integrador vê, depois de
//! um scrub, os dois avançarem **o dobro um do outro**.
//!
//! ## O que a medição diz
//!
//! Este arquivo mede o laço real (`motion.grid → motion.integrate`, com o `pre` de volta por uma
//! `force.attractor`), varrendo `dt` até ao grampo e `strength` até ao fim do arrasto. É de onde
//! sai o número que o doc 91 cita, e é ele quem decide se `0,1` é um teto ou um palpite.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// `grid → integrate.rest` · `integrate.out --pre--> attractor → integrate.state`.
///
/// ⚠️ **A força é a `force.attractor` e não uma constante**, porque é a única do catálogo cuja
/// magnitude depende da POSIÇÃO — é ela que fecha a malha de realimentação, e é uma malha
/// fechada que diverge. Uma força constante acelera para sempre em linha recta, o que é grande
/// mas não é instável, e mediria outra coisa.
fn loop_graph(reg: &NodeRegistry, strength: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 3.0);
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);
    let int = g.add_node("motion.integrate");
    let att = g.add_node("force.attractor");
    g.set_param(att, "strength", strength);
    g.set_param(att, "radius", 8.0);
    g.set_param(att, "target_x", 0.0);
    g.set_param(att, "target_y", 0.0);
    for (from, fp, to, tp, delayed) in [
        (grid, 0u16, int, 0u16, false),
        (int, 0, att, 0, true),
        (att, 0, int, 1, false),
    ] {
        g.connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed,
        })
        .expect("o laco fecha");
    }
    g.validate(reg).expect("o grafo e' valido");
    (g, int)
}

/// `grid → zone` · `zone --pre--> attractor → sim.step → zone.state` — o laço fechado do OUTRO
/// integrador do catálogo, pela porta que o produto usa.
fn zone_graph(reg: &NodeRegistry, strength: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 3.0);
    g.set_param(grid, "cols", 3.0);
    g.set_param(grid, "gap_x", 0.5);
    g.set_param(grid, "gap_y", 0.5);
    let zone = g.add_node("sim.zone");
    let att = g.add_node("force.attractor");
    g.set_param(att, "strength", strength);
    g.set_param(att, "radius", 8.0);
    let st = g.add_node("sim.step");
    for (from, fp, to, tp, delayed) in [
        (grid, 0u16, zone, 0u16, false),
        (zone, 0, att, 0, true),
        (att, 0, st, 0, false),
        (st, 0, zone, 1, false),
    ] {
        g.connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed,
        })
        .expect("o laco fecha");
    }
    g.validate(reg).expect("o grafo e' valido");
    (g, zone)
}

/// A maior distância a que qualquer elemento chegou em `ticks` passos de `dt` — `f32::INFINITY`
/// quando algum deixou de ser finito.
fn excursion(reg: &NodeRegistry, strength: f32, dt: f64, ticks: usize) -> f32 {
    let (g, int) = loop_graph(reg, strength);
    walk(reg, &g, int, dt, ticks)
}

/// O mesmo, pelo laço da zona.
fn zone_excursion(reg: &NodeRegistry, strength: f32, dt: f64, ticks: usize) -> f32 {
    let (g, zone) = zone_graph(reg, strength);
    walk(reg, &g, zone, dt, ticks)
}

fn walk(reg: &NodeRegistry, g: &Graph, sink: NodeId, dt: f64, ticks: usize) -> f32 {
    let mut cook = Cook::new();
    let mut worst = 0.0f32;
    for k in 0..ticks {
        let t = k as f64 * dt;
        let out = cook.cook(g, reg, sink, t).expect("coze");
        if let Some(CookValue::Instances(s)) = out.first()
            && let Some(Column::Vec2(p)) = s.get("P")
        {
            for q in p {
                if !q[0].is_finite() || !q[1].is_finite() {
                    return f32::INFINITY;
                }
                worst = worst.max(q[0].hypot(q[1]));
            }
        }
        cook.advance_tick(g, reg, t).expect("o quadro fecha");
    }
    worst
}

/// **O passo que o produto DE FACTO deu**, lido pelo deslocamento e não pela constante.
///
/// Sob uma aceleração constante `a = 1` e partindo do repouso, um passo semi-implícito desloca
/// `a·dt²` — então `√(deslocamento)` **é** o `dt` efectivo. Isto lê o grampo pela porta do
/// produto: nenhuma constante privada é importada, e um grampo que o kernel aplicasse
/// diferentemente do que a fonte diz seria apanhado aqui.
fn effective_step(reg: &NodeRegistry, zone: bool, asked: f64) -> f32 {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 1.0);
    let wind = g.add_node("force.wind");
    g.set_param(wind, "strength", 1.0);
    g.set_param(wind, "angle", 0.0);
    g.set_param(wind, "gust", 0.0);
    let sink = if zone {
        let z = g.add_node("sim.zone");
        let st = g.add_node("sim.step");
        g.set_param(st, "damping", 1.0);
        for (from, fp, to, tp, delayed) in [
            (grid, 0u16, z, 0u16, false),
            (z, 0, wind, 0, true),
            (wind, 0, st, 0, false),
            (st, 0, z, 1, false),
        ] {
            g.connect(Edge {
                from: (from, fp),
                to: (to, tp),
                delayed,
            })
            .expect("o laco fecha");
        }
        z
    } else {
        let int = g.add_node("motion.integrate");
        for (from, fp, to, tp, delayed) in [
            (grid, 0u16, int, 0u16, false),
            (int, 0, wind, 0, true),
            (wind, 0, int, 1, false),
        ] {
            g.connect(Edge {
                from: (from, fp),
                to: (to, tp),
                delayed,
            })
            .expect("o laco fecha");
        }
        int
    };
    g.validate(reg).expect("o grafo e' valido");
    let read = |cook: &mut Cook, t: f64| -> [f32; 2] {
        let out = cook.cook(&g, reg, sink, t).expect("coze");
        match out.first() {
            Some(CookValue::Instances(s)) => match s.get("P") {
                Some(Column::Vec2(p)) => p[0],
                _ => [0.0, 0.0],
            },
            _ => [0.0, 0.0],
        }
    };
    let mut cook = Cook::new();
    // ⚠️ **DOIS tiques de aquecimento, e o segundo não é decoração:** a entrada de estado da
    // `sim.zone` é gerida pelo motor por uma aresta `pre`, então na 1.ª cozedura ela ainda não
    // circulou e o `sim_t` de ninguém está carimbado — o passo seguinte media **zero**, e a
    // primeira versão desta sonda leu isso como *"o grampo é 0"*. Os dois correm em `t = 0`, ou
    // seja com `dt = 0`: eles preparam o estado sem mover uma peça.
    let mut seed = [0.0f32, 0.0];
    for _ in 0..2 {
        seed = read(&mut cook, 0.0);
        cook.advance_tick(&g, reg, 0.0).expect("o quadro fecha");
    }
    let moved = read(&mut cook, asked);
    (moved[0] - seed[0]).abs().sqrt()
}

/// **O GRAMPO QUE O PRODUTO APLICA É O MEDIDO — nos DOIS integradores.**
///
/// ⚠️ **As duas metades são obrigatórias.** Só *"um passo pequeno passa"* passaria sem grampo
/// nenhum; só *"um salto é grampeado"* passaria com o grampo velho de `0,1`, que a medição do
/// [`measure_the_step_that_a_closed_loop_survives`] mostra deixar a mesma cena chegar a **127**
/// vezes o raio em que nasceu.
#[test]
fn the_clamp_both_integrators_apply_is_the_measured_step() {
    let reg = registry();
    /// O joelho medido: o maior `dt` em que TODO passo até ele dá a mesma resposta.
    const KNEE: f32 = 0.03;
    for zone in [false, true] {
        let who = if zone {
            "sim.zone/sim.step"
        } else {
            "motion.integrate"
        };
        // Um passo PEQUENO passa inteiro — o grampo não morde em regime.
        let small = effective_step(&reg, zone, 0.01);
        assert!(
            (small - 0.01).abs() < 1e-3,
            "{who}: um passo de 0,01 tem de passar inteiro, deu {small}"
        );
        // E um SCRUB de cinco segundos é absorvido no joelho, não tomado.
        let scrub = effective_step(&reg, zone, 5.0);
        eprintln!("{who}: pedido 5,000 s -> passo efectivo {scrub:.4}");
        assert!(
            (scrub - KNEE).abs() < 1e-3,
            "{who}: um scrub tem de ser grampeado em {KNEE}, deu {scrub}"
        );
    }
}

/// **A MEDIÇÃO, impressa** — a tabela que o doc 91 cita.
///
/// ```text
/// cargo test -p ph2d-node-registry-init --test integrator_ceilings measure -- --nocapture
/// ```
#[test]
fn measure_the_step_that_a_closed_loop_survives() {
    let reg = registry();
    let dts = [1.0 / 60.0, 1.0 / 30.0, 0.05, 0.075, 0.1];
    eprintln!("excursao maxima (raio de mundo) em 240 passos -- a grelha nasce dentro de 1,0");
    eprint!("{:>10}", "strength");
    for dt in dts {
        eprint!("{:>12}", format!("dt={dt:.4}"));
    }
    eprintln!();
    for strength in [5.0f32, 10.0, 20.0, 40.0] {
        eprint!("{strength:>10}");
        for dt in dts {
            let e = excursion(&reg, strength, dt, 240);
            if e.is_finite() {
                eprint!("{e:>12.2}");
            } else {
                eprint!("{:>12}", "NAO-FINITO");
            }
        }
        eprintln!();
    }
    // E o OUTRO integrador, na mesma malha: `sim.step` dentro da zona dele.
    eprintln!("o mesmo laco pelo `sim.zone`/`sim.step` (o grampo dele e' 0,05):");
    for strength in [5.0f32, 10.0, 20.0, 40.0] {
        eprint!("{strength:>10}");
        for dt in dts {
            let e = zone_excursion(&reg, strength, dt, 240);
            if e.is_finite() {
                eprint!("{e:>12.2}");
            } else {
                eprint!("{:>12}", "NAO-FINITO");
            }
        }
        eprintln!();
    }
    // O controle: sem força não há excursão nenhuma. Sem esta linha a tabela acima podia estar a
    // medir o grid parado e leria igual de fácil.
    assert!(
        excursion(&reg, 0.0, 1.0 / 60.0, 240) < 1.01,
        "com strength zero a grelha nao sai do sitio"
    );
}

/// **ONDE O JOELHO ESTÁ** — a varredura fina entre o quadro perdido e o grampo de hoje.
///
/// O critério é escrito, não sentido: *um passo legítimo não muda a RESPOSTA, só a resolução*.
/// A régua é a excursão em regime (`dt = 1/60`) e a barra é **o dobro dela** — acima disso a
/// grelha já não está onde nasceu, e o que se vê no ecrã não é a mesma simulação mais grossa, é
/// outra.
///
/// ⚠️ **A RESPOSTA É RESSONANTE, E A PRIMEIRA VERSÃO DESTA SONDA LEU-A AO CONTRÁRIO.** Ela
/// procurava o **primeiro cruzamento** — e devolveu `0,0300`, quando `0,0333` (um quadro perdido
/// a 30 fps, o caso mais comum de todos) mede `0,89` e passa folgadamente. Um laço fechado com
/// uma força central tem ressonâncias: a excursão **não é monótona em `dt`**, então *o primeiro
/// cruzamento é uma ressonância, não a fronteira*. A pergunta certa não é *"este passo
/// sobrevive?"* mas ***"todos os passos até este sobrevivem?"*** — e o prefixo-máximo, sendo
/// monótono por construção, é a única forma de a fazer.
#[test]
fn measure_where_the_closed_loop_stops_being_the_same_answer() {
    let reg = registry();
    let steady = excursion(&reg, 40.0, 1.0 / 60.0, 240);
    let bar = steady * 2.0;
    eprintln!("regime (dt=1/60, strength=40): {steady:.3} · barra = 2x = {bar:.3}");
    let (mut prefix, mut last_ok, mut broke) = (0.0f32, 0.0f64, false);
    for k in 1..=48 {
        let dt = f64::from(k) * 0.0025;
        let e = excursion(&reg, 40.0, dt, 240);
        prefix = if e.is_finite() {
            prefix.max(e)
        } else {
            f32::INFINITY
        };
        eprintln!(
            "  dt={dt:.4} (1/{:>5.1}): excursao {e:>8.2}  prefixo {prefix:.2}",
            1.0 / dt
        );
        if prefix <= bar && !broke {
            last_ok = dt;
        } else {
            broke = true;
        }
    }
    eprintln!(
        "maior dt em que TODO passo ate' ele da' a mesma resposta: {last_ok:.4} (1/{:.1})",
        1.0 / last_ok
    );
    assert!(last_ok >= 1.0 / 60.0, "o passo de regime tem de sobreviver");
}

/// **ATÉ ONDE A FORÇA É HONRADA** — o teto do `strength`, ao relógio da casa.
///
/// ⚠️ **Este é o param que a sonda `what_the_corpus_authors_and_no_one_can_type` acusou e que o
/// gate irmão `param_ceilings` recusou tomar**: ele NÃO é limitado pela precisão. Uma força que
/// faz o laço divergir não é honrada — a guarda de finitude do `motion.integrate` repõe o
/// elemento, e o artista vê a peça a saltar para a pose de repouso. O teto é onde isso começa.
#[test]
fn measure_the_force_the_fixed_tick_survives() {
    let reg = registry();
    let dt = 1.0 / ph2d_core::time::DEFAULT_HZ;
    let steady = excursion(&reg, 40.0, dt, 240);
    let bar = steady * 2.0;
    eprintln!("ao relogio da casa (dt=1/{}): barra = {bar:.3}", 1.0 / dt);
    // Prefixo-máximo, pela MESMA razão do gate acima: a excursão é ressonante em `strength`
    // também (120 mede 1,63 e 160 mede 0,89), então o primeiro cruzamento não é a fronteira.
    let (mut prefix, mut last_ok) = (0.0f32, 0.0f32);
    for k in 1..=40 {
        let s = k as f32 * 40.0;
        let e = excursion(&reg, s, dt, 240);
        prefix = if e.is_finite() {
            prefix.max(e)
        } else {
            f32::INFINITY
        };
        eprintln!(
            "  strength {s:>6}: excursao {:>10}  prefixo {prefix:.2}",
            if e.is_finite() {
                format!("{e:.2}")
            } else {
                "NAO-FINITO".into()
            }
        );
        if prefix <= bar {
            last_ok = s;
        } else {
            break;
        }
    }
    eprintln!("maior strength em que TODA forca ate' ela aguenta: {last_ok}");
    assert!(last_ok >= 40.0, "o fim do arrasto tem de ser estavel");
}
