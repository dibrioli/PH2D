//! Um binário de teste por crate: cada ficheiro de `tests/it/` é um MÓDULO deste binário.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod comp;
mod jogo_camara;
mod jogo_moldura;
mod jogo_moldura_ponteiro;
mod jogo_object_add;
mod jogo_smoke;
mod sprite_image_choice_gate;
mod sprite_image_use_gate;
// Carregado por `#[path]` para os dois filhos (`#[path]` também) se resolverem nesta pasta.
#[path = "sprite_image_use_15.rs"]
mod sprite_image_use_15;
mod sprite_region_pela_aba;
mod sprite_tiled;
