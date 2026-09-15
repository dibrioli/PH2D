#![forbid(unsafe_code)]
// Tecto de LOC NUMERADO em `tests/it/file_loc_caps.rs` — crate-root module hub — the 80+ `mod` declarations are an
// append-only extension point (every drop-in line adds one, and `mod`
// declarations cannot leave the crate root) alongside the winit `App` entry
// impl + `fn main`. Grew past the HR-18 cap by cross-line `mod` accumulation
// during the 2026-07-17 multi-line integration; the growth is structural.

//! Desktop shell — winit 0.30 + wgpu + ECS + sprite render + M6+M7+M12.
//!
//! Run with: `cargo run -p ph2d-host-desktop`
//!
//! Layered subsystems (each gated to keep the demo bootable even if
//! one fails — never crash the shell over an integration-demo issue):
//! - **M5** SpriteRenderer + 1000-sprite Vogel spiral with bouncing motion
//! - **M6** AssetDb loads 16 real PNG files from `assets/sprites/` (auto-
//!   generated on first launch) and composes them into a 256×256
//!   RGBA8 atlas. Falls back to procedural dummy if anything fails.
//! - **M7** ScriptHost with placeholder Luau script; per-frame gc_step
//!   keeps the GC budget warm for future script-driven gameplay.
//! - **M12** editor data layer: `ZenMode` (Tab toggle), `ToastQueue`
//!   (T key adds info toast), theme switch (M key flips Dark↔Light),
//!   and a `FloatingPanel` (Procreate-style selection demo). Visible
//!   via window title since Vello widget paint requires sharing the
//!   wgpu Surface with the sprite pipeline (see `integration.rs`).
//!
//! M8 add-on (gamepad path):
//! - gilrs adapter (`gilrs_adapter` module) pumps gamepad events into
//!   [`ph2d_input::InputState`] each frame, BEFORE sim tick.
//! - Connection / button / axis events log to the terminal at the
//!   `[Nms]` timestamp prefix used by the rest of the shell.
//! - Axis logs filter dead-zone jitter (only `|value| > 0.25`).
//! - Pencil events are wired through the abstraction but not produced
//!   by this shell (iPad shell will, in M9+).
//!
//! Out of scope here:
//! - **M8 → ScriptHost**: `InputState` lives on `App`, but routing into
//!   `ph2d.input` Luau snapshot is a follow-up.
//! - **M11** Vello text/widget overlay — needs surface-sharing pass.

