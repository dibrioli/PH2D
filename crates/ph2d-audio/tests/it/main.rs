//! Um binário de teste por crate (auditoria de velocidade de 2026-09-10, onda W1).
//!
//! Cada ficheiro em `tests/it/` é um MÓDULO deste binário, não um binário próprio: antes eram
//! 14 binários nesta crate, cada um a religar a closure inteira de dependências (matklad,
//! «Delete Cargo Integration Tests»: 3× menos compilação, 5× menos disco). Os nomes dos testes
//! ganham o prefixo do módulo (`ficheiro::fn`); filtros por `test(nome)` continuam a casar.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod bus_routing;
mod command_queue;
mod loop_regions;
mod master_filter;
mod metering;
mod no_alloc_render;
mod no_codec_reaches_the_mixer;
mod no_ml_runtime_reaches_the_mixer;
mod preview;
mod render_silence;
mod streaming_sounds_identical;
mod the_mixer_fits_its_budget;
mod voice_playback;
mod voice_stealing;
