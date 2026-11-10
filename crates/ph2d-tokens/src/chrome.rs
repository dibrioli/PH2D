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
/// A célula do nome do Summary, mais forte que a fita — o «chip» do canal-mestre, idioma do Blender
/// (escolha do dono, 09/11, candidato C).
pub const TIMELINE_SUMMARY_CELL_ALPHA: f32 = 0.45; // LITERAL-PX-OK: opacidade da célula do Summary

/// **A chave da Timeline nos cartões** (dono 09/11: «muito pequenos»). Maquete B1 medida em chrome
/// headless: losango de ~11 px com contorno escuro de ~1 px numa linha de 22. Aqui um pouco maior.
pub const TIMELINE_KEY_HALF_PX: f32 = 6.5; // LITERAL-PX-OK: meia-diagonal do losango da chave
/// O contorno escuro à volta do losango (lê-se sobre a barra da interpolação).
pub const TIMELINE_KEY_RING_PX: f32 = 1.5; // LITERAL-PX-OK: contorno do losango da chave
/// O alvo de clique da chave: o losango com o contorno inteiro (6,5 + 1,5) e o encaixe ao splitter
/// (`TIME_GUTTER` 12 − 8 = o `SPLIT_GRIP` de 4).
pub const TIMELINE_KEY_HIT_HALF_PX: f32 = 8.0; // LITERAL-PX-OK: meia-largura do alvo da chave
/// **A moldura da área das trilhas** (dono 09/11: «uma outline como tem na referência»): maquete B1
/// medida, um pixel cheio e meio antialiasado, raio do cartão.
pub const TIMELINE_FRAME_PX: f32 = 1.5; // LITERAL-PX-OK: espessura da moldura medida na maquete
/// O ponto do cabeçalho do objecto (maquete: ~7 px).
pub const TIMELINE_OBJECT_DOT_HALF_PX: f32 = 4.0; // LITERAL-PX-OK: meia-diagonal do ponto do objecto
/// A peça (strip) no Arrange/Containers, medida no CSS da maquete B (onda 3): o fundo é a cor da peça
/// misturada à fita de 58 % (em cima) a 34 % (em baixo); o contorno, a cor a 80 %.
pub const TIMELINE_PECA_TOPO_ALPHA: f32 = 0.58; // LITERAL-PX-OK: opacidade do topo da peça
/// O pé do gradiente da peça.
pub const TIMELINE_PECA_PE_ALPHA: f32 = 0.34; // LITERAL-PX-OK: opacidade do pé da peça
/// O contorno da peça.
pub const TIMELINE_PECA_BORDA_ALPHA: f32 = 0.8; // LITERAL-PX-OK: opacidade do contorno da peça
/// A peça de uma faixa muda (`.c13st.mu`).
pub const TIMELINE_PECA_MUDA_ALPHA: f32 = 0.4; // LITERAL-PX-OK: opacidade da peça muda
/// A sobreposição de duas peças (a mistura): listras claras e o contorno claro.
pub const TIMELINE_MISTURA_ALPHA: f32 = 0.34; // LITERAL-PX-OK: opacidade das listras da mistura
/// O contorno da mistura.
pub const TIMELINE_MISTURA_BORDA_ALPHA: f32 = 0.55; // LITERAL-PX-OK: opacidade do contorno da mistura
/// O raio da peça e da pílula do peso (`border-radius` 5 e 4 da maquete).
pub const TIMELINE_PECA_RAIO_PX: f32 = 5.0; // LITERAL-PX-OK: raio da peça
/// O raio da pílula do peso.
pub const TIMELINE_PESO_RAIO_PX: f32 = 4.0; // LITERAL-PX-OK: raio da pílula do peso
/// A altura de uma faixa (lane) e de uma linha da lista de containers nos cartões (a maquete B: 30 px, a
/// peça de 21 px dentro dela). As linhas da aba Keys continuam em [`crate::ROW_H_PX`].
pub const TIMELINE_FAIXA_H_PX: f32 = 30.0; // LITERAL-PX-OK: altura da faixa medida na maquete
/// A margem de cima e de baixo da peça dentro da faixa (`top:4px; bottom:4px`).
pub const TIMELINE_PECA_MARGEM_PX: f32 = 4.0; // LITERAL-PX-OK: margem vertical da peça na faixa
/// A barra do tamanho de um container na lista (3 px, raio 2).
pub const TIMELINE_LISTA_BARRA_PX: f32 = 3.0; // LITERAL-PX-OK: altura da barra do container na lista
/// ⭐ **Os estados de uma alça da Timeline** (dono 10/11). O hover aproxima a alça da tinta da chave
/// (a mais clara da folha) por esta fracção: acesa sem perder a cor da operação.
pub const TIMELINE_ALCA_HOVER_MIX: f32 = 0.5; // LITERAL-PX-OK: fracção da mistura repouso → tinta da chave
/// O halo do arrasto: o acento sob a alça, alargado, a esta opacidade.
pub const TIMELINE_ALCA_HALO_ALPHA: f32 = 0.35; // LITERAL-PX-OK: opacidade do halo do arrasto
/// A guia do arrasto (o fio de acento no tempo arrastado).
pub const TIMELINE_ALCA_GUIA_ALPHA: f32 = 0.8; // LITERAL-PX-OK: opacidade da guia do arrasto
/// ⭐ **Além do fim da animação** (dono 10/11): o véu da fita funda sobre o que passa do fim (apaga a
/// peça, como a área fora do trabalho do After Effects) e as listras claras a 45° por cima dele (a
/// mistura vai a 135°). Pisos medidos em `pecas_cartoes_tests` (12 temas × 6 peças + fita).
pub const TIMELINE_ALEM_VEU_ALPHA: f32 = 0.65; // LITERAL-PX-OK: opacidade do véu além do fim
/// As listras de «além do fim».
pub const TIMELINE_ALEM_LISTRA_ALPHA: f32 = 0.42; // LITERAL-PX-OK: opacidade das listras além do fim
/// A espessura de uma listra de «além do fim».
pub const TIMELINE_ALEM_LISTRA_PX: f32 = 2.0; // LITERAL-PX-OK: espessura da listra além do fim
/// O passo horizontal entre duas listras de «além do fim» (mais largo que o da mistura).
pub const TIMELINE_ALEM_PASSO_PX: f32 = 12.0; // LITERAL-PX-OK: passo das listras além do fim
/// A linha do fim da animação (a da ⟷ da duração), régua e faixas inteiras.
pub const TIMELINE_FIM_LINHA_PX: f32 = 2.0; // LITERAL-PX-OK: espessura da linha do fim
/// ⭐ **O laço na régua** (maquete B, `.c13lp`): uma faixa FINA no topo da régua, não a régua inteira —
/// os números ficam na régua sem tinta por baixo. Altura e opacidade da maquete.
pub const TIMELINE_LACO_REGUA_PX: f32 = 5.0; // LITERAL-PX-OK: altura da faixa do laço na régua
/// A opacidade da faixa do laço na régua.
pub const TIMELINE_LACO_REGUA_ALPHA: f32 = 0.7; // LITERAL-PX-OK: opacidade da faixa do laço
/// ⭐ **O trecho entre marcadores** (maquete B6, `.c13sec`): a cor do laço a 20 % com fio de 1 px a 65 %.
pub const TIMELINE_TRECHO_FUNDO_ALPHA: f32 = 0.2; // LITERAL-PX-OK: fundo da caixa do trecho
/// O fio da caixa do trecho (e do vão sob o ponteiro).
pub const TIMELINE_TRECHO_BORDA_ALPHA: f32 = 0.65; // LITERAL-PX-OK: fio da caixa do trecho
/// A folga de cada lado do enquadrar da Timeline (`F` e o abrir), em fracção do que se enquadra: o fim
/// do clipe e um pouco de «além do fim» ficam à vista (estado da arte, ~5–8 %).
pub const TIMELINE_ENQUADRE_MARGEM: f32 = 0.05; // LITERAL-PX-OK: folga do enquadrar, fracção do vão

