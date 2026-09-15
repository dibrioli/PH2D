//! O **gradiente conjugado** sobre as variáveis livres, e o **conjunto activo** que impõe as caixas.
//!
//! ⚠️⚠️ **As caixas `0 ≤ w ≤ 1` não são um detalhe: são metade da lei.** Sem elas a minimização da
//! energia bilaplaciana tem **lóbulos negativos** — a solução ultrapassa e volta, como um spline
//! sobre-oscila —, e um peso negativo faz um osso EMPURRAR arte que devia ignorar. É isso que
//! separa *biharmonic* de *bounded biharmonic*, e é a diferença que o paper nomeia como a que dá
//! **localidade**.
//!
//! ⛔ **Clampar não é resolver.** Resolver sem caixas e cortar o que saiu devolve algo admissível
//! que **não é o mínimo**: a energia salta exactamente nos pontos cortados, e a suavidade — a única
//! coisa que esta lei existe para comprar — vai-se ali. O conjunto activo fixa os violadores, volta
//! a resolver, e **liberta** os que deixaram de querer sair.

use super::Options;
use super::laplacian::Laplacian;

/// O que uma corrida do conjunto activo devolveu.
pub(crate) struct Saida {
    pub(crate) rondas: usize,
    pub(crate) convergiu: bool,
    pub(crate) residuo: f64,
}

/// O estado de cada variável no conjunto activo.
#[derive(Clone, Copy, PartialEq)]
enum Estado {
    Livre,
    /// Presa por Dirichlet (o osso) — ⛔ **nunca libertada**.
    Dirichlet(f64),
    /// Presa numa caixa pelo conjunto activo — pode ser libertada.
    Caixa(f64),
}

/// ⭐⭐⭐ **Minimiza `wᵀ Q w` com `w` fixo onde `fixos[i]` diz, e `0 ≤ w ≤ 1` em toda parte.**
pub(crate) fn active_set(
    lap: &Laplacian,
    fixos: &[Option<f64>],
    opts: Options,
) -> (Vec<f64>, Saida) {
    let n = lap.n();
    let mut estado: Vec<Estado> = fixos
        .iter()
        .map(|f| f.map_or(Estado::Livre, Estado::Dirichlet))
        .collect();
    let mut w = vec![0.0_f64; n];
    for (i, e) in estado.iter().enumerate() {
        if let Estado::Dirichlet(v) = *e {
            w[i] = v;
        }
    }
    let (mut rondas, mut residuo_pior, mut convergiu) = (0usize, 0.0_f64, false);

    for _ in 0..opts.max_rondas {
        rondas += 1;
        let r = resolve_livres(lap, &estado, &mut w, opts);
        residuo_pior = residuo_pior.max(r);

        // ── Quem saiu da caixa passa a estar preso nela ──────────────────────────────────────
        let mut mudou = false;
        for i in 0..n {
            if !matches!(estado[i], Estado::Livre) {
                continue;
            }
            if w[i] < 0.0 {
                estado[i] = Estado::Caixa(0.0);
                w[i] = 0.0;
                mudou = true;
            } else if w[i] > 1.0 {
                estado[i] = Estado::Caixa(1.0);
                w[i] = 1.0;
                mudou = true;
            }
        }
        if mudou {
            continue;
        }

        // ── E quem já não quer sair é LIBERTADO (a metade que um clamp não tem) ──────────────
        //
        // ⭐ O multiplicador de Lagrange de uma variável presa é o gradiente `(Q w)_i`. Ela só deve
        // continuar presa se o gradiente a empurra PARA FORA da caixa; se empurra para dentro, o
        // mínimo está no interior e mantê-la presa é entregar uma solução que não é a mínima.
        let mut grad = vec![0.0_f64; n];
        let mut rascunho = vec![0.0_f64; n];
        lap.mul_q(&w, &mut rascunho, &mut grad);
        let mut libertou = false;
        for i in 0..n {
            match estado[i] {
                Estado::Caixa(b) if b == 0.0 && grad[i] < -1e-12 => {
                    estado[i] = Estado::Livre;
                    libertou = true;
                }
                Estado::Caixa(b) if b == 1.0 && grad[i] > 1e-12 => {
                    estado[i] = Estado::Livre;
                    libertou = true;
                }
                _ => {}
            }
        }
        if !libertou {
            convergiu = true;
            break;
        }
    }
    (
        w,
        Saida {
            rondas,
            convergiu,
            residuo: residuo_pior,
        },
    )
}

