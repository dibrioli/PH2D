//! **O MOLDE DO L-SYSTEM ESCREVE AS DUAS CAIXAS** — as quatro condições de UI para ele.
//!
//! Report do Enio (2026-08-28): *"Axiom e Rules não são nada intuitivos. Alguma soluções para
//! isso?"* ⇒ um selector de moldes, que é o que o L-System SOP do Houdini e o L-studio fazem.
//!
//! ⚠️ **A resposta não foi inventar uma sintaxe amigável**, e a razão é medida: `F[+F]F` é a
//! notação de Lindenmayer, e é ela que está no livro, nos tutoriais e em todo exemplo que o
//! artista vai encontrar. Trocá-la tornaria este nó incompatível com o conhecimento do mundo.

use super::params::{apply_lsystem_preset, param_value};
use crate::motion_state::MotionState;
use ph2d_node_source_lsystem as ls;

fn text_of(motion: &MotionState, nid: ph2d_nodegraph::graph::NodeId, key: &str) -> String {
    motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(key))
        .cloned()
        .unwrap_or_default()
}

/// **Todo molde tem rótulo, axioma e regras, e as regras COMPILAM.**
///
/// ⚠️⚠️ **Este gate CONTAVA ELEMENTOS (`count() > 3`) e a auditoria de 2026-08-29 mostrou que
/// ele não podia reprovar em molde nenhum, por construção.** A Koch passava com 3 126
/// elementos a medir **1 291 unidades de mundo** numa coluna de ~4; o Sprig passava com 16 a
/// desenhar uma linha de largura **exactamente 0,00**. *Uma contagem é a única grandeza que
/// SOBE com este defeito.* A contagem saiu; as réguas que de facto reprovam vivem em
/// [`presets_frame_themselves`](../../../../crates/ph2d-node-source-lsystem/tests/presets_frame_themselves.rs)
/// e medem o TAMANHO, os dois eixos da caixa, e a resposta ao `Angle`.
///
/// O que sobrevive aqui é a costura: a tabela é bem formada e cada texto de facto deriva.
#[test]
fn every_preset_is_a_grammar_that_actually_draws() {
    assert!(!ls::PRESETS.is_empty());
    for (k, p) in ls::PRESETS.iter().enumerate() {
        assert_eq!(p.label, ls::PRESET_LABELS[k], "o rotulo {k} discorda");
        assert!(!p.axiom.trim().is_empty() && !p.rules.trim().is_empty());
        // Deriva com o enquadramento que o próprio molde declara — a mesma coisa que o
        // `apply_lsystem_preset` escreve.
        let s = ls::probe_build(
            p.axiom,
            p.rules,
            p.generations,
            &[
                (ls::param::MODE, ls::MODE_GRAMMAR as f32),
                (ls::param::ANGLE, p.angle),
                (ls::param::STEP, p.step),
            ],
        );
        assert!(s.count() > 3, "o molde {} nao cresce", p.label);
    }
}

/// ⛔ **O gate `the_first_preset_is_what_a_fresh_node_already_is` MORREU, e a premissa dele é
/// que morreu primeiro.**
///
/// Ele comparava `PRESETS[0]` com o `DEFAULT_RULES` e o doc-comment nomeava o defeito que
/// queria impedir: *«um nó novo mostraria «Tree» seleccionado e uma gramática que não é a do
/// Tree — o painel a mentir sobre o próprio estado»*. Desde 2026-08-29 o `Mode` nasce
/// `Guided`, e «o que um nó novo já é» passou a ser a gramática DERIVADA dos sliders — outra
/// planta, **76 % mais alta**, medido. Os dois gates verdes (este e o
/// `converting_to_grammar_bakes_the_plant_the_sliders_were_making`, que assere
/// `assert_ne!(assado, DEFAULT_RULES)`) **provavam juntos o desencontro que o primeiro dizia
/// proibir**.
///
/// ⇒ A cura não é o gate: é o [`ls::PRESET_CUSTOM`] passar a ser o default do selector. É isso
/// que este gate afirma agora.
#[test]
fn a_fresh_node_names_no_preset_because_it_is_none_of_them() {
    let default = ls::MANIFEST
        .params
        .iter()
        .find(|p| p.name == ls::param::PRESET)
        .expect("o param existe")
        .default;
    assert_eq!(
        default.round() as usize,
        ls::PRESET_CUSTOM,
        "um no' novo abre em `Guided`, cuja gramatica nao e' a de molde nenhum"
    );
    // E o CONTROLE que torna isto necessário: a derivada do guiado de fábrica **não** é a
    // gramática do molde `0`. Se um dia voltar a ser, este gate tem de ser reconferido.
    let (_, guided) = ls::grammar_for(2.0, 1.0, 0.0, 0.0);
    assert_ne!(
        guided,
        ls::PRESETS[0].rules,
        "a derivada do guiado voltou a ser a do molde 0 — reveja qual e' o default honesto"
    );
}

