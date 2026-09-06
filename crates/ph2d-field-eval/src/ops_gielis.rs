//! ⭐⭐⭐ **A SUPERFÓRMULA DE GIELIS** (W128) — seis números por curva, centenas de formas
//! orgânicas: folhas, conchas, flores, estrelas do mar.
//!
//! # ⭐⭐⭐ O achado que a torna implementável: ela é a MESMA estrutura da superquadrática
//!
//! O sólido de Gielis é o **produto esférico** de duas curvas planas: a *de cima* traça a secção e
//! a *de lado* o perfil,
//!
//! ```text
//! p = ( r₁(θ)cosθ · r₂(φ)cosφ ,  r₂(φ)sinφ ,  r₁(θ)sinθ · r₂(φ)cosφ )
//! ```
//!
//! e isso **não** é uma função radial: o ângulo polar do ponto não é `φ`, logo `‖p‖ − R(direcção)`
//! descreve outra superfície.
//!
//! ⭐ Mas escreva-se a **medida de Minkowski** de uma curva estrelada — `G(v) = ‖v‖ / r(ângulo de
//! v)`, o factor por que é preciso encolher `v` para ele cair na curva. Então, com `u = (x, z)`:
//!
//! ```text
//! G₁(u) = r₂(φ)cosφ        e        y = r₂(φ)sinφ
//! ```
//!
//! ⇒ o par `(G₁(u), y)` **é** um ponto da segunda curva ⇒ a superfície é exactamente
//! `G₂( G₁(u), y ) = 1`. *Duas medidas encaixadas* — a mesma forma da superquadrática (W127), com
//! as normas-`n` trocadas por estas.
//!
//! # ⭐⭐ E daí sai o divisor, outra vez sem varrer o espaço
//!
//! `g = G₂(G₁(u), y)` é positivamente **homogénea de grau 1** ⇒ `∇g` é homogénea de grau **zero**
//! ⇒ constante ao longo de cada raio ⇒ o máximo sobre a superfície **é** o máximo global.
//!
//! Com `∇G = (1/r)·e_ρ − (r'/r²)·e_α` (a conta de uma medida de curva estrelada):
//!
//! ```text
//! ‖∇g‖² = B_s(φ)² · ‖∇G₁‖²(θ) + B_y(φ)²
//! ```
//!
//! ⇒ **duas varreduras de UMA dimensão** ([`bound`]), e não uma malha `(θ, φ)`: primeiro o máximo em
//! `θ`, depois em `φ`. ⚠️ Isto é o que a torna viável, porque a árvore é recozida **a cada quadro**
//! — uma malha `2048 × 1024` custaria dezenas de milissegundos ali.
//!
//! # ⛔ A SIMETRIA é INTEIRA, e a razão é a costura
//!
//! O `atan2` corta o círculo em `θ = ±π`. A função `A(θ) = |cos(mθ/4)|^n2 + |sin(mθ/4)|^n3` é
//! **par**, logo o VALOR atravessa a costura; a **derivada** troca de sinal e deixa um vinco, a não
//! ser que ela seja zero ali. ⭐ Com o argumento deslocado de meia volta (`m(θ+π)/4`) a costura cai
//! em `α = 0` e `α = mπ/2`, onde `A = 1` e `A' = 0` — **e as duas só coincidem se `m` for inteiro**.
//! *Um `m` fraccionário não faz uma forma nova: faz uma peça rachada de lado a lado.*

use fidget::context::Tree;

/// A regularização que tira o `NaN` do gradiente na origem — a mesma razão e o mesmo valor do
/// [`crate::ops_super`], onde ela está medida.
const EPS: f64 = 1.0e-12;

/// Os quatro números de uma das duas curvas.
#[derive(Clone, Copy, Debug)]
pub struct Curve {
    /// Quantos lobos — **inteiro**, ver o cabeçalho.
    pub symmetry: f64,
    /// O expoente de fora: `r = A^(−1/n1)`. Baixo exagera os lobos, alto achata-os.
    pub n1: f64,
    /// O expoente do braço do cosseno.
    pub n2: f64,
    /// O do braço do seno. ⚠️ Diferente do `n2` ⇒ lobos **assimétricos**, que é metade do que esta
    /// família tem de bonito.
    pub n3: f64,
}

