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
        // ⭐ **UMA família por aparência** (2026-09-04): o redesenho mostra os quatro presets
        //    DERIVADOS (Godot 4.6, `ph2d_tokens::Theme::MODERN`), o clássico os quatro de sempre.
        //    Misturá-los poria um tema tingido ao lado de um plano sem o artista saber que está a
        //    escolher entre dois sistemas. As linhas de baixo (cantos, trilho, espelho, estatísticas,
        //    repor) são as mesmas nas duas.
        ContextMenuKind::ThemeSelector if crate::paint::ui_is_redesign() => &[
            (
                ids::CTX_MENU_THEME_DARK,
                "Dark",
                Some([0x56, 0x9e, 0xff, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_GRAY,
                "Gray",
                Some([0x70, 0xba, 0xfa, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_LIGHT,
                "Light",
                Some([0x2e, 0x80, 0xff, 0xFF]),
            ),
            (
                ids::CTX_MENU_THEME_OLED,
                "Black (OLED)",
                Some([0x73, 0xbf, 0xff, 0xFF]),
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
            (ids::MENUBAR_VIEW_RESET_LAYOUT, "Reset Panel Layout", None),
        ],
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
            // ⭐ **Repor a arrumação** (Enio, 2026-08-30). ⚠️ Um VERBO no meio de estados: o `—` que
            // os outros levam marca *«isto é um sub-estado do Look»*, e este não é um estado.
            (ids::MENUBAR_VIEW_RESET_LAYOUT, "Reset Panel Layout", None),
            // "Show Grid" removed — Grid Settings panel now owns the
            // grid visibility toggle (Display section "Show grid").
        ],
        // ── A BARRA DE MENUS (D2, 2026-08-30) ────────────────────────────────────────────
        // ⭐⭐ **Quase toda linha aqui leva um id que JÁ EXISTIA**, e é essa a decisão: a barra
        // realoja verbos, não os constrói. O `Save` é o do `io_menu`; o `Vector` é o
        // `TOPBAR_VECTOR` que o pill levava, e o painel do vetor continua a ser quem o despacha.
        // ⇒ um verbo, um id, um handler — e nenhuma segunda tabela a divergir da primeira.
        ContextMenuKind::MenuBarFile => &[
            (
                ids::MENUBAR_FILE_NEW,
                "New Image\u{2026} \u{00b7} Cmd+N",
                None,
            ),
            (ids::MENUBAR_FILE_SCENES, "Scenes\u{2026}", None),
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
            (ids::CTX_MENU_SAVE, "Save \u{00b7} Cmd+S", None),
            (
                ids::CTX_MENU_SAVE_AS,
                "Save As\u{2026} \u{00b7} Cmd+Shift+S",
                None,
            ),
            // ⭐ **Export SVG…** (report do Enio, 2026-09-04: *«funções criadas por outros módulos
            // não aparecem na UI»*). ⚠️ Ele **existe desde 2026-09-02** — a `line/Vector`
            // acrescentou-o ao `SaveMenu`, que era o menu do pill `TOPBAR_SAVE`; esta barra tinha
            // substituído o pill dois dias antes, e o merge das duas linhas foi **limpo**: nenhuma
            // tocou na linha da outra, e o verbo ficou a existir num menu que já não tem botão.
            // *Duas linhas a mexer na mesma superfície fundem sem conflito e uma delas evapora.*
            // ⇒ o gate `the_bar_relocated_every_row_of_the_menus_it_replaced` apanha o próximo.
            (ids::CTX_MENU_EXPORT_SVG, "Export SVG\u{2026}", None),
        ],
        // ⚠️ `TOOL_UNDO`/`TOOL_REDO` são os ids do TRILHO, e é de propósito: o verbo é o mesmo, e
        // duplicá-lo daria dois botões a desfazer coisas diferentes no dia em que um deles fosse
        // esquecido. Quem despacha continua a ser o `chrome::rail_tools`.
        ContextMenuKind::MenuBarEdit => &[
            (ids::TOOL_UNDO, "Undo \u{00b7} Cmd+Z", None),
            (ids::TOOL_REDO, "Redo \u{00b7} Cmd+Shift+Z", None),
            (ids::MENUBAR_EDIT_PREFERENCES, "Preferences\u{2026}", None),
        ],
        // ⚠️ **Mirror UI / Show Statistics / Corners / Rail Buttons NÃO se repetem aqui** — eles
        // vivem no `ThemeSelector`, que esta linha abre como categoria. Uma entrada repetida em
        // dois menus é a tabela paralela outra vez, com o sintoma pior: os dois estados a
        // discordar à vista.
        ContextMenuKind::MenuBarView => &[
            (ids::RAIL_SHOW_HIERARCHY, "Hierarchy", None),
            (ids::RAIL_SHOW_INSPECTOR, "Inspector", None),
            (ids::MENUBAR_VIEW_RULERS, "Rulers", None),
            (ids::MENUBAR_VIEW_THEME, "Theme\u{2026}", None),
        ],
        // ⭐ **Os treze toggles de módulo.** Entre a retirada da barra de pills (2026-08-30) e
        // esta barra, o único caminho até eles era a tecla `F9` — que é um interruptor de
        // bissecção, não uma porta de produto.
        ContextMenuKind::MenuBarWindow => &[
            (ids::TOPBAR_VECTOR, "Vector", None),
            (ids::TOPBAR_MOTION, "Motion Nodes", None),
            (ids::TOPBAR_FLIP, "Flip", None),
            (ids::TOPBAR_PHYSICS, "Physics", None),
            (ids::TOPBAR_SCULPT3D, "Sculpt 3D", None),
            (ids::TOPBAR_MODEL3D, "Model 3D", None),
            (ids::TOPBAR_IMAGE_TOOLS, "Image Tools", None),
            (ids::TOPBAR_AUDIO_MIXER, "Audio Mixer", None),
            (ids::TOPBAR_AUDIO_EDITOR, "Audio Editor", None),
            (ids::TOPBAR_TOKENS, "Design Tokens", None),
            (ids::TOPBAR_AUTHORED, "Authored UI", None),
            (ids::TOPBAR_WIDGET_GALLERY, "Widget Gallery", None),
            // ⭐ A BANCADA. ⚠️ Vizinha da galeria na lista porque é onde o leitor a procura, e
            // **separada dela** porque a galeria diz o que o editor É e esta diz o que ele pode
            // vir a ser (`ph2d-panel-widget-lab`, doc-comment do `lib.rs`).
            (ids::TOPBAR_WIDGET_LAB, "Widget Lab", None),
            (ids::TOPBAR_GRID_SETTINGS, "Grid Settings", None),
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
            (ids::TOPBAR_RIGHT_ASSETS, "Assets", None),
        ],
        // ⚠️ **O transporte é UM relógio** (`ph2d_core::Playhead`): física, Motion, Timeline e
        // Flip andam todos nele, e estes três verbos conduzem-nos de uma vez.
        // ⛔ *Rewind* estava **sem porta** desde a retirada dos pills — o `Espaço` alterna
        // tocar/pausar e as vírgulas andam quadro a quadro, mas nada rebobinava.
        ContextMenuKind::MenuBarRun => &[
            (ids::TOPBAR_PLAY_BUTTON, "Play \u{00b7} Space", None),
            (ids::TOPBAR_PAUSE, "Pause \u{00b7} Space", None),
            (ids::TOPBAR_RESET, "Rewind", None),
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
            (ids::CTX_MENU_SETTINGS_ANGLE, "Angle unit", None),
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
        // Angle-unit submenu — o irmão do de cima, para o ÂNGULO. O armazenamento
        // continua em radianos; isto só troca o FORMATO.
        ContextMenuKind::SettingsAngleSubmenu => &[
            (ids::CTX_MENU_ANGLE_DEGREES, "Degrees", None),
            (ids::CTX_MENU_ANGLE_RADIANS, "Radians", None),
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
        // ⭐ **Desenha o próprio corpo** (ver o doc desta função): os chips que não couberam na
        // fila, com os MESMOS ids — ver `context_menu_overlay::paint_tool_bar_overflow`.
        ContextMenuKind::ToolBarOverflow => &[],
        // ⭐⭐⭐ **As rows são DINÂMICAS** — quem as publica é o módulo que tem o canvas
        // (`WidgetStore::area_menus`), e o pintor acrescenta-as a estas. ⛔ Uma tabela estática
        // aqui teria de conhecer os ids de TODO módulo do app, que é o acoplamento que a **D2**
        // existe para não ter.
        //
        // ⚠️ E o `&[]` deixou de ser um caso especial: o pintor soma `estáticas + contribuídas`
        // para **todo** menu, e um pulldown de área é simplesmente aquele cuja metade estática é
        // vazia. É o mesmo mecanismo que põe *Export Draft* no `MenuBarFile` abaixo.
        ContextMenuKind::AreaCommands { .. } => &[],
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
            (ids::CTX_MENU_HIER_MAKE_COMPONENT, "Make Prefab", None),
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
            (
                ids::CTX_MENU_HIER_REMOVE_FROM_LIBRARY,
                "Remove from Library",
                None,
            ),
            (ids::CTX_MENU_HIER_DELETE, "Delete", None),
        ],
        // ⭐⭐ **O cartão da biblioteca** (plano 07, etapa C). Três itens, na ordem do gesto:
        // *usar* · *ver quem usa* · *tirar*.
        //
        // ⚠️ **Plana como a da Hierarquia, e pela mesma razão:** ela não sabe se a célula é um
        // Prefab ou uma Imagem. As duas famílias respondem aos três — e as recusas NOMEIAM o
        // motivo, que numa Imagem é sempre o mesmo facto: ela está na biblioteca porque um objecto
        // a usa, então *«tirar»* teria de tirá-la dos objectos, que é outro gesto.
        // ⭐⭐ A linha de catálogo. ⚠️ Só DOIS itens, e nenhum deles é *«criar»* — criar é o `+` do
        // cabeçalho da coluna, que não precisa de um sujeito.
        ContextMenuKind::CatalogRow { .. } => &[
            (ids::CTX_MENU_CATALOG_RENAME, "Rename\u{2026}", None),
            (ids::CTX_MENU_CATALOG_DELETE, "Delete", None),
        ],
        // ⭐⭐ O cartão da biblioteca: **três itens, na ordem do gesto** — usar · ver quem usa ·
        // tirar. ⚠️ **Este comentário voltou para cima da arm que ele descreve** (auditoria de
        // 2026-08-30): a arm do catálogo foi inserida no meio dele, e o leitor caía num «Três
        // itens» imediatamente acima de uma arm com DOIS, com a arm do cartão a ficar sem nota
        // nenhuma. *Um comentário separado do seu item muda de dono.*
        ContextMenuKind::AssetCard { .. } => &[
            // ⭐⭐⭐ **EDITAR vem PRIMEIRO** (report do Enio, 2026-09-05) — e a ordem é medida, não
            // estética: *Instantiate* tem uma segunda porta (o duplo-clique no cartão) e *Edit* não
            // tinha nenhuma. *O item que é o ÚNICO acesso ao seu verbo lê-se antes do que se
            // alcança de duas maneiras.*
            (ids::CTX_MENU_ASSET_EDIT, "Edit Prefab", None),
            (ids::CTX_MENU_ASSET_INSTANTIATE, "Instantiate", None),
            (ids::CTX_MENU_ASSET_SELECT_USERS, "Select users", None),
            // ⭐⭐ As duas metades de D9. ⚠️ Elas ficam DEPOIS do *Select users* de propósito: a
            // pergunta da cena vem antes da da biblioteca, que é a ordem em que o artista repara
            // que precisa da segunda.
            (ids::CTX_MENU_ASSET_USES, "Show what it uses", None),
            (ids::CTX_MENU_ASSET_USED_BY, "Show what uses it", None),
            (ids::CTX_MENU_ASSET_REMOVE, "Remove from Library", None),
            // ⭐⭐⭐ **A troca por um componente sem parentesco** (plano F5, o último critério).
            //
            // ⚠️ **Três linhas e não uma, porque o MODO é o gesto.** Sem antepassado comum não há
            // mapa derivado, só palpite — e o plano proíbe o app de o escolher sozinho (HR-5).
            // A linha sem adjectivo é o `None` do Unity e o caminho seguro; as duas de baixo usam
            // o prefixo `—` do selector de tema, que é como esta casa já escreve um sub-grupo.
            //
            // ⚠️ **O sujeito é a SELECÇÃO**, ao contrário de todas as linhas acima — daí o rótulo
            // a nomeá-la: um item que age sobre outra coisa que a apontada tem de o dizer.
            (
                ids::CTX_MENU_ASSET_REPLACE,
                "Replace selection with this",
                None,
            ),
            (
                ids::CTX_MENU_ASSET_REPLACE_BY_NAME,
                "\u{2014} and match overrides by name",
                None,
            ),
            (
                ids::CTX_MENU_ASSET_REPLACE_BY_TREE,
                "\u{2014} and match overrides by position",
                None,
            ),
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
/// ⭐⭐⭐ **Os menus do chrome LEGADO e o pill que os abria** — a tabela do que a barra de menus
/// teve de **realojar**.
///
/// Ela existe por um defeito medido (2026-09-04, report do Enio: *«funções criadas por outros
/// módulos não aparecem na UI»*, exemplo *Export SVG…*). A retirada dos pills (2026-08-30) tirou o
/// único botão que abria estes cinco menus; a barra prometeu levar as rows deles para *File* /
/// *Edit* / *View*, e a prova dessa promessa era **prosa numa lista de excepções** do gate
/// `every_topbar_verb_has_a_door_that_is_not_the_legacy_key` — *«as duas linhas do `SaveMenu` estão
/// no menu File»*. ⚠️ **Uma isenção que CONTA linhas envelhece no dia em que alguém acrescenta
/// uma:** a `line/Vector` juntou o *Export SVG…* ao `SaveMenu` em 2026-09-02, a frase passou a
/// descrever três linhas dizendo duas, e o verbo ficou sem porta **sem acordar gate nenhum** — o
/// censo daquele gate é sobre os **ids de pill declarados**, não sobre as **rows** que eles abriam.
///
/// ⇒ a promessa passa a ser DADO, e quem a mede é
/// `the_bar_relocated_every_row_of_the_menus_it_replaced`.
///
/// ⚠️ **Ela tem dois consumidores, de propósito:** o despacho
/// (`interaction::dispatch::pointer_down_menus::menu_opened_by`, que continua a abrir estes menus
/// quando a `F9` devolve o chrome legado) e o gate. *Uma lei escrita em dois sítios ainda não é uma
/// lei — só uma porta é.*
pub const LEGACY_PILL_MENUS: &[(NodeId, ContextMenuKind)] = &[
    (ids::TOPBAR_THEME, ContextMenuKind::ThemeSelector),
    (ids::TOPBAR_SAVE, ContextMenuKind::SaveMenu),
    (ids::TOPBAR_OPEN, ContextMenuKind::OpenMenu),
    (ids::TOPBAR_SETTINGS, ContextMenuKind::SettingsMenu),
    (ids::TOPBAR_PROJECT, ContextMenuKind::SceneList),
];

pub const TOPBAR_LEAF_MENUS: &[ContextMenuKind] = &[
    ContextMenuKind::SaveMenu,
    ContextMenuKind::OpenMenu,
    ContextMenuKind::ThemeSelector,
    ContextMenuKind::SettingsPpmSubmenu,
    ContextMenuKind::SettingsUnitSubmenu,
    ContextMenuKind::SettingsAngleSubmenu,
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
        ContextMenuKind::SettingsAngleSubmenu => "Angle unit",
        ContextMenuKind::SettingsFilterSubmenu => "Image filter",
        ContextMenuKind::SettingsDisplaySubmenu => "Display",
        ContextMenuKind::SettingsTextSubmenu => "Text rendering",
        ContextMenuKind::SettingsMotionSubmenu => "Motion",
        _ => return None,
    })
}
