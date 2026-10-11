//! **A família da luz 2D** — a composição no quadro do editor (plano
//! [`docs/Luz2D/01_plano.md`](https://github.com/dibrioli/PH2D/blob/main/docs/Luz2D/01_plano.md)
//! §2.1, §2.4). A shell guarda UMA [`LuzDoQuadro`] e chama três portas:
//!
//! 1. [`LuzDoQuadro::preparar`] — antes de codificar o documento: coleta, GUARDA e grade. `false` =
//!    a cena não tem `Luz`, nada mais corre e o quadro é o de sempre, byte a byte (G6). Logo a
//!    seguir, [`LuzDoQuadro::desenhar_vetores`]: os caminhos que bloqueiam, na grade (onde a cena
//!    vetorial e os afins do quadro existem).
//! 2. [`LuzDoQuadro::gravar`] — antes do passe de sprite: o solver, encoder e submit próprios.
//! 3. [`LuzDoQuadro::ligar`] — acende (ou apaga) os dois receptores: o passe de sprite e a colagem
//!    do vetor.

mod captura;
pub mod gizmo;
mod grade;
pub mod inspector;
pub mod object_add;
mod quadro;
pub mod qualidade;
pub mod smoke;
mod vetor;

/// ⭐ W5d: a captura dos sprites do produto — para quem mede a luz pelo caminho dele.
pub use captura::sprites_da_cena;
pub use grade::{
    DEGRAU_PADRAO, Degrau, ENTORNO_PADRAO, MARGEM_DO_ENTORNO, captura_do_quadro, fina_do_quadro,
    grade_do_quadro, regiao_da_janela, tela_para_mundo,
};
pub use quadro::LuzDoQuadro;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "light2d",
    routers: smoke::ROUTERS,
};
