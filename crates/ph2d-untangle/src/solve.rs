//! ⭐⭐ **A DESCIDA** — L-BFGS com busca linear de Armijo, e o calendário de `ε` por fora.

use crate::{Element, energy, energy_and_gradient, min_det};

/// Quantos pares `(s, y)` a memória do L-BFGS guarda.
///
/// ⚠️ **`8` é o valor de manual para problemas desta forma** e não uma medição desta casa —
/// está escrito assim de propósito. *Um número herdado que se anuncia como medido é a
/// armadilha que o `ALPHA` desta linha já pagou.* Quem o quiser mover, meça.
const MEMORY: usize = 8;

/// A memória do L-BFGS: os pares `(s, y)` de cada iteração guardada.
///
/// ⚠️ Um alias e não um `struct`: o par **é** a memória, e envolvê-lo num tipo com nomes
/// acrescentaria uma indirecção sem acrescentar uma pergunta.
type History = Vec<(Vec<[f64; 2]>, Vec<[f64; 2]>)>;

/// O que o chamador pode afinar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// A troca ângulo ↔ área. `1` pesa as duas por igual.
    pub lambda: f64,
    /// Tecto de iterações externas (uma por valor de `ε`).
    pub max_outer: usize,
    /// Tecto de iterações internas por valor de `ε`.
    pub max_inner: usize,
    /// A descida pára quando a energia deixa de cair mais que isto em termos relativos.
    pub rel_tol: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lambda: 1.0,
            max_outer: 64,
            max_inner: 32,
            rel_tol: 1e-3,
        }
    }
}

/// O que a descida fez.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Report {
    /// Elementos invertidos **antes**.
    pub flipped_before: usize,
    /// Elementos invertidos **depois**. ⭐ `0` é o objectivo.
    pub flipped_after: usize,
    /// `min det J` antes e depois.
    pub min_det: (f64, f64),
    /// Iterações externas gastas (uma por valor de `ε`).
    pub outer: usize,
    /// Iterações internas ao todo.
    pub inner: usize,
    /// ⛔ **A descida desistiu sem desemaranhar?** `true` quando o tecto foi atingido com
    /// elementos ainda invertidos — *o chamador tem de o ler antes de acreditar na saída.*
    pub gave_up: bool,
}

/// ⭐⭐⭐ **DESEMARANHAR** — move `uv` até `det J > 0` em todo elemento, ou desiste dizendo-o.
///
/// Os vértices em `locked` não se movem. ⚠️ **Sem nenhum bloqueado o problema é invariante a
/// translação**, e a descida vagueia sem nunca piorar — legítimo, mas desperdício; quem chama
/// prende ao menos um.
///
/// ⛔ **O calendário empírico de `ε`** (o que os autores relatam funcionar melhor na esmagadora
/// maioria dos casos, e que faz a base inteira passar no teste de injectividade):
///
/// ```text
/// ε = √( 1e−12 + 4e−2 · [ min(0, min_t det J_t) ]² )
/// ```
///
/// ⚠️ **Ele depende só do PIOR determinante**, e é isso que o faz encolher sozinho à medida que
/// o emaranhado se desfaz: quando `min det ≥ 0`, `ε` cai para `1e−6` e a energia passa a ser a
/// verdadeira.
pub fn untangle(
    elements: &[Element],
    uv: &mut [[f64; 2]],
    locked: &[bool],
    set: Settings,
) -> Report {
    let n = uv.len();
    let flipped_before = crate::flipped(elements, uv);
    let det_before = min_det(elements, uv);
    let mut rep = Report {
        flipped_before,
        flipped_after: flipped_before,
        min_det: (det_before, det_before),
        outer: 0,
        inner: 0,
        gave_up: false,
    };
    if elements.is_empty() || n == 0 {
        return rep;
    }

    let mut grad = vec![[0.0f64; 2]; n];
    let mut hist: History = Vec::new();
    let mut prev_energy = f64::INFINITY;

    for outer in 0..set.max_outer {
        rep.outer = outer + 1;
        let worst = min_det(elements, uv);
        let eps = 4e-2f64.mul_add(worst.min(0.0).powi(2), 1e-12).sqrt();

        // ⚠️ A memória do L-BFGS é limpa a cada `ε` — a energia MUDOU, e curvatura medida
        // sobre outra função é curvatura errada.
        hist.clear();
        let mut e_now = energy_and_gradient(elements, uv, eps, set.lambda, &mut grad);
        for _ in 0..set.max_inner {
            rep.inner += 1;
            let dir = two_loop(&hist, &grad, locked);
            let slope: f64 = dir
                .iter()
                .zip(grad.iter())
                .map(|(d, g)| d[0].mul_add(g[0], d[1] * g[1]))
                .sum();
            // ⛔⛔ **O SINAL, e ele mordeu nos DOIS sítios ao mesmo tempo** (2026-08-30):
            // [`two_loop`] devolve `H·∇F`, que é direcção de **subida**; a descida é `−p`, e
            // ela só é válida quando `⟨−p, ∇F⟩ < 0`, ou seja **`slope > 0`**. A 1.ª redacção
            // tinha `slope >= 0` a disparar o ramo de recuo **e** passava `+grad` como direcção
            // — logo toda iteração subia, a busca linear recusava tudo e o laço saía no
            // primeiro passo com a malha **intacta**.
            //
            // ⭐ *O gate do gradiente passou enquanto isto acontecia* — e é exactamente para
            // isso que ele existe: separar a MATEMÁTICA da DESCIDA. Sem ele, o sintoma («não
            // desemaranha») teria mandado procurar o erro na derivada.
            if !slope.is_finite() || slope <= 0.0 {
                // A direcção não desce (memória degenerada) — recomeça em gradiente puro.
                hist.clear();
                let steep: Vec<[f64; 2]> = grad.iter().map(|g| [-g[0], -g[1]]).collect();
                if !descend(
                    elements, uv, locked, &steep, e_now, eps, set.lambda, &mut e_now,
                ) {
                    break;
                }
                energy_and_gradient(elements, uv, eps, set.lambda, &mut grad);
                continue;
            }
            let before: Vec<[f64; 2]> = uv.to_vec();
            let gbefore: Vec<[f64; 2]> = grad.clone();
            let neg: Vec<[f64; 2]> = dir.iter().map(|d| [-d[0], -d[1]]).collect();
            if !descend(
                elements, uv, locked, &neg, e_now, eps, set.lambda, &mut e_now,
            ) {
                break;
            }
            energy_and_gradient(elements, uv, eps, set.lambda, &mut grad);
            let s: Vec<[f64; 2]> = uv
                .iter()
                .zip(before.iter())
                .map(|(a, b)| [a[0] - b[0], a[1] - b[1]])
                .collect();
            let y: Vec<[f64; 2]> = grad
                .iter()
                .zip(gbefore.iter())
                .map(|(a, b)| [a[0] - b[0], a[1] - b[1]])
                .collect();
            if dot(&s, &y) > 1e-30 {
                hist.push((s, y));
                if hist.len() > MEMORY {
                    hist.remove(0);
                }
            }
        }

        let after = min_det(elements, uv);
        rep.min_det.1 = after;
        // ⭐ Os DOIS critérios do laço externo: sem dobra **e** a energia assentou.
        if after > 0.0 && e_now > (1.0 - set.rel_tol) * prev_energy {
            break;
        }
        prev_energy = e_now;
    }
    rep.flipped_after = crate::flipped(elements, uv);
    rep.min_det.1 = min_det(elements, uv);
    rep.gave_up = rep.flipped_after > 0;
    rep
}

