//! [`ToolRail`] — vertical strip of editor tools.
//!
//! Three entry kinds:
//! - `Icon { id, icon, active }` — square 44x44 icon-only chip.
//! - `Compound { id, label, sub }` — chip with a body label (face)
//!   and a small uppercase mono sub-label below ("Global / SPACE",
//!   "Persp / PROJ", "Home / VIEW").
//! - `Divider` — 24x1 px line in `Border` color.
//!
//! AccessKit `Role::Toolbar` (vertical orientation hinted by the
//! parent layout — AccessKit doesn't expose orientation on Toolbar).

use crate::icons::IconId;
use crate::interaction::WidgetStore;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text_centered, paint_text_rotated_ccw, rect_to_vello,
    resolve, stroke_rounded_rect,
};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{
    ColorToken, DIVIDER_GAP_PX as CHROME_DIVIDER_GAP, Radius, Spacing, StrokeToken,
    TOOL_CHIP_PX as CHROME_TOOL_CHIP, Theme, TypeToken,
};
use ph2d_vector::VectorScene;

/// Width of the LeftRail. Tightly packed: label column on the left,
/// 44-px chip, small right margin. The chip's x is computed from
/// the label column budget so changing label padding only shifts
/// the chip, not the rail width.
/// ⚠️ `fn` e nao `const`: a escala e AUTORAVEL (plano UI/UX W4c.2).
pub fn tool_rail_width_px() -> f32 {
    CHIP_X_OFFSET_PX + TOOL_CHIP_PX + Spacing::Xs.px()
}
/// Per tokens.json `chrome.tool-chip`.
pub const TOOL_CHIP_PX: f32 = CHROME_TOOL_CHIP;
pub const COMPOUND_TOTAL_H_PX: f32 = TOOL_CHIP_PX; // sub-label moved to vertical-left
/// Per tokens.json `chrome.divider-gap`.
pub const DIVIDER_GAP_PX: f32 = CHROME_DIVIDER_GAP;
/// Padding from the rail's left edge to the vertical sub-label.
const LABEL_LEFT_PAD: f32 = 3.0; // LITERAL-PX-OK: rotated sub-label edge inset (chrome-specific)
/// Horizontal extent the rotated sub-label occupies on screen
/// (≈ parley line height for the chosen sub-label font). 11 px
/// fits the Xs - 2 font (8-9 px) plus its ascender/descender margin.
pub const LABEL_VISUAL_EXTENT_PX: f32 = 11.0; // LITERAL-PX-OK: rotated sub-label glyph extent (chrome-specific)
/// Gap between the right edge of the rotated sub-label and the
/// left edge of the chip.
///
/// ⚠️ **`pub` porque a topbar precisa dele.** Enquanto era privado, o `cluster_painter` tinha
/// **quatro literais** `11.0`/`3.0` com o comentário *«mirror of rail's …»* — e um espelho não é
/// uma lei: mudar a constante fazia a topbar discordar do rail **em silêncio**. Gate:
/// `the_topbar_reads_the_rail_constants_instead_of_mirroring_them`.
pub const LABEL_TO_CHIP_GAP_PX: f32 = 3.0; // LITERAL-PX-OK: sub-label → chip gap (chrome-specific)
/// Resulting chip-x offset from the rail's left edge. Public so
/// `left_rail::paint_left_rail`'s hit-register mirrors this exactly.
pub const CHIP_X_OFFSET_PX: f32 = LABEL_LEFT_PAD + LABEL_VISUAL_EXTENT_PX + LABEL_TO_CHIP_GAP_PX;

/// Runtime-configurable rail button size — surfaced in the Themes
/// menu (2026-05-24). `Small` is the canonical default
/// (matches [`TOOL_CHIP_PX`]); `Large` is the pre-2026-05-24 size;
/// `Medium` is the halfway point.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum RailButtonSize {
    #[default]
    Small,
    Medium,
    Large,
}

