//! **Smooth Edges do Wet Paint** (dono 09/10; a INTENSIDADE desde 10/10; **do TRAÇO desde 04/11**: «cada pincel deve
//! ter o seu, assim não perde detalhes»). O valor é o RAIO de um alisamento da MANCHA (`pa ≥ SS0`): [`Suave`], um por
//! raio presente na região ([`Suaves`]), e cada pixel lê o da tinta dele. O miolo longe da borda e o papel aberto
//! ficam iguais ao byte; só a orla muda (forma mais lisa, borda com meio-tom). Nenhuma água nem pigmento se move: é
//! aparência, como o Paper. `0` = o composite de antes, ao byte. O 1.º Smooth Edges (o 3×3 do Watercolor,
//! `aa_coverage`) era fraco demais no máximo (smoke do dono 10/10) — o porquê e as medições: 05 §4b W1g.

use super::grid_map::{SampleU, SampleURow};
use crate::tool::paint::watercolor_render::SS0;
use rayon::prelude::*;

/// O pigmento de um pixel na razão 1 (`o` = o offset do pixel no plano), em premultiplicado normalizado
/// (`pc·pa`) e o `pa` — a forma em que o `over` do composite o consome.
#[inline]
pub(super) fn premul_da_celula(pigment: &[u8], o: usize) -> ([f32; 3], f32) {
    let a = pigment[o + 3] as f32 / 255.0;
    (
        [
            pigment[o] as f32 * a,
            pigment[o + 1] as f32 * a,
            pigment[o + 2] as f32 * a,
        ],
        a,
    )
}

/// O mesmo acima da razão 1: o amostrador devolve `Σ c·a·w` e `Σ a·w` em 0..255; dividir por 255 dá
/// exatamente `pc · pa`, sem passar pela cor straight.
#[inline]
pub(super) fn premul_amostrado(s: [f64; 4]) -> ([f32; 3], f32) {
    (
        [
            (s[0] / 255.0) as f32,
            (s[1] / 255.0) as f32,
            (s[2] / 255.0) as f32,
        ],
        (s[3] / 255.0) as f32,
    )
}

/// O pigmento do pixel `(px, py)` por qualquer razão — a porta das duas acima.
#[inline]
pub(super) fn premul_do_pixel(
    sampler: SampleU,
    srow: Option<SampleURow>,
    pigment: &[u8],
    stride: usize,
    px: usize,
    py: usize,
) -> ([f32; 3], f32) {
    if sampler.is_identity() {
        premul_da_celula(pigment, py * stride + px * 4)
    } else {
        let s = match srow {
            Some(r) => r.at(pigment, px),
            None => sampler.row(py).at(pigment, px),
        };
        premul_amostrado(s)
    }
}

/// O raio do alisamento no máximo do valor, px (a constante é do motor: o relevo lê-a também).
pub(super) const RAIO_MAX: f32 = ph2d_wet_paint::grid::aparencia::SUAVE_RAIO_MAX_PX;

/// (Testes) `true` = a lei de ANTES da D1 W3b: com o Óleo Real o Smooth Edges só amacia a cor (o `Suave` com a
/// orla), o campo não o vê — a variante «antes» da folha e o controlo da prova de mutação, no MESMO processo.
#[cfg(test)]
pub(in crate::tool::paint) mod so_na_cor {
    use std::cell::Cell;

    thread_local! {
        static D: Cell<bool> = const { Cell::new(false) };
    }

    pub(in crate::tool::paint) fn set(v: bool) {
        D.with(|c| c.set(v));
    }

    pub(in crate::tool::paint) fn get() -> bool {
        D.with(Cell::get)
    }
}

/// (Testes) `Some(r)` = a lei de ANTES do dono 04/11, um só [`Suave`] de raio `r` na região inteira: a variante «antes»
/// da régua de custo, no MESMO processo.
#[cfg(test)]
pub(in crate::tool::paint) mod raio_escalar {
    use std::cell::Cell;

