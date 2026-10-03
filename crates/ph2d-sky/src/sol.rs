//! ⭐⭐⭐ **O SOL DO CÉU FOTOGRÁFICO** — o disco que o panorama traz, tirado dele e devolvido como uma
//! luz DIRECCIONAL com raio (a que faz sombra).
//!
//! ⭐ **Conservação por construção:** cada texel do disco acima do [`LIMIAR`] é preso ao limiar (com a
//! cor dele) e o EXCESSO vai para o sol — `céu sem sol + excesso = o panorama`, texel a texel. O sol é
//! esse excesso como uma calote uniforme com a MESMA energia, a MESMA direcção (o centróide) e o
//! mesmo 2.º momento angular.
//!
//! ⭐ **Só o pico e os vizinhos dele** (8-vizinhança, `x` a dar a volta): um céu de várias lâmpadas
//! (interior, estúdio, noite) dá a MAIS FORTE como sol, e as outras ficam no céu — a luz principal
//! com sombra mais o ambiente, como num motor de jogo.
//!
//! A tabela da calote ([`Sol::radiance`]) é a MESMA pergunta do atlas: a média sob o lóbulo GGX de
//! `α` com `N = V = R`, núcleo `K(l) = (r·l)·D(h)` normalizado pelo hemisfério inteiro.

use rayon::prelude::*;

use crate::{Panorama, Rgb};

/// ⭐ **O limiar do sol, em múltiplos da radiância média do panorama.**
///
/// Medido (03/10, `sol_tests::instrumento_sol`, os 8 embarcados): a fracção da energia que o excesso
/// leva fica PARADA de `16×` a `256×` nos céus com sol limpo — cidade `22,2 → 21,3 %`, nascer
/// `56,3 → 54,6 %`, pôr do sol `2,3 → 1,8 %` — e o 2.º momento também (`0,36°`, `0,48°`, `0,32°`).
/// A `8×` o halo do céu junta-se (cidade `0,36° → 1,32°`, nascer `0,48° → 1,09°`). ⇒ `16` é o joelho.
/// (O Eevee do Blender faz a mesma separação com um limiar ABSOLUTO de `10`; estes céus têm média
/// `0,5–1`, logo é a mesma ordem.)
pub const LIMIAR: f64 = 16.0;

/// ⭐ **Abaixo desta fracção da energia o céu NÃO tem sol** — o excesso fica nele e não há luz-chave.
///
/// Medido (03/10, `instrumento_sol`): o menor sol de verdade dos embarcados é o do pôr do sol,
/// `2,3 %`; o pátio nublado dá `0,1 %` (uma nuvem clara a `14°`, sem sombra que se veja). `1 %` fica
/// entre os dois.
pub const FRACAO_MIN: f64 = 0.01;

/// Amostras da tabela no eixo `√α`, de `0` a `1` (as do estúdio).
pub const RUGOSIDADES: usize = 49;
/// Amostras no eixo `√(1 − cos ψ)/√2`, de `0` a `1` — passo [`PASSO`] junto do eixo.
pub const ANGULOS: usize = 513;
/// Floats da tabela.
pub const TABELA: usize = RUGOSIDADES * ANGULOS;

/// O passo angular da tabela junto do eixo (radianos): `ψ ≈ √2·u` e `u` anda `√2/(ANGULOS−1)`.
pub const PASSO: f64 = 2.0 / (ANGULOS - 1) as f64;

/// ⚠️ **O bordo do disco tem DOIS passos da tabela** — a lei do bordo da caixa do estúdio: uma aresta
/// mais dura do que a tabela representa não é uma aresta, é o degrau da tabela a passar por uma.
pub const BORDO: f64 = 2.0 * PASSO;

/// ⭐⭐ **O sol** — no referencial do céu (sem giro, sem força).
#[derive(Clone, Debug, PartialEq)]
pub struct Sol {
    /// A direcção PARA o sol (unitária).
    pub dir: [f32; 3],
    /// O raio angular do disco uniforme com o mesmo 2.º momento (radianos).
    pub raio: f32,
    /// A radiância da calote: `radiancia · Ω_calote` = a energia do excesso.
    pub radiancia: Rgb,
    /// A energia do excesso, `Σ excesso·Ω` (a irradiância a incidência normal).
    pub energia: Rgb,
    /// Que fracção da energia do panorama o sol leva (na luma).
    pub fracao: f32,
    /// `RUGOSIDADES × ANGULOS`: a média da calote de radiância `1` sob o lóbulo.
    tabela: Vec<f32>,
}

