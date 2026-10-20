//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 44 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_opacidade_da_barra_le_a_forma;
mod architecture_sections_read_the_document;
mod as_abas_do_titulo; // F17: Drawing / Interface, o interruptor da ① e o grupo Advanced
mod as_notas_vivem_nas_seccoes; // notas em qualquer painel (2026-10-01)
mod as_seccoes_arrastam_e_tem_tema;
mod bool_registration_parity;
mod connector_section;
mod every_chip_is_linked_to_its_slider;
mod every_name_the_engine_publishes_has_a_key;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod fill_chip_labels_fit;
mod marker_rows;
mod popover_band;
mod probe_gutters;
mod seam;
mod seam_anchors;
mod seam_bool;
mod seam_bucket;
mod seam_components;
mod seam_contour;
mod seam_filters;
mod seam_frame;
mod seam_key_mark;
mod seam_layout;
mod seam_layout_grid_item;
mod seam_marquee;
mod seam_morph_states;
mod seam_node_xy;
mod seam_paint_stack;
mod seam_pencil_knobs;
mod seam_resize_box;
mod seam_signals;
mod seam_spring;
mod seam_stroke_paint;
mod seam_stroke_present;
mod seam_symmetry_segments;
mod seam_text_axes;
mod seam_text_wrap;
mod seam_text_wrap_on_path;
mod seam_texture_pattern;
mod seam_tokens;
mod seam_ui_states;
mod seam_vertex;
mod seam_weld;
mod seam_widget;
mod section_headers_are_collapsible;
mod shape_params_visibility;
mod the_bars_carry_the_gesture;
mod the_panel_shows_what_the_tool_is_for;

/// ⭐ **O host dos testes de costura, com TODAS as secções ABERTAS.**
///
/// ⚠️ D10 (07/10): Contêiner, Interação e Avançado nascem recolhidos no `populate`. Os testes de
/// costura provam a FIAÇÃO de cada controlo (pintado ⇒ registado ⇒ despachado), não a dobra por
/// omissão — essa tem gate próprio (`bars_and_folds::the_container_interaction_and_advanced_start_folded`).
pub fn host_aberto() -> ph2d_ui_testkit::MockPanelHost {
    let mut h = ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_vector::VectorPanel>();
    h.open_all_sections();
    h
}

/// ⭐ **O mesmo host, com a aba «Interface» aberta pelo CLIQUE na aba** (F17): as secções de UI
/// (componente, moldura, layout, âncoras, pele, estados) só pintam lá. ⚠️ A aba é estado da
/// thread do teste: os hosts seguintes do mesmo teste continuam na Interface.
pub fn host_na_interface() -> ph2d_ui_testkit::MockPanelHost {
    let mut h = host_aberto();
    abre_aba(&mut h, ph2d_panel_vector::ids::VECTOR_TAB_INTERFACE);
    h
}

/// Abre a aba `tab` pelo mesmo evento que o ponteiro produz sobre ela.
pub fn abre_aba(h: &mut ph2d_ui_testkit::MockPanelHost, tab: ph2d_a11y::NodeId) {
    let mut st = ph2d_panel_vector::state::VectorPanelState;
    h.apply_panel_event::<ph2d_panel_vector::VectorPanel>(
        &mut st,
        ph2d_editor_core::interaction::WidgetEvent::Click(tab),
    );
}