impl Curve {
    /// `A(α)` da fórmula, com o argumento **já deslocado**.
    fn a_of(self, theta: f64) -> f64 {
        let alpha = self.symmetry * (theta + std::f64::consts::PI) * 0.25;
        alpha.cos().abs().powf(self.n2) + alpha.sin().abs().powf(self.n3)
    }

    /// `dA/dθ`, em forma fechada.
    fn da_of(self, theta: f64) -> f64 {
        let alpha = self.symmetry * (theta + std::f64::consts::PI) * 0.25;
        let (s, c) = alpha.sin_cos();
        let d_alpha = self.symmetry * 0.25;
        d_alpha
            * (self.n2 * c.abs().powf(self.n2 - 1.0) * -c.signum() * s
                + self.n3 * s.abs().powf(self.n3 - 1.0) * s.signum() * c)
    }

    /// `(r, r')` no ângulo dado. ⚠️ `A > 0` sempre (o cosseno e o seno não se anulam juntos).
    /// ⭐⭐ **A janela de `α` que esta curva de facto percorre**, dado o intervalo do ângulo.
    ///
    /// ⛔⛔ **A 1.ª redacção varria `[−largura, largura]` para o perfil, e a janela dele NÃO está
    /// centrada em zero**: `φ ∈ [−π/2, π/2]` dá `α ∈ [mπ/8, 3mπ/8]`. O divisor saía **`69 %` curto**,
    /// que é a direcção que rasga a peça. *Uma janela de varredura são DOIS números, e escrever só a
    /// largura deixa o centro por dizer.*
    ///
    /// ⚠️ **Cortada num período de `π`**: `A` é `π`-periódica em `α`, então varrer mais é repetir —
    /// mas varrer MENOS do que a curva alcança perde o pico.
    fn alpha_window(self, lo: f64, hi: f64) -> (f64, f64) {
        let k = self.symmetry.abs() * 0.25;
        let a = k * (lo + std::f64::consts::PI);
        let b = k * (hi + std::f64::consts::PI);
        (a, a + (b - a).min(std::f64::consts::PI))
    }

    /// ⭐ `(E, L)` no `α` dado: `E = 1/r = A^(1/n1)` e `L = −r'/r = (m/4)·A'_α/(n1·A)`.
    ///
    /// ⚠️ **O `m/4` vive aqui**: `r'` é em `θ`, e a varredura corre em `α`. Esquecê-lo daria um
    /// divisor `m/4` vezes curto — que é exactamente a direcção que rasga a peça.
    fn e_l(self, alpha: f64) -> (f64, f64) {
        let (s, c) = alpha.sin_cos();
        let a = (c.abs().powf(self.n2) + s.abs().powf(self.n3)).max(1.0e-300);
        let da = self.n2 * c.abs().powf(self.n2 - 1.0) * -c.signum() * s
            + self.n3 * s.abs().powf(self.n3 - 1.0) * s.signum() * c;
        (
            a.powf(1.0 / self.n1),
            self.symmetry * 0.25 * da / (self.n1 * a),
        )
    }

    pub(crate) fn r_dr(self, theta: f64) -> (f64, f64) {
        let a = self.a_of(theta).max(1.0e-300);
        let r = a.powf(-1.0 / self.n1);
        // `r' = −(r/(n1·A))·A'`
        (r, -(r / (self.n1 * a)) * self.da_of(theta))
    }
}

