//! **As SEÇÕES do painel de params** (doc 88 B3) — a reordenação por grupo, onde cada
//! cabeçalho começa, e quais deles NASCEM FECHADOS.
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 600 para `shells/desktop`), no corte que
//! a pergunta desenha: lá fica *que rows este nó tem*, aqui *como elas se agrupam*.

use super::*;
use ph2d_panel_motion_params::ParamRow;
use std::collections::BTreeSet;

/// Reordena as rows por GRUPO e devolve `(rows, onde cada seção começa, quais nascem
/// fechadas)`.
///
/// ⚠️ **`sort_by_key` é ESTÁVEL**, então dentro de um grupo a ordem que o autor do nó
/// escreveu sobrevive — a alternativa (ordenar por nome) reescreveria a intenção dele. As
/// soltas ficam primeiro.
pub(super) fn split_into_sections(
    mut rows: Vec<ParamRow>,
    motion: &MotionState,
    type_id: ph2d_nodegraph::node::NodeTypeId,
) -> (Vec<ParamRow>, Vec<(String, usize)>, BTreeSet<String>) {
    let order = motion.registry.param_group_order(type_id);
    let group_of = |row: &ParamRow| -> Option<&'static str> {
        row.params()
            .first()
            .and_then(|p| motion.registry.param_group(type_id, p))
    };
    rows.sort_by_key(|r| {
        group_of(r).map_or(0, |g| {
            1 + order.iter().position(|o| *o == g).unwrap_or(order.len())
        })
    });
    // Onde cada seção começa. Uma seção cujo grupo não produziu row nenhuma (todo param dela
    // escondido por um `ParamGate`) simplesmente não aparece — cabeçalho sem conteúdo é a
    // seção-morta irmã do botão-morto.
    let mut sections: Vec<(String, usize)> = Vec::new();
    let mut prev: Option<&'static str> = None;
    for (i, row) in rows.iter().enumerate() {
        let g = group_of(row);
        if g != prev
            && let Some(g) = g
        {
            sections.push((g.to_string(), i));
        }
        prev = g;
    }
    // ⚠️ **Só os títulos que de facto PRODUZIRAM cabeçalho** — um grupo cujas rows estão
    // todas escondidas por um `ParamGate` não aparece, e semear a dobra dele deixaria uma
    // marca no store para uma seção que ninguém desenha.
    let folded = motion.registry.param_groups_folded(type_id);
    let folded_by_default = sections
        .iter()
        .filter(|(t, _)| folded.contains(&t.as_str()))
        .map(|(t, _)| t.clone())
        .collect();
    (rows, sections, folded_by_default)
}
