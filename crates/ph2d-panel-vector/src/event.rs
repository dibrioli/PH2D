//! Vector Style panel event router (thin forwarder).
//!
//! Same forwarder shape as the `panel_seam!`-generated code (Padding /
//! BgRemoval / …), hand-written here so the Width slider can be **registered at
//! a non-neutral initial value** in `populate.rs` (the macro hardcodes `0.5`,
//! which would render the knob at ~10 px instead of the tool's 3 px default).
//!
//! - Width slider drag (or mirror from the chip) → `ToolPanelEvent::SetValue`
//!   carrying the live track `0..1`; the tool projects it to px.
//! - Width chip `ValueChanged` → swallowed (the dispatch mirror already fired
//!   the slider's `ValueChanged`, handled above — avoids a double notify).
//! - Fill "None" `Click` → `ToolPanelEvent::Click` (tool clears the fill).
//! - Close (X) `Click` → `CancelActiveTool` (deactivates the tool, mirror of
//!   the Padding panel's Cancel).
//!
//! The two colour swatches never reach here — their Down opens the shared OKLCH
//! picker via the generic `is_picker_swatch` dispatch (short-circuits in
//! pointer.rs); the shell's `vector_bridge` reads the pick back into the tool.

use crate::ids;
use crate::state::{self, VectorPanelState};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal, seam_reset_button};
use ph2d_editor_core::tool::PanelEvent;

/// Encaminha o track de um slider ao shell como `SetValue`, já **no domínio do documento**.
///
/// Os três sliders que passam por aqui (Bend, Morph `t`, Blend Steps) diferiam só no default e na
/// conversão, e eram três cópias do mesmo corpo — o terceiro (o Bend, bipolar) foi o que estourou o
/// teto de LOC da função e cobrou o fatoramento.
///
/// A conversão mora **nesta fronteira**, de propósito: o shell recebe o número que o documento
/// guarda e nunca precisa saber que existe um track `0..1`. Um shell que convertesse por conta
/// própria seria uma segunda porta para a mesma pergunta — e a que esquecesse o mapa bipolar
/// carimbaria um preset de força zero quando o artista puxou o slider até a esquerda.
/// A linha e o parâmetro que este id endereça, se ele for um slider da pilha de efeitos.
///
/// A varredura é sobre os TETOS (`MAX_FX_ROWS` × `MAX_FX_ROW_PARAMS` = 16 comparações): os ids
/// são hashes de NOME, então não há aritmética que os inverta. É barato, e é o mesmo padrão
/// que os presets do Envelope já usam.
fn fx_param_of(id: ph2d_a11y::NodeId) -> Option<(usize, usize)> {
    (0..ids::MAX_FX_ROWS).find_map(|row| {
        (0..ids::MAX_FX_ROW_PARAMS)
            .find(|&p| id == ids::vector_fx_param_id(row, p))
            .map(|p| (row, p))
    })
}

/// Este id é um botão da pilha (Add / Remove / Up / Down)?
fn is_fx_button(id: ph2d_a11y::NodeId) -> bool {
    (0..ids::MAX_FX_KINDS).any(|k| id == ids::vector_fx_add_id(k))
        || (0..ids::MAX_FX_ROWS).any(|r| {
            id == ids::vector_fx_remove_id(r)
                || id == ids::vector_fx_up_id(r)
                || id == ids::vector_fx_down_id(r)
                || id == ids::vector_fx_hide_id(r)
                // A CAIXINHA de um parâmetro também é um botão. Ela tem id próprio desde
                // 2026-07-18: partilhar o do slider punha dois tipos de widget num id só, e um
                // slider não emite `Click` no Up.
                || (0..ids::MAX_FX_ROW_PARAMS).any(|p| id == ids::vector_fx_toggle_id(r, p))
        })
}

fn forward_track(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
    default: f32,
    to_doc: fn(f64) -> f64,
) -> bool {
    let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(default);
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
            id,
            to_doc(f64::from(track)),
        )));
    true
}

