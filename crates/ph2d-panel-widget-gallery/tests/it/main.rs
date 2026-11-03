//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 2 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod as_seccoes_arrastam_e_tem_tema; // o tema e o arrasto das secções da galeria
mod cartoes; // a galeria em cartões no encaixe da direita (doc CartoesUI/08)
mod cartoes_reais; // os casos reais e as amostras do Rive (doc CartoesUI/08 §1.3 e §1.4)
mod every_word_this_panel_shows_comes_from_the_string_table;
mod seam;
mod widget_gallery_paint;
