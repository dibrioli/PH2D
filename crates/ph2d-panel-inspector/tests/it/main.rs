//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 28 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_caixa_do_contador_esta_viva;
mod a_field_is_never_narrower_than_its_owner_declared;
mod a_lista_de_tags_cabe_no_popover;
mod a_long_popover_scrolls;
mod a_seccao_counter_watch_esta_viva;
mod a_seccao_gatilho_esta_viva;
mod a_seccao_particles_esta_viva;
mod a_seccao_ray_sensor_esta_viva;
mod a_seccao_script_esta_viva;
mod a_seccao_sequence_esta_viva;
mod a_seccao_tags_esta_viva;
mod a_seccao_tween_diz_onde_mora_o_tempo;
mod a_seccao_weapon_esta_viva;
mod action_verb_is_a_dropdown;
mod as_caixas_que_encurtaram_guardam_a_explicacao;
mod as_quatro_seccoes_que_estreavam_a_catraca;
mod every_form_row_reserves_the_animation_column;
mod every_label_this_panel_paints_fits_its_column;
mod every_painted_id_is_reachable;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod inspector_regression;
mod inspector_regression_anchors;
mod inspector_regression_sections;
mod inspector_regression_slice;
mod nenhum_botao_atravessa_a_linha_a_mao;
mod nenhum_chip_do_tween_sai_cortado;
mod nenhuma_linha_pinta_o_nome_numa_coluna_propria;
mod nenhuma_seccao_declara_a_propria_altura;
mod nenhuma_seccao_pinta_o_proprio_aviso;
mod no_row_paints_its_name_above_its_control;
mod o_chip_da_vigia_mostra_um_sinal;
mod o_chip_de_uma_tag_cabe_na_pilula;
mod o_segmentado_do_fit_tem_um_id_por_modo;
mod o_tutorial_nomeia_rotulos_que_existem;
mod seam;
mod seam_anim;
mod seam_apply_ladder;
mod seam_joint;
mod seam_open_prefab;
mod seam_orphan_list;
mod seam_path_follow;
mod seam_physics;
mod seam_player;
mod seam_precision;
mod seam_properties;
mod seam_render_source;
mod seam_shake;
mod seam_texture_slot;
mod seam_tween;
mod seam_wheel;
mod the_add_component_button_follows_the_selection;
mod the_audio_section_is_alive;
mod the_camera_section_is_alive;
mod the_ordering_labels_come_from_the_descriptor;
mod the_sheet_grid_switch_is_offered_only_where_there_is_a_grid;
mod the_slice_hints_take_the_room_they_actually_use;
mod toda_seccao_viva_chega_a_pixel;
mod todo_chip_de_variante_e_pintado;
mod two_sections_never_stack;
mod uma_linha_de_marcar_ocupa_a_linha_inteira;
