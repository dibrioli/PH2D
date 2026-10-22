//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 19 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_pilha_de_ajustes_fala_a_tabela_e_mostra_o_numero;
mod as_seccoes_arrastam_e_tem_tema; // o tema e o arrasto das secções do corpo do pincel
mod auditoria_l2_continuous; // AUDITORIA lente 2 (15/10): o Continuous pelo ponteiro — sonda, não portão
mod cada_nome_deste_painel_cabe_na_coluna_da_seccao;
mod censo_dos_controlos; // cada controlo pintado, em cada meio, muda o traço (ou diz porquê)
mod curve_handle_menu_e2e;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod falloff_drain_repro;
mod falloff_handle_menu_e2e;
mod macro_estado_inicial; // o estado inicial de uma macro leva cada controlo do Wet Paint
mod o_censo_so_encolhe; // o que muda o ajuste e não a tinta: esmaecido ou com motivo na lista
mod seam;
mod seam_accumulate_por_meio; // o Accumulate e o Space Attenuation só onde o meio os oferece
mod seam_color_inativo; // Bug #51: o Color inativo em Per-Layer Color (12/10)
mod seam_composite_escopo; // o chip do escopo da borracha: pintado, registado, alcançável
mod seam_composite_montagem; // o `+`, o menu e o `x` da pilha: pintados, registados, alcançáveis
mod seam_curve_drag_ownership;
mod seam_deform;
mod seam_dock_modes;
mod seam_grid_stamp;
mod seam_impasto_rig;
mod seam_impasto_tool;
mod seam_line_card;
mod seam_marca; // L4: as linhas da marca no cartão Jitter, pelo ponteiro
mod seam_paint_media;
mod seam_papel; // o seletor da cor do papel aplica o papel nos quatro meios (2026-10-06)
mod seam_pen; // L2: o cartão Pen (a mão do pincel) nos quatro meios, pelo ponteiro
mod seam_pigmento; // a fileira `Pigment`: a tela concorda com a porta, nos dois sentidos
mod seam_pino_da_preview; // o pino que fixa a preview viva: no ecrã, pelo ponteiro, para quem não vê
mod seam_ponta; // L9/L18: a ponta de píxel e a lei do grão pelo clique real
mod seam_same_as_paper; // «Same as Paper» esconde o grão do pincel e a tinta concorda (22/10)
mod seam_sculpt;
mod seam_shape_deposit;
mod seam_spray;
mod seam_substrate;
mod seam_taper;
mod seam_texture_colors;
mod seam_traco; // L5/L7/L8: a cauda, o fall-off, o modo sobre o vazio e o traço como camada, pelo ponteiro
mod seam_watercolor_cards;
mod seam_watercolor_flow; // as linhas da forma da borda: Flow, Size, Angle, Paper Edge (BUGS #31)
mod seam_wet_presets;
mod seam_wetpaint; // W1: o navegador de presets e os volumes do Wet Painter, pelo ponteiro
mod seam_wetpaint_opcoes; // W1g: Live Water e Heavy Edge pelo clique real