pub(crate) fn apply_event(
    _state: &mut VectorPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        // O Bend é BIPOLAR: o track `0..1` vira `-1..1` aqui, na fronteira.
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_ENVELOPE_BEND => {
            forward_track(host, id, 0.5, |t| t.mul_add(2.0, -1.0))
        }
        // O track de um parâmetro de efeito é NORMALIZADO `0..1`; a faixa real é do EFEITO e
        // viaja no snapshot. Quem reconverte é a shell, que a conhece — aqui o que se garante é
        // que o número que sai é o track, sem fingir ser outra coisa.
        WidgetEvent::ValueChanged(id) if fx_param_of(id).is_some() => {
            forward_track(host, id, 0.0, |t| t)
        }
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_MORPH_T => {
            forward_track(host, id, 0.5, |t| t)
        }
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_BLEND_STEPS => {
            forward_track(host, id, 0.0, |t| t)
        }
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_WIDTH => {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    f64::from(track),
                )));
            true
        }
        // Rotation field (R) — a RELATIVE scrub: the panel owns the per-gesture
        // accumulator, so forward the DELTA since the last report (degrees). The
        // shell rotates the selected path incrementally about its bbox center.
        WidgetEvent::ValueChanged(id) if id == ids::VECTOR_TRANSFORM_R => {
            let cur = host.store().number_value(id).unwrap_or(0.0);
            let delta = cur - crate::state::rot_last();
            crate::state::set_rot_last(cur);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id, delta,
                )));
            true
        }
        // Transform number fields (X/Y/W/H) — standalone NumberInputs (NOT slider-
        // linked): forward the committed VALUE as a document command; the shell
        // drain translates (X/Y) / scales (W/H) the selected path.
        WidgetEvent::ValueChanged(id)
            if id == ids::VECTOR_TRANSFORM_X
                || id == ids::VECTOR_TRANSFORM_Y
                || id == ids::VECTOR_TRANSFORM_W
                || id == ids::VECTOR_TRANSFORM_H =>
        {
            let val = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, val)));
            true
        }
        // **Campo de forma** (genérico): o id carrega o ÍNDICE do parâmetro no catálogo;
        // a forma em foco diz o que ele significa. Encaminha o VALOR (não um track).
        WidgetEvent::ValueChanged(id) if state::shape_field_index(id).is_some() => {
            let val = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, val)));
            true
        }
        // **Campo de ESCOLHA** (o ponto de vista de um sólido): o clique CICLA. O botão tem
        // id próprio, mas o valor mora no slot numérico do mesmo índice — então o `SetValue`
        // sai no id do CAMPO, e a shell segue tendo um caminho só para aplicar parâmetro.
        WidgetEvent::Click(id) if state::shape_choice_index(id).is_some() => {
            cycle_shape_choice(host, id)
        }
        // Variation-axis number field — forward the committed axis value; the shell
        // maps the slot index to the font's axis and re-cooks the glyphs.
        WidgetEvent::ValueChanged(id) if state::text_axis_index(id).is_some() => {
            let val = host.store().number_value(id).unwrap_or(0.0);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, val)));
            true
        }
        // Sliders de TEXTO (track 0..1) + opacidade/dash/gradiente — o mesmo formato do
        // Width. Os parâmetros de FORMA não passam aqui: são caixas numéricas (acima).
        WidgetEvent::ValueChanged(id)
            if id == ids::VECTOR_TEXT_SIZE
                || id == ids::VECTOR_TEXT_WEIGHT
                || id == ids::VECTOR_TEXT_LINE_HEIGHT
                || id == ids::VECTOR_TEXT_TRACKING
                || id == ids::VECTOR_STROKE_OPACITY
                || id == ids::VECTOR_FILL_OPACITY
                || id == ids::VECTOR_DASH
                || id == ids::VECTOR_GAP
                || id == ids::VECTOR_GRAD_ANGLE
                || id == ids::VECTOR_GRAD_INFLUENCE
                || id == ids::VECTOR_GRAD_JITTER =>
        {
            let track = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                    id,
                    f64::from(track),
                )));
            true
        }
        // Chip edits already mirrored to their slider (which fires its own
        // ValueChanged, handled above): swallow to avoid a double notify.
        WidgetEvent::ValueChanged(id)
            if id == ids::VECTOR_WIDTH_NUM
                || id == ids::VECTOR_TEXT_SIZE_NUM
                || id == ids::VECTOR_TEXT_WEIGHT_NUM
                || id == ids::VECTOR_TEXT_LINE_HEIGHT_NUM
                || id == ids::VECTOR_TEXT_TRACKING_NUM
                || id == ids::VECTOR_STROKE_OPACITY_NUM
                || id == ids::VECTOR_FILL_OPACITY_NUM
                || id == ids::VECTOR_DASH_NUM
                || id == ids::VECTOR_GAP_NUM
                || id == ids::VECTOR_GRAD_ANGLE_NUM
                || id == ids::VECTOR_GRAD_INFLUENCE_NUM
                || id == ids::VECTOR_GRAD_JITTER_NUM =>
        {
            true
        }
        // **Opção de CATEGORIA** (uma linha do popover do chip): a família é estado
        // só-do-painel (não vira evento de tool). Fechamos o chip à mão — o light-dismiss
        // genérico não dispara, porque o clique é DENTRO do popover (mesmo caminho do
        // dropdown de fonte).
        WidgetEvent::Click(id) if state::shape_group_index(id).is_some() => {
            seam_reset_button(host, id);
            let index = state::shape_group_index(id);
            if let Some(g) = index.and_then(|i| ph2d_tool_vector::shapes::ALL_GROUPS.get(i)) {
                state::set_current_shape_group(Some(*g));
            }
            if let Some(InteractiveState::Dropdown {
                open,
                selected_index,
                ..
            }) = host.store_mut().get_mut(ids::VECTOR_SHAPE_GROUP_DD)
            {
                *open = false;
                *selected_index = index;
            }
            true
        }
        WidgetEvent::Click(id) if state::shape_index(id).is_some() => {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        // **Opção de PONTA** (uma linha do popover de Start / End) — ver [`pick_marker`].
        WidgetEvent::Click(id) if state::marker_option(id).is_some() => pick_marker(host, id),
        // **A junção do Offset Path** — panel-local, ver [`pick_expand_join`].
        WidgetEvent::Click(id) if expand_join_index(id).is_some() => pick_expand_join(host, id),
        // Draw-mode buttons + Boolean buttons: forward the Click over the generic
        // tool channel. Mode clicks land on `VectorTool::handle_panel_event`
        // (sets the mode); Boolean clicks are picked up by the shell drain, which
        // applies the op to the document (they are not Style edits, so the tool
        // ignores them). Same forwarder shape either way.
        WidgetEvent::Click(id) if forwards_plain_click(id) => {
            seam_reset_button(host, id);
            host.bus_mut()
                .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
            true
        }
        WidgetEvent::Click(id) if id == ids::VECTOR_CLOSE => {
            seam_reset_button(host, id);
            host.bus_mut().push(EditorAction::CancelActiveTool);
            true
        }
        // A family row in the open Font dropdown: close the chip (light-dismiss
        // won't fire — the click is INSIDE the popover) and forward the picked
        // index over `SelectOption`; the shell resolves it to a family + regens.
        WidgetEvent::Click(id) if state::font_option_index(id).is_some() => {
            if let Some(i) = state::font_option_index(id) {
                if let Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) = host.store_mut().get_mut(ids::VECTOR_TEXT_FONT_DD)
                {
                    *open = false;
                    *selected_index = Some(i);
                }
                host.bus_mut()
                    .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
                        ids::VECTOR_TEXT_FONT_DD,
                        i.to_string(),
                    )));
            }
            true
        }
        // As PONTAS (Head Size / Head Round) e o CONECTOR (Route / Jetty / Spread / Corner)
        // tratam os seus ids nos módulos deles — que é onde o snapshot de cada um vive.
        // Delegar AQUI (em vez de mais seis arms) é o que mantém `apply_event` sob o teto de
        // 200 LOC por função.
        other => {
            crate::paint_markers::apply_event(host, other)
                || crate::paint_connector::apply_event(host, other)
        }
    };
    EventOutcome::from_bool(consumed)
}

