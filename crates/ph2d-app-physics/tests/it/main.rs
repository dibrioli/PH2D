//! O binário ÚNICO de teste de integração desta crate (DIRETRIZ §6.3).
//!
//! ⚠️ Um ficheiro solto em `tests/` é um binário próprio que religa a closure da
//! crate — eram 1 446 no repo até 10/09, 80 % do tempo de verificação. Teste novo
//! = módulo aqui, mais uma linha `mod`.

mod every_word_this_family_shows_comes_from_the_string_table;
mod outside_frame_has_no_production_caller;
