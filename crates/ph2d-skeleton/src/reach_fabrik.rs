//! ⭐⭐⭐ **UMA PASSAGEM DA CORRENTE** — irmão do [`super::reach`] pelo tecto de 700 LOC
//! (`architecture_workspace_file_loc_cap`), cortado por RESPONSABILIDADE: ali mora a LEI (que pose
//! se pede, que lado se autora, quando se pára); aqui mora a **varredura**, ida e volta, que é o
//! FABRIK propriamente dito.

use super::reach::unit;

/// Uma passagem completa (para trás e para a frente) e o erro da ponta ao alvo no fim dela.
pub(crate) fn sweep(
    joints: &mut [[f64; 2]],
    lengths: &[f64],
    alvo: [f64; 2],
    ancora: [f64; 2],
) -> f64 {
    let n = joints.len();
    // PARA TRÁS: a ponta toma o alvo e a corrente é arrastada atrás dela.
    joints[n - 1] = alvo;
    for i in (0..n - 1).rev() {
        let v = [
            joints[i][0] - joints[i + 1][0],
            joints[i][1] - joints[i + 1][1],
        ];
        let w = unit(v, [1.0, 0.0]);
        joints[i] = [
            joints[i + 1][0] + lengths[i] * w[0],
            joints[i + 1][1] + lengths[i] * w[1],
        ];
    }
    // PARA A FRENTE: a raiz volta ao sítio onde está pregada, e a corrente vem atrás.
    joints[0] = ancora;
    for i in 1..n {
        let v = [
            joints[i][0] - joints[i - 1][0],
            joints[i][1] - joints[i - 1][1],
        ];
        let w = unit(v, [1.0, 0.0]);
        joints[i] = [
            joints[i - 1][0] + lengths[i - 1] * w[0],
            joints[i - 1][1] + lengths[i - 1] * w[1],
        ];
    }
    (joints[n - 1][0] - alvo[0]).hypot(joints[n - 1][1] - alvo[1])
}