/// O clique num **campo de ESCOLHA** de forma (o ponto de vista de um sólido isométrico, por
/// exemplo): o botão CICLA pelas opções. Extraído de `apply_event` (teto de 200 LOC por
/// função dos painéis).
///
/// O hit mora num widget (o botão) e o valor mora em outro (o slot numérico do mesmo índice),
/// então o `SetValue` sai no id do **campo** — a shell segue tendo um caminho só para aplicar
/// parâmetro.
///
/// **Sem forma em foco, recusa.** É a MESMA porta do paint (`shape_focus::resolved`): quando
/// o selecionado não é forma viva, a seção nem foi pintada e este clique não existe — recusar
/// aqui é o que impede um botão sobrevivente de um frame antigo de editar a forma errada.
fn cycle_shape_choice(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    seam_reset_button(host, id);
    let i = state::shape_choice_index(id).unwrap_or(0);
    let field = ids::vector_shape_field_id(i);
    let cur = host.store().number_value(field).unwrap_or(0.0);
    let Some(focus) = crate::shape_focus::resolved(&state::current_snapshot()) else {
        return false;
    };
    // O campo deste índice não é uma escolha nesta forma: o clique não existe.
    let Some(next) = ph2d_tool_vector::shapes::next_choice(focus, i, cur) else {
        return false;
    };
    // O painel adianta o valor no store para o botão já pintar o rótulo novo neste frame; a
    // shell aplica e re-cozinha a forma viva.
    host.store_mut().set_number_value(field, next);
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
            field, next,
        )));
    true
}

