//! **A REFERÊNCIA: a mesma lei por TODOS-OS-PARES** — irmã do [`super`] pelo tecto de LOC (HR-18) e
//! por ASSUNTO: ali mora o caminho do PRODUTO (a grelha, os dois atalhos de paragem, a partição
//! paralela), aqui mora o padrão contra o qual ele se mede.
//!
//! ⛔ **Nenhum caminho de produto a chama.** Ela varre sempre até ao fim e visita todos os pares —
//! é isso que a torna capaz de julgar os atalhos: um atalho que também vivesse aqui não poderia ser
//! medido.

use super::laco::{Nova, aplica, confere};
use super::{Colisor, Pecas, Saida, ativo, varredura};

/// **A referência**: a mesma lei por todos-os-pares, na ordem do laço `i < j`. `O(n²)`.
///
/// Pública para que o gate de cada cliente possa comparar-se com ela; nenhum caminho de produto a
/// chama.
pub fn separate_all_pairs(
    p: &mut [[f32; 2]],
    saida: &mut Saida<'_>,
    pecas: &Pecas<'_>,
    varreduras: usize,
) {
    let n = p.len();
    confere(n, saida, pecas);
    let ativo: Vec<bool> = (0..n)
        .map(|i| ativo(p[i], pecas.colisores[i].as_ref()))
        .collect();
    for _ in 0..varreduras {
        let foto = p.to_vec();
        let girado = saida.giro.to_vec();
        let agora: Vec<Option<Colisor>> = (0..n)
            .map(|i| pecas.colisores[i].map(|c| c.girado(girado[i])))
            .collect();
        let mut novas: Vec<Nova> = (0..n)
            .map(|k| {
                if ativo[k] {
                    varredura::corrigida(k, 0..n, &foto, &agora, pecas, &ativo)
                } else {
                    None
                }
            })
            .collect();
        // ⚠️ A referência varre SEMPRE até ao fim: ela é o padrão contra o qual os dois atalhos se
        // medem, e um atalho que também vivesse aqui não poderia ser medido.
        let _ = aplica(p, saida, &mut novas, 0.0);
    }
}
