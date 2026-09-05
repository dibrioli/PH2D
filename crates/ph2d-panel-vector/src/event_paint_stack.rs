//! ⭐⭐⭐ **AS ROTAS DA PILHA DE APARÊNCIA** (estudo 42 item 4) — as três perguntas que o
//! `apply_event` faz sobre um id desta família, num sítio só.
//!
//! # ⛔ Por que uma PORTA, e não três linhas em três listas
//!
//! Esta família shipou em 2026-09-05 com o modelo, o transform, o renderer, o exportador e o
//! painel gateados, e **nenhuma** das três rotas ligada: os cinco controlos de cada linha caíam no
//! catch-all do `apply_event`, a largura não estava entre os campos da shell, e a opacidade não
//! estava entre os sliders encaminhados. O report do Enio foi *"o olho não funciona; clicar no
//! nome não exibe a largura, a opacidade e a mistura; demais botões não funcionam"* — **três**
//! defeitos que se leem como um.
//!
//! Espalhá-las por três allowlists escritas à mão (que é onde as famílias vizinhas moram) é
//! exactamente como isto aconteceu: cada lista é longa, cada uma é editada por outra wave, e
//! nenhuma delas responde *"a pilha está ligada?"*. Aqui as três estão à vista uma da outra, e o
//! `seam_paint_stack.rs` mede-as pelo mesmo gesto que o artista faz.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::PanelEvent;

use crate::ids;

/// **Este id é um dos controlos que a pilha oferece ao CLIQUE?**
///
/// ⚠️ **Varre o espaço FIXO de ids** ([`ph2d_vec_scene::MAX_PAINT_LAYERS`]), como o `populate` que
/// os regista e o `stack_verb_for_id` da shell que os resolve: uma camada criada neste frame tem
/// de nascer clicável **no mesmo frame**, e uma lista do tamanho da pilha de hoje deixaria a
/// camada nova pintada e muda.
///
/// ⛔ **A SWATCH não está aqui, e a ausência é a decisão.** Ela é uma *picker swatch*
/// (`register_picker_swatch`), e o `pointer_down` curto-circuita o Down dessas: elas **nunca**
/// emitem `Click`. Pô-la nesta lista seria uma linha que não se pode exercitar — quem lê a escolha
/// é a shell, pelo `layer_of_picker_target`.
pub(super) fn is_button(id: ph2d_a11y::NodeId) -> bool {
    id == ids::VECTOR_PAINT_ADD_FILL
        || id == ids::VECTOR_PAINT_ADD_STROKE
        || (0..ph2d_vec_scene::MAX_PAINT_LAYERS).any(|i| {
            id == ids::vector_paint_eye_id(i)
                || id == ids::vector_paint_row_id(i)
                || id == ids::vector_paint_up_id(i)
                || id == ids::vector_paint_down_id(i)
                || id == ids::vector_paint_del_id(i)
        })
}

/// **As três propriedades da camada ABERTA**, quando alguma delas muda de valor.
///
/// `None` = o id não é desta família; o `match` de cima decide.
///
/// ⚠️ **A largura viaja como VALOR e a opacidade como TRACK**, e a diferença não é estilo: a
/// largura é um `NumberInput` cujo número já está na unidade do documento, e a opacidade é um
/// slider cujo trilho é `0..1`. A shell trata as duas em braços separados pela mesma razão —
/// trocá-las escreveria uma largura de `0,25` onde o artista pediu 25 %.
///
/// ⚠️ **O chip da opacidade é ENGOLIDO**: o dispatch já espelha o valor dele no slider, que dispara
/// o próprio `ValueChanged` (tratado acima). Sem esta linha o mesmo gesto chegaria **duas** vezes
/// ao documento — e é a lei que os catorze chips irmãos desta janela já pagam.
pub(super) fn value_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> Option<bool> {
    let WidgetEvent::ValueChanged(id) = ev else {
        return None;
    };
    if id == ids::VECTOR_PAINT_WIDTH {
        return Some(super::forward_number(host, id));
    }
    if id == ids::VECTOR_PAINT_OPACITY {
        let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(1.0);
        host.bus_mut()
            .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                id,
                f64::from(track),
            )));
        return Some(true);
    }
    if id == ids::VECTOR_PAINT_OPACITY_NUM {
        return Some(true);
    }
    None
}
