# HANDOFF de continuação — line/BarraDeMenus — onda 3 feita (12/11/2026)

Worktree `Worktrees/line-BarraDeMenus` · branch `line/BarraDeMenus` · base (merge-base com `main`) `4f46a3573c`.
Commit da onda: `b04e839881` (produto + testes + fotos + script). Onda 2 (aprovada no smoke do dono, 12/11):
[HANDOFF_..._ONDA2.md](HANDOFF_CONTINUACAO_line_BarraDeMenus_2026-11-12_ONDA2.md).
Plano: [01_plano.md](../01_plano.md). Memória: `project-memory/project_menu_bar_version_c_2026_11_12.md`.

## Onda 3 — FEITO
- **Decisão do dono (item 6 da colagem, que veio em branco):** o nome do painel na busca = o NOME DA ABA, lido de
  `manifest.title` (`Panel::TITLE`, TextKey): «Wet Paint», «Grid & Background», «Keyboard Shortcuts».
  `humanise_panel_id` saiu. Teste `global_palette_catalog::a_panel_is_called_in_the_search_what_its_tab_says`
  (registry-init). Os nomes antigos NÃO ficam como keywords (`PaletteItem.keywords` é `&'static str`).
- **Busca vazia: menus primeiro** (`build_global_model`: `bar.extend(groups)`), depois Tools, barras, Document, Painéis,
  depois os achados novos. Teste `na_busca_vazia_os_menus_vem_primeiro`.
- **Achados** (`screens/hero/global_palette_finds.rs`):
  - Ações do Input Map (`hero.input_map`), com as teclas na coluna do atalho (`Key::label`, `label_key` do pad);
    id = `hash("palette.input_action") ^ (ActionId+1)`; escolher → `store.request_input_map_action` →
    `ph2d-app-input-map::frame` põe `tab.doc.kb.open`.
  - Objetos da cena: `store.scene_objects()` publicado por `ph2d-panel-hierarchy::sync_from_hierarchy` (linhas com
    entidade, ordem da lista); escolher → bus `HierRequest::SelectRow Replace` + `flash_anywhere`.
  - Campos dos cartões: `CardDirectory` gravado ao pintar — `panel/cards_cartao.rs` (`card_painting`/`card_painted`,
    só `panel == ids::INSP_PANEL` e cartões com título), `panel/cards_linhas.rs::row_inteira`
    (`card_field_painted(label, Widget::target(), topo)`), origem em `cards_lista.rs` (`card_list_origin`); contador de
    quadro `begin_card_directory_frame` no começo de `paint_hero_screen`. Escolher → centra a lista do Inspector se
    fora de vista, `flash_anywhere`, `set_focus`. Cartão dobrado (sem campos pintados) → entrada pelo título; escolher
    desdobra + pisca. Anel genérico para o que não tem pintor de flash: `store.ring_anywhere`, pintado no hero antes
    da paleta de comandos.
- **File ▸ Open Recent**: linha `MENUBAR_FILE_RECENT_MENU` (depois de Open Project), `ContextMenuKind::MenuBarFileRecent`
  (linhas estáticas `&[]`; dinâmicas por `store.recent_rows()` encadeado em `doc_menu_rows`), `CASCADES` 13, `OWN_ROWS` 21.
  `screens/hero/open_recent.rs` («nome · ~/pasta», «No Recent Projects» cinza, «Clear Recent»); clique →
  `file_menu.open_path` (`FileMenuRequests` perdeu `Copy`); shell `project_io.rs::project_open_recent` (ficheiro em falta
  → sai da lista + toast `chrome.menu.recent_missing`); `note_recent_project` em open/save com sucesso. Os projetos
  recentes também aparecem na busca em File · Open Recent.
- **Memória por utilizador** `screens/hero/menu_memory.rs` (`MenuMemory`, `~/.ph2d/menu-memory.txt`, chave=valor:
  `recent=hex`, `pick=hex n`, `learned=hex`, `project=path`; sem número de schema, como o `prefs.txt`). Carregada em
  shell `init.rs` (`load_home`); gravada por quadro se suja em `global_palette_input.rs` (`save_home_if_changed`).
  O Recent da busca saiu do `PaletteSession` para cá.
