//! **AS TABELAS DE LINHAS DOS MENUS** — os dados que o [`super::menu_rows::menu_rows`] devolve, um
//! `const` por menu.
//!
//! ⚠️ **Cortado do `menu_rows.rs` em 2026-09-16, quando as linhas passaram a guardar CHAVES**
//! (`ids::MenuRow`, HR-15): uma chamada `const fn` num `&[…]` dentro de uma função não é promovida a
//! `'static`, e o `const { … }` por braço mais a indentação funda levaram o ficheiro de 659 a 1234
//! linhas. Aqui cada tabela é um item `const` — o mesmo desenho dos menus da timeline
//! (`ids::TIMELINE_*_MENU`) — e o `match` ficou com uma linha por menu.
//!
//! ⛔ **Uma linha nova entra AQUI, e o gate `every_menu_row_reaches_a_handler` lê este ficheiro**
//! para derivar a população (ids mencionados) — ele deixou de ler só o `menu_rows.rs`.

use crate::ids::{self, MenuRow, menu_row, menu_row_swatch};
use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;

/// As linhas de `ContextMenuKind::CreateNote { .. }`.
pub(super) const CREATE_NOTE_ROWS: &[MenuRow] = &[menu_row(
    ids::CTX_MENU_CREATE_NOTE,
    "chrome.menu.create_note",
)];

// Right-clicked on a note: the 9 background-color options + Duplicate + Delete (reuses the outline color slot ids;
// apply_event branches on `last_context_menu.kind` to set the section outline vs the note bg).
/// As linhas de `ContextMenuKind::NoteBackground { .. }`.
pub(super) const NOTE_BACKGROUND_ROWS: &[MenuRow] = &[
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_0,
        "chrome.menu.yellow",
        HIGHLIGHTER_RGBA[0],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_1,
        "chrome.menu.pink",
        HIGHLIGHTER_RGBA[1],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_2,
        "chrome.menu.green",
        HIGHLIGHTER_RGBA[2],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_3,
        "chrome.menu.blue",
        HIGHLIGHTER_RGBA[3],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_4,
        "chrome.menu.orange",
        HIGHLIGHTER_RGBA[4],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_5,
        "chrome.menu.vivid_red",
        HIGHLIGHTER_RGBA[5],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_6,
        "chrome.menu.vivid_blue",
        HIGHLIGHTER_RGBA[6],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_7,
        "chrome.menu.vivid_green",
        HIGHLIGHTER_RGBA[7],
    ),
    menu_row_swatch(
        ids::CTX_MENU_OUTLINE_8,
        "chrome.menu.vivid_yellow",
        HIGHLIGHTER_RGBA[8],
    ),
    // ⭐⭐ Os dois verbos da nota (2026-09-30). ⚠️ Os ids das cores continuam os do contorno (o
    //    `apply_showcase_event` ramifica pelo tipo do menu aberto); estes dois são só da nota.
    menu_row(ids::CTX_MENU_NOTE_DUPLICATE, "chrome.menu.note_duplicate"),
    menu_row(ids::CTX_MENU_NOTE_DELETE, "chrome.menu.note_delete"),
];

