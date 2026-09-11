//! ⭐⭐⭐ **A ESCOLHA DE COR FEITA NUM EDITOR ABERTO SOBRE O CARTÃO** — que parada de que
//! gradiente, ou que cor de que paleta, o selector aberto está a editar.
//!
//! ⚠️ **Irmão de [`super::color`] por RESPONSABILIDADE:** lá vive *o que uma escolha ESCREVE*
//! (a fronteira sRGB↔linear, as três leis de escrita); aqui *de QUEM ela é* quando quem a
//! abriu foi um cartão. A pergunta é diferente porque a resposta é: no painel há **um** nó
//! seleccionado e o id não precisa de o dizer; no grafo há vinte cartões e o nó **sai do id**.
//!
//! ⛔⛔ **É a metade que o censo do `panel_exit_probe` NÃO vê.** Ele mede o que um clique numa
//! row FAZ (`click_does`), e por isso conta um gradiente como alcançável no dia em que a janela
//! abre — mesmo que a cor escolhida lá dentro não chegue a lado nenhum. *Um controlo que abre e
//! não guarda é a espécie de morto que uma sonda de alcance lê como vivo.*

use crate::motion::motion_state::MotionState;
use ph2d_panel_motion_graph::card_editor_swatch_id;

/// **A que amostra o selector aberto pertence.** `param` é a chave de TEXTO (a serialização), e
/// `index` a parada / a cor dentro dela.
pub(super) struct CardPick {
    pub node: ph2d_nodegraph::graph::NodeId,
    pub param: &'static str,
    pub index: usize,
    /// Um gradiente escreve uma RAMPA e uma paleta escreve uma LISTA — as duas serializações
    /// são diferentes, e a escrita tem de saber qual.
    pub gradient: bool,
}

/// ⭐⭐ **Varre os cartões à procura do dono do selector aberto.**
///
/// ⚠️ **A varredura só corre com um selector ABERTO**, que é um gesto do artista e não um quadro
/// qualquer — a mesma cerca que o `picker_target_of` já declara para as amostras de cor simples.
///
/// ⚠️ **A contagem sai do VALOR VIVO** (as paradas que a string tem agora), porque é ela que o
/// editor pintou: uma contagem fixa deixaria a última amostra de uma rampa longa sem dono, e a
/// escolha do artista evaporava.
pub(super) fn card_editor_pick(
    motion: &MotionState,
    store: &ph2d_editor::interaction::WidgetStore,
) -> Option<CardPick> {
    use ph2d_node_registry::ParamWidget;
    let alvo = store.picker_target()?;
    for inst in motion.doc.graph.nodes() {
        let Some(hints) = motion.registry.param_ui(inst.type_id()) else {
            continue;
        };
        for h in hints {
            let gradient = match h.widget {
                ParamWidget::Gradient => true,
                ParamWidget::Palette => false,
                _ => continue,
            };
            let texto = motion
                .doc
                .graph
                .node_text_param_overrides(inst.id)
                .and_then(|m| m.get(h.param))
                .map(String::as_str)
                .unwrap_or_default();
            let n = if gradient {
                super::current_gradient_len(texto)
            } else {
                super::current_palette_len(texto)
            };
            if let Some(index) =
                (0..n).find(|&i| card_editor_swatch_id(inst.id.0, h.param, i) == alvo)
            {
                return Some(CardPick {
                    node: inst.id,
                    param: h.param,
                    index,
                    gradient,
                });
            }
        }
    }
    None
}