/// ⭐ **Escolher um molde ESCREVE as duas caixas** — a costura que o torna um botão vivo.
#[test]
fn picking_a_preset_writes_both_text_boxes() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    for (k, p) in ls::PRESETS.iter().enumerate() {
        apply_lsystem_preset(&mut motion, n, "source.lsystem", k as f32);
        assert_eq!(
            text_of(&motion, n, ls::AXIOM_PARAM),
            p.axiom,
            "{}: axioma",
            p.label
        );
        assert_eq!(
            text_of(&motion, n, ls::RULES_PARAM),
            p.rules,
            "{}: regras",
            p.label
        );
    }
}

/// ⭐⭐⭐ **E ESCREVE O ENQUADRAMENTO** — a metade sem a qual sete dos oito saíam errados.
///
/// ⚠️ Report do Enio, 2026-08-29. Um molde que escrevesse só o texto entregava a curva de Koch
/// a **25°** (ela é `90` por definição) e a **1 291 unidades de mundo** numa coluna de ~4.
/// *Um molde não é uma gramática: é uma gramática MAIS o enquadramento em que ela se lê.*
///
/// ⚠️ **Os CINCO, um a um** — um gate que verificasse só «alguma coisa mudou» ficaria verde
/// com quatro deles por escrever.
///
/// ⛔⛔ **O quinto entrou em 2026-08-30 e a razão é medida:** um `First Level` único de `3`
/// (o que a árvore de fábrica pede) **esvaziava o `Sprig`** — as `10` marcas dele estão todas
/// na profundidade `1`. *A profundidade de encaixe significa coisas diferentes em gramáticas
/// diferentes.*
#[test]
fn picking_a_preset_also_writes_the_framing_it_needs() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    for (k, p) in ls::PRESETS.iter().enumerate() {
        apply_lsystem_preset(&mut motion, n, "source.lsystem", k as f32);
        for (name, want) in [
            (ls::param::ANGLE, p.angle),
            (ls::param::GENERATIONS, p.generations),
            (ls::param::STEP, p.step),
            (ls::param::WIDTH, p.width),
            (ls::param::LEAF_FIRST_LEVEL, p.leaf_first_level),
        ] {
            assert_eq!(
                param_value(&motion, n, name),
                want,
                "{}: o `{name}` nao foi escrito",
                p.label
            );
        }
    }
}

/// ⚠️ **E ele não toca em mais nada** — nem noutro tipo de nó, nem num índice que não existe.
///
/// O CONTROLE do tipo é o que impede um `preset` de outro nó (o nome é comum) de reescrever
/// texto alheio; o do índice é o que impede um documento carregado com um número velho de
/// apagar a gramática do artista.
#[test]
fn a_foreign_node_and_an_out_of_range_index_are_both_no_ops() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_text_param(n, ls::AXIOM_PARAM, "MEU");
    motion
        .doc
        .graph
        .set_text_param(n, ls::RULES_PARAM, "MEU -> MEU");

    apply_lsystem_preset(&mut motion, n, "motion.grid", 1.0);
    assert_eq!(
        text_of(&motion, n, ls::AXIOM_PARAM),
        "MEU",
        "outro tipo nao toca"
    );

    apply_lsystem_preset(&mut motion, n, "source.lsystem", 999.0);
    assert_eq!(
        text_of(&motion, n, ls::AXIOM_PARAM),
        "MEU",
        "indice fora da faixa"
    );
    assert_eq!(text_of(&motion, n, ls::RULES_PARAM), "MEU -> MEU");
}

/// **E o param existe no manifesto** — senão o selector não teria onde guardar a escolha.
#[test]
fn the_preset_param_is_declared_and_reaches_the_node() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_param(n, ls::param::PRESET, 2.0);
    assert!((param_value(&motion, n, ls::param::PRESET) - 2.0).abs() < 1e-6);
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// A CONVERSÃO — `Guided → Grammar` assa a gramática que os sliders faziam (2026-08-29).
// ─────────────────────────────────────────────────────────────────────────────────────────

use super::params::bake_lsystem_grammar;

