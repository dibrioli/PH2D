//! **A SUPERFÍCIE LOCAL que o `l-mode` dos quatro verbos de plano projecta** —
//! a projeção MLS de Alexa, Behr, Cohen-Or, Fleishman, Levin & Silva 2003
//! (*Computing and Rendering Point Set Surfaces*), §4.
//!
//! ⚠️ **Irmão do [`super::plane`], e o corte são DOIS PAPERS.** Lá mora *que
//! plano a pegada ajusta* — o `calc_area_normal_and_center` da referência, uma
//! média ponderada de posições e normais. Aqui mora *que forma a superfície tem
//! em torno desse plano*, que é outra pergunta com outra fonte. Enfiá-las no
//! mesmo arquivo faria a próxima leitura ter de separar as duas de cabeça.
//!
//! # O que o paper diz, e o que este módulo porta
//!
//! A projeção MLS tem duas metades: **(1)** achar um plano de referência local
//! que minimiza `Σ ⟨n, p_i − q⟩² θ(|p_i − q|)` — uma otimização não-linear em
//! `q` — e **(2)** ajustar um polinómio bivariado sobre esse plano e avaliá-lo.
//!
//! ⚠️ **Só a (2) é portada, e a (1) é substituída pelo plano que o dab JÁ tem.**
//! Não é um atalho de custo: o `fit_plane` é a resposta que os quatro verbos já
//! usam há waves, e re-derivá-la aqui por outro critério seria a **segunda
//! resposta** a *"que plano descreve esta pegada?"* — o `l-mode` deixaria de ser
//! *outra lei sobre a mesma superfície* e passaria a ser outro plano também,
//! com a divergência a somar-se à do polinómio e ninguém a saber qual metade
//! moveu o barro.
//!
//! ⚠️ **E o passo continua a ser ao longo da NORMAL do plano** ([`super::target`]`::aim::to_plane`),
//! não ao longo da normal LOCAL da superfície. A projeção do paper caminha pela
//! normal em cada ponto; a diferença é de segunda ordem na curvatura e o preço
//! de a portar seria uma segunda porta de passo, ao lado da que os quatro verbos
//! partilham. **Divergência declarada**, não esquecida.

use super::plane::PlaneFit;

/// Quantos coeficientes tem um polinómio bivariado de grau 2.
const TERMS: usize = 6;

/// **A superfície local, no frame do plano.**
///
/// `h(u, v) = c0 + c1·u + c2·v + c3·u² + c4·uv + c5·v²`, com `(u, v)` medidos ao
/// longo de [`Self::tu`]/[`Self::tv`] a partir de [`PlaneFit::point`] e
/// **normalizados pelo raio do dab**.
///
/// ⚠️ **A normalização NÃO compra precisão, e a minha primeira versão deste doc
/// afirmava que sim.** Ela dizia que *"o sistema fica mal-condicionado
/// exactamente onde o pincel é grande"* — medido pela sonda
/// `where_the_unnormalised_fit_starts_to_lie`, o `f64` do [`solve`] absorve o
/// mal-condicionamento até um dab de raio **400 000**, com o desvio na altura
/// avaliada a ficar em `2e-16`. *Uma cerca que nenhum oráculo separa tem de ser
/// medida, não defendida por prosa.*
///
/// ⇒ **O que ela compra é o [`PIVOT_FLOOR`] ser livre de escala de cena**, e
/// esse é o lado PEQUENO: sem normalizar, um pincel de raio `4e-4` põe os termos
/// de quarta ordem em `2,5e-14` — abaixo do piso —, o ajuste é **recusado**, e o
/// `l-mode` colapsa no `s-mode` **em silêncio**. O gate
/// `the_fit_is_the_same_surface_at_any_brush_size` mede isso indo para BAIXO
/// (×0,001), e é por isso que ele não vai para cima: a mutação sobreviveu à
/// primeira versão dele, que ampliava.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Quadric {
    pub(super) tu: [f32; 3],
    pub(super) tv: [f32; 3],
    /// `1 / raio do dab` — o mesmo número na hora de ajustar e na de avaliar.
    pub(super) inv_r: f32,
    pub(super) c: [f32; TERMS],
}

impl Quadric {
    /// A altura da superfície acima do plano, no ponto `p`.
    ///
    /// ⚠️ **`p` chega em MUNDO e já relativo ao [`PlaneFit::point`]** — quem
    /// chama é o [`super::target`]`::aim::signed_distance`, que já computou a diferença
    /// para o produto interno com a normal; passá-la de novo aqui seria a
    /// terceira subtracção do mesmo vetor no caminho mais quente do dab.
    #[inline]
    pub(super) fn height_at(&self, d: [f32; 3]) -> f32 {
        let u = (d[0] * self.tu[0] + d[1] * self.tu[1] + d[2] * self.tu[2]) * self.inv_r;
        let v = (d[0] * self.tv[0] + d[1] * self.tv[1] + d[2] * self.tv[2]) * self.inv_r;
        let c = &self.c;
        c[3].mul_add(
            u * u,
            c[4].mul_add(
                u * v,
                c[5].mul_add(v * v, c[2].mul_add(v, c[1].mul_add(u, c[0]))),
            ),
        )
    }
}

