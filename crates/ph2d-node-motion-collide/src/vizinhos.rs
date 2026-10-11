//! **A varredura por VIZINHOS do `push_apart`** — `O(n · k)` em vez de `O(n²)`, e **o mesmo
//! resultado ao bit** que todos-os-pares.
//!
//! Porque ao bit: em todos-os-pares, o `delta[k]` soma os parceiros de `k` em ordem CRESCENTE de
//! índice (os `m < k` chegam pelo laço de fora, depois os `m > k` pelo de dentro). Aqui cada disco
//! junta os seus próprios parceiros, também em ordem crescente (a [`ph2d_contact::GrelhaDeDiscos`]
//! entrega-os assim), e cada par é sempre calculado como `par(min, max)` — os mesmos floats, a
//! mesma ordem de soma. A grelha só tem de entregar um SUPERCONJUNTO de quem toca; quem não toca é
//! recusado pelo `par`, como antes.
//!
//! O arredondamento de `floor(x / lado)` nunca separa por DUAS células um par que toca, a qualquer
//! magnitude: quando o quociente fica grosso o bastante para saltar uma célula, as próprias posições
//! já estão mais espaçadas que `2 · r_max`. Procurado em 11/10 (108 milhões de pares que tocam, de
//! `2⁻¹⁰` a `2⁴⁰`, float32): zero perdidos com `lado = 2 · r_max`, 1,9 milhão com `lado = r_max`.
//!
//! Quando a grelha não serve (coordenada não finita — um NaN passa o guarda do `par` e toca toda a
//! gente —, ou candidatos demais), a passada é a de todos-os-pares.

use super::Discos;
use ph2d_contact::GrelhaDeDiscos;

/// O lado da célula: a maior distância de contacto de um par (`r_i + r_j ≤ 2 · r_max`).
pub(super) fn lado_da_celula(r_max: f32) -> f32 {
    2.0 * r_max
}

/// Constrói a grelha sobre `q` e diz se ela serve para esta passada.
pub(super) fn grelha_serve(grade: &mut GrelhaDeDiscos, q: &[[f32; 2]], lado: f32) -> bool {
    if !lado.is_finite() || q.iter().flatten().any(|v| !v.is_finite()) {
        return false;
    }
    grade.constroi(q, lado);
    vale_a_pena(grade.candidatos_previstos(), q.len())
}

/// A grelha só corre quando o custo dela é menos de METADE do de todos-os-pares (`n²/2` pares).
/// Por disco, com `c` candidatos, ela custa `c · (3 + 0,5·log₂ c)` verificações de par — o modelo
/// ajustado à sonda [`tests::sonda_custo_da_grelha`] (11/10, 16 384 discos, raio 0,3 · 0,6 · 2 ·
/// 12: a grelha `270× · 120× · 8×` mais rápida e `3,5×` mais LENTA no raio 12).
fn vale_a_pena(candidatos: usize, n: usize) -> bool {
    #[expect(clippy::cast_precision_loss, reason = "contagens muito abaixo de 2^52")]
    let (c, n) = (candidatos as f64, n as f64);
    let por_disco = (c / n.max(1.0)).max(1.0);
    c * (3.0 + 0.5 * por_disco.log2()) < n * n / 4.0
}

/// A passada pela grelha: o `delta` e os `contacts` de cada disco, ao bit iguais aos de
/// todos-os-pares.
pub(super) fn varre(
    discos: &Discos<'_>,
    q: &[[f32; 2]],
    grade: &GrelhaDeDiscos,
    viz: &mut Vec<u32>,
    delta: &mut [[f32; 2]],
    contacts: &mut [u32],
) {
    for k in 0..q.len() {
        grade.vizinhos_de(k, viz);
        for &m in viz.iter() {
            let m = m as usize;
            if m < k {
                if let Some(t) = discos.par(q, m, k) {
                    delta[k][0] += t[2];
                    delta[k][1] += t[3];
                    contacts[k] += 1;
                }
            } else if m > k
                && let Some(t) = discos.par(q, k, m)
            {
                delta[k][0] -= t[0];
                delta[k][1] -= t[1];
                contacts[k] += 1;
            }
        }
    }
}

#[cfg(test)]
#[path = "vizinhos_tests.rs"]
mod tests;