// Topbar theme cluster click: the themes + the radius presets. Theme entries carry NO colour here:
// the overlay paints four tiles taken from the theme itself (`theme_menu::cores_do_tema`).
// ⭐ **UMA família por aparência** (2026-09-04): o redesenho mostra os presets DERIVADOS
//    (`ph2d_tokens::Theme::MODERN` — os quatro do Godot 4.6 e os quatro coloridos de 2026-09-30),
//    o clássico os quatro de sempre.
//    Misturá-los poria um tema tingido ao lado de um plano sem o artista saber que está a
//    escolher entre dois sistemas. As linhas de baixo (cantos, trilho, espelho, estatísticas,
//    repor) são as mesmas nas duas.
/// As linhas de `ContextMenuKind::ThemeSelector if crate::paint::ui_is_redesign()`.
pub(super) const THEME_SELECTOR_REDESIGN_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_THEME_DARK, "chrome.menu.dark"),
    menu_row(ids::CTX_MENU_THEME_GRAY, "chrome.menu.gray"),
    menu_row(ids::CTX_MENU_THEME_LIGHT, "chrome.menu.light"),
    menu_row(ids::CTX_MENU_THEME_OLED, "chrome.menu.black_oled"),
    menu_row(ids::CTX_MENU_THEME_PLUMBER_RED, "chrome.menu.plumber_red"),
    menu_row(
        ids::CTX_MENU_THEME_PLUMBER_GREEN,
        "chrome.menu.plumber_green",
    ),
    menu_row(ids::CTX_MENU_THEME_SUNSET, "chrome.menu.sunset"),
    menu_row(ids::CTX_MENU_THEME_CANDY, "chrome.menu.candy"),
    menu_row(ids::CTX_MENU_RADIUS_SHARP, "chrome.menu.corners_sharp"),
    menu_row(ids::CTX_MENU_RADIUS_DEFAULT, "chrome.menu.corners_default"),
    menu_row(ids::CTX_MENU_RADIUS_ROUND, "chrome.menu.corners_round"),
    menu_row(
        ids::CTX_MENU_RAIL_SIZE_SMALL,
        "chrome.menu.rail_buttons_small",
    ),
    menu_row(
        ids::CTX_MENU_RAIL_SIZE_MEDIUM,
        "chrome.menu.rail_buttons_medium",
    ),
    menu_row(
        ids::CTX_MENU_RAIL_SIZE_LARGE,
        "chrome.menu.rail_buttons_large",
    ),
    menu_row(ids::CTX_MENU_MIRROR_UI, "chrome.menu.mirror_ui"),
    menu_row(ids::CTX_MENU_SHOW_STATS, "chrome.menu.show_statistics"),
    menu_row(
        ids::MENUBAR_VIEW_RESET_LAYOUT,
        "chrome.menu.reset_panel_layout",
    ),
];

/// As linhas de `ContextMenuKind::ThemeSelector`.
pub(super) const THEME_SELECTOR_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_THEME_FORGE, "chrome.menu.forge_dark"),
    menu_row(ids::CTX_MENU_THEME_PAINT, "chrome.menu.workshop_dark"),
    menu_row(ids::CTX_MENU_THEME_SUNSTONE, "chrome.menu.sunstone_light"),
    menu_row(ids::CTX_MENU_THEME_BLUEPRINT, "chrome.menu.blueprint_light"),
    menu_row(ids::CTX_MENU_RADIUS_SHARP, "chrome.menu.corners_sharp"),
    menu_row(ids::CTX_MENU_RADIUS_DEFAULT, "chrome.menu.corners_default"),
    menu_row(ids::CTX_MENU_RADIUS_ROUND, "chrome.menu.corners_round"),
    menu_row(
        ids::CTX_MENU_RAIL_SIZE_SMALL,
        "chrome.menu.rail_buttons_small",
    ),
    menu_row(
        ids::CTX_MENU_RAIL_SIZE_MEDIUM,
        "chrome.menu.rail_buttons_medium",
    ),
    menu_row(
        ids::CTX_MENU_RAIL_SIZE_LARGE,
        "chrome.menu.rail_buttons_large",
    ),
    menu_row(ids::CTX_MENU_MIRROR_UI, "chrome.menu.mirror_ui"),
    menu_row(ids::CTX_MENU_SHOW_STATS, "chrome.menu.show_statistics"),
    // ⭐ **Repor a arrumação** (Enio, 2026-08-30). ⚠️ Um VERBO no meio de estados: o `—` que
    // os outros levam marca *«isto é um sub-estado do Look»*, e este não é um estado.
    menu_row(
        ids::MENUBAR_VIEW_RESET_LAYOUT,
        "chrome.menu.reset_panel_layout",
    ),
    // "Show Grid" removed — Grid Settings panel now owns the
    // grid visibility toggle (Display section "Show grid").
];

// ── A BARRA DE MENUS (D2, 2026-08-30) ────────────────────────────────────────────
// ⭐⭐ **Quase toda linha aqui leva um id que JÁ EXISTIA**, e é essa a decisão: a barra
// realoja verbos, não os constrói. O `Save` é o do `io_menu`; o `Motion Nodes` é o
// `TOPBAR_MOTION` que o pill levava, e o `motion_toggle` continua a ser quem o despacha.
// ⇒ um verbo, um id, um handler — e nenhuma segunda tabela a divergir da primeira.
/// As linhas de `ContextMenuKind::MenuBarFile`.
// ⭐ **Arrumado em grupos e submenus** (dono 12/11, `docs/BarraDeMenus/01_plano.md`): oito linhas,
// os mesmos verbos. Os divisores vêm de `menu_bar::GROUP_BREAKS`; as cascatas de `menu_bar::CASCADES`.
// Os comandos de família (Kanban, Planner, Som) entram nos submenus pelo `DocMenu`, não aqui.
pub(super) const MENU_BAR_FILE_ROWS: &[MenuRow] = &[
    menu_row(ids::MENUBAR_FILE_NEW_MENU, "chrome.menu.new_submenu"),
    menu_row(ids::CTX_MENU_OPEN_PROJECT, "chrome.menu.open_project_cmd_o"),
    menu_row(ids::MENUBAR_FILE_SCENES, "chrome.menu.scenes"),
    menu_row(ids::CTX_MENU_SAVE, "chrome.menu.save_cmd_s"),
    menu_row(ids::CTX_MENU_SAVE_AS, "chrome.menu.save_as_cmd_shift_s"),
    menu_row(ids::MENUBAR_FILE_IMPORT_MENU, "chrome.menu.import_submenu"),
    menu_row(ids::MENUBAR_FILE_EXPORT_MENU, "chrome.menu.export_submenu"),
];

