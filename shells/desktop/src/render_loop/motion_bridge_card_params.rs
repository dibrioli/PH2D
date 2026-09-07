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
/// ⭐⭐ **O TEXTO INTEIRO de cada param de texto**, publicado ao lado do snapshot.
///
/// ⚠️ **Publica-se mesmo VAZIO.** A caixa de edição do cartão recusa abrir sem semente — abrir
/// vazia sobre um valor que existe apagá-lo-ia com um `Enter` distraído —, e um param de texto
/// ainda por escrever tem de poder receber o primeiro caractere.
fn publish_card_texts(motion: &MotionState, snap: &ph2d_panel_motion_graph::GraphViewSnapshot) {
    let mut fora: Vec<(u32, &'static str, String)> = Vec::new();
    for node in &snap.nodes {
        let nid = ph2d_nodegraph::graph::NodeId(node.id);
        let Some(type_id) = motion.doc.graph.node(nid).map(|i| i.type_id()) else {
            continue;
        };
        for h in motion.registry.param_ui(type_id).unwrap_or(&[]) {
            if h.widget != ph2d_node_registry::ParamWidget::Text {
                continue;
            }
            let valor = motion
                .doc
                .graph
                .node_text_param_overrides(nid)
                .and_then(|m| m.get(h.param))
                .cloned()
                .unwrap_or_default();
            fora.push((node.id, h.param, valor));
        }
    }
    ph2d_panel_motion_graph::set_card_texts(fora);
}

pub(crate) fn stamp_card_params(
    motion: &MotionState,
    project: ph2d_editor::ProjectSettings,
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
        // ⭐⭐ **A faixa de cada row sai das MESMAS portas do painel** — o canal (uma magnitude
        // mede graus numa Rotation e unidades de mundo num X/Y), o fio (um `value.*` veste a
        // roupa de quem ele conduz), o `contain` (conter o valor vivo) e o tecto/piso digitáveis.
        // ⚠️ Reimplementar a escada aqui seria como o cartão passa a arrastar noutra faixa que
        // o painel, e o defeito que ela cura já foi reportado uma vez (doc 88).
        let type_name = motion
            .doc
            .graph
            .node(nid)
            .map(|i| i.type_name.clone())
            .unwrap_or_default();
        let channel = hints
            .iter()
            .any(|h| h.param == "channel")
            .then(|| param_value(motion, nid, "channel").round() as i32);
        // ⚠️ **O fio só se resolve para quem o LÊ.** O `wire_face` varre o mapa de params
        // dirigidos do documento inteiro, e aqui há um cartão por nó na vista — chamá-lo sempre
        // seria `O(nós × dirigidos)` por quadro. Quem declara `FromWire` é uma minoria conhecida,
        // e para toda a outra a resposta não é lida.
        let le_o_fio = hints.iter().any(|h| {
            ph2d_node_registry::unit_of(
                h.widget,
                motion.registry.param_unit_declared(type_id, h.param),
            ) == ph2d_node_registry::ParamUnit::FromWire
        });
        let wire = le_o_fio
            .then(|| params_wire::wire_face(motion, nid))
            .flatten();
        let ordem = motion.registry.param_group_order(type_id);
        let dobradas = motion.registry.param_groups_folded(type_id);
        let mut linhas: Vec<(Option<&'static str>, ph2d_panel_motion_graph::CardParam)> = hints
            .iter()
            .filter(|h| shown(h.param))
            .map(|h| {
                // ⭐⭐ **A FACE do artista** — a MESMA de `build_params_snapshot`: um comprimento
                // do mundo guarda-se em metros e mostra-se em px. Sem ela o cartão leria `0.94`
                // onde o painel lê `94 px`, em **109 de 454** rows escalares do catálogo.
                let unidade = ph2d_node_registry::unit_of(
                    h.widget,
                    motion.registry.param_unit_declared(type_id, h.param),
                );
                let face =
                    params_wire::display_face(unidade, channel, wire.map(|w| w.unit), project);
                let (min, max, step) = channel
                    .and_then(|ch| {
                        channel_range_override(&motion.registry, &type_name, h.param, ch)
                    })
                    .or_else(|| {
                        (ph2d_node_registry::unit_of(
                            h.widget,
                            motion.registry.param_unit_declared(type_id, h.param),
                        ) == ph2d_node_registry::ParamUnit::FromWire)
                            .then(|| wire.and_then(|w| w.range))
                            .flatten()
                    })
                    .unwrap_or((h.min, h.max, h.step));
                let (min, max) = contain(min, max, param_value(motion, nid, h.param));
                (h, min, max, step, face)
            })
            .map(|(h, min, max, step, face)| {
                // ⚠️ **Tudo vestido de uma vez** — o valor, as duas faixas e o passo. Vestir só
                // o número deixaria o slider a arrastar noutra escala que a que ele mostra, que
                // é o defeito que o doc do `in_display` do painel já nomeia.
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "a face e' um f64 de escala"
                )]
                let vestir = |v: f32| (f64::from(v) * face.scale) as f32;
                ph2d_panel_motion_graph::CardParam {
                    hint: *h,
                    value: vestir(param_value(motion, nid, h.param)),
                    min: vestir(min),
                    max: vestir(max),
                    step: vestir(step),
                    // O tecto/piso DIGITÁVEIS, com a mesma lei do painel: um hard que ficasse do lado
                    // de dentro do arrasto desfaria em silêncio um valor que o dedo ainda alcança.
                    hard_max: vestir(
                        motion
                            .registry
                            .param_hard_max(type_id, h.param)
                            .unwrap_or(max)
                            .max(max),
                    ),
                    hard_min: vestir(
                        motion
                            .registry
                            .param_hard_min(type_id, h.param)
                            .unwrap_or(min)
                            .min(min),
                    ),
                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "a face e' um f64 de escala"
                    )]
                    face_scale: face.scale as f32,
                    face_suffix: face.suffix,
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
                    // O id da amostra — com o NÓ dentro, para dois cartões do mesmo tipo não
                    // pedirem o mesmo selector. Ver `color::card_swatch_id`.
                    swatch_id: match h.widget {
                        ph2d_node_registry::ParamWidget::Color { channels } => {
                            Some(super::super::color::card_swatch_id(nid.0, channels[0]).0)
                        }
                        _ => None,
                    },
                    text: card_text(motion, nid, h),
                }
            })
            .map(|c| (motion.registry.param_group(type_id, c.hint.param), c))
            .collect();
        linhas.sort_by_key(|(g, _)| {
            g.map_or(0, |g| {
                1 + ordem.iter().position(|o| *o == g).unwrap_or(ordem.len())
            })
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
    // ⭐ E o texto INTEIRO ao lado, para a caixa de edição do cartão nunca abrir com o truncado.
    publish_card_texts(motion, snap);
    // ⭐ E as OPÇÕES de cada selector, para as setas e a lista do dropdown (report 07/09).
    super::choices::publish_card_choices(motion, snap);
}

/// Os gates desta faixa — irmão de teste, como em todo o módulo. ⚠️ **Dois ficheiros, duas
/// perguntas** (HR-18): *quais* params o cartão mostra, e *que número* cada um carrega.
#[cfg(test)]
#[path = "motion_bridge_card_params_tests.rs"]
mod card_params_tests;

#[cfg(test)]
#[path = "motion_bridge_card_range_tests.rs"]
mod card_range_tests;

#[cfg(test)]
#[path = "motion_bridge_card_census.rs"]
mod card_census;

/// ⭐⭐ **O QUE UMA ROW DE TEXTO MOSTRA NO CARTÃO** — e as espécies que deliberadamente **não**
/// mostram nada.
///
/// ⚠️ **O valor sai da MESMA expressão que a row do painel usa** (o override de texto do nó,
/// vazio quando não há) — ver `params_text_rows`. Uma segunda forma de ler o mesmo campo seria
/// como o cartão e o painel passam a discordar sobre o que está escolhido.
///
/// ⛔ **Uma CURVA, um GRADIENTE e uma PALETA não trazem texto**, e não é esquecimento: o valor
/// deles é uma **serialização** (`0.0,0.0;0.5,1.0;…`), e pô-la na row encheria o cartão de um
/// texto que não responde a pergunta nenhuma. Para essas o selo continua a ser a resposta.
///
/// ⚠️ **De um FICHEIRO mostra-se o NOME, não o caminho.** A coluna do valor tem ~12 caracteres
/// e o elidor corta pelo FIM — um caminho absoluto mostraria `/home/enio/Doc…`, que é
/// exactamente a metade que não identifica ficheiro nenhum. *Quando o espaço obriga a cortar,
/// o que fica tem de ser a metade que responde.*
fn card_text(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    h: &ph2d_node_registry::ParamUiHint,
) -> ph2d_panel_motion_graph::RowText {
    use ph2d_node_registry::ParamWidget as W;
    let vazio = ph2d_panel_motion_graph::RowText::default();
    let bruto = motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(h.param))
        .map(String::as_str)
        .unwrap_or_default();
    if bruto.is_empty() {
        return vazio;
    }
    match h.widget {
        // ⭐⭐ **De um CANAL mostra-se o NOME que o artista lê no painel, não a coluna crua.**
        // O selector segmentado do painel diz *«Speed»*; a row que dissesse `vel` seria a mesma
        // escolha com dois nomes, e o clique que anda pela lista passaria a ler-se como se
        // tivesse escolhido outra coisa. Fora da lista curada (uma coluna escrita à mão ou vinda
        // da corrente de cima) o nome É a coluna, e é isso que se mostra.
        W::Channels {
            mode_param,
            channels,
        } => {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "o `mode` e' um inteiro pequeno guardado num f32, como no painel"
            )]
            let modo = param_value(motion, nid, mode_param).round() as i32;
            let rotulo = channels
                .iter()
                .find(|c| c.column == bruto && c.mode == modo)
                .map_or(bruto, |c| c.label);
            ph2d_panel_motion_graph::RowText::new(rotulo)
        }
        W::Text | W::Source => ph2d_panel_motion_graph::RowText::new(bruto),
        W::File { .. } => std::path::Path::new(bruto)
            .file_name()
            .and_then(|n| n.to_str())
            .map_or(vazio, ph2d_panel_motion_graph::RowText::new),
        _ => vazio,
    }
}
