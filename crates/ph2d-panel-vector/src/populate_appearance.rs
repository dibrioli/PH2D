//! ⭐ **O REGISTO da secção APPEARANCE** — a opacidade e a mistura do OBJECTO (estudo 42 item 2) e
//! a PILHA de tintas (item 4). Módulo irmão do [`crate::populate`] pelo tecto de 600 LOC do painel,
//! e o corte é por RESPONSABILIDADE: ali regista-se o que a ferramenta e a forma têm; aqui, o que
//! descreve a APARÊNCIA delas.
//!
//! ⚠️ **Registar é o passo que não se vê e sem o qual nada funciona:** um controlo pintado e
//! hit-indexado que não esteja aqui **nunca fica focável**, então o Down/Up nunca dispara — ele
//! desenha, aceita o cursor e é **mudo**. Foi assim que o `VECTOR_PAINT_BLEND` nasceu, e o
//! `hit_indexed_ids_are_registered` apanhou-o.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::DropdownState;

use crate::ids;
use crate::populate::{button, number_field, slider_chip, world_number_field};

/// ⭐⭐⭐ **A APARÊNCIA do objecto** (estudo 42 item 2): o slider de opacidade + o chip de mistura e
/// as linhas do popover dele.
///
/// ⚠️ **As opções são registadas por ÍNDICE**, como as pontas do traço e as formas do catálogo: um
/// modo novo no vocabulário entra na lista derivada e já nasce clicável, sem tocar aqui. O teto é
/// o do vocabulário (`MAX_BLEND_MODES`), e não um número escolhido — a lista OFERECIDA é sempre um
/// subconjunto dele.
pub(crate) fn populate_appearance(store: &mut WidgetStore) {
    slider_chip(
        store,
        ids::VECTOR_OBJ_OPACITY,
        ids::VECTOR_OBJ_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        ph2d_tool_vector::params::OPACITY_SLIDER_SCALE,
        ph2d_tool_vector::params::OPACITY_SLIDER_OFFSET,
    );
    store.register_if_absent(
        ids::VECTOR_OBJ_BLEND,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    for i in 0..usize::from(ph2d_vec_scene::MAX_BLEND_MODES) {
        button(store, ids::vector_obj_blend_option_id(i));
    }
    populate_paint_stack(store);
}

/// ⭐⭐⭐ **A PILHA DE APARÊNCIA** (estudo 42 item 4) — os dois botões que a fazem crescer, os cinco
/// controlos de CADA linha, e as três propriedades da camada aberta.
///
/// ⚠️ **Por ÍNDICE sobre o espaço FIXO** (`MAX_PAINT_LAYERS`), como as opções acima: uma camada
/// criada neste frame tem de nascer clicável **no mesmo frame**, e registar só as que existem hoje
/// faria a linha nova ser pintada, registada no hit-index e **muda ao clique** — a classe exacta
/// que o `hit_indexed_ids_are_registered` apanha, e que ele apanhou aqui.
fn populate_paint_stack(store: &mut WidgetStore) {
    button(store, ids::VECTOR_PAINT_ADD_FILL);
    button(store, ids::VECTOR_PAINT_ADD_STROKE);
    for i in 0..ph2d_vec_scene::MAX_PAINT_LAYERS {
        button(store, ids::vector_paint_eye_id(i));
        button(store, ids::vector_paint_row_id(i));
        button(store, ids::vector_paint_up_id(i));
        button(store, ids::vector_paint_down_id(i));
        button(store, ids::vector_paint_del_id(i));
        button(store, ids::vector_paint_swatch_id(i));
        button(store, ids::vector_paint_blend_option_id(i));
    }
    // ⚠️ As opções de mistura da camada varrem o vocabulário INTEIRO, e não o tecto de camadas —
    // são duas grandezas diferentes, e o laço acima só cobre as `MAX_PAINT_LAYERS` primeiras.
    for i in 0..usize::from(ph2d_vec_scene::MAX_BLEND_MODES) {
        button(store, ids::vector_paint_blend_option_id(i));
    }
    slider_chip(
        store,
        ids::VECTOR_PAINT_OPACITY,
        ids::VECTOR_PAINT_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        ph2d_tool_vector::params::OPACITY_SLIDER_SCALE,
        ph2d_tool_vector::params::OPACITY_SLIDER_OFFSET,
    );
    // ⚠️ A faixa é a mesma do traço de base — a largura de uma camada é a MESMA grandeza, e duas
    // faixas para ela divergiriam no dia em que uma subisse.
    number_field(
        store,
        ids::VECTOR_PAINT_WIDTH,
        0.0,
        ph2d_tool_vector::params::WIDTH_MAX_PX,
        ph2d_tool_vector::params::WIDTH_MIN_PX,
        1.0,
    );
    // ⭐ ONDE a camada desenha (v21) — um par de caixas, como o `X`/`Y` do Transform, e pela MESMA
    // porta: um deslocamento é uma **coordenada de mundo**, logo **sem faixa**.
    //
    // ⛔ A 1.ª redacção emprestou a faixa da LARGURA DO TRAÇO (`±WIDTH_MAX_PX`) — um tecto de outro
    // recurso, exactamente o defeito que o §0.0 nomeia. E o recurso que se temia não existe: a
    // caixa que o deslocamento infla dimensiona o scratch do FX, que **já é limitado** pelo
    // `MAX_FX_SIDE` a jusante.
    for id in [ids::VECTOR_PAINT_DX, ids::VECTOR_PAINT_DY] {
        world_number_field(store, id, 0.0);
    }
    // ⭐ O OFFSET DE CAD (v22) — também uma distância de MUNDO, logo a mesma porta sem faixa.
    world_number_field(store, ids::VECTOR_PAINT_DILATE, 0.0);
    for id in [
        ids::VECTOR_PAINT_JOIN_MITER,
        ids::VECTOR_PAINT_JOIN_ROUND,
        ids::VECTOR_PAINT_JOIN_BEVEL,
    ] {
        button(store, id);
    }
    store.register_if_absent(
        ids::VECTOR_PAINT_BLEND,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
}
