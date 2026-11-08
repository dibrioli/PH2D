//! Um binário de teste por crate: cada ficheiro em `tests/it/` é um MÓDULO deste binário, nunca um
//! `tests/*.rs` solto (auditoria de velocidade de 2026-09-10, onda W1 — 3× menos compilação).
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo.

mod a_aba_das_camadas; // onda 9a: a aba central «Collision Layers», pelo gesto real
mod a_sombra_do_contorno; // fecho 07/11: a sombra escura por baixo do contorno da camada no canvas
mod as_notas_e_o_contorno_vivem_aqui; // notas e contorno em qualquer painel (2026-10-01)
mod as_seccoes_arrastam_e_tem_tema; // o tema e o arrasto das secções do painel
mod every_word_this_panel_shows_comes_from_the_string_table;
mod o_contraste_da_pergunta; // onda 9f: os nomes na caixa do Remove a ≥ 4,5:1
mod os_cartoes; // o painel em cartões: todo id vivo, pelo gesto real