mod active_tool_mirror;
pub(crate) use ph2d_app_vec::align_live;
mod align_smoke;
mod anchor_gizmo_drag;
mod anchor_smoke;
/// **As três formas de âncora, numa sprite só** (`PH2D_SOCKET_SMOKE=1`, ADR-0072).
mod anim_smoke;
/// W2 — como esta shell responde ao `ph2d_app_host::AppHost`, o trait por onde uma família de
/// módulo fala com ela. Um ficheiro, cinco métodos, zero handles devolvidos.
mod app_host;
mod app_state;
/// **O IMPORT do `.ase`** (Enio, 2026-08-23) — o ficheiro NATIVO do Aseprite vira uma sprite com
/// grelha e a biblioteca de animações dele. Irmão do `sheet_import` (o par `.png`+`.json`).
mod ase_import;
/// A cena de smoke do import do `.ase` (`PH2D_ASE_SMOKE=1`) — ela ESCREVE o ficheiro e larga-o
/// pela porta do produto, para o smoke não precisar do Aseprite instalado.
mod ase_smoke;
// ⛔ **As LEIS dos assets mudaram-se para `ph2d-app-components`** (W2 5.ª rodada, 2026-09-13):
// o índice (`asset_index_build`), o que um cartão desenha (`asset_card_art`/`_portrait`), os verbos
// do cartão e do catálogo (`asset_card_verbs`/`asset_catalog_verbs`) e a lei da queda
// (`asset_drop`). Ficam aqui as duas PONTES — o arrasto precisa da câmara e do pick, o braço da
// queda do `SpriteRenderer` e do funil de texturas — e o smoke dirigido pelo ponteiro.
/// ⭐⭐⭐ **O arrasto da biblioteca, ligado ao ponteiro** (plano `docs/Components/07`, etapa B) —
/// a fiação `Down`/`Move`/`Up`. ⚠️ A LEI da queda vive no `ph2d_app_components::asset_drop`, e é pura.
mod asset_drag_wire;
/// ⭐⭐ **O braço da queda** — as três acções, cada uma pela porta que já existe. ⛔ Sem decisões.
mod asset_drop_apply;
/// ⭐⭐⭐ **O navegador de assets DIRIGIDO PELO PONTEIRO** (`PH2D_BUILD_SMOKE=78`) — o buraco que a
/// auditoria da etapa B nomeou: nenhum gate do painel aperta o botão de verdade.
mod asset_menu_smoke;
mod atlas_loader;
// ⛔ **O BACKEND DE ÁUDIO mudou-se para a `ph2d-audio-desktop`** (`line/shell-folhas`, 12/09 — hoje a FAMÍLIA `ph2d-app-audio`, auditoria A1):
// 8 544 linhas e o `cpal` inteiro saíram desta unidade de compilação. A shell continua a ler as
// sete `PH2D_AUDIO_*` — o ROTEADOR é composição e fica aqui; o que saiu foi o motor.
/// **O OBJETO ASSADO** (`docs/3D/02.2`, rota A) — os canais que uma malha doou a um sprite e a luz
/// que os le'. ⚠️ Deliberadamente FORA da feature `sculpt3d`: um objeto assado sobrevive ao modulo.
/// Blend Objects vivos (ADR-0128): o objeto único que interpola 2..=5 formas e as segue
/// (re-cook por frame). Espelha `connector_live`.
pub(crate) use ph2d_app_vec::blend_live;
mod blend_smoke;
/// ⭐ **O gesto do modo OSSO** (estudo 42 item 5): arrastar no vazio faz um osso, e o pai é o osso
/// seleccionado — arrasto-arrasto-arrasto é uma cadeia.
pub(crate) use ph2d_app_skeleton::bone_gesture;
/// ⭐ **O LIMITE DE UMA JUNTA** — irmão do `bone_gesture` pelo teto de LOC, cortado por assunto.
pub(crate) use ph2d_app_skeleton::bone_limit;
/// ⭐ **As três pontes `impl App` da família do esqueleto** — ver o cabeçalho do módulo.
mod skeleton_app_bridge;
// **Os dois gates do esqueleto cujo SUJEITO é a shell** (a captura do undo · o gizmo de grupo).
#[cfg(test)]
#[path = "skeleton_shell_seam_tests.rs"]
mod skeleton_shell_seam_tests;
/// ⭐ **O que a MÃO faz a um osso** — irmão do `bone_gesture`, cortado por responsabilidade.
pub(crate) use ph2d_app_skeleton::bone_pose;
/// ⭐ A sonda do OSSO INTELIGENTE (`PH2D_BONE_SMART_PROBE=1`) — irmã da de cima.
///
/// ⚠️ Ela entrou ACIMA do `bone_undo_probe` na 1.ª versão e roubou-lhe o doc: *um item novo colado
/// a um comentário fica documentado por ele* (auditoria de 2026-09-08).
mod bone_smart_probe;
/// ⭐⭐⭐ **A sonda do UNDO da âncora** (`PH2D_BONE_UNDO_PROBE=1`) — o gesto REAL sobre o *Add IK*, e
/// o que ele deixa na fila de desfazer.
mod bone_undo_probe;
/// Os GESTOS da booleana viva: armar (criar/re-mirar) e consolidar. O documento mora aqui; o
/// motor, no `bool_live`.
pub(crate) use ph2d_app_vec::bool_gesture;
/// A BOOLEANA VIVA (plano UI/UX W1): um grupo cujos filhos se combinam e continuam editáveis.
/// O 7º produtor de `LiveGeometry`, e o segundo que TRANSFORMA o mapa em vez de o estender.
pub(crate) use ph2d_app_vec::bool_live;
/// A cena de smoke da booleana viva (`PH2D_BUILD_SMOKE=48`) — irmã de `build_smoke`, teto de LOC.
mod bool_smoke;
/// ⭐ **DAR e TIRAR o traço de uma forma** (plano 34) — a porta da caixa *Stroke* do painel. Existe
/// porque o `restyle_selected_strokes` recusa quem não tem traço, e essa recusa está CERTA: ele
/// corre por quadro, e criar ali vestiria toda forma selecionada sem ninguém pedir.
mod brush_corner_smoke;
mod brush_smoke;
mod bucket_smoke;
mod buffer_smoke;
mod build_smoke;
mod build_smoke_corner_tools;
mod build_smoke_drive;
/// A cena de smoke do **Expand** (Outline Stroke + Offset Path) — `PH2D_BUILD_SMOKE=17`.
mod build_smoke_expand;
mod build_smoke_router;
// ⛔ A fachada `canvas_area` (um `pub(crate) use ph2d_app_host::canvas_area::visible`) morreu em
// 2026-09-13: o último leitor dela era o palco do prefab, que se mudou para `ph2d-app-components` e
// escreve o caminho da porta.
/// ⭐⭐ **A cor com que a camada de sprites é limpa** — o fundo que o artista vê no canvas, hoje
/// derivado da porta única em vez de escrito à mão. Ver o cabeçalho de lá.
mod canvas_clear;
/// A lei do **zoom do canvas** — a roda escreve um destino, o quadro publica o vivo.
mod canvas_zoom;
mod chrome_hit;
/// Teclado do palette de "Add Node" (busca/filtro, Enter/Backspace/Esc).
mod command_palette_input;
/// ⛔ As DUAS costuras de teste que ficaram na shell quando a família das instâncias saiu — o
/// sujeito delas é meio chrome (o probe do Inspector · o re-alojamento de pixels), e uma crate
/// nunca pode chamar o `bin`. Ver o `test_support` da `ph2d_app_components`.
#[cfg(test)]
mod component_attach_seam_tests;
/// As sementes REAIS de anexar (a tabela da física entregue à porta dos componentes) — mudaram-se
/// da `ph2d-app-components` na auditoria de arquitectura A1 (2026-09-12).
#[cfg(test)]
mod component_seed_seam_tests;
/// O gesto que cria um conector (Down numa forma, Up noutra).
/// ⭐ O PRÓLOGO das cenas da família das instâncias — o invólucro que traduz `&mut App` para a
/// assinatura da [`ph2d_app_components`]. *O que sai são os corpos; o que decide a ordem do quadro
/// fica.*
mod components_scenes;
mod connector_gesture;
/// Conectores vivos: a linha que gruda em duas formas e as segue (re-cook por frame).
#[cfg(test)]
mod instance_paint_seam_tests;
pub(crate) use ph2d_app_vec::connector_live;
pub(crate) use ph2d_app_vec::contour_live;
/// A cena de smoke do **Contour** (`PH2D_BUILD_SMOKE=25`) — irmã de `build_smoke`, teto de LOC.
mod contour_smoke;
mod corner_handles;
mod cursor_pos;
/// **O Width Tool** — as alças de largura na curva (plano 25 W2, ADR-0148).
mod cut_smoke;
/// `PH2D_DITHER_SMOKE` — as faixas e a cura, lado a lado (plano `docs/Sprite_projeto/18` W6.1).
mod dither_smoke;
mod dock_resize;
/// O canal da **DOAÇÃO de forma** para a tinta do Painter — plano de normais + o tamanho do canvas.
/// Sem `cfg`, de propósito: o que atravessa é `Vec<f32>`, nunca um tipo do módulo 3D.
/// ⭐⭐⭐ **As FAIXAS de desenho** (ADR-0154 Fase 2) — a lei que põe vetor e sprite na MESMA ordem
/// total, e parte essa ordem nas passagens que o presente desenha.
mod draw_bands;
/// `PH2D_EMISSIVE_SMOKE` — a sprite como fonte de luz (plano `docs/Sprite_projeto/18` W8).
mod emissive_smoke;
mod envelope_gesture;
mod envelope_live;
/// As cenas de smoke do Envelope (ADR-0129) — irmão de `build_smoke`, teto de LOC.
mod envelope_smoke;
mod expr_blend_smoke;
mod extrap_smoke;
mod falloff_smoke;
// ⭐ **Estes dois VOLTARAM da família, e a razão é o SUJEITO de cada um.**
//
// - `field3d_undo_probe` conduz a `App` REAL pelo ponteiro real (`self.smoke_pointer_move`,
//   `self.probe_grab_widget`) — o arnês é da shell, e uma crate de família não lhe chega.
// - `field3d_snapshot_tests` captura um `ProjectState`, que é a máquina de undo da shell; o doc
//   dele já se chamava *«a metade de SHELL da ponte ECS»*.
//
// ⚠️ **O censo da linha contou `1 impl App` nesta família e eram DOIS** — ele grepou `^impl App`
// e o segundo está escrito `impl crate::App`. *Um censo por forma textual conta a forma, não a
// coisa.*
#[cfg(test)]
#[path = "field3d_snapshot_tests.rs"]
mod field3d_snapshot_tests;
mod field3d_undo_probe;
/// ADR-0161 W109 — o cabeçalho CLICÁVEL de cada vista: o menu que troca a câmera daquele quadrante.
/// Motion Nodes: o gizmo de canvas de um field espacial (`field.box`, …). Espelho do
/// `flip_selection_gizmo` — `GizmoTarget::MotionField`, apply nos params do NÓ.
mod flip;
mod forwarding;
/// A cena de smoke da MOLDURA (`PH2D_BUILD_SMOKE=49`) — irmã de `build_smoke`, teto de LOC.
mod frame_smoke;
mod fx_adjust_smoke;
mod fx_blend_smoke;
pub(crate) use ph2d_app_vec::fx_bridge;
pub(crate) use ph2d_app_vec::fx_bridge_dispatch;
/// **FX raster VIVO** — o cozimento do `ph2d_ecs::VecFilter` (Blur/Glow/Drop Shadow, plano 24):
/// isola a forma, borra/tinge, e injeta a imagem no z dela via `ph2d_vec_render::FxImages`.
mod fx_duotone_smoke;
mod fx_gradient_map_smoke;
pub(crate) use ph2d_app_vec::fx_live;
mod fx_morphology_smoke;
mod fx_raster_smoke;
pub(crate) use ph2d_app_vec::fx_silhouette;
mod fx_smoke;
mod fx_turbulence_smoke;
mod fx_undo_smoke;
mod gizmo_anchor_smoke;
mod global_palette_input;
/// **A cena da SUJIDADE NA LENTE** (`PH2D_GLOW_DIRT_SMOKE=1`, doc 89 folha 11) — a máscara
/// precisa de uma IMAGEM a sério, que é o que os demos de grafo não têm.
mod grid_smoke;
mod group_gizmo_view;
mod guide_gesture;
mod guide_smoke;
/// A cena de smoke das Color Harmonies (abre o picker com Triad) — `PH2D_HARMONY_SMOKE=1`.
mod harmony_smoke;
mod hero_bridge;
mod hero_intents;
/// ⭐⭐⭐ **AGRUPAR / DESAGRUPAR pela Hierarquia** — o alcance de um verbo que já existia em `Ctrl+G`
/// e que nenhum menu, botão ou rótulo do app nomeava.
mod hier_group;
mod hover_highlight;
/// `Export Image…` — a porta dos 16 exportadores (plano `docs/Sprite_projeto/18` W9).
mod image_export;
mod image_import;
/// **UMA lei sobre o que este app importa** — o filtro do diálogo e o roteamento do drop leem
/// daqui (Enio, 2026-08-23: *«.ase não aparece no dialog de import»*).
mod import_router;
mod init;
mod input_dispatch;
mod input_drop;
mod input_handlers;
mod input_log;
mod input_map_drag;
/// ⭐ **A lei da F3 num sítio só** — o Inspector mostra o que o objeto TEM (ADR-0166).
#[cfg(test)]
#[path = "inspector_presence_tests.rs"]
mod inspector_presence_tests;
mod integration;
mod keymap;
/// A cena de smoke do Knot (o entrelace celta over/under) — irmão de `build_smoke`.
mod knot_smoke;
mod ktx2_smoke;
mod label_live;
mod lasso_smoke;
mod layout_live;
mod layout_persist;
mod layout_reorder;
/// `layout_scroll_gesture`: a roda que rola uma moldura (o motor e' o `layout_live::scroll`).
mod layout_scroll_gesture;
mod layout_smoke;
mod legacy_chrome;
/// SONDA (`--ignored`): quanto custa MOVER uma forma que tem geometria viva. A §11 do plano 25
/// afirma que todo memo de geometria e' chaveado no MUNDO — esta sonda pergunta ao produto.
#[cfg(test)]
#[path = "live_memo_probe.rs"]
mod live_memo_probe;
/// `Merge to Layers` — instala no Painter o documento que a fusão produziu (plano Sprite 18 W10).
mod merge_layers;
// ⛔ **O alias `modal` FOI APAGADO** (line/shell-folhas): a lei já vivia na
// [`ph2d_app_host::modal`] desde a W2 e o alias existia só porque cinco linhas estavam a mover
// ficheiros naquele dia. Elas integraram — o prazo que o próprio doc dele escrevia chegou —, e os
// quatro chamadores passaram a escrever o endereço a sério.
#[cfg(test)]
#[path = "modal_tests.rs"]
mod modal_tests;
mod morph_fade_smoke;
pub(crate) use ph2d_app_vec::morph_live;
pub(crate) use ph2d_app_vec::morph_machine_drive;
// A LEI mudou-se para a folha `ph2d-vec-entities`; a CADEIA de gates dela fica, porque
// atravessa o `morph_live`, o `vec_convert`, o `vec_ui_state_edit` e o `render_loop`.
mod connector_handles;
/// A metade do gizmo de campo que fala com a `App` — o prólogo que FICA (W2 Fase C).
mod field_gizmo_host;
#[cfg(test)]
#[path = "morph_set_tests.rs"]
mod morph_set_tests;
mod morph_states_smoke;
/// ⭐ **A familia MOTION** — 269 ficheiros que viviam soltos em `src/`, agrupados em
/// `src/motion/` pela W2/L1 (2026-09-11). As 27 raizes (e a `#[cfg(test)]` de oito
/// delas) mudaram-se para [`crate::motion`], que e' o unico `mod` que fica aqui.
/// A shell preenche o contexto das cenas de Motion — a metade que FICA (W2 Fase C).
mod motion_host;
mod mount_smoke;
mod multi_node_smoke;
mod nest_smoke;
mod node_reach_smoke;
mod node_xy_smoke;
/// **Expand** — os cliques de Offset Path / Outline Stroke (o motor é
/// `ph2d_vec_boolean::expand`; aqui mora o que é de documento: z, pose e undo).
pub(crate) use ph2d_app_vec::offset_live;
/// Onion settings modal — the shell half (ADR-0142 W3b): store→onion read-back + the title-band drag.
mod onion_modal;
/// ⭐⭐ **A cena da OPACIDADE das duas tintas** (`PH2D_BUILD_SMOKE=79`, plano 36 W6) — a estampa e o
/// pincel obedecem à barra *Opacity*, cada um na casa dele.
mod paint_opacity_smoke;
mod palette_persist;
/// A SONDA do drift de pan (`PH2D_PAN_DIAG=1`) — report do Enio de 2026-08-25.
/// **Pattern Along Path** — o cozimento vivo do `VecPatternPath` (plano 23), irmão do `offset_live`.
pub(crate) use ph2d_app_vec::pattern_live;
mod pattern_path_smoke;
/// ⛔⛔ **A MEDIÇÃO da costura do ladrilho** (plano 33, W10) — o amostrador do Vello grampeia os
/// taps na fronteira do ladrilho em vez de dar a volta, e o `High` do vello 0.9+ triplicou a banda.
mod pencil_smoke;
/// A família `physics` (W2/L2): o que dela PRECISA da shell.
/// O resto vive em `crates/ph2d-app-physics`.
mod physics;
/// **O import de uma folha hand-packed** (`folha.png` + `folha.json`) — irmão do
/// `image_import`, e o primeiro consumidor que o `parse_atlas_meta` tem desde 2026-05-12.
/// **A folha como OBJETO** (plano `docs/Sprite_projeto/17` §7) — criar uma a partir da seleção e
/// arranjar as peças dentro dela. Quase nada aqui é código novo: a folha é um retângulo vivo que
/// ganhou um componente, como a moldura.
/// **Um contêiner não rouba o clique dos próprios filhos** — a lei que faz um filho de moldura
/// (ou de folha) ser agarrável no canvas. Pura, e por isso testada.
mod pick_order;
/// As fronteiras da folha — confinar uma peça, e contar o que está mal.
/// **A conversão de precisão dos pixels de uma sprite** — o que os botões `RGBA8 / RGBA16` do
/// Inspector fazem (plano `docs/Sprite_projeto/18` W5).
mod precision_convert;
/// **As ferramentas que só movem pixels preservam a precisão** — veja os docs do módulo.
mod precision_geometry;
/// ⭐⭐⭐ **As duas SAÍDAS do palco do prefab** (`Done`/`Cancel`) — a ponte que fica porque o `Cancel`
/// repõe o `ProjectState`. ⚠️ A LEI do palco vive em `ph2d_app_components::prefab_stage` (2026-09-13).
mod prefab_exit;
mod prefs;
/// ⚠️ NÃO é do Motion apesar do nome: ele toca `self.ui_motion_smoke_done` e abre o painel
/// de física. O `CLAUDE.md` lista-o sob o **Vector** (W2 Fase C).
mod ui_motion_smoke;
// **Estado de PRÉ-VISUALIZAÇÃO contra estado de DOCUMENTO** — a lei mudou-se para a folha
// `ph2d-preview-drive`; os GATES dela ficaram aqui, porque medem a captura desta shell.
#[cfg(test)]
#[path = "preview_drive_tests.rs"]
mod preview_drive_tests;
// **As setas do Morph** — a LEI saiu para `ph2d_app_vec::morph_edit` (W2 Fase C) e o gate da
// COSTURA ficou: ele lê `render_loop/mod.rs` e os dois do teclado, que não se mudaram.
#[cfg(test)]
#[path = "morph_arrow_seam_tests.rs"]
mod morph_arrow_seam_tests;
/// **A sonda da §4.3 do plano da UI viva** — o cursor pode ser PRESO nesta máquina? Só de teste:
/// ela abre janela e precisa de uma mão mexendo o rato, então nunca entra num build de produto.
#[cfg(test)]
mod probe_cursor_grab;
/// **A largura VIVA** — o cozimento do `VecStrokeProfile` (ADR-0148), irmão do `offset_live`.
pub(crate) use ph2d_app_vec::profile_live;
/// A cena de smoke da **largura viva** (`PH2D_BUILD_SMOKE=41`) — irmã de `build_smoke`, teto de LOC.
mod profile_smoke;
mod project;
/// **Os canais assados dentro do arquivo** (ADR-0150 W8.7) — gemeo do `project_painter`.
mod project_baked_form;
/// **A arte dos padrões dentro do ficheiro de projecto** (plano 33, W4).
/// ⭐⭐ **A TAXONOMIA da biblioteca dentro do ficheiro** (plano 07, A3) — blob auto-versionado.
mod project_catalogs;
/// **O ficheiro do projeto tem NOME** — `Save`, `Save As…` e `Open Project…` com diálogo
/// (Enio, 2026-08-23). Até aqui o `Ctrl+S` escrevia sempre no mesmo caminho.
mod project_io;
mod project_library;
mod project_migrate;
/// A migração v97 → v98 (o corte da `Sprite`) — irmã por assunto, não por cap: ela é uma
/// travessia do snapshot, e a v95 é um espelho do ficheiro.
mod project_migrate_sprite;
mod project_painter;
mod project_schema;
/// A metade ARQUIVADA da escada do `PROJECT_SCHEMA` (v2..v79) — irmã por LOC (HR-18).
mod project_schema_history;
/// A escada arquivada de `v83` a `v98` — o corte por idade, a 2.ª vez (ver o cabeçalho dela).
mod project_schema_history_v83;
mod project_schema_history_v99;
/// **As settings do PROJETO viajam no arquivo** (doc 88, D3) — a escala do mundo e a
/// unidade que o artista lê; irmão de `project` pelo teto de LOC.
mod project_settings;
/// **Os pixels próprios de um sprite dentro do arquivo** (plano `docs/Sprite_projeto/17` §3) —
/// irmão do `project_painter`, e o chão que faltava debaixo dele: cobre o funil que TODAS as
/// ferramentas de imagem atravessam, não um produtor só.
mod project_sprite_pixels;
/// ⭐⭐ **A ÁRVORE DE TAGS a chegar à sessão** (TOP-20 #9) — a porta que o load e o undo partilham.
mod project_tags;
mod project_texture_pattern;
/// **A tabela de COR autorada viaja no arquivo** (plano UI/UX W6) — irmão de `project`
/// pelo teto de LOC, cortado por assunto.
mod project_tokens;
mod radial_input;
mod render_loop;
mod scroll_smoke;
/// ⭐ **O gémeo NEUTRO do acima** — as três respostas que o resto do app espera quando a família
/// não foi compilada. Ver o cabeçalho dele: gatear os chamadores era a cura errada.
#[cfg(not(feature = "sculpt3d"))]
mod sculpt3d_absent;
/// A família 3D inteira — **uma pasta, um `mod`** (W2/L3, 2026-09-11). Os 111 ficheiros
/// `sculpt3d_*.rs` que viviam soltos aqui no `src/` passaram a `src/sculpt3d/`, e o
/// `sculpt3d_keys_view` — que era o único irmão declarado à parte — é hoje
/// [`sculpt3d::keys_view`]. ⇒ o corte da Fase B é mover UMA pasta.
/// ⭐ **Onde a shell ATENDE a família da escultura** — os quinze invólucros que desmontam o
/// `AppGfx`. A família mora em [`ph2d_app_sculpt3d`] desde a W2/L3.
/// **O GESTO INTEIRO do bake, num device de verdade** — `#[ignore]`, precisa de adapter.
#[cfg(all(test, feature = "sculpt3d"))]
#[path = "sculpt3d_bake_gesture_tests.rs"]
mod sculpt3d_bake_gesture_tests;
#[cfg(feature = "sculpt3d")]
mod sculpt3d_host;
mod shape_build_gesture;
/// O BAKE da folha — as peças passam a ser N janelas para UMA textura (plano §7.3, W5.2).
mod sheet_bake;
mod sheet_bounds;
/// A EXPORTAÇÃO da folha — `.png` + `.json`, o formato do Aseprite (plano §7.3, W5.2).
mod sheet_export;
mod sheet_frame;
mod sheet_import;
/// `PH2D_SHEET_SMOKE` — a cena que exerce a folha como OBJETO (plano `docs/Sprite_projeto/17` §7).
mod sheet_smoke;
mod signal_smoke;
/// ⭐ A cena da TABELA SINAL → PAPEL (`PH2D_BUILD_SMOKE=68`) — ⚠️ NÃO é o `signal_smoke`, que é
/// a cena do R0 (`PH2D_SIGNAL_SMOKE`): ali o assunto é a SAÍDA, aqui é o CONSUMIDOR.
mod signal_table_smoke;
mod sim_populate;
mod sizing_smoke;
/// ⭐⭐⭐ **A ÂNCORA, viva**: a restrição de cinemática inversa que persiste, resolvida por quadro.
pub(crate) use ph2d_app_skeleton::goal as skeleton_goal;
/// ⭐⭐⭐ **O ESQUELETO, vivo** (estudo 42 item 5): a forma presa aos ossos, re-cozida por quadro.
mod skeleton_live;
/// ⭐⭐⭐ **REVELAR-AO-FOCAR**: quando um osso NOVO entra em foco, a secção Skeleton vem à vista.
pub(crate) use ph2d_app_skeleton::reveal as skeleton_reveal;
mod skeleton_skin_image;
/// ⭐ **Os OSSOS INTELIGENTES** — girar um osso percorre uma animação inteira.
pub(crate) use ph2d_app_skeleton::smart as skeleton_smart;
/// As cenas de smoke do Sketch (=31) e do Hatch (=32) — irmão de `build_smoke`, teto de LOC.
mod sketch_hatch_smoke;
/// **9-slice, lado a lado com o que ele conserta** (`PH2D_SLICE_SMOKE=1`).
mod slice_smoke;
mod smoke_script;
mod snap_label_smoke;
mod socket_smoke;
mod stack_smoke;
mod stagger_smoke;
pub(crate) use ph2d_app_vec::svg_import;
mod svg_import_smoke;
pub(crate) use ph2d_app_vec::symmetry_live;
/// A cena de smoke da SIMETRIA de desenho (`PH2D_BUILD_SMOKE=46`) — irmã de `build_smoke`.
mod symmetry_smoke;
mod text_fx_smoke;
mod text_path_gesture_smoke;
mod text_path_smoke;
/// A cena de smoke do **REFLUXO** de texto (`PH2D_BUILD_SMOKE=63`) — irmã de `build_smoke`.
mod text_wrap_smoke;
/// **Autorar a lei de um padrão de textura** (plano 33, W5) — a porta da secção Pattern.
pub(crate) use ph2d_app_vec::texture_pattern_edit;
/// **A porta que o chip *Tile* abre** (plano 33, W4) — escolher a arte de um padrão.
mod texture_pattern_pick;
/// **A cena de smoke do Texture Pattern** (`PH2D_BUILD_SMOKE=76`, plano 33).
mod texture_pattern_smoke;
mod theme;
/// ⭐ **A REDUÇÃO a uma miniatura de cartão** — uma lei, três consumidores (os dois assadores do
/// Motion e o navegador de assets). Sem vocabulário de painel, de propósito.
mod timeline_onion_smoke;
#[cfg(test)]
#[path = "timeline_orphan_tests.rs"]
mod timeline_orphan_tests;
mod timeline_persist;
/// **A timeline é o TERCEIRO membro da família pré-visualização↔documento** — enquanto o playhead
/// toca, as curvas escrevem poses que não são edições do artista (`crate::preview_drive`).
mod timeline_preview;
mod timescale_smoke;
/// **AS MOLDURAS** (plano UI/UX W0): que intervalo da pilha de z cada `VecFrame` recorta. A
/// metade que a shell possui — o renderer sabe desenhar, a shell sabe a ÁRVORE.
mod token_smoke;
/// A cena de smoke dos **TOKENS** (`PH2D_BUILD_SMOKE=59`) — o painel que re-veste o app.
mod tokens_smoke;
/// ⭐⭐⭐ **A ferramenta TRIM** (plano 38) — a costura entre o ponteiro e a lei da crate.
/// A cena de smoke do **Trim** — `PH2D_BUILD_SMOKE=80` (plano 38).
mod trim_smoke;
/// A cena de smoke do Twist (o remoinho + o Falloff a modulá-lo) — irmão de `build_smoke`.
mod twist_smoke;
/// A metade do CHROME da poeira de impacto — a lei vive na `ph2d-editor-core`.
mod ui_burst_paint;
/// ⭐ O smoke da UI VIVA (`PH2D_UI_MOTION_SMOKE`) — o carácter e a corda.
/// A cena de smoke da **HIERARQUIA** de estados (`PH2D_BUILD_SMOKE=64`) — irmã de `build_smoke`.
mod ui_nested_smoke;
/// A cena de smoke do **PAINEL GERADO** (`PH2D_BUILD_SMOKE=62`) — irmã de `build_smoke`.
mod ui_panel_smoke;
/// **Que painel esta árvore descreve** (plano UI/UX W8b) — a porta única que lê a moldura
/// autorada e devolve o `PanelSpec` que o gerador escreve.
pub(crate) use ph2d_app_vec::ui_panel_spec;
/// A cena de smoke dos **ESTADOS DE UI** (`PH2D_BUILD_SMOKE=61`) — irmã de `build_smoke`.
/// ⭐ A cena da BOOLEANA VIVA dentro de um ESTADO de UI (`PH2D_BUILD_SMOKE=74`) — a troca de
/// operação que MORFA em vez de saltar, com os operandos a mover-se ao mesmo tempo.
/// A BOOLEANA dentro do modo de PREVIEW — irmão dos gates do preview por LOC (HR-18).
#[cfg(test)]
#[path = "ui_preview_bool_tests.rs"]
mod ui_preview_bool_tests;
/// **O RATO dentro do modo de preview** (plano UI/UX W7r) — a metade da shell do
/// `render_loop::ui_preview`: quem aponta, e o gesto modal que precede tudo.
mod ui_preview_gesture;
mod ui_sound;
/// A cena de smoke da **MOLA** (`PH2D_BUILD_SMOKE=65`) — irmã de `build_smoke`.
mod ui_spring_smoke;
mod ui_states_bool_smoke;
mod ui_states_smoke;
mod undo;
mod undo_route;
/// ⭐ **O fantasma na origem** (report do Enio, 2026-08-27) — o mundo e o documento têm de
/// concordar ANTES da fotografia do undo.
#[cfg(test)]
mod undo_vec_ghost_tests;
/// ⭐⭐ **A cena dos EIXOS DE PROPRIEDADE** — `PH2D_BUILD_SMOKE=79`: uma família nomeada
/// `Size=…, State=…` vira uma fileira por pergunta, e um chip muda exactamente um eixo.
///
/// ⚠️ Ela tem doc próprio porque inserir um módulo no meio de um doc-comment **muda o dono dele**
/// (auditoria de 2026-08-30): este bloco descrevia o `vec_variants`, e o módulo novo entrou por
/// baixo dele — ficando a documentar-se com a frase de outro e sem dizer que é um smoke.
mod variant_axes_smoke;
mod variant_flow_smoke;
mod vec_anchor_edit;
mod vec_app_bridge;
pub(crate) use ph2d_app_vec::appearance as vec_appearance;
mod vec_appearance_smoke;
pub(crate) use ph2d_app_vec::bindings as vec_bindings;
pub(crate) use ph2d_app_vec::blend as vec_blend;
/// ⭐⭐⭐ **O desenho ganha OSSOS** (`PH2D_VEC_BONE_SMOKE=1`, estudo 42 item 5) — irmã da cena da
/// pilha de aparência, teto de LOC.
mod vec_bone_smoke;
/// ⭐ Os gates de ROTA da booleana viva até os ESTADOS (auditoria de 2026-08-23): *com uma
/// booleana em mãos, o artista chega às poses dela?* — a pergunta que "o widget existe e o clique
/// chega ao barramento" não faz.
/// **O papel de cada forma dentro de uma booleana viva** — a porta única de *"que verbo é o
/// dela?"*, que o painel e a linha da hierarquia partilham.
pub(crate) use ph2d_app_vec::bool_shape as vec_bool_shape;
/// ⭐⭐⭐ **SOLDAR** (plano 39) — linhas cruzadas partem-se em arcos que partilham o nó.
mod vec_bucket;
// ⛔ **O `bucket_repro` NÃO é re-exportado, e a ausência é a cura** (W2/L4 Fase B): aquele módulo
// abre com `#![cfg(test)]`, e **`cfg(test)` é falso numa crate que é DEPENDÊNCIA** (HOWTO §2.5) —
// da shell ele simplesmente não existe. Ele é a sonda do report de 2026-09-02 e corre com os testes
// da própria `ph2d-app-vec`, que é onde o sujeito dele vive; um alias aqui só pedia à shell um nome
// que nenhuma build dela pode ver.
/// O chip *Clip content* — a projeção e a edição do RECORTE, que vale para qualquer forma
/// vetorial FECHADA (e não só para a moldura, desde 2026-08-21).
mod vec_clip_edit;
pub(crate) use ph2d_app_vec::component_edit as vec_component_edit;
/// ⭐⭐⭐ **A secção *Component* do painel vetorial, ligada ao mecanismo GERAL** (F4.6c) — nasce
/// DESLIGADA (`PH2D_VEC_COMPONENT_GENERAL=1` arma). Ver o cabeçalho de lá.
mod vec_component_general;
/// O painel edita o CONECTOR selecionado (Route / Jetty / Spread) — resolve o valor
/// EFETIVO que o painel exibe e aplica a edição a TODOS os conectores selecionados.
pub(crate) use ph2d_app_vec::connector_panel as vec_connector_panel;
/// Diagnóstico do overlay vetorial (`PH2D_VEC_OVERLAY_DIAG=1`) — nomeia o dono de geometria fora
/// do lugar, em vez de a adivinhar.
mod vec_convert;
/// **A LINHA DE CORTE** (plano 25 §7): o caminho que a tesoura usa como lâmina — adotado depois
/// do `sync`, desenhado pelo overlay, e nunca alvo do próprio corte. Espelha `connector_live`.
pub(crate) use ph2d_app_vec::cut_line as vec_cut_line;
/// ⭐ **A APARÊNCIA CONDUZIDA por um motor** — a metade de shell da ponte que a linha do
/// tempo abre para a opacidade de um caminho vetorial (`ph2d_ecs::VecDrivenStyle`).
/// Irmão do `vec_widget_drive`: o corte é *quem produz o número*, nunca o que se faz com ele.
pub(crate) use ph2d_app_vec::driven_style as vec_driven_style;
// A ponte documento ⇄ árvore mudou-se para a folha `ph2d-vec-entities` (a `motion` e a `flip`
// também a consomem). Ficam os gates que a medem A PARTIR DO GESTO.
#[cfg(test)]
#[path = "vec_entities_tests.rs"]
mod vec_entities_tests;
pub(crate) use ph2d_app_vec::expand as vec_expand;
/// ⭐ A cena de smoke do **fade vetorial** (`PH2D_VEC_FADE_SMOKE=1`) — a linha do tempo a
/// desvanecer um caminho, com e sem filtro raster.
mod vec_fade_smoke;
#[cfg(test)]
#[path = "vec_zorder_fixpoint_tests.rs"]
mod vec_zorder_fixpoint_tests;
#[cfg(test)]
#[path = "vec_zorder_late_writers_tests.rs"]
mod vec_zorder_late_writers_tests;
// ⭐ W2/L4: a família `vec` começou a sair para `crates/ph2d-app-vec`. A re-exportação mantém
// `crate::vec_font::…` a resolver em todo o resto da shell — mover 8 ficheiros custou ZERO
// alterações nos ~60 sítios que os chamam, e é o molde para a Fase B.
#[cfg(feature = "panel-vector")]
pub(crate) use ph2d_app_vec::font_preview as vec_font_preview;
pub(crate) use ph2d_system_fonts::library as vec_font;
/// A moldura da SELEÇÃO (plano UI/UX W0): o que o painel mostra, e o que o chip escreve.
mod vec_frame_edit;
pub(crate) use ph2d_app_vec::frame_labels as vec_frame_labels;
mod vec_frame_resize;
pub(crate) use ph2d_app_vec::frame_spans as vec_frame_spans;
mod vec_gizmo_view;
pub(crate) use ph2d_vec_text::glyph as vec_glyph;
// ⚠️ O alias `vec_glyph_build` SAIU em 2026-09-12: o `motion_text_gen` era o último
// consumidor dele na shell e mudou-se para `ph2d-app-motion`, onde escreve
// `ph2d_vec_text::glyph_build` directamente. *Um alias existe pelos chamadores; quando
// o último sai, ele é ruído — e o compilador disse-o.*
/// A porta única de "onde está o caminho-guia, e como se percorre por arco?" (texto E pattern).
pub(crate) use ph2d_app_vec::guide as vec_guide;
/// A CÓPIA segue a âncora do mestre — o corolário da âncora viva, do lado do componente.
mod vec_layout_edit;
pub(crate) use ph2d_app_vec::marquee as vec_marquee;
pub(crate) use ph2d_app_vec::morph_edit as vec_morph_edit;
pub(crate) use ph2d_app_vec::overlay as vec_overlay;
pub(crate) use ph2d_app_vec::overlay_diag as vec_overlay_diag;
/// O offset de CAD de uma camada da pilha (v22) — o memo do cozimento.
pub(crate) use ph2d_app_vec::paint_dilate as vec_paint_dilate;
pub(crate) use ph2d_app_vec::paint_stack as vec_paint_stack;
/// O **Picker de caminho-guia** — o gesto de duas mãos partilhado pelo Pattern e pelo Text on Path.
pub(crate) use ph2d_app_vec::pick as vec_pick;
pub(crate) use ph2d_app_vec::resize_box_edit as vec_resize_box_edit;
mod vec_selection;
pub(crate) use ph2d_app_vec::shape_live as vec_shape_live;
pub(crate) use ph2d_app_vec::shape_params as vec_shape_params;
mod vec_snap;
/// Os alvos de snap vindos do RASTER (irmão de `vec_snap`, teto de LOC).
mod vec_snap_sprites;
mod vec_stack_smoke;
pub(crate) use ph2d_app_vec::stroke_paint as vec_stroke_paint;
pub(crate) use ph2d_app_vec::stroke_present as vec_stroke_present;
mod vec_svg_export;
mod vec_text;
mod vec_text_object;
mod vec_text_reopen;
mod vec_text_ride;
// A pose das formas vetoriais mudou-se para a folha; fica o gate do quadro do LÁPIS, que
// atravessa o `profile_live`.
/// ⭐⭐⭐ **A RECONCILIAÇÃO ANTES DA CAPTURA** — a rede que apanha todo escritor TARDIO da árvore
/// (apagar · duplicar · *Remove from Sheet*), medida em 2026-09-08.
mod vec_tree_settle;
mod vec_trim;
/// **OS VERBOS DA PELE** (plano UI/UX W6.2) — vestir, trocar de tipo, despir.
pub(crate) use ph2d_app_vec::ui_state_edit as vec_ui_state_edit;
/// **OS VARIANTS** (plano UI/UX W5c) — que versão do componente uma instância é. Um conjunto de
/// variants é DERIVADO (os mestres irmãos), e os eixos saem dos NOMES: zero componente novo.
///
/// ⚠️ **Este doc voltou para cá** (auditoria de 2026-08-30): o módulo da cena de smoke dos eixos
/// entrou por baixo dele e **herdou-o**, deixando este ficheiro — o que a F4.6c vai apagar — sem a
/// única linha que dizia o que ele é. *Um comentário separado do seu item muda de dono.*
pub(crate) use ph2d_app_vec::weld as vec_weld;
pub(crate) use ph2d_app_vec::widget_drive as vec_widget_drive;
pub(crate) use ph2d_app_vec::widget_edit as vec_widget_edit;
pub(crate) use ph2d_app_vec::widget_value as vec_widget_value;
/// A costura do gizmo dos deformadores de quadrilátero com o ponteiro.
mod warp_gizmo_drag;
mod warp_smoke;
mod weld_smoke;
/// **O DESENHO É O GLIFO** — a porta única que normaliza a forma de um `IconButton` na caixa de
/// 24×24. Ela é UMA porque o canvas e o codegen precisam do mesmo glifo por motivos diferentes.
/// **A PELE por-widget** (plano UI/UX W6.2) — uma forma marcada é pintada pelo pintor REAL do
/// catálogo, no z dela. A ponte mora aqui porque só a shell alcança as duas metades.
pub(crate) use ph2d_app_vec::widget_live;
mod widget_skin_smoke;
mod width_handles;
/// A cena de smoke do **Width Tool** (`PH2D_BUILD_SMOKE=42`) — irmã de `build_smoke`, teto de LOC.
mod width_tool_smoke;
mod winit_host;
mod zorder_smoke;

