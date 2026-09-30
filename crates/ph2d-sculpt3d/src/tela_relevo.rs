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
//! ⚠️ **A lei é `nova = antes + altura`** (D3): a partida é a altura de ANTES
//! do traço, como a cor — pousar a mesma tela duas vezes dá o mesmo que uma.

use super::Vista;

/// A espessura numa janela da tela — em PÍXEIS, linha a linha.
#[derive(Clone, Copy)]
pub struct Relevo<'a> {
    /// `janela[2] × janela[3]` alturas.
    pub px: &'a [f32],
    /// `[x, y, largura, altura]` da janela na tela.
    pub janela: [u32; 4],
}

impl Relevo<'_> {
    /// ⚠️ Fora da janela repete-se a BORDA dela — a mesma regra da cor
    /// ([`super::Tela`]); a janela já cobre tudo o que a pousada lê.
    fn texel(&self, i: i64, j: i64) -> f32 {
        let [x, y, w, h] = self.janela;
        if w == 0 || h == 0 {
            return 0.0;
        }
        let i = (i - i64::from(x)).clamp(0, i64::from(w) - 1) as usize;
        let j = (j - i64::from(y)).clamp(0, i64::from(h) - 1) as usize;
        self.px.get(j * w as usize + i).copied().unwrap_or(0.0)
    }

    /// ⭐ **A altura, em píxeis, num ponto contínuo da tela** — bilinear, com o
    /// centro do píxel `i` em `i + 0,5`: a MESMA convenção de
    /// [`super::Tela::amostra`], senão a espessura e a cor de um traço caíam
    /// meio píxel desencontradas.
    #[must_use]
    pub fn em(&self, x: f32, y: f32) -> f32 {
        let (fx, fy) = (x - 0.5, y - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let (i, j) = (x0 as i64, y0 as i64);
        let mut h = 0.0f32;
        for (di, dj, w) in [
            (0, 0, (1.0 - tx) * (1.0 - ty)),
            (1, 0, tx * (1.0 - ty)),
            (0, 1, (1.0 - tx) * ty),
            (1, 1, tx * ty),
        ] {
            if w != 0.0 {
                h += self.texel(i + di, j + dj) * w;
            }
        }
        h
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