fn luma(c: Rgb) -> f64 {
    0.2126 * f64::from(c[0]) + 0.7152 * f64::from(c[1]) + 0.0722 * f64::from(c[2])
}

/// O perfil do disco: `smoothstep` no COSSENO, entre `raio ± BORDO/2`.
#[derive(Clone, Copy, Debug)]
struct Perfil {
    dentro: f64,
    fora: f64,
    /// O raio do suporte (radianos).
    suporte: f64,
}

impl Perfil {
    fn de(raio: f64) -> Self {
        Self {
            dentro: (raio - 0.5 * BORDO).max(0.0).cos(),
            fora: (raio + 0.5 * BORDO).cos(),
            suporte: raio + 0.5 * BORDO,
        }
    }

    fn valor(&self, cos_psi: f64) -> f64 {
        let t = ((cos_psi - self.fora) / (self.dentro - self.fora)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// `∫ perfil dω` — integrado, nunca uma fórmula por forma.
    fn angulo_solido(&self) -> f64 {
        const N: usize = 4096;
        let passo = self.suporte / N as f64;
        (0..N)
            .map(|i| {
                let r = (i as f64 + 0.5) * passo;
                self.valor(r.cos()) * r.sin()
            })
            .sum::<f64>()
            * std::f64::consts::TAU
            * passo
    }
}

/// ⭐ **A normalização do núcleo**: `W(α) = ∫_{r·l>0} (r·l)/den² dω`, `den = (r·h)²(α²−1)+1`.
/// Em forma fechada (com `t = 1 − r·l`, `b = (1−α²)/2`, `x = b/α²`): `W = 2π·(x − ln(1+x))/(x·α²)²`.
fn normalizacao(a2: f64) -> f64 {
    let b = (1.0 - a2).max(0.0) * 0.5;
    let x = b / a2;
    if x < 1.0e-2 {
        // (x − ln(1+x))/x² em série (a forma fechada perde-se no cancelamento); `b² = x²·α⁴`.
        let g = 0.5 - x * (1.0 / 3.0 - x * (0.25 - x * 0.2));
        std::f64::consts::TAU * g / (a2 * a2)
    } else {
        std::f64::consts::TAU * (x - x.ln_1p()) / (b * b)
    }
}

fn nucleo(rl: f64, a2: f64) -> f64 {
    if rl <= 0.0 {
        return 0.0;
    }
    let nh2 = (1.0 + rl) * 0.5;
    let den = nh2 * (a2 - 1.0) + 1.0;
    rl / (den * den)
}

/// O eixo a `ψ` do sol: a direcção no plano `x-z` do referencial em que o sol é `+z`.
fn eixo(ai: usize) -> (f64, f64) {
    let u = std::f64::consts::SQRT_2 * ai as f64 / (ANGULOS - 1) as f64;
    let c = (1.0 - u * u).clamp(-1.0, 1.0);
    (c, (1.0 - c * c).max(0.0).sqrt())
}

/// ⭐ A célula `(α, ψ)` pela QUADRATURA SOBRE A CALOTE — válida quando o lóbulo é largo face às
/// células (`α ≳ suporte/6`).
fn pela_calote(p: &Perfil, a2: f64, ai: usize) -> f64 {
    const NR: usize = 32;
    const NF: usize = 64;
    let (c, s) = eixo(ai);
    let (dr, df) = (p.suporte / NR as f64, std::f64::consts::TAU / NF as f64);
    let mut soma = 0.0;
    for i in 0..NR {
        let rho = (i as f64 + 0.5) * dr;
        let (sr, cr) = rho.sin_cos();
        let w = p.valor(cr) * sr * dr * df;
        if w <= 0.0 {
            continue;
        }
        for j in 0..NF {
            let phi = (j as f64 + 0.5) * df;
            // l = (sr cosφ, sr sinφ, cr) com o sol em +z; r = (s, 0, c).
            let rl = s * sr * phi.cos() + c * cr;
            soma += w * nucleo(rl, a2);
        }
    }
    soma / normalizacao(a2)
}

/// ⭐ A célula `(α, ψ)` pela AMOSTRAGEM DO LÓBULO (Hammersley, o ratio da `BoxPrefilter` do estúdio)
/// — válida quando o lóbulo é estreito face ao bordo.
fn pelo_lobo(p: &Perfil, a2: f64, ai: usize) -> f64 {
    const N: u32 = 1024;
    let (c, s) = eixo(ai);
    let (mut soma, mut peso) = (0.0, 0.0);
    for i in 0..N {
        let u1 = (f64::from(i) + 0.5) / f64::from(N);
        let u2 = f64::from(i.reverse_bits()) / f64::from(u32::MAX);
        let ch2 = (1.0 - u1) / (1.0 + (a2 - 1.0) * u1);
        let ch = ch2.max(0.0).sqrt();
        let sh = (1.0 - ch2).max(0.0).sqrt();
        let phi = std::f64::consts::TAU * u2;
        let (hx, hz) = (sh * phi.cos(), ch);
        // l no referencial de r (= +z): l = 2(r·h)h − r.
        let (lx, lz) = (2.0 * ch * hx, 2.0 * ch * hz - 1.0);
        if lz <= 0.0 {
            continue;
        }
        // Para o referencial do sol: r = (s, 0, c), a base de r é (c, 0, −s), (0, 1, 0), r.
        let cos_sol = -s * lx + c * lz;
        soma += lz * p.valor(cos_sol);
        peso += lz;
    }
    soma / peso.max(1.0e-300)
}

/// ⭐ O `α` a partir do qual a tabela é a quadratura sobre a calote.
fn alfa_da_calote(p: &Perfil) -> f64 {
    (0.5 * BORDO).max(p.suporte / 6.0)
}

/// ⚠️ **Longe do disco a cauda é da CALOTE, mesmo com o lóbulo estreito** — medido (03/10, disco de
/// `5°`, `α = 0,004`): `1024` amostras do lóbulo põem `~2,5` além de `9°`, e a célula ali erra
/// `±10–17 %`; somada sobre a área da cauda dava `+1,2 %` de energia. Fora de
/// `suporte·1,25 + 4α` o núcleo é liso à escala das células da calote (`suporte/32`).
fn perto(p: &Perfil, a: f64, ai: usize) -> bool {
    eixo(ai).0.acos() < 1.25 * p.suporte + 4.0 * a
}

fn tabela(p: &Perfil) -> Vec<f32> {
    let corte = alfa_da_calote(p);
    (0..RUGOSIDADES)
        .into_par_iter()
        .flat_map_iter(|ri| {
            let r = ri as f64 / (RUGOSIDADES - 1) as f64;
            let a = r * r;
            let a2 = a * a;
            (0..ANGULOS).map(move |ai| {
                (if ri == 0 {
                    p.valor(eixo(ai).0)
                } else if a < corte && perto(p, a, ai) {
                    pelo_lobo(p, a2, ai)
                } else {
                    pela_calote(p, a2, ai)
                }) as f32
            })
        })
        .collect()
}

impl Sol {
    /// ⭐ A média da calote de radiância `1` sob o lóbulo GGX de `α`, na direcção a `cos ψ` do sol —
    /// bilinear em (`√α`, ângulo). ⚠️ A MESMA ordem de contas do `sky_sol_tabela` do WGSL.
    #[must_use]
    pub fn tabela_em(&self, alpha: f32, cos_psi: f32) -> f32 {
        let r = alpha.clamp(0.0, 1.0).sqrt() * (RUGOSIDADES - 1) as f32;
        let a = (1.0 - cos_psi.clamp(-1.0, 1.0)).max(0.0).sqrt() / core::f32::consts::SQRT_2
            * (ANGULOS - 1) as f32;
        let (r0, a0) = ((r as usize).min(RUGOSIDADES - 2), (a as usize).min(ANGULOS - 2));
        let (fr, fa) = (r - r0 as f32, a - a0 as f32);
        let l = |ri: usize, ai: usize| self.tabela[ri * ANGULOS + ai];
        let baixo = l(r0, a0) + (l(r0, a0 + 1) - l(r0, a0)) * fa;
        let cima = l(r0 + 1, a0) + (l(r0 + 1, a0 + 1) - l(r0 + 1, a0)) * fa;
        baixo + (cima - baixo) * fr
    }

    /// A tabela, para quem a sobe (`RUGOSIDADES × ANGULOS`, linha a linha).
    #[must_use]
    pub fn tabela(&self) -> &[f32] {
        &self.tabela
    }

    /// ⭐ A radiância do sol pré-filtrada pela direcção `dir` (referencial do céu).
    #[must_use]
    pub fn radiance(&self, dir: [f32; 3], alpha: f32) -> Rgb {
        let t = self.tabela_em(alpha, cosseno(dir, self.dir));
        self.radiancia.map(|c| c * t)
    }

    /// ⭐ A irradiância normalizada `E(n)/π` do sol — a linha `α = 1` (a mesma lei do atlas: com
    /// `α = 1` o núcleo é o cosseno).
    #[must_use]
    pub fn irradiance(&self, n: [f32; 3]) -> Rgb {
        self.radiance(n, 1.0)
    }
}

fn cosseno(a: [f32; 3], b: [f32; 3]) -> f32 {
    let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1.0e-20);
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / la
}

impl Panorama {
    /// ⭐⭐ **Tira o sol** — devolve o céu sem ele e o sol (`None` quando nenhum texel passa do
    /// [`LIMIAR`]: o céu fica intacto).
    #[must_use]
    pub fn separa_sol(&self) -> (Self, Option<Sol>) {
        let media = luma(self.media());
        let limiar = LIMIAR * media;
        let (w, h) = (self.largura as usize, self.altura as usize);
        let lum: Vec<f64> = self.rgb.iter().map(|c| luma(*c)).collect();
        let Some((pico, &lp)) = lum.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)) else {
            return (self.clone(), None);
        };
        if !(lp > limiar) || !(media > 0.0) {
            return (self.clone(), None);
        }
        // O componente do pico (8-vizinhança, `x` dá a volta).
        let mut dentro = vec![false; w * h];
        let mut pilha = vec![pico];
        dentro[pico] = true;
        let mut membros = Vec::new();
        while let Some(i) = pilha.pop() {
            membros.push(i);
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            for dy in -1..=1i64 {
                for dx in -1..=1i64 {
                    let yy = y + dy;
                    if yy < 0 || yy >= h as i64 {
                        continue;
                    }
                    let xx = (x + dx).rem_euclid(w as i64);
                    let j = yy as usize * w + xx as usize;
                    if !dentro[j] && lum[j] > limiar {
                        dentro[j] = true;
                        pilha.push(j);
                    }
                }
            }
        }
        let mut sem = self.clone();
        let (mut energia, mut centro, mut peso) = ([0.0f64; 3], [0.0f64; 3], 0.0f64);
        for &i in &membros {
            let (x, y) = ((i % w) as u32, (i / w) as u32);
            let om = self.angulo_solido(y);
            let fica = (limiar / lum[i]) as f32;
            let c = self.rgb[i];
            sem.rgb[i] = c.map(|v| v * fica);
            let ex = c.map(|v| f64::from(v * (1.0 - fica)));
            let we = (lum[i] - limiar) * om;
            let d = self.direcao(x, y).map(f64::from);
            for k in 0..3 {
                energia[k] += ex[k] * om;
                centro[k] += we * d[k];
            }
            peso += we;
        }
        let fracao = luma(energia.map(|v| v as f32)) / (media * 4.0 * std::f64::consts::PI);
        if fracao < FRACAO_MIN {
            return (self.clone(), None);
        }
        let lc = (centro[0] * centro[0] + centro[1] * centro[1] + centro[2] * centro[2]).sqrt();
        let dir = centro.map(|v| v / lc);
        // O 2.º momento, com o do PRÓPRIO texel (um quadrado de lado Δ tem Δ²/12 por eixo): um sol de
        // um texel tem o tamanho do texel, não zero.
        let pi = std::f64::consts::PI;
        let mut m2 = 0.0f64;
        for &i in &membros {
            let (x, y) = ((i % w) as u32, (i / w) as u32);
            let we = (lum[i] - limiar) * self.angulo_solido(y);
            let d = self.direcao(x, y).map(f64::from);
            let psi = (d[0] * dir[0] + d[1] * dir[1] + d[2] * dir[2]).clamp(-1.0, 1.0).acos();
            let dy = pi / h as f64;
            let dx = std::f64::consts::TAU / w as f64 * (1.0 - d[1] * d[1]).max(0.0).sqrt();
            m2 += we * (psi * psi + (dx * dx + dy * dy) / 12.0);
        }
        let raio = (2.0 * m2 / peso).sqrt();
        let perfil = Perfil::de(raio);
        let om = perfil.angulo_solido();
        let sol = Sol {
            dir: dir.map(|v| v as f32),
            raio: raio as f32,
            radiancia: energia.map(|v| (v / om) as f32),
            energia: energia.map(|v| v as f32),
            fracao: fracao as f32,
            tabela: tabela(&perfil),
        };
        (sem, Some(sol))
    }
}

#[cfg(test)]
#[path = "sol_tests.rs"]
mod tests;
