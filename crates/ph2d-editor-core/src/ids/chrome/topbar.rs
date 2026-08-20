//! TopBar cluster chrome NodeIds (TOPBAR_*).
use super::{NodeId, hash_node_id};

pub const TOPBAR_THEME: NodeId = hash_node_id("topbar_theme");
pub const TOPBAR_SAVE: NodeId = hash_node_id("topbar_save");
pub const TOPBAR_PROJECT: NodeId = hash_node_id("topbar_project");
pub const TOPBAR_PLAY_TOGGLE: NodeId = hash_node_id("topbar_play_toggle");
pub const TOPBAR_PLAY_BUTTON: NodeId = hash_node_id("topbar_play_button");
pub const TOPBAR_RIGHT_LAYERS: NodeId = hash_node_id("topbar_right_layers");
pub const TOPBAR_RIGHT_ASSETS: NodeId = hash_node_id("topbar_right_assets");
pub const TOPBAR_RIGHT_SCRIPT: NodeId = hash_node_id("topbar_right_script");
pub const TOPBAR_PAUSE: NodeId = hash_node_id("topbar_pause");
pub const TOPBAR_RESET: NodeId = hash_node_id("topbar_reset");
pub const TOPBAR_SAVE_AS: NodeId = hash_node_id("topbar_save_as");
pub const TOPBAR_OPEN: NodeId = hash_node_id("topbar_open");
/// Settings cluster (gear icon) — opens the SettingsMenu context menu
/// with project-level toggles (pixels-per-meter presets, future
/// global config). Added M14.4d retrofit.
pub const TOPBAR_SETTINGS: NodeId = hash_node_id("topbar_settings");
/// Frosted-glass agrupador backdrops behind each topbar cluster
/// group. Painted before the chips so clicks on chips win; clicks
/// on the empty backdrop space land here.
pub const TOPBAR_LEFT_BACKDROP: NodeId = hash_node_id("topbar_left_backdrop");
pub const TOPBAR_RIGHT_BACKDROP: NodeId = hash_node_id("topbar_right_backdrop");
pub const TOPBAR_IMAGE_TOOLS_BACKDROP: NodeId = hash_node_id("topbar_image_tools_backdrop");
/// Image Tools cluster — toggle entry-point for the image-editing
/// action row (Trim Transparency in V1; BG Removal / Equalize / etc.
/// to follow). Click flips the TopBar between Edit mode and
/// ImageTools mode; the state lives on
/// [`crate::screens::HeroScreen::image_tools_mode`].
pub const TOPBAR_IMAGE_TOOLS: NodeId = hash_node_id("topbar_image_tools");
/// Audio Mixer cluster — TopBar single-pill (left group, next to Image
/// Tools) that toggles the floating Audio Mixer panel (mirrors the
/// Widget Gallery / Grid Settings panel-toggle pattern). Handled by
/// `ph2d_panel_audio_mixer::AudioMixerPanel::apply_event`.
pub const TOPBAR_AUDIO_MIXER: NodeId = hash_node_id("topbar_audio_mixer");
/// Audio Editor cluster — TopBar single-pill (left group, next to Audio Mixer)
/// that toggles the docked Audio Editor panel + its floating waveform overlay
/// (mirrors the Audio Mixer panel-toggle pattern). Handled by
/// `ph2d_panel_audio_editor::AudioEditorPanel::apply_event`.
pub const TOPBAR_AUDIO_EDITOR: NodeId = hash_node_id("topbar_audio_editor");
/// Vector tool pill — TopBar single-pill that activates the Vector drawing
/// tool (ADR-0108 cutover; sole `vector_tools` member). Click pushes
/// `EditorAction::ActivateTool { tool_id: "vector" }`; the shell drain in
/// `render_loop::mod` calls `tools.set_active(&ToolId::new("vector"))`.
///
/// **Hash key = `hash_node_id("vector")`** (the manifest id, NOT
/// `"topbar_vector"`) so the Pressed-highlight reconcile loop discovers the
/// pill via `hash_node_id(manifest.id)` — same path bgremoval/image pills use.
pub const TOPBAR_VECTOR: NodeId = hash_node_id("vector");
/// Motion Nodes tool pill (Motion Nodes M0.T9). **Hash key =
/// `hash_node_id("motion")`** (the manifest id, like [`TOPBAR_VECTOR`]) so the
/// active-tool Pressed-highlight reconcile discovers the pill via
/// `hash_node_id(manifest.id)`. Click routes through `chrome::motion_toggle`.
pub const TOPBAR_MOTION: NodeId = hash_node_id("motion");
/// Flip tool pill (ADR-0114 W2). **Hash key = `hash_node_id("flip")`** (the
/// manifest id, like [`TOPBAR_VECTOR`]) so the active-tool Pressed-highlight
/// reconcile discovers the pill via `hash_node_id(manifest.id)`. Click routes
/// through `chrome::flip_toggle` → `ActivateTool { tool_id: "flip" }`.
pub const TOPBAR_FLIP: NodeId = hash_node_id("flip");
/// Widget Gallery cluster — toggles the floating reference panel
/// that showcases every canonical widget (Inputs / Slider /
/// Switches / Lists / Vector / Status / Color / Actions / Identity /
/// Card). Peripheral agents open this from the live app as the
/// single in-app source of truth for UI decoration. Visibility lives
/// in `HeroScreen::panel_visibility` (keyed `"widget_gallery"`) after
/// ADR-0029 Phase C.3; persistent rect lives on
/// `ph2d_panel_widget_gallery::WidgetGalleryState::rect`.
pub const TOPBAR_WIDGET_GALLERY: NodeId = hash_node_id("topbar_widget_gallery");
/// Grid Settings cluster — opens the floating Grid Settings panel
/// (grid-snap subsystem). Toggles
/// `HeroScreen::panel_visibility["grid_snap"]` via the typed
/// `GridSnapPanel::apply_event` after ADR-0029 Phase C.4.
pub const TOPBAR_GRID_SETTINGS: NodeId = hash_node_id("topbar_grid_settings");