/// As linhas de `ContextMenuKind::MenuBarFileNew` (+ New Kanban e New Sound, pelas famílias).
pub(super) const MENU_BAR_FILE_NEW_ROWS: &[MenuRow] =
    &[menu_row(ids::MENUBAR_FILE_NEW, "chrome.menu.new_image_cmd_n")];

/// As linhas de `ContextMenuKind::MenuBarFileImport` (+ Import from Trello, pelo Kanban). As do
/// Pixel ficam CINZENTAS sem um Pixel Sprite em Draw (`menu_row_is_unavailable`): nunca somem.
pub(super) const MENU_BAR_FILE_IMPORT_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_IMPORT, "chrome.menu.import_cmd_shift_i"),
    menu_row(
        ids::CTX_MENU_PIXEL_IMPORT_SHEET,
        "chrome.menu.pixel_import_sheet",
    ),
    menu_row(ids::CTX_MENU_PIXEL_OPEN, "chrome.menu.pixel_open"),
];

/// As linhas de `ContextMenuKind::MenuBarFileExport` (+ Export Kanban, pelo Kanban).
pub(super) const MENU_BAR_FILE_EXPORT_ROWS: &[MenuRow] = &[
    // ⭐ **Export SVG…** (report do Enio, 2026-09-04): o gate
    // `the_bar_relocated_every_row_of_the_menus_it_replaced` garante que ele tem porta.
    menu_row(ids::CTX_MENU_EXPORT_SVG, "chrome.menu.export_svg"),
    menu_row(ids::CTX_MENU_PIXEL_EXPORT_AS, "chrome.menu.pixel_export_as"),
    menu_row(
        ids::CTX_MENU_PIXEL_EXPORT_SHEET,
        "chrome.menu.pixel_export_sheet",
    ),
    menu_row(
        ids::CTX_MENU_PIXEL_EXPORT_TILESET,
        "chrome.menu.pixel_export_tileset",
    ),
    menu_row(
        ids::CTX_MENU_PIXEL_REPEAT_EXPORT,
        "chrome.menu.pixel_repeat_export",
    ),
];

// ⚠️ `TOOL_UNDO`/`TOOL_REDO` são os ids do TRILHO, e é de propósito: o verbo é o mesmo, e
// duplicá-lo daria dois botões a desfazer coisas diferentes no dia em que um deles fosse
// esquecido. Quem despacha continua a ser o `chrome::rail_tools`.
/// As linhas de `ContextMenuKind::MenuBarEdit`.
pub(super) const MENU_BAR_EDIT_ROWS: &[MenuRow] = &[
    menu_row(ids::TOOL_UNDO, "chrome.menu.undo_cmd_z"),
    menu_row(ids::TOOL_REDO, "chrome.menu.redo_cmd_shift_z"),
    menu_row(ids::MENUBAR_EDIT_PREFERENCES, "chrome.menu.preferences"),
];

