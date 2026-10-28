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
- [x] FOTO `3d27f7ec45` (46 fotos, PDF 22 p.). DEFEITOS → lote D9 (depois do clippy): export sem aviso na aba; Convert to sticky note na aba sem Quadro activo descarta; edição rápida por baixo da lista DOING; textos cortados (Mirror card, Card aging, Convert to sticky…); pesquisa duplicada. Fotos 8.2/9.8 por cena (CODE_SMOKE=10, PLANNER_SMOKE=1); parte 6 por fotografar
- [x] fmt `91e1ddad42`; clippy `96ffa55c9b` + ronda 3 — LIMPO em tudo excepto app-kanban (7 erros → D9). [x] D9 `f1543d91f5` `5997209547` `0374a2e155` `4dcd892367` `7ac5cf14c2` `078c731a74` `61488e56c3` `a3d6f888d0` — CLIPPY LIMPO NA LINHA TODA. [x] FOTO2 `9bd14fa500` (tutorial sem «sem foto»). [x] handoff `500d9838cc`. [x] lock `ba017139ea`. MISSÃO FEITA — falta só a mensagem ao dono
- [x] J7 `9bba7079d4` `c3b7fa1ba5` (PROTOCOL 14, Join do Quadro aceita ph2d-kanban://, retrocesso único, presença no Quadro, versões com ids certos, dep família→família cortada por ph2d_kanban_sync::port; foto cena30_aba perdida). R2: to_wire fallback + smoke_live_tests.rs 629>600. Portões UI/arquitectura: 25/25 depois de J7/P2 (reconfirmar no fecho); antes: J7 (app-board→app-kanban) e [x] P2 `97f371f9de` `29427ab3c6` (modelo v7, portões verdes; falta teste do lado Kanban de `request_open_card` no frame.rs). [x] Tutorial `e11d1eb99c` docs/TrelloClone/tutorial/01_kanban.pdf (18 p.) — 22 passos sem foto → lote FOTO depois do J7, depois regenerar PDF
- [x] J1 `d38bee1f6c` (+linhas por comitar em lib.rs/tab.rs/Cargo.toml/smoke.rs/tests main.rs/familias.rs → COMMIT CONJUNTO no fim de J2/J3; HEAD não compila até lá). Cena 7. J4: sessão do Quadro leva Kanban, retrato fora da janela (medir), link próprio, reconexão
- [x] J2 `9c088afccc` (cena 8). Por ligar (lote J5): botões do quadro no cabeçalho (`auto::buttons::paint_board_buttons`/`press_board`); teste `verso_recebe_ficheiros_largados…` vermelho (lib.rs::frame do J1 esvazia docs.notices). canvas.rs filtro auto aplicado pelo coordenador
- [x] J5 `ec4784635d` botões de quadro + teste notices (lib.rs e main.rs `mod j5_botoes;` por comitar → conjunto). Revisor: 8 defeitos → R1 (frac, desfazer vazio, agenda religada, MonthDay inválido, spec unwrap, sort O(n²)) e J4 (Lamport u64::MAX, pending sem tecto)
- [x] J3 `efd1e1ac7d` `55ffc8a654` `3d2401eed4` (ph2d-kanban-hooks, scripts, ph2d-kanban-scenes, CODE_SMOKE=10, Import/Export). [x] R1 `9b23dfbd98`…`37b83a8c1e`. [x] J6 `02039c9c7a` (verso/botões pela porta, import report em docs.rs, cena 9, fotos W10). [x] J4 `a1861d1992` `07051c5317` (sessão do Quadro leva Kanbans, retrato num fio, ph2d-kanban://, reconexão, MAX_STAMP, PENDING_MAX). Por fazer: is_link do Join do Quadro aceitar KANBAN_SCHEME (editor-core board_live.rs); avatares esmaecidos ao reconectar; Quadro convidado com retrocesso exponencial; presença Kanban dentro da sessão do Quadro; histórico de versões do Quadro guarda ids locais
- J2 (verso+Butler) · J3 (ganchos, scripts, cenas, import/export, serve). [x] X1 `66fc6afca1`. Clippy crates estáveis LIMPO (`84e4debf44`, `6230d6c770`)
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
- [x] W6 lote G `8d9871cba8` (cenas 5–6). DEFEITO: painel do filtro corta o topo (título/fechar/palavra) → J1. Falta teste no elemento do Quadro; limite 6 sugestões @/# por medir
- LIGAÇÃO (app-kanban) em 3 lotes: J1 cabeçalho (seletor de vistas W7, Share/avatares/presença W5, filtro com cabeçalho fixo) · J2 verso+Butler (Scene object W11, botões de cartão, painel de automação, put_file_keyed, datepick) · J3 ganchos (react/run_scheduled/scripts, import/export W10, serve(hero), Convert to sticky note, crate ph2d-kanban-scenes + CODE_SMOKE 10)
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
