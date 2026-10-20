//! **A água depois do fim do clipe** (W14, `docs/Macros/12_W14_a_agua_depois_do_parar.md`; decisão do dono
//! 20/10): com a cabeça no fim do clipe, a água da gravação continua no palco, ao relógio do quadro, até à
//! imagem do artista — assentada, ou parada onde ela a guardou
//! ([`ph2d_tool_painter::macro_track::marca::MarcaDoArtista::agua_da_gravacao_parou`]). O clipe não fica
//! mais comprido.

use super::PalcosDoPainter;

impl PalcosDoPainter {
    /// **O tempo deste quadro** (ms do relógio de parede), antes de [`Self::sobe`]: o que a cauda anda.
    pub fn dt_do_quadro(&mut self, dt_ms: f32) {
        self.dt_da_cauda_ms = dt_ms;
    }
}
