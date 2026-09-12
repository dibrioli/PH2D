//! O binário ÚNICO de teste de integração desta crate (DIRETRIZ §6.3).
//!
//! ⚠️ Um ficheiro solto em `tests/` é um binário próprio que religa a closure da crate —
//! eram 1 446 no repo até 10/09, 80 % do tempo de verificação. Teste novo = módulo aqui,
//! mais uma linha `mod`.
//!
//! ⭐ **O que vem para cá, e o que fica na shell** (HOWTO §2.6): um gate que mede a **lei da
//! família** viaja com ela; um que mede *que a SHELL chama a família* fica lá, apontando para
//! fora; e um que mede a `App`/`ProjectState`/o arnês do ponteiro nunca se mexe.

mod the_eraser_uses_the_erasers_own_numbers;
mod the_flip_pass_asks_before_it_rasterises;
mod the_shell_turns_on_the_features_this_family_reads;
