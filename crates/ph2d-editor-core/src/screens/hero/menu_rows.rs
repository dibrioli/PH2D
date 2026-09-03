//! **A TABELA de rows de cada menu** — a porta única de *«que linhas este menu tem?»*.
//!
//! Ela vivia INLINE dentro do `paint_context_menu_overlay`, e enquanto o único consumidor era o
//! pintor isso estava certo. Deixou de estar quando a **paleta de comandos global** passou a
//! oferecer os mesmos verbos: uma segunda lista escrita à mão ali seria a tabela paralela que este
//! repo já viu apodrecer duas vezes no chrome da timeline (*o `Nearest` entrou na tabela e a row
//! nasceu morta*), e desta vez com o sintoma pior — a paleta a oferecer um comando que o menu já
//! não tem, ou a esquecer um que ele ganhou.
//!
//! ⚠️ **Isto é *pure code motion*:** o `match` é o mesmo, arm a arm, e o pintor passou a chamá-lo.
//!
//! # O que uma row É, e a fronteira que a paleta respeita
//!
//! Uma row é `(id, rótulo, swatch opcional)`. O **id é o verbo**: apertar a row é levantar
//! `WidgetEvent::Click(id)`, e o `chrome::dispatch_all` resolve — sem rectângulo nenhum. É por
//! isso que a paleta consegue oferecer estas rows e **não** consegue oferecer os *pills* que as
//! abrem (um pill ancora um menu a um rectângulo, e um pick não tem rectângulo).
//!
//! ⚠️ Mas nem toda row é servível assim, e a lista está em
//! [`ContextMenuKind::rows_are_context_free_leaves`].

use crate::ids;
use crate::interaction::ContextMenuKind;
use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;
use ph2d_a11y::NodeId;

