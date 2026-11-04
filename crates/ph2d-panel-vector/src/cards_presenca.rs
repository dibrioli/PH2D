//! ⭐ **Que cartões estão à vista, com que título e que «+»** (fatia G8). A tabela é a mesma pergunta que
//! cada pintor de `PH2D_CARDS=0` faz antes do cabeçalho, sem correr pintor nenhum. O gate
//! `g8_os_cartoes::presence_answers_like_the_old_panel` compara os dois modos estado a estado.

use crate::paint_body_plan::corpo;
use crate::paint_connector::current_connector;
use crate::paint_sections::AcaoDoCabecalho;
use crate::paint_sections::fx_list::{Pilha, is_open};
use crate::state::{self, contour as cst, filters as fst};
use crate::{ids, ids as p};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_i18n::tr;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::ids as t;

/// Uma entrada de topo à vista neste quadro.
pub(crate) struct Presente {
    pub(crate) id: NodeId,
    pub(crate) titulo: &'static str,
    pub(crate) acao: Option<AcaoDoCabecalho>,
}

/// O cartão `id` está à vista? Com que chave de título?
fn titulo_de(id: NodeId) -> Option<&'static str> {
    let (visivel, chave) = if id == p::VECTOR_SECTION_COMPONENT {
        (
            state::component_state().is_some(),
            "panel.vector.section.component",
        )
    } else if id == t::VECTOR_SECTION_TRANSFORM {
        (
            state::current_transform().is_some(),
            "panel.vector.section.transform",
        )
    } else if id == p::VECTOR_SECTION_ANCHORS {
        (
            state::anchor_state().is_some(),
            "panel.vector.section.anchors",
        )
    } else if id == t::VECTOR_SECTION_SNAP {
        (true, "panel.vector.section.snap")
    } else if id == t::VECTOR_SECTION_FRAME {
        (state::frame_present(), "panel.vector.section.frame")
    } else if id == t::VECTOR_SECTION_CLIP {
        (state::frame_clip().is_some(), "panel.vector.section.clip")
    } else if id == p::VECTOR_SECTION_LAYOUT {
        (
            state::frame_clip().is_some() || state::layout_item().is_some(),
            "panel.vector.section.layout",
        )
    } else if id == p::VECTOR_SECTION_APPEARANCE {
        (
            state::current_appearance().is_some(),
            "panel.vector.section.appearance",
        )
    } else if id == t::VECTOR_SECTION_FILL {
        (true, "panel.vector.section.fill_stroke")
    } else if id == p::VECTOR_SECTION_TEXPAT {
        (
            state::current_texture_pattern(0).is_some(),
            "panel.vector.section.texpat",
        )
    } else if id == p::VECTOR_SECTION_TEXPAT_STROKE {
        (
            state::current_texture_pattern(1).is_some(),
            "panel.vector.section.texpat_stroke",
        )
    } else if id == p::VECTOR_SECTION_BRUSH {
        (
            state::current_brush().is_some(),
            "panel.vector.section.brush",
        )
    } else if id == t::VECTOR_SECTION_CONNECTOR {
        (
            current_connector().is_some(),
            "panel.vector.section.connector",
        )
    } else if id == t::VECTOR_SECTION_TEXT {
        (state::text_visible(), "panel.vector.section.text")
    } else if id == t::VECTOR_SECTION_FONT {
        (state::text_visible(), "panel.vector.section.font")
    } else if id == t::VECTOR_SECTION_PARAGRAPH {
        (state::text_visible(), "panel.vector.section.paragraph")
    } else if id == p::VECTOR_SECTION_TEXTPATH {
        (state::text_visible(), "panel.vector.section.textpath")
    } else if id == t::VECTOR_SECTION_AXES {
        (
            state::text_visible() && state::with_text_axes(|a| !a.is_empty()),
            "panel.vector.section.axes",
        )
    } else if id == t::VECTOR_SECTION_VERTEX {
        (
            state::current_vertex_type().is_some(),
            "panel.vector.section.vertex",
        )
    } else if id == t::VECTOR_SECTION_EFFECTS {
        (state::has_target(), "panel.vector.section.effects")
    } else if id == p::VECTOR_SECTION_FILTERS {
        (
            !fst::stack().is_empty() || fst::can_add(),
            "panel.vector.section.filters",
        )
    } else if id == p::VECTOR_SECTION_ADVANCED {
        (true, "panel.vector.section.advanced")
    } else if id == p::VECTOR_SECTION_WIDGET {
        (
            state::widget_skin_state().is_some(),
            "panel.vector.section.widget",
        )
    } else if id == p::VECTOR_SECTION_STATES {
        (
            state::ui_states_state().is_some(),
            "panel.vector.section.states",
        )
    } else if id == p::VECTOR_SECTION_MORPH_STATES {
        (
            state::morph_states_state().is_some(),
            "panel.vector.section.morph_states",
        )
    } else {
        return None;
    };
    visivel.then_some(chave)
}

/// O «+» do cabeçalho das listas do Figma (F43/F44): `(id, ícone, aceso, vivo)`, como o de antes.
fn acao_de(id: NodeId, store: &WidgetStore) -> Option<AcaoDoCabecalho> {
    let (menu, vivo) = if id == t::VECTOR_SECTION_FILL {
        (
            ids::VECTOR_PAINT_ADD_MENU,
            state::current_appearance().is_some() || state::stroke_present().is_some(),
        )
    } else if id == t::VECTOR_SECTION_EFFECTS {
        (
            Pilha::Efeitos.add_menu(),
            !state::bound() && state::stack().len() < t::MAX_FX_ROWS,
        )
    } else if id == p::VECTOR_SECTION_FILTERS {
        (
            Pilha::Filtros.add_menu(),
            fst::can_add() && fst::stack().len() < ids::MAX_FILTER_ROWS,
        )
    } else {
        return None;
    };
    Some((menu, IconId::Plus, vivo && is_open(store, menu), vivo))
}

/// Os cartões de topo deste quadro, pela ordem natural: a aba e a seleção filtram, a tabela responde.
pub(crate) fn presentes(store: &WidgetStore, snap: &VectorStyleSnapshot) -> Vec<Presente> {
    corpo(snap)
        .de_topo_a_vista()
        .into_iter()
        .filter_map(|id| {
            Some(Presente {
                id,
                titulo: tr(titulo_de(id)?),
                acao: acao_de(id, store),
            })
        })
        .collect()
}

/// Contour: há um, ou a seleção permite criá-lo.
pub(crate) fn contorno_a_vista() -> bool {
    cst::present() || cst::can_add()
}

/// Pattern on Path: vinculado, ou a seleção permite vincular ou escolher a guia.
pub(crate) fn padrao_em_caminho_a_vista() -> bool {
    state::pp_linked() || state::pp_can_link() || state::pp_can_pick()
}
