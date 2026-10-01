//! ⭐⭐⭐ **A ESPESSURA da tela pousada na peça** (`docs/3D/29`, D2/D3) — filho
//! de `tela_na_malha.rs`. O Painter entrega a altura do impasto em PÍXEIS de
//! ecrã; a peça guarda-a em unidades de OBJECTO ao longo da normal.
//!
//! # A conversão
//!
//! `altura_objecto = altura_px × mundo_por_píxel(p)` — o tamanho de um píxel
//! NAQUELE ponto da superfície, na vista em que foi pintado. ⇒ o mesmo traço
//! dá o mesmo relevo *visto do ecrã* perto ou longe da câmara, que é o que o
//! artista pinta (a mesma escolha que o Painter faz numa sprite: a espessura
//! é da tinta no ecrã, não do mundo).
//!
//! ⚠️ **A lei é a DIFERENÇA** (`docs/3D/29` §6): a partida é o relevo de ANTES
//! do traço, como a cor, e o que se soma é o que a tela MUDOU desde a semente
//! dela — `nova = antes + (tela − semente)`, na altura e no corpo. Pousar a
//! mesma tela duas vezes dá o mesmo que uma, e o que o pincel não tocou tem
//! diferença ZERO (os mesmos números dos dois lados). Sem semente (a peça
//! ainda não tinha relevo) a semente é zero e a lei reduz à de antes.

use super::{TelaNaMalha, Vista};

/// O relevo numa janela da tela — a espessura em PÍXEIS e o corpo `0..1`,
/// linha a linha.
#[derive(Clone, Copy)]
pub struct Relevo<'a> {
    /// `janela[2] × janela[3]` alturas.
    pub px: &'a [f32],
    /// `janela[2] × janela[3]` corpos, `0..1` — QUANTA tinta está em cada
    /// píxel. ⚠️ Sem ele a encosta que o alisamento do impasto espalha para
    /// fora da tinta acendia o barro nu (o anel do report de 01/10).
    pub corpo: &'a [f32],
    /// `[x, y, largura, altura]` da janela na tela.
    pub janela: [u32; 4],
}

impl Relevo<'_> {
    /// ⚠️ Fora da janela repete-se a BORDA dela — a mesma regra da cor
    /// ([`super::Tela`]); a janela já cobre tudo o que a pousada lê.
    fn texel(&self, i: i64, j: i64) -> [f32; 2] {
        let [x, y, w, h] = self.janela;
        if w == 0 || h == 0 {
            return [0.0; 2];
        }
        let i = (i - i64::from(x)).clamp(0, i64::from(w) - 1) as usize;
        let j = (j - i64::from(y)).clamp(0, i64::from(h) - 1) as usize;
        let k = j * w as usize + i;
        [
            self.px.get(k).copied().unwrap_or(0.0),
            self.corpo.get(k).copied().unwrap_or(0.0),
        ]
    }

    /// ⭐ **A altura (em píxeis) e o corpo num ponto contínuo da tela** —
    /// bilinear, com o centro do píxel `i` em `i + 0,5`: a MESMA convenção de
    /// [`super::Tela::amostra`], senão a espessura e a cor de um traço caíam
    /// meio píxel desencontradas.
    #[must_use]
    pub fn em(&self, x: f32, y: f32) -> [f32; 2] {
        let (fx, fy) = (x - 0.5, y - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let (i, j) = (x0 as i64, y0 as i64);
        let mut h = [0.0f32; 2];
        for (di, dj, w) in [
            (0, 0, (1.0 - tx) * (1.0 - ty)),
            (1, 0, tx * (1.0 - ty)),
            (0, 1, (1.0 - tx) * ty),
            (1, 1, tx * ty),
        ] {
            if w != 0.0 {
                let t = self.texel(i + di, j + dj);
                h[0] += t[0] * w;
                h[1] += t[1] * w;
            }
        }
        h
    }
}

