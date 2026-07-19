//! A geometria da **quina viva** (Live Corners) que as ferramentas Fillet / Chamfer usam —
//! módulo irmão do `PenTool` (LOC cap).
//!
//! [`frame_at_flat`] traduz o índice PLANO de um vértice (o que o hit-test e a seleção usam)
//! para o [`CornerFrame`] que o motor de arredondamento quer: a âncora, a bissetriz, o recuo
//! atual e o teto. O press das ferramentas de quina (`on_press_corner`) e o arrasto de raio
//! (`on_drag`, `Part::Radius`) chamam esta função — o predicado `corner_at` é o MESMO do
//! cozimento, então uma quina que o motor não arredondaria não vira alvo do gesto.
//!
//! NOTA: a antiga ALÇA de raio na bissetriz (`view` / `handle_pos_local`) foi REMOVIDA junto
//! com o modo Node de arredondar — o gesto agora é clicar-e-arrastar sobre a própria quina.
//! O que sobrou é a geometria da quina (`frame_at_flat`), compartilhada pelas duas ferramentas.

use ph2d_vec_scene::VecPath;
use ph2d_vec_scene::corner_live::CornerFrame;

/// A quina do vértice PLANO `i` deste path, se existir. Traduz o índice plano (que o
/// hit-test e a seleção usam) para o `(contorno, índice local)` que o motor quer, e devolve
/// `None` onde não há quina arredondável (ponta de caminho aberto, ou quina colinear).
#[must_use]
pub fn frame_at_flat(path: &VecPath, i: usize) -> Option<CornerFrame> {
    let (c, local) = path.locate_vert(i)?;
    let (verts, closed) = path.contour(c)?;
    ph2d_vec_scene::corner_live::corner_at(verts, closed, local)
}
