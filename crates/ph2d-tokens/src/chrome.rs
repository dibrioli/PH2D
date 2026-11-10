//! Chrome layout dimensions. Source: `docs/design/tokens.json` → `chrome.*`.
//!
//! Wave 5 stage A: every fixed dimension that previously lived as a
//! hardcoded `f32` literal in `screens/hero/style.rs` or in widget
//! modules (`tool_rail`, `topbar`, `checkbox`) now flows from
//! tokens.json. Designer fully owns the dimensional side of UI.
//!
//! The three originals (`ROW_H_PX`, `ICON_BTN_SIZE_PX`,
//! `INLINE_ICON_PX`) remain in `spacing.rs` for source compatibility;
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

/// **A largura mais ESTREITA a que um painel pode ser arrastado.** Per tokens.json
/// `chrome.panel-min-w`.
///
/// ⚠️ **Ele é o PIOR CASO de quem desenha dentro de um painel** — um conteúdo que só cabe na
/// largura confortável parte-se quando o artista aperta a janela. Quem conta quantas coisas
/// cabem numa linha conta contra ESTE número, e não contra a largura de hoje. Primeiro leitor:
/// o teto de paradas do editor de gradiente (bloco Z, doc 91), que era um `8` sem derivação.
pub const PANEL_MIN_W_PX: f32 = crate::generated::CHROME_PANEL_MIN_W;

/// Panel resize-gripper hit-zone square edge.
/// Per tokens.json `chrome.panel-resize-handle-size`.
pub const PANEL_RESIZE_HANDLE_SIZE_PX: f32 = crate::generated::CHROME_PANEL_RESIZE_HANDLE_SIZE;

/// LeftRail tool chip square edge. Per tokens.json `chrome.tool-chip`.
pub const TOOL_CHIP_PX: f32 = crate::generated::CHROME_TOOL_CHIP;

/// LeftRail divider gap. Per tokens.json `chrome.divider-gap`.
pub const DIVIDER_GAP_PX: f32 = crate::generated::CHROME_DIVIDER_GAP;

/// Padding das **pílulas de acção do topbar**. Per tokens.json `chrome.pill-padding`.
///
/// ⚠️ O nome vem de um widget `PillGroup` que **deixou de existir** em 2026-09-03 (zero
/// consumidores, decisão do Enio). O token FICA porque tem consumidores a sério — o
/// `image_action_row` e o `topbar` —, e é deles que ele fala agora.
pub const PILL_PADDING_PX: f32 = crate::generated::CHROME_PILL_PADDING;

/// Checkbox box edge length. Per tokens.json `chrome.checkbox-box`.
pub const CHECKBOX_BOX_PX: f32 = crate::generated::CHROME_CHECKBOX_BOX;

/// ⭐ **O FANTASMA de um cartão arrastado — a fracção do tamanho dele** (ordem do dono,
/// 2026-09-30: *«permita ver o card sendo arrastado, menor e meio transparente»*).
///
/// ⚠️ Adimensional: é uma ESCALA aplicada ao cartão inteiro (fundo e conteúdo) à volta do ponto
/// por onde a mão o pegou. `0,75` lê-se ainda como *o mesmo cartão* e deixa ver por baixo onde ele
/// vai cair — menor que isto o texto do fantasma deixa de ser legível numa coluna de ~260 px.
pub const DRAG_GHOST_SCALE: f32 = 0.75; // LITERAL-PX-OK: escala adimensional do fantasma

/// **A linha escolhida de uma lista: o acento a esta opacidade** (dono, 06/11: o tom antigo diferia
/// do cartão em 5 a 8 pontos de cor). `color_tests` mede o contraste em todo tema.
pub const ROW_SELECTED_ALPHA: f32 = 0.35; // LITERAL-PX-OK: opacidade do realce

/// **A linha «Summary» da Timeline: `timeline-summary-row` a esta opacidade sobre o `Bg0`** (dono,
/// 09/11: a 0,15 sobre o cabeçalho lia-se cinza sujo). Medida nos 8 temas (OKLab): a 0,25 o pior caso
/// contra cabeçalho, zebra e fundo é 0,045; a 0,20 cai a 0,020 (handoff da onda 2, 2e).
pub const TIMELINE_SUMMARY_TINT_ALPHA: f32 = 0.25; // LITERAL-PX-OK: opacidade da tinta do Summary

/// **A chave da Timeline nos cartões** (dono 09/11: «muito pequenos»). Maquete B1 medida em chrome
/// headless: losango de ~11 px com contorno escuro de ~1 px numa linha de 22. Aqui um pouco maior.
pub const TIMELINE_KEY_HALF_PX: f32 = 6.5; // LITERAL-PX-OK: meia-diagonal do losango da chave
/// O contorno escuro à volta do losango (lê-se sobre a barra da interpolação).
pub const TIMELINE_KEY_RING_PX: f32 = 1.5; // LITERAL-PX-OK: contorno do losango da chave
/// O alvo de clique da chave: o losango com o contorno inteiro (6,5 + 1,5) e o encaixe ao splitter
/// (`TIME_GUTTER` 12 − 8 = o `SPLIT_GRIP` de 4).
pub const TIMELINE_KEY_HIT_HALF_PX: f32 = 8.0; // LITERAL-PX-OK: meia-largura do alvo da chave
/// O ponto do cabeçalho do objecto (maquete: ~7 px).
pub const TIMELINE_OBJECT_DOT_HALF_PX: f32 = 4.0; // LITERAL-PX-OK: meia-diagonal do ponto do objecto

/// ⭐ **O FANTASMA de um cartão arrastado — a opacidade dele.** Meio transparente: a marca da
/// queda (a barra de acento entre dois cartões) tem de se ver através dele.
pub const DRAG_GHOST_ALPHA: f32 = 0.55; // LITERAL-PX-OK: opacidade do fantasma
