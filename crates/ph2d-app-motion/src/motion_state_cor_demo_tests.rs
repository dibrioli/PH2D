//! Os gates da cena `=133`: a fixtura CONTÉM o fenómeno — o meio do degradê da esquerda (linear)
//! é cinza e o da direita (OKLCH) é cor viva — medido NA SAÍDA, pela bomba do produto.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// A croma OKLCH do `tint` das peças do MEIO da ordem, por coluna, depois de `tiques` quadros.
fn croma_do_meio(tiques: u64) -> [f32; 2] {
    let mut m = MotionState::new();
    let colunas = monta(&mut m.doc, &m.registry).expect("a cena monta");
    m.sinks = colunas.iter().map(|c| c.saida).collect();
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    for t in 0..tiques {
        #[expect(clippy::cast_precision_loss, reason = "um tique")]
        crate::motion_externals::publish_all(&mut m, t as f64 / 60.0);
        let sinks = m.sinks.clone();
        let _ = m.pump.advance_or_scrub_scoped(
            &m.doc.graph,
            &m.registry,
            &sinks,
            t,
            |t| t as f64 / 60.0,
            m.default_uv_rect,
            m.default_size,
            &scopes,
        );
    }
    colunas.map(|c| {
        let s = m.pump.cook.peek(c.saida).expect("a saida cozeu")[0]
            .as_stream()
            .clone();
        let Some(Column::Vec4(t)) = s.get("tint") else {
            panic!("a saida tem tint")
        };
        let n = t.len();
        let meio = &t[n / 2 - 8..n / 2 + 8];
        #[expect(clippy::cast_precision_loss, reason = "16 pecas")]
        let media = meio
            .iter()
            .map(|c| {
                let o = ph2d_color::perceptual::to_oklab([c[0], c[1], c[2]]);
                o[1].hypot(o[2])
            })
            .sum::<f32>()
            / meio.len() as f32;
        media
    })
}

/// ⭐⭐ **A cena ensina o que acontece** (`CLAUDE.md` §5.0): a meio do degradê azul → amarelo a
/// esquerda (`Linear RGB`) é CINZA e a direita (`OKLCH`) é cor — e a galáxia já girou.
#[test]
fn the_middle_is_grey_on_the_left_and_coloured_on_the_right() {
    let [esq, dir] = croma_do_meio(60);
    eprintln!("cena 133: croma a meio — linear {esq:.3} · oklch {dir:.3}");
    assert!(esq < 0.03, "a esquerda passa pelo cinza: {esq}");
    assert!(dir > 0.1, "a direita fica viva: {dir}");
}

/// As duas colunas são IGUAIS menos o `Color Space` do `Tint`.
#[test]
fn the_two_columns_differ_only_in_the_colour_space() {
    let mut m = MotionState::new();
    let [a, b] = monta(&mut m.doc, &m.registry).expect("a cena monta");
    let g = &m.doc.graph;
    let (mut pa, mut pb) = (
        g.node_param_overrides(a.tint).cloned().unwrap_or_default(),
        g.node_param_overrides(b.tint).cloned().unwrap_or_default(),
    );
    assert_ne!(pa.remove("space"), pb.remove("space"));
    assert_eq!(pa, pb);
}

const TUTORIAL: &str =
    include_str!("../../../docs/Motion Nodes/tutoriais/src/11_a_cor_que_o_olho_ve.html");

fn linhas_do_tint(m: &MotionState, no: NodeId) -> Vec<String> {
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let v = snap.nodes.iter().find(|v| v.id == no.0).expect("o cartao");
    v.params
        .iter()
        .map(|c| {
            if c.hint.label.starts_with("node.") {
                ph2d_i18n::tr(c.hint.label).to_string()
            } else {
                c.hint.label.to_string()
            }
        })
        .collect()
}

/// ⭐⭐⭐ **CADA PASSO DO TUTORIAL 11 É POSSÍVEL NO APP**: o `Color Space` está no cartão `Tint` em
/// `Gradient` e SOME em `Solid` (o passo 4), as figuras existem, e o texto abre a `=133`.
#[test]
fn every_step_of_the_colour_tutorial_is_reachable() {
    let mut m = MotionState::new();
    let [esq, _] = monta(&mut m.doc, &m.registry).expect("a cena monta");
    let rows = linhas_do_tint(&m, esq.tint);
    assert!(
        rows.iter().any(|r| r == "Color Space"),
        "o cartao mostra {rows:?}"
    );
    assert!(TUTORIAL.contains("<code>Color Space</code>"));
    m.doc.graph.set_param(esq.tint, "mode", 0.0);
    let rows = linhas_do_tint(&m, esq.tint);
    assert!(
        !rows.iter().any(|r| r == "Color Space"),
        "em Solid some: {rows:?}"
    );
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Motion Nodes/tutoriais/fig");
    for pedaco in TUTORIAL.split("src=\"../fig/").skip(1) {
        let nome = pedaco.split('"').next().expect("o nome");
        assert!(dir.join(nome).is_file(), "falta a figura `{nome}`");
    }
    assert!(TUTORIAL.contains("PH2D_GPU_COOK_DEMO=133"));
    assert!(crate::motion_state::demo_router::is_cycle_scene("133"));
}
