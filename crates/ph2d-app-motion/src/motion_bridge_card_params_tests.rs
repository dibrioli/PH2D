//! **OS PARAMS DO CARTÃO** — os gates do ciclo 1 da dinâmica
//! ([doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)): decisão do Enio de
//! 2026-09-05, *os params dos nós são desenhados nos nós*.
//!
//! ⚠️ Irmão de `motion_bridge_params_visible_tests` por RESPONSABILIDADE: aquele pergunta
//! *«quando é que uma row aparece?»* (as três famílias de gate) e este *«o que é que o CARTÃO
//! recebe?»* — a mesma porta de visibilidade, um segundo consumidor.

use super::*;
use crate::motion_state::MotionState;

/// **ABRE TODAS AS SECÇÕES** de um nó — o que torna a pergunta *«o cartão OFERECE isto?»*
/// diferente de *«o cartão MOSTRA isto agora?»*.
///
/// ⚠️ Os dois gates de alcance abaixo pediam a segunda por engano até as secções existirem: uma
/// row dobrada não está na lista e continua **alcançável** (um clique no cabeçalho). *Uma régua
/// de alcance que mede a visibilidade acusa toda a dobra do produto.*
pub(super) fn open_every_section(m: &mut MotionState, id: ph2d_nodegraph::graph::NodeId) {
    let Some(tid) = m.doc.graph.node(id).map(|i| i.type_id()) else {
        return;
    };
    let grupos: Vec<&'static str> = m
        .registry
        .param_ui(tid)
        .map(|hs| {
            hs.iter()
                .filter_map(|h| m.registry.param_group(tid, h.param))
                .collect()
        })
        .unwrap_or_default();
    for g in grupos {
        m.card_sections.insert((id.0, g), true);
    }
}

/// Um nó solto, cozido até à lista de params que o CARTÃO dele recebe. Devolve também o estado
/// e o id, porque os gates precisam de perguntar à mesma árvore (dois `MotionState` seriam duas
/// árvores, e o gate mediria a coincidência entre elas).
fn one_node(
    type_name: &str,
) -> (
    MotionState,
    ph2d_nodegraph::graph::NodeId,
    Vec<ph2d_panel_motion_graph::CardParam>,
) {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node(type_name.to_string());
    open_every_section(&mut m, id);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
    let params = snap
        .nodes
        .iter()
        .find(|n| n.id == id.0)
        .map(|n| n.params.clone())
        .unwrap_or_default();
    (m, id, params)
}

/// **O CARTÃO RECEBE OS PARAMS DO NÓ** — e não uma lista vazia. FALSIFICADO por
/// `stamp_card_params` não ser chamada (a cena fica com cartões nus e o painel lateral volta a
/// ser o único sítio onde um param existe, que é o que esta wave apaga).
#[test]
fn a_card_carries_the_params_of_its_node() {
    let (_, _, p) = one_node("motion.grid");
    assert!(
        p.len() >= 4,
        "o Grid tem rows/cols/gap_x/gap_y no cartao, e nao {}",
        p.len()
    );
    assert!(
        p.iter().any(|c| c.hint.param == "rows"),
        "o param `rows` esta' na lista do cartao"
    );
}

/// ⭐⭐ **UMA COR É UMA AMOSTRA, NÃO UM NÚMERO** — e o resto do gate é um **CENSO**, porque a
/// mutação me corrigiu.
///
/// Eu escrevera um filtro que suprimia os canais crus de um hint [`ParamWidget::Color`], e a
/// mutação que o removia **SOBREVIVEU**. O censo do registry diz porquê: das **5** cores
/// declaradas, **0** têm um canal não-âncora com `ParamUiHint` próprio — os canais nunca
/// entram na lista do cartão porque nunca são declarados. O filtro foi apagado; o que sobra é
/// esta contagem, que **fala** no dia em que o mundo passar a conter o caso.
///
/// FALSIFICADO por: a amostra deixar de ser preenchida (a cor passa a ler-se `0.50`), ou por
/// alguém declarar um hint para um canal cru sem repor a supressão.
#[test]
fn a_colour_is_a_swatch_and_no_raw_channel_declares_a_row() {
    let (m, id, p) = one_node("motion.tint");
    let hints = m
        .registry
        .param_ui(m.doc.graph.node(id).expect("no'").type_id())
        .expect("o tint declara hints");
    let visivel = params_visible::shown_params(&m, id);
    // ⚠️ Só as âncoras VISÍVEIS: um modo pode esconder a segunda cor, e exigir a row dela
    // mediria o gate de modo em vez desta lei.
    let anchors: Vec<&str> = hints
        .iter()
        .filter(|h| matches!(h.widget, ph2d_node_registry::ParamWidget::Color { .. }))
        .map(|h| h.param)
        .filter(|a| visivel(a))
        .collect();
    assert!(
        !anchors.is_empty(),
        "o tint mostra pelo menos uma cor no modo de omissao"
    );
    for a in &anchors {
        let row = p
            .iter()
            .find(|c| c.hint.param == *a)
            .unwrap_or_else(|| panic!("a ancora de cor `{a}` tem row no cartao"));
        assert!(
            row.swatch.is_some(),
            "a row da ancora `{a}` leva a AMOSTRA, nao um numero"
        );
    }

    // O CENSO, sobre o registry inteiro — a população que a supressão apagada serviria.
    let mut cores = 0usize;
    let mut canais_com_hint = Vec::new();
    for (_, hints) in todos_os_hints(&m) {
        let declarados: Vec<&str> = hints.iter().map(|h| h.param).collect();
        for h in hints {
            let ph2d_node_registry::ParamWidget::Color { channels } = h.widget else {
                continue;
            };
            cores += 1;
            for c in channels {
                if c != h.param && declarados.contains(&c) {
                    canais_com_hint.push(c);
                }
            }
        }
    }
    assert!(
        cores >= 5,
        "o registry declara pelo menos as 5 cores medidas em 2026-09-05, e nao {cores}"
    );
    assert!(
        canais_com_hint.is_empty(),
        "de {cores} cores, {} canais crus passaram a declarar hint proprio ({canais_com_hint:?}) \
         -- o cartao vai mostrar um NUMERO onde pertence uma amostra; repor a supressao",
        canais_com_hint.len()
    );
}

