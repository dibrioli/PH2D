//! ⭐⭐⭐ **O BOUND DA COMPOSIÇÃO** — `σ_max` do PRODUTO das matrizes, e não o produto dos `σ_max`.
//!
//! Plano e medições: `docs/3DModeling/09_o_bound_da_composicao.md`.
//!
//! # O que se cobrava, e o que é verdade
//!
//! O campo de uma pilha `[m₀ … mₙ]` num ponto do mundo é
//!
//! ```text
//! F(p) = inner( Φ(p) ) · S(p)      com  Φ = φ_{m₀} ∘ … ∘ φ_{mₙ}   e   S = Π kⱼ
//! ∇F   = S · (JΦ)ᵀ ∇inner + inner(Φ(p)) · ∇S
//! ```
//!
//! ⛔ O divisor de hoje cobra `σ(J_{m₀}) · … · σ(J_{mₙ})` — **cada um no pior ponto da caixa
//! inteira, independentemente dos outros**. A igualdade `σ(AB) = σ(A)σ(B)` exige que as direcções de
//! esticadela coincidam; com três matrizes ela exige o alinhamento nas três etapas ao mesmo tempo.
//!
//! Medido no trio `[Bend, Twist, Taper]` (spike `the_composed_jacobian_is_not_the_product`):
//!
//! | grandeza | valor |
//! |---|---:|
//! | divisor cobrado | `15,85` |
//! | produto dos `σ`, já **encadeado** ponto a ponto | `10,53` |
//! | **`σ_max` da COMPOSIÇÃO** | **`6,04`** |
//! | bound com o termo aditivo | **`4,68`** |
//!
//! ⇒ `1,5×` vem de cada `σ` ser tomado num pior caso **independente**, e `1,7×` do **desalinhamento**
//! das três direcções.
//!
//! # ⭐ Por que FROBENIUS, e não o `√(‖M‖₁‖M‖∞)`
//!
//! Medido nas matrizes desta família: `‖M‖_F` fica **`5,1 %`** acima do `σ_max` verdadeiro e
//! `√(‖M‖₁‖M‖∞)` fica **`27,6 %`**. A razão é estrutural — estes jacobianos têm **um** valor
//! singular grande e os outros dois `≤ 1`, e ali `‖M‖_F = √(σ² + …) ≈ σ`. *Um majorante escolhe-se
//! medindo-o na família em que vai viver, não pelo nome.*
//!
//! # ⚠️ O que é preciso para isto ser um MAJORANTE e não uma amostra
//!
//! O jacobiano é avaliado em **aritmética de intervalos com derivada** ([`crate::bounds_iv`]) sobre
//! sub-caixas do recorte: cada sub-caixa devolve um majorante válido, e o **máximo** sobre a
//! cobertura majora o todo. A sub-divisão existe para matar a dependência da aritmética de
//! intervalos, e não por gosto.
//!
//! ⛔ **E o resultado entra sempre por `min` com o produto de hoje** — a lei nova nunca pode ser
//! pior do que a que já defende o sítio.

use crate::bounds::Ball;
use crate::bounds_iv::{D, Iv};
use ph2d_field::Unary;

/// Quantas caixas o refinamento pode abrir antes de desistir e devolver o que tem.
///
/// # ⛔⛔ A sub-divisão UNIFORME está MEDIDA e REFUTADA
///
/// O erro da aritmética de intervalos aqui é de **primeira ordem** na largura da caixa, e uma grelha
/// uniforme paga `n³` para o dividir por `n`:
///
/// | grelha | caixas | bound | relógio |
/// |---:|---:|---:|---:|
/// | `8³` | `512` | `33,7` | `0,23 ms` |
/// | `16³` | `4 096` | `18,9` | `1,78` |
/// | `32³` | `32 768` | `12,6` | `14,1` |
/// | `64³` | `262 144` | `9,7` | **`110`** |
///
/// ⇒ para chegar perto do verdadeiro (`≈ 5`) seriam **milhões** de caixas. *O produto de hoje
/// (`15,85`) só é batido a partir de `16³`, e ali já custa mais do que compra.*
///
/// ⭐⭐⭐ **A cura é REFINAR SÓ QUEM DISPUTA O MÁXIMO** (branch-and-bound): a esmagadora maioria das
/// caixas está muito abaixo do pior, e parti-las não muda a resposta. O minorante que diz quando
/// parar sai de graça — o valor da MESMA lei numa caixa **degenerada** (um ponto), que é exacto.
const ORCAMENTO: usize = 3_000;