    thread_local! {
        static D: Cell<Option<f32>> = const { Cell::new(None) };
    }

    pub(in crate::tool::paint) fn set(v: Option<f32>) {
        D.with(|c| c.set(v));
    }

    pub(in crate::tool::paint) fn get() -> Option<f32> {
        D.with(Cell::get)
    }
}

/// (Testes) `true` = o composite sem o [`Suave`] dos píxeis (o campo do óleo continua a amaciar): a régua de que a
/// orla da tinta com o Óleo Real é a do campo.
#[cfg(test)]
pub(in crate::tool::paint) mod sem_suave {
    use std::cell::Cell;

    thread_local! {
        static D: Cell<bool> = const { Cell::new(false) };
    }

    pub(in crate::tool::paint) fn set(v: bool) {
        D.with(|c| c.set(v));
    }

    pub(in crate::tool::paint) fn get() -> bool {
        D.with(Cell::get)
    }
}

/// Quantos px à volta de uma mudança um pixel do Smooth Edges pode mudar com `raio` — a tenda tem meia
/// largura `raio`, e a orla anti-serrilhada mais um.
#[must_use]
pub(super) fn alcance(raio: f32) -> usize {
    raio.ceil() as usize + 1
}

/// Quantos px à volta de uma tinta que mudou um pixel pode mudar: dentro da tinta ele lê a tinta desfocada (a tenda)
/// a `raio` px de cada lado, logo o alcance dobra.
#[must_use]
pub(super) fn alcance_da_mudanca(raio: f32) -> usize {
    alcance(raio) * 2
}

/// Uma passada de caixa de meia largura FRACIONÁRIA `h` (as pontas `⌊h⌋ + 1` pesam `h − ⌊h⌋`), lida em
/// `ler(i)` com o índice preso a `0..n` (a borda repete-se, como na tela).
#[inline]
fn caixa(n: usize, h: f32, i: usize, ler: impl Fn(usize) -> f32) -> f32 {
    let (k, f) = (h.floor() as isize, h - h.floor());
    let em = |d: isize| ler((i as isize + d).clamp(0, n as isize - 1) as usize);
    let mut s = 0.0;
    for d in -k..=k {
        s += em(d);
    }
    if f > 0.0 {
        s += f * (em(-k - 1) + em(k + 1));
    }
    s / (2.0 * k as f32 + 1.0 + 2.0 * f)
}

/// [`caixa`] para os quatro canais da tinta de uma vez.
#[inline]
fn caixa4(n: usize, h: f32, i: usize, ler: impl Fn(usize) -> [f32; 4]) -> [f32; 4] {
    let (k, f) = (h.floor() as isize, h - h.floor());
    let em = |d: isize| ler((i as isize + d).clamp(0, n as isize - 1) as usize);
    let mut s = [0.0f32; 4];
    for d in -k..=k {
        let v = em(d);
        for c in 0..4 {
            s[c] += v[c];
        }
    }
    if f > 0.0 {
        let (a, b) = (em(-k - 1), em(k + 1));
        for c in 0..4 {
            s[c] += f * (a[c] + b[c]);
        }
    }
    let n = 2.0 * k as f32 + 1.0 + 2.0 * f;
    s.map(|v| v / n)
}

/// O degrau de cor DENTRO da tinta (a altura, em `0..1` por canal premultiplicado, entre a tinta
/// desfocada a `raio` px de cada lado) a partir do qual a borda interior amacia, e por inteiro. Medido
/// (dono 10/10, 4.º smoke; a borda de um azul sobre um vermelho · o S aprovado, píxeis que mudam > 8
/// níveis): 0,04 → −48 % · 10 679 px; 0,10 → −42 % · 2 639; **0,15 → −35 % · 512**; 0,20 → −27 % · 109.
/// Abaixo ficam as células da aguada e o grão; acima, o degrau entre duas tintas e a tinta levantada.
pub(super) const DEGRAU_MIN: f32 = 0.15;
const DEGRAU_CHEIO: f32 = 0.40;

