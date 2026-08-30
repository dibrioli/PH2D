//! **Os ids do cluster SETTINGS** (a engrenagem) — irmão do [`super::menus`] pelo teto de
//! LOC, e o corte é por assunto: aqui moram os ids das cascatas que a engrenagem abre.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho.** O `menus.rs` responde *"que
//! ids têm os menus do app"*; este responde *"…e os da engrenagem"*, que é o único
//! cluster do ficheiro com uma raiz só (`CTX_MENU_SETTINGS_*` → uma cascata cada) e o que
//! cresce a cada preferência nova. Precedente aberto pelo [`super::menus_timeline`].
//!
//! Nasceu quando a unidade de ÂNGULO (Enio, 2026-08-30) levou o `menus.rs` a 709 LOC
//! contra o teto de 700. ⭐ *O teto não pediu uma excepção: pediu o corte que este
//! ficheiro é.*
//!
//! ⚠️ Os ids continuam a sair do `hash_node_id`, então **mudar de ficheiro não muda
//! nenhum valor** — o `node_id_collisions` continua a ser quem prova a unicidade, e ele
//! varre a crate, não o ficheiro.

use ph2d_a11y::NodeId;

use super::hash_node_id;
// Pixels-per-meter presets — opened from the Settings cluster (gear).
// Drives `HeroScreen.project.pixels_per_meter`. The values are the
// canonical presets surfaced as labels in `SettingsMenu`.
//
// Pre-PR-11.3 these sat at hand-picked integers 940..944 to dodge
// the SceneList popover rows (CTX_SCENE_ROW_*) — exactly the type
// of cross-cluster collision the M14.4d audit caught when an early
// draft reused 930..934. Now hash-derived; the regression test in
// `tests/architecture/node_id_collisions.rs` catches reuse mechanically.
pub const CTX_MENU_PPM_16: NodeId = hash_node_id("ctx_menu_ppm_16");
pub const CTX_MENU_PPM_32: NodeId = hash_node_id("ctx_menu_ppm_32");
pub const CTX_MENU_PPM_100: NodeId = hash_node_id("ctx_menu_ppm_100");
pub const CTX_MENU_PPM_256: NodeId = hash_node_id("ctx_menu_ppm_256");
pub const CTX_MENU_PPM_1024: NodeId = hash_node_id("ctx_menu_ppm_1024");

/// M14.7 polish (6.3): top-level Settings cascade entry that opens
/// the Pixels-per-meter submenu.
pub const CTX_MENU_SETTINGS_PPM: NodeId = hash_node_id("ctx_menu_settings_ppm");

/// **Input Map…** — abre a janela flutuante do mapa de entradas (plano 30 §0.2).
///
/// ⚠️ **No menu Settings, e é onde o Godot o põe** (*Project Settings > Input Map*) — o pedido do
/// Enio foi *"equivalente ao da godot"*, e a casa do gesto faz parte da equivalência.
///
/// ⛔ Ao contrário das outras entradas deste menu, esta **não abre um submenu**: ela abre uma
/// janela. Ver a nota em `menu_rows.rs`.
pub const CTX_MENU_SETTINGS_INPUT_MAP: NodeId = hash_node_id("ctx_menu_settings_input_map");

/// Top-level Settings entry that opens the Display-unit submenu
/// (Meters / Pixels). Companion of `CTX_MENU_SETTINGS_PPM`.
pub const CTX_MENU_SETTINGS_UNIT: NodeId = hash_node_id("ctx_menu_settings_unit");
pub const CTX_MENU_UNIT_METERS: NodeId = hash_node_id("ctx_menu_unit_meters");
pub const CTX_MENU_UNIT_PIXELS: NodeId = hash_node_id("ctx_menu_unit_pixels");

