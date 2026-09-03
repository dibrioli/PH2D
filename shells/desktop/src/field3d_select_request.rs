//! ⭐⭐⭐ **O QUE O SHELL TEM DE FAZER À SELEÇÃO** — o vocabulário do pedido, e a porta única que o
//! aplica.
//!
//! # Por que um arquivo irmão
//!
//! O [`super`] passou dos `600` do teto de LOC do shell (HR-18) quando a W112 acrescentou o
//! `RemoveMany`. ⛔ *Split, nunca allowlist* — e o corte é por assunto: aqui está *«o que se faz à
//! seleção»*, e no pai fica *«o que corre por quadro»*.

/// ⭐⭐⭐ **A LEI DA SELEÇÃO, NUMA PORTA SÓ** — o app e os gates chamam **esta**.
///
/// ⚠️ **Ela nasceu de duas mutações que sobreviveram** (W58d): o gate lia o [`SelectRequest`] e
/// aplicava-o com uma **cópia** da lei escrita dentro do teste, então trocar `add_to_selection` por
/// `toggle_in_selection` no `render_loop` — que é **exactamente** o defeito reportado — ficava
/// verde. *É a terceira vez nesta linha que a metade que falta é a de quem executa: duas cópias de
/// uma lei é uma lei que gate nenhum defende.*
pub(crate) fn apply(gizmo: &mut ph2d_editor::screens::hero::GizmoStateGroup, req: SelectRequest) {
    match req {
        SelectRequest::Entity(bits) => gizmo.replace_selection(Some(bits)),
        SelectRequest::Clear => gizmo.clear_all_selection(),
        SelectRequest::Toggle(bits) => gizmo.toggle_in_selection(bits),
        // ⭐ **ACRESCENTA**, nunca alterna — ver [`SelectRequest::AddMany`].
        SelectRequest::AddMany(all) => {
            for bits in all {
                gizmo.add_to_selection(bits);
            }
        }
        // ⭐⭐ **TIRA**, e pela mesma razão por que o irmão não alterna (W112).
        SelectRequest::RemoveMany(all) => {
            for bits in all {
                gizmo.remove_from_selection(bits);
            }
        }
    }
}

/// Corre uma vez por quadro, antes do traçado. No-op silencioso quando o módulo não está armado.
/// **O que o shell tem de fazer à seleção** depois de a ponte correr.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SelectRequest {
    Entity(u64),
    /// O clique caiu no fundo. ⚠️ Limpar é a resposta certa e é o que todo modelador faz — a
    /// alternativa (manter a seleção) deixaria o gizmo aceso em cima de nada.
    Clear,
    /// ⭐⭐ **Alternar um objeto na seleção** (W58) — o clique com `Shift`/`Ctrl`, o mesmo verbo que
    /// o canvas 2D já usa.
    Toggle(u64),
    /// ⭐⭐ **O que o LAÇO apanhou** (W58) — e ele **ACRESCENTA**, nunca alterna.
    ///
    /// ⛔ **A W58 fê-lo alternar, e estava errado** (Enio, 2026-08-24: *"se uma peça estiver
    /// selecionada e outra não, o retângulo não seleciona todas, mas inverte a seleção"*).
    ///
    /// ⭐ **A assimetria com o clique é a lei, não uma inconsistência:** um **clique** tem um alvo
    /// **único e visível**, então alternar é preciso e reversível — o artista vê exactamente o que
    /// vai mudar. Um **rectângulo** cobre vários, e alternar mistura estados que ele **não vê**: o
    /// resultado passa a depender de qual estava selecionado por baixo, e o mesmo gesto sobre a
    /// mesma tela dá resultados diferentes. *Um gesto cujo resultado depende de estado invisível
    /// não é usável.* É também o que todo editor faz — o laço com modificador **soma**.
    ///
    /// ⚠️ Vazio = o laço não apanhou nada, e aí não se mexe na seleção (o artista falhou a mira;
    /// limpar seria castigá-lo).
    AddMany(Vec<u64>),
    /// ⭐⭐⭐ **O que o laço apanhou, TIRADO da seleção** (W112) — a metade que faltava desde a W58.
    ///
    /// ⚠️ **Ela não reabre a recusa do [`Self::AddMany`], e a diferença é o que a torna usável:**
    /// alternar mistura estados que o artista não vê, mas *tirar* é **determinístico** — tudo o que
    /// o rectângulo cobre sai, seja qual for o estado por baixo. É por isso que todo editor sério
    /// tem **duas** operações de marquise e não uma que inverte.
    ///
    /// ⚠️ Qual das duas é o gesto **não** vem de uma tecla: as quatro saídas de modificador estão
    /// medidas e fechadas (doc 06 §79.3), e o que escolhe é um chip do painel — ver
    /// `ph2d_panel_model3d::ModelSnapshot::selects`.
    RemoveMany(Vec<u64>),
}
