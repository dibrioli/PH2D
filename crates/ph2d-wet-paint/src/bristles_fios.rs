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
const SEMENTE_PONTA: u32 = 0x6f69_0007;
const SEMENTE_VEIO: u32 = 0x6f69_0008;

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
    /// ⭐ **O VEIO** (J4 óleo, [`crate::tuning::Knob::BristleStreak`]): o quanto a luminosidade HSLuv de um fio DESCE nos
    /// troços em que ele pesa e SOBE nos troços em que falta (`[escuro, claro]`, fracções de 100; `[0, 0]` = nenhum). Os
    /// troços são RAROS e têm princípio e fim — o ruído do fio acima de [`Self::veio_corte`] — e o resto do fio fica
    /// na sua cor: o «aparece e some» do Rebelle, contra o ruído contínuo de [`Self::cor`] que tinge o fio inteiro.
    pub veio: [f64; 2],
    /// O comprimento de caminho (px) em que o veio de um fio muda.
    pub veio_px: f64,
    /// O limiar do `|ruído|` do fio a partir do qual o troço pesa (ruído > 0) ou falta (< 0).
    pub veio_corte: f64,
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
        veio: [0.0; 2],
        veio_px: 300.0,
        veio_corte: 0.5,
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

    /// O ruído de cor (ou o veio) age.
    #[must_use]
    pub fn tem_cor(&self) -> bool {
        self.cor != [0.0; 3] || self.veio != [0.0; 2]
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

/// O que é de UM fio num dab ([`FiosNoDab::tabela`]).
#[derive(Clone, Copy, Debug)]
struct DoFio {
    span: f64,
    ganho: f64,
    cor: [f64; 3],
}

/// A tabela de um dab: o que é de cada fio da faixa `k0..`, calculado uma vez ([`FiosNoDab::tabela`]).
#[derive(Clone, Debug)]
pub struct TabelaDosFios {
    k0: i32,
    v: Vec<DoFio>,
}

impl TabelaDosFios {
    /// Os fios da tabela: `(primeiro, último)`.
    #[must_use]
    pub fn faixa(&self) -> (i32, i32) {
        (self.k0, self.k0 + self.v.len() as i32 - 1)
    }

    fn do_fio(&self, f: &FiosNoDab, k: i32) -> DoFio {
        usize::try_from(k - self.k0)
            .ok()
            .and_then(|i| self.v.get(i).copied())
            .unwrap_or_else(|| f.do_fio(k))
    }
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
        self.no_ponto_por(d, |k| (self.span_do_fio(k), self.ganho_do_fio(k)))
    }

    /// [`Self::no_ponto`] com o que é do FIO lido da tabela `t` do dab ([`Self::tabela`]) — a mesma conta, ao bit.
    #[must_use]
    pub fn no_ponto_em(&self, d: [f64; 2], t: &TabelaDosFios) -> Ponto {
        self.no_ponto_por(d, |k| {
            let x = t.do_fio(self, k);
            (x.span, x.ganho)
        })
    }

    fn no_ponto_por(&self, d: [f64; 2], do_fio: impl Fn(i32) -> (f64, f64)) -> Ponto {
        let f = &self.fios;
        let n = self.normal;
        let mut q = [d[0] - n[0] * self.desvio_px, d[1] - n[1] * self.desvio_px];
        let l = q[0] * n[0] + q[1] * n[1];
        let fio = self.fio(l / f.largura_px.max(0.25));
        let (span, ganho) = do_fio(fio);
        if f.span_px != 0.0 {
            q = [q[0] - n[0] * span, q[1] - n[1] * span];
        }
        Ponto { q, ganho, fio }
    }

    /// O que só depende do fio `fio` neste dab: o desvio do Random Span, o ganho, o ruído de cor.
    fn do_fio(&self, fio: i32) -> DoFio {
        DoFio {
            span: self.span_do_fio(fio),
            ganho: self.ganho_do_fio(fio),
            cor: self.ruido_de_cor(fio),
        }
    }

    /// O desvio (px) do Random Span do fio `fio` neste dab (`0` sem span).
    fn span_do_fio(&self, fio: i32) -> f64 {
        let f = &self.fios;
        if f.span_px == 0.0 {
            return 0.0;
        }
        f.span_px
            * ruido(
                self.arco / f.span_comprimento_px.max(1.0),
                fio,
                SEMENTE_SPAN,
            )
    }

    /// ⭐ **A tabela do dab** (auditoria do traço rápido, 18/10): o que é de cada FIO que um ponto a até `raio_px`
    /// do centro pode ler, calculado UMA vez por dab — por célula eram 4 Perlin (o span e o ganho) e, com cor,
    /// mais 6 e o HSLuv; o traço rápido do Oil Bristle a 103 px pagava 13,6 ms por quadro contra 8,7 sem os fios
    /// (doc 50_medicao §4). Fora da faixa, a conta directa.
    #[must_use]
    pub fn tabela(&self, raio_px: f64) -> TabelaDosFios {
        let (k0, k1) = self.faixa(raio_px);
        TabelaDosFios {
            k0,
            v: (k0..=k1).map(|k| self.do_fio(k)).collect(),
        }
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

    /// A normal do traço neste dab (unitária) — a tangente é `[n₁, −n₀]`.
    #[must_use]
    pub fn normal(&self) -> [f64; 2] {
        self.normal
    }

    /// O fio do ponto a `d` px do centro do dab (o de [`Self::no_ponto`], sem a conta do ganho).
    #[must_use]
    pub fn fio_no_ponto(&self, d: [f64; 2]) -> i32 {
        let n = self.normal;
        let q = [d[0] - n[0] * self.desvio_px, d[1] - n[1] * self.desvio_px];
        self.fio((q[0] * n[0] + q[1] * n[1]) / self.fios.largura_px.max(0.25))
    }

    /// ⭐ **O comprimento do fio `fio`** (J2, 03_MOTORES §7 c — a ponta das cerdas): a fracção `1 + ρ·(1 − 2u)` do
    /// comprimento médio, `u` sorteado por fio (o mesmo o traço inteiro) — `ρ = 0` = todos iguais.
    #[must_use]
    pub fn comprimento_do_fio(fio: i32, rho: f64) -> f64 {
        1.0 + rho * (1.0 - 2.0 * crate::rng::hash2(fio, 0, SEMENTE_PONTA))
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
        if !self.fios.tem_cor() {
            return rgb;
        }
        crate::bristles_cor::com_ruido(rgb, self.ruido_de_cor(fio))
    }

    /// [`Self::cor_do_fio`] com o ruído do fio lido da tabela `t` do dab — a mesma conta, ao bit.
    #[must_use]
    pub fn cor_do_fio_em(&self, rgb: [f32; 3], fio: i32, t: &TabelaDosFios) -> [f32; 3] {
        if !self.fios.tem_cor() {
            return rgb;
        }
        crate::bristles_cor::com_ruido(rgb, t.do_fio(self, fio).cor)
    }

    /// O ruído HSLuv do fio `fio` (matiz, saturação, luminosidade), que anda ao longo do caminho — com o veio na
    /// luminosidade.
    fn ruido_de_cor(&self, fio: i32) -> [f64; 3] {
        let c = self.fios.cor;
        let veio = self.veio_do_fio(fio);
        if c == [0.0; 3] {
            return [0.0, 0.0, veio];
        }
        let t = self.arco / self.fios.ruido_px.max(1.0);
        let r = |i: u32| ruido(t, fio, SEMENTE_COR.wrapping_add(i));
        [c[0] * r(0), c[1] * r(1), c[2] * r(2) + veio]
    }

    /// ⭐ **O veio do fio `fio` neste dab** ([`Fios::veio`]): o troço pesa (luminosidade − escuro) onde o ruído do fio passa
    /// o limiar para cima e falta (+ claro) onde passa para baixo; entra a meio caminho entre o limiar e o tecto
    /// (`(|n| − corte) ÷ (1 − corte)`, ×2, preso a 1) — o fio chega à cor do troço e volta, sem degrau. `0` sem veio.
    #[must_use]
    pub fn veio_do_fio(&self, fio: i32) -> f64 {
        let f = &self.fios;
        if f.veio == [0.0; 2] {
            return 0.0;
        }
        let n = ruido(self.arco / f.veio_px.max(1.0), fio, SEMENTE_VEIO);
        let corte = f.veio_corte.clamp(0.0, 0.95);
        let e = (2.0 * (n.abs() - corte) / (1.0 - corte)).clamp(0.0, 1.0);
        if n > 0.0 {
            -f.veio[0] * e
        } else {
            f.veio[1] * e
        }
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