pub(crate) use app_state::{
    App, AppGfx, HeroLive, ImageEditSnapshot, ImageEditTransaction, commit_image_edit_transaction,
    is_image_edit_tool, palette_visible_tool_indices,
};

// forwarding::* moved to input_dispatch.rs (PR 9b).
// cursor_pos::live_cursor_in_window + image_import::import_images_grid
// moved to input_handlers.rs (Wave 3.2 stage B).
// keymap::winit_to_editor_keycode moved to input_dispatch.rs (PR 9b).
// theme::parse_theme_env moved to init.rs (PR 9c).
use winit_host::LoggingHandler;

use ph2d_core::{FixedStep, Playhead, Vec2, install_panic_hook, panic};
use ph2d_ecs::scene::build_hierarchy_snapshot;
use ph2d_ecs::{Component, SimComponent, SimWorld, Transform};
use ph2d_editor_core::paint::Paint;
// NodeId surfaces in our `dragging` field; re-exported by ph2d-editor.
use ph2d_editor_core::NodeId;
use ph2d_host::{HostHandler, Lifecycle, PlatformHost};
use ph2d_input::InputState;
use ph2d_render::SpriteRenderer;
use std::time::Instant;

mod gilrs_adapter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::WindowId;

pub(crate) const SPRITE_COUNT: u32 = 1000;
/// Half-extent of the bouncing world in meters. Camera default has
/// `height_world = 10`, so [-5, 5] in Y is exactly the visible region;
/// X depends on aspect (narrower than visible at 4:3+).
pub(crate) const WORLD_HALF: f32 = 5.0;

