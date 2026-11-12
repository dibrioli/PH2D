//! **O corte além da pegada, por LINHA** (B14.3): o `t` da pegada plana é convexo ao longo de uma linha, e as células
//! que o `fora` do motor deixa passar cabem num intervalo. Fora dele (com a folga dos arredondamentos do `f32` posta
//! no limiar e uma célula a mais de cada lado) o `fora` dá `true` sem ser avaliado — ao bit. E o miolo: onde o `t` é
//! minorado abaixo do limiar com a mesma folga (`t ≤ |f|₂`), dá `false` sem ser avaliado; só a orla se avalia.

use super::super::grid_map;
use ph2d_painter_brush::FootprintDeform;

/// Por linha `y0 + j`: `[x0, x1]` (células) fora do qual o corte dá `true` e `[m0, m1]` dentro do qual dá `false`.
pub(super) struct CorteDasLinhas {
    y0: i32,
    x: Vec<[i32; 4]>,
}

impl CorteDasLinhas {
    /// `limiar`: o `t` exacto a partir do qual o `fora_t` do `f32` corta (a soma das suas parcelas, sem folga).
    pub(super) fn nova(
        fp: &FootprintDeform,
        limiar: f64,
        centro: [f32; 2],
        inv_r: f32,
        ratio: u8,
        lado: f32,
    ) -> Self {
        // A folga cobre o erro relativo do `t` em `f32` (≈ 1e-6) e o das parcelas do `fora_t` com larga margem.
        let lim = limiar * (1.0 + 1e-3) + 1e-3;
        let lim_miolo = limiar * (1.0 - 1e-3) - 1e-3;
        let r = f64::from(grid_map::clamp_ratio(ratio));
        let (meio, inv_r) = (0.5 * f64::from(lado), f64::from(inv_r));
        // A célula (contínua) cujo sub-ponto `0` cai no píxel `p`: o inverso de `cell_subsample_px`.
        let celula = |p: f64| (p - meio) / r + 1.0;
        let mv = fp.falloff_t_abaixo_meia_altura(lim) * (1.0 + 1e-9);
        let c1 = f64::from(centro[1]);
        let y0 = celula(c1 - mv / inv_r).floor() as i32 - 1;
        let y1 = celula(c1 + mv / inv_r).ceil() as i32 + 1;
        let c0 = f64::from(centro[0]);
        let x = (y0..=y1.max(y0 - 1))
            .map(|cy| {
                let sy = grid_map::cell_subsample_px(cy, ratio, 0, lado);
                let v = (f64::from(sy) - c1) * inv_r;
                let [x0, x1] = fp
                    .falloff_t_abaixo_na_linha(v, lim)
                    .map_or([1, 0], |[lo, hi]| {
                        [
                            celula(c0 + lo / inv_r).floor() as i32 - 1,
                            celula(c0 + hi / inv_r).ceil() as i32 + 1,
                        ]
                    });
                let [m0, m1] = fp
                    .falloff_t_abaixo_na_linha_de_certeza(v, lim_miolo)
                    .map_or([1, 0], |[lo, hi]| {
                        [
                            celula(c0 + lo / inv_r).ceil() as i32 + 1,
                            celula(c0 + hi / inv_r).floor() as i32 - 1,
                        ]
                    });
                [x0, x1, m0, m1]
            })
            .collect();
        Self { y0, x }
    }

    /// O corte da célula quando a linha o decide sem avaliar (`Some(true)` fora, `Some(false)` no miolo); `None` na orla.
    #[inline(always)]
    pub(super) fn decide(&self, cx: i32, cy: i32) -> Option<bool> {
        let Some(&[a, b, m0, m1]) = usize::try_from(cy - self.y0)
            .ok()
            .and_then(|j| self.x.get(j))
        else {
            return Some(true);
        };
        if cx < a || cx > b {
            Some(true)
        } else if cx >= m0 && cx <= m1 {
            Some(false)
        } else {
            None
        }
    }
}
