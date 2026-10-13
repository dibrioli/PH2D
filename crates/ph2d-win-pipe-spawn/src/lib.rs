//! W21 ([ADR-0185] §W19.8) — **lançar um processo no Windows herdando SÓ dois tubos anónimos**, e
//! dizer-lhe os valores dos dois handles (`--remote-debugging-io-pipes=<leitura>,<escrita>` no navegador
//! do quadro: um handle herdado tem o MESMO valor no filho). Nenhuma porta TCP; só o filho herda
//! (`PROC_THREAD_ATTRIBUTE_HANDLE_LIST`). O `unsafe` mora só em `spawn` (Windows); a linha de comando
//! é pura e testa-se em qualquer SO.
//!
//! [ADR-0185]: ../../../docs/architecture/decisions/0185-o-video-do-quadro-toca-no-navegador-da-pessoa-projectado-no-canvas.md

mod cmdline;
#[cfg(windows)]
mod spawn;

pub use cmdline::command_line;
#[cfg(windows)]
pub use spawn::{Child, Spawned, is_alive, spawn};

#[cfg(test)]
mod tests;