/// Um CG pré-condicionado sobre as variáveis LIVRES. Devolve o resíduo relativo que sobrou.
///
/// ⚠️ **As variáveis presas entram no lado direito**, não no sistema: minimizar em `w_livre` com
/// `w_preso` fixo é resolver `Q_ll · w_l = −Q_lp · w_p`. ⛔ Montar `Q_ll` seria montar `Q`; aqui o
/// produto restringe-se **zerando** as componentes presas antes e depois.
fn resolve_livres(lap: &Laplacian, estado: &[Estado], w: &mut [f64], opts: Options) -> f64 {
    let n = lap.n();
    let livre = |i: usize| matches!(estado[i], Estado::Livre);

    // `b = −(Q · w_preso)` restrito às livres.
    let mut so_presos = vec![0.0_f64; n];
    for (i, sp) in so_presos.iter_mut().enumerate() {
        *sp = match estado[i] {
            Estado::Livre => 0.0,
            Estado::Dirichlet(v) | Estado::Caixa(v) => v,
        };
    }
    let (mut rascunho, mut tmp) = (vec![0.0_f64; n], vec![0.0_f64; n]);
    lap.mul_q(&so_presos, &mut rascunho, &mut tmp);
    let mut b = vec![0.0_f64; n];
    for (i, bi) in b.iter_mut().enumerate() {
        *bi = if livre(i) { -tmp[i] } else { 0.0 };
    }

    // `x` = a parte livre da solução; começa onde ela está.
    let mut x = vec![0.0_f64; n];
    for (i, xi) in x.iter_mut().enumerate() {
        *xi = if livre(i) { w[i] } else { 0.0 };
    }
    let mut ax = vec![0.0_f64; n];
    let mut r = vec![0.0_f64; n];
    let aplica = |lap: &Laplacian, v: &[f64], rascunho: &mut [f64], out: &mut [f64]| {
        lap.mul_q(v, rascunho, out);
    };
    aplica(lap, &x, &mut rascunho, &mut ax);
    for (i, ri) in r.iter_mut().enumerate() {
        *ri = if livre(i) { b[i] - ax[i] } else { 0.0 };
    }
    let norma_b = b.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norma_b <= f64::MIN_POSITIVE {
        // ⚠️ Nada a fazer: sem carga do lado direito, a solução livre é a que já lá está.
        return 0.0;
    }
    let mut z = vec![0.0_f64; n];
    let precond = |lap: &Laplacian, r: &[f64], z: &mut [f64]| {
        for (i, zi) in z.iter_mut().enumerate() {
            let d = lap.diag_q[i];
            *zi = if d > f64::MIN_POSITIVE {
                r[i] / d
            } else {
                r[i]
            };
        }
    };
    precond(lap, &r, &mut z);
    for (i, zi) in z.iter_mut().enumerate() {
        if !livre(i) {
            *zi = 0.0;
        }
    }
    let mut p = z.clone();
    let mut rz: f64 = r.iter().zip(z.iter()).map(|(a, b)| a * b).sum();
    let mut residuo = r.iter().map(|v| v * v).sum::<f64>().sqrt() / norma_b;

    for _ in 0..opts.max_cg {
        if residuo <= opts.tol_cg {
            break;
        }
        aplica(lap, &p, &mut rascunho, &mut ax);
        for (i, a) in ax.iter_mut().enumerate() {
            if !livre(i) {
                *a = 0.0;
            }
        }
        let pap: f64 = p.iter().zip(ax.iter()).map(|(a, b)| a * b).sum();
        if pap.abs() <= f64::MIN_POSITIVE {
            break;
        }
        let alfa = rz / pap;
        for i in 0..n {
            x[i] = alfa.mul_add(p[i], x[i]);
            r[i] = (-alfa).mul_add(ax[i], r[i]);
        }
        precond(lap, &r, &mut z);
        for (i, zi) in z.iter_mut().enumerate() {
            if !livre(i) {
                *zi = 0.0;
            }
        }
        let rz_novo: f64 = r.iter().zip(z.iter()).map(|(a, b)| a * b).sum();
        let beta = rz_novo / rz;
        rz = rz_novo;
        for (i, pi) in p.iter_mut().enumerate() {
            *pi = beta.mul_add(*pi, z[i]);
        }
        residuo = r.iter().map(|v| v * v).sum::<f64>().sqrt() / norma_b;
    }

    for (i, wi) in w.iter_mut().enumerate() {
        if livre(i) {
            *wi = x[i];
        }
    }
    residuo
}
