//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 17 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_aba_das_camadas_sem_o_painel; // onda 9b (GEF02)
mod a_barra_abre_com_a_aba_atras;
mod a_column_gives_back_exactly_what_it_took;
mod a_docked_panel_never_reaches_the_drawing_area;
mod a_fileira_de_cada_item_tem_canal;
mod a_lista_aberta_fica_por_cima;
mod a_lista_de_cartoes_flutua_no_hero;
mod a_marca_tem_a_altura_da_linha;
mod a_ordem_das_seccoes_e_a_da_paleta;
mod a_paleta_fecha_ao_escolher_por_cima_de_um_painel;
mod a_panel_paints_where_its_tab_says;
mod a_roda_e_o_transform_da_imagem;
mod a_slot_with_two_panels_shows_tabs;
mod a_tab_dragged_to_another_dock_moves_the_panel;
mod a_tinta_de_base_no_cartao;
mod cartoes_mao;
mod censo_estados;
mod colisao_segue_a_largura;
mod diag_os_comandos_do_vector;
mod f7_fotos;
mod global_palette_catalog;
mod lei_da_compactacao;
mod nenhum_botao_das_barras_de_modulo_sumiu;
mod nenhum_nome_carrega_uma_regra;
mod nenhum_nome_por_cima_do_controlo;
mod nenhum_rotulo_do_app_pinta_nada;
mod o_alcance_e_as_alcas_do_bones;
mod o_alfinete_com_corda;
mod o_alfinete_da_sprite;
mod o_alfinete_de_todo_cartao;
mod o_alfinete_do_inspector;
mod o_alfinete_dos_cartoes_c1;
mod o_alfinete_fill_stroke_c2;
mod o_bloco11_tema_por_copia;
mod o_bloco5_abertos;
mod o_bloco7_duas_copias;
mod o_bloco8_player_solto;
mod o_bloco9_reset_e_alfinete;
mod o_bloco_a_camera_e_presas;
mod o_cadeado_do_cartao_objeto;
mod o_capricho_do_inspector;
mod o_cartao_solto_bloco4;
mod o_cartao_solto_no_canvas;
mod o_gizmo_da_forma_anda;
mod o_inspector_armado;
#[cfg(feature = "panel-painter-layers")]
mod o_painter_armado;
mod o_preso_do_bones_r3;
mod o_que_o_artista_nao_alcanca;
mod o_renomear_no_cartao_preso;
mod o_scrub_dos_componentes;
mod o_smart_bone_em_cartoes;
mod o_transform_do_vector_em_draw;
mod onde_comeca_o_valor;
mod os_campos_de_texto_tem_nome;
mod os_componentes_em_cartoes;
mod os_corpos_dos_componentes_em_cartoes;
mod os_estilos_de_cor;
mod os_soltos_no_topo_e_as_cordas;
mod paineis_armados;
mod q_fotos;
mod quantas_entradas_tem_cada_painel;
mod quantas_mensagens_o_painel_escreve;
mod resetting_the_layout_puts_all_three_things_back;
mod scrub_range_census;
mod staleness;
mod switching_layout_rearranges_the_screen;
mod the_app_frame_is_reachable_by_the_hit_index;
mod the_area_hands_its_commands_to_the_bar_and_the_app_menu;
mod the_mode_selector_answers_a_real_click;
mod the_name_of_a_panel_has_one_source;
#[cfg(feature = "panel-vector")]
mod the_painter_tools_live_in_the_vector_column; // as ferramentas do pintor na coluna do vetor (23/10)
mod the_palette_reaches_the_vector_bars;
#[cfg(feature = "panel-pixel-tools")]
mod the_pixel_column_answers_a_real_pointer; // as ferramentas do Pixel na coluna da esquerda (25/10)
#[cfg(feature = "panel-pixel-export")]
mod the_pixel_export_answers_a_real_pointer;
#[cfg(feature = "panel-pixel-keys")]
mod the_pixel_keys_answers_a_real_pointer;
#[cfg(feature = "panel-pixel-tools")]
mod the_pixel_layer_user_data_answers_a_real_pointer;
#[cfg(all(feature = "panel-pixel-prefs", feature = "panel-pixel-timeline"))]
mod the_pixel_onion_range_answers_a_real_pointer;
#[cfg(feature = "panel-pixel-prefs")]
mod the_pixel_prefs_answers_a_real_pointer;
#[cfg(feature = "panel-pixel-slices")]
mod the_pixel_slices_answers_a_real_pointer;
#[cfg(feature = "panel-pixel-tools")]
mod the_pixel_sprite_properties_answers_a_real_pointer;
#[cfg(feature = "panel-pixel-tiles")]
mod the_pixel_tiles_answers_a_real_pointer;
#[cfg(feature = "panel-pixel-slices")]
mod the_pixel_windows_close_and_live_in_the_window_menu;
mod the_pixel_windows_open_clear_of_each_other;
mod the_properties_fold_into_an_icon_strip_on_small_tablets;
mod the_slices_window_widens_by_its_outer_edge;
mod the_tab_and_the_menu_call_a_panel_the_same_thing;
mod the_tool_bar_never_grows_and_the_rest_is_behind_the_dots;
mod the_tool_options_can_live_at_the_bottom;
mod the_vector_bars_answer_a_real_pointer;
mod the_vector_control_bar;
mod the_window_menu_reaches_every_module;
mod toda_dica_dos_cartoes;
mod ui_motion_frame_halves;
mod ui_motion_population_census;
