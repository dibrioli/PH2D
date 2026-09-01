//! ⭐⭐ **Escolher a VERSÃO de uma instância** — o chip do cartão de propriedades.
//!
//! # ⛔⛔⛔ O que este ficheiro já teve, e por que saiu (Enio, 2026-09-01)
//!
//! Ele chegou a ter o campo que renomeia o valor de uma propriedade e o formulário de *Salvar
//! Variação…*. As duas encarnações do mecanismo de propriedades foram recusadas pelo dono — a das
//! chaves no nome em 31/08, a do dado + botão em 01/09 (*«não ficou bom e não funcionou»*) — e o
//! mecanismo está **adiado para o fim do plano**.
//!
//! ⚠️ **O código saiu inteiro; ele não ficou comentado nem atrás de bandeira.** *Meio-feito é pior
//! que não começar*, e uma feature adiada que fica no fonte é a que volta sozinha.
//!
//! ⇒ o que resta é o gesto que já existia e ninguém recusou: **carregar num chip troca a versão**.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

/// `true` quando o evento era desta família.
pub(crate) fn apply_value_event(
    state: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let _ = state;
    match ev {
        WidgetEvent::Click(id) => chip_click(host, id),
        _ => false,
    }
}

/// ⭐⭐ **O chip: trocar de versão.**
///
/// ⚠️ **O painel manda o `StableId` do mestre, e não o índice do chip.** O índice é uma posição
/// numa lista que o construtor refaz por quadro, e uma reordenação entre o pintar e o clicar
/// escolheria a versão errada sem erro nenhum. *A identidade viaja; a posição fica no painel.*
fn chip_click(host: &mut dyn PanelHostInternal, id: ids::NodeId) -> bool {
    // ⚠️ **A leitura inversa vem da PORTA** (`ids::instance_axis_option`), e ela vem ANTES do
    // estado: este braço corre em TODO clique do Inspector, e o `current_inspector_properties()`
    // **clona** o cartão inteiro. Perguntar primeiro o que custa 3 ns é a ordem certa das duas.
    let Some((a, v)) = ids::instance_axis_option(id) else {
        return false;
    };
    let Some(info) = crate::state::current_inspector_properties() else {
        return false;
    };
    let Some(choice) = info.rows.get(a).and_then(|ax| ax.options.get(v)) else {
        return false;
    };
    // ⛔ Carregar na versão vigente é um no-op — e a ausência de resposta é a resposta certa: o
    // artista carregou no botão que diz onde ele já está.
    if choice.current || choice.master == 0 || info.root_bits == 0 {
        return true;
    }
    host.bus_mut().push(EditorAction::InspectorSwapVariant {
        root_bits: info.root_bits,
        master: choice.master,
    });
    true
}
