//! **Os controles da seção COMPONENT** — irmão do [`super::populate_anchors`], mesma razão.
//!
//! ⚠️ Sem este registro os botões ficariam pintados, com hit-rect, e **MORTOS sob o mouse** — a
//! checagem de focabilidade mora no store. É o defeito que este painel já pagou cinco vezes,
//! e o seam é o que o prova.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

use crate::ids;

/// Os verbos de seção.
///
/// ⚠️ **A lista é uma só, e é a mesma que o `paint_components` desenha.** Um verbo novo entra aqui
/// e ali; o gate de seam clica **todos** eles, que é o que impede a lista de registro de ficar
/// para trás da lista pintada.
pub(crate) const COMPONENT_BUTTONS: &[ph2d_a11y::NodeId] = &[
    ids::VECTOR_COMPONENT_CREATE,
    ids::VECTOR_COMPONENT_PLACE,
    ids::VECTOR_COMPONENT_DETACH,
    ids::VECTOR_COMPONENT_RESET,
    ids::VECTOR_COMPONENT_UPDATE_MAIN,
    ids::VECTOR_COMPONENT_SWAP,
];

pub(super) fn component_controls(store: &mut WidgetStore) {
    for &id in COMPONENT_BUTTONS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // **As LINHAS de peça** (W5b). O teto é registado SEMPRE, e não a contagem viva: o `populate`
    // corre antes do corpo, então registar `pieces().len()` acoplaria o registo à ordem de duas
    // fases — e uma peça a mais num frame nasceria morta sob o mouse até ao frame seguinte. Os
    // slots a mais são widgets que nada pinta, que é o que o `paint` decide.
    for row in 0..ids::MAX_INSTANCE_PIECES {
        store.register(
            ids::vector_instance_piece_show_id(row),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        // ⚠️ A swatch é alvo de PICKER, não botão: registá-la como botão faria o clique acender o
        // widget e **nunca abrir o picker** — a cor ficaria ineditável com todos os gates verdes.
        store.register_picker_swatch(ids::vector_instance_piece_colour_id(row));
    }
    // **Os chips de VARIANT** (W5c). O teto é registado SEMPRE, pela mesma razão das peças: o
    // `populate` corre antes do corpo, e registar a contagem viva faria um chip novo nascer morto
    // sob o mouse até ao frame seguinte.
    for axis in 0..ids::MAX_VARIANT_AXES {
        for value in 0..ids::MAX_VARIANT_VALUES {
            store.register(
                ids::vector_variant_option_id(axis, value),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    // **A PELE por-widget** (W6.2): os dois verbos e os chips de tipo. Mesmo teto-sempre-registado
    // pela mesma razão — e aqui o esquecimento seria mais caro, porque a seção INTEIRA fica muda.
    for &id in WIDGET_BUTTONS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for i in 0..ids::MAX_WIDGET_KINDS {
        store.register(
            ids::vector_widget_kind_id(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // O chip do PICKER de ícone: um `Dropdown`, para o despacho genérico abrir/fechar o popover.
    // ⚠️ As LINHAS dele não são registadas aqui — elas nascem no `paint` do popover, como as do
    // dropdown de fontes: são 137 e só existem enquanto a lista está aberta.
    store.register(
        ids::VECTOR_WIDGET_ICON_DD,
        InteractiveState::Dropdown {
            state: ph2d_editor_core::widget::DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    // **OS ESTADOS de UI** (W7): três verbos por papel. Registados para TODO papel do catálogo e
    // não só para os que a seleção de agora tem — o `populate` corre uma vez na instalação do
    // painel, quando não existe seleção nenhuma, então registar "o que faz sentido agora" seria
    // registar nada.
    for i in 0..ids::MAX_STATE_ROLES {
        for id in [
            ids::vector_state_record_id(i),
            ids::vector_state_clear_id(i),
            ids::vector_state_apply_id(i),
        ] {
            store.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    signal_table_controls(store);
    easing_controls(store);
}

/// **O SELETOR DE CURVA e os dois trilhos da MOLA** (W7) — extraídos do [`component_controls`] pelo
/// mesmo teto, e pelo mesmo corte: *que forma tem a transição* é uma pergunta, e *que poses este
/// hospedeiro tem* é outra.
fn easing_controls(store: &mut WidgetStore) {
    // **O SELETOR DE CURVA** (W7): as onze famílias e os três modos. Registados TODOS, e não só
    // os da família de agora — o `populate` corre uma vez, sem seleção, e a fileira do modo é
    // pintada ou escondida por frame conforme a família escolhida use o modo ou não. Registar "o
    // que está visível agora" deixaria os chips do modo mortos sob o rato no primeiro `Elastic`.
    for i in 0..ids::MAX_EASING_FAMILIES {
        store.register(
            ids::vector_easing_family_id(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    for i in 0..ids::MAX_EASING_MODES {
        store.register(
            ids::vector_easing_mode_id(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // **O MODO DE PREVIEW** (W7r): o interruptor que entrega o rato aos papéis.
    store.register(
        ids::VECTOR_STATE_PREVIEW,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // **Mover o widget com TODOS os estados** (W7r). ⚠️ Registado como `Button`, e não como
    // `Checkbox`: é o precedente do `VECTOR_TRANSFORM_RESIZE_BOX`, o outro checkbox deste painel
    // — o que atravessa o barramento é um `Click`, e quem decide o DESENHO é o `checkbox_row`.
    // Duas convenções de registo para a mesma caixa dariam dois caminhos de evento.
    store.register(
        ids::VECTOR_STATE_MOVE_ALL,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // A DURAÇÃO: o slider e o chip que o espelha. `link_slider_number_mapped` com escala
    // `MAX_DURATION_S` é o que faz o trilho `0..1` e o número em SEGUNDOS serem o mesmo valor —
    // sem ele o artista arrastaria o trilho e leria um número que não é o dele.
    store.register(
        ids::VECTOR_STATE_DURATION,
        ph2d_editor_core::interaction::InteractiveState::Slider {
            state: ph2d_editor_core::widget::SliderState::Normal,
            value: STATE_DURATION_DEFAULT_S / STATE_DURATION_MAX_S,
            orientation: ph2d_editor_core::widget::SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::VECTOR_STATE_DURATION_NUM,
        ph2d_editor_core::interaction::InteractiveState::NumberInput {
            state: ph2d_editor_core::widget::TextInputState::Normal,
            value: f64::from(STATE_DURATION_DEFAULT_S),
            buffer: format!("{STATE_DURATION_DEFAULT_S:.2}"),
            caret: 0,
            last_committed: f64::from(STATE_DURATION_DEFAULT_S),
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(
        ids::VECTOR_STATE_DURATION,
        ids::VECTOR_STATE_DURATION_NUM,
        STATE_DURATION_MAX_S,
        0.0,
    );
    // **A MOLA** — o checkbox e os dois sliders. Eles são REGISTADOS sempre, mesmo quando o
    // `paint` não os desenha: registar é dizer *"este widget existe e é focável"*, e a decisão de
    // o pintar é do estado publicado. É o mesmo protocolo das linhas de duração, que também não
    // são pintadas com a preview ligada.
    store.register(
        ids::VECTOR_STATE_SPRING,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    for (slider, num, lo, hi, def) in [
        (
            ids::VECTOR_STATE_STIFFNESS,
            ids::VECTOR_STATE_STIFFNESS_NUM,
            SPRING_STIFFNESS_MIN,
            SPRING_STIFFNESS_MAX,
            SPRING_STIFFNESS_DEFAULT,
        ),
        (
            ids::VECTOR_STATE_DAMPING,
            ids::VECTOR_STATE_DAMPING_NUM,
            SPRING_DAMPING_MIN,
            SPRING_DAMPING_MAX,
            SPRING_DAMPING_DEFAULT,
        ),
    ] {
        store.register(
            slider,
            ph2d_editor_core::interaction::InteractiveState::Slider {
                state: ph2d_editor_core::widget::SliderState::Normal,
                value: (def - lo) / (hi - lo),
                orientation: ph2d_editor_core::widget::SliderOrientation::Horizontal,
            },
        );
        store.register(
            num,
            ph2d_editor_core::interaction::InteractiveState::NumberInput {
                state: ph2d_editor_core::widget::TextInputState::Normal,
                value: f64::from(def),
                buffer: format!("{def:.2}"),
                caret: 0,
                last_committed: f64::from(def),
                selection_anchor: None,
            },
        );
        // ⚠️ O mapeamento é AFIM (`escala`, `offset`), e não só uma escala: a régua da mola não
        // começa em zero — `MIN_STIFFNESS` é 1 e `MIN_DAMPING` é 0,1. Sem o offset o trilho no
        // canto esquerdo leria um número que a porta depois clampa.
        store.link_slider_number_mapped(slider, num, hi - lo, lo);
    }
}

/// As réguas da MOLA — os mesmos números do modelo (`ph2d_ui_state::*`), com gate a compará-los.
///
/// ⚠️ Estes **não são valores de design**: são as pontas da régua de uma grandeza FÍSICA (rigidez
/// em unidades de mola, amortecimento adimensional). Não existe token de escala para *"quão dura é
/// uma mola"*, e inventar um poria uma constante de física dentro do design system.
const SPRING_STIFFNESS_MIN: f32 = 1.0;
const SPRING_STIFFNESS_MAX: f32 = 60.0; // LITERAL-PX-OK: régua de FÍSICA, não de design
const SPRING_STIFFNESS_DEFAULT: f32 = 12.0; // LITERAL-PX-OK: régua de FÍSICA, não de design
const SPRING_DAMPING_MIN: f32 = 0.1; // LITERAL-PX-OK: régua de FÍSICA, não de design
const SPRING_DAMPING_MAX: f32 = 2.0;
const SPRING_DAMPING_DEFAULT: f32 = 1.0;

/// O default da duração — **o token do design system**, cujo doc diz literalmente *"button
/// press, icon swap"*: a pergunta *"quanto tempo leva a reação de um controle neste app?"* já
/// tinha dono, e um literal aqui seria a segunda resposta a ela.
///
/// ⚠️ O modelo (`ph2d_ui_state::DEFAULT_DURATION_S`) carrega o mesmo número porque ele precisa de
/// responder `timing()` para um hospedeiro que ninguém autorou, e uma crate-folha de dados não
/// depende do design system. O gate compara os dois.
const STATE_DURATION_DEFAULT_S: f32 = ph2d_tokens::Duration::Fast.secs();
/// O teto do slider — o MESMO número do modelo (`ph2d_ui_state::MAX_DURATION_S`), com gate.
const STATE_DURATION_MAX_S: f32 = 2.0;

/// Os verbos da seção WIDGET SKIN — a mesma lista que o `paint_widget` desenha.
///
/// ⚠️ Registados **todos, sempre**, mesmo os que a seção só pinta em certos estados: o registro é
/// o que torna um id CLICÁVEL, e registá-lo por estado significaria que o botão nasce morto sob o
/// rato no frame em que aparece — o defeito exato que as 36 células da matriz de física pagaram.
pub(crate) const WIDGET_BUTTONS: &[ph2d_a11y::NodeId] = &[
    ids::VECTOR_WIDGET_WEAR,
    ids::VECTOR_WIDGET_REMOVE,
    ids::VECTOR_WIDGET_BIND,
    ids::VECTOR_WIDGET_UNBIND,
];

/// **A TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres) — extraída do
/// [`component_controls`] pelo teto de 200 LOC por função, e o corte é por ASSUNTO: aqui mora
/// *quem escuta um nome*, e o que fica lá responde outra pergunta (*que poses este hospedeiro
/// tem*).
///
/// ⚠️ **Ela nasceu DENTRO do irmão e passou por cima do teto sem ninguém ver:** o gate mora em
/// `ph2d-editor-core/tests/`, e um fechamento por `cargo test -p ph2d-panel-vector` não o alcança —
/// a mesma causa estrutural que três linhas deste repo já pagaram. Medido: a função estava em 200
/// no `main` e saiu da wave anterior em 232.
fn signal_table_controls(store: &mut WidgetStore) {
    // ⭐ **A TABELA SINAL → PAPEL** (item 4 do estudo dos contêineres): por linha do pool, os
    // quatro chips de papel e a lixeira; mais o botão que acrescenta. Registados para o POOL
    // inteiro pela mesma razão dos verbos acima — o `populate` corre uma vez, sem seleção, então
    // registar "as ligações de agora" seria registar nenhuma, e as linhas nasceriam pintadas e
    // mortas sob o rato assim que o artista apertasse *Add*.
    //
    // ⚠️ **O campo de NOME não é registado aqui**, e é a única assimetria: ele é semeado pelo
    // espelho (`paint_signals::mirror`), que corre por frame com a loja mutável porque o buffer
    // tem de acompanhar o texto autorado. Registá-lo duas vezes daria duas respostas para *"o
    // que está escrito neste campo?"*.
    for i in 0..ids::MAX_SIGNAL_BINDINGS {
        for r in 0..ids::MAX_STATE_ROLES {
            store.register(
                ids::vector_state_signal_role_id(i, r),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        store.register(
            ids::vector_state_signal_remove_id(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.register(
        ids::VECTOR_STATE_SIGNAL_ADD,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
