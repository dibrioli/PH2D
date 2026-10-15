//! **Uma família que grava** (W5d): o painel MACROS arma-a, a sessão drena-a a cada quadro com o tempo
//! do chamador, e pára-a. O núcleo não sabe o que cada família grava — só a ordem das três portas.

use crate::recorder::{MacroRecorder, TrackId};

pub trait Gravadora {
    /// Arma a gravação e abre a trilha em `rec`; a família escolhe o alvo (`StableId`; `0` = a entidade
    /// da própria `Macro`).
    ///
    /// # Errors
    /// A chave da frase que diz ao artista porque não (`ph2d_i18n::tr`).
    fn abre(&mut self, rec: &mut MacroRecorder) -> Result<TrackId, &'static str>;
    /// Os passos desde a última chamada, codificados.
    fn passos(&mut self) -> Vec<Vec<u8>>;
    /// Pára e devolve os últimos passos.
    fn para(&mut self) -> Vec<Vec<u8>>;
}
