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

    // ⭐⭐⭐ **A TABELA DOS CONTROLOS, DERIVADA DA MESMA PORTA QUE O CARTÃO LÊ.**
    //
    // ⚠️ **Ela era derivada do `param_ui` CRU, e isso passou a ser uma mentira em 2026-09-05**,
    // quando o cartão passou a vestir a FACE do artista: das 39 células deste tutorial, **23**
    // têm escala ou sufixo — o `Gap X` de uma grade mostra-se em **px** (×100) e a tabela
    // imprimia o número em unidades de mundo. *Um tutorial que diz «0 a 4» sobre um controlo que
    // mostra «0 a 400 px» ensina o contrário do que acontece.*
    //
    // ⇒ a tabela sai agora do `build_params_snapshot`, que é a porta de onde o painel e o cartão
    // tiram os números — então ela não pode divergir do que o artista vê.
    //
    // ⚠️ **Ela lista o que o nó MOSTRA no estado de omissão** (a mesma lei de visibilidade do
    // cartão): um controlo escondido por modo não entra, exactamente como não aparece no nó
    // acabado de largar.
    let mut html = String::from("<!-- GERADO por dump_arranjo_figures. Nao editar a mao. -->\n");
    for fig in FIGS {
        let mut aux = MotionState::new();
        let id = aux.doc.graph.add_node(fig.node.to_string());
        let tid = aux.doc.graph.node(id).expect("no'").type_id();
        let nome = aux
            .registry
            .ui_manifest(tid)
            .map_or(fig.node, |u| u.display_name);
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let painel = crate::render_loop::motion_bridge::params::build_params_snapshot(
            &aux,
            ph2d_editor::ProjectSettings::default(),
        );
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
        let painel = painel.unwrap_or_else(|| panic!("o painel de `{}` monta", fig.node));
        html.push_str(&format!(
            "<h4 id=\"p-{}\">{nome}</h4>\n<table class=\"params\">\n<tr><th>controlo</th><th>faixa</th><th>unidade</th></tr>\n",
            fig.file
        ));
        let mut linhas = 0usize;
        for row in &painel.rows {
            let (label, faixa, unidade) = match row {
                // ⚠️ **A faixa do DESLIZANTE não é o tecto do param.** A 1.ª versão desta tabela
                // escrevia só `min..max` e o PDF ensinava que o `Radius` de um leque vai «até 20
                // px» — enquanto a caixa aceita `4000` e a figura deste tutorial usa `200`.
                ph2d_panel_motion_params::ParamRow::Scalar(r) => {
                    let faixa = if (r.hard_max - r.max).abs() > 1e-6 {
                        format!(
                            "{} a {} <span class=\"soft\">(digitável até {})</span>",
                            fmt_num(r.min),
                            fmt_num(r.max),
                            fmt_num(r.hard_max)
                        )
                    } else {
                        format!("{} a {}", fmt_num(r.min), fmt_num(r.max))
                    };
                    (r.label.clone(), faixa, r.display.suffix.to_string())
                }
                ph2d_panel_motion_params::ParamRow::Angle(r) => (
                    r.label.clone(),
                    format!("{} a {}", fmt_num(r.min_deg), fmt_num(r.max_deg)),
                    "graus".to_string(),
                ),
                ph2d_panel_motion_params::ParamRow::Seed(r) => (
                    r.label.clone(),
                    format!("{} a {}", fmt_num(r.min), fmt_num(r.max)),
                    String::new(),
                ),
                ph2d_panel_motion_params::ParamRow::Toggle(r) => {
                    (r.label.clone(), "liga / desliga".to_string(), String::new())
                }
                // ⭐ Um enum não tem faixa: tem OPÇÕES, e é isso que serve a quem lê o tutorial.
                ph2d_panel_motion_params::ParamRow::Enum(r) => {
                    (r.label.clone(), r.labels.join(" · "), String::new())
                }
                _ => continue,
            };
            linhas += 1;
            html.push_str(&format!(
                "<tr><td>{label}</td><td>{faixa}</td><td>{unidade}</td></tr>\n"
            ));
        }
        assert!(linhas > 0, "`{}` nao pos uma linha na tabela", fig.node);
        html.push_str("</table>\n");
        // ⭐⭐ **E OS CONTROLOS QUE SÓ APARECEM NOUTRO MODO.**
        //
        // ⚠️ A tabela lista o que o nó mostra por omissão, e por isso o `Hole` de uma grade
        // sumia dela — ele só é pintado com `Shape = Ring`. *Numa tabela de REFERÊNCIA a
        // omissão é pior que no painel:* ali um controlo escondido está a um clique e vê-se
        // aparecer; aqui ele simplesmente não existe, e o artista não sabe o que procurar.
        // ⇒ a nota diz o que existe **e o que o acende**, derivada dos `ParamGate` do registry.
        let mostrados: Vec<&str> = painel.rows.iter().flat_map(|r| r.params()).collect();
        let hints = aux.registry.param_ui(tid).unwrap_or(&[]);
        let rotulo = move |nome: &'static str| -> &'static str {
            hints
                .iter()
                .find(|h| h.param == nome)
                .map_or(nome, |h| h.label)
        };
        let mut notas: Vec<String> = Vec::new();
        for g in aux.registry.param_gates(tid).unwrap_or(&[]) {
            if mostrados.contains(&g.param) {
                continue; // já está na tabela: o modo de omissão acende-o
            }
            // Os VALORES do gate são índices do enum que o acende — o artista lê nomes.
            let opcoes = hints.iter().find(|h| h.param == g.when).map(|h| h.widget);
            let quais: Vec<String> = g
                .values
                .iter()
                .map(|v| match opcoes {
                    Some(ph2d_node_registry::ParamWidget::Enum { labels }) => labels
                        .get(*v as usize)
                        .map_or_else(|| v.to_string(), |s| (*s).to_string()),
                    _ => v.to_string(),
                })
                .collect();
            notas.push(format!(
                "<b>{}</b> aparece quando <b>{}</b> é {}",
                rotulo(g.param),
                rotulo(g.when),
                quais.join(" ou ")
            ));
        }
        if !notas.is_empty() {
            html.push_str(&format!(
                "<p class=\"soft\">Só noutro modo: {}.</p>\n",
                notas.join(" · ")
            ));
        }
    }
    let path = dir.join("params.html");
    std::fs::write(&path, html).expect("escrever a tabela");
    eprintln!("  tabela derivada │ {}", path.display());
}