/// A LINHA COMPRIDA dentro da tinta (≈ contraste × largura, em `0..1` por canal) a partir da qual amacia, e
/// por inteiro (5.º smoke do dono, 10/10: as linhas escuras da Heavy Edge sobre a tinta). Medido (as linhas
/// de uma aguada azul com a Heavy Edge a escorrer sobre um vermelho · píxeis do S aprovado > 8 níveis):
/// sem linha −1 % · 512; 0,10 −31 % · 4 480 (a olho igual); 0,15 −18 % · 3 032; 0,25 −6 % · 2 019. Sem o
/// critério de COMPRIDA, 0,15 dava −37 % mas 6 713 (as células da aguada contavam como linhas).
const LINHA_MIN: f32 = 0.10;
const LINHA_CHEIA: f32 = 0.20;

/// **O Smooth Edges de uma região do composite** (dono 10/10: «não vi benefício no máximo»): uma BORDA
/// SUAVE de largura ajustável. A mancha (`pa ≥ SS0`) desfocada por uma tenda de meia largura `raio` (duas
/// caixas de `raio/2`, separáveis) diz onde fica a orla: `B ∈ (0, 1)`. Nessa faixa o pixel mostra a tinta
/// (cor e cobertura, premultiplicadas) desfocada pela MESMA tenda, com peso `4·B·(1 − B)` — inteiro na
/// orla, zero no miolo e no papel aberto (o caminho de antes). O contorno escuro e a silhueta amaciam
/// juntos e a tinta conserva-se. **Dentro** da tinta (onde a mancha é toda 1 — a borda de um traço sobre
/// outra cor, a tinta levantada) a borda é um DEGRAU de cor: a tinta desfocada a `raio` px de cada lado
/// mede a altura do degrau (no grão os dois lados são iguais), o peso vai de 0 a 1 entre [`DEGRAU_MIN`] e
/// [`DEGRAU_CHEIO`] e é um sino na posição dentro do degrau (dono 10/10, 4.º smoke). `raio = 0` ⇒ `B` é a própria mancha ⇒ peso 0 ⇒ o desligado, ao byte.
pub(super) struct Suave {
    x0: usize,
    y0: usize,
    rw: usize,
    w: usize,
    h: usize,
    b: Vec<f32>,
    raio: f32,
    /// As linhas que se procuram: as que correm na direção da GRAVIDADE (a Heavy Edge, a tinta levantada,
    /// os fios) — `Some(true)` verticais, `Some(false)` horizontais, `None` com a tela direita. As listras
    /// das cerdas seguem o traço em qualquer direção e não são bordas (medido: o miolo de um traço
    /// horizontal mudava −26 % com as linhas em todas as direções).
    linhas: Option<bool>,
    /// A tinta da região (premultiplicada + `pa`) desfocada pela MESMA tenda que a mancha, nas linhas a
    /// `alcance + 1` da tinta (as que [`Suave::aplica`] lê, com o vizinho do declive); o resto não se lê.
    pbar: Vec<[f32; 4]>,
    /// `false` = a SILHUETA já veio amaciada do campo do Óleo Real (`oleo_real::CampoAltura`, D1 W3b: o relevo e a
    /// cobertura seguem o mesmo contorno): aqui só os degraus e as linhas DENTRO da tinta, e o píxel SEM tinta do campo
    /// (`pa < SS0`: a metade de fora da tenda, os vãos da orla rasgada) pela tinta desfocada.
    orla: bool,
}

