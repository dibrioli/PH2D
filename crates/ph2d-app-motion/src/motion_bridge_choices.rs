//! ⭐⭐⭐ **AS OPÇÕES DE UM SELECTOR DO CARTÃO** — quais são, o que uma SETA faz e o que a LINHA
//! escolhida escreve (report do Enio, 2026-09-07, com a foto do selector do Blender: *«para esse
//! tipo de campo deveríamos ter duas setas laterais e se clicar no centro (nome) abre-se um
//! dropdown»*).
//!
//! ⚠️ **Irmão de [`super::card`] por RESPONSABILIDADE** (HR-18): aquele responde *«que params o
//! cartão carrega»*, este *«que OPÇÕES um selector oferece»*. As três metades vivem juntas de
//! propósito — a lista publicada, o passo da seta e a escrita da linha escolhida têm de falar da
//! **mesma** lista, na **mesma** ordem, ou o artista vê `Speed` na linha 2 e escolhe `Size`.
//!
//! ⛔ **Um ENUM não passa por aqui para ESCREVER, e passa para ser MOSTRADO.** O valor dele *é* o
//! índice da opção e os rótulos vivem no `ParamUiHint` que o cartão já carrega, então o painel
//! escreve-o sozinho — a shell não teria nada a resolver. Mas a LISTA publica-se à mesma, porque
//! quem a lê (o popup) não tem por que saber de que espécie ela veio.

use super::*;
use ph2d_node_registry::ParamWidget as W;
use ph2d_panel_motion_graph::CardChoices;