// ADR-0025: `Position(Vec2)` removed — `Transform` from `ph2d_ecs` is
// the canonical pose component now. `Velocity` stays local to the
// demo because no other crate consumes it yet (M14.2+ may promote it).
#[derive(Component, Copy, Clone, Debug)]
pub(crate) struct Velocity(pub(crate) Vec2);
impl SimComponent for Velocity {}

impl App {
    fn new() -> Self {
        let gilrs = app_state::devices::init_gamepads();
        let audio = app_state::devices::init_audio();
        Self {
            dock_seam_drag: None,
            window: None,
            host: None,
            gfx: None,
            exiting: false,
            handler: LoggingHandler::new(),
            fixed_step: FixedStep::default(),
            // Start paused: the transport should not run the moment the window
            // opens (the playhead default is "playing" — foundational, motion
            // relies on it — so the app pauses its own playhead explicitly).
            playhead: {
                let mut ph = Playhead::default();
                ph.pause();
                ph
            },
            // The Keys view's clip-time clock — paused for the same reason, and its
            // own so scrubbing/playing a clip's keys never disturbs the timeline.
            clip_playhead: {
                let mut ph = Playhead::default();
                ph.pause();
                ph
            },
            // The Containers view's interior-time clock — paused, its own, for the
            // same reason: playing a container's lanes never disturbs the scene.
            container_playhead: {
                let mut ph = Playhead::default();
                ph.pause();
                ph
            },
            last_timeline_keys_mode: false,
            last_timeline_container: None,
            timeline_last_selected: None,
            // A 4 s composition on open, not an open-ended one pinned at t = 0 (Enio,
            // 2026-07-23): an AUTHORED default duration so the comp end — and the veil
            // past it — is there from the first frame. A loaded project keeps its own
            // saved duration (`apply_project` replaces this whole `TimelineState`).
            timeline: ph2d_timeline::TimelineState::with_default_duration(),
            timeline_intents: Vec::new(),
            skeleton: Default::default(),
            timeline_reveal_after_apply: false,
            timeline_view: ph2d_timeline::TimelineViewSnapshot::default(),
            timeline_signals: Default::default(),
            signals: ph2d_runtime::SignalOutbox::new(),
            signal_readers: crate::app_state::app_state_signal_readers::SignalReaders::new(),
            last_audio_report: Default::default(),
            last_camera_report: Default::default(),
            timeline_insert_key: false,
            autokey: Default::default(),
            last_frame: Instant::now(),
            pending_group_collapse: None,
            pending_resize: None,
            resize_saved_present_mode: None,
            resize_settle_frames: 0,
            modifiers: ModifiersState::default(),
            last_pointer: (0.0, 0.0),
            hovered_object: None,
            prefab_stage_pending: None,
            prefab_stage: None,
            prefab_editing: None,
            prefab_cancel: None,
            prefab_cancel_pending: false,
            pending_ui_sound: None,
            ui_burst: ph2d_editor_core::motion_burst::BurstField::default(),
            hover_outline: Vec::new(),
            dragging: None,
            title_dirty: true,
            impasto_smoke_done: false,
            substrate_smoke_done: false,
            line_smoke_done: false,
            // ⭐ Os cinco pedidos da escultura num sítio só (W2/L3-A2) — e o `Default` é o
            // estado INERTE: sem ninguém pedir nada, o laço do quadro não tem trabalho de
            // escultura para fazer (gate na crate).
            sculpt3d_req: Default::default(),
            sculpt_doc: Vec::new(),
            mask_smoke_done: false,
            sheet_smoke_done: false,
            demo_tool_forced: false,
            glow_dirt_smoke_done: false,
            slice_smoke_done: false,
            socket_smoke_done: false,
            mount_smoke_done: false,
            anim_smoke_done: false,
            ase_smoke_done: false,
            dither_smoke_done: false,
            emissive_smoke_done: false,
            taper_smoke_done: false,
            wetpaint_smoke_done: false,
            stack_smoke_done: false,
            timeline_onion_smoke_done: false,
            harmony_smoke_done: false,
            signal_smoke_done: false,
            components_smokes: crate::app_state::ComponentsSmokeLatches::default(),
            game_camera_preview: false,
            ui_motion_smoke_done: false,
            timescale_smoke_done: false,
            stagger_smoke_done: false,
            buffer_smoke_done: false,
            extrap_smoke_done: false,
            expr_blend_smoke_done: false,
            morph_fade_smoke_done: false,
            vec: ph2d_app_vec::state::VecState::default(),
            svg_import_smoke_done: false,
            nest_smoke_done: false,
            instance_echo: Default::default(),
            show_colliders: true,
            onion_ghosts: ph2d_render::LiftedInstances::default(),
            emissive_instances: Default::default(),
            frost_instances: Default::default(),
            blast_flash: None,
            bake_channels: ph2d_app_physics::bake::BakeChannels::default(),
            gilrs,
            audio,
            #[cfg(feature = "panel-audio-editor")]
            audio_sel_drag: None,
            #[cfg(feature = "panel-audio-editor")]
            audio_scrub_drag: false,
            input: InputState::new(),
            input_actions: ph2d_input::ActionState::new(),
            pan_anchor: None,
            held_button: None,
            eyedropper_dragging: false,
            last_cursor: (0.0, 0.0),
            hovered_files: Vec::new(),
            pending_drops: Vec::new(),
            frame_cpu_ms_ewma: 1.0, // optimistic baseline; reseeds on
            // the first frame's measurement
            pivot_content_center: None,
            frame_resize_start: None,
            rubber_band: None,
            pending_single_replace: None,
            group_drag_starts: Vec::new(),
            cycle_pick_world: None,
            cycle_pick_hits: Vec::new(),
            cycle_pick_idx: 0,
            cycle_pick_count: 0,
            cycle_pick_selection: None,
            last_bgremoval_pushed_entity: None,
            last_color_equalization_pushed_entity: None,
            color_equalization_previews: std::collections::BTreeMap::new(),
            last_upscale_pushed_entity: None,
            upscale_preview: None,
            bgremoval_preview: None,
            bgremoval_preview_gpu: None,
            bgremoval_tint_gpu: None,
            bgremoval_tint_extra: Default::default(),
            last_painter_pushed_entity: None,
            pending_painter_move: None,
            input_events_this_frame: 0,
            paint_stamps_this_frame: 0,
            last_dispatch_us: 0,
            paint_stamp_us_this_frame: 0,
            last_paint_stamp_us: 0,
            last_paint_stamps: 0,
            paint_ms_ewma: 0.0,
            painter_preview: None,
            painter_preview_gpu: None,
            painter_shape_source_preview_gpu: None,
            painter_gpu_preview: None,
            painter_commit_requested: false,
            donated_form: ph2d_form_donation::donated_form::DonatedForm::default(),
            painter_undo_requested: false,
            painter_redo_requested: false,
            // ADR-0114 W2: estado de desenho do Flip (publicado pelo flip_bridge).
            flip_state: Default::default(),
            ui_state_live: false,
            ui_cooked: crate::render_loop::ui_state_bridge::Cooked::default(),
            ui_preview: crate::render_loop::ui_preview::UiPreview::default(),
            ui_states_move_all: false,
            ui_states_anchor: None,
            ui_preview_leave: false,
            player_tape: ph2d_physics_ecs::InputTape::new(),
            discarded_run: ph2d_physics_ecs::InputTape::new(),
            field_gizmo_drag: None,
            warp_drag: None,
            pending_sheet_targets: Vec::new(),
            morph_preview: false,
            morph_preview_leave: false,
            joint_body_pick: None,
            joint_clipboard: None,
            wheel_body_pick: None,
            wheel_rope_pick: None,
            anchor_gizmo_drag: None,
            // O agregado da família `physics` (W2/L2). O `Default` reproduz,
            // campo a campo, os sete inicializadores que viviam soltos aqui.
            physics: ph2d_app_physics::physics_state::PhysicsState::default(),
            morph_machines: Default::default(),
            offset_live: crate::offset_live::OffsetLive::default(),
            profile_live: crate::profile_live::ProfileLive::default(),
            contour_live: crate::contour_live::ContourLive::default(),
            paint_dilate_live: crate::vec_paint_dilate::PaintDilateLive::default(),
            layout_live: crate::layout_live::LayoutLive::default(),
            align_live: crate::align_live::AlignLive::default(),
            bool_live: crate::bool_live::BoolLive::default(),
            symmetry_live: crate::symmetry_live::SymmetryLive::default(),
            pattern_live: crate::pattern_live::PatternLive::default(),
            fx_live: crate::fx_live::FxLive::default(),
            texture_pattern_live: ph2d_vec_art_live::pattern::TexturePatternLive::default(),
            brush_live: ph2d_vec_art_live::brush::BrushLive::default(),
            texture_pattern_scratch: None,
            fx_silhouette: crate::fx_silhouette::FxSilhouette::default(),
            undo: crate::undo::ProjectUndo::default(),
            undo_baseline: None,
            undo_baseline_selection: crate::undo::SelectionMark::default(),
            undo_request: None,
            undo_button: None,
            any_input_this_frame: false,
            preview_drive: ph2d_preview_drive::PreviewDrive::default(),
            project_path: crate::App::initial_project_path(),
            // ⭐ O cadeado do padrão nasce LIGADO — o comportamento que a secção tinha antes de os
            // dois eixos existirem (plano 33, W10).
            texpat_lock_aspect: [true, true],
            texpat_gap_link: [true, true],
            guide_drag: None,
            motion_shell: Default::default(),
            // ⭐ E os quatro que só existem com o módulo ligado (W2/L3-A2): UM `cfg` no lugar
            // de quatro, que é o ponto — a fronteira entre este e o `sculpt3d_req` acima é
            // imposta pela `cfg`, não escolhida (ver `sculpt3d/shell_state.rs`).
            #[cfg(feature = "sculpt3d")]
            sculpt3d: Default::default(),
            frame_ms_ewma: 16.7, // ~60 Hz baseline so the first
                                 // frame's status bar doesn't display
                                 // a wild value while the EWMA seeds.
        }
    }

