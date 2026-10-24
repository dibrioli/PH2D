//! **O CONTROLO de cada fileira da secção Script** — irmão de [`super::script`] pelo tecto de 600
//! LOC dos painéis: lá a secção e a [`super::script`] `linha` (nome · controlo · `Reset`), aqui o
//! que cada TIPO pinta e a coluna do losango dela (F27 E).

use super::*;
use ph2d_editor_core::script_edits::{InspectorScriptProp, InspectorScriptValue};
use ph2d_editor_core::widget::key_mark;
use ph2d_editor_core::widget::{
    ColorSwatch, Dropdown, SwatchSize, paint_color_swatch, paint_dropdown_chip,
};

/// Os campos que a linha `i` pinta, pelo tipo (a posição são duas caixas e um losango).
pub(super) fn campos_da_linha(i: usize, p: &InspectorScriptProp) -> Vec<NodeId> {
    match &p.value {
        InspectorScriptValue::Number(_) => vec![ids::INSP_SCRIPT_NUM[i]],
        InspectorScriptValue::Bool(_) => vec![ids::INSP_SCRIPT_BOOL[i]],
        InspectorScriptValue::Text(_) if !p.options.is_empty() => vec![ids::INSP_SCRIPT_ENUM[i]],
        InspectorScriptValue::Text(_) => vec![ids::INSP_SCRIPT_TEXT[i]],
        InspectorScriptValue::Vec2(_) => {
            vec![ids::INSP_SCRIPT_VEC2_X[i], ids::INSP_SCRIPT_VEC2_Y[i]]
        }
        InspectorScriptValue::Color(_) => vec![ids::INSP_SCRIPT_COLOR[i]],
    }
}