// ⚠️ **Mirror UI / Show Statistics / Corners / Rail Buttons NÃO se repetem aqui** — eles
// vivem no `ThemeSelector`, que esta linha abre como categoria. Uma entrada repetida em
// dois menus é a tabela paralela outra vez, com o sintoma pior: os dois estados a
// discordar à vista.
/// As linhas de `ContextMenuKind::MenuBarView`.
pub(super) const MENU_BAR_VIEW_ROWS: &[MenuRow] = &[
    menu_row(ids::RAIL_SHOW_HIERARCHY, "chrome.menu.hierarchy"),
    menu_row(ids::RAIL_SHOW_INSPECTOR, "chrome.menu.inspector"),
    menu_row(ids::MENUBAR_VIEW_RULERS, "chrome.menu.rulers"),
    // F5: a porta VISÍVEL do toque de 4 dedos — um verbo (com o modo ligado a barra some).
    menu_row(ids::MENUBAR_VIEW_FOCUS, "chrome.menu.focus_mode"),
    // F8: a porta VISÍVEL do beliscão rápido, e o enquadrar da seleção (verbos).
    menu_row(ids::MENUBAR_VIEW_FRAME_ALL, "chrome.menu.frame_all"),
    menu_row(
        ids::MENUBAR_VIEW_FRAME_SELECTED,
        "chrome.menu.frame_selected",
    ),
    menu_row(
        ids::MENUBAR_VIEW_TOOL_ROW_BOTTOM,
        "chrome.menu.tool_row_bottom",
    ),
    menu_row(ids::MENUBAR_VIEW_THEME, "chrome.menu.theme"),
    // ⭐⭐⭐ **O verbo de RECUPERAÇÃO mora aqui** (2026-09-07). Ele existia só dentro do
    // popup do TEMA, e o dono — a precisar dele depois de uma coluna lhe encher o ecrã de
    // painéis — não o encontrou. ⚠️ *Um verbo de recuperação escondido dentro de um
    // selector de aparência é um verbo que não existe no minuto em que é preciso.*
    //
    // ⚠️ **A nota acima proíbe repetir ESTADOS, e isto é um VERBO** — o id já se chama
    // `MENUBAR_VIEW_…`, e o que aquela lei teme (dois estados a discordar à vista) não tem
    // como acontecer a uma acção sem estado.
    menu_row(
        ids::MENUBAR_VIEW_RESET_LAYOUT,
        "chrome.menu.reset_panel_layout",
    ),
    // ⭐⭐⭐ **AS DUAS COLUNAS** (2026-09-19) — a metade da ordem de 2026-09-09 que faltava.
    //
    // > *«Vamos retirar a opção de colapsar arrastando. Deixa o colapsar apenas no menu da barra
    // > superior.»* — Enio
    //
    // ⛔ O gesto da borda saiu na w49 e este item nunca foi escrito ⇒ entre esse dia e hoje **não
    // havia maneira nenhuma de fechar uma coluna**. A lei (`dock_columns`) estava viva, gateada, e
    // com ZERO chamadores de produto.
    //
    // ⚠️ **Elas ficam DEPOIS do `Reset Panel Layout` de propósito:** quem abre este menu com uma
    // coluna a encher-lhe o ecrã procura primeiro o verbo de recuperação, e foi por ele estar
    // escondido dentro do selector de TEMA que o dono não o achou em 2026-09-07.
    menu_row(ids::MENUBAR_VIEW_COLUMN_LEFT, "chrome.menu.left_column"),
    menu_row(ids::MENUBAR_VIEW_COLUMN_RIGHT, "chrome.menu.right_column"),
];

// ⭐ **Os treze toggles de módulo.** Entre a retirada da barra de pills (2026-08-30) e
// esta barra, o único caminho até eles era a tecla `F9` — que é um interruptor de
// bissecção, não uma porta de produto.
/// As linhas de `ContextMenuKind::MenuBarWindow`.
// ⭐ **Arrumado** (dono 12/11): Input Map e Assets em cima (abas e bibliotecas), os painéis de mundo,
// e os submenus Audio · Pixel · Plan. As ferramentas de quem faz o app foram para Help ▸ Developer.
pub(super) const MENU_BAR_WINDOW_ROWS: &[MenuRow] = &[
    // O Input Map é uma aba central; morava em Edit ▸ Preferences ▸ (a 14.ª linha). Mesmo id.
    menu_row(ids::CTX_MENU_SETTINGS_INPUT_MAP, "chrome.menu.input_map"),
    // ⭐⭐⭐ **A BIBLIOTECA** (report do Enio, 2026-09-05: *«vc não colocou nenhum meio de abrir a
    // janela de assets»*). O censo `every_topbar_verb_has_a_door_that_is_not_the_legacy_key` vela.
    menu_row(ids::TOPBAR_RIGHT_ASSETS, "chrome.menu.assets"),
    menu_row(ids::TOPBAR_PHYSICS, "chrome.menu.physics"),
    // ⭐⭐⭐ **AS TAGS** (TOP-20 #9, W4): a **única** porta do painel num projecto sem tag nenhuma.
    menu_row(ids::TOPBAR_TAGS, "chrome.menu.tags"),
    // ⭐⭐⭐ **OS OSSOS** (ordem do dono, 2026-09-09): a única porta numa cena sem ossos.
    menu_row(ids::TOPBAR_SKELETON, "chrome.menu.bones"),
    // As MACROS (ADR-0186, W5d): a única porta do painel, que nasce fechado.
    menu_row(ids::TOPBAR_MACROS, "chrome.menu.macros"),
    menu_row(ids::TOPBAR_MOTION, "chrome.menu.motion_nodes"),
    menu_row(ids::TOPBAR_GRID_SETTINGS, "chrome.menu.grid_settings"),
    menu_row(ids::MENUBAR_WINDOW_AUDIO_MENU, "chrome.menu.audio_submenu"),
    menu_row(ids::MENUBAR_WINDOW_PIXEL_MENU, "chrome.menu.pixel_submenu"),
    menu_row(ids::MENUBAR_WINDOW_PLAN_MENU, "chrome.menu.plan_submenu"),
];

