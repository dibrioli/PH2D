//! Image Tools action pills (IMAGE_ACTION_*) + tool panel markers (CEQ/EQS/UPS).
use super::{NodeId, hash_node_id};

/// Image Tools action — Trim Transparency pill. Lives in the action
/// row that replaces the right-side TopBar clusters when
/// `image_tools_mode` is on. Click is no-op for now — wiring to the
/// `ph2d_tool_trim_transparency::trim_transparency()` on a selected sprite
/// requires the live asset model (out of scope for this PR).
// Wave 2 PR 11.4: the three Image Tools action pills are now derived
// from the `image_tools` cluster in the runtime registry. To make
// hand-written click dispatch (`id == ids::IMAGE_ACTION_*`) work
// against registry-derived pills, each chrome const hashes the SAME
// slug as the matching tool manifest's `id` field. The
// `chrome_manifest_coverage` integration test pins this contract.
pub const IMAGE_ACTION_TRIM: NodeId = hash_node_id("trim_transparency");

/// Image Tools action — Make Square pill. Sibling of `IMAGE_ACTION_TRIM`,
/// pads the selected sprite with transparent pixels on the shorter axis
/// so width == height. Click raises `pending_make_square` on `HeroScreen`;
/// host drains, runs the algorithm, replaces sprite pixels + reprojects
/// pivot. Algorithm in crate `ph2d-tool-make-square` (ADR-0040 T1.5).
pub const IMAGE_ACTION_MAKE_SQUARE: NodeId = hash_node_id("make_square");

/// Image Tools action — Background Removal pill. Unlike `IMAGE_ACTION_TRIM`
/// and `IMAGE_ACTION_MAKE_SQUARE` (one-shot algorithms), this one
/// ACTIVATES the stateful `BgRemovalTool` so its floating panel opens
/// at the BottomCenter with a live 160×160 preview. Click raises
/// `pending_activate_bgremoval` on `HeroScreen`; host drains via
/// `tools.set_active(ToolId::new("bgremoval"))` and force-refreshes
/// the snapshot push.
pub const IMAGE_ACTION_BGREMOVAL: NodeId = hash_node_id("bgremoval");

/// Image Tools action — Real Size pill. One-shot like Trim / Make Square:
/// resets the selected sprite's `Transform.scale` to 1:1 (preserving flip
/// sign). Click raises `EditorAction::OneShotImageOp { tool_id: "real_size" }`; the shell drain mutates
/// the ECS `Transform`. Algorithm in `ph2d-tool-real-size`.
pub const IMAGE_ACTION_REAL_SIZE: NodeId = hash_node_id("real_size");

/// Image Tools action — Padding pill. Unlike the one-shots, this ACTIVATES
/// the stateful Padding tool (panel with 4 signed per-edge fields + Apply;
/// the directional-expand gizmo edge-drag is a v2). Click raises
/// `EditorAction::ActivateTool { tool_id: "padding" }`; the shell sets the tool active.
/// Condenses the legacy Image Padding + Directional Expand.
pub const IMAGE_ACTION_PADDING: NodeId = hash_node_id("padding");

/// Color Equalization panel marker NodeId. Right-docked in the
/// Inspector geometry slot while the `color_equalization` tool is
/// active. Hash matches `ph2d_tool_color_equalization::ids::CEQ_PANEL`
/// (the tool crate owns the canonical const for its own widgets;
/// editor-core mirrors it here so `paint_hero_screen`'s z_order
/// fallback can walk the panel without a circular dep on the tool
/// crate). Same hash key (`"panel.color_equalization"`), same
/// resolved id.
pub const CEQ_PANEL: NodeId = hash_node_id("panel.color_equalization");

/// Equalize Sizes panel marker NodeId. Mirror of `CEQ_PANEL` for the
/// multi-sprite size-normalization tool. Hash matches
/// `ph2d_tool_equalize_sizes::ids::EQS_PANEL` and
/// `ph2d_panel_equalize_sizes::EqualizeSizesPanel::NODE_ID` — keeping
/// the dispatcher's `panel_at` lookup, the typed panel registry, and
/// `paint_hero_screen`'s z_order fallback consistent.
pub const EQS_PANEL: NodeId = hash_node_id("panel.equalize_sizes");