/// Um número para a tabela: inteiro quando é inteiro, senão até três casas — o mesmo critério
/// do `format_number` da casa, sem os zeros que uma faixa não precisa.
fn fmt_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        let t = format!("{v:.3}");
        t.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// **A TABELA DO TUTORIAL AINDA DIZ O QUE O CARTÃO MOSTRA?** — a pergunta que a FACE abriu
/// (2026-09-05): o cartão passou a vestir a unidade do artista (`94 px` onde o documento tem
/// `0,94`), e a tabela do tutorial é derivada do `param_ui` **cru**. Se algum param do grupo
/// tiver escala de face, a tabela impressa passa a mentir sobre a faixa.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins does_the_tutorial_table_still_match_the_card -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de censo, nao um gate"]
fn does_the_tutorial_table_still_match_the_card() {
    let mut vestidos: Vec<String> = Vec::new();
    let mut total = 0usize;
    for fig in FIGS {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(fig.node.to_string());
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let Some(painel) = crate::render_loop::motion_bridge::params::build_params_snapshot(
            &m,
            ph2d_editor::ProjectSettings::default(),
        ) else {
            continue;
        };
        for row in &painel.rows {
            let ph2d_panel_motion_params::ParamRow::Scalar(r) = row else {
                continue;
            };
            total += 1;
            if (r.display.scale - 1.0).abs() > 1e-9 || !r.display.suffix.is_empty() {
                vestidos.push(format!(
                    "{}::{} escala {:.1} sufixo {:?}",
                    fig.node, r.name, r.display.scale, r.display.suffix
                ));
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    eprintln!(
        "\n  {total} controlos nos {} nos do tutorial · {} com FACE (escala ou sufixo):\n",
        FIGS.len(),
        vestidos.len()
    );
    for v in &vestidos {
        eprintln!("    {v}");
    }
    eprintln!();
}
