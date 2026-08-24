//! Headless tests for the **param rows** the bridge builds (split for the HR-18
//! LOC cap). Declared by the parent as a `#[path]` sibling, so `super` is
//! `render_loop::motion_bridge` and the param-authoring helpers are in the
//! sibling `params` submodule.
//!
//! Their common theme is the class of bug the Enio caught: a widget that cannot
//! represent its value paints a clamped number and destroys the real one on the
//! first touch. `every_row_range_contains_its_value_for_every_node_and_param` is
//! the gate for that whole class.
//!
//! The COLOUR-authoring half moved to the sibling `motion_bridge_colour_tests.rs`
//! (a swatch, a picker, a palette) and the CHANNEL half to
//! `motion_bridge_channel_tests.rs`; both cuts are by subject, not by line count.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
// The display face a row is built in. `default()` is what the APP ships
// (Pixels, 100 px/m), so a fixture reads the same numbers the artist does.
use ph2d_editor::ProjectSettings;

/// A selected `motion.expression` resolves to a **Formula** text row that carries the
/// graph's text-param value and sits FIRST (the formula is the node's primary control).
/// Proves the `ParamWidget::Text` hint flows through the additive text channel to a
/// paintable row (docs/Motion Nodes/33). FALSIFIED if the text param were dropped (an
/// empty field) or never surfaced (no Text row).
#[test]
fn selected_expression_node_yields_a_formula_text_row() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let ex = motion.doc.graph.add_node("motion.expression");
    motion.doc.graph.set_text_param(ex, "expr", "sin(t) * a");
    ph2d_panel_motion_graph::set_graph_selection(vec![ex.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("expression node is resolvable");
    match &snap.rows[0] {
        ParamRow::Text(t) => {
            assert_eq!(t.name, "expr");
            assert_eq!(
                t.value, "sin(t) * a",
                "the formula flows from the text channel"
            );
        }
        other => panic!("first row should be the Formula text field, got {other:?}"),
    }
    // The a..d coefficients remain scalar rows below the formula.
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Scalar(s) if s.name == "a")),
        "the coefficient params remain scalar rows"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// A selected `field.remap` resolves to an interactive **Curve** row (A1-ui), FIRST, that
/// carries the graph's `curve` text-param — not a raw text field. Proves the
/// `ParamWidget::Curve` hint surfaces as the draggable editor; FALSIFIED if it fell back
/// to a Text row (the A1-core interim) or were dropped. The Contour selector + the scalar
/// knobs remain below it.
#[test]
fn selected_field_remap_yields_an_interactive_curve_row() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let rm = motion.doc.graph.add_node("field.remap");
    // ⚠️ **O contorno é ESCOLHIDO, e isso é a reconciliação que a cerca do nó nomeava.**
    // Desde 2026-08-24 o editor de curva é gateado a `contour = Curve`: ele deixou de ser
    // oferecido nos contornos que não o lêem, que é o que o report do `motion.oscillator`
    // mostrou ser lido como «não está funcionando». Sem esta linha a fixture pergunta pelo
    // editor num modo em que ele já não existe — e isso é produto correcto, não regressão.
    motion.doc.graph.set_param(rm, "contour", 4.0);
    motion
        .doc
        .graph
        .set_text_param(rm, "curve", "c1 0:0:L 0.5:1:S 1:0:L");
    ph2d_panel_motion_graph::set_graph_selection(vec![rm.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("field.remap node is resolvable");
    match &snap.rows[0] {
        ParamRow::Curve(c) => {
            assert_eq!(c.name, "curve");
            assert_eq!(
                c.value, "c1 0:0:L 0.5:1:S 1:0:L",
                "the curve flows from the text channel to the editor"
            );
        }
        other => panic!("first row should be the interactive Curve editor, got {other:?}"),
    }
    // The Contour enum + the scalar knobs remain below the curve.
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "contour")),
        "the Contour selector remains an enum row"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// A behaviour's enum / boolean params resolve to NAMED widget rows, not