/// Image Tools action — Color Equalization pill. Stateful tool: opens
/// the right-docked panel with 5 slider+chip rows (clip limit, tile
/// grid size, brightness, contrast, saturation), an Auto-WB toggle,
/// and Cancel/Apply. Pipeline (CPU, zero-deps): CLAHE (Zuiderveld
/// 1994), then brightness/contrast/saturation in linear sRGB, then
/// optional Gray-World auto-WB. Click raises `EditorAction::ActivateTool
/// { tool_id: "color_equalization" }`; Apply pushes one
/// `EditorAction::OneShotImageOp { tool_id: "color_equalization",
/// entity_bits }` per selected sprite, and the shell drain reads each
/// sprite's source then bakes via the tool's `run_full_resolution`.
pub const IMAGE_ACTION_COLOR_EQUALIZATION: NodeId = hash_node_id("color_equalization");

/// Image Tools action — Equalize Sizes pill. Stateful, multi-sprite:
/// opens the right-docked panel with target-mode radio (Max/Fixed/Grid),
/// per-mode chips / slider, Upscale-if-smaller + algorithm radio,
/// Rasterize-after toggle, Cancel/Apply. Click raises
/// `EditorAction::ActivateTool { tool_id: "equalize_sizes" }`; Apply
/// arms the tool's latch which the bridge drains into a single
/// `run_full_resolution_multi` over `hero.gizmo.iter_selected()` (the
/// only cross-sprite Image Tool — Max/Grid modes compute global
/// targets, so per-sprite `OneShotImageOp` broadcast won't work here).
pub const IMAGE_ACTION_EQUALIZE_SIZES: NodeId = hash_node_id("equalize_sizes");

/// Image Tools action — Rasterize pill. One-shot: bakes the sprite's
/// active Transform (scale + rotation + flip) into the source pixel
/// buffer and resets `Transform.scale = (1,1)` / `rotation = 0`. Click
/// raises one `EditorAction::OneShotImageOp { tool_id: "rasterize",
/// entity_bits }` per selected sprite; the shell drain calls
/// `ph2d_tool_rasterize::rasterize`, commits via
/// `texture_edit::commit_edited_texture`, then writes the identity
/// Transform.
pub const IMAGE_ACTION_RASTERIZE: NodeId = hash_node_id("rasterize");

/// Upscale panel marker NodeId. Right-docked in the Inspector geometry
/// slot while the `upscale` tool is active. Hash matches
/// `ph2d_tool_upscale::tool::ids` namespace and
/// `ph2d_panel_upscale::UpscalePanel::NODE_ID` — keeping the
/// dispatcher's `panel_at` lookup, the typed panel registry, and
/// `paint_hero_screen`'s z_order fallback consistent.
pub const UPS_PANEL: NodeId = hash_node_id("panel.upscale");

/// Image Tools action — Upscale pill. Stateful, sabor 3: opens the
/// right-docked panel with a 3-way algorithm radio (Lanczos3 / Nearest
/// / xBR), a 1×–16× scale slider paired with a number chip, and
/// Cancel / Apply. Click raises `EditorAction::ActivateTool { tool_id:
/// "upscale" }`; Apply arms the tool's latch, and the bridge drains
/// it per-sprite via `UpscaleTool::run_full_resolution`.
pub const IMAGE_ACTION_UPSCALE: NodeId = hash_node_id("upscale");

/// Image Tools action — Painter pill. Stateful workhorse (sucessor do
/// Procreate, cascata W0 ratificada 2026-05-26 — ADRs 0043..0053). Click
/// raises `EditorAction::ActivateTool { tool_id: "painter" }`; activation
/// installs the takeover model (suprime chrome PH2D normal + instala
/// chrome Procreate-style + sidebar — vide ADR-0043 §1.1).
pub const IMAGE_ACTION_PAINTER: NodeId = hash_node_id("painter");
