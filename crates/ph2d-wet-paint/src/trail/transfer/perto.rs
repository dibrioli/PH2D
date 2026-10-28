//! **O misturador PERTO da tinta** ([`Knob::BlendReach`], J6 lacuna 6 — `docs/Painter/Pinceis/exames/mistura.md`):
//! a média para onde cada célula da máscara relaxa é a das células da janela num quadrado de meio-lado `alcance` à
//! volta dela, e não a da janela inteira. A janela do rasto arrastava a mistura por todo o seu comprimento (o Blender
//! Grainy levava o vermelho 52–66 px para fora da faixa); o Blend com grão do Rebelle só a leva ~28 px. `0` = a média
//! da janela, ao bit (o [`Local::nova`] devolve `None` e o [`Media`] da janela é o de sempre).

use super::*;

/// As médias para onde uma célula relaxa (as da janela inteira, ou as locais).
#[derive(Clone, Copy)]
pub(super) struct Media {
    pub susp: f64,
    pub sett: f64,
    pub film: f64,
    pub wet: f64,
    pub tem_susp: bool,
    pub tem_sett: bool,
    /// A cor do suspenso, pesada pela massa suspensa.
    pub a: [f64; 3],
    /// A cor do assente, pesada pela massa assente.
    pub b: [f64; 3],
}

/// Os canais somados: contagem · susp · sett · film · wet · susp·rgb · sett·rgb.
const CANAIS: usize = 11;

thread_local! {
    /// A tabela de somas (reutilizada: uma janela de 200 px são ~250 mil células × 11 canais).
    static SOMAS: std::cell::RefCell<Vec<f64>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A tabela de somas acumuladas (summed-area) da janela, sobre as MESMAS células que a média da janela conta
/// (máscara `> 0`, dentro da grade com a margem de 2).
pub(super) struct Local {
    somas: Vec<f64>,
    lx0: i32,
    ly0: i32,
    /// Largura e altura da tabela (a extensão + 1).
    tw: usize,
    th: usize,
    r: i32,
}

impl Local {
    /// `None` com `alcance ≤ 0` (a média da janela, ao bit).
    pub(super) fn nova(t: &Trail, g: &Grid, alcance: f64) -> Option<Local> {
        if alcance <= 0.0 || !alcance.is_finite() || t.lx1 < t.lx0 {
            return None;
        }
        let (w, h, s) = (g.w as i32, g.h as i32, g.s);
        let tw = (t.lx1 - t.lx0 + 2) as usize;
        let th = (t.ly1 - t.ly0 + 2) as usize;
        let mut somas = SOMAS.with(|c| std::mem::take(&mut *c.borrow_mut()));
        somas.clear();
        somas.resize(tw * th * CANAIS, 0.0);
        for ly in t.ly0..=t.ly1 {
            let cy = t.anchor_y + (ly - t.half);
            let ty = (ly - t.ly0 + 1) as usize;
            let mut linha = [0.0f64; CANAIS];
            for lx in t.lx0..=t.lx1 {
                let cx = t.anchor_x + (lx - t.half);
                let dentro = cy >= 2 && cy <= h - 1 && cx >= 2 && cx <= w - 1;
                if dentro && t.mask[(lx + ly * t.size) as usize] > 0.0 {
                    let i = cx as usize + cy as usize * s;
                    let (su, se) = (g.susp[i] as f64, g.sett[i] as f64);
                    let (uc, sc) = (g.susp_rgb[i], g.sett_rgb[i]);
                    let v = [
                        1.0,
                        su,
                        se,
                        g.film[i] as f64,
                        g.wet[i] as f64,
                        uc[0] as f64 * su,
                        uc[1] as f64 * su,
                        uc[2] as f64 * su,
                        sc[0] as f64 * se,
                        sc[1] as f64 * se,
                        sc[2] as f64 * se,
                    ];
                    for (l, x) in linha.iter_mut().zip(v) {
                        *l += x;
                    }
                }
                let tx = (lx - t.lx0 + 1) as usize;
                let (cima, aqui) = (((ty - 1) * tw + tx) * CANAIS, (ty * tw + tx) * CANAIS);
                for c in 0..CANAIS {
                    somas[aqui + c] = somas[cima + c] + linha[c];
                }
            }
        }
        Some(Local {
            somas,
            lx0: t.lx0,
            ly0: t.ly0,
            tw,
            th,
            r: alcance.round().max(1.0) as i32,
        })
    }

    /// As médias das células da janela no quadrado de meio-lado `r` à volta da célula local `(lx, ly)`.
    pub(super) fn media(&self, lx: i32, ly: i32) -> Media {
        let x = lx - self.lx0;
        let y = ly - self.ly0;
        let x0 = (x - self.r).max(0) as usize;
        let y0 = (y - self.r).max(0) as usize;
        let x1 = ((x + self.r + 1) as usize).min(self.tw - 1);
        let y1 = ((y + self.r + 1) as usize).min(self.th - 1);
        let mut v = [0.0f64; CANAIS];
        let (a, b, c, d) = (
            (y1 * self.tw + x1) * CANAIS,
            (y0 * self.tw + x1) * CANAIS,
            (y1 * self.tw + x0) * CANAIS,
            (y0 * self.tw + x0) * CANAIS,
        );
        for (k, vk) in v.iter_mut().enumerate() {
            *vk = self.somas[a + k] - self.somas[b + k] - self.somas[c + k] + self.somas[d + k];
        }
        // A própria célula está na máscara: `n ≥ 1` (a soma em f64 de inteiros é exacta). As massas por diferença de
        // somas levam o resto de arredondamento das somas grandes: abaixo de `VAZIO` a vizinhança não tem tinta.
        const VAZIO: f64 = 1e-6;
        let n = v[0].max(1.0);
        let ws = if v[1] > VAZIO { v[1] } else { 0.0 };
        let wt = if v[2] > VAZIO { v[2] } else { 0.0 };
        v[3] = v[3].max(0.0);
        v[4] = v[4].max(0.0);
        Media {
            susp: ws / n,
            sett: wt / n,
            film: v[3] / n,
            wet: v[4] / n,
            tem_susp: ws > 0.0,
            tem_sett: wt > 0.0,
            a: if ws > 0.0 { [v[5] / ws, v[6] / ws, v[7] / ws] } else { [0.0; 3] },
            b: if wt > 0.0 { [v[8] / wt, v[9] / wt, v[10] / wt] } else { [0.0; 3] },
        }
    }
}

impl Drop for Local {
    fn drop(&mut self) {
        let somas = std::mem::take(&mut self.somas);
        SOMAS.with(|c| *c.borrow_mut() = somas);
    }
}
