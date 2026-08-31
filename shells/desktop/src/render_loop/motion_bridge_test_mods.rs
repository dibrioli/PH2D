//! **Os módulos de TESTE do `motion_bridge`** — só as declarações, num irmão.
//!
//! ⚠️ **Este arquivo existe por um teto de LOC** (HR-18, 600 para `shells/desktop`), e o
//! corte é por RESPONSABILIDADE: o pai é a membrana (o que a ponte FAZ), aqui ficam os 22
//! irmãos que a medem. É o mesmo molde do `motion_state_conferencia_mods.rs`.
//!
//! ⚠️ **O `pub(crate) use super::*` é load-bearing:** cada um daqueles arquivos abre com
//! `use super::*`, e depois deste corte o `super` deles é ESTE módulo. Sem a re-exportação
//! eles deixariam de ver a membrana que testam — 22 arquivos a não compilar por uma linha.

#[allow(unused_imports, clippy::wildcard_imports)]
pub(crate) use super::*;

#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_arrange_tests.rs"]
mod arrange_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_backdrop_tests.rs"]
mod backdrop_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_bypass_tests.rs"]
mod bypass_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_channel_tests.rs"]
mod channel_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_colour_tests.rs"]
mod colour_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_connect_tests.rs"]
mod connect_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_dock_height_tests.rs"]
mod dock_height_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_dock_tests.rs"]
mod dock_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_edit_tests.rs"]
mod edit_tests;
/// O censo do TETO de opções — o mesmo do `rowcap_tests` um nível abaixo: aquele pina que a
/// ROW aparece, este que as OPÇÕES dentro dela aparecem.
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_enumcap_tests.rs"]
mod enumcap_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_library_tests.rs"]
mod library_tests;
/// O selector de MOLDES do `source.lsystem` — a resposta ao *"Axiom e Rules não são nada
/// intuitivos"* (Enio, 2026-08-28).
/// A costura da queixa: uma regra que o parser deita fora chega à row do painel.
#[path = "motion_bridge_lsystem_complaint_tests.rs"]
mod lsystem_complaint_tests;

#[path = "motion_bridge_lsystem_preset_tests.rs"]
mod lsystem_preset_tests;

/// O SELECTOR de moldes pelo despacho REAL — irmão cortado pelo teto de LOC (HR-18),
/// no corte que a pergunta desenha: lá *o que a tabela é*, aqui *o que o clique faz*.
#[path = "motion_bridge_lsystem_selector_tests.rs"]
mod lsystem_selector_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_param_tests.rs"]
mod param_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_plumbing_tests.rs"]
mod plumbing_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_range_tests.rs"]
mod range_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_rename_tests.rs"]
mod rename_tests;
/// O gesto de REVERTER um param ao default — a metade que vive na ponte (o conjunto de
/// modificados que o painel lê, pelos dois canais).
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_reset_tests.rs"]
mod reset_tests;
/// O censo do TETO de linhas — o terceiro irmão do `range_tests`/`unit_tests`: eles pinam a
/// escala e a unidade de um valor, este pina que o valor **aparece**.
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_rowcap_tests.rs"]
mod rowcap_tests;
/// **O que um fio alcanca numa FORMA** — o menu que so' oferece os knobs da especie escolhida,
/// e o fio que cai quando a especie muda (report do Enio, 2026-08-27).
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_shape_link_tests.rs"]
mod shape_link_tests;
/// O CENSO dos params de FORMA (curva, rampa, paleta, texto) — a espécie que a caça aos
/// knobs mortos não podia ver, porque ela varre o `MANIFEST` e uma forma não é um `ParamSpec`.
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_shape_reach_tests.rs"]
mod shape_reach_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_subgraph_ports_tests.rs"]
mod subgraph_ports_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_subgraph_tests.rs"]
mod subgraph_tests;
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_tests.rs"]
mod tests;
/// The display FACE of a row (doc 88) — a sibling of `range_tests` because it is
/// the same subject seen from the other side: that one pins the SCALE a value is
/// read against, this one pins the UNIT it is read in.
#[cfg(all(test, feature = "panel-motion-graph", feature = "panel-motion-params"))]
#[path = "motion_bridge_unit_tests.rs"]
mod unit_tests;