/// Os hints de todo tipo de nó do registry — a população dos censos deste ficheiro.
pub(super) fn todos_os_hints(
    m: &MotionState,
) -> Vec<(&'static str, &'static [ph2d_node_registry::ParamUiHint])> {
    m.registry
        .manifests()
        .filter_map(|man| m.registry.param_ui(man.id).map(|h| (man.name, h)))
        .collect()
}

/// **O CARTÃO E O PAINEL MOSTRAM A MESMA LISTA** — a porta de visibilidade é uma só. Um param
/// escondido por modo (`ParamGate`) não pode aparecer num sítio do app e não noutro.
/// FALSIFICADO por o cartão deixar de filtrar por `shown_params`.
#[test]
fn the_card_and_the_panel_agree_on_which_params_are_visible() {
    let mut m = MotionState::new();
    // O `motion.noise` tem gates de modo (`range_mode` revela `min`/`max`), que é a fixtura
    // que contém o fenómeno — um nó sem gates concordaria por vazio.
    let id = m.doc.graph.add_node("motion.noise".to_string());
    open_every_section(&mut m, id);
    for modo in [0.0_f32, 1.0] {
        m.doc.graph.set_param(id, "range_mode", modo);
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
        let no_cartao: Vec<&str> = snap
            .nodes
            .iter()
            .find(|n| n.id == id.0)
            .map(|n| n.params.iter().map(|c| c.hint.param).collect())
            .unwrap_or_default();
        let visivel = params_visible::shown_params(&m, id);
        let hints = m
            .registry
            .param_ui(m.doc.graph.node(id).expect("no'").type_id())
            .expect("o noise declara hints");
        for h in hints {
            assert_eq!(
                no_cartao.contains(&h.param),
                visivel(h.param),
                "`{}` com range_mode={modo}: o cartao e o painel discordam",
                h.param
            );
        }
    }
}