/// Escolheu uma **ponta de traço** no popover (Start ou End).
///
/// Duas coisas, nesta ordem:
///
/// 1. **Fecha o chip à mão.** O light-dismiss genérico não dispara — o clique é DENTRO do
///    popover (a mesma armadilha do dropdown de categoria e do de fonte). Sem isto o
///    popover ficaria pendurado sobre o painel depois da escolha.
/// 2. **Emite o DISCRIMINANTE da ponta no id do CHIP** (`SetValue`), não um `Click` no id
///    da opção: a tool tem UM braço por seletor, e uma ponta nova (`ALL_MARKERS` cresce)
///    não pode exigir um braço novo lá. É o mesmo desenho do campo de escolha das formas
///    — o hit mora num widget, o valor mora em outro.
///
/// `false` = o índice não é uma ponta que exista (não pode acontecer pelo guard do
/// chamador, mas o caminho recusa em vez de adivinhar).
fn pick_marker(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    seam_reset_button(host, id);
    let Some((slot, index)) = state::marker_option(id) else {
        return false;
    };
    let Some(marker) = ph2d_vec_scene::ALL_MARKERS.get(index).copied() else {
        return false; // slot de id vazio (o espaço é fixo, o catálogo é menor)
    };
    let dd = crate::paint_markers::marker_dd_id(slot);
    if let Some(InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(dd)
    {
        *open = false;
        *selected_index = Some(index);
    }
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
            dd,
            f64::from(marker.as_u8()),
        )));
    true
}

/// Grava a junção escolhida para o **Offset Path** e ENGOLE o clique.
///
/// Panel-local de propósito: escolher a quina não edita o documento (só o clique em "Offset
/// Path" edita), então ela para aqui em vez de virar um `ToolPanelEvent` que a shell teria
/// de guardar num 2º lugar — e dois lugares guardando a mesma escolha divergem.
fn pick_expand_join(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    seam_reset_button(host, id);
    if let Some(j) = expand_join_index(id) {
        state::set_expand_join(j);
    }
    true
}

