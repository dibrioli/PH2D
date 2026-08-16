//! **O TETO DA TAXA** — a metade do *Lag CHOP* que este nó não tinha.
//!
//! O irmão ao lado responde *quão ATRASADA* a saída chega; este responde *quão
//! DEPRESSA ela tem licença para se mover no caminho*. São duas perguntas, e o
//! corte entre elas é o que mantém o `lib.rs` sob o teto de LOC.

/// A magnitude euclidiana de um vetor de `dim` componentes.
pub(crate) fn mag(v: &[f32]) -> f32 {
    v.iter().map(|c| c * c).sum::<f32>().sqrt()
}

/// **O TETO DA TAXA** — limita quão depressa a saída pode mudar, e quão depressa
/// essa taxa pode mudar. É a metade do *Lag CHOP* (TouchDesigner: *Max Slope* e
/// *Max Accel*) que este nó não tinha; a outra metade — as réguas de subida e
/// descida — já eram o `ticks`/`ticks_down`.
///
/// Escreve o valor limitado em `want` e **devolve a velocidade emitida**, que é o
/// estado do teto de aceleração.
///
/// ⚠️ **A régua é a MAGNITUDE do elemento, nunca a componente.** Um teto por
/// componente deixa o movimento diagonal ser `√2` vezes mais rápido que o axial —
/// a taxa passaria a depender da orientação dos eixos, que é a anisotropia que
/// esta casa já recusou no `sim.step::limit_speed`, cuja lei esta função espelha.
///
/// ⚠️ **A ACELERAÇÃO vem ANTES do passo, e a ordem é load-bearing:** o teto de
/// passo é o teto DURO (a saída nunca anda mais que ele num tique) e o de
/// aceleração governa como se chega lá. Aplicado por último, o passo é
/// inviolável; na ordem oposta uma velocidade anterior grande re-infla o
/// resultado e o número que o artista digitou deixa de ser um teto.
///
/// ⚠️ **A unidade é POR TIQUE, e é a régua do nó.** O `ticks` já é uma contagem
/// de tiques, então um teto em unidades por SEGUNDO seria a segunda régua no
/// mesmo painel — e dois números sobre a mesma grandeza são piores que um botão
/// morto. O preço fica nomeado: como o lag, o teto acompanha a taxa de quadros.
pub(crate) fn rate_limit(
    want: &mut [f32],
    prev: &[f32],
    prev_vel: &[f32],
    dim: usize,
    max_step: f32,
    max_accel: f32,
) -> Vec<f32> {
    let mut vel = vec![0.0f32; want.len()];
    let mut v = vec![0.0f32; dim];
    for e in 0..(want.len() / dim) {
        let s = e * dim;
        for (d, slot) in v.iter_mut().enumerate() {
            *slot = want[s + d] - prev[s + d];
        }
        if max_accel > 0.0 {
            let dv: Vec<f32> = (0..dim).map(|d| v[d] - prev_vel[s + d]).collect();
            let m = mag(&dv);
            if m > max_accel {
                let k = max_accel / m;
                for (d, slot) in v.iter_mut().enumerate() {
                    *slot = prev_vel[s + d] + dv[d] * k;
                }
            }
        }
        if max_step > 0.0 {
            let m = mag(&v);
            if m > max_step {
                let k = max_step / m;
                for slot in v.iter_mut() {
                    *slot *= k;
                }
            }
        }
        for (d, slot) in v.iter().enumerate() {
            want[s + d] = prev[s + d] + slot;
            vel[s + d] = *slot;
        }
    }
    vel
}