/// **Physics (world)** — abre/fecha o painel de física, o mesmo que a tecla `W`.
///
/// ⚠️ Um painel de MUNDO não é tool-gated, então ele não tem chip no rail
/// esquerdo (que é de FERRAMENTAS); sem um abridor visível ele é uma feature
/// que só existe para quem sabe o atalho. O pill é esse abridor, e é a MESMA
/// visibilidade que a tecla escreve — duas portas para o mesmo bool.
pub const TOPBAR_PHYSICS: NodeId = hash_node_id("topbar_physics");

/// **O pill TOK** — o abridor visível do painel de Tokens (plano UI/UX W6).
///
/// ⚠️ Irmão exacto do `TOPBAR_PHYSICS` acima, e pela MESMA razão: um painel de MUNDO não tem chip
/// no rail (que é de FERRAMENTAS), então sem um abridor visível ele é uma feature que só existe
/// para quem já sabe o atalho — a queixa que o Enio fez sobre o painel de física em 2026-07-27, e
/// que ele fez outra vez sobre este em 2026-08-04. É a MESMA visibilidade que a tecla `T` escreve.
pub const TOPBAR_TOKENS: NodeId = hash_node_id("topbar_tokens");

/// O pill **UI** — abre o painel que o ARTISTA desenhou (plano UI/UX W8b.3).
///
/// ⚠️ Terceiro irmão do PHYS e do TOK, e pela mesma queixa: o único abridor era o chip *Show as
/// Panel*, que exige a ferramenta Vector **e** a moldura selecionada.
pub const TOPBAR_AUTHORED: NodeId = hash_node_id("topbar_authored");

/// O pill **SCULPT** — **entra e sai** do modo escultura 3D (ADR-0150).
///
/// ⚠️ **Ele NÃO é irmão do PHYS/TOK/UI, e a diferença decide o desenho:** aqueles três abrem um
/// PAINEL, e um painel aberto não toma o canvas. Este muda quem é dono do PONTEIRO — com o barro
/// na tela todo clique é da cena 3D, então sem uma saída visível o artista fica preso nela (a
/// queixa do Enio, 2026-08-09: *"não consigo configurar a textura da sprite já que não posso sair
/// do modo escultura"*). A vizinhança certa é a dos pills de MODO (VECTOR/MOTION/FLIP), que é onde
/// ele está.
///
/// ⚠️ **Ele não é uma [`crate::tool::Tool`], e o hash diz isso:** a chave é `"topbar_sculpt3d"` e
/// não um id de manifesto, porque a navegação 3D mora no SHELL de propósito (ADR-0150) — é essa
/// decisão que mantém a superfície congelada (`Tool=12`) fora do caminho, e um id de manifesto aqui
/// faria o reconcile de ferramenta ativa procurar uma tool que não existe.
pub const TOPBAR_SCULPT3D: NodeId = hash_node_id("topbar_sculpt3d");

/// O pill **MODEL** — abre o módulo de **modelagem 3D** por campo implícito (ADR-0161).
///
/// ⚠️ **Não confundir com o SCULPT ao lado.** Aquele é escultura de malha; este é modelagem
/// paramétrica, onde a booleana e o arredondamento são aritmética de campo. São dois módulos 3D e
/// duas linhas de trabalho, e a vizinhança dos dois pills é de propósito — quem procura "3D" tem de
/// encontrar os dois e ver que são coisas diferentes.
///
/// ⚠️ **Ele é irmão do PHYS/TOK, e não do SCULPT:** abre um MÓDULO com o seu painel, e não muda quem
/// é dono do ponteiro fora da área que ele desenha. Por isso não precisa da saída de emergência que
/// a nota do vizinho descreve.
pub const TOPBAR_MODEL3D: NodeId = hash_node_id("topbar_model3d");
