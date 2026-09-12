//! Os gates da cena `=56` — a volta completa de uma coluna nomeada.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Coze a cena e devolve, por banda, o `size.x` de cada elemento.
fn sizes() -> Vec<Vec<f32>> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_column_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            let v = cook.cook(&doc.graph, &reg, *s, 0.0).expect("a banda coze");
            match v[0].as_stream().get("size") {
                Some(Column::Vec2(p)) => p.iter().map(|q| q[0]).collect(),
                _ => panic!("sem size"),
            }
        })
        .collect()
}

fn span(v: &[f32]) -> f32 {
    v.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        - v.iter().copied().fold(f32::INFINITY, f32::min)
}

/// **A CENA MONTA AS QUATRO BANDAS E CADA UMA TERMINA NUM OUTPUT.**
#[test]
fn the_scene_builds_four_bands_that_all_end_in_an_output() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_column_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas, dois pares");
    for s in &sinks {
        assert_eq!(
            doc.graph.node(*s).map(|n| n.type_name.as_str()),
            Some("motion.output"),
            "toda banda termina num Output"
        );
    }
    assert_eq!(band_labels().count(), 4, "um rótulo por banda");
}

/// **A VOLTA FECHA, E O CONTROLE PROVA QUE ELA PRECISA DO ESCRITOR.**
#[test]
fn the_round_trip_closes_and_the_control_is_flat() {
    let s = sizes();
    let (control, full) = (span(&s[0]), span(&s[1]));
    assert!(
        control < 1e-4,
        "o CONTROLE: sem escritor o leitor não acha nada, e a fileira é PLANA ({control:.6})"
    );
    assert!(
        full > 0.3,
        "com o escritor a fileira CRESCE ao longo do índice ({full:.4})"
    );
}

/// **O NOME É UMA PALAVRA, E A PALAVRA ERRADA É O SILÊNCIO.**
///
/// ⚠️ As duas metades: a banda 3 (o nome certo) tem de crescer e a 4 (o nome
/// errado) tem de ser plana. Sem a primeira, *"a 4 é plana"* seria satisfeito por
/// um escritor que nunca escreveu.
#[test]
fn reading_a_name_nobody_wrote_is_silence() {
    let s = sizes();
    let (right, wrong) = (span(&s[2]), span(&s[3]));
    assert!(right > 0.3, "o nome certo cresce ({right:.4})");
    assert!(
        wrong < 1e-4,
        "e o nome que ninguém escreveu volta a ser plano ({wrong:.6})"
    );
}

/// **A SONDA** — imprime os números que a mensagem do smoke cita.
#[test]
#[ignore = "sonda"]
fn measure_what_the_scene_shows() {
    let s = sizes();
    let (col, wrong) = names();
    println!("cena 56 (coluna `{col}`, nome errado `{wrong}`):");
    for (i, label) in band_labels() {
        println!(
            "  {}. {label}\n       tamanho {:.3} .. {:.3}   (vão {:.4})",
            i + 1,
            s[i].iter().copied().fold(f32::INFINITY, f32::min),
            s[i].iter().copied().fold(f32::NEG_INFINITY, f32::max),
            span(&s[i])
        );
    }
}