    /// Per-frame render orchestration — body lifted to
    /// [`crate::render_loop`] (Wave 3.1 stage C). See its module docs
    /// for the rationale + the split-impl pattern.
    fn render_frame(&mut self) {
        // A janela já está fechando e a GPU já foi derrubada por `on_close_request` — winit pode
        // entregar um `RedrawRequested` atrasado na mesma iteração, e desenhá-lo seria pedir um frame
        // a um dispositivo que não existe mais.
        if self.exiting {
            return;
        }
        // §4.C — o PEDAÇO sob o cursor no modo Segment (hover). ANTES do render: o overlay
        // o lê no mesmo frame. Barato e guardado (só recomputa quando o cursor move).
        self.flip_segment_hover_refresh();
        // A escolha da paleta de comandos GLOBAL, executada ANTES do render do frame — assim o
        // comando (pegar uma ferramenta, mostrar um painel) já está reflectido no que este frame
        // desenha, em vez de aparecer um quadro depois.
        self.global_palette_drain();
        self.run_render_frame();
        // **A SELEÇÃO** (`flip_select`, W6): no modo Edit ela é o alvo dos ajustes do
        // painel. Só a MUDANÇA de estilo age.
        // ⭐ W2/L5 2.ª volta: as duas pedem o DOCUMENTO e o relógio, nunca a `App`.
        {
            let playhead = self.playhead;
            if let Some(gfx) = self.gfx.as_mut()
                && ph2d_app_flip::select::flip_edit_style_refresh(
                    &mut self.flip_state,
                    &mut gfx.flip,
                    &playhead,
                )
            {
                self.title_dirty = true;
            }
        }
        // **O DOMÍNIO da seleção** (W8): a troca Stroke↔Point converte a seleção no
        // documento (broadcast/promoção) — uma vez, quando o toggle muda.
        {
            let playhead = self.playhead;
            if let Some(gfx) = self.gfx.as_mut()
                && ph2d_app_flip::select_points::flip_edit_domain_refresh(
                    &mut self.flip_state,
                    &mut gfx.flip,
                    &playhead,
                )
            {
                self.title_dirty = true;
            }
        }
        // Depois do frame (estado já reconciliado pelo `sync`, `self` livre do borrow
        // do render loop): drena um Ctrl+Z/Y pendente e registra a ação do frame na
        // fila de undo global, por diff de estado (ver `undo::post_frame_undo`).
        // ⭐ **O SYNC das instâncias** (ADR-0164 / F4.3) — aqui, e a posição é lei: DEPOIS do
        // quadro (as edições do Inspector já chegaram ao mundo) e ANTES da captura (senão a
        // escrita do sync vira um passo de undo que ninguém deu). Ver o doc da função.
        self.sync_instances();
        // ⭐⭐⭐ **A SAÍDA da sessão de receita** (`Done`/`Enter` · `Cancel`/`Esc`) — servida aqui,
        // com o `self` livre, e ANTES do `post_frame_undo`: um cancelamento é uma mudança do
        // documento como outra qualquer, e o passo por diff regista-o (o `Ctrl+Z` traz as edições
        // de volta). Ver [`ph2d_app_components::prefab_stage`].
        self.serve_prefab_exit();
        // ⚠️ **A rede dos escritores tardios da árvore mudou-se para DENTRO do
        // [`crate::undo_app`], colada à captura que ela serve** — ela custa uma varredura O(formas)
        // e não tinha nada a proteger nos quadros em que a fotografia é suprimida. Ver
        // [`crate::vec_tree_settle`] e o comentário no sítio novo.
        self.post_frame_undo();
        // **O menu Ficheiro**, no mesmo sítio e pela mesma razão: `self` está livre do borrow do
        // render loop, e um diálogo nativo é modal — abri-lo a meio do frame prenderia o `gfx`.
        self.drain_project_io();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // PR 9c of the convention-by-discovery migration: subsystem
        // boot pipeline lives in `init::build_initial_state`. This
        // method now only wires the produced state into `self` and
        // fires lifecycle hooks. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        let (window, host, gfx) = init::build_initial_state(&self.handler, event_loop);
        let size = gfx.surface.size();
        let scale = host.scale_factor();
        self.window = Some(window);
        self.host = Some(host);
        self.gfx = Some(gfx);
        self.handler.on_lifecycle(Lifecycle::Foreground);
        self.handler.on_resize(size, scale);
        self.title_dirty = true;
        if let Some(host_ref) = self.host.as_ref() {
            host_ref.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // PR 9b of the convention-by-discovery migration: per-arm
        // handlers live in `input_dispatch::App::on_<arm>`. This
        // method is a pure dispatch table — adding a new arm is one
        // line here + one method there. See
        // `docs/Migracao/2026-05-convention-by-discovery.md`.
        match event {
            WindowEvent::CloseRequested => self.on_close_request(event_loop),
            WindowEvent::Resized(size) => self.on_resized(size),
            WindowEvent::HoveredFile(path) => self.on_hovered_file(path),
            WindowEvent::HoveredFileCancelled => self.on_hovered_file_cancelled(),
            WindowEvent::DroppedFile(path) => self.on_dropped_file(path),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.on_scale_factor_changed(scale_factor)
            }
            // Perder o foco SOLTA as teclas de caminhada: o `Up` de uma tecla
            // presa nunca chega quando a janela vai embora, e sem isto o player
            // anda sozinho até alguém tocá-la de novo (W3).
            WindowEvent::Focused(false) => {
                // ⚠️ **UMA memória da mão, e ela larga tudo.** O `PlayerKeys` que vivia ao lado
                // foi removido na W5 do plano 30: duas memórias divergiriam no primeiro `Up` que
                // uma recebesse e a outra não.
                self.input.apply_event(ph2d_input::Event::FocusLost);
            }
            WindowEvent::ModifiersChanged(mods) => self.on_modifiers_changed(mods),
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => self.on_ime_commit(text),
            WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved(position),
            WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel(delta),
            WindowEvent::MouseInput { state, button, .. } => self.on_mouse_input(state, button),
            WindowEvent::KeyboardInput { event, .. } => self.on_keyboard_input(event),
            WindowEvent::RedrawRequested => {
                self.render_frame();
                self.exit_after_frames_tick(event_loop);
            }
            _ => {}
        }
    }
}

