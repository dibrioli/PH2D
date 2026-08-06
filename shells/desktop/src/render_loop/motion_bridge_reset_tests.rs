//! **A quarta condição de UI: a sequência LEVA a algum lugar** — o gesto de reverter, do
//! documento até o que o painel volta a ler.
//!
//! Os gates do painel (`lib_reset_tests`) provam que a seta existe, é registrada e que o
//! clique chega ao barramento; os do `ph2d-nodegraph` provam que limpar REMOVE o override.
//! Falta o meio: que a ponte PUBLIQUE quais params estão modificados, e que devolver um
//! deles ao default seja visível pela mesma porta que o painel lê.
//!
//! ⚠️ O braço do `match` que consome o intent vive num `fn` que exige `WidgetStore` e a
//! seleção viva, fora do alcance de um teste headless — ele é preso por um arch-gate sobre o
//! fonte (`tests/the_bridge_reverts_a_param_by_clearing_both_channels.rs`), não por uma porta
//! só-para-teste ao lado do produto: a porta de teste seria uma SEGUNDA resposta a *como se
//! reverte um param*, e a que fica verde quando o produto perde o braço.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_panel_motion_params::ParamRow;

fn modified_of(motion: &MotionState) -> std::collections::BTreeSet<String> {
    build_params_snapshot(motion, ProjectSettings::default())
        .map(|s| s.modified)
        .unwrap_or_default()
}

fn scalar_value(motion: &MotionState, param: &str) -> Option<f64> {
    build_params_snapshot(motion, ProjectSettings::default())?
        .rows
        .into_iter()
        .find_map(|r| match r {
            ParamRow::Scalar(s) if s.name == param => Some(s.value),
            _ => None,
        })
}

/// **Mexer num param o marca; reverter o desmarca E devolve o default.**
///
/// As duas metades num gate só, de propósito: separadas, *"marca"* passaria com um conjunto
/// que nunca esvazia e *"desmarca"* passaria com um conjunto sempre vazio. O que o artista
/// precisa é que o par feche — ele mexe, vê que mexeu, reverte, e a marca some junto com o
/// valor.
#[test]
fn touching_a_param_marks_it_and_reverting_unmarks_it_and_restores_the_default() {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("motion.grid");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);

    let before = scalar_value(&motion, "rows").expect("o grid tem `rows`");
    assert!(
        modified_of(&motion).is_empty(),
        "um nó recém-criado não tem nada modificado"
    );

    // O artista arrasta. O valor tem de DIFERIR do default, senão o gate ficaria verde
    // sobre um reset que não reverte coisa nenhuma.
    motion
        .doc
        .graph
        .set_param(node, "rows", (before + 5.0) as f32);
    assert!(
        modified_of(&motion).contains("rows"),
        "o override tem de aparecer no conjunto que o painel lê para oferecer a seta"
    );
    assert_ne!(scalar_value(&motion, "rows"), Some(before));

    // E reverte, pela porta do documento.
    assert!(motion.doc.graph.clear_param(node, "rows"));
    assert!(
        !modified_of(&motion).contains("rows"),
        "revertido, o param não pode continuar marcado — a seta some porque a CHAVE saiu"
    );
    assert_eq!(
        scalar_value(&motion, "rows"),
        Some(before),
        "e o valor volta ao default do manifesto, pela mesma porta que o cook usa"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **O conjunto de modificados fala pelos DOIS canais.**
///
/// Um param viaja pelo canal de `f32` ou pelo de texto, nunca pelos dois, e o painel não sabe
/// (nem deveria) por qual — a §5 registra dois params MIGRANDO de canal (o gradiente do
/// `color_ramp`, a paleta do `color_array`). Se a publicação lesse só um deles, a seta
/// simplesmente não apareceria numa curva, num gradiente ou numa paleta, sem erro nenhum.
#[test]
fn the_modified_set_speaks_for_the_text_channel_too() {
    let mut motion = MotionState::new();
    let node = motion.doc.graph.add_node("motion.color_array");
    ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);

    assert!(modified_of(&motion).is_empty());
    motion
        .doc
        .graph
        .set_text_param(node, "palette", "p1 1,0,0,1");
    assert!(
        modified_of(&motion).contains("palette"),
        "um override de TEXTO conta como modificado tanto quanto um de f32"
    );

    assert!(motion.doc.graph.clear_text_param(node, "palette"));
    assert!(modified_of(&motion).is_empty(), "e some quando a chave sai");
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
