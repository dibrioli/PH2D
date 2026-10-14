//! **Os gestos de uma macro de tutorial do Motion** — FORMAS (`source.shape` → `motion.duplicator`) e
//! uma SIMULAÇÃO com campo (`motion.integrate` ⇄ `field.box` → `force.wind` → `force.drag`): o gesto,
//! a intenção que o painel emitiria, e a [`CENA`] de `PH2D_MACRO_SMOKE=2` ([`super::smoke`]). A
//! fixture do gate 3 (`gate_tests::MACRO`) usa os mesmos gestos.

use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::GraphIntent;

/// Um gesto da macro. Os nós referem-se por ORDEM DE CRIAÇÃO (slot).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Gesto {
    Add(&'static str),
    Liga(usize, u16, usize, u16),
    Param(usize, &'static str, f32),
}

/// **A macro da cena `PH2D_MACRO_SMOKE=2`** — os mesmos oito nós, por uma ordem em que cada gesto se
/// vê: a saída liga-se cedo (uma forma, depois a grelha), e o vento só sopra quando a caixa e o
/// arrasto já o seguram (sem eles, empurrava a grelha inteira para fora da tela em 2 s).
/// Slots: 0 forma · 1 duplicador · 2 saída · 3 grelha · 4 integrador · 5 arrasto · 6 caixa · 7 vento.
pub(crate) const CENA: &[Gesto] = &[
    Gesto::Add("source.shape"),
    Gesto::Param(0, "size", 0.12),
    Gesto::Add("motion.duplicator"),
    Gesto::Liga(0, 0, 1, 0),
    Gesto::Add("motion.output"),
    Gesto::Liga(1, 0, 2, 0),
    Gesto::Add("motion.grid"),
    Gesto::Liga(3, 0, 1, 1),
    Gesto::Param(3, "rows", 6.0),
    Gesto::Param(3, "cols", 10.0),
    Gesto::Param(3, "gap_x", 0.35),
    Gesto::Param(3, "gap_y", 0.35),
    Gesto::Add("motion.integrate"),
    Gesto::Liga(3, 0, 4, 0),
    Gesto::Liga(4, 0, 1, 1),
    Gesto::Add("force.drag"),
    Gesto::Add("field.box"),
    Gesto::Param(6, "width", 1.2),
    Gesto::Add("force.wind"),
    Gesto::Liga(6, 0, 7, 0),
    Gesto::Liga(7, 0, 5, 0),
    // O último elo: o editor fecha o laço do integrador à cabeça da cadeia (a caixa), e o vento
    // sopra na coluna dentro dela.
    Gesto::Liga(5, 0, 4, 1),
    Gesto::Param(6, "width", 2.4),
];

/// A intenção que o painel emitiria para o gesto `i`, com os ids da sessão.
pub(crate) fn intent_de(g: Gesto, slots: &[NodeId], i: usize) -> GraphIntent {
    match g {
        Gesto::Add(t) => GraphIntent::AddNode {
            type_name: t,
            x: i as f32 * 120.0,
            y: 0.0,
        },
        Gesto::Liga(a, ap, b, bp) => GraphIntent::Connect {
            from_node: slots[a].0,
            from_port: ap,
            to_node: slots[b].0,
            to_port: bp,
        },
        Gesto::Param(n, k, v) => GraphIntent::SetParam {
            node: slots[n].0,
            param: k,
            value: v,
        },
    }
}
