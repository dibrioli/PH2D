//! Um binário de teste por crate: cada ficheiro de `tests/it/` é um MÓDULO deste binário.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod a_lista_de_sons;
mod comp;
mod jogo_arte;
mod jogo_camara;
mod jogo_camera_intuitiva;
mod jogo_encaixe;
mod jogo_miniatura;
mod jogo_moedas;
mod jogo_moldura;
mod jogo_moldura_lei;
mod jogo_moldura_ponteiro;
mod jogo_object_add;
mod jogo_smoke;
mod plataforma_alargada;
mod receitas;
mod sound_set;
mod sprite_espelhar;
mod sprite_image_choice_gate;
mod sprite_image_use_gate;
mod sprite_marcadores;
// Carregado por `#[path]` para os dois filhos (`#[path]` também) se resolverem nesta pasta.
mod sprite_card_edits;
mod sprite_corte_velho;
#[path = "sprite_image_use_15.rs"]
mod sprite_image_use_15;
mod sprite_region_pela_aba;
mod sprite_regra_acesa;
#[path = "sprite_starts_with.rs"]
mod sprite_starts_with;
mod sprite_tick_sinais;
mod sprite_tiled;
mod sprite_transport;
mod sprite_vazio_e_marcador;
