//! **AS FIGURAS DO TUTORIAL DO CICLO 1** — geradas COZINHANDO os nós, nunca desenhadas à mão.
//!
//! ⚠️ É a lei do [doc 103 §3](../../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md): *as
//! imagens de um tutorial saem do próprio app*. Um desenho meu de um ecrã que não existe é a
//! família da cena de smoke que ensina o contrário do que acontece — e essa já custou uma
//! jornada a esta casa.
//!
//! Cada figura é a saída REAL do nó, cozida pelo `Cook` do produto e escrita em SVG.
//! A tabela dos controlos sai do **registry** (`param_ui` + `param_units`), pela mesma razão:
//! uma tabela escrita à mão envelhece no primeiro param novo.
//!
//! `cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture dump_arranjo_figures`

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::Edge;

/// ⚠️ **Relativo ao MANIFESTO, não ao processo.** `cargo test` corre com a cwd na raiz do
/// PACOTE (`shells/desktop`), então um caminho relativo à árvore escrevia as figuras em
/// `shells/desktop/docs/…` — medido em 2026-09-05, e um gerador que escreve no sítio errado é
/// pior que um que falha: ele diz «ok».
const DIR: &str = "../../docs/Motion Nodes/tutoriais/fig";
/// A moldura de uma figura, em unidades de mundo (a nuvem é centrada na origem).
const VIEW: f32 = 460.0;

/// Uma figura: o nó, os params que a compõem, e se ela precisa de uma entrada.
struct Fig {
    file: &'static str,
    node: &'static str,
    params: &'static [(&'static str, f32)],
    feed: bool,
}

const FIGS: &[Fig] = &[
    Fig {
        file: "grid",
        node: "motion.grid",
        params: &[
            ("rows", 12.0),
            ("cols", 12.0),
            ("gap_x", 34.0),
            ("gap_y", 34.0),
        ],
        feed: false,
    },
    Fig {
        file: "scatter",
        node: "motion.scatter",
        params: &[
            ("count", 240.0),
            ("width", 400.0),
            ("height", 400.0),
            ("seed", 7.0),
        ],
        feed: false,
    },
    Fig {
        file: "radial",
        node: "motion.distribute_radial",
        params: &[
            ("count", 180.0),
            ("rings", 5.0),
            ("radius", 200.0),
            ("inner", 40.0),
        ],
        feed: false,
    },
    Fig {
        file: "fibonacci",
        node: "motion.fibonacci",
        params: &[("count", 400.0), ("spacing", 10.5), ("angle", 137.5)],
        feed: false,
    },
    Fig {
        file: "lattice",
        node: "motion.lattice",
        params: &[("rows", 13.0), ("cols", 13.0), ("spacing", 32.0)],
        feed: false,
    },
    Fig {
        file: "voronoi",
        node: "motion.voronoi",
        params: &[
            ("count", 160.0),
            ("width", 400.0),
            ("height", 400.0),
            ("seed", 3.0),
            ("iterations", 6.0),
        ],
        feed: false,
    },
    Fig {
        file: "poisson",
        node: "motion.distribute_poisson",
        params: &[
            ("radius", 26.0),
            ("width", 400.0),
            ("height", 400.0),
            ("seed", 5.0),
        ],
        feed: false,
    },
    Fig {
        file: "curve",
        node: "motion.distribute_curve",
        params: &[
            ("count", 60.0),
            ("p0x", -200.0),
            ("p0y", -120.0),
            ("p1x", -90.0),
            ("p1y", 200.0),
            ("p2x", 90.0),
            ("p2y", -200.0),
            ("p3x", 200.0),
            ("p3y", 120.0),
        ],
        feed: false,
    },
    Fig {
        file: "clone",
        node: "motion.clone",
        params: &[("count", 8.0), ("distance", 46.0), ("angle", 25.0)],
        feed: true,
    },
];

