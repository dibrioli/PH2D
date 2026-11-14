//! A porta das SONDAS para o tamanho do pincel (B44.1, missão 4K N10): põe o raio e imprime o
//! `brush_size_px()` LIDO, uma vez por par (pedido, lido) — o corte em `raio_maximo` deixava a sonda
//! rotular um raio que o pincel não tinha. Portão: `as_sondas_passam_o_raio_ao_set_brush_size_px`.

use std::collections::BTreeSet;
use std::sync::Mutex;

use crate::tool::PainterTool;

static VISTOS: Mutex<BTreeSet<(u32, u32)>> = Mutex::new(BTreeSet::new());

impl PainterTool {
    /// `set_brush_size_px(raio)` + imprime o raio lido; devolve-o (o que a coluna da sonda deve rotular).
    #[doc(hidden)]
    pub fn sonda_raio(&mut self, raio: f32) -> f32 {
        self.set_brush_size_px(raio);
        let lido = self.brush_size_px();
        let novo = VISTOS
            .lock()
            .map(|mut v| v.insert((raio.to_bits(), lido.to_bits())))
            .unwrap_or(true);
        if novo {
            let corte = if lido == raio { "" } else { " ⚠ CORTADO" };
            eprintln!("[sonda] raio pedido {raio} → brush_size_px() = {lido}{corte}");
        }
        lido
    }
}