/// Quão perto do minorante o majorante tem de chegar antes de o refinamento parar.
///
/// ⚠️ Ele **não** é uma barra de segurança: o valor devolvido é sempre um majorante válido, refinado
/// ou não. É só onde deixa de valer a pena gastar relógio.
const TOL: f64 = 0.02;

/// O que cada passo da pilha precisa de saber, já tirado das mesmas portas que a árvore usa.
enum Passo {
    Bend {
        k: f64,
        reach: f64,
        lower: f64,
        upper: f64,
        falloff: f64,
        shift: usize,
    },
    Twist {
        k: f64,
        lower: f64,
        upper: f64,
        falloff: f64,
        shift: usize,
    },
    Taper {
        slope: f64,
        piso: f64,
        shift: usize,
    },
}

/// ⭐⭐⭐ **O bound de Lipschitz da pilha inteira** — `None` quando esta lei não modela a pilha.
///
/// ⚠️ **`None` não é falha, é abstenção**: o chamador fica com o produto de sempre. Modelar aqui
/// uma repetição (que é um `min` sobre células, não um mapa liso) exigiria outra matemática, e
/// devolver um número para ela seria inventar um majorante.
pub fn stack_lipschitz(mods: &[Unary], local: Ball) -> Option<f64> {
    let passos = passos(mods, local)?;
    let fim = crate::bounds::envelope(local, mods);
    let (lo, hi) = crate::bounds_clip::march_clip(fim);
    // ⭐ O alcance do `inner`: a superfície dele vive dentro da bola LOCAL, logo `|inner(q)|` é no
    // máximo a distância de `q` a essa bola.
    let alvo = (
        local.center.map(f64::from),
        f64::from(local.radius.max(0.0)),
    );
    let raiz: [Iv; 3] = std::array::from_fn(|e| Iv::new(f64::from(lo[e]), f64::from(hi[e])));
    let mut fila = vec![(bound_na_caixa(&passos, raiz, alvo)?, raiz)];
    let mut inferior = bound_na_caixa(&passos, centro(raiz), alvo)?;
    let mut abertas = 1usize;
    while abertas < ORCAMENTO {
        // A caixa que disputa o máximo — as outras não mudam a resposta.
        let (i, &(sup, caixa)) = fila
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.0.total_cmp(&b.1.0))?;
        if sup <= inferior * (1.0 + TOL) {
            break;
        }
        fila.swap_remove(i);
        // Parte pelo eixo MAIS LARGO — é o que mais encolhe a dependência.
        let eixo = (0..3)
            .max_by(|&a, &b| (caixa[a].hi - caixa[a].lo).total_cmp(&(caixa[b].hi - caixa[b].lo)))?;
        let meio = 0.5 * (caixa[eixo].lo + caixa[eixo].hi);
        for metade in 0..2 {
            let mut c = caixa;
            c[eixo] = if metade == 0 {
                Iv::new(caixa[eixo].lo, meio)
            } else {
                Iv::new(meio, caixa[eixo].hi)
            };
            let s = bound_na_caixa(&passos, c, alvo)?;
            inferior = inferior.max(bound_na_caixa(&passos, centro(c), alvo)?);
            fila.push((s, c));
            abertas += 1;
        }
    }
    let pior = fila.iter().map(|x| x.0).fold(0.0f64, f64::max);
    (pior > 0.0 && pior.is_finite()).then_some(pior.max(1.0))
}

