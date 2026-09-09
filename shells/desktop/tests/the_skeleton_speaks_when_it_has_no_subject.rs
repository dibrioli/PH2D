//! ⭐⭐⭐ **UM CONTROLO DO ESQUELETO QUE MORRE EM SILÊNCIO DÁ O MESMO SINTOMA QUE UMA ROTA CORTADA.**
//!
//! ⛔⛔ Foi esse, letra por letra, o report do dono de 2026-09-07 (*«Add IK não funciona»*), cuja
//! causa era outra. O braço que diz *«nenhum osso em foco»* existe para separar *«o app recusou»* de
//! *«o botão está morto»* — e ele era uma **disjunção escrita à mão**: nasceu com `2` verbos, tinha
//! `8` quando a auditoria de 2026-09-08 o apanhou, e os `9` campos e as duas fileiras de chips nunca
//! lá entraram. *Uma cura escrita para os verbos que existiam não segue os que vêm.*
//!
//! ⚠️ **A cobertura da pergunta derivada é gateada na `ph2d-editor-core`**
//! (`every_skeleton_control_declares_whose_subject_it_is`). O que **só aqui** se pode medir é a
//! outra metade: *o dreno pergunta-a, e pergunta-a nos DOIS caminhos de evento* — um clique e um
//! campo numérico chegam por portas diferentes, e alimentar só uma deixa metade da seção calada.

const LOOP: &str = include_str!("../src/render_loop/mod.rs");

/// **O fonte sem comentários** — um censo textual que não os tira mente nos dois sentidos: uma nota
/// que cita a chamada conta como chamada, e uma chamada comentada conta como viva.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O braço que fala lê a pergunta DERIVADA, e não uma lista.**
#[test]
fn the_arm_that_speaks_reads_the_derived_question() {
    let src = code_only(LOOP);
    assert!(
        src.contains("} else if pending_bone_needs_focus {"),
        "o braço «nenhum osso em foco» deixou de ler a pergunta derivada — se ele voltou a uma \
         disjunção escrita à mão, os controlos que vierem a seguir morrem calados, que é o report \
         de 2026-09-07"
    );
}

/// ⭐⭐⭐ **E ela é alimentada pelos DOIS caminhos de evento** — o clique e o campo numérico.
///
/// ⚠️ **A prova de mutação desta wave mostrou que era preciso:** apagar a alimentação do caminho dos
/// CAMPOS compila limpo, passa a suíte inteira, e deixa metade da seção (os nove campos) a morrer em
/// silêncio outra vez.
///
/// ⚠️ O piso é `2` e é literal **contra o vácuo**, não um censo da população: o que se afirma é que
/// há mais do que uma porta, e as portas são duas porque `PanelEvent` tem duas variantes que esta
/// seção usa.
#[test]
fn both_event_paths_feed_it() {
    let src = code_only(LOOP);
    let feeds: Vec<&str> = src
        .lines()
        .filter(|l| l.contains("pending_bone_needs_focus |="))
        .collect();
    assert!(
        feeds.len() >= 2,
        "só {} caminho(s) alimenta(m) a pergunta — um clique e um campo numérico chegam por portas \
         diferentes, e alimentar só uma deixa metade da secção calada",
        feeds.len()
    );
    // ⚠️ E cada alimentação vem da porta derivada: uma escrita `= true` num braço qualquer seria a
    // lista escrita à mão de volta, com outro nome.
    for (i, l) in feeds.iter().enumerate() {
        let seguinte = src
            .lines()
            .skip_while(|x| x != l)
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            seguinte.contains("needs_focused_bone"),
            "a alimentação {i} não vem da porta derivada: {seguinte}"
        );
    }
}
