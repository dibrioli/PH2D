//! **Os FIOS com física própria** (W2, prompt 48; doc 48_medicao). O tapete de cerdas (`brush.rs`) é lido no
//! desvio ao centro do dab, então cada fio do tapete arrasta a MESMA faixa pelo traço inteiro — os «carris».
//! Aqui cada fio ganha o que um pelo real tem ao longo do caminho, deterministicamente (semente fixa, sem
//! estado): a PRESSÃO própria (ruído de Perlin no comprimento do caminho, por fio), a LARGURA que ondula (as
//! fronteiras entre fios andam) e a OSCILAÇÃO do pincel inteiro na normal do traço. Tudo em píxeis do canvas,
//! como o tapete. [`Fios::NEUTRO`] = nenhum dos três: o hospedeiro nem chama a lei (a de antes ao bit).

use crate::rng::hash2_signed;

const SEMENTE_PRESSAO: u32 = 0x6f69_0001;
const SEMENTE_ONDA: u32 = 0x6f69_0002;
const SEMENTE_OSCILACAO: u32 = 0x6f69_0003;
const SEMENTE_SPAN: u32 = 0x6f69_0004;
const SEMENTE_COR: u32 = 0x6f69_0005;

/// O tecto da onda da largura: a esta amplitude um fio estreita até ¼ da largura e nunca some (o `min_width`
/// 0,25 dos fios do Rebelle, doc 02 §2.4).
pub const ONDA_MAX: f64 = 0.75;

/// Os knobs dos fios de um pincel (os do mapa `engine` do preset).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fios {
    /// A amplitude da pressão de cada fio: o ganho é `max(0, 1 + pressao · ruído − levanta · (1 − contacto))`
    /// (`0` = todos a `1`).
    pub pressao: f64,
    /// Quanto a mão LEVE levanta os fios (o `contacto` é o raio do dab ÷ o nominal: a pressão pela curva do
    /// tamanho): os fios fracos saem do papel primeiro, como no Rebelle (o leve é mais texturado; `0` = nada).
    pub levanta: f64,
    /// O comprimento de caminho (px) em que a pressão de um fio (e a oscilação) muda.
    pub ruido_px: f64,
    /// A largura de um fio (px): o que partilha a mesma pressão através do traço.
    pub largura_px: f64,
    /// A amplitude com que a largura de cada fio ondula (`0` = fixa; até [`ONDA_MAX`]).
    pub onda: f64,
    /// O comprimento da onda da largura (px de caminho).
    pub onda_px: f64,
    /// A oscilação do pincel na normal do traço (px; `0` = nenhuma).
    pub oscilacao_px: f64,
    /// O «Random Span»: quanto (px) cada fio se desvia da sua faixa, por fio, ao longo do caminho (`0` = nada).
    pub span_px: f64,
    /// O comprimento de caminho (px) em que o desvio de um fio muda.
    pub span_comprimento_px: f64,
    /// O ruído de cor de cada fio em HSLuv: o matiz (voltas), a saturação e a luminosidade (fracções de 100).
    pub cor: [f64; 3],
}

impl Fios {
    pub const NEUTRO: Fios = Fios {
        pressao: 0.0,
        levanta: 0.0,
        ruido_px: 150.0,
        largura_px: 3.0,
        onda: 0.0,
        onda_px: 500.0,
        oscilacao_px: 0.0,
        span_px: 0.0,
        span_comprimento_px: 10.0,
        cor: [0.0; 3],
    };

    /// Nenhum dos três age: a lei de antes.
    #[must_use]
    pub fn neutro(&self) -> bool {
        self.pressao == 0.0
            && self.levanta == 0.0
            && self.onda == 0.0
            && self.oscilacao_px == 0.0
            && self.span_px == 0.0
            && !self.tem_cor()
    }

    /// O ruído de cor age.
    #[must_use]
    pub fn tem_cor(&self) -> bool {
        self.cor != [0.0; 3]
    }

    /// A lei num dab no comprimento de caminho `arco` (px) com o rumo `rumo` (unitário; `[0, 0]` = o 1.º dab,
    /// lido como horizontal) e o `contacto` (`0..1`, ver [`Self::levanta`]) — `None` no neutro.
    #[must_use]
    pub fn no_dab(&self, arco: f64, rumo: [f64; 2], contacto: f64) -> Option<FiosNoDab> {
        if self.neutro() {
            return None;
        }
        Some(self.resolvido(arco, rumo, contacto))
    }

