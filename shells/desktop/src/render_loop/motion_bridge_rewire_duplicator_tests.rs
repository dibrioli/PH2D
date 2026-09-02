//! **O REPORT DO DUPLICATOR, 2026-09-01** — as sondas e os gates que saíram do report do
//! Enio, cortados do `motion_bridge_rewire_tests.rs` no teto de LOC do HR-18 (600).
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, não por contagem:** o irmão responde *«mexer num fio
//! faz o que se pede?»* (mover a ponta, inserir um nó, recusar um tipo que não encaixa) e
//! este responde *«o que acontece quando o nó inserido é o `motion.duplicator`?»* — a porta
//! em que o fio cai, as colunas que sobrevivem ao carimbo, e se a simulação a jusante ainda
//! anda. O primeiro cresce quando o gesto muda; este cresce quando um nó tem lei própria.
//!
//! Mecanismo e tabelas: [doc 98 §4.2 e §4.3](../../../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md).

use super::*;
use crate::motion_state::MotionState;
use ph2d_editor::ToastQueue;
use ph2d_nodegraph::graph::Edge;

/// ⛔⛔⛔ **O REPORT DO ENIO, 2026-09-01** — *«o simples facto de tentar colocar um duplicator
/// logo após o Emitter já quebra a cena … trava … automaticamente o fio do emitter entra no
/// input errado do duplicator (o da shape ou objeto)»*.
///
/// A sonda monta o gesto REAL (`splice_node`, o mesmo que o menu chama), cozinha, e imprime o
/// que sai: a porta em que o fio caiu, a contagem emitida e o relógio.
///
/// `cargo test -p ph2d-host-desktop --release --bins -- --ignored --nocapture the_enio_duplicator_after_emitter`
#[test]
#[ignore = "sonda de reproducao, nao um gate"]
fn the_enio_duplicator_after_emitter() {
    use ph2d_nodegraph::cook::Cook;
    let mut motion = MotionState::new();
    // emissor → saída, o mínimo que reproduz «logo após o Emitter».
    let g = &mut motion.doc.graph;
    let em = g.add_node("motion.emitter".to_string());
    let out = g.add_node("motion.output".to_string());
    g.connect(Edge {
        from: (em, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("emitter -> output");
    let cook_it = |motion: &mut MotionState, rotulo: &str| {
        let mut cook = Cook::new();
        let alvo = motion
            .doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == "motion.output")
            .map(|n| n.id)
            .expect("output");
        let t = std::time::Instant::now();
        let r = cook.cook(&motion.doc.graph, &motion.registry, alvo, 0.5);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        let n = match r {
            Ok(v) => v.first().map_or(usize::MAX, |c| c.as_stream().count()),
            Err(_) => usize::MAX,
        };
        eprintln!("  {rotulo:<34} │ {n:>10} linhas │ {ms:>8.1} ms");
    };
    cook_it(&mut motion, "ANTES (so' o emissor)");

    let mut toasts = ToastQueue::default();
    splice_node(
        &mut motion,
        &mut toasts,
        out.0,
        0,
        "motion.duplicator",
        0.0,
        0.0,
    );
    let dup = motion
        .doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "motion.duplicator")
        .map(|n| n.id)
        .expect("o duplicator entrou no grafo");
    let porta = motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == dup && e.from.0 == em)
        .map(|e| e.to.1);
    eprintln!(
        "  => o fio do emissor caiu na porta {porta:?} ({})",
        match porta {
            Some(0) => "shape -- o que o Enio diz estar ERRADO",
            Some(1) => "points",
            _ => "NENHUMA",
        }
    );
    cook_it(&mut motion, "DEPOIS do splice");
    // ⚠️ **A cena ficar vazia so' e' aceitavel se o app DISSER porque.** O `Deficit::MissingInput`
    // ja' existe e o duplicator declara `shape`/`points` como obrigatorias -- resta saber se o
    // selo ⚠ de facto aparece neste estado, que e' o que o artista ve'.
    let badges = crate::render_loop::motion_bridge::heal::inert_reaching_output(&motion);
    eprintln!(
        "  => selos ⚠ no grafo: {badges:?} (node_help={})",
        motion.node_help_enabled
    );
    for d in ph2d_motion_diagnose::diagnose(&motion.doc.graph, &motion.registry) {
        eprintln!("      no' {:?} -> {:?}", d.node, d.deficit);
    }

    // E a forma que o artista de facto quer: o emissor nas DUAS portas (carimbar a
    // partícula em cada partícula) -- que e' onde o produto satura.
    motion
        .doc
        .graph
        .connect(Edge {
            from: (em, 0),
            to: (dup, if porta == Some(0) { 1 } else { 0 }),
            delayed: false,
        })
        .expect("a segunda porta");
    cook_it(&mut motion, "com o emissor nas DUAS portas");
}

