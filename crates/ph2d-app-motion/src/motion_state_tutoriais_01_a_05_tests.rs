//! Os gates dos TUTORIAIS 01–05 — «cada linha que o texto nomeia está no cartão» (doc 103 §1),
//! o mesmo gate que os de 06–09 têm (`motion_state_*_tutorial_tests.rs`).
//!
//! ⛔ O 01 ensinou durante semanas a linha `Shape` do Grid, que saiu no `54ba1cbaa` (19/09): um
//! passo que manda clicar numa linha AFIRMA que ela existe, e sem gate a afirmação apodrece.
//!
//! ⚠️ O cartão é lido com **todas as secções abertas** — o texto manda abri-las — e as linhas que
//! só aparecem num modo são pedidas **depois** de o gate escolher esse modo, como o passo faz.

use crate::motion_state::MotionState;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_panel_motion_graph::GraphViewSnapshot;

const T01: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/01_arranjo.html");
const T02: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/02_animadores.html");
const T03: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/03_transformes.html");
const T04: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/04_campos.html");
const T05: &str = include_str!("../../../docs/Motion Nodes/tutoriais/src/05_simulacao.html");

type Pedidos<'a> = &'a [(&'a str, &'a [&'a str])];

/// A cena que o tutorial manda abrir (`PH2D_GPU_COOK_DEMO=<n>`) mais os nós que ele manda largar.
fn cena(tutorial: &str, nivel: &str, largados: &[&str]) -> MotionState {
    assert!(
        tutorial.contains(&format!("PH2D_GPU_COOK_DEMO={nivel} ")),
        "o tutorial nao abre a cena `={nivel}`"
    );
    let mut m = MotionState::new();
    let _ = crate::motion_demo_legend::monta(nivel, &mut m.doc, &m.registry);
    for t in largados {
        let _ = m.doc.graph.add_node(*t);
    }
    m
}

fn carimba(m: &MotionState) -> GraphViewSnapshot {
    let mut s = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut s,
    );
    s
}

/// Os cartões como o dono os vê, com todas as secções abertas.
fn cartoes(m: &mut MotionState) -> GraphViewSnapshot {
    let s = carimba(m);
    for v in &s.nodes {
        for sec in &v.sections {
            m.card_sections.insert((v.id, sec.key), true);
        }
    }
    carimba(m)
}

/// ⚠️ Um passo que manda ABRIR uma secção já aberta fecha-a — e o dono vê os controlos sumirem.
/// Confere como cada secção NASCE (antes de qualquer clique) contra o que o texto diz.
fn confere_nascimento(m: &mut MotionState, pedidos: &[(&str, &str, bool)]) {
    let lembradas = std::mem::take(&mut m.card_sections);
    let s = carimba(m);
    m.card_sections = lembradas;
    for (titulo, secao, aberta) in pedidos {
        let estados: Vec<bool> = s
            .nodes
            .iter()
            .filter(|v| v.display_name == *titulo)
            .flat_map(|v| {
                v.sections
                    .iter()
                    .filter(|x| x.label == *secao)
                    .map(|x| x.open)
            })
            .collect();
        assert!(
            !estados.is_empty(),
            "o cartao `{titulo}` nao tem a seccao `{secao}`"
        );
        assert!(
            estados.iter().all(|e| e == aberta),
            "o texto diz que a seccao `{secao}` do `{titulo}` nasce {}, e ela nasce {estados:?}",
            if *aberta { "aberta" } else { "fechada" }
        );
    }
}

/// O texto nomeia `palavra` como coisa da tela: num passo (`<code>`), em negrito, ou numa linha
/// de cartão desenhado (`<span class="l">`).
fn nomeia(tutorial: &str, palavra: &str) -> bool {
    [
        format!("<code>{palavra}</code>"),
        format!("<b>{palavra}</b>"),
        format!("<span class=\"l\">{palavra}</span>"),
        // a escolha escrita dentro do passo: `<code>Mode = Add</code>`
        format!(" = {palavra}</code>"),
    ]
    .iter()
    .any(|f| tutorial.contains(f.as_str()))
}

/// Cada `(cartão, linhas)` pedido existe: ALGUM cartão com esse título mostra cada linha.
/// Devolve quantas linhas conferiu.
fn confere(tutorial: &str, snap: &GraphViewSnapshot, pedidos: Pedidos) -> usize {
    let mut n = 0;
    for (titulo, linhas) in pedidos {
        let mesmos: Vec<_> = snap
            .nodes
            .iter()
            .filter(|v| v.display_name == *titulo)
            .collect();
        assert!(
            !mesmos.is_empty(),
            "o tutorial nomeia o cartao `{titulo}` e a cena tem {:?}",
            snap.nodes
                .iter()
                .map(|v| v.display_name.as_str())
                .collect::<Vec<_>>()
        );
        assert!(
            tutorial.contains(titulo),
            "o gate defende `{titulo}` e o tutorial nao o nomeia"
        );
        for l in *linhas {
            let tem = mesmos
                .iter()
                .any(|v| v.params.iter().any(|c| c.hint.label == *l));
            assert!(
                tem,
                "o tutorial manda mexer em `{l}` no cartao `{titulo}`, e o cartao mostra {:?}",
                mesmos[0]
                    .params
                    .iter()
                    .map(|c| c.hint.label)
                    .collect::<Vec<_>>()
            );
            assert!(
                nomeia(tutorial, l),
                "o gate defende `{l}` e o tutorial nunca a nomeia"
            );
            n += 1;
        }
    }
    n
}

/// Cada secção que o texto manda abrir existe no cartão.
fn confere_secoes(tutorial: &str, snap: &GraphViewSnapshot, pedidos: Pedidos) -> usize {
    let mut n = 0;
    for (titulo, secoes) in pedidos {
        for s in *secoes {
            let tem = snap
                .nodes
                .iter()
                .filter(|v| v.display_name == *titulo)
                .any(|v| v.sections.iter().any(|x| x.label == *s));
            assert!(
                tem,
                "o tutorial manda abrir a seccao `{s}` do cartao `{titulo}`, e ela nao existe"
            );
            assert!(
                nomeia(tutorial, s),
                "o gate defende a seccao `{s}` e o tutorial nao a nomeia"
            );
            n += 1;
        }
    }
    n
}

/// As palavras do menu de um param de escolha, como o artista as lê.
fn opcoes(m: &MotionState, tipo: &str, param: &str) -> Vec<&'static str> {
    m.registry
        .param_ui(NodeTypeId::of(tipo))
        .and_then(|h| h.iter().find(|h| h.param == param))
        .and_then(|h| match h.widget {
            ph2d_node_registry::ParamWidget::Enum { labels } => {
                Some(labels.iter().map(|l| ph2d_i18n::tr(l)).collect())
            }
            // ⭐ O `Ease` (ciclo 17): as miniaturas do editor gráfico, uma por família.
            ph2d_node_registry::ParamWidget::Ease => Some(
                ph2d_ease::CHAVES_FAMILIA
                    .iter()
                    .map(|l| ph2d_i18n::tr(l))
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

/// Cada opção que o texto manda escolher existe no menu. Devolve quantas conferiu.
fn confere_opcoes(tutorial: &str, m: &MotionState, pedidos: &[(&str, &str, &[&str])]) -> usize {
    let mut n = 0;
    for (tipo, param, palavras) in pedidos {
        let menu = opcoes(m, tipo, param);
        for p in *palavras {
            assert!(
                menu.contains(p),
                "o tutorial manda escolher `{p}` em `{tipo}.{param}` e o menu tem {menu:?}"
            );
            assert!(
                nomeia(tutorial, p),
                "o gate defende a opcao `{p}` e o tutorial nao a nomeia"
            );
            n += 1;
        }
    }
    n
}

/// O gesto do passo: escolhe `opcao` no menu `param` de todo nó `tipo` da cena.
fn escolhe(m: &mut MotionState, tipo: &str, param: &str, opcao: &str) {
    let i = opcoes(m, tipo, param)
        .iter()
        .position(|o| *o == opcao)
        .unwrap_or_else(|| panic!("`{tipo}.{param}` nao tem a opcao `{opcao}`"));
    let alvos: Vec<_> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == tipo)
        .map(|n| n.id)
        .collect();
    assert!(!alvos.is_empty(), "a cena nao tem `{tipo}`");
    for id in alvos {
        #[expect(clippy::cast_precision_loss, reason = "indice de menu pequeno")]
        m.doc.graph.set_param(id, param, i as f32);
    }
}

/// Os cartões DESENHADOS no texto (`<div class="card-demo">`): título e linhas.
fn desenhados(tutorial: &str) -> Vec<(&str, Vec<&str>)> {
    tutorial
        .split("<div class=\"card-demo\">")
        .skip(1)
        .map(|bloco| {
            let titulo = bloco
                .split("class=\"h\"")
                .nth(1)
                .and_then(|r| r.split('>').nth(1))
                .and_then(|r| r.split('<').next())
                .expect("o cartao desenhado tem titulo");
            let linhas = bloco
                .split("<span class=\"l\">")
                .skip(1)
                .filter_map(|r| r.split('<').next())
                .collect();
            (titulo, linhas)
        })
        .collect()
}

/// ⭐ Um cartão desenhado no PDF é uma fotografia do cartão: cada linha dele existe no de verdade.
fn confere_desenhados(tutorial: &str, snap: &GraphViewSnapshot) -> usize {
    let mut n = 0;
    for (titulo, linhas) in desenhados(tutorial) {
        let pedido: &[&str] = &linhas;
        n += confere(tutorial, snap, &[(titulo, pedido)]);
    }
    n
}

/// ⭐⭐⭐ **Tutorial 01 (Arranjo)** — cena `=1` mais os nós que ele manda largar.
#[test]
fn every_row_the_arrangement_tutorial_names_is_on_the_card() {
    let mut m = cena(
        T01,
        "1",
        &[
            "motion.fibonacci",
            "motion.distribute_radial",
            "motion.clone",
            "motion.scatter",
        ],
    );
    let snap = cartoes(&mut m);
    let mut n = confere_desenhados(T01, &snap);
    n += confere(
        T01,
        &snap,
        &[
            ("Grid", &["Rows", "Columns", "Gap X", "Gap Y"]),
            ("Oscillator", &["Wave"]),
            ("Output", &[]),
            ("Fibonacci Spiral", &["Count", "Spacing", "Angle"]),
            ("Radial Array", &["Start Angle", "End Angle"]),
            ("Clone", &[]),
            ("Scatter", &["Density Falloff"]),
        ],
    );
    n += confere_opcoes(T01, &m, &[("motion.oscillator", "wave", &["Sine"])]);
    assert_eq!(n, 16, "conferi {n}");
    // «Quatro cartões: Grid, Oscillator, Move e Output» — a cena que abre, sem nada largado.
    let abre: Vec<String> = carimba(&cena(T01, "1", &[]))
        .nodes
        .iter()
        .map(|v| v.display_name.clone())
        .collect();
    assert_eq!(
        abre,
        ["Grid", "Oscillator", "Move", "Output"],
        "o passo 1 da seccao 2 conta-os"
    );
    assert!(
        T01.contains("Quatro cartões"),
        "o texto tem de contar os quatro cartoes"
    );
}

/// ⭐⭐⭐ **Tutorial 02 (Animadores)** — cena `=1` mais os oito animadores.
#[test]
fn every_row_the_animators_tutorial_names_is_on_the_card() {
    let mut m = cena(
        T02,
        "1",
        &[
            "motion.stagger",
            "motion.spring",
            "motion.wiggle",
            "motion.noise",
            "motion.orbit",
            "motion.delay",
            "value.lfo",
            "motion.drive",
        ],
    );
    let snap = cartoes(&mut m);
    let mut n = confere_desenhados(T02, &snap);
    n += confere(
        T02,
        &snap,
        &[
            ("Grid", &["Rows", "Columns", "Gap X"]),
            (
                "Oscillator",
                &[
                    "Channel",
                    "Wave",
                    "Amplitude",
                    "Frequency",
                    "Stagger",
                    "Time Mode",
                ],
            ),
            ("Stagger", &["Max", "Ease", "Order", "Reverse", "Channel"]),
            ("Spring", &["Mode", "Tension", "Friction", "Channel"]),
            ("Wiggle", &["Amplitude", "Frequency", "Octaves"]),
            (
                "Noise",
                &[
                    "Channel",
                    "Amplitude",
                    "Scale",
                    "Octaves",
                    "Rotation",
                    "Uniform",
                ],
            ),
            ("Orbit", &["Speed", "Pivot X", "Carry Rotation"]),
            ("Delay", &["Rise", "Fall", "Mode"]),
            ("LFO", &[]),
            ("Drive", &["Channel", "Mode"]),
        ],
    );
    n += confere_secoes(T02, &snap, &[("Noise", &["Space"])]);
    // «Procure a seção Space — ela nasce fechada».
    confere_nascimento(&mut m, &[("Noise", "Space", false)]);
    n += confere_opcoes(
        T02,
        &m,
        &[
            (
                "motion.oscillator",
                "wave",
                &["Sine", "Tri", "Square", "Saw", "Spike", "Custom"],
            ),
            ("motion.oscillator", "channel", &["Y"]),
            (
                "motion.stagger",
                "ease_curve",
                &["Quad", "Cubic", "Back", "Bounce"],
            ),
            (
                "motion.stagger",
                "order",
                &["Index", "From Center", "Random"],
            ),
            ("motion.stagger", "channel", &["Size"]),
            ("motion.spring", "mode", &["Physics", "Time"]),
            ("motion.spring", "channel", &["Position XY"]),
            ("motion.noise", "channel", &["Position XY", "Y"]),
            ("motion.delay", "mode", &["Delay", "Average", "Blend"]),
            ("motion.drive", "channel", &["Y", "Size", "Rotation"]),
            ("motion.drive", "mode", &["Add"]),
        ],
    );
    // As linhas que só aparecem DEPOIS de um passo escolher o modo.
    escolhe(&mut m, "motion.oscillator", "wave", "Custom");
    escolhe(&mut m, "motion.stagger", "order", "Random");
    escolhe(&mut m, "motion.spring", "mode", "Time");
    escolhe(&mut m, "motion.noise", "channel", "Position XY");
    let ruido: Vec<_> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|i| i.type_name == "motion.noise")
        .map(|i| i.id)
        .collect();
    for id in ruido {
        m.doc.graph.set_param(id, "uniform", 0.0);
    }
    let snap = cartoes(&mut m);
    n += confere(
        T02,
        &snap,
        &[
            ("Oscillator", &["Custom Wave"]),
            ("Stagger", &["Seed"]),
            ("Spring", &["Duration", "Bounce"]),
            ("Noise", &["Scale Y"]),
        ],
    );
    assert_eq!(n, 74, "conferi {n}");
}

/// ⭐⭐⭐ **Tutorial 03 (Transformes)** — cena `=111` mais os deformadores que ele manda largar.
#[test]
fn every_row_the_deformers_tutorial_names_is_on_the_card() {
    let mut m = cena(
        T03,
        "111",
        &[
            "motion.kaleidoscope",
            "motion.bend",
            "motion.spline_wrap",
            "motion.look_at",
            "motion.bezier_warp",
            "motion.rotate",
        ],
    );
    let snap = cartoes(&mut m);
    let mut n = confere(
        T03,
        &snap,
        &[
            ("Twist", &["Pivot", "Pivot X", "Pivot Y"]),
            ("Transform", &["Skew X", "Skew Y", "Scale", "Pivot"]),
            (
                "Mirror",
                &["Axis Offset", "Axis", "Flip Orientation", "Reindex"],
            ),
            ("Grid", &["Rows", "Columns", "Gap X", "Gap Y"]),
            ("Kaleidoscope", &["Pivot"]),
            ("Bend", &[]),
            ("Spline Wrap", &["Shape", "Use Selected Path"]),
            ("Look At", &["Target X"]),
            ("Move", &[]),
            ("Scale", &[]),
            ("Rotate", &[]),
            ("LFO", &[]),
        ],
    );
    n += confere_secoes(T03, &snap, &[("Spline Wrap", &["Curve"])]);
    // «a seção Curve [...] nasce fechada».
    confere_nascimento(&mut m, &[("Spline Wrap", "Curve", false)]);
    n += confere_opcoes(
        T03,
        &m,
        &[(
            "motion.twist",
            "pivot_mode",
            &["Point", "Centroid", "World Origin"],
        )],
    );
    // «o Bezier Warp tem cinco [secções]: uma para os cantos e uma por borda».
    let bezier = snap
        .nodes
        .iter()
        .find(|v| v.display_name == "Bezier Warp")
        .expect("o Bezier Warp");
    assert_eq!(
        bezier.sections.len(),
        5,
        "o texto promete cinco seccoes no Bezier Warp"
    );
    assert_eq!(n, 23, "conferi {n}");
}

/// ⭐⭐⭐ **Tutorial 04 (Campos)** — cena `=112` mais os campos que ele manda largar.
#[test]
fn every_row_the_fields_tutorial_names_is_on_the_card() {
    let mut m = cena(
        T04,
        "112",
        &[
            "field.index_range",
            "field.combine",
            "field.shape",
            "field.box",
            "field.radial_sweep",
            "motion.rotate",
        ],
    );
    let snap = cartoes(&mut m);
    let mut n = confere(
        T04,
        &snap,
        &[
            ("Falloff", &["Shape", "Radius"]),
            ("Remap", &["Curvature", "Multiplier"]),
            ("Scale", &[]),
            ("Index Range", &[]),
            ("Combine Fields", &[]),
            ("Shape Field", &[]),
            ("Rotate", &[]),
        ],
    );
    n += confere_opcoes(T04, &m, &[("motion.falloff", "shape", &["Circle", "Rect"])]);
    // A `Rotation` só aparece em `Rect` — e o passo 6 diz que ela SOME em `Circle`.
    let rotacao = |s: &GraphViewSnapshot| {
        s.nodes
            .iter()
            .filter(|v| v.display_name == "Falloff")
            .any(|v| v.params.iter().any(|c| c.hint.label == "Rotation"))
    };
    assert!(
        !rotacao(&snap),
        "em Circle a `Rotation` do Falloff tem de estar escondida"
    );
    escolhe(&mut m, "motion.falloff", "shape", "Rect");
    let snap = cartoes(&mut m);
    n += confere(T04, &snap, &[("Falloff", &["Rotation"])]);
    assert_eq!(n, 7, "conferi {n}");
}

/// ⭐⭐⭐ **Tutorial 05 (Simulação)** — cena `=113` (cap. 1–5) e `=115` (cap. 6).
#[test]
fn every_row_the_simulation_tutorial_names_is_on_the_card() {
    let mut m = cena(T05, "113", &["force.attractor"]);
    let snap = cartoes(&mut m);
    let mut n = confere(
        T05,
        &snap,
        &[
            ("Collider", &["Shape", "Box Width", "Box Height"]),
            ("Wind", &["Acts As", "Gust", "Gust Frequency"]),
            ("Simulation Zone", &["Life Cycle"]),
            ("Simulation Step", &[]),
            ("Attractor", &[]),
        ],
    );
    n += confere_secoes(
        T05,
        &snap,
        &[
            ("Wind", &["Gust", "Timing"]),
            ("Attractor", &["Placement", "Falloff"]),
        ],
    );
    // O passo 8 diz que a `Gust` «já vem aberta» (e o 10 vai direto à `Timing`).
    confere_nascimento(&mut m, &[("Wind", "Gust", true), ("Wind", "Timing", true)]);
    n += confere_opcoes(
        T05,
        &m,
        &[("force.wind", "mode", &["Force", "Target Velocity"])],
    );
    let colisor = opcoes(&m, "sim.collide", "shape");
    for f in ["Plane", "Disc", "Bowl", "Box"] {
        assert!(
            colisor.contains(&f),
            "o Collider tem de oferecer `{f}`: {colisor:?}"
        );
        assert!(
            nomeia(T05, f),
            "o gate defende a forma `{f}` e o texto nao a nomeia"
        );
        n += 1;
    }
    escolhe(&mut m, "force.wind", "mode", "Target Velocity");
    let snap = cartoes(&mut m);
    n += confere(T05, &snap, &[("Wind", &["Air Resistance"])]);

    let mut m = cena(T05, "115", &[]);
    let snap = cartoes(&mut m);
    n += confere(
        T05,
        &snap,
        &[
            ("Friction 1: ROLA", &["Friction", "Rolling Friction"]),
            ("Bounciness 0,9: SALTA", &["Bounciness"]),
        ],
    );
    n += confere_secoes(T05, &snap, &[("Friction 1: ROLA", &["Collision"])]);
    // O passo 13 diz que a `Collision` «já vem aberta».
    confere_nascimento(&mut m, &[("Friction 1: ROLA", "Collision", true)]);
    assert_eq!(n, 22, "conferi {n}");
}