/// ⭐⭐ **O relevo com que a tela do Painter COMEÇOU** — a semente de relevo,
/// irmã do retrato de cor ([`TelaNaMalha::com_semente`]), a tela inteira.
pub(super) struct SementeDoRelevo {
    px: Vec<f32>,
    corpo: Vec<f32>,
    largura: u32,
    altura: u32,
}

impl TelaNaMalha {
    /// ⭐⭐ **A tela começou com este relevo da peça** (`docs/3D/29` §6) — a
    /// espessura em píxeis e o corpo, a tela inteira, por píxel.
    ///
    /// ⚠️ **Têm de ser os MESMOS números que a tela tem** — quem semeia lê-os
    /// DE VOLTA da tela depois de a semear, senão o que o pincel não tocou
    /// deixa de se anular na diferença. ⛔ Recusa (`false`, e nada muda) um
    /// tamanho que não é o da vista.
    pub fn com_semente_relevo(&mut self, px: Vec<f32>, corpo: Vec<f32>) -> bool {
        let (w, h) = self.vista.tamanho();
        let n = (w as usize) * (h as usize);
        if px.len() != n || corpo.len() != n {
            return false;
        }
        self.semente_relevo = Some(SementeDoRelevo {
            px,
            corpo,
            largura: w,
            altura: h,
        });
        true
    }

    /// A semente de relevo como uma janela — a tela inteira —, ou `None` sem
    /// ela (a lei da diferença compara então contra zero).
    #[must_use]
    pub fn semente_relevo(&self) -> Option<Relevo<'_>> {
        self.semente_relevo.as_ref().map(|s| Relevo {
            px: &s.px,
            corpo: &s.corpo,
            janela: [0, 0, s.largura, s.altura],
        })
    }
}

impl Vista {
    /// ⭐⭐ **Quanto mede um píxel NAQUELE ponto** — em unidades locais da
    /// peça, PARALELO ao plano da imagem. `None` atrás do olho ou no próprio
    /// olho.
    ///
    /// Mede-se projectando o ponto e dois vizinhos deslocados no plano da
    /// imagem (uma fracção fixa da distância ao olho, para a conta não
    /// depender da escala da peça), e a medida é a média geométrica das duas
    /// direcções. ⚠️ **Paralelo à IMAGEM e não perpendicular ao RAIO:** fora do
    /// eixo os dois diferem (medido: `0,5 %` no canto da tela de teste), e a
    /// espessura que o Painter pinta é medida no plano do ecrã.
    ///
    /// O «em frente» da imagem é a linha `w` da matriz (a profundidade de uma
    /// perspectiva); numa vista sem perspectiva essa linha é nula e usa-se a
    /// linha `z`.
    #[must_use]
    pub fn mundo_por_pixel(&self, p: [f32; 3]) -> Option<f32> {
        let d = [
            p[0] - self.olho[0],
            p[1] - self.olho[1],
            p[2] - self.olho[2],
        ];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if dist.is_nan() || dist <= 0.0 {
            return None;
        }
        let m = &self.local_para_clip;
        let unit = |v: [f32; 3]| {
            let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            (n > 0.0).then(|| [v[0] / n, v[1] / n, v[2] / n])
        };
        let cruz = |a: [f32; 3], b: [f32; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let frente = unit([m[3], m[7], m[11]]).or_else(|| unit([m[2], m[6], m[10]]))?;
        let cima = if frente[1].abs() < 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let u = unit(cruz(frente, cima))?;
        let v = unit(cruz(frente, u))?;
        let eps = dist * 1e-3;
        let a = self.ecra(p)?;
        let mede = |dir: [f32; 3]| -> Option<f32> {
            let b = self.ecra([
                p[0] + dir[0] * eps,
                p[1] + dir[1] * eps,
                p[2] + dir[2] * eps,
            ])?;
            Some(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt())
        };
        let px = (mede(u)? * mede(v)?).sqrt();
        (px > 0.0).then(|| eps / px)
    }
}
