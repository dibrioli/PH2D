//! **AS PORTAS ≠ 0 NA ANCESTRALIDADE DE CADA SAÍDA** — irmã por RESPONSABILIDADE (HR-18) do
//! [`super::census::motion_route_census`], que diz QUE rota cada cena toma; esta diz, para as que
//! não vão inteiras à placa, se o caminho até à saída passa por uma aresta que sai de uma porta
//! `≠ 0` de um nó COM kernel (`gpu_kernel(ty).is_some()`).
//!
//! ⚠️ É a população sobre a qual o W1(b) — o COMPLEMENTO de um Compact — vai agir: medi-la antes
//! de construir é o §5.0 (*«antes de construir um item de lista aberta, MEÇA»*). Ela IMPRIME e não
//! julga: uma porta `≠ 0` na ancestralidade não prova que é ELA que morde.

use super::*;
use ph2d_nodegraph::gpu::KernelResolver;

/// `(cena, motivo da rota, arestas «tipo:porta → tipo» achadas)` — vazio = nenhuma.
pub(super) type Achado = (u32, &'static str, Vec<String>);

/// As arestas da ancestralidade das `sinks` que saem de uma porta `≠ 0` de um nó com kernel.
pub(super) fn portas_nao_zero(state: &MotionState, sinks: &[NodeId]) -> Vec<String> {
    let g = &state.doc.graph;
    let tipo = |id: NodeId| g.node(id).map(|n| n.type_name.clone()).unwrap_or_default();
    let mut vistos: std::collections::BTreeSet<NodeId> = sinks.iter().copied().collect();
    let mut fila: Vec<NodeId> = sinks.to_vec();
    let mut achadas = std::collections::BTreeSet::new();
    while let Some(n) = fila.pop() {
        for e in g.edges().iter().filter(|e| e.to.0 == n) {
            let (src, porta) = e.from;
            let com_kernel = g
                .node(src)
                .is_some_and(|m| state.registry.gpu_kernel(m.type_id()).is_some());
            if porta != 0 && com_kernel {
                achadas.insert(format!("{}:{porta} -> {}", tipo(src), tipo(n)));
            }
            if vistos.insert(src) {
                fila.push(src);
            }
        }
    }
    achadas.into_iter().collect()
}

/// A secção «portas ≠ 0» do censo de rota.
pub(super) fn imprime(achados: &[Achado]) {
    eprintln!("  ─── portas ≠ 0 na ancestralidade ───");
    let (com, sem): (Vec<&Achado>, Vec<&Achado>) =
        achados.iter().partition(|(_, _, a)| !a.is_empty());
    eprintln!(
        "  cenas fora do device inteiro │ {}   ·   com aresta de porta ≠ 0 (no' com kernel) │ {}",
        achados.len(),
        com.len()
    );
    for (cena, porque, arestas) in &com {
        eprintln!("    ={cena:<3} [{porque}]  {}", arestas.join(" · "));
    }
    let sem: Vec<u32> = sem.iter().map(|(c, _, _)| *c).collect();
    eprintln!("  sem nenhuma │ {sem:?}");
}
