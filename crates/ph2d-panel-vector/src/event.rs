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
pub(super) fn is_fx_button(id: ph2d_a11y::NodeId) -> bool {
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

/// Os sliders cujo valor viaja como **track** (ou como uma conversão de uma linha dele).
///
/// Extraídos do `apply_event` pelo teto de 200 LOC por função. O critério não é "os primeiros
/// braços": é *"o valor que sai daqui é o track, ou uma função pura dele"* — todo o resto lê
/// estado do painel (o acumulador da rotação, o valor comprometido de um campo) e por isso
/// **não** cabe aqui.
///
/// `None` = não é um destes; o `match` de baixo decide.
fn track_slider_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> Option<bool> {
    let WidgetEvent::ValueChanged(id) = ev else {
        return None;
    };
    // O Bend é BIPOLAR: o track `0..1` vira `-1..1` aqui, na fronteira.
    if id == ids::VECTOR_ENVELOPE_BEND {
        return Some(forward_track(host, id, 0.5, |t| t.mul_add(2.0, -1.0)));
    }
    // O track de um parâmetro de efeito é NORMALIZADO `0..1`; a faixa real é do EFEITO e viaja
    // no snapshot. Quem reconverte é a shell, que a conhece — aqui o que se garante é que o
    // número que sai é o track, sem fingir ser outra coisa.
    if fx_param_of(id).is_some() || id == ids::VECTOR_BLEND_STEPS {
        return Some(forward_track(host, id, 0.0, |t| t));
    }
    if id == ids::VECTOR_MORPH_T {
        return Some(forward_track(host, id, 0.5, |t| t));
    }
    // O Offset do texto em caminho é uma FRAÇÃO do comprimento (o `startOffset` do SVG): track
    // e valor de documento são o MESMO número, então a fronteira não converte nada. É o único
    // slider do painel em que isso é verdade, e é o que torna o campo legível — `0.50` é meio
    // caminho, em qualquer curva.
    if id == ids::VECTOR_TEXTPATH_OFFSET {
        return Some(forward_track(host, id, 0.0, |t| t));
    }
    // Pattern on Path: o Start é FRAÇÃO (track == valor, como o Offset do texto); o Spacing mapeia
    // o track `0..1` na faixa `SPACING_MIN..SPACING_MAX` — a MESMA fronteira que o `scale`/`offset`
    // do chip no `populate` usa, senão o slider e o campo numérico divergiriam.
    if id == ids::VECTOR_PATTERNPATH_START {
        return Some(forward_track(host, id, 0.0, |t| t));
    }
    if id == ids::VECTOR_PATTERNPATH_END {
        return Some(forward_track(host, id, 1.0, |t| t));
    }
    // Slide é o CENTRO do trecho (fração, track == valor); o drain re-centra a janela.
    if id == ids::VECTOR_PATTERNPATH_SLIDE {
        return Some(forward_track(host, id, 0.5, |t| t));
    }
    if id == ids::VECTOR_PATTERNPATH_SPACING {
        return Some(forward_track(host, id, 0.5, |t| {
            t.mul_add(crate::SPACING_MAX - crate::SPACING_MIN, crate::SPACING_MIN)
        }));
    }
    // Offset: BIPOLAR `−OFFSET_MAX..OFFSET_MAX` (o mesmo mapa do Bend), `0.5` = zero.
    if id == ids::VECTOR_PATTERNPATH_OFFSET {
        return Some(forward_track(host, id, 0.5, |t| {
            t.mul_add(2.0 * crate::OFFSET_MAX, -crate::OFFSET_MAX)
        }));
    }
    // Rotation: BIPOLAR em GRAUS `−ROTATION_MAX..ROTATION_MAX`, `0.5` = deitado na curva. O mesmo
    // mapa do Offset — e o MESMO que o `populate` dá ao chip, senão slider e campo divergiriam.
    if id == ids::VECTOR_PATTERNPATH_ROTATION {
        return Some(forward_track(host, id, 0.5, crate::rotation_from_track));
    }
    // ⭐ Os sliders do PINCEL (plano 36, W4) — irmãos dos do padrão, e pela mesma porta.
    if let Some(consumed) = texpat::brush_slider_event(host, id) {
        return Some(consumed);
    }
    // ⭐ A CONTAGEM de cópias da simetria radial (e o chip ligado a ela) — ver `event_symmetry`.
    if let Some(consumed) = symmetry::segments_slider_event(host, id) {
        return Some(consumed);
    }
    if let Some(consumed) = texpat::texpat_slider_event(host, id) {
        return Some(consumed);
    }
    if let Some(consumed) = contour::contour_slider_event(host, id) {
        return Some(consumed);
    }
    // O arrasto de um punho da rampa chega como `ValueChanged` do TRILHO (o pai que cada
    // `CurvePoint` carrega) — antes dos sliders, porque não é um slider.
    if (0..ids::MAX_FILTER_ROWS).any(|r| id == ids::filter_ramp_id(r)) {
        return Some(filters::ramp_drag(host, id));
    }
    if let Some(consumed) = filters::filters_slider_event(host, id) {
        return Some(consumed);
    }
    None
}

/// Os sliders cujo valor viaja como **track normalizado** (`0..=1`) para o tool converter.
///
/// Lista e não `match` de propósito: ela é a resposta a *"este slider é encaminhado cru?"*, e um
/// slider novo entra aqui em vez de num braço próprio com um corpo copiado.
const FORWARDED_TRACK_SLIDERS: &[ph2d_a11y::NodeId] = &[
    ids::VECTOR_WIDTH,
    ids::VECTOR_PENCIL_FIDELITY,
    ids::VECTOR_PENCIL_STABILIZER,
    // A DURAÇÃO da transição de estado (W7): o track vai cru e a shell o multiplica pela régua
    // — a mesma que o painel usa para encher o trilho.
    ids::VECTOR_STATE_DURATION,
    // A MOLA: rigidez e amortecimento seguem a mesma rota — track cru, e a shell aplica a régua
    // afim. Um braço próprio aqui seria um corpo copiado com um `unwrap_or` a divergir.
    ids::VECTOR_STATE_STIFFNESS,
    ids::VECTOR_STATE_DAMPING,
];

/// É este id uma linha de algum dos dois popovers de mistura?
fn is_blend_option(id: ph2d_a11y::NodeId) -> bool {
    state::blend_option_index(id).is_some() || state::paint_blend_option_index(id).is_some()
}

/// Encaminha a escolha para o dono do popover. ⛔ A ordem é a da pergunta acima, e o `else`
/// **não** é um catch-all: um id que não seja de nenhum dos dois nunca chega aqui (o guarda do
/// braço já o filtrou).
fn blend_option(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    if state::blend_option_index(id).is_some() {
        clicks::pick_object_blend(host, id)
    } else {
        clicks::pick_layer_blend(host, id)
    }
}

pub(crate) fn apply_event(
    _state: &mut VectorPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let Some(consumed) = track_slider_event(host, ev) {
        return EventOutcome::from_bool(consumed);
    }
    let consumed = match ev {
        // **Os sliders cujo TRACK vai cru para o tool** — Width e os dois knobs do lápis. Um braço
        // só porque a pergunta é literalmente a mesma (*qual é o novo track deste slider?*); o que
        // difere é o mapeamento para a unidade autorada, e esse é do tool. Três braços com corpos
        // idênticos é como o quarto slider nasce com um `unwrap_or` diferente dos outros.
        WidgetEvent::ValueChanged(id) if FORWARDED_TRACK_SLIDERS.contains(&id) => {
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
        // Os nove campos do AUTO LAYOUT seguem exactamente a mesma rota, e pela mesma razão: o
        // valor mora no COMPONENTE, então quem o escreve é a shell. A lista é a que o `populate`
        // regista — um campo novo entra numa lista só.
        //
        // ⚠️ **O Z-INDEX entrou aqui em 2026-08-04, e a ausência dele foi um bug de PRODUTO:**
        // ele era pintado, registado no `populate` e vivo sob o mouse — o artista clicava, ganhava
        // foco, digitava e via o número mudar —, mas o `ValueChanged` do commit caía no catch-all
        // e **nunca virava `SetValue`**. Enio: *"Z-index não funcionou"*. Um campo que aceita
        // teclas e não fala com ninguém é a forma mais cara de um controlo nascer morto, porque
        // parece vivo. O gate que existia media o CLIQUE (o foco); o que faltava mede o COMMIT.
        WidgetEvent::ValueChanged(id) if is_shell_owned_number(id) => forward_number(host, id),
        // **Campo de forma** (genérico): o id carrega o ÍNDICE do parâmetro no catálogo;
        // a forma em foco diz o que ele significa. Encaminha o VALOR (não um track).
        WidgetEvent::ValueChanged(id) if state::shape_field_index(id).is_some() => {
            forward_number(host, id)
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
                || id == ids::VECTOR_TEXT_WRAP_W
                || id == ids::VECTOR_STROKE_OPACITY
                || id == ids::VECTOR_FILL_OPACITY
                // ⭐ A opacidade do OBJECTO (estudo 42 item 2) — mesmo formato de track `0..1`, e
                // ⚠️ **outro sujeito**: as duas de cima são a tinta da ferramenta, esta é a forma
                // selecionada. Elas convivem, e o id é o que as separa.
                || id == ids::VECTOR_OBJ_OPACITY
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
                || id == ids::VECTOR_TEXT_WRAP_W_NUM
                || id == ids::VECTOR_STROKE_OPACITY_NUM
                || id == ids::VECTOR_FILL_OPACITY_NUM
                || id == ids::VECTOR_OBJ_OPACITY_NUM
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
        // ⭐ Uma linha de um dos DOIS popovers de mistura — o do objecto (item 2) e o de uma
        // CAMADA (item 4). ⚠️ **Os dois num braço só**, e não por economia: eles têm espaços de
        // ids próprios de propósito (podem existir no mesmo frame), e é justamente por isso que a
        // pergunta *"qual dos dois?"* tem de ser feita **uma vez, num sítio** — dois braços
        // vizinhos com a mesma forma são onde o segundo envelhece.
        WidgetEvent::Click(id) if is_blend_option(id) => blend_option(host, id),
        // **A junção do Offset Path** — panel-local, ver [`pick_expand_join`].
        WidgetEvent::Click(id) if expand_join_index(id).is_some() => pick_expand_join(host, id),
        // **O lado do Offset Path** (Outer/Inner/Both) — panel-local, ver [`pick_expand_side`].
        WidgetEvent::Click(id) if expand_side_index(id).is_some() => pick_expand_side(host, id),
        // Draw-mode buttons + Boolean buttons: forward the Click over the generic
        // tool channel. Mode clicks land on `VectorTool::handle_panel_event`
        // (sets the mode); Boolean clicks are picked up by the shell drain, which
        // applies the op to the document (they are not Style edits, so the tool
        // ignores them). Same forwarder shape either way.
        // Uma linha do picker de TOKEN aberto: fecha o chip e encaminha a escolha.
        //
        // ⚠️ O light-dismiss NÃO dispara aqui — o clique é DENTRO do popover —, então sem este
        // braço o card fica aberto por cima da seção depois de escolher, e o artista tem de o
        // dispensar à mão. É a mesma cicatriz que a row de família do dropdown de FONTE já
        // carrega, e a razão de este braço vir ANTES do encaminhador genérico.
        WidgetEvent::Click(id) if token_option_chip(id).is_some() => pick_token(host, id),
        // ⭐ **O COMMIT do nome de um sinal** — Enter (`Submit`) ou o campo a perder o foco
        // (`Blur`), as duas portas que o dispatch global já emite. ⚠️ **As duas, e não só o
        // Enter:** um campo abandonado com o nome certo escrito dentro dele lê-se como *"eu
        // autorei isto"*, e exigir o Enter faria o artista descobrir a regra pelo silêncio.
        //
        // ⚠️ **Ele viaja por `SelectOption`, e não por um variante novo:** o `PanelEvent` é
        // CONTRATO CONGELADO (§6, quatro variantes), e o `SelectOption(NodeId, String)` já é o
        // canal string-valued deste app — o Painter carrega nele payloads estruturados
        // (`"layer:channel:index:x:y"`) que não são opção de rádio nenhuma. Um variante novo
        // custaria ADR + Coord-only para dizer o que este já diz.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if signal_name_row(id).is_some() => {
            signal_name_commit(host, id)
        }
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
        // As PONTAS (Head Size / Head Round) e o CONECTOR (Route / Jetty / Spread / Corner)
        // tratam os seus ids nos módulos deles — que é onde o snapshot de cada um vive.
        // Delegar AQUI (em vez de mais seis arms) é o que mantém `apply_event` sob o teto de
        // 200 LOC por função.
        other => {
            crate::paint_markers::apply_event(host, other)
                || crate::paint_connector::apply_event(host, other)
                || crate::paint_layout::apply_event(host, other)
                || crate::font_dropdown::apply_event(host, other)
        }
    };
    EventOutcome::from_bool(consumed)
}

/// O índice da linha de ligação a que um id de campo de NOME pertence, se ele for de alguma.
///
/// ⚠️ A varredura é sobre o MESMO teto que o `populate` regista e que o `paint` percorre
/// (`MAX_SIGNAL_BINDINGS`): os ids são hashes de nome, então não há aritmética que os inverta, e
/// um teto que só um dos três conhecesse deixaria as últimas linhas pintadas e mudas.
fn signal_name_row(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..ids::MAX_SIGNAL_BINDINGS).find(|&i| ids::vector_state_signal_name_id(i) == id)
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

/// **Uma linha do picker de TOKEN foi escolhida:** fecha o chip e encaminha a escolha.
///
/// ⚠️ O light-dismiss NÃO dispara aqui — o clique é DENTRO do popover —, então sem o fecho o card
/// fica aberto por cima da secção depois de escolher e o artista tem de o dispensar à mão (Enio,
/// 2026-08-02). É a mesma cicatriz que a row de família do dropdown de FONTE já carrega.
///
/// ⚠️ O corpo vive numa função e não no braço, como o `pick_expand_side` ao lado: o `apply_event`
/// está no teto de 200 LOC por função, e onze linhas dentro do `match` o estouram — foi o que
/// aconteceu ao escrever isto.
fn pick_token(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    if let Some(chip) = token_option_chip(id)
        && let Some(InteractiveState::Dropdown { open, .. }) = host.store_mut().get_mut(chip)
    {
        *open = false;
    }
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::Click(id)));
    true
}