/// number sliders: the selected stagger node yields an `Enum` Channel row
/// (X/Y/Rotation/Size — one vocabulary across the whole family, audit
/// 2026-07-10), an `Enum` Easing row, and a `Toggle` Reverse row — the
/// exact fix the Enio asked for (no memorising slider steps).
#[test]
fn stagger_params_are_named_enums_and_a_checkbox() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let st = motion.doc.graph.add_node("motion.stagger");
    ph2d_panel_motion_graph::set_graph_selection(vec![st.0]);

    let snap =
        build_params_snapshot(&motion, ProjectSettings::default()).expect("stagger resolvable");
    let channel = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Enum(e) if e.name == "channel" => Some(e),
            _ => None,
        })
        .expect("channel is a named Enum row, not a slider");
    assert_eq!(channel.labels, ["X", "Y", "Rotation", "Size"]);
    let ease = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Enum(e) if e.name == "ease_curve" => Some(e),
            _ => None,
        })
        .expect("ease_curve is a named Enum row");
    // The rich curve family set (Penner minus the transcendental ones).
    assert!(ease.labels.contains(&"Bounce") && ease.labels.contains(&"Back"));
    // ⚠️ **Com a curva em `Linear` — o estado em que o nó nasce — a DIREÇÃO não aparece**, e é
    // a cura de 2026-08-22 (doc 90 §2): o `Linear` devolve `t` antes de olhar para a direção,
    // logo In/Out/In-Out davam a mesma saída ao bit.
    //
    // ⚠️ O assunto deste teste é o WIDGET (*"um enum vira um seletor nomeado, não um slider de
    // passos a decorar"*). Ele perguntava com a curva no default e, sem querer, fixava a
    // visibilidade defeituosa — *um teste pode pinar um bug de desenho enquanto prova outra
    // coisa, e continua verde a fazê-lo.*
    assert!(
        !snap
            .rows
            .iter()
            .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "ease_dir")),
        "em Linear a direcao nao e' pintada — ela nao muda nada ai'"
    );
    motion.doc.graph.set_param(st, "ease_curve", 1.0);
    let eased =
        build_params_snapshot(&motion, ProjectSettings::default()).expect("stagger resolvable");
    assert!(
        eased
            .rows
            .iter()
            .any(|r| matches!(r, ParamRow::Enum(e) if e.name == "ease_dir")),
        "ease_dir (In/Out/In-Out) is its own named Enum row"
    );
    assert!(
        snap.rows
            .iter()
            .any(|r| matches!(r, ParamRow::Toggle(t) if t.name == "reverse")),
        "reverse is a checkbox (Toggle) row, not a 0/1 slider"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **The invariant, over every registered Motion node and every param:** a row's
/// widget range CONTAINS its value. A row that violates it is a lying widget —
/// the track clamps, the panel paints the clamped number, and the first touch
/// writes it back, destroying the authored value.
///
/// `Graph::set_param` never clamps to the hint, so a preset, an undo, or a loaded
/// document can put any value on any param. This drives every node type with a
/// value far outside its hint (both signs) and asserts the row still contains it.
/// It is the gate for the whole bug class, not for one node.
#[test]
fn every_row_range_contains_its_value_for_every_node_and_param() {
    use ph2d_nodegraph::cook::OpResolver;
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    // EVERY registered node type — the real registry, not a stub list, and no prefix
    // filter. This used to keep only `motion.*`, which quietly excluded `sim.*`,
    // `value.*`, `force.*` and `pulse.*` from a gate whose name promised "every node
    // and param" — so the entire `sim.*` family shipped with no hints and a runaway
    // slider range, under a green test. A filter inside a gate is a hole in it.
    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    assert!(
        types.len() >= 80,
        "the registry really has the node library"
    );

    for ty in types {
        for extreme in [-9999.0f32, 9999.0] {
            let node = motion.doc.graph.add_node(ty);
            // Shove the extreme onto EVERY declared param of this node.
            let params: Vec<&'static str> = motion
                .registry
                .resolve(motion.doc.graph.node(node).unwrap().type_id())
                .unwrap()
                .manifest()
                .params
                .iter()
                .map(|p| p.name)
                .collect();
            for p in params {
                motion.doc.graph.set_param(node, p, extreme);
            }
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);

            let snap = build_params_snapshot(&motion, ProjectSettings::default())
                .unwrap_or_else(|| panic!("{ty} must resolve a snapshot"));
            for row in &snap.rows {
                let (name, value, min, max) = match row {
                    ParamRow::Scalar(r) => (r.name, r.value, r.min, r.max),
                    ParamRow::Angle(r) => (r.name, r.deg, r.min_deg, r.max_deg),
                    ParamRow::Seed(r) => (r.name, r.value, r.min, r.max),
                    // Color / Toggle / Enum carry no continuous range.
                    _ => continue,
                };
                assert!(
                    min <= value && value <= max,
                    "{ty}.{name}: value {value} escapes the widget range [{min}, {max}] \
                     -> the panel would paint a clamped number and destroy it on touch"
                );
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// Angle params resolve to a `deg` number-box row, and the row is degrees end to
/// end — the param stores exactly what the box shows. `motion.rotate` (which adds
/// to the `rot` column) and `motion.orbit` (whose trig is cycle-based) both
/// author in the SAME unit; radians and turns exist nowhere on this surface.
#[test]
fn angle_params_resolve_to_degree_rows() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();

    let angle_row = |motion: &MotionState, who: &str| {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("node resolvable")
            .rows
            .into_iter()
            .find_map(|r| match r {
                ParamRow::Angle(a) if a.name == "angle" => Some(a),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{who} has no Angle row"))
    };

    // motion.rotate feeds the `rot` column: a full-circle range in degrees.
    let rot = motion.doc.graph.add_node("motion.rotate");
    ph2d_panel_motion_graph::set_graph_selection(vec![rot.0]);
    let a = angle_row(&motion, "rotate");
    assert_eq!(
        (a.min_deg, a.max_deg),
        (-180.0, 180.0),
        "degrees, not radians"
    );
    assert_eq!(a.deg, 0.0, "default 0 deg");

    // motion.orbit's polar angle: the same unit, a wider range.
    let orbit = motion.doc.graph.add_node("motion.orbit");
    ph2d_panel_motion_graph::set_graph_selection(vec![orbit.0]);
    let a = angle_row(&motion, "orbit");
    assert_eq!(
        (a.min_deg, a.max_deg),
        (-360.0, 360.0),
        "degrees, not turns"
    );

    // Setting 90 deg on the doc reads back as 90 in the row — no conversion.
    motion.doc.graph.set_param(orbit, "angle", 90.0);
    assert_eq!(angle_row(&motion, "orbit").deg, 90.0);

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// A Seed param resolves to a Seed row (whole-number box + re-roll button), never
/// a slider the artist must drag through a range that means nothing.
#[test]
fn seed_param_resolves_to_a_seed_row_not_a_slider() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let wig = motion.doc.graph.add_node("motion.wiggle");
    ph2d_panel_motion_graph::set_graph_selection(vec![wig.0]);

    let snap =
        build_params_snapshot(&motion, ProjectSettings::default()).expect("wiggle resolvable");
    let seed = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Seed(s) => Some(s),
            _ => None,
        })
        .expect("seed is a Seed row");
    assert_eq!(seed.name, "seed");
    assert!(seed.min < seed.max, "the seed box has a usable range");
    assert!(
        !snap
            .rows
            .iter()
            .any(|r| matches!(r, ParamRow::Scalar(s) if s.name == "seed")),
        "seed must not ALSO appear as a scalar slider"
    );

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **The rename + re-tint UI exists** (F2): with a BACKDROP selected the params
/// panel stops showing node params and shows the backdrop's own — a Title text box
/// and the 8-tint picker, whose labels name the hue ramp the tokens actually walk.
/// A backdrop is not a node (no manifest, never cooks), so without this branch it
/// would be unnameable: created, and then stuck as "Group" forever.
/// FALSIFIED if the panel fell through to the node path (`None`, an empty panel).
#[test]
fn a_selected_backdrop_yields_its_title_and_colour_rows() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    super::backdrops::add(&mut motion, 0.0, 0.0, 300.0, 200.0);
    let id = motion.doc.backdrops[0].id;
    super::backdrops::set_title(&mut motion, id, "Force chain".to_string());
    super::backdrops::set_color(&mut motion, id, 4);
    ph2d_panel_motion_graph::set_graph_backdrop_selection(Some(id));

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("the backdrop is the subject");
    assert_eq!(snap.node, id);
    match &snap.rows[0] {
        ParamRow::Text(t) => {
            assert_eq!(t.name, "title");
            assert_eq!(t.value, "Force chain", "the box opens on the current name");
        }
        other => panic!("the first row should be the Title box, got {other:?}"),
    }
    match &snap.rows[1] {
        ParamRow::Enum(e) => {
            assert_eq!(e.name, "color");
            assert_eq!(e.selected, 4, "the picker opens on the current tint");
            assert_eq!(e.labels.len(), 8, "one label per `graph-backdrop-*` token");
        }
        other => panic!("the second row should be the Color picker, got {other:?}"),
    }

    ph2d_panel_motion_graph::set_graph_backdrop_selection(None);
}

/// And the subjects are mutually exclusive: a selected NODE still shows node
/// params (the backdrop branch cannot hijack the panel once a node is picked).
#[test]
fn a_selected_node_still_wins_the_params_panel() {
    let mut motion = MotionState::new();
    super::backdrops::add(&mut motion, 0.0, 0.0, 300.0, 200.0);
    let grid = motion.doc.graph.add_node("motion.grid");
    ph2d_panel_motion_graph::set_graph_backdrop_selection(None);
    ph2d_panel_motion_graph::set_graph_selection(vec![grid.0]);

    let snap = build_params_snapshot(&motion, ProjectSettings::default())
        .expect("the node is the subject");
    assert_eq!(snap.node, grid.0);
    assert_eq!(snap.title, "Grid");

    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **A look-at can be told WHERE to look, and the picker only shows where it means
/// something.**
///
/// The other node the Enio named: `motion.look_at` could aim anywhere and could not
/// aim at anything the artist can NAME or at the cursor — the two things the node is
/// for in every other tool. Wiring two `value.*` nodes to follow the mouse is not a
/// workaround, it is impossible, because the cursor is not in the graph.
///
/// The gate is presence AND absence: an object picker offered in Point or Cursor mode
/// is a control the cook will never read, which is the dead row this codebase keeps one
/// table per menu to prevent.
#[test]
fn the_look_at_picks_its_target_and_offers_the_picker_only_in_object_mode() {
    use ph2d_panel_motion_params::ParamRow;
    let mut motion = MotionState::new();
    let la = motion.doc.graph.add_node("motion.look_at");
    ph2d_panel_motion_graph::set_graph_selection(vec![la.0]);

    let rows = |motion: &MotionState| {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("resolvable")
            .rows
    };
    // The mode is a NAMED choice, not an index the artist has to decode.
    let modes = rows(&motion)
        .into_iter()
        .find_map(|r| match r {
            ParamRow::Enum(e) if e.name == "mode" => Some(e.labels),
            _ => None,
        })
        .expect("`Aim At` is a named selector");
    assert_eq!(modes, ["Point", "Object", "Cursor"]);

    // The offset is DEGREES and says so — an `Angle` row, which no unit table can
    // contradict (doc 88).
    assert!(
        rows(&motion)
            .iter()
            .any(|r| matches!(r, ParamRow::Angle(a) if a.name == "offset")),
        "the offset is an angle, not a bare number"
    );

    let has_picker = |motion: &MotionState| {
        rows(motion)
            .iter()
            .any(|r| matches!(r, ParamRow::Source(s) if s.param == "target"))
    };
    // Default (Point): no object to name.
    assert!(!has_picker(&motion), "Point aims at the value inputs");
    motion.doc.graph.set_param(la, "mode", 1.0);
    assert!(has_picker(&motion), "Object mode offers the name picker");
    motion.doc.graph.set_param(la, "mode", 2.0);
    assert!(!has_picker(&motion), "the Cursor is not an object name");
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **The picker offers what the ARTIST named, never the editor's own values.**
///
/// The reserved namespace carries the cursor and a `$at:<name>` position beside every
/// published object. Both live in the table the picker reads, so without a filter they
/// appear as objects you can aim at — the namespace leaking into the UI it exists to
/// keep clean, and a list where half the entries are implementation.
#[test]
fn the_source_picker_hides_the_editors_reserved_namespace() {
    use ph2d_nodegraph::attr::{Column, Stream};
    let mut motion = MotionState::new();
    let at = |x: f32| Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
    motion.pump.cook.set_external("Sun".to_string(), at(1.0));
    motion
        .pump
        .cook
        .set_external(ph2d_nodegraph::external::position_of("Sun"), at(1.0));
    motion
        .pump
        .cook
        .set_external(ph2d_nodegraph::external::CURSOR.to_string(), at(2.0));

    let opts = super::params::source_options_for_tests(&motion);
    assert_eq!(
        opts,
        vec!["Sun".to_string()],
        "only the artist's name is pickable: {opts:?}"
    );
}

/// **A row DIRIGIDA diz QUEM a dirige, e o nome é o que está escrito no card** (doc 88 B3).
///
/// Nasceu VERMELHO: a row carregava um `bool`. O artista via um número acentuado que não
/// obedecia ao dedo e não tinha uma palavra sobre a procedência — a resposta exigia sair do
/// inspector e caçar o fio no grafo, num nó que ele ainda não sabia qual era.
///
/// As duas metades, e a segunda é a que importa: o nome sai da porta única
/// `ph2d_node_registry::card_title`, então **um rename move os dois** — o card e a row. Uma
/// escada de fallbacks copiada aqui ficaria verde neste gate e mentiria no dia do rename, que
/// é justamente o dia em que o artista precisa do nome para achar o nó.
#[test]
fn a_driven_row_names_the_card_that_drives_it() {
    use ph2d_nodegraph::attr::{Column, Stream};
    use std::collections::BTreeMap;

    let mut motion = MotionState::new();
    let driver = motion.doc.graph.add_node("value.gain");
    let target = motion.doc.graph.add_node("value.gain");
    motion
        .doc
        .graph
        .drive_param(target, "strength", (driver, 0))
        .expect("o fio entra no param");
    // O tap é o caminho de um frame de GPU (o default do app); o memo estaria vazio.
    motion.gpu_tap = Some(BTreeMap::from([(
        driver,
        Stream::new(1).with("v", Column::Scalar(vec![42.0])),
    )]));
    ph2d_panel_motion_graph::set_graph_selection(vec![target.0]);

    let driven_by = |motion: &MotionState| -> Option<String> {
        build_params_snapshot(motion, ProjectSettings::default())
            .expect("o alvo resolve")
            .rows
            .iter()
            .find_map(|r| match r {
                ph2d_panel_motion_params::ParamRow::Scalar(s) if s.name == "strength" => {
                    Some(s.driven_by.clone())
                }
                _ => None,
            })
            .expect("a row do param dirigido existe")
    };

    // Sem rename: o card diz o nome do TIPO, e a row diz o mesmo.
    assert_eq!(
        driven_by(&motion).as_deref(),
        Some("Gain"),
        "a row dirigida nomeia o card que a dirige"
    );

    // Com rename: o nome do ARTISTA vence nos dois lugares.
    motion.doc.graph.set_label(driver, "Volume");
    assert_eq!(
        driven_by(&motion).as_deref(),
        Some("Volume"),
        "e o nome segue o rename — é a MESMA porta que escreve o título do card"
    );

    // O CONTROLE: sem fio não há nome. `driven_by` é o fato inteiro, então isto é o que
    // impede a row de nascer com dono e sem procedência.
    ph2d_panel_motion_graph::set_graph_selection(vec![driver.0]);
    assert_eq!(
        driven_by(&motion),
        None,
        "um param que ninguém dirige não tem quem o nomeie"
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}

/// **O picker de canais CABE no teto do painel — e o teto não é o que o doc dizia.**
///
/// O doc-comment do `READ_CHANNELS` afirmava *"seven of them + Custom = 8 = the segmented
/// selector's ceiling"*. O teto real é `MAX_ENUM_OPTIONS = 48` (DERIVADO, com o
/// `CHANNELS_EXTRA_BASE` começando exatamente onde ele acaba) e a fileira **quebra em quatro
/// colunas**, crescendo a própria altura — então o 8 era um palpite sobre LARGURA vestindo a
/// palavra *teto*, e o canal `Falloff` (2026-08-09) o teria "estourado" sem nada acontecer.
///
/// ⚠️ **Este gate mora AQUI porque é o único lugar onde as duas metades se encontram:** a
/// tabela vive na crate do nó (que não conhece painel) e o teto vive na crate do painel (que
/// não conhece registry). Ele afirma a PROPRIEDADE — *toda opção pintada tem id próprio* —
/// e não a contagem de hoje, então um canal novo passa e o quadragésimo nono reprova, que é
/// exatamente onde a decisão volta a ser necessária.
#[test]
fn the_channel_picker_fits_the_panels_ceiling() {
    use ph2d_panel_motion_params::{MAX_ENUM_OPTIONS, ParamRow};
    let mut motion = MotionState::new();
    let attr = motion.doc.graph.add_node("value.attribute");
    ph2d_panel_motion_graph::set_graph_selection(vec![attr.0]);

    let snap =
        build_params_snapshot(&motion, ProjectSettings::default()).expect("attribute resolvable");
    let ch = snap
        .rows
        .iter()
        .find_map(|r| match r {
            ParamRow::Channels(c) => Some(c),
            _ => None,
        })
        .expect("o picker de canais é uma row de Channels");

    // O peso de um campo é OFERECIDO, não só legível se digitado (doc 89, folha 12).
    assert!(
        ch.channels
            .iter()
            .any(|(l, c, _)| *l == "Falloff" && *c == "falloff"),
        "o picker oferece o peso que as `field.*` escrevem: {:?}",
        ch.channels.iter().map(|(l, ..)| *l).collect::<Vec<_>>()
    );
    // A propriedade: cada canal + o "Custom…" final ganha um botão com id próprio. Acima do
    // teto o `.min(MAX_ENUM_OPTIONS)` do painter simplesmente PARA de desenhar — a opção
    // excedente nasceria invisível e inalcançável, em silêncio.
    let painted = ch.channels.len() + 1; // os canais curados + o "Custom…" final
    assert!(
        painted <= MAX_ENUM_OPTIONS,
        "{} canais + Custom = {painted} passam do teto de {MAX_ENUM_OPTIONS} — o excedente não é pintado",
        ch.channels.len()
    );
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
}
