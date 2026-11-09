//! Um binário de teste por crate: cada ficheiro de `tests/it/` é um MÓDULO deste binário.
//! ⚠️ Teste novo = ficheiro novo AQUI + uma linha `mod` abaixo — nunca um `tests/*.rs` solto.

mod ajustes;
mod ajustes_app;
mod alvo_todos;
mod camadas;
mod cartao;
mod catalogo;
mod cena18;
mod cena19;
mod cena25;
mod cena26;
mod comando;
mod comum;
#[allow(
    dead_code,
    reason = "arnês partilhado com ph2d-app-light2d: só a cadeia do quadro é usada aqui"
)]
#[path = "../../../../crates/ph2d-app-light2d/tests/it/comum.rs"]
mod comum_luz;
mod conta_gotas;
mod convolucao;
mod degrade;
mod desenho;
mod dinamica;
mod edit;
mod enquadrar;
mod exportar;
mod fecho;
mod filtros;
mod formas;
mod historia;
mod intervalo;
mod painel_e_dialogo;
mod palco_do_jogo;
mod paleta;
mod pincel;
mod pivo;
mod pixel_na_luz;
mod pontas;
mod preview;
mod quadros;
mod selecao;
mod slices;
mod slices_cena;
mod slices_janela;
mod sobreviventes;
mod sprite_props;
mod teclas;
mod texto;
mod texto_fontes;
mod tilemap_recusa;
mod tiles;
mod tocar;
mod user_data_donos;
mod viva;
