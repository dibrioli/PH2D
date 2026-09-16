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

/// As linhas de `ContextMenuKind::SectionOutline { .. }`.
pub(super) const SECTION_OUTLINE_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_OUTLINE_NONE, "chrome.menu.no_outline"),
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
];

// Right-clicked on a note: 5 background-color options (reuses the outline color slot ids;
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
];

// Topbar theme cluster click: 4 themes + 3 radius presets. Theme entries get a small accent
// swatch tinted with each theme's flavor so the user can recognize them at a glance.
// ⭐ **UMA família por aparência** (2026-09-04): o redesenho mostra os quatro presets
//    DERIVADOS (Godot 4.6, `ph2d_tokens::Theme::MODERN`), o clássico os quatro de sempre.
//    Misturá-los poria um tema tingido ao lado de um plano sem o artista saber que está a
//    escolher entre dois sistemas. As linhas de baixo (cantos, trilho, espelho, estatísticas,
//    repor) são as mesmas nas duas.
/// As linhas de `ContextMenuKind::ThemeSelector if crate::paint::ui_is_redesign()`.
pub(super) const THEME_SELECTOR_REDESIGN_ROWS: &[MenuRow] = &[
    menu_row_swatch(
        ids::CTX_MENU_THEME_DARK,
        "chrome.menu.dark",
        [0x56, 0x9e, 0xff, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_THEME_GRAY,
        "chrome.menu.gray",
        [0x70, 0xba, 0xfa, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_THEME_LIGHT,
        "chrome.menu.light",
        [0x2e, 0x80, 0xff, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_THEME_OLED,
        "chrome.menu.black_oled",
        [0x73, 0xbf, 0xff, 0xFF],
    ),
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
    menu_row_swatch(
        ids::CTX_MENU_THEME_FORGE,
        "chrome.menu.forge_dark",
        [0xc8, 0x4b, 0xa0, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_THEME_PAINT,
        "chrome.menu.workshop_dark",
        [0x4b, 0xa0, 0xc8, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_THEME_SUNSTONE,
        "chrome.menu.sunstone_light",
        [0xf0, 0xc0, 0x4f, 0xFF],
    ),
    menu_row_swatch(
        ids::CTX_MENU_THEME_BLUEPRINT,
        "chrome.menu.blueprint_light",
        [0x6c, 0x8e, 0xc8, 0xFF],
    ),
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
// realoja verbos, não os constrói. O `Save` é o do `io_menu`; o `Vector` é o
// `TOPBAR_VECTOR` que o pill levava, e o painel do vetor continua a ser quem o despacha.
// ⇒ um verbo, um id, um handler — e nenhuma segunda tabela a divergir da primeira.
/// As linhas de `ContextMenuKind::MenuBarFile`.
pub(super) const MENU_BAR_FILE_ROWS: &[MenuRow] = &[
    menu_row(ids::MENUBAR_FILE_NEW, "chrome.menu.new_image_cmd_n"),
    menu_row(ids::MENUBAR_FILE_SCENES, "chrome.menu.scenes"),
    menu_row(ids::CTX_MENU_OPEN_PROJECT, "chrome.menu.open_project_cmd_o"),
    menu_row(ids::CTX_MENU_IMPORT, "chrome.menu.import_cmd_shift_i"),
    menu_row(ids::CTX_MENU_SAVE, "chrome.menu.save_cmd_s"),
    menu_row(ids::CTX_MENU_SAVE_AS, "chrome.menu.save_as_cmd_shift_s"),
    // ⭐ **Export SVG…** (report do Enio, 2026-09-04: *«funções criadas por outros módulos
    // não aparecem na UI»*). ⚠️ Ele **existe desde 2026-09-02** — a `line/Vector`
    // acrescentou-o ao `SaveMenu`, que era o menu do pill `TOPBAR_SAVE`; esta barra tinha
    // substituído o pill dois dias antes, e o merge das duas linhas foi **limpo**: nenhuma
    // tocou na linha da outra, e o verbo ficou a existir num menu que já não tem botão.
    // *Duas linhas a mexer na mesma superfície fundem sem conflito e uma delas evapora.*
    // ⇒ o gate `the_bar_relocated_every_row_of_the_menus_it_replaced` apanha o próximo.
    menu_row(ids::CTX_MENU_EXPORT_SVG, "chrome.menu.export_svg"),
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
];

// ⭐ **Os treze toggles de módulo.** Entre a retirada da barra de pills (2026-08-30) e
// esta barra, o único caminho até eles era a tecla `F9` — que é um interruptor de
// bissecção, não uma porta de produto.
/// As linhas de `ContextMenuKind::MenuBarWindow`.
pub(super) const MENU_BAR_WINDOW_ROWS: &[MenuRow] = &[
    menu_row(ids::TOPBAR_VECTOR, "chrome.menu.vector"),
    menu_row(ids::TOPBAR_MOTION, "chrome.menu.motion_nodes"),
    menu_row(ids::TOPBAR_FLIP, "chrome.menu.flip"),
    menu_row(ids::TOPBAR_PHYSICS, "chrome.menu.physics"),
    // ⭐⭐⭐ **OS OSSOS** (ordem do dono, 2026-09-09: *«o Menu Windows deve receber a opção
    // de Bones»*). ⚠️ Vizinho da Física porque os dois são painéis de MUNDO — e é a única
    // porta do painel numa cena **sem** ossos, que é onde o artista carrega em *Create*
    // para fazer o primeiro.
    menu_row(ids::TOPBAR_SKELETON, "chrome.menu.bones"),
    menu_row(ids::TOPBAR_SCULPT3D, "chrome.menu.sculpt_3d"),
    menu_row(ids::TOPBAR_MODEL3D, "chrome.menu.model_3d"),
    menu_row(ids::TOPBAR_IMAGE_TOOLS, "chrome.menu.image_tools"),
    menu_row(ids::TOPBAR_AUDIO_MIXER, "chrome.menu.audio_mixer"),
    menu_row(ids::TOPBAR_AUDIO_EDITOR, "chrome.menu.audio_editor"),
    menu_row(ids::TOPBAR_TOKENS, "chrome.menu.design_tokens"),
    menu_row(ids::TOPBAR_AUTHORED, "chrome.menu.authored_ui"),
    menu_row(ids::TOPBAR_WIDGET_GALLERY, "chrome.menu.widget_gallery"),
    // ⭐ A BANCADA. ⚠️ Vizinha da galeria na lista porque é onde o leitor a procura, e
    // **separada dela** porque a galeria diz o que o editor É e esta diz o que ele pode
    // vir a ser (`ph2d-panel-widget-lab`, doc-comment do `lib.rs`).
    menu_row(ids::TOPBAR_WIDGET_LAB, "chrome.menu.widget_lab"),
    menu_row(ids::TOPBAR_GRID_SETTINGS, "chrome.menu.grid_settings"),
    // ⭐⭐⭐ **A BIBLIOTECA** (report do Enio, 2026-09-05: *«vc não colocou nenhum meio de
    // abrir a janela de assets»* — e ele estava certo).
    //
    // ⛔⛔ **A porta foi CLASSIFICADA COMO LIXO por um motivo que expirou no mesmo dia.**
    // A `line/UIUX` tirou os 29 pills (a pedido dele) e pôs este id no `NO_DOOR_PENDING`
    // com a razão *«MORTO PRE-EXISTENTE: … SEM consumidor nenhum no repo inteiro»* — que
    // **era verdade quando ela varreu**, e deixou de ser horas depois, quando a
    // `line/components` lhe deu o navegador de assets como consumidor. As duas linhas
    // compilaram, o merge não teve conflito, e o app ficou com um painel vivo, registado,
    // despachado e **inalcançável**.
    //
    // ⚠️ *É o §0.0 com um MOTIVO no lugar do número: quem torna alcançável o que uma nota
    // declarou morto tem de reconferir a nota* — e quem a escreveu não tinha como saber.
    // ⭐ O censo `every_topbar_verb_has_a_door_that_is_not_the_legacy_key` **apanhou-o**:
    // a metade de obsolescência dele recusou a entrada no instante em que esta linha
    // nasceu.
    menu_row(ids::TOPBAR_RIGHT_ASSETS, "chrome.menu.assets"),
];

// ⚠️ **O transporte é UM relógio** (`ph2d_core::Playhead`): física, Motion, Timeline e
// Flip andam todos nele, e estes três verbos conduzem-nos de uma vez.
// ⛔ *Rewind* estava **sem porta** desde a retirada dos pills — o `Espaço` alterna
// tocar/pausar e as vírgulas andam quadro a quadro, mas nada rebobinava.
/// As linhas de `ContextMenuKind::MenuBarRun`.
pub(super) const MENU_BAR_RUN_ROWS: &[MenuRow] = &[
    menu_row(ids::TOPBAR_PLAY_BUTTON, "chrome.menu.play_space"),
    menu_row(ids::TOPBAR_PAUSE, "chrome.menu.pause_space"),
    menu_row(ids::TOPBAR_RESET, "chrome.menu.rewind"),
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

// Settings cluster (gear) — TOP-LEVEL categories; each gets a `ChevronRight` from the row loop.
/// As linhas de `ContextMenuKind::SettingsMenu`.
pub(super) const SETTINGS_MENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_SETTINGS_PPM, "chrome.menu.pixels_per_meter"),
    menu_row(ids::CTX_MENU_SETTINGS_UNIT, "chrome.menu.display_unit"),
    menu_row(ids::CTX_MENU_SETTINGS_ANGLE, "chrome.menu.angle_unit"),
    menu_row(ids::CTX_MENU_SETTINGS_FILTER, "chrome.menu.image_filter"),
    menu_row(ids::CTX_MENU_SETTINGS_DISPLAY, "chrome.menu.display"),
    menu_row(ids::CTX_MENU_SETTINGS_TEXT, "chrome.menu.text_rendering"),
    menu_row(ids::CTX_MENU_SETTINGS_MOTION, "chrome.menu.motion"),
    // ⚠️ **Esta entrada NÃO é uma categoria** — ela abre a janela flutuante do Input Map,
    // não um submenu. Fica aqui porque é a casa que o Godot lhe dá (*Project Settings >
    // Input Map*) e a equivalência era o pedido; as reticências dizem *"isto abre uma
    // janela"*, que é a convenção que toda a UI de desktop usa.
    // ⛔ **A NOTAR NO SMOKE:** o laço deste menu põe um `ChevronRight` em cada linha por
    // ser `SettingsMenu`, e um chevron promete um submenu que esta linha não tem. Se o Enio
    // o vir como errado, a cura é o laço perguntar pela LINHA e não pelo tipo do menu.
    menu_row(ids::CTX_MENU_SETTINGS_INPUT_MAP, "chrome.menu.input_map"),
];

// Pixels-per-meter submenu — 5 presets (retro 16 · Unity 32 · Godot 100 · HD 256 · 4K 1024).
/// As linhas de `ContextMenuKind::SettingsPpmSubmenu`.
pub(super) const SETTINGS_PPM_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_PPM_16, "chrome.menu.n16_retro_tile"),
    menu_row(ids::CTX_MENU_PPM_32, "chrome.menu.n32_unity_2d"),
    menu_row(ids::CTX_MENU_PPM_100, "chrome.menu.n100_godot"),
    menu_row(ids::CTX_MENU_PPM_256, "chrome.menu.n256_hd_2d"),
    menu_row(ids::CTX_MENU_PPM_1024, "chrome.menu.n1024_4k_ref"),
];

/// As linhas de `ContextMenuKind::SettingsUnitSubmenu`.
pub(super) const SETTINGS_UNIT_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_UNIT_METERS, "chrome.menu.meters"),
    menu_row(ids::CTX_MENU_UNIT_PIXELS, "chrome.menu.pixels"),
];

// Angle-unit submenu — o irmão do de cima, para o ÂNGULO. O armazenamento
// continua em radianos; isto só troca o FORMATO.
/// As linhas de `ContextMenuKind::SettingsAngleSubmenu`.
pub(super) const SETTINGS_ANGLE_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_ANGLE_DEGREES, "chrome.menu.degrees"),
    menu_row(ids::CTX_MENU_ANGLE_RADIANS, "chrome.menu.radians"),
];

