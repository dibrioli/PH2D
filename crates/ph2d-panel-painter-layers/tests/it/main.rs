//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 19 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

/// O painel de antes: estes testes descrevem o corpo de `PH2D_CARDS=0` (`docs/CartoesUI/13_PAINTER_PLANO.md`).
pub(crate) fn host_de_antes() -> ph2d_ui_testkit::MockPanelHost {
    ph2d_panel_painter_layers::cards::force_cards_on_this_thread(Some(false));
    ph2d_ui_testkit::MockPanelHost::with_panel::<ph2d_panel_painter_layers::PainterLayersPanel>()
}

mod a_pilha_de_ajustes_fala_a_tabela_e_mostra_o_numero;
mod as_seccoes_arrastam_e_tem_tema; // o tema e o arrasto das secções do corpo do pincel
mod auditoria_l2_continuous; // AUDITORIA lente 2 (15/10): o Continuous pelo ponteiro — sonda, não portão
mod bristle_combing; // os fios que penteiam, no cartão Shape (dono 07/11)
mod cada_nome_deste_painel_cabe_na_coluna_da_seccao;
mod censo_dos_controlos; // cada controlo pintado, em cada meio, muda o traço (ou diz porquê)
mod censo_ferramenta_x_linha; // a ferramenta do trilho × cada linha do painel (23/10)
mod color_mixing; // a mistura opaca do Digital, no cartão Basics, nos dois painéis
mod curve_handle_menu_e2e;
mod every_word_this_panel_shows_comes_from_the_string_table;
mod f3_preset_voa_e_nada_cortado; // o voo da linha de preset e a fila On switch
mod falloff_drain_repro;
mod falloff_handle_menu_e2e;
mod h10_fotos; // os estados que a cena do app não semeia
mod h10_os_cartoes; // o fim da H9: os widgets próprios no corpo do cartão, sem a captura
mod h11_fotos; // as fotos e o custo por quadro do fecho
mod h11_os_cartoes; // o fecho: ajustes e camada Texture em linhas de cartão
mod h12_preset_estrela; // a ordem do topo e a estrela do favorito (dono 04/11)
mod h1_contraprovas; // os mutantes da captura que a H1 deixava passar
mod h1_os_cartoes; // o painel em cartões: a lista, a captura e os corpos nativos (04/11)
mod h2_os_cartoes; // o topo e os básicos em linhas de cartão
mod h3_os_cartoes;
mod h4_os_cartoes; // o cartão Wet Paint em linhas de cartão
mod h5_os_cartoes; // Shape e Shape Color em linhas de cartão, e as dívidas de H3 e H4
mod h6_os_cartoes; // Paper, Grain e Randomize Color em linhas de cartão
mod h7_os_cartoes; // Stroke e Pen em linhas de cartão
mod h8_os_cartoes; // Composite, Line, Clone, Inpaint, Mask, Deform, Selection, Sculpt
mod h9_os_cartoes; // a vista Layers
mod macro_estado_inicial; // o estado inicial de uma macro leva cada controlo do Wet Paint
mod o_censo_so_encolhe; // o que muda o ajuste e não a tinta: esmaecido ou com motivo na lista
mod o_flow_a_vista_e_o_flow_que_age; // o Flow à vista ⟺ a ferramenta no meio dela o lê (24/10)
mod o_painel_de_cada_meio_da_ferramenta; // o meio é da ferramenta na mão: só as linhas dela (24/10)
mod o_painel_mostra_o_que_a_ferramenta_le; // a ferramenta na mão × o painel: os dois lados (23/10)
mod o_seletor_fechado_nao_fica_fixado; // fechar o seletor de cor desafixa-o, por qualquer porta (24/10)
mod pontas_e_fio_pelo_rato; // Fine Thread, Flat ends, Ends beyond, Ends fade pelo clique e arrasto reais (J6)
mod seam;
mod seam_accumulate_por_meio; // o Accumulate e o Space Attenuation só onde o meio os oferece
mod seam_color_inativo; // Bug #51: o Color inativo em Per-Layer Color (12/10)
mod seam_composite_escopo; // o chip do escopo da borracha: pintado, registado, alcançável
mod seam_composite_montagem; // o `+`, o menu e o `x` da pilha: pintados, registados, alcançáveis
mod seam_curve_drag_ownership;
mod seam_deform;
mod seam_dock_modes;
mod seam_estado_escondido; // o interruptor que neutraliza: a tinta depois dele é a do pincel limpo (22/10)
mod seam_flow; // o Flow (a taxa) abaixo da Strength (o tecto), onde a ferramenta e o meio o lêem (24/10)
mod seam_grid_stamp;
mod seam_hardness; // a Hardness abaixo do Falloff, digitada pelo ponteiro (24/10)
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
mod seam_wetpaint_oleo_real;
mod seam_wetpaint_opcoes;
mod size_px_e_dab_fixo; // dono 08/11: Size em px e Fixed dab size em todo meio // W1g: Live Water e Heavy Edge pelo clique real // Óleo Real (doc 49): a caixa e as cinco linhas pelo clique real