/// ⭐⭐ **O CONTROLO de uma fileira — um por TIPO, e só esse.**
///
/// ⚠️ **Saiu do [`linha`] pelo tecto de 200 LOC por função** quando a posição e a cor chegaram, e o
/// corte é por RESPONSABILIDADE: a [`linha`] reparte a fileira (nome · controlo · `Reset` · o
/// refluxo) e esta pinta **o que o tipo pede**. ⛔ Ela saiu **verbatim**, e é isso que a mantém
/// honesta: *um corte que muda o que se pinta não é um corte, é uma mudança de comportamento com
/// cara de arrumação.*
#[allow(clippy::too_many_arguments)]
pub(super) fn controlo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    ctrl: Rect,
    i: usize,
    p: &InspectorScriptProp,
    linha: (f32, ph2d_editor_core::property_row::Seccao),
    (dot, campos): (Option<Rect>, &[NodeId]),
) {
    let ctrl_w = ctrl.w;
    let row_y = ctrl.y;
    // A coluna do losango (o canal registado pela `linha`): a da caixa é a da fileira inteira.
    if let Some(dot) = dot
        && let Some(alvo) = key_mark::paint_decorator_for(scene, theme, dot, campos)
    {
        hit_index.register(alvo, dot);
    }
    match &p.value {
        InspectorScriptValue::Number(_) => {
            let id = ids::INSP_SCRIPT_NUM[i];
            hit_index.register(id, ctrl);
            let (state, value, buffer, caret, anchor) = read_number_input(store, id);
            let input = NumberInput::new(id, "", value)
                .step(p.step.unwrap_or(1.0))
                .visual((state, store.hover_live(id)));
            paint_number_input_with_buffer(
                &input,
                Some(buffer),
                store.number_input_mixed(id),
                caret,
                anchor,
                ctrl,
                scene,
                text_system,
                theme,
            );
        }
        InspectorScriptValue::Bool(_) => {
            let id = ids::INSP_SCRIPT_BOOL[i];
            hit_index.register(id, ctrl);
            let (_, value) = store
                .checkbox(id)
                .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
            // A caixa recebe a fileira INTEIRA com a secção das vizinhas: só assim a coluna do
            // valor dela é a delas (dentro do `ctrl` ela partia-o outra vez em nome + valor).
            let (x, seccao) = linha;
            let fileira = Rect::new(x, row_y, ctrl.x + ctrl_w - x, ctrl.h);
            super::rows::coluna_da_caixa(scene, theme, hit_index, fileira, id, "");
            paint_checkbox(
                &Checkbox::new(id, "")
                    .coluna_de_fora()
                    .visual(store.checkbox_visual(id))
                    .value(value)
                    .seccao(seccao),
                Rect::new(x, row_y, ctrl.x + ctrl_w - x, ctrl.h),
                scene,
                text_system,
                theme,
            );
        }
        // ⭐⭐⭐ **Um texto com LISTA é um CHIP; sem ela é um campo livre.**
        //
        // ⚠️⚠️ **O discriminador é a LISTA e não o tipo**, e é isso que faz o enum não custar uma
        // variante no fio: o valor continua a ser o texto que o script compara.
        InspectorScriptValue::Text(_) if !p.options.is_empty() => {
            let id = ids::INSP_SCRIPT_ENUM[i];
            hit_index.register(id, ctrl);
            let aberto = matches!(
                store.get(id),
                Some(InteractiveState::Dropdown { open: true, .. })
            );
            let mut dd = Dropdown::new(id, "", super::script::opcoes_da_linha(p)).open(aberto);
            // ⚠️ **A escolha vem do SNAPSHOT e o `open` do store** — a mesma lei do chip do
            // gatilho. E ⛔ um valor FORA da lista não selecciona nada: ele nem chega aqui (a lei
            // do `resolve` não o aplica), e se chegasse o `position` devolveria `None` em vez de
            // encostar à primeira — *encostar é o «aceita e mente» que esta wave existe para
            // matar*.
            if let InspectorScriptValue::Text(t) = &p.value
                && let Some(j) = p.options.iter().position(|o| o == t)
            {
                dd.select(j);
            }
            paint_dropdown_chip(&dd, ctrl, scene, text_system, theme);
            // ⛔⛔ **UM popover de cada vez, e a cerca é esta linha:** a tabela de ids das opções
            // é PARTILHADA por todas as linhas, logo dois chips abertos registariam os mesmos ids
            // duas vezes e o índice de acerto ficaria com o ÚLTIMO — o artista carregaria numa
            // opção e escolheria para a outra linha. O `set` guarda o último, e o `take` do passe
            // diferido pinta um só.
            if aberto {
                crate::state_popovers::set_pending_script_enum(Some((i, ctrl)));
            }
        }
        // ⭐⭐⭐ **UMA posição é UMA fileira com DOIS campos** — X e Y, na ordem em que toda a casa
        // os escreve.
        //
        // ⛔ **Sem letras de eixo numa coluna própria**, e a decisão não é desta linha: a linha de
        // propriedade do `line/UIUX` mediu que elas custam a disposição que o dono aprovou, e a
        // fileira do `Transform` — o padrão do app — largou-as.
        InspectorScriptValue::Vec2(v) => {
            let gap_eixos = ph2d_tokens::control_gap_px();
            let meia = ((ctrl_w - gap_eixos) * 0.5).max(0.0);
            for (k, &id) in [ids::INSP_SCRIPT_VEC2_X[i], ids::INSP_SCRIPT_VEC2_Y[i]]
                .iter()
                .enumerate()
            {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "k e' 0 ou 1: nao ha' precisao a perder"
                )]
                let x = ctrl.x + (meia + gap_eixos) * k as f32;
                let r = Rect::new(x, row_y, meia, ctrl.h);
                hit_index.register(id, r);
                let (state, _, buffer, caret, anchor) = read_number_input(store, id);
                let input = NumberInput::new(id, "", v[k])
                    .step(p.step.unwrap_or(1.0))
                    .visual((state, store.hover_live(id)));
                paint_number_input_with_buffer(
                    &input,
                    Some(buffer),
                    store.number_input_mixed(id),
                    caret,
                    anchor,
                    r,
                    scene,
                    text_system,
                    theme,
                );
            }
        }
        // ⭐⭐⭐ **UMA cor é UMA amostra** — e ela abre o selector da casa, que é a única superfície
        // onde uma cor se JULGA.
        //
        // ⚠️ **A amostra ocupa a coluna do controlo inteira**, como o campo de um número: ela é o
        // controlo desta fileira, não um enfeite ao lado de um.
        InspectorScriptValue::Color(c) => {
            let id = ids::INSP_SCRIPT_COLOR[i];
            hit_index.register(id, ctrl);
            // ⚠️ **A cor VIVA do store ganha à do instantâneo enquanto o selector está aberto** —
            // é isso que faz o desenho seguir o dedo do artista antes de o commit chegar; é o
            // mesmo regime das amostras de tinta do Sprite.
            let rgba = store
                .widget_color(id)
                .unwrap_or_else(|| crate::state_tint::cor_do_script(*c));
            paint_color_swatch(
                &ColorSwatch::new(id, "", rgba).size(SwatchSize::Sm),
                ctrl,
                scene,
                theme,
            );
        }
        InspectorScriptValue::Text(_) => {
            let id = ids::INSP_SCRIPT_TEXT[i];
            hit_index.register(id, ctrl);
            let (state, text, caret, anchor) = match store.get(id) {
                Some(InteractiveState::TextInput {
                    state,
                    text,
                    caret,
                    selection_anchor,
                }) => (*state, Some(text.as_str()), *caret, *selection_anchor),
                _ => (TextInputState::Normal, None, 0, None),
            };
            let input = TextInput::new(id, "").visual((state, store.hover_live(id)));
            paint_text_input_with_buffer(
                &input,
                text,
                Some(caret),
                anchor,
                ctrl,
                scene,
                text_system,
                theme,
            );
        }
    }
}
