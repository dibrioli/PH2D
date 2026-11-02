//! **A fronteira tool → realce**: o [`crate::selection_overlay`] não depende da crate da tool, e a
//! tradução mora aqui (antes vivia na `fase_selection_highlight` da shell).

use crate::selection_overlay::OverlayDomain;
use ph2d_tool_flip::{EditDomain, FlipMode, FlipStyleSnapshot};

/// W8/§4.C: o realce fala a linguagem do DOMÍNIO — halo de traço (Stroke), dots (Point), ou halo do
/// PEDAÇO + preview de hover (Segment). Devolve também se a tool está em modo Edit.
#[must_use]
pub fn da_tool(style: Option<FlipStyleSnapshot>) -> (bool, OverlayDomain) {
    let edit = matches!(style.map(|s| s.mode), Some(FlipMode::Edit));
    let dominio = match style.map(|s| s.edit_domain) {
        Some(EditDomain::Point) => OverlayDomain::Point,
        Some(EditDomain::Segment) => OverlayDomain::Segment,
        _ => OverlayDomain::Stroke,
    };
    (edit, dominio)
}