impl Suave {
    /// Monta a mancha e o alisado da região `(x0, y0, x1, y1)` (px) com a margem que o raio lê.
    pub(super) fn nova(
        sampler: SampleU,
        (pigment, stride): (&[u8], usize),
        (w, h): (usize, usize),
        (x0, y0, x1, y1): (usize, usize, usize, usize),
        raio: f32,
        linhas: Option<bool>,
        orla: bool,
    ) -> Self {
        let a = alcance_da_mudanca(raio);
        let (rx0, ry0) = (x0.saturating_sub(a), y0.saturating_sub(a));
        let (rx1, ry1) = ((x1 + a).min(w), (y1 + a).min(h));
        let (rw, rh) = (rx1 - rx0, ry1 - ry0);
        let mut m = vec![false; rw * rh];
        m.par_chunks_mut(rw).enumerate().for_each(|(k, linha)| {
            let y = ry0 + k;
            let r = (!sampler.is_identity()).then(|| sampler.row(y));
            for (v, x) in linha.iter_mut().zip(rx0..rx1) {
                *v = premul_do_pixel(sampler, r, pigment, stride, x, y).1 >= SS0;
            }
        });
        let hb = raio * 0.5;
        // As passadas só onde há tinta perto: longe dela o desfocado é EXACTAMENTE zero (a mancha é
        // zero em toda a janela), e a maior parte de uma tela grande está longe da tinta.
        let tem: Vec<bool> = m.par_chunks(rw).map(|l| l.contains(&true)).collect();
        let mut acc = vec![0u32; rh + 1];
        for y in 0..rh {
            acc[y + 1] = acc[y] + u32::from(tem[y]);
        }
        let al = alcance(raio);
        let perto: Vec<bool> = (0..rh)
            .map(|y| acc[(y + al + 1).min(rh)] > acc[y.saturating_sub(al)])
            .collect();
        let mut pre = vec![[0.0f32; 4]; rw * rh];
        pre.par_chunks_mut(rw).enumerate().for_each(|(k, linha)| {
            if acc[(k + 2 * al + 1).min(rh)] == acc[k.saturating_sub(2 * al)] {
                return;
            }
            let y = ry0 + k;
            let r = (!sampler.is_identity()).then(|| sampler.row(y));
            for (v, x) in linha.iter_mut().zip(rx0..rx1) {
                let (p, pa) = premul_do_pixel(sampler, r, pigment, stride, x, y);
                *v = [p[0], p[1], p[2], pa];
            }
        });
        let mut t = vec![0.0f32; rw * rh];
        let mut b = vec![0.0f32; rw * rh];
        // Duas caixas horizontais (m → t → b), depois duas verticais (b → t → b); o resultado fica em `b`.
        t.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !tem[y] {
                return;
            }
            let fonte = &m[y * rw..(y + 1) * rw];
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rw, hb, x, |i| f32::from(u8::from(fonte[i])));
            }
        });
        b.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !tem[y] {
                return;
            }
            let fonte = &t[y * rw..(y + 1) * rw];
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rw, hb, x, |i| fonte[i]);
            }
        });
        t.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !perto[y] {
                return;
            }
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rh, hb, y, |j| b[j * rw + x]);
            }
        });
        b.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if !perto[y] {
                return;
            }
            for (x, v) in l.iter_mut().enumerate() {
                *v = caixa(rh, hb, y, |j| t[j * rw + x]);
            }
        });
        // A tinta desfocada: duas caixas horizontais nas linhas onde a tinta foi lida (pre → tmp → pre) e
        // duas verticais nas que se leem (pre → tmp → pre).
        let lida = |y: usize| acc[(y + 2 * al + 1).min(rh)] > acc[y.saturating_sub(2 * al)];
        let lida_v = |y: usize| acc[(y + al + 2).min(rh)] > acc[y.saturating_sub(al + 1)];
        let mut tmp = vec![[0.0f32; 4]; rw * rh];
        tmp.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida(y) {
                let fonte = &pre[y * rw..(y + 1) * rw];
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rw, hb, x, |i| fonte[i]);
                }
            }
        });
        pre.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida(y) {
                let fonte = &tmp[y * rw..(y + 1) * rw];
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rw, hb, x, |i| fonte[i]);
                }
            }
        });
        tmp.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida_v(y) {
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rh, hb, y, |j| pre[j * rw + x]);
                }
            }
        });
        pre.par_chunks_mut(rw).enumerate().for_each(|(y, l)| {
            if lida_v(y) {
                for (x, v) in l.iter_mut().enumerate() {
                    *v = caixa4(rh, hb, y, |j| tmp[j * rw + x]);
                }
            }
        });
        Suave {
            x0: rx0,
            y0: ry0,
            rw,
            w,
            h,
            b,
            raio,
            linhas,
            pbar: pre,
            orla,
        }
    }

    /// O pigmento que o pixel `(px, py)` mostra: `dele` longe de qualquer borda (o caminho de antes); na
    /// orla com o papel e nos degraus de cor dentro da tinta, a mistura dele com a tinta desfocada.
    pub(super) fn aplica(&self, px: usize, py: usize, dele: ([f32; 3], f32)) -> ([f32; 3], f32) {
        self.aplica_com(px, py, dele, self.orla)
    }

    /// [`Self::aplica`] com a orla escolhida POR PÍXEL (dono 28/10: numa tela com óleo, a tinta sem óleo guarda a
    /// orla de sempre).
    pub(super) fn aplica_com(
        &self,
        px: usize,
        py: usize,
        dele: ([f32; 3], f32),
        orla: bool,
    ) -> ([f32; 3], f32) {
        let i = (py - self.y0) * self.rw + (px - self.x0);
        let b = self.b[i];
        let mut peso = if orla {
            4.0 * b * (1.0 - b)
        } else if dele.1 < SS0 {
            // A silhueta é do campo; o píxel SEM tinta dele (a metade de fora da tenda e os vãos sem pigmento da orla
            // rasgada) recebe a tinta desfocada — sem relevo: o relevo não sai do pigmento.
            (2.0 * b).min(1.0)
        } else {
            0.0
        };
        // Dentro da tinta: um DEGRAU de cor — a tinta desfocada a `r` px de cada lado do pixel (num
        // degrau os dois lados diferem; no grão, desfocado, são iguais) e a posição do pixel dentro dele
        // (`u`), com o peso em sino `4·u·(1−u)` como na orla com o papel.
        let dentro = if orla {
            (2.0 * b - 1.0).clamp(0.0, 1.0)
        } else if b >= 1.0 - 1e-6 {
            // A silhueta é do campo: só onde a tenda inteira é tinta.
            1.0
        } else {
            0.0
        };
        if dentro > 0.0 {
            let r = (self.raio.round() as usize).max(1);
            let altura = self.b.len() / self.rw;
            let q = |x: usize, y: usize| {
                self.pbar[(y.clamp(self.y0, self.y0 + altura - 1) - self.y0) * self.rw
                    + (x.clamp(self.x0, self.x0 + self.rw - 1) - self.x0)]
            };
            let meio = self.pbar[i];
            let mut melhor = 0.0f32;
            for (a, c) in [
                (q(px.saturating_sub(r), py), q((px + r).min(self.w - 1), py)),
                (q(px, py.saturating_sub(r)), q(px, (py + r).min(self.h - 1))),
            ] {
                for k in 0..4 {
                    let esc = if k < 3 { 1.0 / 255.0 } else { 1.0 };
                    let d = (c[k] - a[k]) * esc;
                    if d.abs() <= DEGRAU_MIN {
                        continue;
                    }
                    let u = (((meio[k] - a[k]) * esc) / d).clamp(0.0, 1.0);
                    let w = ((d.abs() - DEGRAU_MIN) / (DEGRAU_CHEIO - DEGRAU_MIN)).clamp(0.0, 1.0);
                    melhor = melhor.max(w * w * (3.0 - 2.0 * w) * 4.0 * u * (1.0 - u));
                }
            }
            // Uma LINHA COMPRIDA (os dois lados iguais, o meio diferente — a linha escura da Heavy Edge,
            // a tinta levantada, que descem com a gravidade): a tinta desfocada no pixel contra a média dos
            // lados a `r` px, × `(r + 1)` (o pico da tenda) ⇒ ≈ contraste × largura; e só se a mesma
            // linha continuar a `r` px ao LONGO dela (o mínimo dos três) — as células da aguada são curtas
            // e o grão, desfocado em 2D, fica `r + 1` vezes mais fraco.
            let crista = |x: usize, y: usize, horizontal: bool| {
                let (a, c) = if horizontal {
                    (q(x.saturating_sub(r), y), q((x + r).min(self.w - 1), y))
                } else {
                    (q(x, y.saturating_sub(r)), q(x, (y + r).min(self.h - 1)))
                };
                let m = q(x, y);
                (0..4)
                    .map(|k| {
                        let esc = if k < 3 { 1.0 / 255.0 } else { 1.0 };
                        ((m[k] - (a[k] + c[k]) * 0.5) * esc).abs()
                    })
                    .fold(0.0f32, f32::max)
                    * (self.raio + 1.0)
            };
            let (cima, baixo) = (py.saturating_sub(r), (py + r).min(self.h - 1));
            let (esq, dir) = (px.saturating_sub(r), (px + r).min(self.w - 1));
            let linha = match self.linhas {
                // Uma linha vertical: o corte é horizontal, e ela continua para cima e para baixo.
                Some(true) => crista(px, py, true)
                    .min(crista(px, cima, true))
                    .min(crista(px, baixo, true)),
                Some(false) => crista(px, py, false)
                    .min(crista(esq, py, false))
                    .min(crista(dir, py, false)),
                None => 0.0,
            };
            let wl = ((linha - LINHA_MIN) / (LINHA_CHEIA - LINHA_MIN)).clamp(0.0, 1.0);
            melhor = melhor.max(wl * wl * (3.0 - 2.0 * wl));
            peso = peso.max(melhor * dentro);
        }
        if peso < 1e-3 {
            return dele;
        }
        let s = self.pbar[i];
        let l = |d: f32, o: f32| d + (o - d) * peso;
        (
            [l(dele.0[0], s[0]), l(dele.0[1], s[1]), l(dele.0[2], s[2])],
            l(dele.1, s[3]),
        )
    }
}

