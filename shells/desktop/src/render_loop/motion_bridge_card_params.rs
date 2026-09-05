//! **O QUE CADA CARTÃO CARREGA** — a faixa de params desenhada dentro do nó (ciclo 1 da
//! dinâmica, [doc 103](../../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md); decisão do
//! Enio de 2026-09-05: *os params dos nós são desenhados nos nós e o painel lateral sai*).
//!
//! ⚠️ Irmão de [`super`] por RESPONSABILIDADE e não por contagem: o pai responde *«o que o
//! PAINEL mostra»*, este *«o que o CARTÃO carrega»* — e as duas respostas saem das **mesmas
//! portas** (`params_visible::shown_params` para a visibilidade, `param_value` para o número,
//! `param_group`/`param_group_order`/`param_groups_folded` para as secções). Uma segunda
//! conjunção de gates aqui seria exactamente como um param passa a aparecer num sítio do app e
//! não noutro.

use super::*;

/// ⭐⭐⭐ **OS PARAMS QUE CADA CARTÃO MOSTRA** — o ciclo 1 da dinâmica
/// ([doc 103](../../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md); decisão do Enio,
/// 2026-09-05: os params vivem no cartão e o painel lateral sai).
///
/// ⚠️ **A porta da visibilidade é a MESMA do painel** ([`params_visible::shown_params`]) e a do
/// valor também ([`edit::param_value`]). Reimplementar a conjunção dos três gates aqui seria
/// exactamente como um param passa a aparecer num sítio do app e não noutro — o defeito que o
/// doc daquela função já nomeia.
///
/// ⚠️ **Corre ANTES do `fold`**, como o `readout::stamp`: aqui todo nó da vista é um nó
/// simples, e um cartão de subgrafo (que o fold cria depois) não tem params próprios.
///
/// ⚠️ **Zero `String`**: [`ph2d_panel_motion_graph::CardParam`] leva o `ParamUiHint` do
/// registry (`Copy`, `&'static`) e o `f32` vivo — a medição do doc 103 §7 diz porquê (uma row
/// custa 13,5 µs, e 20 cartões × 5 rows seriam 200 alocações por quadro).
pub(crate) fn stamp_card_params(
    motion: &MotionState,
    snap: &mut ph2d_panel_motion_graph::GraphViewSnapshot,
) {
    for node in &mut snap.nodes {
        let nid = ph2d_nodegraph::graph::NodeId(node.id);
        let Some(type_id) = motion.doc.graph.node(nid).map(|i| i.type_id()) else {
            continue;
        };
        let Some(hints) = motion.registry.param_ui(type_id) else {
            continue;
        };
        let shown = params_visible::shown_params(motion, nid);
        let sources = motion.doc.graph.param_sources(nid);
        // ⚠️ **Um hint de COR ancora quatro params** (`channels`) e mostra UMA row com a
        // amostra — nunca quatro números (um `0.50` linear lê-se como cinzento claro).
        //
        // ⛔⛔ **E a supressão dos canais crus foi ESCRITA, MEDIDA e APAGADA:** a mutação que a
        // removia SOBREVIVEU ao gate, e o censo do registry diz porquê — das **5** cores
        // declaradas, **0** têm um canal não-âncora com `ParamUiHint` próprio. Os canais nunca
        // entram nesta lista porque nunca são declarados, e o filtro era código que não podia
        // disparar. O que fica é o gate, agora um CENSO com a contagem: se um dia alguém
        // declarar `g` como hint, ele diz — e aí a supressão volta, com um caso que a exerce.
        // ⭐⭐ **AS SECÇÕES, pela MESMA lei do painel** (`sections::split_into_sections`): a
        // ordem é a que o registry declara (`param_group_order`), os sem grupo vêm primeiro, e
        // a ordenação é ESTÁVEL para a ordem dos hints sobreviver dentro de cada grupo.
        let ordem = motion.registry.param_group_order(type_id);
        let dobradas = motion.registry.param_groups_folded(type_id);
        let mut linhas: Vec<(Option<&'static str>, ph2d_panel_motion_graph::CardParam)> = hints
            .iter()
            .filter(|h| shown(h.param))
            .map(|h| ph2d_panel_motion_graph::CardParam {
                hint: *h,
                value: param_value(motion, nid, h.param),
                driven: sources.is_some_and(|s| s.contains_key(h.param)),
                // A amostra em bytes sRGB — pela MESMA porta que semeia o picker do painel
                // (os params guardam RGBA linear).
                swatch: match h.widget {
                    ph2d_node_registry::ParamWidget::Color { channels } => {
                        Some(super::super::color::linear_rgba_to_srgb8([
                            param_value(motion, nid, channels[0]),
                            param_value(motion, nid, channels[1]),
                            param_value(motion, nid, channels[2]),
                            param_value(motion, nid, channels[3]),
                        ]))
                    }
                    _ => None,
                },
            })
            .map(|c| (motion.registry.param_group(type_id, c.hint.param), c))
            .collect();
        linhas.sort_by_key(|(g, _)| {
            g.map_or(0, |g| 1 + ordem.iter().position(|o| *o == g).unwrap_or(ordem.len()))
        });

        // A dobra EFECTIVA: o que o artista tocou, senão o que o registry declarou.
        let aberta = |g: &'static str| {
            motion
                .card_sections
                .get(&(node.id, g))
                .copied()
                .unwrap_or(!dobradas.contains(&g))
        };
        let mut params = Vec::with_capacity(linhas.len());
        let mut sections: Vec<ph2d_panel_motion_graph::CardSection> = Vec::new();
        let mut grupo_anterior: Option<&'static str> = None;
        for (g, c) in linhas {
            match g {
                None => params.push(c),
                Some(g) => {
                    if grupo_anterior != Some(g) {
                        grupo_anterior = Some(g);
                        sections.push(ph2d_panel_motion_graph::CardSection {
                            title: g,
                            // Aponta para onde a primeira row DESTA secção fica na lista
                            // filtrada — numa secção fechada, para onde ela ficaria.
                            at: u16::try_from(params.len()).unwrap_or(u16::MAX),
                            open: aberta(g),
                            hidden: 0,
                        });
                    }
                    let s = sections.last_mut().expect("acabou de ser empurrada");
                    if s.open {
                        params.push(c);
                    } else {
                        s.hidden = s.hidden.saturating_add(1);
                    }
                }
            }
        }
        node.params = params;
        node.sections = sections;
    }
}

/// Os gates desta faixa — irmão de teste, como em todo o módulo.
#[cfg(test)]
#[path = "motion_bridge_card_params_tests.rs"]
mod card_params_tests;
