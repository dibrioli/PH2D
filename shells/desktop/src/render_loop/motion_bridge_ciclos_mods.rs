//! **Os instrumentos da DINÂMICA DOS CICLOS** ([doc 103](../../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md))
//! — só as declarações, num irmão.
//!
//! ⚠️ **Este arquivo existe por um teto de LOC** (HR-18, 600 para `shells/desktop`), e o corte
//! é por RESPONSABILIDADE — o mesmo molde do `motion_bridge_test_mods.rs`: o pai é a membrana,
//! aqui ficam as sondas do **passo 5** (a medição de um grupo) e os geradores do **passo 6**
//! (as figuras e a tabela de um tutorial).
//!
//! ⚠️ **Os seis referem-se uns aos outros por `super::`**, e é por isso que eles têm de ficar
//! todos no MESMO pai: a tabela de controlos é uma porta só, lida pelos geradores dos dois
//! ciclos, e a figura da mola pede os ajudantes de grafo do gerador irmão.

/// A medição do grupo do ciclo aberto (doc 104) — sonda `#[ignore]`, não um gate.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_arranjo_probe.rs"]
mod arranjo_probe;

/// ⭐ A tabela de controlos de um tutorial — **a porta única**, lida pelos geradores dos DOIS
/// ciclos (doc 103 §3). Ela nasceu dentro do gerador do ciclo 1 e saiu antes de haver cópia.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_tutorial_table.rs"]
mod tutorial_table;

/// A figura da MOLA — irmã da de baixo por responsabilidade: ali uma nuvem num instante, aqui
/// uma curva no TEMPO, que é a única forma de mostrar um perseguidor.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_animadores_spring_fig.rs"]
mod animadores_spring_fig;

/// As figuras do tutorial do ciclo 2 — cozidas, e com o fantasma como régua.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_animadores_figures.rs"]
mod animadores_figures;

/// A medição do grupo do ciclo 2 (doc 105 §4) — sonda `#[ignore]`, não um gate.
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_animadores_probe.rs"]
mod animadores_probe;

/// As figuras do tutorial do ciclo 1 — geradas COZINHANDO os nós (doc 103 §3).
#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_bridge_arranjo_figures.rs"]
mod arranjo_figures;