fn centro(c: [Iv; 3]) -> [Iv; 3] {
    std::array::from_fn(|e| Iv::pt(0.5 * (c[e].lo + c[e].hi)))
}

/// O majorante desta caixa — `S·‖JΦ‖_F + |inner|·‖∇S‖`, tudo em intervalos.
fn bound_na_caixa(passos: &[Passo], caixa: [Iv; 3], alvo: ([f64; 3], f64)) -> Option<f64> {
    let (c, r) = alvo;
    let p: [D; 3] = std::array::from_fn(|e| D::var(caixa[e], e));
    let (phi, s) = aplica(passos, p);
    let mut f2 = 0.0f64;
    for saida in &phi {
        for parcial in &saida.d {
            if !parcial.is_finite() {
                return None;
            }
            f2 += parcial.mag() * parcial.mag();
        }
    }
    if !s.v.is_finite() || s.d.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let dist = (0..3)
        .map(|e| {
            let d = phi[e].v.sub(Iv::pt(c[e])).mag();
            d * d
        })
        .sum::<f64>()
        .sqrt();
    let grad_s = s.d.iter().map(|x| x.mag() * x.mag()).sum::<f64>().sqrt();
    let aqui = s.v.mag().mul_add(f2.sqrt(), (dist + r) * grad_s);
    aqui.is_finite().then_some(aqui)
}

fn passos(mods: &[Unary], local: Ball) -> Option<Vec<Passo>> {
    use ph2d_field::mods::{BEND_AXIS, TAPER_AXIS, TWIST_AXIS};
    let mut ball = local;
    let mut out = Vec::with_capacity(mods.len());
    for m in mods {
        let shift = crate::bounds::axis_shift_of(*m);
        out.push(match *m {
            Unary::Bend {
                turns,
                lower,
                upper,
                falloff,
                ..
            } => {
                let canon = ball.to_canonical(shift);
                let depois = crate::bounds::step_mod(ball, *m).to_canonical(shift);
                Passo::Bend {
                    k: crate::stack_bend::bend_curvature(turns, canon),
                    reach: crate::stack_bend::bend_reach(depois),
                    lower: f64::from(lower),
                    upper: f64::from(upper),
                    falloff: f64::from(falloff),
                    shift,
                }
            }
            Unary::Twist {
                turns,
                lower,
                upper,
                falloff,
                ..
            } => Passo::Twist {
                k: f64::from(turns) * std::f64::consts::TAU,
                lower: f64::from(lower),
                upper: f64::from(upper),
                falloff: f64::from(falloff),
                shift,
            },
            Unary::Taper { slope, .. } => Passo::Taper {
                slope: f64::from(slope),
                // ⚠️ O piso sai da bola **LOCAL**, como na [`crate::stack::stacked`].
                piso: crate::stack_taper::taper_floor(f64::from(slope), local.to_canonical(shift)),
                shift,
            },
            // ⛔ Abstém-se: ver o doc da [`stack_lipschitz`].
            _ => return None,
        });
        ball = crate::bounds::step_mod(ball, *m);
    }
    let _ = (BEND_AXIS, TWIST_AXIS, TAPER_AXIS);
    Some(out)
}

/// A composição `Φ = φ_{m₀} ∘ … ∘ φ_{mₙ}` e o multiplicador de valor `S`.
///
/// ⚠️ **A ordem é a INVERSA da lista**: a [`crate::stack::stacked`] põe `mods[0]` por DENTRO, logo
/// o ponto do mundo entra pelo ÚLTIMO.
fn aplica(passos: &[Passo], p: [D; 3]) -> ([D; 3], D) {
    let mut q = p;
    let mut s = D::cte(1.0);
    for passo in passos.iter().rev() {
        let (novo, mult) = um(passo, q);
        q = novo;
        s = s.mul(mult);
    }
    (q, s)
}