    /// [`Self::no_dab`] mesmo no neutro: as faixas dos fios existem sem pressão nem cor (o RELEVO de cada fio,
    /// W3, lê-as).
    #[must_use]
    pub fn resolvido(&self, arco: f64, rumo: [f64; 2], contacto: f64) -> FiosNoDab {
        let t = if rumo == [0.0, 0.0] { [1.0, 0.0] } else { rumo };
        let osc = self.oscilacao_px * ruido(arco / self.ruido_px.max(1.0), 0, SEMENTE_OSCILACAO);
        FiosNoDab {
            fios: *self,
            arco,
            normal: [-t[1], t[0]],
            desvio_px: osc,
            baixa: self.levanta * (1.0 - contacto.clamp(0.0, 1.0)),
        }
    }
}

/// Um ponto do dab lido pelos fios ([`FiosNoDab::no_ponto`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ponto {
    /// O desvio (px) ao centro do dab onde o tapete se lê.
    pub q: [f64; 2],
    /// O ganho da pressão do fio (`≥ 0`).
    pub ganho: f64,
    /// O fio do ponto (a faixa na normal).
    pub fio: i32,
}

/// [`Fios`] resolvidos num dab.
#[derive(Clone, Copy, Debug)]
pub struct FiosNoDab {
    fios: Fios,
    arco: f64,
    normal: [f64; 2],
    desvio_px: f64,
    /// O que a mão leve tira do ganho de todos os fios (`levanta · (1 − contacto)`).
    baixa: f64,
}

impl FiosNoDab {
    /// Num ponto a `d` px do centro do dab: o desvio onde o tapete se lê (a oscilação desloca o pincel, então
    /// o ponto lê o pelo que está a `−desvio` dele; o span, o pelo do MESMO fio a `−span` na normal), o ganho da
    /// pressão do fio desse pelo e o fio.
    #[must_use]
    pub fn no_ponto(&self, d: [f64; 2]) -> Ponto {
        let f = &self.fios;
        let n = self.normal;
        let mut q = [d[0] - n[0] * self.desvio_px, d[1] - n[1] * self.desvio_px];
        let l = q[0] * n[0] + q[1] * n[1];
        let fio = self.fio(l / f.largura_px.max(0.25));
        if f.span_px != 0.0 {
            let s = f.span_px
                * ruido(
                    self.arco / f.span_comprimento_px.max(1.0),
                    fio,
                    SEMENTE_SPAN,
                );
            q = [q[0] - n[0] * s, q[1] - n[1] * s];
        }
        let ganho = self.ganho_do_fio(fio);
        Ponto { q, ganho, fio }
    }

    /// ⭐ **Entre que dois fios** está o ponto a `d` px do centro do dab: os dois fios vizinhos cujos CENTROS o
    /// cercam na normal, e onde ele cai entre eles (`0` = no centro do 1.º, `1` = no do 2.º). É o que deixa o
    /// relevo CONTÍNUO através dos fios (W3b, [`crate::bristles_relevo::entre`]): o fio de [`Self::no_ponto`]
    /// é um degrau, e um degrau de altura num traço inclinado é uma escada (doc 50_medicao).
    #[must_use]
    pub fn entre_fios(&self, d: [f64; 2]) -> ([i32; 2], f64) {
        let n = self.normal;
        let q = [d[0] - n[0] * self.desvio_px, d[1] - n[1] * self.desvio_px];
        let u = (q[0] * n[0] + q[1] * n[1]) / self.fios.largura_px.max(0.25);
        let k = self.fio(u);
        let centro = |k: i32| 0.5 * (self.fronteira(k) + self.fronteira(k + 1));
        let [a, b] = if u >= centro(k) {
            [k, k + 1]
        } else {
            [k - 1, k]
        };
        let (ca, cb) = (centro(a), centro(b));
        ([a, b], ((u - ca) / (cb - ca)).clamp(0.0, 1.0))
    }