impl RailButtonSize {
    /// Chip edge (square) in px for this size preset.
    pub const fn chip_px(self) -> f32 {
        match self {
            Self::Small => 36.0,  // LITERAL-PX-OK: Themes-menu preset (Small)
            Self::Medium => 40.0, // LITERAL-PX-OK: Themes-menu preset (Medium)
            Self::Large => 44.0,  // LITERAL-PX-OK: Themes-menu preset (Large)
        }
    }

    /// Total rail column width for this size preset (mirrors
    /// [`tool_rail_width_px`] formula but using this size's chip px).
    pub fn rail_width_px(self) -> f32 {
        CHIP_X_OFFSET_PX + self.chip_px() + Spacing::Xs.px()
    }
}

#[derive(Clone, Debug)]
pub enum ToolRailEntry {
    Icon {
        id: NodeId,
        label: String,
        icon: IconId,
        active: bool,
        /// Short UPPERCASE tag painted vertically to the LEFT of
        /// the chip. Empty string means "no sub-label".
        sub: String,
    },
    Compound {
        id: NodeId,
        label: String,
        face: String,
        sub: String,
    },
    /// A colour-swatch chip: the chip is filled with `color` (a live colour box) instead of an icon,
    /// with the same state border + vertical sub-label as [`Self::Icon`]. Used by the painter rail's Fill
    /// button, which doubles as the colour selector.
    Swatch {
        id: NodeId,
        label: String,
        color: [u8; 4],
        active: bool,
        sub: String,
    },
    Divider,
}

impl ToolRailEntry {
    /// O id do chip. `None` só para o [`Self::Divider`], que não é clicável.
    ///
    /// Existe para o gate anti-botão-morto (`every_painted_rail_button_is_dispatched`): sem
    /// ele, a lista do gate seria escrita à mão e driftaria da lista que o rail pinta — que
    /// é exatamente como o botão Redo passou meses pintado, clicável e órfão.
    #[must_use]
    pub fn node_id(&self) -> Option<NodeId> {
        match self {
            Self::Icon { id, .. } | Self::Compound { id, .. } | Self::Swatch { id, .. } => {
                Some(*id)
            }
            Self::Divider => None,
        }
    }

    /// O nome humano do chip — `None` só para o [`Self::Divider`], que não é clicável.
    ///
    /// Irmão exato do [`Self::node_id`], e existe pelo mesmo motivo: a **paleta de comandos global**
    /// projeta a lista que o rail PINTA, e uma segunda tabela de nomes noutro sítio driftaria da
    /// tela — que é como o botão Redo passou meses pintado, clicável e órfão.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        match self {
            Self::Icon { label, .. }
            | Self::Compound { label, .. }
            | Self::Swatch { label, .. } => Some(label),
            Self::Divider => None,
        }
    }

    pub fn icon(id: NodeId, label: impl Into<String>, icon: IconId) -> Self {
        Self::Icon {
            id,
            label: label.into(),
            icon,
            active: false,
            sub: String::new(),
        }
    }

    /// A colour-swatch chip entry (the Fill button): the chip is the colour box.
    pub fn swatch(id: NodeId, label: impl Into<String>, color: [u8; 4]) -> Self {
        Self::Swatch {
            id,
            label: label.into(),
            color,
            active: false,
            sub: String::new(),
        }
    }

    /// Builder shortcut for the Icon/Swatch variants — sets the vertical
    /// sub-label tag (short uppercase, e.g. "MOVE", "ROT", "UNDO").
    /// (Named `with_sub` rather than `sub` to avoid colliding with
    /// `std::ops::Sub::sub`.)
    pub fn with_sub(mut self, sub: impl Into<String>) -> Self {
        match &mut self {
            Self::Icon { sub: s, .. } | Self::Swatch { sub: s, .. } => *s = sub.into(),
            _ => {}
        }
        self
    }

    pub fn compound(
        id: NodeId,
        label: impl Into<String>,
        face: impl Into<String>,
        sub: impl Into<String>,
    ) -> Self {
        Self::Compound {
            id,
            label: label.into(),
            face: face.into(),
            sub: sub.into(),
        }
    }

    /// Builder shortcut for the Icon/Swatch variants — flips `active` true.
    pub fn active(mut self) -> Self {
        match &mut self {
            Self::Icon { active, .. } | Self::Swatch { active, .. } => *active = true,
            _ => {}
        }
        self
    }

    /// Vertical extent this entry needs at the given button size.
    /// `size` is the runtime [`RailButtonSize`] (defaults to `Small`
    /// in the store's initial state).
    pub fn height(&self, size: RailButtonSize) -> f32 {
        match self {
            Self::Icon { .. } => size.chip_px(),
            Self::Compound { .. } => size.chip_px(),
            Self::Swatch { .. } => size.chip_px(),
            Self::Divider => 1.0 + DIVIDER_GAP_PX * 2.0,
        }
    }
}