/// As linhas de `ContextMenuKind::MenuBarWindowAudio` (+ os sons abertos, pelo SFX Lab).
pub(super) const MENU_BAR_WINDOW_AUDIO_ROWS: &[MenuRow] = &[
    menu_row(ids::TOPBAR_AUDIO_MIXER, "chrome.menu.audio_mixer"),
    menu_row(ids::TOPBAR_AUDIO_EDITOR, "chrome.menu.audio_editor"),
];

/// `ContextMenuKind::MenuBarWindowPlan`: só linhas de família (Kanban, Planner), pelo `DocMenu`.
pub(super) const MENU_BAR_WINDOW_PLAN_ROWS: &[MenuRow] = &[];

/// As linhas de `ContextMenuKind::MenuBarHelp`.
pub(super) const MENU_BAR_HELP_ROWS: &[MenuRow] = &[
    menu_row(ids::MENUBAR_HELP_COMMANDS, "chrome.menu.commands_cmd_k"),
    menu_row(ids::MENUBAR_HELP_DEVELOPER_MENU, "chrome.menu.developer_submenu"),
];

/// As linhas de `ContextMenuKind::MenuBarHelpDeveloper` — as ferramentas de quem faz o app.
pub(super) const MENU_BAR_HELP_DEVELOPER_ROWS: &[MenuRow] = &[
    menu_row(ids::TOPBAR_TOKENS, "chrome.menu.design_tokens"),
    menu_row(ids::TOPBAR_AUTHORED, "chrome.menu.authored_ui"),
    menu_row(ids::TOPBAR_WIDGET_GALLERY, "chrome.menu.widget_gallery"),
    // ⭐ A BANCADA (`ph2d-panel-widget-lab`): a galeria diz o que o editor É; esta o que pode ser.
    menu_row(ids::TOPBAR_WIDGET_LAB, "chrome.menu.widget_lab"),
];

/// As linhas de `ContextMenuKind::MenuBarWindowPixel`.
pub(super) const MENU_BAR_WINDOW_PIXEL_ROWS: &[MenuRow] = &[
    // O PIXEL (ADR-0196): abre sozinho no Pixel Sprite ▸ Edit; esta é a porta fora dele.
    menu_row(ids::TOPBAR_PIXEL, "chrome.menu.pixel"),
    // A janela Preview do Pixel (P5): F7 e Shift+Enter abrem-na em Draw; esta é a porta à vista.
    menu_row(ids::TOPBAR_PIXEL_PREVIEW, "chrome.menu.pixel_preview"),
    // As janelas acessórias do Pixel (smoke do dono 08/11). O rótulo é o `TITLE` do painel: um nome.
    menu_row(ids::TOPBAR_PIXEL_SLICES, "panel.pixel_slices.title"),
    menu_row(ids::TOPBAR_PIXEL_TILES, "panel.pixel_tiles.title"),
    menu_row(ids::TOPBAR_PIXEL_HISTORY, "panel.pixel_history.title"),
    menu_row(ids::TOPBAR_PIXEL_KEYS, "panel.pixel_keys.title"),
    menu_row(ids::TOPBAR_PIXEL_PREFS, "panel.pixel_prefs.title"),
    menu_row(ids::TOPBAR_PIXEL_LINT, "panel.pixel_lint.title"),
];

/// As linhas de `ContextMenuKind::SaveMenu`.
pub(super) const SAVE_MENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_SAVE, "chrome.menu.save_cmd_s"),
    menu_row(ids::CTX_MENU_SAVE_AS, "chrome.menu.save_as_cmd_shift_s"),
    menu_row(ids::CTX_MENU_EXPORT_SVG, "chrome.menu.export_svg"),
];