/// ⭐⭐ **AS OPÇÕES DE CADA SELECTOR VISÍVEL**, publicadas ao lado do snapshot (shell → painel).
///
/// ⚠️ **Zero alocação no caso comum**: um enum viaja como `&'static` (são **138** das 143 rows de
/// selector do catálogo) e só as listas VIVAS — o que o artista desenhou, o que a corrente de cima
/// cozinhou — pagam `String`s, que são meia dúzia. É a mesma medição que mantém o `CardParam` sem
/// texto (doc 103 §7).
pub fn publish_card_choices(
    motion: &MotionState,
    snap: &ph2d_panel_motion_graph::GraphViewSnapshot,
) {
    let mut fora: Vec<(u32, &'static str, CardChoices, u16)> = Vec::new();
    for node in &snap.nodes {
        let nid = ph2d_nodegraph::graph::NodeId(node.id);
        let Some(type_id) = motion.doc.graph.node(nid).map(|i| i.type_id()) else {
            continue;
        };
        for h in motion.registry.param_ui(type_id).unwrap_or(&[]) {
            match h.widget {
                W::Enum { labels } if !labels.is_empty() => {
                    // ⚠️ O valor CRU, não a face: um enum não tem unidade, e o índice é o valor.
                    let i = param_value(motion, nid, h.param).round().max(0.0);
                    fora.push((
                        node.id,
                        h.param,
                        CardChoices::Static(labels),
                        u16::try_from(i as usize).unwrap_or(u16::MAX),
                    ));
                }
                W::Source => {
                    let atual = text_of(motion, nid, h.param);
                    let opcoes = source_options_live(motion);
                    let i = opcoes
                        .iter()
                        .position(|o| *o == atual)
                        .unwrap_or(opcoes.len());
                    fora.push((
                        node.id,
                        h.param,
                        CardChoices::Live(opcoes),
                        u16::try_from(i).unwrap_or(u16::MAX),
                    ));
                }
                W::Channels {
                    mode_param,
                    channels,
                } => {
                    let (rotulos, i) = channel_labels(motion, nid, h.param, mode_param, channels);
                    fora.push((node.id, h.param, CardChoices::Live(rotulos), i));
                }
                _ => {}
            }
        }
    }
    ph2d_panel_motion_graph::set_card_choices(fora);
}

/// O override de texto de `(nó, param)`, ou vazio — a mesma leitura que o construtor de rows faz.
fn text_of(motion: &MotionState, nid: ph2d_nodegraph::graph::NodeId, param: &str) -> String {
    motion
        .doc
        .graph
        .node_text_param_overrides(nid)
        .and_then(|m| m.get(param))
        .cloned()
        .unwrap_or_default()
}

/// ⭐⭐ **UMA SETA** — a opção anterior (`delta = -1`) ou a seguinte (`+1`), escrita pelas mesmas
/// portas que a chip do painel usa.
///
/// ⚠️ **Um canal escreve DUAS coisas** (a coluna e o `mode`) e uma fonte escreve uma; a diferença
/// é do substrato, e é por isso que a resolução vive na shell e não no cartão.
pub fn step_choice(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &'static str,
    delta: i8,
) {
    let Some(widget) = widget_of(motion, nid, param) else {
        return;
    };
    match widget {
        W::Source => {
            let atual = text_of(motion, nid, param);
            let opcoes = source_options_live(motion);
            if let Some(proxima) = next_source_for(&opcoes, &atual, delta) {
                write_source(nid, param, proxima);
            }
        }
        W::Channels {
            mode_param,
            channels,
        } => {
            if let Some(escolha) = next_channel_for(motion, nid, param, mode_param, channels, delta)
            {
                write_channel(nid, param, mode_param, escolha);
            }
        }
        _ => {}
    }
}

/// ⭐⭐ **UMA LINHA DA LISTA** — o índice é contra a lista que [`publish_card_choices`] publicou e
/// o popup desenhou, resolvido aqui pela MESMA porta que a produziu.
pub fn pick_choice(
    motion: &MotionState,
    nid: ph2d_nodegraph::graph::NodeId,
    param: &'static str,
    index: u16,
) {
    let Some(widget) = widget_of(motion, nid, param) else {
        return;
    };
    let i = index as usize;
    match widget {
        W::Source => {
            if let Some(nome) = source_options_live(motion).get(i).cloned() {
                write_source(nid, param, nome);
            }
        }
        W::Channels {
            mode_param,
            channels,
        } => {
            if let Some(escolha) = channel_choice_at(motion, nid, param, mode_param, channels, i) {
                write_channel(nid, param, mode_param, escolha);
            }
        }
        _ => {}
    }
}

/// O widget declarado para `(nó, param)` — a espécie decide o que uma escolha escreve.
fn widget_of(motion: &MotionState, nid: ph2d_nodegraph::graph::NodeId, param: &str) -> Option<W> {
    let type_id = motion.doc.graph.node(nid).map(|i| i.type_id())?;
    motion
        .registry
        .param_ui(type_id)?
        .iter()
        .find(|h| h.param == param)
        .map(|h| h.widget)
}

fn write_source(nid: ph2d_nodegraph::graph::NodeId, param: &'static str, value: String) {
    ph2d_panel_motion_params::push_param_intent(
        ph2d_panel_motion_params::MotionParamIntent::SetTextParam {
            node: nid.0,
            param,
            value,
        },
    );
}

/// ⚠️ **As duas escritas, sempre juntas**: escrever só a coluna deixaria o nó a ler o sítio certo
/// no modo errado, que é ler **zeros em silêncio** — o defeito que a própria lista de canais do
/// `value.attribute` existe para não ter.
fn write_channel(
    nid: ph2d_nodegraph::graph::NodeId,
    param: &'static str,
    mode_param: &'static str,
    (coluna, modo): (String, i32),
) {
    write_source(nid, param, coluna);
    ph2d_panel_motion_params::push_param_intent(
        ph2d_panel_motion_params::MotionParamIntent::SetParam {
            node: nid.0,
            param: mode_param,
            value: f64::from(modo),
        },
    );
}

/// **A LISTA VIVA de nomes publicados e a lei de andar por ela** — as duas metades da mesma
/// pergunta, expostas ao irmão `intents` porque o **cartão** as pede (`GraphIntent::CycleSource`)
/// e a row do painel as usa. ⚠️ Uma cópia de qualquer das duas do lado do cartão faria o mesmo
/// clique escolher coisas diferentes conforme a superfície.
pub fn source_options_live(motion: &MotionState) -> Vec<String> {
    super::params_stream::source_options(motion)
}

/// A **fonte seguinte** na lista viva — ver [`source_options_live`].
pub fn next_source_for(opcoes: &[String], atual: &str, delta: i8) -> Option<String> {
    super::params_stream::next_source(opcoes, atual, delta)
}

/// ⭐⭐ **O CANAL VIZINHO de um selector de canais** — o irmão de [`next_source_for`], pedido
/// pelo cartão (`GraphIntent::StepChoice`) e resolvido aqui porque a lista é **viva**: ela é os
/// canais curados do nó MAIS as colunas que a corrente de cima cozinhou neste quadro.
///
/// Devolve `(coluna, mode)` — as **duas** escritas que um canal faz, num par, porque escrever
/// só uma deixaria o nó a ler a coluna certa no modo errado (que é ler zeros em silêncio).
pub fn next_channel_for(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    text_param: &str,
    mode_param: &str,
    channels: &'static [ph2d_node_registry::ReadChannel],
    delta: i8,
) -> Option<(String, i32)> {
    let (lista, atual) =
        super::params_stream::channel_walk(motion, node, text_param, mode_param, channels);
    super::params_stream::next_channel(&lista, &atual, delta)
}

/// ⭐ **A opção `index` da lista de um canal** — o que a LINHA escolhida do dropdown significa.
/// A mesma `channel_walk` por onde as setas andam, para o índice que o menu desenhou e o índice
/// que a shell resolve serem o mesmo.
pub fn channel_choice_at(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    text_param: &str,
    mode_param: &str,
    channels: &'static [ph2d_node_registry::ReadChannel],
    index: usize,
) -> Option<(String, i32)> {
    let (lista, _) =
        super::params_stream::channel_walk(motion, node, text_param, mode_param, channels);
    lista.get(index).cloned()
}

/// ⭐ **Os RÓTULOS da lista de um canal**, na ordem por onde as setas andam — o que o dropdown
/// mostra. Um canal curado lê-se pelo nome que o painel dá (`Speed`); uma coluna vinda da
/// corrente de cima **é** o próprio nome.
pub fn channel_labels(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    text_param: &str,
    mode_param: &str,
    channels: &'static [ph2d_node_registry::ReadChannel],
) -> (Vec<String>, u16) {
    let (lista, atual) =
        super::params_stream::channel_walk(motion, node, text_param, mode_param, channels);
    let rotulos = lista
        .iter()
        .map(|(col, modo)| {
            channels
                .iter()
                .find(|c| c.column == *col && c.mode == *modo)
                .map_or_else(|| col.clone(), |c| ph2d_i18n::tr(c.label).to_string())
        })
        .collect();
    // ⚠️ Fora da lista ⇒ um índice ALÉM do fim, que é como se diz *«nenhuma»* sem inventar uma.
    let atual_i = lista
        .iter()
        .position(|o| *o == atual)
        .unwrap_or(lista.len());
    (rotulos, u16::try_from(atual_i).unwrap_or(u16::MAX))
}

/// Os RÓTULOS e a marca, para o gate que ata o que a lista MOSTRA ao que a escolha escreve.
#[cfg(test)]
pub fn channel_labels_for_tests(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    text_param: &str,
    mode_param: &str,
    channels: &'static [ph2d_node_registry::ReadChannel],
) -> (Vec<String>, u16) {
    channel_labels(motion, node, text_param, mode_param, channels)
}

/// A LISTA por onde o clique anda, para o gate que a ata à ordem que o painel PINTA.
#[cfg(test)]
pub fn channel_walk_for_tests(
    motion: &MotionState,
    node: ph2d_nodegraph::graph::NodeId,
    text_param: &str,
    mode_param: &str,
    channels: &'static [ph2d_node_registry::ReadChannel],
) -> (Vec<(String, i32)>, (String, i32)) {
    super::params_stream::channel_walk(motion, node, text_param, mode_param, channels)
}