/// **De que CHIP esta linha de picker de token é** — `None` se o id não é uma linha de picker.
///
/// A enumeração é a mesma do `event_clicks::is_token_option` (um `NodeId` é hash e não se
/// inverte); o que muda é a resposta, porque aqui a pergunta é *de quem é este popover?*.
fn token_option_chip(id: ph2d_a11y::NodeId) -> Option<ph2d_a11y::NodeId> {
    ids::TOKEN_SLOTS
        .iter()
        .find(|s| (0..=s.table.len()).any(|i| id == ids::vector_token_option_id(s.code, i)))
        .map(|s| s.chip)
}

/// A lista de ids cujo `Click` é apenas ENCAMINHADO — irmão pelo teto de 600 LOC do painel.
#[path = "event_clicks.rs"]
mod clicks;
use clicks::forwards_plain_click;

/// O roteamento dos controles do **Contour** — irmão pelo teto de 600 LOC do painel.
#[path = "event_contour.rs"]
mod contour;

/// O roteamento dos sliders do **Texture Pattern** (plano 33) — irmão pelo mesmo teto, e pelo
/// mesmo corte: um assunto, uma porta.
#[path = "event_texpat.rs"]
mod texpat;

#[path = "event_filters.rs"]
mod filters;

/// ⭐ **O slider de SEGMENTS da simetria radial** — irmão pelo mesmo teto, e pelo mesmo corte.
/// Ele nasceu aqui porque o controlo estava MORTO: quatro sítios a declará-lo, pintá-lo e
/// registá-lo, e **zero** braços de evento (caça de 2026-08-30, ver o módulo).
#[path = "event_symmetry.rs"]
mod symmetry;

