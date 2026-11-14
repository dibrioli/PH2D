//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 4 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.
//! ⛔ Excepção: um ficheiro com `#[global_allocator]` tem de ser binário PRÓPRIO (dois alocadores
//! globais não cabem num binário, e um contador global veria as alocações dos vizinhos): fica em `tests/`.

mod a_borda_pela_area; // Bokeh e Halftone: a borda pela área do píxel
mod a_borracha_de_textura; // a Texture Eraser morde o grão à Strength 1 (25/10)
mod a_caneta_deitada;
mod a_cauda_das_pontas; // L5 + L18/L19: a cauda pela porta do despacho e com o fim do trilho de píxel
mod a_estrela_pequena; // o Stars pequeno sem degraus
mod a_lei_da_tinta;
mod a_luz_do_impasto_e_a_de_antes_da_poda;
mod a_mistura_opaca_nos_presets; // a mistura opaca com os presets de fábrica (smoke do dono 07/11)
mod a_mistura_pigmento_digital; // o Color mixing «Pigment mix» e o Watercolor Basic intacto (07/11)
mod a_ponta_de_n_imagens;
mod a_ponta_de_pixel;
mod as_formas;
mod as_leis_da_escala; // 03_MOTORES §7 c, d, b-resto: o relevo, a ponta das cerdas, a lente, o grão fino
mod as_sondas_dizem_o_perfil; // B44.4: perfil dito, smoke no cabeçalho, dabs do Digital (censo, N10)
mod as_sondas_dizem_o_que_medem; // B44.7: o censo único das sondas (declarada + perfil, N10)
mod as_sondas_passam_o_raio; // B44.1: set_brush_size_px toma o raio (censo, N10)
mod cerda_e_marca;
mod curas_da_mistura; // as curas da mistura Wet blend de 08/11 (Bugs #73–#78)
mod dente_do_papel_por_cima_do_grain; // o papel morde por cima do Grain do pincel (dono 08/11)
mod exame_bancada;
mod familias; // J3: as receitas da fábrica, um ficheiro por família (familias/mod.rs as lista todas)
mod leis_sistemicas;
mod macro_agua;
mod macro_colordrop;
mod macro_comum;
mod macro_em_cena;
mod macro_etapas;
mod macro_instantaneos;
mod macro_portas;
mod macro_traco;
mod macro_traco_camada;
mod macro_trilho;
mod macro_tutorial_fill;
mod measure_undo_capacity;
mod midias_de_fabrica;
mod o_custo_no_caminho_do_app; // o depósito de um pincel pesado pela rota que paga (exames/custo_no_app.md)
mod o_dente_enche; // o Tooth fill do Height: o pó de antes do traço enche o papel (dono 09/11)
mod o_dual_brush;
mod o_grao_do_lapis; // 03_MOTORES §7.2: o grão é o papel (o mesmo a todo tamanho) e o do lápis é fino
mod o_modo_do_grao;
mod o_motor_nao_se_espera_dentro_do_pool; // o congelamento de 05/11: nenhuma espera sem prazo em job do pool
mod o_pigmento_por_comprimento;
mod o_po_do_aerografo;
mod o_preset_nao_herda; // o preset escolhido depois de sujar o pincel é o preset (a família do estado escondido)
mod o_relevo_dos_fios;
mod o_spray_do_wet; // J6 I-11: as marcas do spray do Wet perto do traço, e a memória do preset com jitter alto
mod o_wash; // o Wash: o traço pára no maior dab (dono 25/10)
mod orcamento_4k; // B44.8: o traço a 4K custa o mesmo que a 1024² nos 4 meios (N10)
mod os_fios;
mod os_outros_meios_nao_mudam;
mod pinceis_da_ponta;
mod prancha_b28_3; // missão 4K B28.3: a amostra esparsa do Color mixing (prancha + relógio, ignored)
mod prancha_b29_1; // missão 4K B29.1: Flow baixo em camadas estanca (prancha do antes, ignored)
mod prancha_b29_3; // missão 4K B29.3: o grão é tecto com o filme AA (prancha, ignored)
mod prancha_b31_1; // B31.1: o aro branco do Impasto no papel, antes/depois
mod prancha_b34; // missão 4K B34.1: o papel à escala do documento (prancha, ignored)
mod presets_de_fabrica;
mod presets_dos_meios;
mod push_look_probe; // render-and-look do Push (veio do ph2d-app-painter, 10/11: só a API do PainterTool)
mod receitas_de_fabrica;
mod regua_escada;
mod regua_midia;
mod sonda_auditoria_composite_camadas; // auditoria 08/11: Wet blend × Composite × Per-Layer Color × camadas (sonda)
mod sonda_auditoria_papel_grao; // auditoria 08/11: Wet blend × papel, grão, cor do grão e do Shape (sonda)
mod sonda_cartao_pen; // auditoria do cartão Pen (24/10)
mod sonda_composite_vs_mixing; // Composite × Color mixing (pergunta do dono 07/11)
mod sonda_continuo; // o Continuous em contas (dono 25/10)
mod sonda_csp_comparacao; // os mesmos traços do Clip Studio nos nossos pincéis (pedido do dono 08/11, sonda)
mod sonda_csp_secos; // o lote B CSP (secos, aguadas, Pointillism) pela receita (08/11, sonda)
mod sonda_csp_tinta_grossa; // o lote A CSP (tinta grossa, Wet blend) pela receita (08/11, sonda)
mod sonda_csp_v3_revisao; // a cena v3 partilhada com o CSP, 22 pincéis no tamanho de fábrica (08/11, sonda)
mod sonda_custo_do_veu; // o custo do véu do Wet Wash Mix: afinado × antigo × Off (10/11, sonda)
mod sonda_custo_no_app;
mod sonda_etapas;
mod sonda_falloff_do_traco; // o falloff do traço × o de um dab, e o Accumulate do Glaze Round (24/10)
mod sonda_gestos;
mod sonda_largura;
mod sonda_macro_custo;
mod sonda_mescla_espacial; // a mescla espacial do Watercolor Basic, ingrediente a ingrediente (08/11, sonda)
mod sonda_midias;
mod sonda_pen_up_4k; // o pen-up numa tela 4K (dono 10/11, sonda)
mod sonda_secao_render; // a secção Rendering do Digital, antes e depois da retirada (24/10)
mod sonda_spacing_continuo;
mod sonda_tilt_do_preset;
mod sonda_traco_rapido;
mod sonda_tracos_por_meio;
mod sonda_watercolor_no_digital; // a mistura do Watercolor Basic no Digital? (pedido do dono 07/11)
mod textura_do_tamanho; // W3 K2: a textura acompanha o tamanho do pincel e a marca do spray pousa // W3 K3b: as cerdas crescem com o pincel e a ponta esfarela pelo grão
mod um_so_sim_hz; // B42.4: o 40 Hz da sim tem uma fonte
mod wet_brush_look_probe; // render-and-look do pincel GRANDE do Wet Paint (veio da shell, 09/11)
mod wet_grid_look_probe; // render-and-look da razão da grade do fluido (veio da shell, 09/11)
