# HANDOFF de continuação — line/BarraDeMenus — onda 1 feita (12/11/2026)

Worktree `Worktrees/line-BarraDeMenus` · branch `line/BarraDeMenus` · base (merge-base com `main`) `4f46a3573c`.
Commits: `ba11e223ec` (onda 1, produto) · `c8f16e99b7` (testes adaptados) · eventual commit de fotos depois.
Plano: [01_plano.md](../01_plano.md). Memória: `project-memory/project_menu_bar_version_c_2026_11_12.md`.
Antes de ler: `cd` + `pwd` + `git branch --show-current`.

## Decisões do dono (12/11)
Versão C da crítica; **revoga a D2** (menus à vista); layouts Scene·Draw·… SAEM da faixa; lições fora da busca;
tirar a barra do sistema = decisão técnica delegada, **vai por último**, testada no KWin antes.

## Onda 1 — FEITO
- Título da janela = `"<projeto> — PH2D"` (`shells/desktop/src/render_loop/present_title.rs`; removido o campo morto `atlas_is_real`).
- Menu **Help** (Commands… abre a paleta Ctrl+K; Developer ▸ com os 4 painéis de dev).
- Submenus por `menu_bar::CASCADES` + `cascade_of` + `is_bar_menu` + `leaf_rows`; divisores por `GROUP_BREAKS` (pintados em `context_menu_overlay`).
- Coluna de atalho derivada do rótulo: `command_palette::shortcut_column/split_shortcut`. Chevron só em cascatas + SettingsMenu.
- Pixel: linhas do File **cinzentas, não escondidas** (`menu_row_is_unavailable`); `close_on_row_click` ignora linha indisponível.
- `DocMenu::File` virou New/Import/Export/Plan/Audio (kanban/planner/sfxlab reapontados). Input Map saiu de Settings para Window (mesmo id `CTX_MENU_SETTINGS_INPUT_MAP`).
- Abas de layout não são mais pintadas (o módulo `layout_tabs` ainda existe: candidato a remoção/censo de código morto).
- Ids novos em `ids/menubar.rs`: `MENUBAR_HELP`, `MENUBAR_FILE_NEW_MENU/IMPORT_MENU/EXPORT_MENU`, `MENUBAR_WINDOW_AUDIO_MENU/PIXEL_MENU/PLAN_MENU`, `MENUBAR_HELP_COMMANDS`, `MENUBAR_HELP_DEVELOPER_MENU`.
- `ContextMenuKind` novos: `MenuBarHelp`, `MenuBarFileNew`, `MenuBarFileImport`, `MenuBarFileExport`, `MenuBarWindowAudio`, `MenuBarWindowPixel`, `MenuBarWindowPlan`, `MenuBarHelpDeveloper`.
- i18n (`ph2d-i18n/src/chrome_menus.rs`): `chrome.menu_bar.help`, `chrome.menu.{new,import,export,audio,pixel,plan,developer}_submenu`, `chrome.menu.commands_cmd_k`.

## Testes adaptados (verdes por nome)
`the_menu_bar_relocates_the_verbs_it_shows` (WINDOW_VERBS; `module_rows` sobre folhas de Window+Developer) · contagens em `barras_cartoes_tests` ·
`the_window_menu_reaches_every_module` · `the_pixel_windows_close_and_live_in_the_window_menu` · `the_pixel_export_answers_a_real_pointer` ·
`exportar.rs` (`o_menu_file_mostra_as_linhas_sempre_e_so_as_acende_em_draw`) · `the_tab_and_the_menu_call_a_panel_the_same_thing`.
⚠ O rustfmt rodou DEPOIS da última corrida: rerrodar estes por nome uma vez (`bash scripts/cargo-test-narrow.sh <crate> <nome>`).

## Limites conhecidos da onda 1
O submenu substitui o menu-pai (um só slot de menu de contexto) e abre ao clicar, não ao pairar. NÃO feitos: Open Recent, Keyboard Shortcuts, About
(precisam de armazém de preferências / diálogo: onda 3); Play com nome (onda 2); «Undo Move» com nome.

## Ainda NÃO rodado (fecho)
fmt/clippy das crates tocadas (`PH2D_SUITE=fecho`) · censo de i18n (achar o gate que confere que toda chave `chrome.*` é usada/inglês) ·
`the_shell_only_shrinks` (a shell encolheu) · censos-da-árvore-combinada · **smoke do Enio** (passos numerados, na UI de cartões).

## PRÓXIMO — onda 2 (conforme o plano)
Botão W rotulado «Menu» contendo os 5 menus, Alt abre · campo central «Find anything» abrindo a paleta · paleta como LISTA com caminho
(«File · Export») e atalho por linha, recentes no topo · Play com nome. Depois ondas 3 e 4. Barra do sistema por último, após teste no KWin.

## Arquivos-chave
`crates/ph2d-editor-core/src/screens/hero/`: `menu_bar.rs`, `menu_tables.rs`, `menu_rows.rs`, `context_menu_overlay.rs`, `chrome/menu_bar.rs`,
`document_tabs.rs`, `global_palette.rs` · `.../doc_surface.rs` · `widget/command_palette/` · `transport_bar.rs` ·
`shells/desktop/src/init.rs:89` (janela, decorações do SO).

## Fotos da onda 1 (commit 7de8ff4c28) — o que elas mostraram

Fotos em `docs/BarraDeMenus/fotos/onda1_*.png` (Xvfb + xdotool, método de `docs/CodeEditor/ferramentas/smoke/cena14/fotos_q3.sh`).
- ✅ grupos, coluna de atalhos, Pixel cinzento, Input Map em Window, Help, layouts fora.
- ✅ curado depois das fotos: «›» em Preferences… e Theme… (abrem submenu e não o levavam).
- ⛔ ABERTO (para a onda 2, que de qualquer forma põe os cinco menus em cascata dentro do W): o submenu
  **substitui** o pai (um só slot de menu no `WidgetStore`) e abre só no **clique**, não no hover. A cascata
  de verdade pede pilha de menus abertos (pai fica, filho ao lado, hover abre). Os testes de UI que abrem
  `File ▸ Export` com dois cliques (`abre_file_sub`, `abre_window_pixel`) terão de mudar junto.
- ⚠️ o «›» fica ~3–4 px acima da linha de base do texto e apagado (Text3) — alinhar ao centro óptico.