// Image-filter submenu — the single global sampling mode
// applied to every sprite/texture + the Vello preview.
/// As linhas de `ContextMenuKind::SettingsFilterSubmenu`.
pub(super) const SETTINGS_FILTER_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_FILTER_PIXELART, "chrome.menu.pixel_art_crisp"),
    menu_row(ids::CTX_MENU_FILTER_SMOOTH, "chrome.menu.smooth_bilinear"),
];

// Display submenu — runtime swap-chain present mode. VSync is
// perfectly smooth; Immediate is non-blocking (no mouse-stutter)
// at the cost of vsync-pacing.
/// As linhas de `ContextMenuKind::SettingsDisplaySubmenu`.
pub(super) const SETTINGS_DISPLAY_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_DISPLAY_VSYNC, "chrome.menu.vsync_smooth"),
    menu_row(
        ids::CTX_MENU_DISPLAY_IMMEDIATE,
        "chrome.menu.immediate_no_stutter",
    ),
];

// Text rendering submenu — 4 presets, monotonic in aggressiveness: Default (historic) →
// Crisp Light (boost 30/20/10 + snap-X) → Crisp (60/40/20) → Crisp Heavy (100/70/40).
/// As linhas de `ContextMenuKind::SettingsTextSubmenu`.
pub(super) const SETTINGS_TEXT_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_TEXT_DEFAULT, "chrome.menu.default"),
    menu_row(ids::CTX_MENU_TEXT_CRISP_HEAVY, "chrome.menu.crisp_heavy"),
    menu_row(
        ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS,
        "chrome.menu.crisp_heavy_plus",
    ),
];

