//! ⭐ **A ORLA DO TRAÇO** (L8, `ph2d_painter_brush::traco`; bug #58, `docs/Painter/BUGS_painter_fechados_41_a_46.md`):
//! o termo `e ∈ [0, 1]` das orlas molhada e queimada, por texel.
//!
//! ## A lei
//!
//! 1. **Uma máscara, a VISÍVEL:** `v = min(alfa do plano, tecto)` — o que o artista vê, a união do traço inteiro.
//!    Nunca o alfa cru: a acumulação por baixo do tecto é invisível e não pode desenhar fronteiras.
//! 2. **A fronteira é do TRAÇO, e de UMA passagem:** o nível `τ = ½·max(v)`, o máximo corrente até o traço percorrer
//!    um diâmetro de dab e daí FIXO (`PlanoDoTraco::fronteira`) — com Strength 0,3 a fronteira existe (um nível fixo
//!    não a via), e a sobreposição do traço consigo nunca a sobe: a borda do que já está pintado não anda.
//! 3. **A distância é EXACTA e isotrópica:** a distância euclidiana de cada texel de dentro (`v > τ`) aos segmentos
//!    do contorno `v = τ` (marching squares sobre `v` interpolado nas arestas — sub-píxel, exacto numa borda recta a
//!    qualquer ângulo), presa a `W` (o Edge width). Sem núcleos quadrados.
//! 4. **O perfil:** `e = 1 − smoothstep(0, W, d)` dentro; `e = 1` na franja de fora (`0 < v ≤ τ`: a franja É a orla,
//!    e o antialias fica contínuo com ela); `0` onde não há tinta.
//!
//! ⚠️ **A janela não muda um bit:** os segmentos e as distâncias contam-se em coordenadas da TELA (diferenças
//! inteiras + o sub-píxel do corte), o `min` não depende da ordem, e tudo o que está a mais de `W` fica preso a `W`
//! — logo qualquer janela com folga [`alcance_da_janela`] dá o que a tela inteira dá.

use super::Region;
use rayon::prelude::*;

/// A partir de quantos texels a orla corre no pool (o mesmo limiar da composição, `traco_plano`).
const EM_PARALELO_DESDE: usize = 16_384;

/// A folga (px) que a janela de `v` precisa à volta da região pedida para a orla de largura `w` ser exacta.
#[must_use]
pub(super) fn alcance_da_janela(w: f32) -> u32 {
    w.ceil() as u32 + 3
}