/// As linhas de `ContextMenuKind::OpenMenu`.
pub(super) const OPEN_MENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_OPEN_PROJECT, "chrome.menu.open_project_cmd_o"),
    menu_row(ids::CTX_MENU_IMPORT, "chrome.menu.import_cmd_shift_i"),
];

// M14.6 F + M14.7: Rename, additive ops, the milder revert, Delete last (Unity/Godot/Blender).
/// As linhas de `ContextMenuKind::HierarchyRow { .. }`.
pub(super) const HIERARCHY_ROW_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_HIER_RENAME, "chrome.menu.rename"),
    menu_row(ids::CTX_MENU_HIER_DUPLICATE, "chrome.menu.duplicate"),
    menu_row(ids::CTX_MENU_HIER_ADD_CHILD, "chrome.menu.add_child"),
    // ⭐ Group/Ungroup (Enio, 30/08): a forma mais suave de juntar; com Merge e Pack abaixo, os três
    // respondem «quão junto?» em ordem crescente de dano. O par fica junto (o inverso vê-se).
    menu_row(ids::CTX_MENU_HIER_GROUP, "chrome.menu.group"),
    menu_row(ids::CTX_MENU_HIER_UNGROUP, "chrome.menu.ungroup"),
    menu_row(
        ids::CTX_MENU_HIER_MERGE_SPRITES,
        "chrome.menu.merge_sprites",
    ),
    // A mesma fusão, reversível: cada sprite numa camada do Painter.
    menu_row(
        ids::CTX_MENU_HIER_MERGE_TO_LAYERS,
        "chrome.menu.merge_to_layers",
    ),
    // Vizinho do Merge: ARRANJA as peças numa folha e mantém cada uma editável.
    menu_row(ids::CTX_MENU_HIER_PACK_SHEET, "chrome.menu.pack_into_sheet"),
    // Os três verbos da folha ficam juntos e nesta ordem — entrar, arrumar, sair —, que é
    // a ordem em que o artista os encontra. O do meio ESTEVE dentro do primeiro, e foi
    // por isso que ninguém o achou.
    menu_row(
        ids::CTX_MENU_HIER_ARRANGE_SHEET,
        "chrome.menu.auto_arrange_pieces",
    ),
    // As duas SAÍDAS do bake, lado a lado e nesta ordem: assar muda a cena, exportar
    // escreve ficheiros. Ler uma a seguir à outra é o que torna a diferença óbvia.
    menu_row(ids::CTX_MENU_HIER_BAKE_SHEET, "chrome.menu.bake_sheet"),
    menu_row(ids::CTX_MENU_HIER_EXPORT_SHEET, "chrome.menu.export_sheet"),
    // A exportação de UMA sprite, ao lado da da folha: os dois escrevem ficheiros, e o
    // nome diz qual. Plano `docs/Sprite_projeto/18` W9 (Enio, 2026-08-21).
    menu_row(ids::CTX_MENU_HIER_EXPORT_IMAGE, "chrome.menu.export_image"),
    // Só para Image · Vector · Flip: o pintor esconde-a nas outras (`menu_row_is_hidden`).
    menu_row(
        ids::CTX_MENU_HIER_SEND_TO_BOARD,
        "chrome.menu.send_to_board",
    ),
    // Só para Image · Flip (os que têm píxeis): o pintor esconde-a nas outras.
    menu_row(
        ids::CTX_MENU_HIER_KANBAN_BACKGROUND,
        "chrome.menu.kanban_background",
    ),
    menu_row(
        ids::CTX_MENU_HIER_REMOVE_FROM_SHEET,
        "chrome.menu.remove_from_sheet",
    ),
    menu_row(
        ids::CTX_MENU_HIER_USE_AS_BRUSH_SHAPE,
        "chrome.menu.use_as_brush_shape",
    ),
    menu_row(
        ids::CTX_MENU_HIER_USE_AS_BRUSH_TEXTURE,
        "chrome.menu.use_as_brush_grain",
    ),
    menu_row(
        ids::CTX_MENU_HIER_USE_AS_PAPER,
        "chrome.menu.use_as_watercolor_paper",
    ),
    menu_row(
        ids::CTX_MENU_HIER_USE_AS_GRANULATION,
        "chrome.menu.use_as_granulation",
    ),
    menu_row(ids::CTX_MENU_HIER_USE_AS_FLOW, "chrome.menu.use_as_flow"),
    // Só para Vector: o pintor esconde-a nas outras (`menu_row_is_hidden`).
    menu_row(
        ids::CTX_MENU_HIER_CONVERT_TO_OUTLINE,
        "chrome.menu.convert_to_collision_outline",
    ),
    menu_row(
        ids::CTX_MENU_HIER_RESET_TRANSFORM,
        "chrome.menu.reset_transform",
    ),
    // ⚠️ Numa linha que NÃO é instância ele responde com um aviso, e não com nada: a
    // tabela deste menu é plana (não sabe o que a linha é), e um item que come o clique
    // em silêncio é pior que um ausente.
    // ⭐ **A família da INSTÂNCIA** (ADR-0164 / F4.5), na ordem do gesto: criar a receita ·
    // pôr outra cópia · promover a excepção · devolvê-la · cortar o vínculo.
    // ⚠️ Todos respondem numa linha a que não se aplicam — a tabela é plana.
    menu_row(ids::CTX_MENU_HIER_MAKE_COMPONENT, "chrome.menu.make_prefab"),
    // ⭐⭐⭐ **ABRIR** vem logo a seguir a CRIAR, e antes de instanciar: é a ordem em que o
    // artista os encontra — faço um, entro nele, ponho mais cópias.
    menu_row(ids::CTX_MENU_HIER_EDIT_PREFAB, "chrome.menu.edit_prefab"),
    menu_row(ids::CTX_MENU_HIER_INSTANTIATE, "chrome.menu.instantiate"),
    menu_row(
        ids::CTX_MENU_HIER_INSTANTIATE_LINKED,
        "chrome.menu.instantiate_linked",
    ),
    menu_row(
        ids::CTX_MENU_HIER_APPLY_TO_MASTER,
        "chrome.menu.apply_to_prefab",
    ),
    menu_row(
        ids::CTX_MENU_HIER_REVERT_TO_MASTER,
        "chrome.menu.revert_to_prefab",
    ),
    menu_row(ids::CTX_MENU_HIER_DETACH, "chrome.menu.detach_from_prefab"),
    menu_row(
        ids::CTX_MENU_HIER_REMOVE_FROM_LIBRARY,
        "chrome.menu.remove_from_library",
    ),
    menu_row(ids::CTX_MENU_HIER_DELETE, "chrome.menu.delete"),
];

