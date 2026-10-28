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
- [x] W3 lote C `3e5dd5b05a` modelo v2 · `6b4ff2373e` tracking no ph2d-text (falta ligar paint_text.rs/text_elide.rs + list_title → D2). ADR-0195 desactualizado (v1, ImageFile) → documentador
- [x] W3 lote D1 `251b2d2652` `59d6c6828b` verso + cena 4
- W3 lote D3 (pontas: anexo de ficheiro, abrir link, copy link, duplo clique) — back/**, surface, docs, pointer
- F também cura: 2 desfazer seguidos no mesmo cartão (History::rebase × meta.version)
- [x] W4 lote M `4f06569b3a` `36d3fd8c1e` (ElementKind::Kanban, FORMAT 18, PROTOCOL 12, ganchos KanbanPainter/KanbanElementInput, botão Kanban, Open in tab na barra)
  - pendente p/ lote V: ContextMenuKind::BoardKanban (types_menu.rs+menu_rows.rs); key_of→Key::from; paint.rs usar kanban_board_bg/kanban_list_colors; MEDIR limiar LOD; foto; desfazer inserir deixa doc vazio órfão (decidir: apagar se intocado); board_nav.rs=700
- W6 lote F (puro): ph2d-kanban-filter + ops do modelo (modelos, espelhos, copiar/mover)
- [x] W3 lote A `f94679ce9c` richdoc · `ba73d150f5` kanban-md · `4771ddc2a5` kanban-dates
- [x] W3 lote D2 `b011d604b1` menção · `3033008f8f` tracking · `553bf822e3` frente · `d31a89fb33` fotos (4 ligações → D3)
- [x] W3 lote B `97de715bf7` (calendar, date_picker, rich_editor; Cargo.lock por comitar com o lote A)
- Plano W3: `W3_plano.md` — depois: lote C (modelo v2) quando W2b acabar; lote D (verso na app-kanban)

## Próxima cena de smoke: 3 (conferir no smoke.rs)