/// ⭐⭐⭐ **O FIO ENTRA PELA PORTA QUE O TIPO DECLARA** — o gate do report do Enio de
/// 2026-09-01 (*«automaticamente o fio do emitter entra no input errado do duplicator»*).
///
/// ⚠️ **As duas metades mordem, e a segunda é a que impede um estrago em massa:** o
/// `motion.duplicator` DECLARA a porta 1 (`points`) e tem de a receber; o `motion.scale`, que
/// não declara nada, tem de continuar a receber a `0` — os 133 tipos que não declaram nada
/// ficam byte-idênticos, que é a única razão por que esta mudança é segura.
///
/// ⛔ **O tipo não podia acusar isto:** as duas entradas do duplicator são `INST_VEC2`, então
/// o `validate` do trial aceita as duas e o erro é SILENCIOSO. É por isso que a resposta é
/// side-metadata declarada, e não uma inferência sobre tipos.
#[test]
fn a_spliced_wire_enters_the_port_the_type_declares() {
    let porta_de = |ty: &str| -> Option<u16> {
        let mut motion = MotionState::new();
        let g = &mut motion.doc.graph;
        let src = g.add_node("motion.grid".to_string());
        let out = g.add_node("motion.output".to_string());
        g.connect(Edge {
            from: (src, 0),
            to: (out, 0),
            delayed: false,
        })
        .expect("grid -> output");
        let mut toasts = ToastQueue::default();
        splice_node(&mut motion, &mut toasts, out.0, 0, ty, 0.0, 0.0);
        let novo = motion
            .doc
            .graph
            .nodes()
            .iter()
            .find(|n| n.type_name == ty)
            .map(|n| n.id)?;
        motion
            .doc
            .graph
            .edges()
            .iter()
            .find(|e| e.to.0 == novo && e.from.0 == src)
            .map(|e| e.to.1)
    };
    assert_eq!(
        porta_de("motion.duplicator"),
        Some(1),
        "o duplicator declara `points` (porta 1) como o fio principal — carimbar a FORMA em \
         cada ponto significa que o fluxo que atravessa a cadeia são os pontos, e pô-lo no \
         `shape` mete-o no lado que entra no PRODUTO"
    );
    assert_eq!(
        porta_de("motion.scale"),
        Some(0),
        "um tipo que NÃO declara nada continua a receber na porta 0 — sem isto a mudança \
         mexia nos 133 tipos que estavam certos"
    );
}