/// O perfil da orla à distância `d` do contorno, para a largura `w`.
#[must_use]
#[inline]
pub(super) fn perfil(d: f32, w: f32) -> f32 {
    let t = (d / w).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// Um segmento do contorno numa célula: os dois cantos em coordenadas LOCAIS da célula (`0..1`).
#[derive(Clone, Copy)]
struct Seg {
    cx: i32,
    cy: i32,
    a: [f32; 2],
    b: [f32; 2],
}

/// Os segmentos do contorno `v = τ` na célula de canto `(x, y)` da janela (os quatro texels `v`), no sentido dos
/// cantos `00 · 10 · 11 · 01`. O ponto de sela decide-se pela média dos quatro.
fn segmentos_da_celula(c: [f32; 4], tau: f32, mut f: impl FnMut([f32; 2], [f32; 2])) {
    let [v00, v10, v11, v01] = c;
    let k = u8::from(v00 > tau)
        | u8::from(v10 > tau) << 1
        | u8::from(v11 > tau) << 2
        | u8::from(v01 > tau) << 3;
    if k == 0 || k == 15 {
        return;
    }
    let corte = |a: f32, b: f32| ((tau - a) / (b - a)).clamp(0.0, 1.0);
    let cima = || [corte(v00, v10), 0.0];
    let dir = || [1.0, corte(v10, v11)];
    let baixo = || [corte(v01, v11), 1.0];
    let esq = || [0.0, corte(v00, v01)];
    let centro_dentro = (v00 + v10 + v11 + v01) * 0.25 > tau;
    match k {
        1 | 14 => f(cima(), esq()),
        2 | 13 => f(cima(), dir()),
        4 | 11 => f(dir(), baixo()),
        8 | 7 => f(esq(), baixo()),
        3 | 12 => f(esq(), dir()),
        6 | 9 => f(cima(), baixo()),
        // As selas: com o centro dentro os cantos de FORA ficam isolados; com o centro fora, os de dentro.
        5 => {
            if centro_dentro {
                f(cima(), dir());
                f(esq(), baixo());
            } else {
                f(cima(), esq());
                f(dir(), baixo());
            }
        }
        _ => {
            if centro_dentro {
                f(cima(), esq());
                f(dir(), baixo());
            } else {
                f(cima(), dir());
                f(esq(), baixo());
            }
        }
    }
}

/// O quadrado da distância da origem ao segmento `a → b`.
#[inline]
fn dist2_ao_segmento(a: [f32; 2], b: [f32; 2]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let l2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if l2 > 0.0 {
        (-(a[0] * ab[0] + a[1] * ab[1]) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let p = [a[0] + t * ab[0], a[1] + t * ab[1]];
    p[0] * p[0] + p[1] * p[1]
}

/// Os segmentos do contorno `v = τ` de uma janela, em baldes: a distância exacta de um texel ao contorno.
struct Contorno<'a> {
    v: &'a [f32],
    jan: Region,
    tau: f32,
    alcance: i32,
    lado: usize,
    cw: usize,
    ch: usize,
    nbx: usize,
    nby: usize,
    por_balde: Vec<Vec<Seg>>,
}

impl<'a> Contorno<'a> {
    /// O contorno de `v` (`jan.w × jan.h`) ao nível `tau`, para distâncias até `w` px.
    fn new(v: &'a [f32], jan: Region, tau: f32, w: f32) -> Self {
        let (jw, jh) = (jan.w as usize, jan.h as usize);
        debug_assert_eq!(v.len(), jw * jh);
        let (cw, ch) = (jw.saturating_sub(1), jh.saturating_sub(1));
        // As células em baldes de lado `b` (≥ W + 2): um segmento a menos de `W` de um texel cai no balde dele ou
        // num vizinho.
        let alcance = w.ceil() as i32 + 1;
        let lado = (alcance + 1) as usize;
        let (nbx, nby) = (cw.div_ceil(lado).max(1), ch.div_ceil(lado).max(1));
        let mut por_balde: Vec<Vec<Seg>> = vec![Vec::new(); nbx * nby];
        for cy in 0..ch {
            for cx in 0..cw {
                let i = cy * jw + cx;
                let c = [v[i], v[i + 1], v[i + jw + 1], v[i + jw]];
                segmentos_da_celula(c, tau, |a, b| {
                    por_balde[(cy / lado) * nbx + cx / lado].push(Seg {
                        cx: (jan.x as usize + cx) as i32,
                        cy: (jan.y as usize + cy) as i32,
                        a,
                        b,
                    });
                });
            }
        }
        Self {
            v,
            jan,
            tau,
            alcance,
            lado,
            cw,
            ch,
            nbx,
            nby,
            por_balde,
        }
    }

    /// `v` no texel `(x, y)` da tela.
    fn v(&self, x: usize, y: usize) -> f32 {
        self.v[(y - self.jan.y as usize) * self.jan.w as usize + (x - self.jan.x as usize)]
    }

    /// O quadrado da distância do texel `(x, y)` da tela ao contorno, preso a `teto2`.
    fn d2(&self, x: usize, y: usize, teto2: f32) -> f32 {
        let (px, py) = (x as i32, y as i32);
        let (lx, ly) = (px - self.jan.x as i32, py - self.jan.y as i32);
        let (alcance, lado) = (self.alcance, self.lado);
        let bx0 = ((lx - alcance).max(0) as usize) / lado;
        let bx1 = (((lx + alcance).max(0) as usize).min(self.cw.saturating_sub(1)) / lado)
            .min(self.nbx - 1);
        let by0 = ((ly - alcance).max(0) as usize) / lado;
        let by1 = (((ly + alcance).max(0) as usize).min(self.ch.saturating_sub(1)) / lado)
            .min(self.nby - 1);
        let mut melhor = teto2;
        for by in by0..=by1 {
            for bx in bx0..=bx1 {
                for s in &self.por_balde[by * self.nbx + bx] {
                    let (ox, oy) = ((s.cx - px) as f32, (s.cy - py) as f32);
                    let d2 =
                        dist2_ao_segmento([ox + s.a[0], oy + s.a[1]], [ox + s.b[0], oy + s.b[1]]);
                    melhor = melhor.min(d2);
                }
            }
        }
        melhor
    }
}

/// `f(x, y)` por texel de `r` (linha a linha, `r.w × r.h`), no pool acima de [`EM_PARALELO_DESDE`].
fn por_texel(r: Region, f: impl Fn(usize, usize) -> f32 + Sync) -> Vec<f32> {
    let (rw, rh) = (r.w as usize, r.h as usize);
    let mut e = vec![0.0f32; rw * rh];
    if rw == 0 {
        return e;
    }
    let linha = |(dy, l): (usize, &mut [f32])| {
        for (dx, o) in l.iter_mut().enumerate() {
            *o = f(r.x as usize + dx, r.y as usize + dy);
        }
    };
    if rw * rh < EM_PARALELO_DESDE {
        e.chunks_mut(rw).enumerate().for_each(linha);
    } else {
        e.par_chunks_mut(rw).enumerate().for_each(linha);
    }
    e
}

/// ⭐ **A orla em `r`**: `e` por texel de `r` (linha a linha, `r.w × r.h`), a partir da máscara visível `v` na janela
/// `jan` (`jan.w × jan.h`, em coordenadas da tela), que contém `r` com a folga [`alcance_da_janela`] (presa à tela).
/// `tau` é o nível da fronteira e `w` a largura (px, `≥ 1`).
#[must_use]
pub(super) fn orla_do_traco(v: &[f32], jan: Region, r: Region, tau: f32, w: f32) -> Vec<f32> {
    let c = Contorno::new(v, jan, tau, w);
    let w2 = w * w;
    por_texel(r, |x, y| {
        let vi = c.v(x, y);
        if vi <= tau {
            return if vi > 0.0 { 1.0 } else { 0.0 };
        }
        perfil(c.d2(x, y, w2).sqrt(), w)
    })
}

/// ⭐ **A distância ASSINADA ao contorno** `v = τ` por texel de `r` (positiva dentro, `v > τ`; negativa na franja,
/// `0 < v ≤ τ`), presa a `±alcance`; `−alcance` onde não há tinta. A régua do teto do aro da aguada
/// ([`super::watercolor_rim::straight_edge_cap`]) no Wet do Digital ([`super::traco_aguada`]). A janela com a folga
/// [`alcance_da_janela`]`(alcance)` dá o que a tela inteira dá.
#[must_use]
pub(super) fn distancia_assinada(
    v: &[f32],
    jan: Region,
    r: Region,
    tau: f32,
    alcance: f32,
) -> Vec<f32> {
    let c = Contorno::new(v, jan, tau, alcance);
    let a2 = alcance * alcance;
    por_texel(r, |x, y| {
        let vi = c.v(x, y);
        if vi <= 0.0 {
            return -alcance;
        }
        let d = c.d2(x, y, a2).sqrt();
        if vi > c.tau { d } else { -d }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LADO: u32 = 160;

    fn tela() -> Region {
        Region {
            x: 0,
            y: 0,
            w: LADO,
            h: LADO,
        }
    }

    /// `v` a partir da distância assinada `sd(x, y)` (positiva dentro): uma rampa LINEAR de 5 px à volta de
    /// `v = 0,25`, saturada em `0..0,5` — o contorno `τ = 0,25` é exactamente `sd = 0`.
    fn campo(sd: impl Fn(f32, f32) -> f32) -> Vec<f32> {
        (0..LADO * LADO)
            .map(|i| {
                let (x, y) = ((i % LADO) as f32, (i / LADO) as f32);
                (0.25 + 0.1 * sd(x, y)).clamp(0.0, 0.5)
            })
            .collect()
    }

    /// O maior erro de `e` contra o perfil analítico `perfil(sd)` nos texels de dentro, longe da borda da tela.
    fn erro(v: &[f32], sd: impl Fn(f32, f32) -> f32, w: f32) -> f32 {
        let e = orla_do_traco(v, tela(), tela(), 0.25, w);
        let mut max = 0.0f32;
        for y in 20..LADO - 20 {
            for x in 20..LADO - 20 {
                let s = sd(x as f32, y as f32);
                if s > 0.0 {
                    max = max.max((e[(y * LADO + x) as usize] - perfil(s, w)).abs());
                }
            }
        }
        max
    }

    /// ⭐ **(c) Isotropia**: um semiplano a 0°, 17° e 45° e um disco dão o perfil analítico `1 − smoothstep(0, W, d)`
    /// a ±1/255 — a distância é a euclidiana ao contorno sub-píxel, não a de um núcleo quadrado.
    #[test]
    fn a_orla_e_isotropica() {
        for w in [3.0, 7.0, 20.0] {
            for graus in [0.0f32, 17.0, 45.0] {
                let (s, c) = graus.to_radians().sin_cos();
                let sd = move |x: f32, y: f32| (x - 80.3) * c + (y - 80.3) * s;
                let err = erro(&campo(sd), sd, w);
                assert!(err <= 1.0 / 255.0, "W {w}, {graus}°: erro {err}");
            }
            let sd = |x: f32, y: f32| 50.0 - (x - 80.5).hypot(y - 79.7);
            let err = erro(&campo(sd), sd, w);
            assert!(err <= 1.0 / 255.0, "W {w}, disco: erro {err}");
        }
    }

    /// ⭐ **(d) A janela não muda um bit**: qualquer recorte com a folga dá o que a tela inteira dá.
    #[test]
    fn qualquer_janela_da_a_orla_da_tela() {
        // Um campo com ilhas, um cruzamento e um vale (a soma de dois discos e um anel).
        let v: Vec<f32> = (0..LADO * LADO)
            .map(|i| {
                let (x, y) = ((i % LADO) as f32, (i / LADO) as f32);
                let d1 = (30.0 - (x - 60.0).hypot(y - 70.0)) * 0.04;
                let d2 = (25.0 - (x - 95.0).hypot(y - 80.0)) * 0.04;
                let ilha = (3.0 - (x - 130.0).hypot(y - 30.0)) * 0.2;
                (d1.clamp(0.0, 0.3) + d2.clamp(0.0, 0.3) + ilha.clamp(0.0, 0.5)).min(0.5)
            })
            .collect();
        let w = 9.0;
        let tudo = orla_do_traco(&v, tela(), tela(), 0.2, w);
        let folga = alcance_da_janela(w);
        for (x, y, rw, rh) in [
            (10, 10, 40, 30),
            (55, 61, 7, 50),
            (100, 5, 60, 155),
            (0, 0, 1, 1),
        ] {
            let r = Region { x, y, w: rw, h: rh };
            let x0 = x.saturating_sub(folga);
            let y0 = y.saturating_sub(folga);
            let jan = Region {
                x: x0,
                y: y0,
                w: (x + rw + folga).min(LADO) - x0,
                h: (y + rh + folga).min(LADO) - y0,
            };
            let vj: Vec<f32> = (0..jan.h)
                .flat_map(|jy| {
                    let v = &v;
                    (0..jan.w).map(move |jx| v[((jan.y + jy) * LADO + jan.x + jx) as usize])
                })
                .collect();
            let e = orla_do_traco(&vj, jan, r, 0.2, w);
            for yy in 0..rh {
                for xx in 0..rw {
                    let a = e[(yy * rw + xx) as usize];
                    let b = tudo[((y + yy) * LADO + x + xx) as usize];
                    assert!(
                        a.to_bits() == b.to_bits(),
                        "({}, {}): {a} × {b}",
                        x + xx,
                        y + yy
                    );
                }
            }
        }
    }
}