/// **Os dois selectores do Offset Path** — irmão pelo mesmo teto. Eles saíram deste ficheiro
/// quando a porta da simetria acima o levou ao cap de 600: *levar só o novo deixaria o número
/// onde estava, e ficar no mesmo sítio não é encolher.*
#[path = "event_expand.rs"]
mod expand;
use expand::{expand_join_index, expand_side_index, pick_expand_join, pick_expand_side};

/// **O COMMIT do campo de nome de um sinal** (item 4 do estudo dos contêineres) — extraído do
/// [`apply_event`] pelo teto de 200 LOC por função.
///
/// ⚠️ Ele viaja por `SelectOption`, e não por um variante próprio: o `PanelEvent` é **contrato
/// congelado** (§6), e o `SelectOption(NodeId, String)` já é o canal string-valued deste app — o
/// Painter carrega `"layer:channel:index:x:y"` nele.
///
/// ⚠️ E o braço nasceu dentro do irmão, empurrando-o de 195 para 213 sem ninguém ver: o gate mora
/// em `ph2d-editor-core/tests/`, e um fechamento por `cargo test -p ph2d-panel-vector` não o
/// alcança.
fn signal_name_commit(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    let text = host.store().text(id).unwrap_or_default().to_string();
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            id, text,
        )));
    true
}