/// ⛔⛔⛔ **«O nó entrou em points corretamente mas a simulação morreu»** (Enio, 2026-09-01,
/// com foto: `Shape(1) + Emitter(4096) → Duplicator → Integrate.rest → Output`).
///
/// A sonda monta a MESMA cadeia de simulação da cena `=5` (a fonte do emissor) e mede o
/// deslocamento máximo depois de N tiques, **com e sem** o duplicator no meio. E imprime as
/// colunas que chegam ao integrador — que é onde a resposta está.
///
/// `cargo test -p ph2d-host-desktop --release --bins -- --ignored --nocapture the_enio_sim_died_after_the_duplicator`
#[test]
#[ignore = "sonda de reproducao, nao um gate"]
fn the_enio_sim_died_after_the_duplicator() {
    use ph2d_nodegraph::cook::Cook;
    const TIQUES: u64 = 40;
    let marchar = |com_dup: bool| -> (f32, Vec<String>, usize) {
        let mut motion = MotionState::new();
        let g = &mut motion.doc.graph;
        let em = g.add_node("motion.emitter".to_string());
        g.set_param(em, "rate", 400.0);
        g.set_param(em, "max", 4096.0);
        let ig = g.add_node("motion.integrate".to_string());
        let grav = g.add_node("force.wind".to_string());
        g.set_param(grav, "angle", 270.0);
        g.set_param(grav, "strength", 9.8);
        g.set_param(grav, "gust", 0.0);
        let out = g.add_node("motion.output".to_string());
        // A entrada do `rest`: o emissor directo, ou o duplicator com uma FORMA de 1.
        let rest_from = if com_dup {
            // ⚠️ **Uma forma que de facto EMITE.** A 1.ª versão usava `motion.make_point`, que
            // sem as portas de valor ligadas emite ZERO — e com `ns == 0` o duplicator devolve
            // a forma tal e qual, ou seja a fixtura NÃO CONTINHA o fenómeno e mediu a cura
            // como inútil ([[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]).
            // A foto do Enio diz `Shape · 1 inst`.
            let shape = g.add_node("motion.grid".to_string());
            g.set_param(shape, "rows", 1.0);
            g.set_param(shape, "cols", 1.0);
            let dup = g.add_node("motion.duplicator".to_string());
            g.connect(Edge {
                from: (shape, 0),
                to: (dup, 0),
                delayed: false,
            })
            .expect("shape");
            g.connect(Edge {
                from: (em, 0),
                to: (dup, 1),
                delayed: false,
            })
            .expect("points");
            dup
        } else {
            em
        };
        for (from, to, port, delayed) in [
            (rest_from, ig, 0u16, false),
            (ig, grav, 0, true), // a realimentacao que o artista nunca desenha
            (grav, ig, 1, false),
            (ig, out, 0, false),
        ] {
            g.connect(Edge {
                from: (from, 0),
                to: (to, port),
                delayed,
            })
            .expect("fio");
        }
        let mut cook = Cook::new();
        let mut maior = 0.0f32;
        let mut colunas = Vec::new();
        let mut n = 0;
        for t in 0..=TIQUES {
            let ph = t as f64 * (1.0 / 60.0);
            if cook
                .cook(&motion.doc.graph, &motion.registry, out, ph)
                .is_err()
            {
                break;
            }
            cook.advance_tick(&motion.doc.graph, &motion.registry, ph)
                .ok();
            if t == TIQUES {
                if let Some(v) = cook.peek(rest_from) {
                    let s = v[0].as_stream();
                    colunas = s.columns().map(|(k, _)| k.clone()).collect();
                }
                if let Some(v) = cook.peek(out) {
                    let s = v[0].as_stream();
                    n = s.count();
                    if let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = s.get("P") {
                        maior = p.iter().map(|q| q[1].abs()).fold(0.0, f32::max);
                    }
                }
            }
        }
        (maior, colunas, n)
    };
    for (rotulo, com) in [("SEM duplicator", false), ("COM duplicator", true)] {
        let (queda, cols, n) = marchar(com);
        eprintln!("  {rotulo:<16} │ {n:>5} linhas │ queda maxima {queda:>8.3}");
        eprintln!("      colunas que chegam ao `rest`: {cols:?}");
    }
    eprintln!("  (a gravidade a 9,8 durante 40 tiques tem de mover as particulas MUITO)");
}