/// ⭐⭐⭐ **NENHUM PARAM DO PAINEL FICA FORA DO CARTÃO** — o critério de aceitação para o painel
/// lateral SAIR (doc 103; decisão do Enio, 2026-09-05).
///
/// A porta de visibilidade é a mesma nos dois, mas o painel monta as rows por outro caminho
/// (`build_params_snapshot`), e um param pode estar lá **dobrado** dentro de um controlo
/// composto. A régua é por isso de **ALCANCE**, não de contagem: um param do painel está
/// coberto quando tem row no cartão **ou** é
/// - um dos `channels` de um hint [`ParamWidget::Color`] que o cartão mostra (o artista mexe-o
///   pela AMOSTRA — o `motion.tint` dobra `g`/`b`/`a` na cor ancorada em `r`, e o
///   `motion.strobe` dobra o `flash_amount`, que é a alfa daquela cor), **ou**
/// - o `mode_param` de um hint [`ParamWidget::Channels`] que o cartão mostra (o `value.attribute`
///   dobra o `mode` no selector de canal).
///
/// ⚠️ **Medido em 2026-09-05**, quando este gate nasceu: painel **713** params · cartão **700**
/// · **13** de diferença, e as 13 são exactamente estas duas famílias — **zero** buracos reais.
/// ⛔ **A lista de excepções é DERIVADA dos hints**, nunca escrita: um controlo composto novo
/// entra sozinho, e um param que caia fora das duas famílias acende este gate.
///
/// FALSIFICADO por o cartão deixar de receber uma família de params (o gate nomeia o nó e os
/// params, que é o que um agente precisa para saber o que ficou inalcançável).
#[test]
fn no_param_the_panel_offers_falls_off_the_card() {
    let base = MotionState::new();
    let tipos: Vec<String> = base
        .registry
        .manifests()
        .map(|m| m.name.to_string())
        .collect();
    drop(base);
    let mut buracos: Vec<String> = Vec::new();
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        open_every_section(&mut m, id);
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let painel = build_params_snapshot(&m, ph2d_editor_core::ProjectSettings::default());
        let no_painel: Vec<String> = painel
            .as_ref()
            .map(|s| {
                s.rows
                    .iter()
                    .flat_map(|r| {
                        r.params()
                            .iter()
                            .map(|p| (*p).to_string())
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
        let no_cartao: Vec<&'static str> = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map(|v| v.params.iter().map(|c| c.hint.param).collect())
            .unwrap_or_default();
        // As duas famílias de irmãos, DERIVADAS dos hints das rows que o cartão mostra.
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        let mut irmaos: Vec<&'static str> = Vec::new();
        if let Some(hints) = m.registry.param_ui(tid) {
            for h in hints.iter().filter(|h| no_cartao.contains(&h.param)) {
                match h.widget {
                    ph2d_node_registry::ParamWidget::Color { channels } => {
                        irmaos.extend(channels);
                    }
                    ph2d_node_registry::ParamWidget::Channels { mode_param, .. } => {
                        irmaos.push(mode_param);
                    }
                    _ => {}
                }
            }
        }
        for p in &no_painel {
            if !no_cartao.contains(&p.as_str()) && !irmaos.contains(&p.as_str()) {
                buracos.push(format!("{nome}::{p}"));
            }
        }
    }
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    assert!(
        buracos.is_empty(),
        "{} params do painel ficam INALCANCAVEIS no cartao (o painel nao pode sair enquanto \
         isto durar): {buracos:?}",
        buracos.len()
    );
}

/// ⭐⭐ **UMA SECÇÃO DOBRADA ESCONDE AS ROWS E DIZ QUANTAS** — o «painel dentro do nó» do
/// Blender 4.x, e a razão de o cartão do `source.lsystem` caber: ele declara **30** params.
///
/// ⚠️ A dobra por omissão é a que o **registry** declara (`param_groups_folded`), a mesma que o
/// painel usa — o cartão não tem uma segunda tabela de defaults a envelhecer.
///
/// FALSIFICADO por o cartão ignorar a dobra: as 30 rows voltam, e o cartão passa de ~500 para
/// ~730 unidades de altura.
#[test]
fn a_folded_section_hides_its_rows_and_counts_them() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("source.lsystem".to_string());
    let tid = m.doc.graph.node(id).expect("no'").type_id();
    let dobradas = m.registry.param_groups_folded(tid);
    assert!(
        !dobradas.is_empty(),
        "o l-system declara grupos dobrados por omissao — sem isso este gate seria vacuo"
    );
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
    let v = snap.nodes.iter().find(|v| v.id == id.0).expect("cartao");
    let fechadas: Vec<_> = v.sections.iter().filter(|s| !s.open).collect();
    assert!(
        !fechadas.is_empty(),
        "pelo menos uma seccao nasce dobrada, e nasceram {:?}",
        v.sections
    );
    for s in &fechadas {
        assert!(
            s.hidden > 0,
            "a seccao `{}` esconde rows e DIZ quantas (uma dobrada nao pode parecer vazia)",
            s.label
        );
    }
    // E o total bate: o que se vê mais o que está dobrado é tudo o que a visibilidade deixa.
    let escondidas: usize = v.sections.iter().map(|s| s.hidden as usize).sum();
    let visivel = params_visible::shown_params(&m, id);
    let total = m
        .registry
        .param_ui(tid)
        .expect("hints")
        .iter()
        .filter(|h| visivel(h.param))
        .count();
    assert_eq!(
        v.params.len() + escondidas,
        total,
        "nenhuma row se perde entre o visivel e o dobrado"
    );
}

/// ⭐ **ABRIR UMA SECÇÃO TRAZ AS ROWS DE VOLTA** — o estado vive na shell (`card_sections`) e só
/// guarda o que o artista TOCOU: ausente é o que o registry declarou.
/// FALSIFICADO por o toggle não ser lido, ou por guardar o default (que envelhece).
#[test]
fn opening_a_section_brings_its_rows_back() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("source.lsystem".to_string());
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
    let antes = snap
        .nodes
        .iter()
        .find(|v| v.id == id.0)
        .map_or(0, |v| v.params.len());
    open_every_section(&mut m, id);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
    let depois = snap
        .nodes
        .iter()
        .find(|v| v.id == id.0)
        .map_or(0, |v| v.params.len());
    assert!(
        depois > antes,
        "abrir as seccoes traz rows ({antes} -> {depois})"
    );
    let v = snap.nodes.iter().find(|v| v.id == id.0).expect("cartao");
    assert!(
        v.sections.iter().all(|s| s.open && s.hidden == 0),
        "com tudo aberto nenhuma seccao esconde nada"
    );
}
