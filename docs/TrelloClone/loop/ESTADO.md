# ESTADO da missão autónoma W2b→W11 (coordenador relê depois de cada compactação)

- Worktree `Worktrees/line-TrelloClone` · branch `line/TrelloClone` · base = main `c3662cd8e4` (0 atrás, 28/10)
- Roteiro: `../handoffs/PROMPT_LOOP_AUTONOMO_W2b_a_W11.md` · plano `../01_plano.md` §6
- Hook de aprovação instalado em `.claude/settings.local.json` da worktree (git-ignorado)
- Último commit verde: `23c12ab9e1` (prompt da missão)

## Ondas
- [x] W2b — `b7b46b7555` (títulos 15px SemiBold MAIÚSC., lista em voo, cena 3). Pendente: tracking das maiúsculas (→ lote C)
- [ ] W3 — verso completo (+datas, capas, campos); widgets date picker/calendário/editor rico no editor-core; crates `ph2d-kanban-dates` (jiff), `ph2d-kanban-md` (pulldown-cmark)
- [ ] W4 — Kanban no canvas do Quadro (+Show on board/Place on board/Open in tab), LOD
- [ ] W6 — filtro/pesquisa/modelos/espelhos/copiar-mover/envelhecimento/E/atalhos (`ph2d-kanban-filter`)
- [ ] W5 — ao vivo (loopback 2 processos), Share/avatares, PROTOCOL 12
- [ ] W7 — vistas Tabela/Calendário/Linha do tempo/Painel
- [ ] W8 — automação + `ph2d.kanban.*` Luau (`ph2d-kanban-auto`)
- [ ] W9 — Calendário do projecto, Inbox, Planner, Os meus cartões, notificações
- [ ] W10 — importar Trello JSON (fixture sintética) / exportar CSV+md
- [ ] W11 — Power-Ups PH2D: cartão↔objecto/script/macro/timeline; nota adesiva↔cartão
- [ ] FIM — fecho da linha, handoff de integração, smoke final ao dono (PDF)

## Agentes em voo
- [x] W8 lote L `35373e3520` ph2d-kanban-script (ph2d.kanban.*). Falta: app-kanban chamar run/react + Butler após cada gesto; UI do Butler
- [ ] W8 L2 (adiado p/ ligação): mover short_film/sky_png p/ crate baixa (ph2d-kanban-scenes, now por argumento); app correr scripts Kanban (Run/Ctrl+S → KanbanScripts::run; handlers on() activos); depois cena CODE_SMOKE=10 + foto
- [x] W7 lote W `fb27d29542` `3be4ca777f` `7525ed9173` `b4349d438c` ph2d-kanban-views (+Dia/Agenda/semana horas). Falta: seletor de vistas no app-kanban; back/dates.rs → usar datepick.rs da views (uma lei)
- [x] W11 lote K `53883fefd8` `246d58ecfe` (modelo v6 ProjectLink, ph2d-kanban-links, notas↔cartões). Fiação por fazer: frame.rs serve(hero); menu do Quadro «Convert to card»; «Convert to sticky note» no cartão; shell publish_scene + drenar take(Scene/Timeline); app-code take(Script)→CodeDocs::open; Macros take(Macro); secção «Scene object» no verso; nota nova do cartão no tamanho do texto
- [x] W3 lote C `3e5dd5b05a` modelo v2 · `6b4ff2373e` tracking no ph2d-text (falta ligar paint_text.rs/text_elide.rs + list_title → D2). ADR-0195 desactualizado (v1, ImageFile) → documentador
- [x] W3 lote D1 `251b2d2652` `59d6c6828b` verso + cena 4
- [x] W3 lote D3 `da79404217` ligações D2 · `ea80f10a4a` anexos/links/copy/cliques
- [ ] W3 fecho (lote E): catraca da shell 189086>189041 (MOVER); gancho «quer outro quadro» p/ front::cover::pending(); medir tecto de anexo; xdg-open duplicado → ph2d-app-host; fmt+clippy W3; revisor
- [x] W4 lote V `1a59f46c7c` `2603d3023c` `13459e3837` (vista Affine, ganchos, Show/Place on board, menu, duas janelas, cena Quadro 30). app-board→app-kanban dep
- [x] W4 lote V2 `e8eb96649e` `4cbbbb532e` `f29cef9136` (roda, IME, cursor, scroll_y; board FORMAT 19, PROTOCOL 13). W4 FEITA (falta fecho batched: fmt/clippy/revisor)
- W6 lote G (UI): filtro, pesquisa, E, copiar/mover, modelos, espelhos, envelhecimento, atalhos — app-kanban
- F também cura: 2 desfazer seguidos no mesmo cartão (History::rebase × meta.version)
- [x] W4 lote M `4f06569b3a` `36d3fd8c1e` (ElementKind::Kanban, FORMAT 18, PROTOCOL 12, ganchos KanbanPainter/KanbanElementInput, botão Kanban, Open in tab na barra)
  - pendente p/ lote V: ContextMenuKind::BoardKanban (types_menu.rs+menu_rows.rs); key_of→Key::from; paint.rs usar kanban_board_bg/kanban_list_colors; MEDIR limiar LOD; foto; desfazer inserir deixa doc vazio órfão (decidir: apagar se intocado); board_nav.rs=700
- [x] W10 lote I `c1d6f02b14` ph2d-kanban-import (UI File→Import/Export por fazer). Cargo.lock por comitar quando D3 acabar
- [x] W5 lote S `258968e864` ph2d-kanban-sync (LAN+QUIC loopback convergem; desfazer por campo). UI Share/avatares por fazer
- [x] W5 lote S2 `7871d1c4a6` `d9e9b202b3` (set_session, reacções conjunto, retrato em blocos 5,2× menor, put_file_keyed). UI W5 por fazer: Share/avatares/cursor, arm/disarm, retrato do anfitrião num fio, chamar put_file_keyed no back/attach
- [x] W9 lote P `1fd42be44d` modelo v5+agenda · `b74e72502f` ph2d-app-planner (HEAD só compila com o calendário do lote W). Falta: Inbox→escolher Kanban/lista; Dia/Agenda/semana com horas (pedido ao W); sessão partilhar AGENDA (→ UI W5)
- [x] W6 lote F `50426ea408` desfazer · `0efbd1ed18` modelo v3 · `65f3ea9fed` ph2d-kanban-filter (UI da W6 por fazer)
- [x] W8 lote U `d8ed63cb74` ph2d-kanban-auto + modelo v4 (UI, i18n kanban.auto.*, Luau por fazer; react NÃO ao desfazer; run_scheduled fora do desfazer). Lock por comitar
- [x] W3 lote E `2403db62e1` catraca 189023≤189041 · `a0a562322b` wants_frame · `5961b5f80a` anexos grandes · `6a80664997` ph2d_app_host::open (V2 liga wants_frame e tira dep board-video→app-host; S2 faz put_file_keyed)
- [x] W3 lote A `f94679ce9c` richdoc · `ba73d150f5` kanban-md · `4771ddc2a5` kanban-dates
- [x] W3 lote D2 `b011d604b1` menção · `3033008f8f` tracking · `553bf822e3` frente · `d31a89fb33` fotos (4 ligações → D3)
- [x] W3 lote B `97de715bf7` (calendar, date_picker, rich_editor; Cargo.lock por comitar com o lote A)
- Plano W3: `W3_plano.md` — depois: lote C (modelo v2) quando W2b acabar; lote D (verso na app-kanban)

## Próxima cena de smoke: 3 (conferir no smoke.rs)
