//! **How a COLOUR is authored in the params panel** — the gates for the one subject
//! that ties `motion.tint` and `motion.color_array` together (split from
//! `motion_bridge_param_tests.rs` for the HR-18 LOC cap; `super` is
//! `render_loop::motion_bridge`).
//!
//! A colour reaches the artist as a SWATCH that opens the OKLCH picker, and the wire
//! carries linear-straight channels. Everything here guards that boundary: the swatch
//! is offered instead of raw channel sliders, the pick round-trips through it, and
//! merely OPENING the picker is not an edit.

use super::color::{apply_color_to_node, channel_values, linear_rgba_to_srgb8};
use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;

/// The reported-bug + colour-authoring seam, end to end and headless: a
/// selected `motion.tint` node resolves to a named Mode selector + colour
/// SWATCH rows (not raw channel sliders), the Start swatch's channels are the
/// RGBA params, and its display colour is **opaque white** — the identity
/// default that killed the red dominance. Proves the `Color`/`Enum` hints flow
/// all the way to paintable rows (registry -> snapshot builder).
#[test]
fn selected_tint_node_yields_mode_and_colour_swatch_rows() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let tint = motion.doc.graph.add_node("motion.tint");
    ph2d_panel_motion_graph::set_graph_selection(vec![tint.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("tint node is resolvable");
    // A named Mode enum (Solid/Gradient), never a number slider.
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "mode")),
        "mode is a named Enum row"
    );
    // The Start colour is a swatch over r/g/b/a, opaque white by default.
    let start = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Color(c) if c.channels == ["r", "g", "b", "a"] => Some(c),
            _ => None,
        })
        .expect("Start colour is a swatch, not four sliders");
    assert_eq!(start.srgb, [255, 255, 255, 255]);
    // ⚠️ **Em Solid — o modo em que o nó nasce — a segunda cor NÃO aparece**, e é a cura de
    // 2026-08-22 (doc 90 §2): o braço `Gradient` é o único que lê `r2..a2`, então pintar o
    // swatch em Solid era um controle que não mudava a imagem.
    //
    // ⚠️ Este teste é sobre o WIDGET (*"uma cor é um swatch, nunca quatro sliders"*), não sobre
    // a visibilidade — então ele passa a perguntar no modo em que a row existe. *Um teste cujo
    // assunto declarado é outro não deve fixar, de passagem, o defeito que a wave cura.*
    assert!(
        !snap
            .rows
            .iter()
            .any(|r| matches!(r, ParamRow::Color(c) if c.channels == ["r2", "g2", "b2", "a2"])),
        "em Solid o swatch End nao e' pintado — ele nao faz nada ai'"
    );
    // Em Gradient ele é o seu próprio swatch sobre r2/g2/b2/a2.
    motion.doc.graph.set_param(tint, "mode", 1.0);
    let grad = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("tint node is resolvable");
    assert!(
        grad.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Color(c) if c.channels == ["r2", "g2", "b2", "a2"])),
        "End colour is its own swatch"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// The colour read-back is the inverse of the swatch display: writing a
/// picked sRGB colour lands linear-straight channel values on the node, and
/// re-reading them rebuilds the same sRGB swatch (round-trip stable). Guards
/// the sRGB↔linear boundary the bridge owns (the Motion wire is linear).
#[test]
fn color_pick_writes_linear_and_round_trips_to_srgb() {
    let mut motion = MotionState::new();
    let tint = motion.doc.graph.add_node("motion.tint");
    let picked = [40, 160, 220, 128]; // a saturated sRGB blue, half alpha

    apply_color_to_node(&mut motion, tint, ["r", "g", "b", "a"], picked);

    // The stored channels are linear-straight (RGB gamma-decoded, alpha /255).
    let lin = channel_values(&motion, tint, ["r", "g", "b", "a"]);
    assert!(lin[0] < lin[2], "blue channel dominates in linear too");
    assert!((lin[3] - 128.0 / 255.0).abs() < 1e-6, "alpha is straight");
    // Re-encoding the stored linear colour reproduces the pick (±1 LSB).
    let srgb = linear_rgba_to_srgb8(lin);
    for (got, want) in srgb.into_iter().zip(picked) {
        assert!(
            got.abs_diff(want) <= 1,
            "round-trip {srgb:?} ≈ {picked:?} within 1 LSB"
        );
    }
}