/// As rows deste menu, na ordem em que ele as pinta.
///
/// Um `&[]` significa *este menu desenha o próprio corpo* (a lista de cenas, o diálogo de rename, o
/// de imagem nova) — não *«menu vazio»*.
#[must_use]
pub fn menu_rows(kind: ContextMenuKind) -> &'static [(NodeId, &'static str, Option<[u8; 4]>)] {
    match kind {
        ContextMenuKind::CreateNote { .. } => &[(ids::CTX_MENU_CREATE_NOTE, "Create note", None)],
        ContextMenuKind::SectionOutline { .. } => &[
            (ids::CTX_MENU_OUTLINE_NONE, "No outline", None),
            (ids::CTX_MENU_OUTLINE_0, "Yellow", Some(HIGHLIGHTER_RGBA[0])),
            (ids::CTX_MENU_OUTLINE_1, "Pink", Some(HIGHLIGHTER_RGBA[1])),
            (ids::CTX_MENU_OUTLINE_2, "Green", Some(HIGHLIGHTER_RGBA[2])),
            (ids::CTX_MENU_OUTLINE_3, "Blue", Some(HIGHLIGHTER_RGBA[3])),
            (ids::CTX_MENU_OUTLINE_4, "Orange", Some(HIGHLIGHTER_RGBA[4])),
        ],
        // Right-clicked on a note: 5 background-color options (reuses the outline color slot ids;
        // apply_event branches on `last_context_menu.kind` to set the section outline vs the note bg).
        ContextMenuKind::NoteBackground { .. } => &[
            (ids::CTX_MENU_OUTLINE_0, "Yellow", Some(HIGHLIGHTER_RGBA[0])),
            (ids::CTX_MENU_OUTLINE_1, "Pink", Some(HIGHLIGHTER_RGBA[1])),
            (ids::CTX_MENU_OUTLINE_2, "Green", Some(HIGHLIGHTER_RGBA[2])),
            (ids::CTX_MENU_OUTLINE_3, "Blue", Some(HIGHLIGHTER_RGBA[3])),
            (ids::CTX_MENU_OUTLINE_4, "Orange", Some(HIGHLIGHTER_RGBA[4])),
        ],
        // Topbar theme cluster click: 4 themes + 3 radius presets. Theme entries get a small accent
        // swatch tinted with each theme's flavor so the user can recognize them at a glance.
        ContextMenuKind::ThemeSelector => &[
            (
                ids::CTX_MENU_THEME_FORGE,
                "Forge (dark)",
                Some([0xc8, 0x4b, 0xa0, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_PAINT,
                "Workshop (dark)",
                Some([0x4b, 0xa0, 0xc8, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_SUNSTONE,
                "Sunstone (light)",
                Some([0xf0, 0xc0, 0x4f, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_BLUEPRINT,
                "Blueprint (light)",
                Some([0x6c, 0x8e, 0xc8, 0xFF]),
            ),
            (ids::CTX_MENU_RADIUS_SHARP, "— Corners: Sharp", None),
            (ids::CTX_MENU_RADIUS_DEFAULT, "— Corners: Default", None),
            (ids::CTX_MENU_RADIUS_ROUND, "— Corners: Round", None),
            (ids::CTX_MENU_RAIL_SIZE_SMALL, "— Rail Buttons: Small", None),
            (
                ids::CTX_MENU_RAIL_SIZE_MEDIUM,
                "— Rail Buttons: Medium",
                None,
            ),
            (ids::CTX_MENU_RAIL_SIZE_LARGE, "— Rail Buttons: Large", None),
            (ids::CTX_MENU_MIRROR_UI, "— Mirror UI", None),
            (ids::CTX_MENU_SHOW_STATS, "— Show Statistics", None),
            // "Show Grid" removed — Grid Settings panel now owns the
            // grid visibility toggle (Display section "Show grid").
        ],
        ContextMenuKind::SaveMenu => &[
            (ids::CTX_MENU_SAVE, "Save \u{00b7} Cmd+S", None),
            (
                ids::CTX_MENU_SAVE_AS,
                "Save As\u{2026} \u{00b7} Cmd+Shift+S",
                None,
            ),
            (ids::CTX_MENU_EXPORT_SVG, "Export SVG\u{2026}", None),
        ],
        ContextMenuKind::OpenMenu => &[
            (
                ids::CTX_MENU_OPEN_PROJECT,
                "Open Project\u{2026} \u{00b7} Cmd+O",
                None,
            ),
            (
                ids::CTX_MENU_IMPORT,
                "Import\u{2026} \u{00b7} Cmd+Shift+I",
                None,
            ),
        ],
        // Settings cluster (gear) — TOP-LEVEL categories; each gets a `ChevronRight` from the row loop.
        ContextMenuKind::SettingsMenu => &[
            (ids::CTX_MENU_SETTINGS_PPM, "Pixels per meter", None),
            (ids::CTX_MENU_SETTINGS_UNIT, "Display unit", None),
            (ids::CTX_MENU_SETTINGS_FILTER, "Image filter", None),
            (ids::CTX_MENU_SETTINGS_DISPLAY, "Display", None),
            (ids::CTX_MENU_SETTINGS_TEXT, "Text rendering", None),
            (ids::CTX_MENU_SETTINGS_MOTION, "Motion", None),
            // ⚠️ **Esta entrada NÃO é uma categoria** — ela abre a janela flutuante do Input Map,
            // não um submenu. Fica aqui porque é a casa que o Godot lhe dá (*Project Settings >
            // Input Map*) e a equivalência era o pedido; as reticências dizem *"isto abre uma
            // janela"*, que é a convenção que toda a UI de desktop usa.
            // ⛔ **A NOTAR NO SMOKE:** o laço deste menu põe um `ChevronRight` em cada linha por
            // ser `SettingsMenu`, e um chevron promete um submenu que esta linha não tem. Se o Enio
            // o vir como errado, a cura é o laço perguntar pela LINHA e não pelo tipo do menu.
            (ids::CTX_MENU_SETTINGS_INPUT_MAP, "Input Map\u{2026}", None),
        ],
        // Pixels-per-meter submenu — 5 presets (retro 16 · Unity 32 · Godot 100 · HD 256 · 4K 1024).
        ContextMenuKind::SettingsPpmSubmenu => &[
            (ids::CTX_MENU_PPM_16, "16 (retro tile)", None),
            (ids::CTX_MENU_PPM_32, "32 (Unity 2D)", None),
            (ids::CTX_MENU_PPM_100, "100 (Godot)", None),
            (ids::CTX_MENU_PPM_256, "256 (HD 2D)", None),
            (ids::CTX_MENU_PPM_1024, "1024 (4K ref)", None),
        ],
        ContextMenuKind::SettingsUnitSubmenu => &[
            (ids::CTX_MENU_UNIT_METERS, "Meters", None),
            (ids::CTX_MENU_UNIT_PIXELS, "Pixels", None),
        ],
        // Image-filter submenu — the single global sampling mode
        // applied to every sprite/texture + the Vello preview.
        ContextMenuKind::SettingsFilterSubmenu => &[
            (ids::CTX_MENU_FILTER_PIXELART, "Pixel Art (crisp)", None),
            (ids::CTX_MENU_FILTER_SMOOTH, "Smooth (bilinear)", None),
        ],
        // Display submenu — runtime swap-chain present mode. VSync is
        // perfectly smooth; Immediate is non-blocking (no mouse-stutter)
        // at the cost of vsync-pacing.
        ContextMenuKind::SettingsDisplaySubmenu => &[
            (ids::CTX_MENU_DISPLAY_VSYNC, "VSync (smooth)", None),
            (
                ids::CTX_MENU_DISPLAY_IMMEDIATE,
                "Immediate (no stutter)",
                None,
            ),
        ],
        // Text rendering submenu — 4 presets, monotonic in aggressiveness: Default (historic) →
        // Crisp Light (boost 30/20/10 + snap-X) → Crisp (60/40/20) → Crisp Heavy (100/70/40).
        ContextMenuKind::SettingsTextSubmenu => &[
            (ids::CTX_MENU_TEXT_DEFAULT, "Default", None),
            (ids::CTX_MENU_TEXT_CRISP_HEAVY, "Crisp Heavy", None),
            (ids::CTX_MENU_TEXT_CRISP_HEAVY_PLUS, "Crisp Heavy +", None),
        ],
        // Motion submenu — o carácter da UI viva + o reduced motion.
        //
        // ⚠️ As duas primeiras linhas são um RÁDIO (o gosto) e a terceira é um TOGGLE (a garantia).
        // O bullet significa a mesma coisa nas três — *este é o estado corrente* — que é a
        // convenção de menu de plataforma, e é por isso que as três cabem numa tabela só.
        ContextMenuKind::SettingsMotionSubmenu => &[
            (ids::CTX_MENU_MOTION_EXPRESSIVE, "Expressive", None),
            (ids::CTX_MENU_MOTION_DISCRETE, "Discrete", None),
            (ids::CTX_MENU_MOTION_REDUCED, "Reduced motion", None),
        ],
        // The SceneList kind is rendered by its dedicated branch
        // below — `items` stays empty so the simple-row loop is
        // skipped.
        ContextMenuKind::SceneList => &[],
        // The palette-rename modal paints a TextInput + Rename button in its own branch below.
        ContextMenuKind::RenamePaletteDialog => &[],
        // The New-image modal paints its size/bg radios + Create in its own branch below.
        ContextMenuKind::NewImageDialog => &[],
        // O modal de resolução da folha desenha o próprio corpo, como os irmãos acima.
        ContextMenuKind::SheetSizeDialog => &[],
        // M14.6 F + M14.7: per-row Hierarchy actions. Order follows
        // the Unity / Godot / Blender convention: Rename first (the
        // most common edit), then additive ops (Duplicate, Add
        // Child), then the milder revert (Reset Transform), with
        // Delete last as the destructive endpoint.
        ContextMenuKind::HierarchyRow { .. } => &[
            (ids::CTX_MENU_HIER_RENAME, "Rename\u{2026}", None),
            (ids::CTX_MENU_HIER_DUPLICATE, "Duplicate", None),
            (ids::CTX_MENU_HIER_ADD_CHILD, "Add Child", None),
            // ⭐⭐⭐ **AGRUPAR / DESAGRUPAR** (Enio, 2026-08-30), e ficam AQUI de propósito: são a
            // forma mais **suave** de juntar a seleção num objeto, e o bloco abaixo é o das outras
            // duas — o Merge FUNDE os pixels e destrói os originais, o Pack ARRANJA-os numa folha.
            // Lidos em sequência, os três respondem *"quão junto?"* em ordem crescente de dano.
            //
            // ⚠️ O par fica junto porque **um verbo cujo inverso não se vê não se usa**.
            (ids::CTX_MENU_HIER_GROUP, "Group", None),
            (ids::CTX_MENU_HIER_UNGROUP, "Ungroup", None),
            (ids::CTX_MENU_HIER_MERGE_SPRITES, "Merge Sprites", None),
            // A mesma fusão, mas reversível: cada sprite fica numa camada do Painter. Vizinha da
            // de cima porque a escolha entre as duas só existe neste instante.
            (ids::CTX_MENU_HIER_MERGE_TO_LAYERS, "Merge to Layers", None),
            // Vizinho do Merge de propósito: os dois juntam a seleção num objeto. O Merge FUNDE
            // os pixels e destrói os originais; este ARRANJA-os e mantém cada peça viva e
            // editável dentro da folha. A ordem lê-se como "junte-os" → "quão junto?".
            (ids::CTX_MENU_HIER_PACK_SHEET, "Pack into Sheet", None),
            // Os três verbos da folha ficam juntos e nesta ordem — entrar, arrumar, sair —, que é
            // a ordem em que o artista os encontra. O do meio ESTEVE dentro do primeiro, e foi
            // por isso que ninguém o achou.
            (
                ids::CTX_MENU_HIER_ARRANGE_SHEET,
                "Auto-Arrange Pieces",
                None,
            ),
            // As duas SAÍDAS do bake, lado a lado e nesta ordem: assar muda a cena, exportar
            // escreve ficheiros. Ler uma a seguir à outra é o que torna a diferença óbvia.
            (ids::CTX_MENU_HIER_BAKE_SHEET, "Bake Sheet", None),
            (ids::CTX_MENU_HIER_EXPORT_SHEET, "Export Sheet", None),
            // A exportação de UMA sprite, ao lado da da folha: os dois escrevem ficheiros, e o
            // nome diz qual. Plano `docs/Sprite_projeto/18` W9 (Enio, 2026-08-21).
            (
                ids::CTX_MENU_HIER_EXPORT_IMAGE,
                "Export Image\u{2026}",
                None,
            ),
            (
                ids::CTX_MENU_HIER_REMOVE_FROM_SHEET,
                "Remove from Sheet",
                None,
            ),
            (
                ids::CTX_MENU_HIER_USE_AS_BRUSH_SHAPE,
                "Use as Brush Shape",
                None,
            ),
            (
                ids::CTX_MENU_HIER_USE_AS_BRUSH_TEXTURE,
                "Use as Brush Grain",
                None,
            ),
            (
                ids::CTX_MENU_HIER_USE_AS_PAPER,
                "Use as Watercolor Paper",
                None,
            ),
            (
                ids::CTX_MENU_HIER_USE_AS_GRANULATION,
                "Use as Granulation",
                None,
            ),
            (ids::CTX_MENU_HIER_RESET_TRANSFORM, "Reset Transform", None),
            // ⚠️ Numa linha que NÃO é instância ele responde com um aviso, e não com nada: a
            // tabela deste menu é plana (não sabe o que a linha é), e um item que come o clique
            // em silêncio é pior que um ausente.
            // ⭐ **A família da INSTÂNCIA** (ADR-0164 / F4.5), na ordem do gesto: criar a receita ·
            // pôr outra cópia · promover a excepção · devolvê-la · cortar o vínculo.
            // ⚠️ Todos respondem numa linha a que não se aplicam — a tabela é plana.
            (ids::CTX_MENU_HIER_MAKE_COMPONENT, "Make Component", None),
            (ids::CTX_MENU_HIER_INSTANTIATE, "Instantiate", None),
            (
                ids::CTX_MENU_HIER_INSTANTIATE_LINKED,
                "Instantiate Linked",
                None,
            ),
            (ids::CTX_MENU_HIER_APPLY_TO_MASTER, "Apply to Master", None),
            (
                ids::CTX_MENU_HIER_REVERT_TO_MASTER,
                "Revert to Master",
                None,
            ),
            (ids::CTX_MENU_HIER_DETACH, "Detach from Master", None),
            (ids::CTX_MENU_HIER_DELETE, "Delete", None),
        ],
        // Painter Falloff curve point handle (Blender per-point handle types).
        ContextMenuKind::FalloffPointHandle => &[
            (ids::CTX_MENU_FALLOFF_HANDLE_VECTOR, "Vector", None),
            (ids::CTX_MENU_FALLOFF_HANDLE_AUTO, "Auto", None),
        ],
        // On-canvas Curve / Free Hand point handle (the five vector-app continuity kinds).
        ContextMenuKind::CurvePointHandle => &[
            (ids::CTX_MENU_CURVE_HANDLE_FREE, "Free", None),
            (ids::CTX_MENU_CURVE_HANDLE_ALIGNED, "Aligned", None),
            (ids::CTX_MENU_CURVE_HANDLE_SYMMETRIC, "Symmetric", None),
            (ids::CTX_MENU_CURVE_HANDLE_VECTOR, "Vector", None),
            (ids::CTX_MENU_CURVE_HANDLE_AUTO, "Auto", None),
        ],
        // On-canvas motion-path anchor handle types (the vector Node trio, ADR-0141).
        ContextMenuKind::MotionPathAnchor { .. } => &[
            (ids::CTX_MENU_PATH_HANDLE_CORNER, "Corner", None),
            (ids::CTX_MENU_PATH_HANDLE_SMOOTH, "Smooth", None),
            (ids::CTX_MENU_PATH_HANDLE_SYMMETRIC, "Symmetric", None),
        ],
        // Timeline key: the interpolation leaving it, and the family submenu the
        // three cascade rows open. Both tables live in `ids` so the overlay, the
        // populate pass and the shell's resolver can never drift apart.
        // The SCOPE picks the table: a key gets the interpolation presets, a strip's fade
        // gets the easings alone (`TimelineInterpScope::menu_table` — the same door the
        // chrome handler asks, so painted rows and live rows cannot drift).
        ContextMenuKind::TimelineSegment { scope } => scope.menu_table(),
        ContextMenuKind::TimelineSegmentEase { .. } => &ids::TIMELINE_EASE_MENU,
        // Timeline track row (label column): whole-track actions.
        ContextMenuKind::TimelineTrack { .. } => &ids::TIMELINE_TRACK_MENU,
        ContextMenuKind::TimelineTrackAxis { .. } => &ids::TIMELINE_AXIS_TRACK_MENU,
        ContextMenuKind::TimelineTrackPath { .. } => &ids::TIMELINE_PATH_TRACK_MENU,
        ContextMenuKind::TimelineTrackTimeRemap { .. } => &ids::TIMELINE_TIMEREMAP_TRACK_MENU,
        // The four-mode extrapolation submenu (plan §6), opened by a track menu's
        // Pre/Post cascade row.
        ContextMenuKind::TimelineExtrap { .. } => &ids::TIMELINE_EXTRAP_MENU,
        // Timeline clip strip (a stack lane): what a pointer cannot say.
        ContextMenuKind::TimelineStrip { .. } => &ids::TIMELINE_STRIP_MENU,
        // Timeline stack lane (its label): how it blends, and whether it stays.
        ContextMenuKind::TimelineLane { .. } => &ids::TIMELINE_LANE_MENU,
        // Timeline marker pennant: its whole edit surface (ADR-0143).
        ContextMenuKind::TimelineMarker { .. } => &ids::TIMELINE_MARKER_MENU,
    }
}

/// **Os menus da barra de topo cujas rows são verbos SERVÍVEIS por id.**
///
/// A paleta de comandos global projecta as rows destes — e de mais nenhum. As três exclusões são
/// por MECANISMO, não por gosto, e cada uma tem um modo de falha diferente:
///
/// - **Menus parameterizados pelo alvo do clique** (`CreateNote { panel }`, `SectionOutline
///   { section }`, `NoteBackground`, `HierarchyRow`, os da timeline): o verbo precisa de um sujeito
///   que o CLIQUE forneceu. Servido de uma paleta, ele agiria sobre o que quer que o
///   `last_context_menu` ainda tivesse — um comando que faz alguma coisa, à coisa errada.
/// - **[`ContextMenuKind::SettingsMenu`]**: cada row dele ABRE um submenu, ancorado ao rectângulo
///   da row. Servida de uma paleta, ela abriria um menu numa posição obsoleta. As rows dos
///   SUBMENUS entram, porque essas são folhas.
/// - **Os que desenham o próprio corpo** (`SceneList`, `RenamePaletteDialog`, `NewImageDialog`):
///   [`menu_rows`] devolve `&[]` para eles, então já não contribuem nada.
///
/// ⚠️ A prova de que os que ENTRAM são context-free não é opinião: os `chrome/*.rs` que os tratam
/// só tocam o contexto para `close_context_menu()`, que é um no-op sem menu aberto. Há gate.
pub const TOPBAR_LEAF_MENUS: &[ContextMenuKind] = &[
    ContextMenuKind::SaveMenu,
    ContextMenuKind::OpenMenu,
    ContextMenuKind::ThemeSelector,
    ContextMenuKind::SettingsPpmSubmenu,
    ContextMenuKind::SettingsUnitSubmenu,
    ContextMenuKind::SettingsFilterSubmenu,
    ContextMenuKind::SettingsDisplaySubmenu,
    ContextMenuKind::SettingsTextSubmenu,
    ContextMenuKind::SettingsMotionSubmenu,
];

/// O nome humano do menu — o título do grupo na paleta.
///
/// ⚠️ Escrito à mão, e é a única coisa desta wave que é: os `ContextMenuKind` não carregam rótulo
/// (o pill que os abre é que tem um ícone), e derivá-lo do `Debug` daria *«SettingsPpmSubmenu»*.
#[must_use]
pub fn menu_title(kind: ContextMenuKind) -> Option<&'static str> {
    Some(match kind {
        ContextMenuKind::SaveMenu => "File",
        ContextMenuKind::OpenMenu => "Open",
        ContextMenuKind::ThemeSelector => "Look",
        ContextMenuKind::SettingsPpmSubmenu => "Pixels per meter",
        ContextMenuKind::SettingsUnitSubmenu => "Display unit",
        ContextMenuKind::SettingsFilterSubmenu => "Image filter",
        ContextMenuKind::SettingsDisplaySubmenu => "Display",
        ContextMenuKind::SettingsTextSubmenu => "Text rendering",
        ContextMenuKind::SettingsMotionSubmenu => "Motion",
        _ => return None,
    })
}