// ⭐ Os dois pisos da matemática de importação mudaram-se para a folha
// [`ph2d_image_import`] com a lei que os lê (W2/L4 Fase B, 2.ª volta). A re-exportação mantém os
// 21 ficheiros que escrevem `crate::EPS_PIXELS_PER_METER` byte a byte iguais — ⛔ declará-los aqui
// OUTRA vez seriam duas respostas à mesma pergunta, e a que envelhece é a que o artista vê.
pub(crate) use ph2d_image_import::EPS_PIXELS_PER_METER;

pub(crate) use ph2d_image_import::MIN_SPRITE_SIZE;

/// Query the live cursor position relative to `window` in physical
/// pixels (top-left origin). Returns `None` if the platform path
/// fails; callers fall back to a cached value.
///
/// Existence rationale: winit 0.30 on macOS does not emit
/// `CursorMoved` during external file drag operations, so by the
/// time `DroppedFile` fires the cached cursor is stale. We bypass
/// the event stream by asking CoreGraphics for the live cursor
/// directly. Other platforms reach here only as a no-op stub.
/// Resolve the editor theme from a name (typically read from the
/// `PH2D_THEME` env var), falling back to [`Theme::Forge`] for
/// missing/invalid values. Recognised names match `Theme::id()`
/// (`forge`, `workshop`, `sunstone`, `blueprint`).
fn main() {
    install_panic_hook();
    // **A math dos tokens de design** (plano UI/UX W4c.3) — uma linha, e é a única.
    //
    // ⚠️ Ela vive AQUI e não dentro da `ph2d-tokens` por uma aresta MEDIDA: o parser arrasta
    // `ph2d-expr` → `ph2d-nodegraph`, e a `ph2d-tokens` é a folha de que 44 widgets dependem — um
    // botão de ícone passaria a compilar o motor de cozimento para saber de que cor é.
    //
    // ⚠️ E o modo de falha de a ESQUECER é o certo: o app compila, corre, e o painel simplesmente
    // não OFERECE o botão de fórmula (`math_available()` é falso) — em vez de o oferecer e não
    // fazer nada. É o padrão do `set_ml_available` do AI Denoise.
    //
    // ⚠️ Antes do `App::new()`, e na thread do laço de eventos: o host mora num `thread_local`, e
    // quem pergunta por ele é o painel a pintar.
    ph2d_token_math::install();
    let event_loop = EventLoop::new().expect("create EventLoop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    println!("PH2D desktop shell starting (close window or Cmd+Q to exit)…");
    event_loop.run_app(&mut app).expect("event loop crashed");
    println!("PH2D desktop shell exited cleanly.");
}

#[cfg(test)]
#[path = "main_theme_env_tests.rs"]
mod theme_env_tests;
