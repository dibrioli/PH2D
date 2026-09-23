//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 17 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_column_gives_back_exactly_what_it_took;
mod a_docked_panel_never_reaches_the_drawing_area;
mod a_marca_tem_a_altura_da_linha;
mod a_paleta_de_pinceis_fecha_ao_escolher;
mod a_panel_paints_where_its_tab_says;
mod a_slot_with_two_panels_shows_tabs;
mod a_tab_dragged_to_another_dock_moves_the_panel;
mod global_palette_catalog;
mod nenhum_nome_carrega_uma_regra;
mod nenhum_rotulo_do_app_pinta_nada;
mod o_inspector_armado;
mod o_model3d_armado;
#[cfg(feature = "panel-painter-layers")]
mod o_painter_armado;
mod o_que_o_artista_nao_alcanca;
mod o_sculpt3d_armado;
mod os_campos_de_texto_tem_nome;
mod paineis_armados;
mod quantas_entradas_tem_cada_painel;
mod quantas_mensagens_o_painel_escreve;
mod resetting_the_layout_puts_all_three_things_back;
mod scrub_range_census;
mod staleness;
mod switching_layout_rearranges_the_screen;
mod the_app_frame_is_reachable_by_the_hit_index;
mod the_area_hands_its_commands_to_the_bar_and_the_app_menu;
mod the_name_of_a_panel_has_one_source;
mod the_tab_and_the_menu_call_a_panel_the_same_thing;
mod the_tool_bar_never_grows_and_the_rest_is_behind_the_dots;
mod the_window_menu_reaches_every_module;
mod ui_motion_frame_halves;
mod ui_motion_population_census;