- **Chip que ensina o atalho** `screens/hero/menu_teach.rs`: contado na porta única `menu_bar::close_on_row_click`
  (clique ou Enter numa linha de cascata); na 3.ª escolha (`TEACH_AT`), se o rótulo tem acorde depois de « · » e não
  está aprendido → `store.set_teach_chip` (um de cada vez, o novo substitui). Pintado à direita do campo Find na faixa
  (só se o texto cabe inteiro); ids `MENUBAR_TEACH` / `MENUBAR_TEACH_CLOSE` (dica «Got it · I won't show this again»).
  Ouvinte de tecla no TOPO de `input_dispatch/keyboard.rs::key_input` com `winit_to_input_keycode` (o keymap do editor
  perde o Z — a 1.ª corrida de fotos mostrou Ctrl+Z sem fechar o chip; curado). × ou a tecla → aprendido para sempre.
- **Alt+Enter «mostrar onde mora»**: shell `keyboard_palette.rs` (Enter com alt) → `command_palette_show_where` (só a
  paleta global em lista) → `screens/hero/menu_reveal.rs::show`: `path_to` percorre `MenuBarRoot` por `cascade_of` +
  Preferências/Tema; abre a raiz (`toggle_root`); `menu_reveal::tick` (em `tick_motion`) clica uma linha de cascata por
  quadro assim que o rect dela está pintado e por fim `set_hot` + `flash_anywhere` na linha. Fora dos menus →
  `flash_anywhere`. Rodapé «Alt+Enter to show where it lives».

## Fotos — `bash docs/BarraDeMenus/ferramentas/fotos_onda3.sh`
Xvfb, HOME falso semeado com dois projetos recentes, `PH2D_GAME_SMOKE=2` → `docs/BarraDeMenus/fotos/onda3_*.png` +
`onda3_app.log` (0 panics) + `onda3_menu-memory.txt`. Todas conferidas: busca vazia com menus primeiro; «jump» = Input
Map com «Up Arrow, Z» + campos Jump Height/Air Jumps do Inspector; objeto player; «air j» + Enter rola o Inspector e
foca Air Jumps com anel; Alt+Enter em «export svg» abre Menu▸File▸Export e acende Export SVG…; Open Recent mostra
forest_le… · ~/Games/Forest, cave_boss, Clear Recent; chip «Ctrl+Z is Undo ×» após 3 Edit▸Undo pelo menu, some com
Ctrl+Z (aprendido gravado).

## Testes (verdes por nome)
editor-core lib 231 (menu_teach, open_recent, menu_memory, menu_reveal, global_palette*, menu_bar, menu_cascade,
menu_faixa, command_palette, context_menu, cards, hover, tooltip, barras_cartoes) · editor-core it 47
(`architecture_*` incl. `the_shell_only_shrinks` e tectos de LOC, `every_menu_row*`, hr15, `no_literal_color`,
`the_bar_relocated`, `the_file_menu_items`, `the_menu_bar_relocates`) · registry-init 32 · host-desktop 34
(`um_modal_aberto`, keyboard, alt, project_io, palette) · panel-hierarchy 8.
`every_global_palette_item_is_dispatched_by_somebody` agora usa `route_global_pick` (a porta real) em vez de
`apply_event(Click)`. fmt feito.

## Aberto / decisões
- Nomes de projeto longos são elididos no submenu Open Recent (`MENU_W` fixo; a coluna da pasta leva espaço).
- Escolher uma ação do Input Map abre-a na vista Keyboard (`kb.open`), mas não troca a vista se a aba mostra List/Try.
- Campos do cartão Object (sem título) não são indexados; campos de painéis fora do Inspector não são, por desenho.
- Digitar direto no campo da faixa continua por fazer (desde a onda 2). Módulo `layout_tabs` morto (desde a onda 1).
- SEM provas de mutação das ondas 2 e 3; clippy/fecho (`PH2D_SUITE=fecho`) por correr.

## PRÓXIMO
Smoke do dono da onda 3. Depois onda 4: faixa única sem a barra do SO, testada no KWin.