// ⭐⭐ **O cartão da biblioteca** (plano 07, etapa C). Três itens, na ordem do gesto:
// *usar* · *ver quem usa* · *tirar*.
//
// ⚠️ **Plana como a da Hierarquia, e pela mesma razão:** ela não sabe se a célula é um
// Prefab ou uma Imagem. As duas famílias respondem aos três — e as recusas NOMEIAM o
// motivo, que numa Imagem é sempre o mesmo facto: ela está na biblioteca porque um objecto
// a usa, então *«tirar»* teria de tirá-la dos objectos, que é outro gesto.
// ⭐⭐ A linha de catálogo. ⚠️ Só DOIS itens, e nenhum deles é *«criar»* — criar é o `+` do
// cabeçalho da coluna, que não precisa de um sujeito.
/// As linhas de `ContextMenuKind::CatalogRow { .. }`.
pub(super) const CATALOG_ROW_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_CATALOG_RENAME, "chrome.menu.rename"),
    menu_row(ids::CTX_MENU_CATALOG_DELETE, "chrome.menu.delete"),
];

// ⭐⭐ O cartão da biblioteca: **três itens, na ordem do gesto** — usar · ver quem usa ·
// tirar. ⚠️ **Este comentário voltou para cima da arm que ele descreve** (auditoria de
// 2026-08-30): a arm do catálogo foi inserida no meio dele, e o leitor caía num «Três
// itens» imediatamente acima de uma arm com DOIS, com a arm do cartão a ficar sem nota
// nenhuma. *Um comentário separado do seu item muda de dono.*
/// As linhas de `ContextMenuKind::AssetCard { .. }`.
pub(super) const ASSET_CARD_ROWS: &[MenuRow] = &[
    // ⭐⭐⭐ **EDITAR vem PRIMEIRO** (report do Enio, 2026-09-05) — e a ordem é medida, não
    // estética: *Instantiate* tem uma segunda porta (o duplo-clique no cartão) e *Edit* não
    // tinha nenhuma. *O item que é o ÚNICO acesso ao seu verbo lê-se antes do que se
    // alcança de duas maneiras.*
    menu_row(ids::CTX_MENU_ASSET_EDIT, "chrome.menu.edit_prefab"),
    menu_row(ids::CTX_MENU_ASSET_INSTANTIATE, "chrome.menu.instantiate"),
    // ⭐⭐ **O IRMÃO, colado a ele** — a escolha entre os dois só existe neste instante.
    menu_row(
        ids::CTX_MENU_ASSET_INSTANTIATE_LINKED,
        "chrome.menu.instantiate_linked",
    ),
    menu_row(ids::CTX_MENU_ASSET_SELECT_USERS, "chrome.menu.select_users"),
    // ⭐⭐ As duas metades de D9. ⚠️ Elas ficam DEPOIS do *Select users* de propósito: a
    // pergunta da cena vem antes da da biblioteca, que é a ordem em que o artista repara
    // que precisa da segunda.
    menu_row(ids::CTX_MENU_ASSET_USES, "chrome.menu.show_what_it_uses"),
    menu_row(ids::CTX_MENU_ASSET_USED_BY, "chrome.menu.show_what_uses_it"),
    menu_row(
        ids::CTX_MENU_ASSET_REMOVE,
        "chrome.menu.remove_from_library",
    ),
    // ⭐⭐⭐ **A troca por um componente sem parentesco** (plano F5, o último critério).
    //
    // ⚠️ **Três linhas e não uma, porque o MODO é o gesto.** Sem antepassado comum não há
    // mapa derivado, só palpite — e o plano proíbe o app de o escolher sozinho (HR-5).
    // A linha sem adjectivo é o `None` do Unity e o caminho seguro; as duas de baixo usam
    // o prefixo `—` do selector de tema, que é como esta casa já escreve um sub-grupo.
    //
    // ⚠️ **O sujeito é a SELECÇÃO**, ao contrário de todas as linhas acima — daí o rótulo
    // a nomeá-la: um item que age sobre outra coisa que a apontada tem de o dizer.
    menu_row(
        ids::CTX_MENU_ASSET_REPLACE,
        "chrome.menu.replace_selection_with_this",
    ),
    menu_row(
        ids::CTX_MENU_ASSET_REPLACE_BY_NAME,
        "chrome.menu.and_match_overrides_by_name",
    ),
    menu_row(
        ids::CTX_MENU_ASSET_REPLACE_BY_TREE,
        "chrome.menu.and_match_overrides_by_position",
    ),
];