/// Quantas amostras a varredura **grossa** usa por período de `α` — só para **separar os picos**,
/// e não para os medir.
///
/// ⛔⛔⛔ **VARRER EM `θ` NÃO ENCONTRA ESTE MÁXIMO, e o erro é para o lado PERIGOSO.** A estrutura da
/// fórmula vive em `α = m(θ+π)/4`, então uma feição de largura `Δα` mede `4Δα/m` em `θ`: a `m = 24`
/// ela encolhe **24×**, e uma grelha uniforme em `θ` passa entre as amostras.
///
/// Medido sobre a fila inteira (simetria `1..24` × `n1` × `n2` × `n3`, `1 728` combinações), o
/// **défice** do divisor — ele ficar CURTO, que é o que faz a peça rasgar:
///
/// | varredura | pior défice |
/// |---|---:|
/// | grelha uniforme em `θ`, `512` amostras | **`16,3 %`** |
/// | + bracket e secção áurea | `4,5 %` |
/// | + contagem a seguir a simetria (`64`/lobo) | `23,9 %` (a `m = 1`) |
/// | **em `α`, um período, `512` + secção áurea** | ver `probe_gielis` |
///
/// ⚠️ *Adensar não era a cura*: o erro de uma grelha uniforme sobre um pico cai como `1/n²`, e a
/// árvore é recozida **por quadro**. A cura é varrer na variável em que a feição tem largura
/// constante.
const SCAN: usize = 512;

/// Quantos passos de secção áurea refinam cada pico depois de a grelha o **bracketar**.
///
/// ⚠️ `60` passos encolhem o intervalo por `0,618⁶⁰ ≈ 2·10⁻¹³` — muito antes disso o refinamento
/// deixa de ser o termo de erro.
const REFINE: usize = 60;