// Motion submenu — o carácter da UI viva + o reduced motion.
//
// ⚠️ As duas primeiras linhas são um RÁDIO (o gosto) e a terceira é um TOGGLE (a garantia).
// O bullet significa a mesma coisa nas três — *este é o estado corrente* — que é a
// convenção de menu de plataforma, e é por isso que as três cabem numa tabela só.
/// As linhas de `ContextMenuKind::SettingsMotionSubmenu`.
pub(super) const SETTINGS_MOTION_SUBMENU_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_MOTION_EXPRESSIVE, "chrome.menu.expressive"),
    menu_row(ids::CTX_MENU_MOTION_DISCRETE, "chrome.menu.discrete"),
    menu_row(ids::CTX_MENU_MOTION_REDUCED, "chrome.menu.reduced_motion"),
];

// M14.6 F + M14.7: per-row Hierarchy actions. Order follows
// the Unity / Godot / Blender convention: Rename first (the
// most common edit), then additive ops (Duplicate, Add
// Child), then the milder revert (Reset Transform), with
// Delete last as the destructive endpoint.
/// As linhas de `ContextMenuKind::HierarchyRow { .. }`.
pub(super) const HIERARCHY_ROW_ROWS: &[MenuRow] = &[
    menu_row(ids::CTX_MENU_HIER_RENAME, "chrome.menu.rename"),
    menu_row(ids::CTX_MENU_HIER_DUPLICATE, "chrome.menu.duplicate"),
    menu_row(ids::CTX_MENU_HIER_ADD_CHILD, "chrome.menu.add_child"),
    // ⭐⭐⭐ **AGRUPAR / DESAGRUPAR** (Enio, 2026-08-30), e ficam AQUI de propósito: são a
    // forma mais **suave** de juntar a seleção num objeto, e o bloco abaixo é o das outras
    // duas — o Merge FUNDE os pixels e destrói os originais, o Pack ARRANJA-os numa folha.
    // Lidos em sequência, os três respondem *"quão junto?"* em ordem crescente de dano.
    //
    // ⚠️ O par fica junto porque **um verbo cujo inverso não se vê não se usa**.
    menu_row(ids::CTX_MENU_HIER_GROUP, "chrome.menu.group"),
    menu_row(ids::CTX_MENU_HIER_UNGROUP, "chrome.menu.ungroup"),
    menu_row(
        ids::CTX_MENU_HIER_MERGE_SPRITES,
        "chrome.menu.merge_sprites",
    ),
    // A mesma fusão, mas reversível: cada sprite fica numa camada do Painter. Vizinha da
    // de cima porque a escolha entre as duas só existe neste instante.
    menu_row(
        ids::CTX_MENU_HIER_MERGE_TO_LAYERS,
        "chrome.menu.merge_to_layers",
    ),
    // Vizinho do Merge de propósito: os dois juntam a seleção num objeto. O Merge FUNDE
    // os pixels e destrói os originais; este ARRANJA-os e mantém cada peça viva e
    // editável dentro da folha. A ordem lê-se como "junte-os" → "quão junto?".
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