fn cook_points(fig: &Fig) -> Vec<[f32; 2]> {
    let mut m = MotionState::new();
    let src = m.doc.graph.add_node(fig.node.to_string());
    for (p, v) in fig.params {
        m.doc.graph.set_param(src, *p, *v);
    }
    if fig.feed {
        let feed = m.doc.graph.add_node("motion.grid".to_string());
        for (p, v) in [
            ("rows", 3.0),
            ("cols", 3.0),
            ("gap_x", 90.0),
            ("gap_y", 90.0),
        ] {
            m.doc.graph.set_param(feed, p, v);
        }
        let _ = m.doc.graph.connect(Edge {
            from: (feed, 0),
            to: (src, 0),
            delayed: false,
        });
    }
    let out = m.doc.graph.add_node("motion.output".to_string());
    let _ = m.doc.graph.connect(Edge {
        from: (src, 0),
        to: (out, 0),
        delayed: false,
    });
    let mut cook = Cook::new();
    match cook.cook(&m.doc.graph, &m.registry, out, 0.0) {
        Ok(v) => match v.first().map(|c| c.as_stream()).and_then(|s| s.get("P")) {
            Some(Column::Vec2(p)) => p.clone(),
            _ => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

/// Um SVG quadrado com um ponto por elemento. ⚠️ Sem eixos nem grelha: a figura responde
/// *«que forma esta caixa faz?»*, e uma grelha por cima responderia a outra pergunta.
fn svg(points: &[[f32; 2]]) -> String {
    let r = if points.len() > 600 { 2.0 } else { 3.4 };
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.0} {:.0} {VIEW:.0} {VIEW:.0}\" \
         width=\"260\" height=\"260\" role=\"img\">\n\
         <rect x=\"{:.0}\" y=\"{:.0}\" width=\"{VIEW:.0}\" height=\"{VIEW:.0}\" rx=\"14\" fill=\"#141317\"/>\n",
        -VIEW / 2.0,
        -VIEW / 2.0,
        -VIEW / 2.0,
        -VIEW / 2.0
    );
    for p in points {
        // O y do mundo cresce para CIMA e o do SVG para baixo — a figura tem de mostrar o que
        // o artista vê no canvas, não o espelho dele.
        s.push_str(&format!(
            "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{r}\" fill=\"#c9a6ff\"/>\n",
            p[0], -p[1]
        ));
    }
    s.push_str("</svg>\n");
    s
}

#[test]
#[ignore = "gerador de figuras do tutorial"]
fn dump_arranjo_figures() {
    let dir = std::path::Path::new(DIR);
    std::fs::create_dir_all(dir).expect("a pasta das figuras");
    for fig in FIGS {
        let pts = cook_points(fig);
        assert!(
            !pts.is_empty(),
            "a figura `{}` saiu VAZIA — uma figura vazia num tutorial ensina o contrario do que acontece",
            fig.file
        );
        let path = dir.join(format!("{}.svg", fig.file));
        std::fs::write(&path, svg(&pts)).expect("escrever a figura");
        eprintln!("  {:>6} pontos │ {}", pts.len(), path.display());
    }

    // A tabela dos controlos, DERIVADA do registry — nunca escrita à mão.
    let m = MotionState::new();
    let mut html = String::from("<!-- GERADO por dump_arranjo_figures. Nao editar a mao. -->\n");
    for fig in FIGS {
        let mut aux = MotionState::new();
        let id = aux.doc.graph.add_node(fig.node.to_string());
        let tid = aux.doc.graph.node(id).expect("no'").type_id();
        let nome = m
            .registry
            .ui_manifest(tid)
            .map_or(fig.node, |u| u.display_name);
        html.push_str(&format!("<h4 id=\"p-{}\">{nome}</h4>\n<table class=\"params\">\n<tr><th>controlo</th><th>faixa</th><th>unidade</th></tr>\n", fig.file));
        let unidades = m.registry.param_units(tid);
        for h in m.registry.param_ui(tid).unwrap_or(&[]) {
            let u = unidades
                .iter()
                .flat_map(|us| us.iter())
                .find(|d| d.param == h.param)
                .map_or("", |d| match d.unit {
                    ph2d_node_registry::ParamUnit::Length => "px",
                    ph2d_node_registry::ParamUnit::Angle => "graus",
                    ph2d_node_registry::ParamUnit::Seconds => "s",
                    ph2d_node_registry::ParamUnit::Hertz => "Hz",
                    ph2d_node_registry::ParamUnit::Decibel => "dB",
                    _ => "",
                });
            // ⚠️ **A faixa do DESLIZANTE não é o tecto do param.** A 1.ª versão desta
            // tabela escrevia só `min..max` do hint e o PDF ensinava que o `Radius` de um
            // leque vai «até 20 px» — enquanto a caixa aceita `4000` e a própria figura deste
            // tutorial usa `200`. *Uma faixa que diz um máximo que não é o máximo é a família
            // do controlo que mente.* Quando há tecto duro e ele difere, a tabela di-lo.
            let duro = m.registry.param_hard_max(tid, h.param);
            let faixa = match duro {
                Some(d) if (d - h.max).abs() > f32::EPSILON => {
                    format!(
                        "{} a {} <span class=\"soft\">(digitável até {d})</span>",
                        h.min, h.max
                    )
                }
                _ => format!("{} a {}", h.min, h.max),
            };
            html.push_str(&format!(
                "<tr><td>{}</td><td>{faixa}</td><td>{u}</td></tr>\n",
                h.label
            ));
        }
        html.push_str("</table>\n");
    }
    let path = dir.join("params.html");
    std::fs::write(&path, html).expect("escrever a tabela");
    eprintln!("  tabela derivada │ {}", path.display());
}
