//! ⭐⭐⭐ **A TABELA DE CONTROLOS DE UM TUTORIAL — a porta ÚNICA**, derivada da mesma função de
//! onde o painel e o cartão tiram os números ([doc 103 §3](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)).
//!
//! ⚠️ **Ela nasceu dentro do gerador do ciclo 1 e saiu de lá no ciclo 2, antes de existir uma
//! segunda cópia.** Uma tabela de referência escrita duas vezes diverge em silêncio, e o lado
//! que envelhece é o que o artista lê — a mesma lei que este módulo já pagou no `Radius` que
//! dizia «até 20 px» sobre uma caixa que aceita `4000`.
//!
//! O que ela promete, e que só esta porta consegue cumprir:
//! - os números na **FACE do artista** (`94 px`, não `0,94`) — porque sai do `build_params_snapshot`;
//! - a faixa do **arrasto** e, quando diferem, o tecto **digitável**;
//! - o que existe mas **só noutro modo**, derivado dos `ParamGate` do registry.

use crate::motion_state::MotionState;

/// Um número para a tabela: inteiro quando é inteiro, senão até três casas.
fn fmt_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v.round() as i64)
    } else {
        let t = format!("{v:.3}");
        t.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// `entradas = [(âncora, tipo do nó)]` → o HTML da secção «o que cada controlo faz».
///
/// ⚠️ **Lista o que o nó MOSTRA no estado de omissão** (a mesma lei de visibilidade do cartão):
/// um controlo escondido por modo não entra na tabela — entra na nota `Só noutro modo:`, que
/// diz **o que o acende**. *Numa tabela de referência a omissão é pior que no painel:* ali o
/// controlo está a um clique e vê-se aparecer; aqui ele simplesmente não existe.
pub fn derive(entradas: &[(&str, &str)]) -> String {
    let mut html = String::from("<!-- GERADO pelo gerador do tutorial. Nao editar a mao. -->\n");
    for (ancora, node) in entradas {
        let mut aux = MotionState::new();
        let id = aux.doc.graph.add_node((*node).to_string());
        let tid = aux.doc.graph.node(id).expect("no'").type_id();
        let nome = aux
            .registry
            .ui_manifest(tid)
            .map_or(*node, |u| u.display_name);
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let painel = crate::motion_bridge::params::build_params_snapshot(
            &aux,
            ph2d_editor::ProjectSettings::default(),
        );
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
        let painel = painel.unwrap_or_else(|| panic!("o painel de `{node}` monta"));
        html.push_str(&format!(
            "<h4 id=\"p-{ancora}\">{nome}</h4>\n<table class=\"params\">\n\
             <tr><th>controlo</th><th>faixa</th><th>unidade</th></tr>\n"
        ));
        let mut linhas = 0usize;
        for row in &painel.rows {
            let Some((label, faixa, unidade)) = celula(row) else {
                continue;
            };
            linhas += 1;
            html.push_str(&format!(
                "<tr><td>{label}</td><td>{faixa}</td><td>{unidade}</td></tr>\n"
            ));
        }
        assert!(linhas > 0, "`{node}` nao pos uma linha na tabela");
        html.push_str("</table>\n");
        if let Some(nota) = so_noutro_modo(&aux, tid, &painel) {
            html.push_str(&format!("<p class=\"soft\">Só noutro modo: {nota}.</p>\n"));
        }
    }
    html
}

/// `(rótulo, faixa, unidade)` de uma row — `None` para as que não são um controlo de valor.
fn celula(row: &ph2d_panel_motion_params::ParamRow) -> Option<(String, String, String)> {
    use ph2d_panel_motion_params::ParamRow;
    Some(match row {
        // ⚠️ **A faixa do DESLIZANTE não é o tecto do param**, e o tutorial do ciclo 1 ensinou
        // o número errado antes de esta coluna existir.
        ParamRow::Scalar(r) => {
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
        // ⛔⛔ **O sufixo é PEDIDO à porta da unidade, nunca escrito aqui.** Ele esteve
        // literal (`"graus"`) e a tabela do ciclo 5 imprimiu **as duas palavras na mesma
        // página**: o `Angle` do `Collider` saía `deg` (é um `Scalar` com
        // [`ParamUnit::Angle`], que já perguntava à porta) e o do `Wind` saía `graus`, só
        // porque um nó usa o widget de ângulo e o outro um deslizante. *Duas palavras para a
        // mesma unidade, decididas pelo widget que o nó calhou usar.*
        ParamRow::Angle(r) => (
            r.label.clone(),
            format!("{} a {}", fmt_num(r.min_deg), fmt_num(r.max_deg)),
            ph2d_node_registry::ParamUnit::Angle
                .fixed_suffix()
                .unwrap_or_default()
                .to_string(),
        ),
        ParamRow::Seed(r) => (
            r.label.clone(),
            format!("{} a {}", fmt_num(r.min), fmt_num(r.max)),
            String::new(),
        ),
        ParamRow::Toggle(r) => (r.label.clone(), "liga / desliga".to_string(), String::new()),
        // ⭐ Um enum não tem faixa: tem OPÇÕES, e é isso que serve a quem lê o tutorial.
        ParamRow::Enum(r) => (r.label.clone(), r.labels.join(" · "), String::new()),
        _ => return None,
    })
}

/// A nota dos controlos que existem e o modo de omissão não acende — derivada dos `ParamGate`.
fn so_noutro_modo(
    aux: &MotionState,
    tid: ph2d_nodegraph::node::NodeTypeId,
    painel: &ph2d_panel_motion_params::ParamsSnapshot,
) -> Option<String> {
    let mostrados: Vec<&str> = painel.rows.iter().flat_map(|r| r.params()).collect();
    let hints = aux.registry.param_ui(tid).unwrap_or(&[]);
    let rotulo = |nome: &'static str| -> &'static str {
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
    (!notas.is_empty()).then(|| notas.join(" · "))
}

/// ⭐⭐⭐ **UMA UNIDADE, UMA PALAVRA — em toda a tabela.**
///
/// ⛔⛔ Achado ao **ler o PDF do ciclo 5**: a mesma página imprimia `deg` no `Angle` do
/// `Collider` e `graus` no `Angle` do `Wind`. Os dois são ângulos; o que os separava era o
/// **widget** — um é um `Scalar` com [`ph2d_node_registry::ParamUnit::Angle`] (e essa rota já
/// perguntava à porta da unidade), o outro é um `ParamRow::Angle`, cujo sufixo estava escrito
/// à mão aqui. *Uma lei escrita em dois sítios ainda não é uma lei; só uma PORTA é.*
///
/// ⚠️ **Nenhum gate do repo o via, e nenhum o veria:** o `no_tofu_glyphs` mede glifos, o
/// `hr15_no_hardcoded_ui_strings` mede os widgets, e esta tabela é um artefacto de
/// documentação gerado por um teste `#[ignore]`. O que a apanhou foi olhar para a página.
///
/// A régua aqui é derivada: para cada unidade com face fixa, a tabela de um grupo que a use
/// não pode conter nenhuma **outra** palavra para ela.
#[test]
fn one_unit_one_word_in_a_generated_table() {
    use ph2d_node_registry::ParamUnit;
    // Um nó com o widget de ângulo (`force.wind`) e um com um escalar em graus
    // (`sim.collide`) — é exactamente o par que discordava.
    let html = derive(&[("wind", "force.wind"), ("collider", "sim.collide")]);
    let deg = ParamUnit::Angle
        .fixed_suffix()
        .expect("o ângulo tem face fixa");
    assert!(
        html.contains(&format!("<td>{deg}</td>")),
        "a tabela tem de trazer a face canónica do ângulo (`{deg}`), e trouxe:\n{html}"
    );
    for outra in ["graus", "degrees", "°"] {
        assert!(
            !html.contains(&format!("<td>{outra}</td>")),
            "a tabela imprime `{outra}` para uma unidade cuja face canónica é `{deg}` -- duas \
             palavras para a mesma unidade, na mesma página, decididas pelo widget que o nó \
             calhou usar"
        );
    }
}