    /// O ganho da pressão do fio `fio` neste dab — o mesmo para todos os pontos dele.
    #[must_use]
    pub fn ganho_do_fio(&self, fio: i32) -> f64 {
        let f = &self.fios;
        if f.pressao == 0.0 && self.baixa == 0.0 {
            1.0
        } else {
            (1.0 + f.pressao * ruido(self.arco / f.ruido_px.max(1.0), fio, SEMENTE_PRESSAO)
                - self.baixa)
                .max(0.0)
        }
    }

    /// Os fios que um ponto a até `raio_px` do centro pode ler: `(primeiro, último)` (a oscilação e a onda incluídas).
    #[must_use]
    pub fn faixa(&self, raio_px: f64) -> (i32, i32) {
        let w = self.fios.largura_px.max(0.25);
        let l = raio_px + self.desvio_px.abs();
        ((-l / w).floor() as i32 - 1, (l / w).floor() as i32 + 1)
    }

    /// A cor sRGB `0..255` do fio `fio` a partir de `rgb`: o ruído HSLuv do fio, que anda ao longo do caminho
    /// como a pressão (cada componente com o seu ruído); sem ruído de cor, `rgb` intacto.
    #[must_use]
    pub fn cor_do_fio(&self, rgb: [f32; 3], fio: i32) -> [f32; 3] {
        let c = self.fios.cor;
        if c == [0.0; 3] {
            return rgb;
        }
        let t = self.arco / self.fios.ruido_px.max(1.0);
        let r = |i: u32| ruido(t, fio, SEMENTE_COR.wrapping_add(i));
        crate::bristles_cor::com_ruido(rgb, [c[0] * r(0), c[1] * r(1), c[2] * r(2)])
    }

    /// O fio em `u` (larguras de fio na normal): o `k` com `fronteira(k) ≤ u < fronteira(k + 1)`. Cada fronteira
    /// anda no máximo meia largura (`onda ≤ ONDA_MAX < 1`), então ela é crescente e o fio é um dos três vizinhos.
    fn fio(&self, u: f64) -> i32 {
        let k0 = u.floor() as i32;
        if self.fios.onda.clamp(0.0, ONDA_MAX) == 0.0 {
            return k0;
        }
        if u < self.fronteira(k0) {
            k0 - 1
        } else if u >= self.fronteira(k0 + 1) {
            k0 + 1
        } else {
            k0
        }
    }

    /// A fronteira de baixo do fio `k` (larguras de fio na normal): `k`, mais a onda dela.
    fn fronteira(&self, k: i32) -> f64 {
        let f = &self.fios;
        let onda = f.onda.clamp(0.0, ONDA_MAX);
        if onda == 0.0 {
            return f64::from(k);
        }
        f64::from(k) + 0.5 * onda * perlin(self.arco / f.onda_px.max(1.0), k, SEMENTE_ONDA)
    }
}

/// ⭐ **O ruído de um fio** ao longo do caminho: o [`perlin`] da fila `k` com uma FASE própria e uma 2.ª
/// oitava (×2,13, metade da força), dividido por 1,5 — ~`[-1, 1]`. Sem a fase, o Perlin de todos os fios passa
/// por zero no mesmo arco a cada `ruido_px` e o traço inteiro volta ao neutro com esse período (medido no
/// Express Oil: 149 px a 57 % da potência, com `ruido_px` 150).
#[must_use]
pub fn ruido(t: f64, k: i32, semente: u32) -> f64 {
    let fase = |o: i32| crate::rng::hash2(k, o, semente);
    (perlin(t + fase(1), k, semente) + 0.5 * perlin(2.13 * t + fase(2), k, semente ^ 0x9e37)) / 1.5
}

/// O ruído de Perlin 1-D da fila `k` em `t` (gradientes em `[-1, 1)` nos inteiros, `fade` quíntico), escalado
/// para ~`[-1, 1]`; `0` em todo inteiro.
#[must_use]
pub fn perlin(t: f64, k: i32, semente: u32) -> f64 {
    let i = t.floor();
    let f = t - i;
    let i = i as i32;
    let g0 = hash2_signed(k, i, semente);
    let g1 = hash2_signed(k, i.wrapping_add(1), semente);
    let fade = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    2.0 * (g0 * f * (1.0 - fade) + g1 * (f - 1.0) * fade)
}

#[cfg(test)]
#[path = "bristles_fios_tests.rs"]
mod tests;