/// ⭐⭐ **O EIXO em que um rail se dispõe** — a coluna de sempre, ou a fila horizontal por cima da
/// área de desenho (2026-08-30, o modelo do Godot).
///
/// ⚠️ **A ADVANCE é a mesma nos dois** (um chip anda `chip_px`, um divisor anda `1 + 2·gap`); o que
/// muda é o eixo em que ela corre e onde fica o rótulo — à esquerda do chip na coluna, por cima
/// dele na fila. Escrever duas aritméticas seria escrever a mesma lei duas vezes, que é
/// precisamente a dívida que este ficheiro tinha.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum RailAxis {
    /// A coluna lateral — o rótulo roda 90° e vive à ESQUERDA do chip.
    #[default]
    Vertical,
    /// A fila horizontal — o rótulo fica direito, POR CIMA do chip.
    Horizontal,
}

/// **Onde uma entrada do rail cai** — a resposta da porta única [`entry_rects`].
///
/// `id` é `None` só para o [`ToolRailEntry::Divider`], e nesse caso `rect` é a LINHA que ele
/// desenha, não um alvo: um divisor não se clica.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EntrySlot {
    /// O índice na lista de entradas — o que o `build_entry_a11y` pede.
    pub index: usize,
    /// O id do chip, ou `None` num divisor.
    pub id: Option<NodeId>,
    /// O rectângulo do chip em REPOUSO (ou a linha, num divisor).
    pub rect: Rect,
}