/// O índice da junção do Offset Path que este id escolhe (`0` Miter · `1` Round · `2`
/// Bevel), ou `None` se o id não é um dos três chips. Porta única: o `apply_event` a
/// consulta para saber se ATENDE o clique, e [`pick_expand_join`] para saber o QUE gravar —
/// duas perguntas, uma tabela.
fn expand_join_index(id: ph2d_a11y::NodeId) -> Option<u8> {
    match id {
        _ if id == ids::VECTOR_EXPAND_JOIN_MITER => Some(0),
        _ if id == ids::VECTOR_EXPAND_JOIN_ROUND => Some(1),
        _ if id == ids::VECTOR_EXPAND_JOIN_BEVEL => Some(2),
        _ => None,
    }
}

/// Ids dos botões cujo `Click` o painel apenas ENCAMINHA (`PanelEvent::Click`) para o
/// shell/tool aplicar (modos, Cap/Join, Vertex, Boolean, Arrange, Fill kind, Align/
/// Distribute, alinhamento de texto…). Extraído de `apply_event` para caber no teto de
/// 200 LOC por função dos painéis — a lista só cresce.
fn forwards_plain_click(id: ph2d_a11y::NodeId) -> bool {
    id == ids::VECTOR_MODE_SELECT
        || id == ids::VECTOR_MODE_NODE
        || id == ids::VECTOR_MODE_PEN
        // O 5º pill: re-arma o gesto de forma. Ausente daqui, o botão pintaria e estaria
        // MORTO — exatamente o bug que o smoke do Line/Arc pegou (Enio 2026-07-09).
        || id == ids::VECTOR_MODE_SHAPE
        || id == ids::VECTOR_MODE_TEXT
        // O 6º pill (Connect). Fora daqui, o botão pinta e está MORTO — é o gate do
        // seam (`clicking_connect_pill_reaches_the_tool`).
        || id == ids::VECTOR_MODE_CONNECT
        // O 7º pill (Build / Shape Builder).
        || id == ids::VECTOR_MODE_BUILD
        // O 8º pill (Pick Shapes / Blend). Fora daqui, o pill pinta e está MORTO.
        || id == ids::VECTOR_MODE_PICKBLEND
        // O 9º e 10º pills (Fillet / Chamfer). Fora daqui, pintam e estão MORTOS.
        || id == ids::VECTOR_MODE_FILLET
        || id == ids::VECTOR_MODE_CHAMFER
        || id == ids::VECTOR_TEXT_FONT_PREV
        || id == ids::VECTOR_TEXT_FONT_NEXT
        || id == ids::VECTOR_TEXT_FONT_IMPORT
        || id == ids::VECTOR_TEXT_ALIGN_LEFT
        || id == ids::VECTOR_TEXT_ALIGN_CENTER
        || id == ids::VECTOR_TEXT_ALIGN_RIGHT
        || id == ids::VECTOR_CAP_BUTT
        || id == ids::VECTOR_CAP_ROUND
        || id == ids::VECTOR_CAP_SQUARE
        || id == ids::VECTOR_JOIN_MITER
        || id == ids::VECTOR_JOIN_ROUND
        || id == ids::VECTOR_JOIN_BEVEL
        || id == ids::VECTOR_VERT_CORNER
        || id == ids::VECTOR_VERT_SMOOTH
        || id == ids::VECTOR_VERT_SYMMETRIC
        || id == ids::VECTOR_VERT_DELETE
        || id == ids::VECTOR_BLEND_RUN
        || id == ids::VECTOR_BLEND_RESET_SPINE
        || id == ids::VECTOR_BLEND_EXPAND
        || id == ids::VECTOR_BLEND_RELEASE
        || id == ids::VECTOR_MORPH_RUN
        || is_fx_button(id)
        // O botão de SEÇÃO "Apply" (assa a pilha de efeitos). Fora daqui pintaria e estaria
        // MORTO — a shell classifica-o via `fx_bridge_dispatch::classify_click`.
        || id == ids::VECTOR_FX_APPLY
        || id == ids::VECTOR_ENVELOPE_RUN
        || id == ids::VECTOR_ENVELOPE_EXPAND
        || id == ids::VECTOR_ENVELOPE_RELEASE
        || id == ids::VECTOR_ENVELOPE_PERSPECTIVE
        || id == ids::VECTOR_ENVELOPE_MESH
        || id == ids::VECTOR_ENVELOPE_PINS
        || id == ids::VECTOR_ENVELOPE_CLEAR_PINS
        || (0..ids::MAX_ENVELOPE_PRESETS).any(|i| id == ids::vector_envelope_preset_id(i))

        || id == ids::VECTOR_BOOL_UNION
        || id == ids::VECTOR_BOOL_SUBTRACT
        || id == ids::VECTOR_BOOL_INTERSECT
        || id == ids::VECTOR_BOOL_EXCLUDE
        || id == ids::VECTOR_COMPOUND_MAKE
        || id == ids::VECTOR_COMPOUND_RELEASE
        // Expand: os dois COMANDOS (a junção não vem aqui — é panel-local).
        || id == ids::VECTOR_EXPAND_OFFSET_PATH
        || id == ids::VECTOR_EXPAND_OUTLINE_STROKE
        || id == ids::VECTOR_FILL_RULE_NONZERO
        || id == ids::VECTOR_FILL_RULE_EVENODD
        || id == ids::VECTOR_SNAP_OFF
        || id == ids::VECTOR_SNAP_ON
        || id == ids::VECTOR_ARRANGE_DUPLICATE
        || id == ids::VECTOR_ARRANGE_TO_BACK
        || id == ids::VECTOR_ARRANGE_BACKWARD
        || id == ids::VECTOR_ARRANGE_FORWARD
        || id == ids::VECTOR_ARRANGE_TO_FRONT
        || id == ids::VECTOR_ARRANGE_FLIP_H
        || id == ids::VECTOR_ARRANGE_FLIP_V
        || id == ids::VECTOR_ARRANGE_ROTATE_CW
        || id == ids::VECTOR_ARRANGE_ROTATE_CCW
        || id == ids::VECTOR_PATH_SMOOTH
        || id == ids::VECTOR_PATH_SHARPEN
        || id == ids::VECTOR_PATH_SIMPLIFY
        || id == ids::VECTOR_PATH_SUBDIVIDE
        || id == ids::VECTOR_PATH_CLOSE
        || id == ids::VECTOR_FILL_KIND_SOLID
        || id == ids::VECTOR_FILL_KIND_LINEAR
        || id == ids::VECTOR_FILL_KIND_RADIAL
        || id == ids::VECTOR_FILL_KIND_MULTI
        || id == ids::VECTOR_GRAD_ADD_POINT
        || id == ids::VECTOR_GRAD_REMOVE_POINT
        || id == ids::VECTOR_GRAD_ADD_STOP
        || id == ids::VECTOR_GRAD_REMOVE_STOP
        || id == ids::VECTOR_ALIGN_LEFT
        || id == ids::VECTOR_ALIGN_HCENTER
        || id == ids::VECTOR_ALIGN_RIGHT
        || id == ids::VECTOR_ALIGN_TOP
        || id == ids::VECTOR_ALIGN_VCENTER
        || id == ids::VECTOR_ALIGN_BOTTOM
        || id == ids::VECTOR_DISTRIBUTE_H
        || id == ids::VECTOR_DISTRIBUTE_V
        || id == ids::VECTOR_PIVOT_EDIT
        || id == ids::VECTOR_CONVERT_TO_CURVES
        // **Both Ends** (a dupla via). Um `Click` puro — e não um `SetValue` com um booleano
        // — porque o estado é DERIVADO das duas pontas: quem o resolve é a tool, que as
        // possui. Fora daqui o botão pintaria e estaria MORTO.
        || id == ids::VECTOR_MARKER_BOTH
}