/// ⭐⭐⭐ **UMA SIMULAÇÃO SOBREVIVE A UM DUPLICATOR NO MEIO** — o gate do report do Enio de
/// 2026-09-01 (*«o nó entrou em points corretamente mas a simulação morreu, parou de
/// funcionar»*), e ele mede o SINTOMA DO ARTISTA, não o mecanismo.
///
/// ⛔⛔ **Por que o sintoma e não as colunas:** havia gates a fio sobre o `transfer` — e todos
/// verdes — enquanto a cadeia inteira estava morta. Eles perguntavam *«o modo faz o que diz?»*
/// e nenhum perguntava *«a simulação ainda anda?»*. Uma coluna perdida só é um defeito porque
/// alguém a jusante precisa dela: sem `vel` não há o que integrar, sem `id` o integrador não
/// reconhece a partícula do tique anterior.
///
/// A cadeia é a da cena `=5` (a fonte do emissor) com o duplicator enfiado no `rest`, e a
/// barra é o próprio caminho SEM ele: as duas têm de cair o MESMO — o duplicator carimba uma
/// forma de um elemento em cada partícula, logo não muda a física de nenhuma.
#[test]
fn a_simulation_survives_a_duplicator_in_the_middle() {
    use ph2d_nodegraph::cook::Cook;
    const TIQUES: u64 = 40;
    let queda = |com_dup: bool| -> (f32, usize) {
        let mut motion = MotionState::new();
        let g = &mut motion.doc.graph;
        let em = g.add_node("motion.emitter".to_string());
        g.set_param(em, "rate", 400.0);
        g.set_param(em, "max", 4096.0);
        let ig = g.add_node("motion.integrate".to_string());
        let grav = g.add_node("force.wind".to_string());
        g.set_param(grav, "angle", 270.0);
        g.set_param(grav, "strength", 9.8);
        g.set_param(grav, "gust", 0.0);
        let out = g.add_node("motion.output".to_string());
        let rest_from = if com_dup {
            let shape = g.add_node("motion.grid".to_string());
            g.set_param(shape, "rows", 1.0);
            g.set_param(shape, "cols", 1.0);
            let dup = g.add_node("motion.duplicator".to_string());
            g.connect(Edge {
                from: (shape, 0),
                to: (dup, 0),
                delayed: false,
            })
            .expect("shape");
            g.connect(Edge {
                from: (em, 0),
                to: (dup, 1),
                delayed: false,
            })
            .expect("points");
            dup
        } else {
            em
        };
        for (from, to, port, delayed) in [
            (rest_from, ig, 0u16, false),
            (ig, grav, 0, true),
            (grav, ig, 1, false),
            (ig, out, 0, false),
        ] {
            g.connect(Edge {
                from: (from, 0),
                to: (to, port),
                delayed,
            })
            .expect("fio");
        }
        let mut cook = Cook::new();
        let (mut maior, mut n) = (0.0f32, 0);
        for t in 0..=TIQUES {
            let ph = t as f64 * (1.0 / 60.0);
            cook.cook(&motion.doc.graph, &motion.registry, out, ph)
                .expect("a cadeia cozinha");
            cook.advance_tick(&motion.doc.graph, &motion.registry, ph)
                .ok();
        }
        if let Some(v) = cook.peek(out) {
            let s = v[0].as_stream();
            n = s.count();
            if let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = s.get("P") {
                maior = p.iter().map(|q| q[1].abs()).fold(0.0, f32::max);
            }
        }
        (maior, n)
    };
    let (sem, n_sem) = queda(false);
    let (com, n_com) = queda(true);
    assert!(
        sem > 0.1,
        "o CONTROLO tem de cair — se a cadeia sem duplicator ja' nao anda, este gate nao \
         mede o duplicator (queda {sem}, {n_sem} linhas)"
    );
    assert_eq!(
        n_com, n_sem,
        "uma forma de UM elemento carimba uma copia por particula"
    );
    assert!(
        (com - sem).abs() < 1e-4,
        "a simulacao MORREU com o duplicator no meio: caiu {com} contra {sem} sem ele. \
         Um duplicator que deita fora as colunas dos pontos tira o `vel` (nada a integrar) \
         e o `id` (o integrador nao reconhece a particula do tique anterior)."
    );
}
