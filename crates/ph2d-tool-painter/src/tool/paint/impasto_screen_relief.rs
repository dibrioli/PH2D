//! ⭐⭐⭐ **O CORPO e a SEMENTE do relevo na tela da vista 3D** (`docs/3D/29`
//! §6) — irmão (`#[path]`) do `impasto_live.rs`, que é a ESPESSURA da janela.
//!
//! # As duas portas, e o report que cada uma cura (dono, 01/10)
//!
//! - **O CORPO** ([`PainterTool::layer_cover_in`]) — *«o traço tem um relevo
//!   indesejado na borda»*. O impasto assenta a espessura com um alisamento e
//!   ela ESPALHA-SE para fora da tinta; aqui a luz pesa-a pela cobertura
//!   (`impasto_light::paint_body`, a cura do halo de 2026-07-12), e a peça
//!   recebia a espessura SEM a cobertura. A janela leva os DOIS, pela mesma
//!   composição do `impasto_fields::cover_at` de uma camada só.
//! - **A SEMENTE** ([`PainterTool::seed_screen_canvas_relief`]) — *«smooth,
//!   knife e outras tools não funcionam no relevo»*. Cada pincelada começa por
//!   semear a tela com o RETRATO da cor, e a semente passa pelo `set_source`, que
//!   APAGA o relevo das camadas: o alisar e a faca liam uma tela lisa. Medido:
//!   sem semente o `Smooth` não devolve um quadro e a faca devolve `21` sem
//!   espessura; com ela os dois mudam o relevo (`9,7` e `2,8` px).

use std::sync::Arc;

use crate::tool::{PainterTool, RtLayerId};

impl PainterTool {
    /// ⭐⭐ **A COBERTURA de `id` numa JANELA, `0..1`** — o corpo da tinta que a
    /// tela da vista leva à peça ao lado da [`Self::layer_height_px_in`].
    ///
    /// É o `cover_at` do passe de luz numa camada só: o MÁXIMO do comprometido e
    /// do filme do traço aberto (a cobertura é uma PRESENÇA, não uma quantidade),
    /// com o traço aberto pelas MESMAS cercas da espessura (camada activa, sem
    /// borracha). `None` quando a camada não tem cobertura nenhuma.
    #[must_use]
    pub fn layer_cover_in(
        &self,
        id: RtLayerId,
        (x, y, jw, jh): (u32, u32, u32, u32),
    ) -> Option<Vec<f32>> {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let committed = self.covers.get(&id).filter(|c| c.len() == n);
        let live = (self.paint.relief.stroke_film.len() == n
            && self.layers.active() == Some(id)
            && !self.paint.eraser)
            .then_some(&self.paint.relief.stroke_film);
        if committed.is_none() && live.is_none() {
            return None;
        }
        let (x, y) = (x.min(w), y.min(h));
        let (jw, jh) = (jw.min(w - x), jh.min(h - y));
        let mut out = Vec::with_capacity((jw as usize) * (jh as usize));
        for j in y..y + jh {
            for i in x..x + jw {
                let k = (j as usize) * (w as usize) + i as usize;
                let c = committed.map_or(0, |c| c[k]);
                let l = live.map_or(0, |l| l[k]);
                out.push(f32::from(c.max(l)) / 255.0);
            }
        }
        Some(out)
    }

    /// ⭐⭐⭐ **Semeia o RELEVO da peça na tela da vista** — a espessura em
    /// PÍXEIS (a unidade da [`Self::layer_height_px_in`]) e a cobertura em
    /// bytes, por píxel da tela, para a camada activa.
    ///
    /// ⚠️ **Chama-se DEPOIS da semente da cor** ([`Self::seed_screen_canvas`]):
    /// aquela passa pelo `set_source`, que limpa o relevo das camadas.
    ///
    /// ⚠️ **A conversão é a inversa EXACTA da janela** — divide pela
    /// `impasto_depth` da camada e pelo `DEPTH_UNIT_PX` —, e quem guarda a
    /// semente do lado da peça LÊ-A DE VOLTA pela janela em vez de reusar o que
    /// mandou: um ULP de ida-e-volta seria uma diferença em toda amostra que o
    /// pincel não tocou.
    ///
    /// Devolve `false` — e nada muda — fora da tela da vista, com tamanhos que
    /// não são os da tela, sem camada activa, ou com profundidade nula (que não
    /// tem inversa).
    pub fn seed_screen_canvas_relief(&mut self, px: &[f32], cover: &[u8]) -> bool {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if !self.on_screen_canvas() || px.len() != n || cover.len() != n {
            return false;
        }
        let Some(id) = self.layers.active() else {
            return false;
        };
        let escala = self.layers.get(id).map_or(1.0, |l| l.impasto_depth)
            * super::super::impasto_light::DEPTH_UNIT_PX;
        if !escala.is_finite() || escala == 0.0 {
            return false;
        }
        self.drop_live_relief();
        self.heights
            .insert(id, Arc::new(px.iter().map(|&v| v / escala).collect()));
        self.covers.insert(id, Arc::new(cover.to_vec()));
        self.sync_relief_flags();
        true
    }
}