/// **Os campos numéricos cujo valor mora no DOCUMENTO**, e por isso viajam para a shell.
///
/// ⚠️ A lista dos treze do auto layout é a que o `populate` regista — um campo novo entra numa
/// lista só. E o **Z-INDEX está aqui por um bug de produto**: ele era pintado, registado e vivo sob
/// o rato, mas o `ValueChanged` do commit caía no catch-all e nunca virava `SetValue` (Enio,
/// 2026-08-04: *"Z-index não funcionou"*). Um campo que aceita teclas e não fala com ninguém é a
/// forma mais cara de um controlo nascer morto, porque parece vivo.
fn is_shell_owned_number(id: ph2d_a11y::NodeId) -> bool {
    id == ids::VECTOR_TRANSFORM_X
        || id == ids::VECTOR_TRANSFORM_Y
        || id == ids::VECTOR_TRANSFORM_W
        || id == ids::VECTOR_TRANSFORM_H
        || id == ids::VECTOR_ARRANGE_Z
        || id == ids::VECTOR_VERT_X
        || id == ids::VECTOR_VERT_Y
        || crate::populate::layout::LAYOUT_FIELDS.contains(&id)
}

/// **Encaminha o VALOR de um campo numérico** — a porta que os dois braços de commit partilham.
///
/// ⚠️ Ela existe porque o corpo estava escrito DUAS vezes, uma em cada braço, e duas cópias do
/// mesmo encaminhamento é como a terceira nasce com um `unwrap_or` diferente das outras.
fn forward_number(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    let val = host.store().number_value(id).unwrap_or(0.0);
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, val)));
    true
}