/// Top-level Settings entry that opens the Angle-unit submenu
/// (Degrees / Radians) — **o irmão de `CTX_MENU_SETTINGS_UNIT` para o ÂNGULO**
/// (Enio, 2026-08-30: *"devemos ter ambas as opções no app"*).
///
/// ⚠️ Duas entradas separadas, e não uma «Units» que abrisse as duas: comprimento
/// e ângulo trocam-se por razões diferentes (a escala do projecto contra o hábito
/// do artista), e juntá-las obrigaria a um submenu de dois níveis para uma escolha
/// de dois estados.
pub const CTX_MENU_SETTINGS_ANGLE: NodeId = hash_node_id("ctx_menu_settings_angle");
pub const CTX_MENU_ANGLE_DEGREES: NodeId = hash_node_id("ctx_menu_angle_degrees");
pub const CTX_MENU_ANGLE_RADIANS: NodeId = hash_node_id("ctx_menu_angle_radians");

/// Top-level Settings entry that opens the Image-filter submenu
/// (Pixel Art / Smooth). Companion of `CTX_MENU_SETTINGS_UNIT`.
/// Selecting a mode flips the app-wide `ImageFilterMode` — the single
/// sampler/quality applied to every sprite + the Vello preview.
pub const CTX_MENU_SETTINGS_FILTER: NodeId = hash_node_id("ctx_menu_settings_filter");
pub const CTX_MENU_FILTER_PIXELART: NodeId = hash_node_id("ctx_menu_filter_pixelart");
pub const CTX_MENU_FILTER_SMOOTH: NodeId = hash_node_id("ctx_menu_filter_smooth");
/// Top-level Settings entry that opens the Display submenu (present
/// mode). Selecting a mode switches the swap-chain present mode at
/// runtime: VSync (`Fifo`, smooth motion) vs Immediate (non-blocking,
/// no mouse-stutter — the M5-demo-continuous-render tradeoff).
pub const CTX_MENU_SETTINGS_DISPLAY: NodeId = hash_node_id("ctx_menu_settings_display");
pub const CTX_MENU_DISPLAY_VSYNC: NodeId = hash_node_id("ctx_menu_display_vsync");
pub const CTX_MENU_DISPLAY_IMMEDIATE: NodeId = hash_node_id("ctx_menu_display_immediate");
/// Top-level Settings entry that opens the Text rendering submenu —
/// toggle entre `Default` (histórico) e `Crisp Heavy` (ExtraBold +
/// snap-X + hint=false). Os presets intermediários (Crisp Light, Crisp)
/// foram removidos em 2026-05-25 por serem visualmente equivalentes
/// — vide `docs/UI_Fonts/2026-05-25-crisp-heavy-implementation.md`.
pub const CTX_MENU_SETTINGS_TEXT: NodeId = hash_node_id("ctx_menu_settings_text");
pub const CTX_MENU_TEXT_DEFAULT: NodeId = hash_node_id("ctx_menu_text_default");
pub const CTX_MENU_TEXT_CRISP_HEAVY: NodeId = hash_node_id("ctx_menu_text_crisp_heavy");
pub const CTX_MENU_TEXT_CRISP_HEAVY_PLUS: NodeId = hash_node_id("ctx_menu_text_crisp_heavy_plus");

/// Top-level Settings entry that opens the **Motion** submenu — o carácter da UI viva
/// (`crate::motion::UiCharacter`) e o interruptor de *reduced motion*.
///
/// ⚠️ **São DOIS eixos numa submenu só, e a distinção é o que a torna correcta:** as duas primeiras
/// linhas são um **rádio** (o GOSTO — Expressivo ou Discreto, nunca os dois), a terceira é um
/// **toggle** (a GARANTIA — reduced motion, que se sobrepõe a qualquer carácter). Colapsá-las num
/// selector de três posições entregaria uma garantia de acessibilidade disfarçada de gosto, e
/// tornaria *Expressivo + reduced* — uma combinação legítima — inexprimível.
pub const CTX_MENU_SETTINGS_MOTION: NodeId = hash_node_id("ctx_menu_settings_motion");
pub const CTX_MENU_MOTION_EXPRESSIVE: NodeId = hash_node_id("ctx_menu_motion_expressive");
pub const CTX_MENU_MOTION_DISCRETE: NodeId = hash_node_id("ctx_menu_motion_discrete");
pub const CTX_MENU_MOTION_REDUCED: NodeId = hash_node_id("ctx_menu_motion_reduced");
