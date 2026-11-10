//! **O que um estado pinta sobre a superfície de um controlo**, nos dois temas (estado da arte): a
//! camada do hover (Material 3, «state layers»).
//! Porquê: `docs/CodeEditor/handoffs/HANDOFF_line_ScriptEditor_2026-11-10_ACABAMENTO.md` §A.

/// A opacidade da camada de HOVER: a tinta do texto da superfície, a 8 % por cima dela (o
/// `state-layer` de hover do Material 3). Num tema escuro a linha CLAREIA; num claro escurece (a
/// superfície de um tema claro já é a mais clara).
pub const HOVER_LAYER_ALPHA: f32 = 0.08;