/// ⭐⭐ **A PORTA ÚNICA da geometria de um rail** — *onde cai cada entrada?*
///
/// ⛔⛔ **Ela existe porque a resposta estava escrita TRÊS vezes**, e nada no repo ligava as
/// cópias: o pintor (`tool_rail/paint.rs`), o registo de hit do trilho (`hero/left_rail.rs`) e o
/// registo de hit do flyout — cada um com o seu `let mut y`, o seu `gap` e o seu `chip_x`. O
/// comentário do segundo dizia *«Hit-rects MUST mirror exactly what `paint_tool_rail` paints»*,
/// que é a confissão do defeito: **um espelho não é uma lei**. Um pintor horizontal com um hit
/// vertical compilaria e passaria a suíte inteira.
///
/// ⚠️ **O rect devolvido é o de REPOUSO**, antes do `hover_lift`: o desenho cresce, o alvo não —
/// um alvo que se move debaixo do dedo é um alvo que foge.
#[must_use]
pub fn entry_rects(
    rail: &ToolRail,
    rect: Rect,
    size: RailButtonSize,
    axis: RailAxis,
) -> Vec<EntrySlot> {
    let gap = Spacing::Xs.px();
    let chip_px = size.chip_px();
    // O deslocamento no eixo TRANSVERSAL: na coluna o chip afasta-se da borda esquerda para dar
    // sítio ao rótulo rodado; na fila ele desce para o rótulo caber por cima.
    let cross = match axis {
        RailAxis::Vertical => rect.x + CHIP_X_OFFSET_PX,
        RailAxis::Horizontal => rect.y + LABEL_VISUAL_EXTENT_PX + LABEL_TO_CHIP_GAP_PX,
    };
    let mut along = match axis {
        RailAxis::Vertical => rect.y,
        RailAxis::Horizontal => rect.x,
    };
    let mut out = Vec::with_capacity(rail.entries.len());
    for (index, entry) in rail.entries.iter().enumerate() {
        if index > 0 {
            along += gap;
        }
        let (r, advance) = match entry {
            ToolRailEntry::Divider => {
                // O divisor é uma linha FINA no eixo, centrada no transversal — a mesma lei nos
                // dois eixos, com `w` e `h` trocados.
                let len = Spacing::Xl2.px();
                let line = match axis {
                    RailAxis::Vertical => Rect::new(
                        rect.x + (rect.w - len) * 0.5,
                        along + DIVIDER_GAP_PX,
                        len,
                        1.0,
                    ),
                    RailAxis::Horizontal => Rect::new(
                        along + DIVIDER_GAP_PX,
                        rect.y + (rect.h - len) * 0.5,
                        1.0,
                        len,
                    ),
                };
                (line, 1.0 + DIVIDER_GAP_PX * 2.0)
            }
            _ => {
                let chip = match axis {
                    RailAxis::Vertical => Rect::new(cross, along, chip_px, chip_px),
                    RailAxis::Horizontal => Rect::new(along, cross, chip_px, chip_px),
                };
                (chip, chip_px)
            }
        };
        out.push(EntrySlot {
            index,
            id: entry.node_id(),
            rect: r,
        });
        along += advance;
    }
    out
}

#[derive(Clone, Debug)]
pub struct ToolRail {
    pub id: NodeId,
    pub label: String,
    pub entries: Vec<ToolRailEntry>,
}

impl ToolRail {
    pub fn new(id: NodeId, label: impl Into<String>, entries: Vec<ToolRailEntry>) -> Self {
        Self {
            id,
            label: label.into(),
            entries,
        }
    }

    pub fn preferred_height(&self, size: RailButtonSize) -> f32 {
        let gap = Spacing::Xs.px();
        let mut total = 0.0_f32;
        for (i, e) in self.entries.iter().enumerate() {
            if i > 0 {
                total += gap;
            }
            total += e.height(size);
        }
        total
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let kids = self.entries.iter().filter_map(|e| match e {
            ToolRailEntry::Icon { id, .. }
            | ToolRailEntry::Compound { id, .. }
            | ToolRailEntry::Swatch { id, .. } => Some(*id),
            ToolRailEntry::Divider => None,
        });
        NodeBuilder::new(Role::Toolbar)
            .label(&self.label)
            .bounds(x, y, w, h)
            .children(kids)
            .build()
    }

    pub fn build_entry_a11y(&self, index: usize, x: f64, y: f64, w: f64, h: f64) -> Option<Node> {
        match self.entries.get(index)? {
            ToolRailEntry::Icon { id: _, label, .. } => Some(
                NodeBuilder::new(Role::Button)
                    .label(label)
                    .bounds(x, y, w, h)
                    .focusable(true)
                    .action(Action::Click)
                    .build(),
            ),
            ToolRailEntry::Compound { id: _, label, .. }
            | ToolRailEntry::Swatch { id: _, label, .. } => Some(
                NodeBuilder::new(Role::Button)
                    .label(label)
                    .bounds(x, y, w, h)
                    .focusable(true)
                    .action(Action::Click)
                    .build(),
            ),
            ToolRailEntry::Divider => None,
        }
    }
}

#[path = "tool_rail/paint.rs"]
mod paint;
pub use paint::{paint_tool_rail, paint_tool_rail_axis, paint_tool_rail_t};

#[cfg(test)]
mod tests;
