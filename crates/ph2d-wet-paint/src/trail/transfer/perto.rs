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
                let dentro = cy >= 2 && cy < h && cx >= 2 && cx < w;
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
            a: if ws > 0.0 {
                [v[5] / ws, v[6] / ws, v[7] / ws]
            } else {
                [0.0; 3]
            },
            b: if wt > 0.0 {
                [v[8] / wt, v[9] / wt, v[10] / wt]
            } else {
                [0.0; 3]
            },
        }
    }
}

impl Drop for Local {
    fn drop(&mut self) {
        let somas = std::mem::take(&mut self.somas);
        SOMAS.with(|c| *c.borrow_mut() = somas);
    }
}

/// Os canais do material e da marca, somados pela massa `w = susp + sett`: `w` · os 12 lineares × `w` ·
/// `vidro_claro × w` · `vidro_claro × transmissao × w` · `nivelado × w`.
const CANAIS_MAT: usize = 16;

/// ⭐ **O material e a marca seguem a MESMA vizinhança da cor** (auditoria J6, motor 2): com o [`Local`] a cor, a massa,
/// o filme e a água relaxam para a média PERTO; o material (Paper Press, Metallic, Coat…) e a marca do nivelado
/// relaxavam para a da janela inteira — o material viajava a janela toda (~60 px) onde a cor já não misturava. A mesma
/// tabela de somas, os mesmos quadrados, pesada pela massa como a dobra da janela ([`crate::grid::material::media`]).
pub(super) struct LocalMat {
    somas: Vec<f64>,
    lx0: i32,
    ly0: i32,
    tw: usize,
    th: usize,
    r: i32,
    material: bool,
    marca: bool,
}

impl LocalMat {
    /// `None` sem o [`Local`] (a média da janela, ao bit) ou sem material nem marca a misturar.
    pub(super) fn nova(
        t: &Trail,
        g: &Grid,
        local: Option<&Local>,
        marca: bool,
    ) -> Option<LocalMat> {
        let local = local?;
        let material = !g.material.is_empty();
        if !material && !marca {
            return None;
        }
        let (w, h, s) = (g.w as i32, g.h as i32, g.s);
        let (tw, th) = (local.tw, local.th);
        let mut somas = vec![0.0f64; tw * th * CANAIS_MAT];
        for ly in t.ly0..=t.ly1 {
            let cy = t.anchor_y + (ly - t.half);
            let ty = (ly - t.ly0 + 1) as usize;
            let mut linha = [0.0f64; CANAIS_MAT];
            for lx in t.lx0..=t.lx1 {
                let cx = t.anchor_x + (lx - t.half);
                let dentro = cy >= 2 && cy < h && cx >= 2 && cx < w;
                if dentro && t.mask[(lx + ly * t.size) as usize] > 0.0 {
                    let i = cx as usize + cy as usize * s;
                    let m = f64::from(g.susp[i]) + f64::from(g.sett[i]);
                    if m > 0.0 {
                        let mut v = [0.0f64; CANAIS_MAT];
                        v[0] = m;
                        if material {
                            let c = g.material[i];
                            let lin = [
                                c.metal,
                                c.rugosidade,
                                c.nivel_ior,
                                c.verniz,
                                c.verniz_rugosidade,
                                c.veludo,
                                c.subsuperficie,
                                c.raio_subsuperficie,
                                c.transparencia,
                                c.transmissao,
                                c.relevo_papel,
                                c.amassa_papel,
                            ];
                            for (k, x) in lin.into_iter().enumerate() {
                                v[1 + k] = f64::from(x) * m;
                            }
                            v[13] = f64::from(c.vidro_claro) * m;
                            v[14] =
                                f64::from(c.vidro_claro) * f64::from(c.transmissao.max(0.0)) * m;
                        }
                        if marca {
                            v[15] = f64::from(g.nivelado[i]) * m;
                        }
                        for (l, x) in linha.iter_mut().zip(v) {
                            *l += x;
                        }
                    }
                }
                let tx = (lx - t.lx0 + 1) as usize;
                let (cima, aqui) = (
                    ((ty - 1) * tw + tx) * CANAIS_MAT,
                    (ty * tw + tx) * CANAIS_MAT,
                );
                for c in 0..CANAIS_MAT {
                    somas[aqui + c] = somas[cima + c] + linha[c];
                }
            }
        }
        Some(LocalMat {
            somas,
            lx0: local.lx0,
            ly0: local.ly0,
            tw,
            th,
            r: local.r,
            material,
            marca,
        })
    }

    /// O material e a marca PERTO da célula local `(lx, ly)` (`None` onde a vizinhança não tem tinta, ou sem o campo).
    pub(super) fn media(
        &self,
        lx: i32,
        ly: i32,
    ) -> (Option<crate::material::MaterialCelula>, Option<f64>) {
        let x = lx - self.lx0;
        let y = ly - self.ly0;
        let x0 = (x - self.r).max(0) as usize;
        let y0 = (y - self.r).max(0) as usize;
        let x1 = ((x + self.r + 1) as usize).min(self.tw - 1);
        let y1 = ((y + self.r + 1) as usize).min(self.th - 1);
        let mut v = [0.0f64; CANAIS_MAT];
        let (a, b, c, d) = (
            (y1 * self.tw + x1) * CANAIS_MAT,
            (y0 * self.tw + x1) * CANAIS_MAT,
            (y1 * self.tw + x0) * CANAIS_MAT,
            (y0 * self.tw + x0) * CANAIS_MAT,
        );
        for (k, vk) in v.iter_mut().enumerate() {
            *vk = self.somas[a + k] - self.somas[b + k] - self.somas[c + k] + self.somas[d + k];
        }
        const VAZIO: f64 = 1e-6;
        let wm = v[0];
        if wm <= VAZIO {
            return (None, None);
        }
        let q = |k: usize| (v[k] / wm) as f32;
        let mat = self.material.then(|| {
            let wt = v[10];
            crate::material::MaterialCelula {
                metal: q(1),
                rugosidade: q(2),
                nivel_ior: q(3),
                verniz: q(4),
                verniz_rugosidade: q(5),
                veludo: q(6),
                subsuperficie: q(7),
                raio_subsuperficie: q(8),
                transparencia: q(9),
                transmissao: q(10),
                relevo_papel: q(11),
                amassa_papel: q(12),
                // O modo pela massa TRANSMISSIVA (a lei de `MaterialCelula::mistura`); sem ela, linear.
                vidro_claro: if wt > VAZIO {
                    (v[14] / wt).clamp(0.0, 1.0) as f32
                } else {
                    q(13)
                },
            }
        });
        (mat, self.marca.then(|| v[15] / wm))
    }
}