/// ⭐⭐⭐ **CONVERTER MOSTRA A GRAMÁTICA QUE OS SLIDERS ESTAVAM A FAZER** — e não a de fábrica.
///
/// ⚠️ É a resposta inteira ao report de 2026-08-29 (*"O Blender e Houdini usam Axiom e
/// Rules?"* — o Houdini sim, o Blender **não tem L-System nenhum**). O nó abre em sliders; o
/// artista que quiser a gramática muda o modo e encontra lá **a planta que estava a ver**.
/// Se a conversão escrevesse o default, ela seria um botão que **destrói o trabalho** e
/// ninguém o carregaria uma segunda vez.
#[test]
fn converting_to_grammar_bakes_the_plant_the_sliders_were_making() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_param(n, ls::param::BRANCHES, 3.0);
    motion.doc.graph.set_param(n, ls::param::SEGMENTS, 2.0);
    bake_lsystem_grammar(&mut motion, n, "source.lsystem");

    let (want_axiom, want_rules) = ls::grammar_for(3.0, 2.0, 0.0, 0.0);
    assert_eq!(text_of(&motion, n, ls::AXIOM_PARAM), want_axiom);
    assert_eq!(text_of(&motion, n, ls::RULES_PARAM), want_rules);
    // ⚠️ E o CONTROLE: o que foi assado NÃO é a gramática de fábrica. Sem ele, um `bake` que
    // escrevesse o default passaria a primeira metade em qualquer forma que lhe dessem.
    assert_ne!(
        text_of(&motion, n, ls::RULES_PARAM),
        ls::DEFAULT_RULES,
        "a conversao escreveu o default e deitou fora os sliders do artista"
    );
}

/// ⚠️ **A conversão lê os SLIDERS DAQUELE nó, não os defaults do manifesto.**
///
/// Duas formas diferentes têm de assar duas gramáticas diferentes — senão o `bake` é uma
/// constante com cara de função, e o gate acima passaria com ele a ignorar o nó por inteiro.
#[test]
fn the_bake_reads_that_nodes_own_sliders_and_not_a_constant() {
    let mut motion = MotionState::new();
    let a = motion.doc.graph.add_node("source.lsystem");
    let b = motion.doc.graph.add_node("source.lsystem");
    motion.doc.graph.set_param(a, ls::param::BRANCHES, 2.0);
    motion.doc.graph.set_param(b, ls::param::BRANCHES, 5.0);
    motion.doc.graph.set_param(b, ls::param::BEND, 9.0);
    bake_lsystem_grammar(&mut motion, a, "source.lsystem");
    bake_lsystem_grammar(&mut motion, b, "source.lsystem");
    assert_ne!(
        text_of(&motion, a, ls::RULES_PARAM),
        text_of(&motion, b, ls::RULES_PARAM),
        "duas formas diferentes assaram a MESMA gramatica"
    );
}

/// **A porta não toca em nó que não é dela** — a mesma cerca do [`apply_lsystem_preset`].
#[test]
fn the_bake_never_writes_on_another_node_type() {
    let mut motion = MotionState::new();
    let n = motion.doc.graph.add_node("motion.grid");
    bake_lsystem_grammar(&mut motion, n, "motion.grid");
    assert!(text_of(&motion, n, ls::RULES_PARAM).is_empty());
}

/// ⭐⭐ **O QUE FOI ASSADO DESENHA A MESMA PLANTA** — a propriedade que faz a conversão ser
/// uma conversão, e não um recomeço.
///
/// ⚠️ Ela não é óbvia e podia falhar de duas maneiras: o `bake` podia montar a string por
/// outro caminho que o `build` (dois geradores), ou a gramática assada podia perder um param
/// pelo caminho (o literal em vez do nome). A régua é a CONTAGEM de elementos e a altura, dos
/// dois lados, com a fixtura a ter uma forma que **não** é o default.
#[test]
fn what_was_baked_draws_exactly_what_the_sliders_drew() {
    let shape = [
        (ls::param::BRANCHES, 3.0f32),
        (ls::param::SEGMENTS, 2.0),
        (ls::param::BEND, 6.0),
    ];
    let mut guided: Vec<(&str, f32)> = vec![(ls::param::MODE, ls::MODE_GUIDED as f32)];
    guided.extend_from_slice(&shape);
    let before = ls::probe_build(ls::DEFAULT_AXIOM, ls::DEFAULT_RULES, 5.0, &guided);

    let (axiom, rules) = ls::grammar_for(3.0, 2.0, 0.0, 6.0);
    let mut authored: Vec<(&str, f32)> = vec![(ls::param::MODE, ls::MODE_GRAMMAR as f32)];
    authored.extend_from_slice(&shape);
    let after = ls::probe_build(axiom, &rules, 5.0, &authored);

    assert_eq!(
        before.count(),
        after.count(),
        "a conversao mudou a planta: {} elementos antes, {} depois",
        before.count(),
        after.count()
    );
}