/// ⛔⛔⛔ **A ORDEM das duas permutações, e eu escrevi-a ao contrário** (2026-09-02).
///
/// A [`crate::stack::conjugado`] monta `f(dentro).remap_xyz(leva(3−s), …)`, e um `remap_xyz`
/// **substitui as coordenadas de quem avalia**: ⇒ como MAPA DE PONTO, o ponto do mundo passa
/// primeiro pela permutação de `3−s` (a [`entra`]) e só depois de `f` é que sofre a de `s` (a
/// [`sai`]). Escrevê-las trocadas dá **a identidade para o eixo de omissão** — e por isso só o
/// `the_law_on_another_axis_is_the_canonical_law_conjugated` a apanha.
///
/// *Uma permutação errada é invisível no caso em que ela é a identidade, que é o caso que toda
/// fixtura canónica exercita.*
fn entra(v: [D; 3], s: usize) -> [D; 3] {
    [v[(3 - s) % 3], v[(4 - s) % 3], v[(5 - s) % 3]]
}

fn sai(v: [D; 3], s: usize) -> [D; 3] {
    [v[s % 3], v[(1 + s) % 3], v[(2 + s) % 3]]
}

fn um(passo: &Passo, p: [D; 3]) -> ([D; 3], D) {
    match *passo {
        Passo::Taper { slope, piso, shift } => {
            let c = entra(p, shift);
            let k = c[1].escala(slope).add(D::cte(1.0)).max_pt(piso);
            let out = [c[0].div(k), c[1], c[2].div(k)];
            (sai(out, shift), k)
        }
        Passo::Twist {
            k,
            lower,
            upper,
            falloff,
            shift,
        } => {
            let c = entra(p, shift);
            let banda = soft_clamp(c[2], lower.min(upper), upper.max(lower), falloff.max(0.0));
            let ang = banda.escala(-k);
            let (sn, cs) = (ang.sin(), ang.cos());
            let out = [
                c[0].mul(cs).sub(c[1].mul(sn)),
                c[0].mul(sn).add(c[1].mul(cs)),
                c[2],
            ];
            (sai(out, shift), D::cte(1.0))
        }
        Passo::Bend {
            k,
            reach,
            lower,
            upper,
            falloff,
            shift,
        } => {
            let c = entra(p, shift);
            let s = if k < 0.0 { -1.0 } else { 1.0 };
            let rho = (1.0 / k).abs();
            let piso = (rho - reach.abs()).max(rho * (1.0 - crate::stack_bend::BEND_FOLD_MARGIN));
            let a = D::cte(rho).sub(c[0].escala(s)).max_pt(piso);
            let b = c[2];
            let rr = a.square().add(b.square()).sqrt();
            // ⭐ `a ≥ piso > 0` ⇒ `atan2(b, a) = atan(b/a)`, que é MONÓTONA sobre intervalos.
            let theta = b.div(a).atan();
            let theta_c = soft_clamp(
                theta,
                lower.min(upper) / rho,
                upper.max(lower) / rho,
                falloff.max(0.0) / rho,
            );
            let d = theta.sub(theta_c);
            let out = [
                D::cte(rho).sub(rr.mul(d.cos())).escala(s),
                c[1],
                theta_c.escala(rho).add(rr.mul(d.sin())),
            ];
            (sai(out, shift), D::cte(1.0))
        }
    }
}

/// A MESMA lei da [`crate::stack::soft_clamp`], escrita sobre a derivada.
fn soft_clamp(z: D, lo: f64, hi: f64, w: f64) -> D {
    let meia = (hi - lo).abs() * 0.5;
    let w = w.min(meia);
    if w <= 0.0 || !w.is_finite() {
        return z.max_pt(lo).min_pt(hi);
    }
    let suave = |a: D, b: f64, cima: bool| {
        let d = a.sub(D::cte(b)).abs();
        let h = D::cte(w).sub(d).max_pt(0.0).escala(1.0 / w);
        let corda = h.square().escala(w * 0.25);
        if cima {
            a.max_pt(b).add(corda)
        } else {
            a.min_pt(b).sub(corda)
        }
    };
    suave(suave(z, lo, true), hi, false)
}
