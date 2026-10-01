//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 2 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod as_seccoes_arrastam_e_tem_tema; // o tema e o arrasto das secções de efeito do master
mod every_word_this_panel_shows_comes_from_the_string_table;
mod nenhum_nome_de_barra_corta_na_coluna;
mod seam;
mod the_mixer_asks_the_store_how_its_widgets_look;