/// As linhas de `ContextMenuKind::AssetDocCard { .. }` (D17): um documento abre-se ou apaga-se.
pub(super) const ASSET_DOC_CARD_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_ASSET_OPEN, "chrome.menu.open"),
    menu_row(ids::CTX_MENU_ASSET_DELETE, "chrome.menu.delete_document"),
];

// Painter Falloff curve point handle (Blender per-point handle types).
/// As linhas de `ContextMenuKind::FalloffPointHandle`.
pub(super) const FALLOFF_POINT_HANDLE_ROWS: &[MenuRow] = &[
    menu_row(
        ids::CTX_MENU_FALLOFF_HANDLE_VECTOR,
        "chrome.menu.handle.vector",
    ),
    menu_row(ids::CTX_MENU_FALLOFF_HANDLE_AUTO, "chrome.menu.handle.auto"),
];

// On-canvas Curve / Free Hand point handle (the five vector-app continuity kinds).
/// As linhas de `ContextMenuKind::CurvePointHandle`.
pub(super) const CURVE_POINT_HANDLE_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_CURVE_HANDLE_FREE, "chrome.menu.handle.free"),
    menu_row(
        ids::CTX_MENU_CURVE_HANDLE_ALIGNED,
        "chrome.menu.handle.aligned",
    ),
    menu_row(
        ids::CTX_MENU_CURVE_HANDLE_SYMMETRIC,
        "chrome.menu.handle.symmetric",
    ),
    menu_row(
        ids::CTX_MENU_CURVE_HANDLE_VECTOR,
        "chrome.menu.handle.vector",
    ),
    menu_row(ids::CTX_MENU_CURVE_HANDLE_AUTO, "chrome.menu.handle.auto"),
];

// On-canvas motion-path anchor handle types (the vector Node trio, ADR-0141).
/// As linhas de `ContextMenuKind::MotionPathAnchor { .. }`.
pub(super) const MOTION_PATH_ANCHOR_ROWS: &[MenuRow] = &[
    menu_row(
        ids::CTX_MENU_PATH_HANDLE_CORNER,
        "chrome.menu.handle.corner",
    ),
    menu_row(
        ids::CTX_MENU_PATH_HANDLE_SMOOTH,
        "chrome.menu.handle.smooth",
    ),
    menu_row(
        ids::CTX_MENU_PATH_HANDLE_SYMMETRIC,
        "chrome.menu.handle.symmetric",
    ),
];