/// Merely OPENING a colour picker must not edit the document. The picker is seeded
/// with the swatch's 8-bit sRGB display colour and reports it straight back every
/// frame it is open; if the guard compared LINEAR values, a doc colour that is not
/// an exact 8-bit round-trip (here `0.5`) would be silently quantized — a doc edit
/// and an undo step the artist never asked for. The guard compares sRGB8, so an
/// unmoved picker is a no-op.
#[test]
fn opening_the_picker_does_not_quantize_an_unmoved_colour() {
    let mut motion = MotionState::new();
    let tint = motion.doc.graph.add_node("motion.tint");
    // A linear value that does NOT survive an 8-bit round-trip exactly.
    for name in ["r", "g", "b"] {
        motion.doc.graph.set_param(tint, name, 0.5);
    }
    let before = channel_values(&motion, tint, ["r", "g", "b", "a"]);

    // The picker reports back exactly what the swatch seeded it with.
    apply_color_to_node(
        &mut motion,
        tint,
        ["r", "g", "b", "a"],
        linear_rgba_to_srgb8(before),
    );

    assert_eq!(
        channel_values(&motion, tint, ["r", "g", "b", "a"]),
        before,
        "an unmoved picker must not rewrite the doc"
    );

    // A real pick still lands (the guard is not simply dead).
    apply_color_to_node(&mut motion, tint, ["r", "g", "b", "a"], [10, 20, 30, 255]);
    assert_ne!(channel_values(&motion, tint, ["r", "g", "b", "a"]), before);
}

/// **A TRANSPARÊNCIA CHEGA À PALETA** — o defeito de 2026-08-08.
///
/// ⚠️ Nasceu VERMELHO. O caminho de volta do pick preservava a alfa ANTIGA do slot e
/// descartava a escolhida, sob um comentário que afirmava *"o picker OKLCH é opaco"* —
/// e ele **não é**: tem a 4ª linha de canal (R+G+B+**A**) e um campo `#RRGGBBAA`. O
/// artista movia o slider e o número morria no caminho.
///
/// O oráculo é o `tint` COZIDO, não o texto: é o número que a lowering entrega ao
/// shader, e é onde uma alfa perdida em qualquer degrau do caminho aparece.
#[test]
fn a_translucent_palette_pick_reaches_the_cooked_tint() {
    use super::color::apply_palette_pick;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;

    let mut motion = MotionState::new();
    // ⚠️ Os dois nós são MODIFICADORES: sem fonte o cook devolve zero elementos e o gate
    // ficaria verde-sobre-vazio (a 1ª versão deste teste era exatamente isso).
    let grid = motion.doc.graph.add_node("motion.grid");
    let arr = motion.doc.graph.add_node("motion.color_array");
    motion
        .doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (grid, 0),
            to: (arr, 0),
            delayed: false,
        })
        .expect("grid -> array");
    motion.doc.graph.set_text_param(
        arr,
        "palette",
        ph2d_color::serialize_palette(&[[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]),
    );
    // Uma cor com METADE da alfa.
    apply_palette_pick(&mut motion, arr, "palette", 0, [255, 0, 0, 128]);

    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
    let mut cook = Cook::new();
    let out = cook
        .cook(&motion.doc.graph, &reg, arr, 0.0)
        .expect("the array cooks");
    match out[0].as_stream().get("tint") {
        Some(Column::Vec4(v)) => assert!(
            (v[0][3] - 128.0 / 255.0).abs() < 1e-3,
            "a alfa escolhida tinha de chegar ao tint cozido, e chegou {}",
            v[0][3]
        ),
        _ => panic!("tint"),
    }
}

/// **E CHEGA AO GRADIENTE** — a outra metade, que era mais funda: o formato `g1` não
/// tinha onde guardar a alfa (`serialize_gradient` fazia `let [r, g, b, _a]`), então
/// nem o pick nem o documento podiam expressá-la. O `g2` abriu o campo.
#[test]
fn a_translucent_gradient_stop_reaches_the_cooked_tint() {
    use super::color::apply_gradient_stop_pick;
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;

    let mut motion = MotionState::new();
    let grid = motion.doc.graph.add_node("motion.grid");
    let ramp = motion.doc.graph.add_node("motion.color_ramp");
    motion
        .doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (grid, 0),
            to: (ramp, 0),
            delayed: false,
        })
        .expect("grid -> ramp");
    apply_gradient_stop_pick(&mut motion, ramp, "ramp", 0, [255, 0, 0, 64]);

    let text = motion
        .doc
        .graph
        .node_text_param_overrides(ramp)
        .and_then(|m| m.get("ramp"))
        .expect("o gradiente foi escrito")
        .to_string();
    assert!(
        text.starts_with("g2 "),
        "o header novo carrega a alfa: {text}"
    );

    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
    let mut cook = Cook::new();
    let out = cook
        .cook(&motion.doc.graph, &reg, ramp, 0.0)
        .expect("the ramp cooks");
    match out[0].as_stream().get("tint") {
        Some(Column::Vec4(v)) => assert!(
            v.iter().any(|c| c[3] < 0.9),
            "algum elemento tinha de sair translúcido: {:?}",
            v.iter().map(|c| c[3]).collect::<Vec<_>>()
        ),
        _ => panic!("tint"),
    }
}