mod suaves;
pub(super) use suaves::Suaves;

/// O CUSTO do Smooth Edges no composite (§0.0: medir antes de aceitar), as variantes no MESMO processo, intercaladas, o
/// MÍNIMO de cada: a tela inteira (o pior caso) e o retângulo de um traço (o quadro normal). As tintas: `0` sem o campo
/// (o caminho de antes), `0` com o campo alocado por outro controlo, `0,12` em tudo, e `0,5` coberto por um traço a `0`
/// (a mistura pela massa: o pior número de degraus). `cargo test … --lib medida_do_smooth_edges -- --ignored --nocapture`.
#[cfg(test)]
mod medida_tests;

/// **A lei, num campo desenhado à mão** (5.º smoke do dono, 10/10: «heavy edge nem sempre aceita a
/// influência de smooth edge» — as linhas escuras finas por dentro da tinta): um vermelho cheio com uma
/// LINHA roxa vertical de 2 px, uma faixa CLARA vertical (a tinta levantada) e listras HORIZONTAIS (as
/// cerdas de um traço deitado). Com a gravidade para baixo a linha e a faixa amaciam e as listras não;
/// com a tela direita nada muda. Mutação que sangra: as linhas a zero.
#[cfg(test)]
mod linhas_tests;

/// §BUG-bordas-W3b: os PLANOS de uma cena de traços horizontais do Dabber com o Óleo Real (o pigmento da grade, a
/// sombra no papel e a tela final) para achar o mecanismo dos pontinhos escuros da orla. `PH2D_OLEO_PNG=<pasta>`.
#[cfg(test)]
mod diag_w3b;
