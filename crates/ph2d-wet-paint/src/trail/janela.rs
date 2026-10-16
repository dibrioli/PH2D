//! **A janela do rasto cresce NO LUGAR** — filho de [`super`] (LOC cap).
//!
//! Smoke do dono (2026-10-16): «parecem dois dabs de cada vez, mas o segundo cortado». A janela ancora no
//! 1.º dab e só era medida ali; um dab seguinte que passasse de `âncora ± half` (a corda do pincel, o raio
//! que cresce com a caneta, o jitter, os sub-dabs da cápsula) molhava a célula e PERDIA o pigmento numa
//! recta vertical. Aqui a janela cresce com a âncora fixa e o conteúdo recentrado: a tinta acumulada, o
//! bico sujo e a tinta por célula ficam onde estavam em relação à âncora — nada se perde, e uma janela
//! maior não muda a lei de nenhuma célula (cada texel é independente), só deixa de cortar. Gate:
//! `tests/it/the_window_holds_the_whole_dab.rs`.

use super::{Dab, Trail};

/// Um plano `s0 × s0` recentrado num `s1 × s1` (`d = (s1 − s0) / 2` de margem), o resto em `fill`. Um plano
/// não alocado (a tinta por célula antes do 1.º dab com cor) fica como está.
fn recentra(v: &mut Vec<f32>, s0: usize, s1: usize, fill: f32) {
    if v.len() != s0 * s0 {
        return;
    }
    let d = (s1 - s0) / 2;
    let mut n = vec![fill; s1 * s1];
    for (ly, linha) in v.chunks_exact(s0).enumerate() {
        let o = (ly + d) * s1 + d;
        n[o..o + s0].copy_from_slice(linha);
    }
    *v = n;
}

impl Trail {
    /// Cresce a janela até à meia-largura `half` (no-op se já cabe), com a âncora FIXA: cada plano é
    /// recentrado — o pigmento, a água e a máscara com zero à volta, o bico com a cor base e a fração limpa
    /// com `1` (o que um texel novo seria depois do `start_stroke`).
    pub(super) fn crescer(&mut self, half: i32) {
        if half <= self.half {
            return;
        }
        let (s0, s1) = (self.size as usize, (half * 2 + 1) as usize);
        let base = [self.base_r as f32, self.base_g as f32, self.base_b as f32];
        for (plano, fill) in [
            (&mut self.pig, 0.0),
            (&mut self.water, 0.0),
            (&mut self.mask, 0.0),
            (&mut self.tip_r, base[0]),
            (&mut self.tip_g, base[1]),
            (&mut self.tip_b, base[2]),
            (&mut self.tinta_r, 0.0),
            (&mut self.tinta_g, 0.0),
            (&mut self.tinta_b, 0.0),
            (&mut self.limpo, 1.0),
        ] {
            recentra(plano, s0, s1, fill);
        }
        let d = half - self.half;
        if self.lx1 >= self.lx0 {
            self.lx0 += d;
            self.ly0 += d;
            self.lx1 += d;
            self.ly1 += d;
        }
        self.half = half;
        self.size = half * 2 + 1;
    }

    /// **A rede**: o dab que vai pousar cabe na janela? As caixas dos dois caminhantes são
    /// `⌈c − r⌉ ..= ⌊c + r⌋` (`brush.rs`), e um texel local é `x − âncora + half ∈ [0, size)`.
    pub(super) fn ensure_fits(&mut self, dab: &Dab) {
        let r = dab.r * dab.reach.max(1.0);
        let lado = |c: f64, a: i32| {
            let (lo, hi) = ((c - r).ceil() as i32, (c + r).floor() as i32);
            (a - lo).max(hi - a)
        };
        let precisa = lado(dab.x, self.anchor_x).max(lado(dab.y, self.anchor_y));
        self.crescer(precisa);
    }
}