/// **A paleta é UMA row, e o contador morreu com o cap.**
///
/// O nó pintava doze sliders de canal, depois quatro swatches gateadas por um slider
/// `colors` — e esse slider existia só para encolher uma lista FIXA de quatro. Com a
/// paleta virando uma lista de verdade, *quantas cores há* é `len()`, e um número ao lado
/// dela seria uma segunda resposta à mesma pergunta.
///
/// FALSIFICADO se voltasse a haver params `f32` de cor (as doze rows cruas que o Enio
/// nomeou) ou um contador ao lado da faixa.
#[test]
fn the_palette_is_one_row_with_no_count_beside_it() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let ca = motion.doc.graph.add_node("motion.color_array");
    ph2d_panel_motion_graph::set_graph_selection(vec![ca.0]);
    let rows = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("resolvable")
        .rows;
    let palettes = rows
        .iter()
        .filter(|r| matches!(r, ParamRow::Palette(_)))
        .count();
    assert_eq!(palettes, 1, "exactly one Palette row: {rows:?}");
    assert!(
        !rows.iter().any(|r| matches!(r, ParamRow::Scalar(_))),
        "no scalar row survives — neither a raw channel nor a `colors` counter: {rows:?}"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **A PALETA NÃO TEM LIMITE, e isto é a prova de ponta a ponta.**
///
/// O Enio: *"color array poderia ter quantas cores o usuário quisesse. Tire os limites."*
/// O cap de quatro era quantos `ParamSpec` alguém escreveu — um limite da REPRESENTAÇÃO —,
/// então a cura não é um número maior: é a paleta virar uma LISTA (o text param), cujo
/// comprimento o `cycle` lê com `len()`.
///
/// O oráculo é o **COOK**: uma paleta de doze cores tem de pintar doze tints distintos
/// antes de repetir. Um gate que só lesse a string ficaria verde com o nó ainda ciclando
/// quatro.
#[test]
fn a_palette_of_any_length_cycles_through_all_of_it() {
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry");
    let mut g = Graph::new();
    let src = g.add_node("motion.grid");
    let ca = g.add_node("motion.color_array");
    g.connect(Edge {
        from: (src, 0),
        to: (ca, 0),
        delayed: false,
    })
    .expect("edge");

    // DOZE cores — três vezes o cap que esta wave removeu.
    #[expect(
        clippy::cast_precision_loss,
        reason = "twelve fixture indices, exact in f32"
    )]
    let twelve: Vec<[f32; 4]> = (0..12).map(|i| [i as f32 / 12.0, 0.0, 0.0, 1.0]).collect();
    g.set_text_param(ca, "palette", ph2d_color::serialize_palette(&twelve));
    g.set_param(src, "cols", 6.0);
    g.set_param(src, "rows", 4.0);

    let mut cook = Cook::new();
    let set = cook.cook(&g, &reg, ca, 0.0).expect("cooks");
    let stream: &Stream = &set.iter().next().expect("stream").as_stream().clone();
    let Some(Column::Vec4(tint)) = stream.get("tint") else {
        panic!("no tint column")
    };
    assert!(
        tint.len() >= 12,
        "need at least twelve rows: {}",
        tint.len()
    );
    let distinct: std::collections::BTreeSet<u32> =
        tint.iter().take(12).map(|c| c[0].to_bits()).collect();
    assert_eq!(
        distinct.len(),
        12,
        "the first twelve rows must take twelve DIFFERENT colours; got {} — the cycle is \
         still capped at the old slot count",
        distinct.len()
    );
    // …e a décima terceira repete a primeira: é um ciclo, não uma lista que acaba.
    assert!(
        (tint[12][0] - tint[0][0]).abs() < 1e-6,
        "the thirteenth row wraps to the first"
    );
}
