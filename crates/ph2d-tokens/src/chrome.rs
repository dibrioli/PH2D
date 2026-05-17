//! Chrome layout dimensions. Source: `docs/design/tokens.json` → `chrome.*`.
//!
//! Wave 5 stage A: every fixed dimension that previously lived as a
//! hardcoded `f32` literal in `screens/hero/style.rs` or in widget
//! modules (`tool_rail`, `pill_group`, `checkbox`) now flows from
//! tokens.json. Designer fully owns the dimensional side of UI.
//!
//! The three originals (`ROW_H_PX`, `ICON_BTN_SIZE_PX`,
//! `SECTION_GAP_PX`) remain in `spacing.rs` for source compatibility;
//! semantically they are also chrome dims, but the imports were stable
//! across the rest of the workspace by Wave 4.

/// Default mockup viewport width (iPad 12.9 landscape).
/// Per tokens.json `chrome.hero-viewport-w`.
pub const HERO_VIEWPORT_W_PX: f32 = crate::generated::CHROME_HERO_VIEWPORT_W;

/// Default mockup viewport height (iPad 12.9 landscape).
/// Per tokens.json `chrome.hero-viewport-h`.
pub const HERO_VIEWPORT_H_PX: f32 = crate::generated::CHROME_HERO_VIEWPORT_H;

/// Padding from the screen edge to chrome (TopBar inset, Hierarchy
/// pinned-right inset, etc). Per tokens.json `chrome.edge-pad`.
pub const EDGE_PAD_PX: f32 = crate::generated::CHROME_EDGE_PAD;

/// TopBar chrome height. Per tokens.json `chrome.topbar-h`.
pub const TOPBAR_H_PX: f32 = crate::generated::CHROME_TOPBAR_H;

/// Inter-cluster gap inside the TopBar. Per tokens.json `chrome.topbar-gap`.
pub const TOPBAR_GAP_PX: f32 = crate::generated::CHROME_TOPBAR_GAP;

/// Inspector panel fixed width. Per tokens.json `chrome.inspector-w`.
pub const INSPECTOR_W_PX: f32 = crate::generated::CHROME_INSPECTOR_W;

/// Hierarchy panel fixed width. Per tokens.json `chrome.hierarchy-w`.
pub const HIERARCHY_W_PX: f32 = crate::generated::CHROME_HIERARCHY_W;

/// Bottom HUD strip height. Per tokens.json `chrome.hud-h`.
pub const HUD_H_PX: f32 = crate::generated::CHROME_HUD_H;

/// Bottom HUD inset from the viewport bottom edge.
/// Per tokens.json `chrome.hud-bottom-pad`.
pub const HUD_BOTTOM_PAD_PX: f32 = crate::generated::CHROME_HUD_BOTTOM_PAD;

/// Floating-panel corner radius. Per tokens.json `chrome.panel-radius`.
pub const PANEL_RADIUS_PX: f32 = crate::generated::CHROME_PANEL_RADIUS;

/// Floating-panel header inset. Per tokens.json `chrome.panel-head-pad`.
pub const PANEL_HEAD_PAD_PX: f32 = crate::generated::CHROME_PANEL_HEAD_PAD;

/// Hierarchy row height. Per tokens.json `chrome.hier-row-h`.
pub const HIER_ROW_H_PX: f32 = crate::generated::CHROME_HIER_ROW_H;

/// Panel resize-gripper hit-zone square edge.
/// Per tokens.json `chrome.panel-resize-handle-size`.
pub const PANEL_RESIZE_HANDLE_SIZE_PX: f32 = crate::generated::CHROME_PANEL_RESIZE_HANDLE_SIZE;

/// LeftRail tool chip square edge. Per tokens.json `chrome.tool-chip`.
pub const TOOL_CHIP_PX: f32 = crate::generated::CHROME_TOOL_CHIP;

/// LeftRail divider gap. Per tokens.json `chrome.divider-gap`.
pub const DIVIDER_GAP_PX: f32 = crate::generated::CHROME_DIVIDER_GAP;

/// Padding inside a `PillGroup`. Per tokens.json `chrome.pill-padding`.
pub const PILL_PADDING_PX: f32 = crate::generated::CHROME_PILL_PADDING;

/// Checkbox box edge length. Per tokens.json `chrome.checkbox-box`.
pub const CHECKBOX_BOX_PX: f32 = crate::generated::CHROME_CHECKBOX_BOX;