/// ⭐ **O FANTASMA de um cartão arrastado — a opacidade dele.** Meio transparente: a marca da
/// queda (a barra de acento entre dois cartões) tem de se ver através dele.
pub const DRAG_GHOST_ALPHA: f32 = 0.55; // LITERAL-PX-OK: opacidade do fantasma

// ── onda 1 funções novas — strip (doc 17 item 4) ──
/// ⭐ **As voltas do loop dentro da peça** (maquete B4, `.c13lm`): um risco pontilhado a cada recomeço.
pub const TIMELINE_VOLTA_PX: f32 = 1.5; // LITERAL-PX-OK: espessura do risco da volta, a da maquete
/// A opacidade do risco da volta (`rgba(255,255,255,.7)` da maquete).
pub const TIMELINE_VOLTA_ALPHA: f32 = 0.7; // LITERAL-PX-OK: opacidade do risco da volta
/// Voltas mais juntas que isto não se pintam: a 1,5 px de risco e pontos de 1,5 px, abaixo de 4× a
/// espessura sobram < 4,5 px de corpo entre riscos e a peça vira hachura (o zoom afastado da cena 4).
pub const TIMELINE_VOLTA_MIN_PX: f32 = 6.0; // LITERAL-PX-OK: vão mínimo entre dois riscos de volta
