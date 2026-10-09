//! **A família da luz 2D** — a composição no quadro do editor (plano
//! [`docs/Luz2D/01_plano.md`](https://github.com/dibrioli/PH2D/blob/main/docs/Luz2D/01_plano.md)
//! §2.1, §2.4). A shell guarda UMA [`LuzDoQuadro`] e chama três portas:
//!
//! 1. [`LuzDoQuadro::preparar`] — antes de codificar o documento: coleta, GUARDA e grade. `false` =
//!    a cena não tem `Luz`, nada mais corre e o quadro é o de sempre, byte a byte (G6).
//! 2. [`LuzDoQuadro::gravar`] — antes do passe de sprite: o solver, encoder e submit próprios.
//! 3. [`LuzDoQuadro::ligar`] — acende (ou apaga) os dois receptores: o passe de sprite e a colagem
//!    do vetor.

mod captura;
mod grade;
mod quadro;

pub use grade::{DEGRAU_PADRAO, Degrau, grade_do_quadro, tela_para_mundo};
pub use quadro::LuzDoQuadro;

/// A declaração da família à shell. ⚠️ Sem roteador de smoke ainda: as cenas
/// `PH2D_LIGHT2D_SMOKE=1..4` são da W4 (o artista).
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "light2d",
    routers: &[],
};
