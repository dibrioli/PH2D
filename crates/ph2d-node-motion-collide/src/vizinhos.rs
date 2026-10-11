//! **A varredura por VIZINHOS do `push_apart`** — `O(n · k)` em vez de `O(n²)`, e **o mesmo
//! resultado ao bit** que todos-os-pares.
//!
//! Porque ao bit: é o MESMO laço — `i` a crescer por fora, os `j > i` a crescer por dentro, cada
//! par calculado uma vez e espalhado aos dois —, só que os `j` de dentro vêm da grelha (a
//! [`ph2d_contact::GrelhaDeDiscos`] entrega-os em ordem crescente) em vez de serem todos. A grelha
//! só tem de entregar um SUPERCONJUNTO de quem toca; quem não toca é recusado pelo `par`, e não
//! entra em soma nenhuma — logo cada `delta` recebe as mesmas parcelas pela mesma ordem.
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
/// Por disco, com `c` candidatos, ela custa `c · (3 + 0,5·log₂ c)` verificações de par — modelo
/// ajustado à [`tests::sonda_custo_da_grelha`] quando cada par se calculava dos dois lados (raio 12
/// era `3,5×` mais LENTO pela grelha). `--release`, 11/10, load 1,5: raio 0,3 · 0,6 · 2 a `595× ·
/// 198× · 21×`; a cena `=8` (32 761 pontos, 8 passadas) `6,38 s → 0,105 s`; raio 12 recusado.
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
    for i in 0..q.len() {
        grade.vizinhos_de(i, viz);
        for &j in viz.iter() {
            let j = j as usize;
            if j > i
                && let Some(t) = discos.par(q, i, j)
            {
                delta[i][0] -= t[0];
                delta[i][1] -= t[1];
                delta[j][0] += t[2];
                delta[j][1] += t[3];
                contacts[i] += 1;
                contacts[j] += 1;
            }
        }
    }
}

#[cfg(test)]
#[path = "vizinhos_tests.rs"]
mod tests;