/// O máximo de `f` em `[lo, hi]`, supondo **um** pico lá dentro — secção áurea.
fn golden_max(f: &impl Fn(f64) -> f64, lo: f64, hi: f64) -> f64 {
    const PHI: f64 = 0.618_033_988_749_895;
    let (mut a, mut b) = (lo, hi);
    let (mut c, mut d) = (b - PHI * (b - a), a + PHI * (b - a));
    let (mut fc, mut fd) = (f(c), f(d));
    for _ in 0..REFINE {
        if fc > fd {
            b = d;
            d = c;
            fd = fc;
            c = b - PHI * (b - a);
            fc = f(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + PHI * (b - a);
            fd = f(d);
        }
    }
    fc.max(fd)
}

/// ⭐⭐ **A grelha SEPARA os picos; a secção áurea MEDE-OS.**
///
/// ⚠️ Cada amostra maior que as duas vizinhas está a bracketar um pico, e é aí — e só aí — que se
/// refina. ⚠️ **As pontas entram como candidatas**: o `α` de uma curva percorre um intervalo
/// FECHADO, e o máximo pode estar na borda.
fn scan_max(lo: f64, hi: f64, n: usize, f: impl Fn(f64) -> f64) -> f64 {
    let mut pior = candidatos_criticos(lo, hi, &f);
    let at = |i: usize| lo + (hi - lo) * (i as f64 + 0.5) / n as f64;
    let v: Vec<f64> = (0..n).map(|i| f(at(i))).collect();
    for i in 0..n {
        let esq = if i == 0 { f64::NEG_INFINITY } else { v[i - 1] };
        let dir = if i + 1 == n {
            f64::NEG_INFINITY
        } else {
            v[i + 1]
        };
        // ⛔ **`>=` dos DOIS lados faz de cada amostra de um PATAMAR um «pico»** — a 1.ª redacção
        // refinava as `512` e o divisor passou a custar `1,45 ms` **por quadro**. Um `>` de um lado
        // só é o tratamento clássico de patamar.
        if v[i] > esq && v[i] >= dir {
            let a = at(i.saturating_sub(1));
            let b = at((i + 1).min(n - 1));
            pior = pior.max(golden_max(&f, a, b).max(v[i]));
        }
    }
    pior
}

/// ⭐⭐⭐ **OS ÂNGULOS CRÍTICOS, avaliados dos DOIS lados** — nenhuma grelha os alcança.
///
/// ⛔⛔ Com `n₃ = 1` o termo `n₃·|s|^(n₃−1)·sgn(s)·c` de `dA/dα` vale `sgn(s)·c` e **salta** quando
/// `s` cruza zero: `E²·M(L)` é **descontínua** em `α = kπ/2`, e o supremo é um limite lateral. Uma
/// grelha só chega a meia célula dele — medido, o divisor ficava **`1,7 %` curto**, e uma grelha
/// `34×` mais fina só encurtava a distância.
///
/// ⭐ A cura é estrutural: *a fórmula diz onde estão as suas próprias esquinas*. Avalia-se em
/// `kπ/2 ± ε` e nas duas pontas da janela.
fn candidatos_criticos(lo: f64, hi: f64, f: &impl Fn(f64) -> f64) -> f64 {
    const LADO: f64 = 1.0e-9;
    let mut pior = f(lo).max(f(hi));
    let meia = std::f64::consts::FRAC_PI_2;
    #[allow(clippy::cast_possible_truncation)]
    let primeiro = (lo / meia).floor() as i64;
    #[allow(clippy::cast_possible_truncation)]
    let ultimo = (hi / meia).ceil() as i64;
    for k in primeiro..=ultimo {
        #[allow(clippy::cast_precision_loss)]
        let a = k as f64 * meia;
        for x in [a - LADO, a, a + LADO] {
            if x >= lo && x <= hi {
                pior = pior.max(f(x));
            }
        }
    }
    pior
}

/// ⭐⭐⭐ **O máximo, sobre TODAS as direcções, de `(c − L·s)²·wa + (s + L·c)²·wb`** — em forma
/// fechada.
///
/// É a peça que tira o ângulo da varredura: para cada `α` o gradiente é um vector cuja direcção
/// depende de `θ`, e o maior valor dessa forma quadrática sai de
/// `(a+b)/2 + √( ((a−b)/2)² + (c/2)² )`.
///
/// ⚠️ **É um MAJORANTE do que a peça atinge**, não uma igualdade: os `θ` que caem num dado `α` são
/// `m` e não todos. Majorar aqui deixa o divisor um fio grande — o que custa passos de marcha e
/// **nunca** segurança. *A direcção errada de um divisor é a única que rasga a peça.*
fn max_over_direction(l: f64, wa: f64, wb: f64) -> f64 {
    let a = wa + l * l * wb;
    let b = l * l * wa + wb;
    let c = 2.0 * l * (wb - wa);
    0.5 * (a + b) + (0.25 * (a - b) * (a - b) + 0.25 * c * c).sqrt()
}

/// Amostras da varredura do **raio máximo** — `E` não tem picos estreitos (é `A^(1/n1)`, com `A`
/// suave entre as esquinas), logo esta varredura é `4×` mais barata que a do divisor.
const SCAN_NORM: usize = 128;

/// ⭐⭐⭐ **O RAIO MÁXIMO da curva — e é ele que faz a peça ter o TAMANHO que o painel diz.**
///
/// ⛔⛔ **Sem isto o controlo mente:** `r = A^(−1/n1)` chega a `8` com `m = 5, n2 = n3 = 8`, e a peça
/// media `2,8` numa caixa de meia-medida `0,35` — **oito vezes** o que o artista escreveu, e a
/// marcha começava DENTRO dela (`0` passos). *Mexer num EXPOENTE mudava o TAMANHO.* Normalizada por
/// `max r`, a peça encosta na caixa e os expoentes só mexem na forma.
fn r_max(c: Curve, lo: f64, hi: f64) -> f64 {
    // `max r = 1/min E`, e o mínimo acha-se maximizando `−E`.
    let neg = scan_max(lo, hi, SCAN_NORM, |alpha| -c.e_l(alpha).0);
    1.0 / (-neg).max(1.0e-300)
}

/// ⭐⭐ **O divisor: `max‖∇g‖` sobre a superfície** — duas varreduras em `α`.
///
/// ⚠️ **A decomposição é uma demonstração; o que se varre é o máximo de uma função de uma
/// variável.** Um gate que a recalculasse ao lado seria um oráculo feito da função sob teste — quem
/// a confere é a medição do campo **já dividido**.
#[must_use]
pub fn bound(half: [f64; 3], top: Curve, side: Curve) -> f64 {
    // (1) o de cima, sobre a volta inteira. ⚠️ **Sobre a curva NORMALIZADA** (`Ê = E·max r`), e é
    // por isso que o quadrado do factor entra aqui.
    let (lo, hi) = top.alpha_window(-std::f64::consts::PI, std::f64::consts::PI);
    let n_top = r_max(top, lo, hi);
    let q = n_top
        * n_top
        * scan_max(lo, hi, SCAN, |alpha| {
            let (e, l) = top.e_l(alpha);
            e * e * max_over_direction(l, 1.0 / (half[0] * half[0]), 1.0 / (half[2] * half[2]))
        });
    // (2) e o perfil, que só percorre meia volta — e não a meia volta em torno de zero.
    let (lo, hi) = side.alpha_window(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
    let n_side = r_max(side, lo, hi);
    (n_side
        * n_side
        * scan_max(lo, hi, SCAN, |alpha| {
            let (e, l) = side.e_l(alpha);
            e * e * max_over_direction(l, q, 1.0 / (half[1] * half[1]))
        }))
    .sqrt()
}

/// `|v|^n` regularizado — o chamador entrega algo em `[0, 1]`.
fn abs_pow(v: &Tree, n: f64) -> Tree {
    ((v.clone() * v.clone() + Tree::constant(EPS)).ln() * Tree::constant(n * 0.5)).exp()
}

/// `A(ângulo)^(1/n1)`, que é **`1/r`** — a árvore nunca precisa de `r`, só do inverso.
fn inv_r(theta: &Tree, c: Curve, escala: f64) -> Tree {
    let alpha =
        (theta.clone() + Tree::constant(std::f64::consts::PI)) * Tree::constant(c.symmetry * 0.25);
    let a = abs_pow(&alpha.clone().cos(), c.n2) + abs_pow(&alpha.sin(), c.n3);
    (a.ln() * Tree::constant(1.0 / c.n1)).exp() * Tree::constant(escala)
}

/// ⭐⭐⭐ **O SÓLIDO DE GIELIS**, pelas duas medidas encaixadas — ver o cabeçalho.
#[must_use]
pub fn sd_superformula(half: [f64; 3], top: Curve, side: Curve) -> Tree {
    let q = |t: Tree, h: f64| t / Tree::constant(h);
    let (qx, qy, qz) = (
        q(Tree::x(), half[0]),
        q(Tree::y(), half[1]),
        q(Tree::z(), half[2]),
    );
    // A medida da curva de CIMA, no plano `X–Z` (o eixo de cima desta casa é o `Y`).
    let raio_plano = (qx.clone() * qx.clone() + qz.clone() * qz.clone()).sqrt();
    let (lo, hi) = top.alpha_window(-std::f64::consts::PI, std::f64::consts::PI);
    let s = raio_plano * inv_r(&qz.atan2(qx), top, r_max(top, lo, hi));
    // E a do PERFIL, no meio-plano `(s, qy)`.
    let raio_perfil = (s.clone() * s.clone() + qy.clone() * qy.clone()).sqrt();
    let (lo, hi) = side.alpha_window(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
    let g = raio_perfil * inv_r(&qy.atan2(s), side, r_max(side, lo, hi));
    (g - 1.0) / Tree::constant(bound(half, top, side))
}

/// ⭐ `(r, r')` de uma curva — **exposta só para a sonda** poder construir a referência densa sem
/// passar pelo caminho do produto. *Um oráculo que chama a função sob teste não é um oráculo.*
#[must_use]
pub fn r_dr_of(c: Curve, theta: f64) -> (f64, f64) {
    c.r_dr(theta)
}

/// A janela varrida — **exposta só para a sonda**, ver [`r_dr_of`].
#[must_use]
pub fn alpha_window_of(c: Curve, lo: f64, hi: f64) -> (f64, f64) {
    c.alpha_window(lo, hi)
}