/// Uma base ortonormal cujo Z é `n`, **função pura da normal**.
///
/// ⚠️ **A escolha do par tangente NÃO é observável no resultado**, e é isso que
/// a torna segura: girar `(tu, tv)` em torno de `n` re-parametriza o polinómio e
/// deixa a SUPERFÍCIE onde estava, então a altura devolvida pelo
/// [`Quadric::height_at`] é a mesma. O que ela precisa de ser é **estável** —
/// um par que dependesse da ordem dos vértices faria os coeficientes tremerem
/// entre dabs sem a superfície se mexer, e o `f32` acabaria por mostrar isso.
///
/// ⚠️ **A troca de semente em `|n.x| ≥ 0,9` é uma descontinuidade no FRAME**, e
/// ela é inofensiva pela mesma razão: os coeficientes saltam, a superfície não.
/// O gate `the_surface_is_the_same_in_any_tangent_frame` mede-o em vez de o
/// prometer.
fn tangent_frame(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let seed = if n[0].abs() < 0.9 {
        [1.0f32, 0.0, 0.0]
    } else {
        [0.0f32, 1.0, 0.0]
    };
    let mut tu = super::target::cross(seed, n);
    let len = (tu[0] * tu[0] + tu[1] * tu[1] + tu[2] * tu[2]).sqrt();
    let inv = if len > 1e-12 { 1.0 / len } else { 0.0 };
    for x in &mut tu {
        *x *= inv;
    }
    let tv = super::target::cross(n, tu);
    (tu, tv)
}

/// Resolve `A x = b` para o sistema `6×6` simétrico das equações normais —
/// eliminação de Gauss com pivotamento parcial, em `f64`.
///
/// `None` = **singular**, e o chamador cai no plano. Isso não é um caso de erro:
/// é a resposta certa para uma pegada degenerada (todos os pontos numa reta, ou
/// menos pontos que coeficientes), onde não existe quadric determinado. Uma
/// pseudo-inversa daria uma superfície que os dados não sustentam.
fn solve(mut a: [[f64; TERMS]; TERMS], mut b: [f64; TERMS]) -> Option<[f64; TERMS]> {
    for col in 0..TERMS {
        let mut piv = col;
        for r in col + 1..TERMS {
            if a[r][col].abs() > a[piv][col].abs() {
                piv = r;
            }
        }
        if a[piv][col].abs() < PIVOT_FLOOR {
            return None;
        }
        a.swap(col, piv);
        b.swap(col, piv);
        for r in col + 1..TERMS {
            let f = a[r][col] / a[col][col];
            let (lo, hi) = a.split_at_mut(r);
            for (dst, src) in hi[0][col..].iter_mut().zip(&lo[col][col..]) {
                *dst -= f * src;
            }
            b[r] -= f * b[col];
        }
    }
    let mut x = [0.0f64; TERMS];
    for i in (0..TERMS).rev() {
        let mut s = b[i];
        for j in i + 1..TERMS {
            s -= a[i][j] * x[j];
        }
        x[i] = s / a[i][i];
    }
    Some(x)
}

/// O piso abaixo do qual um pivô é tratado como zero.
///
/// ⚠️ **Ele é sobre as EQUAÇÕES NORMAIS, não sobre a malha:** com `u`, `v`
/// normalizados a `[−1, 1]` e os pesos em `[0, 1]`, a entrada `[0][0]` é a soma
/// dos pesos e as demais vivem abaixo dela — nenhuma escala de cena entra aqui,
/// que é precisamente o que a normalização compra.
const PIVOT_FLOOR: f64 = 1e-9;

/// **O AJUSTE.** `(u, v, h)` ponderados → os seis coeficientes.
///
/// ⚠️ **O peso é o MESMO que o plano usa — a MÁSCARA, nunca o falloff.** O
/// [`super::plane`] documenta a razão e ela vale aqui com mais força: o plano e
/// a superfície descrevem *que forma a pegada tem*, e força/pressão dizem
/// *quanto agir sobre ela*. Pesar a superfície pelo falloff faria o alvo mudar
/// quando o artista mexesse na força, com a geometria parada.
pub(super) fn fit(
    samples: impl Iterator<Item = (f32, f32, f32, f32)>,
    frame: ([f32; 3], [f32; 3]),
    inv_r: f32,
) -> Option<Quadric> {
    let mut ata = [[0.0f64; TERMS]; TERMS];
    let mut atb = [0.0f64; TERMS];
    let mut n = 0usize;
    for (u, v, h, w) in samples {
        let (u, v, h, w) = (f64::from(u), f64::from(v), f64::from(h), f64::from(w));
        let row = [1.0, u, v, u * u, u * v, v * v];
        for i in 0..TERMS {
            for j in 0..TERMS {
                ata[i][j] += w * row[i] * row[j];
            }
            atb[i] += w * row[i] * h;
        }
        n += 1;
    }
    // ⚠️ **Menos amostras que coeficientes NÃO é singularidade a descobrir pelo
    // pivô — é um facto sobre a pegada**, e recusar aqui poupa a aritmética de
    // um sistema que não tem resposta. Seis é o mínimo aritmético; a pegada de
    // um dab real traz dezenas.
    if n < TERMS {
        return None;
    }
    let c = solve(ata, atb)?;
    // ⚠️ **Uma pegada patológica pode resolver para números enormes** (quase
    // singular, mas acima do piso do pivô), e um `NaN`/`inf` aqui viajaria para
    // dentro do `signed_distance` e de lá para a POSIÇÃO do vértice. O recuo é o
    // plano, que é sempre uma resposta válida.
    if !c.iter().all(|x| x.is_finite()) {
        return None;
    }
    let mut out = [0.0f32; TERMS];
    for (o, v) in out.iter_mut().zip(c) {
        *o = v as f32;
    }
    Some(Quadric {
        tu: frame.0,
        tv: frame.1,
        inv_r,
        c: out,
    })
}

/// A base tangente que o [`fit`] espera, derivada do plano.
pub(super) fn frame_of(plane: &PlaneFit) -> ([f32; 3], [f32; 3]) {
    tangent_frame(plane.normal)
}