fn dot(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x[0].mul_add(y[0], x[1] * y[1]))
        .sum()
}

/// A recursão de dois laços do L-BFGS. Devolve a direcção **de subida** (o chamador nega).
fn two_loop(hist: &History, grad: &[[f64; 2]], locked: &[bool]) -> Vec<[f64; 2]> {
    let mut q: Vec<[f64; 2]> = grad.to_vec();
    for (i, l) in locked.iter().enumerate() {
        if *l {
            q[i] = [0.0, 0.0];
        }
    }
    let mut alpha = vec![0.0f64; hist.len()];
    for (k, (s, y)) in hist.iter().enumerate().rev() {
        let rho = 1.0 / dot(y, s);
        alpha[k] = rho * dot(s, &q);
        for i in 0..q.len() {
            q[i][0] -= alpha[k] * y[i][0];
            q[i][1] -= alpha[k] * y[i][1];
        }
    }
    if let Some((s, y)) = hist.last() {
        let scale = dot(s, y) / dot(y, y);
        for v in &mut q {
            v[0] *= scale;
            v[1] *= scale;
        }
    }
    for (k, (s, y)) in hist.iter().enumerate() {
        let rho = 1.0 / dot(y, s);
        let beta = rho * dot(y, &q);
        for i in 0..q.len() {
            q[i][0] += s[i][0] * (alpha[k] - beta);
            q[i][1] += s[i][1] * (alpha[k] - beta);
        }
    }
    for (i, l) in locked.iter().enumerate() {
        if *l {
            q[i] = [0.0, 0.0];
        }
    }
    q
}

/// Busca linear de Armijo ao longo de `dir` (que já é direcção de **descida**).
///
/// ⚠️ **Devolve `false` quando nenhum passo desce** — e nesse caso `uv` fica **intacto**. *Um
/// laço que aceita um passo que sobe troca uma malha emaranhada por outra pior, em silêncio.*
#[expect(
    clippy::too_many_arguments,
    reason = "e' a assinatura de uma busca linear: malha, estado, cerca, direccao, energia, os dois parametros e a saida"
)]
fn descend(
    elements: &[Element],
    uv: &mut [[f64; 2]],
    locked: &[bool],
    dir: &[[f64; 2]],
    e0: f64,
    eps: f64,
    lambda: f64,
    out: &mut f64,
) -> bool {
    let before: Vec<[f64; 2]> = uv.to_vec();
    let mut t = 1.0f64;
    for _ in 0..40 {
        for i in 0..uv.len() {
            if locked.get(i).copied().unwrap_or(false) {
                continue;
            }
            uv[i][0] = t.mul_add(dir[i][0], before[i][0]);
            uv[i][1] = t.mul_add(dir[i][1], before[i][1]);
        }
        let e = energy(elements, uv, eps, lambda);
        if e.is_finite() && e < e0 {
            *out = e;
            return true;
        }
        t *= 0.5;
    }
    uv.copy_from_slice(&before);
    false
}
