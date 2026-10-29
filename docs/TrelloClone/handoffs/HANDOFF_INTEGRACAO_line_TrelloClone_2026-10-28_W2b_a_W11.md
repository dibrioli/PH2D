# Handoff de INTEGRAÇÃO — `line/TrelloClone` W2b → W11 + ligação J1–J6 (28/10/2026)

> ✅ **INTEGRADO NO MAIN em 29/10** (ordem do dono: «na sua vez integre sem pedir permissões», depois da UIUX), por `--ff-only` sobre `be35cb386a` (GameEngineFuncional + BlenderConstraints + UIUX). Smoke do dono aprovado 28/10. Ver §10.
>
> ⛔ **Suíte completa NÃO corrida na linha (dono 23/10) — é do integrador.** Só testes por NOME, clippy `--all-targets` e fmt (LIMPOS em toda a linha: `96ffa55c9b` `d871ad7f54` `61488e56c3` `91e1ddad42`), portões de UI/arquitectura 25/25 por nome depois do J7/P2.

Leitor: o integrador e a próxima LLM. Plano [`01_plano`](../01_plano.md) · ADR [0195](../../architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md) (provisório) · loop [`loop/`](../loop/) (`ESTADO`, `MEDICOES`, `BLOQUEIOS`, `DECISOES_SEM_O_DONO`) · antecessor [W2→W3](HANDOFF_CONTINUACAO_W2_para_W3_2026-10-28.md).

## 1. Base e HEAD

- Branch `line/TrelloClone` (`Worktrees/line-TrelloClone`) · base = merge-base `c3662cd8e4` (main em 28/10, 0 atrás; **leia o main no ficheiro**, `collision-surface.sh` pelo caminho absoluto).
- **HEAD de código `6fdc67ca7f`** (o último do smoke do dono 28/10; o commit deste handoff vem a seguir, só docs): 129 commits desde a base, 810 ficheiros, +96 480 / −860. Árvore LIMPA.

## 2. Commits por onda (hashes = os que fecham cada lote)

| Onda | O que entrou | Commits |
|---|---|---|
| W0–W1 | plano, `ph2d-kanban-model` + `FracKey`, `PROJECT_SCHEMA 193→194`, ADR-0195, maquete | `622f11da11` `b30313cebb` `86ab320490` `da655e8075` `a09c623648` `bf893b2729` |
| W2 | tokens `kanban-*`, `ph2d-app-kanban` (aba), seam tests, fecho/fidelidade | `86a83f07d1` `0deede112f` `c644b6d738` `e0cc013232` `a227acc576` |
| W2b | títulos de lista em MAIÚSC. 15 px, lista em voo | `b7b46b7555` |
| W3 | richdoc, kanban-md, kanban-dates (A) · widgets calendar/date_picker/rich_editor (B) · modelo v2 (C) · verso, frente, menção, ligações, anexos (D) · catraca/`wants_frame`/`ph2d_app_host::open` (E) | `f94679ce9c` `97de715bf7` `3e5dd5b05a` `ea80f10a4a` `6a80664997` |
| W4 | `ElementKind::Kanban` (Quadro v18, PROTOCOL 12), vista numa `Affine`, Show/Place on board, Open in tab, roda/IME (v19, PROTOCOL 13), cena 30 | `4f06569b3a` `1a59f46c7c` `13459e3837` `f29cef9136` |
| W5 | `ph2d-kanban-sync` (LAN+QUIC, loopback 2 processos), `set_session`, retrato em blocos | `258968e864` `7871d1c4a6` `d9e9b202b3` |
| W6 | desfazer (F), modelo v3 (modelos/cópia/espelhos), `ph2d-kanban-filter`, UI (filtro, `/`, `E`, atalhos), cenas 5–6 | `0efbd1ed18` `65f3ea9fed` `8d9871cba8` |
| W7 | `ph2d-kanban-views` (Tabela, Calendário mês/semana/Dia/Agenda, Linha do tempo, Painel) | `fb27d29542` `b4349d438c` |
| W8 | Butler `ph2d-kanban-auto` (modelo v4), `ph2d.kanban.*` Luau (`ph2d-kanban-script`) | `d8ed63cb74` `35373e3520` |
| W9 | agenda no `KanbanSet` v5, `ph2d-app-planner`, Inbox/My cards/sino/.ics; «To Kanban», séries recorrentes | `1fd42be44d` `b74e72502f` `97f371f9de` `29427ab3c6` |
| W10 | `ph2d-kanban-import` (Trello JSON, CSV, markdown); `export_csv` anti-injecção de fórmulas | `c1d6f02b14` `56b1bb9ce6` |
| W11 | `KanbanSet` v6 `ProjectLink`, `ph2d-kanban-links`, canal `ph2d_editor_core::kanban_links`, nota adesiva ↔ cartão | `53883fefd8` `246d58ecfe` `66fc6afca1` |
| Ligação J1–J6 | cabeçalho (vistas, Share, presença, filtro fixo), verso+Butler, `ph2d-kanban-hooks` (porta única do gesto), `ph2d-kanban-scenes` + `PH2D_CODE_SMOKE=10`, Import/Export no menu File, sala do Quadro leva Kanbans (`ph2d-kanban://`, reconexão) | `d38bee1f6c` `9c088afccc` `efd1e1ac7d` `07051c5317` `590f1c8ae6` `02039c9c7a` |
| R1 (revisor) | 8 defeitos: `FracKey`, desfazer vazio, agenda, religar, ordenar O(n log n) | `9b23dfbd98`…`37b83a8c1e` |
| Fecho/docs | clippy/fmt, ADR-0195 segue o construído, tutorial PDF, T1 (seam do pedido de abrir cartão; `@`/`#` sem tecto fixo) | `84e4debf44` `6230d6c770` `9bdce8b5bf` `e11d1eb99c` `aa75d50ab1` `00401bbc49` |
| J7 | `PROTOCOL` 14; Join do Quadro aceita `ph2d-kanban://`; retrocesso único `ph2d_board_net::{backoff,Redial}`; presença do Kanban na sessão do Quadro; versões do Quadro com ids traduzidos; dep família→família cortada por `ph2d_kanban_sync::port` (`app-board` já não depende de `app-kanban`); teste `j7_porta.rs` | `9bba7079d4` `c3b7fa1ba5` |
| R2 | `to_wire` de um Kanban fora da sala envia `KANBAN_WAITING`, nunca o id local; `smoke_live_tests` 629→<600 (cena 22 + link da internet no irmão) | `b770e99d04` `2913777e47` |
| P2 | modelo v7 (excepções das séries), `ph2d-app-planner` sem dep. de `app-kanban`; falta teste do lado Kanban de `request_open_card` (`frame.rs`) | `97f371f9de` `29427ab3c6` |
| T1 | tutorial: fotos de 14 passos sem foto, claro e escuro, por `foto_tutorial.rs` (`#[ignore]`, a placa); PDF regenerado | `3d27f7ec45` `a3d6f888d0` |
| D9 (defeitos das fotos) | aviso de um pedido servido (`tell`) em qualquer aba de família (editor-core); «Convert to sticky note» na aba converte no Quadro onde o Kanban está posto; botões da edição rápida opacos; pesquisa com uma só caixa; texto que explica quebra em linhas (nunca elide); `Sheet::fitted` | `f1543d91f5` `5997209547` `0374a2e155` `4dcd892367` `7ac5cf14c2` |
| Smoke do dono 28/10 | D10 escrever no verso/filtro/automação + menu «Aa»; D12 capa por upload; D11 fundo do quadro (`KanbanSet` v8, `PROTOCOL` 15) | `6739befe3b` `8ef6bdf648` |
| fmt/clippy | `cargo fmt` nas crates tocadas; clippy `--all-targets` pela raiz (código morto apagado, aliases, args agrupados), LIMPO na linha toda | `91e1ddad42` `96ffa55c9b` `d871ad7f54` `078c731a74` `61488e56c3` |

## 3. Crates NOVAS — 16

`ph2d-frac` (`FracKey`, partilhado com o Quadro) · `ph2d-kanban-model` (`KanbanSet`: registos, ops, `History`, vistas, agenda, `ProjectLink`) · `ph2d-richdoc` (documento rico) · `ph2d-kanban-md` (markdown↔RichDoc, `pulldown-cmark`) · `ph2d-kanban-dates` (`jiff`, `due_tone`) · `ph2d-kanban-filter` · `ph2d-kanban-auto` (Butler) · `ph2d-kanban-script` (`ph2d.kanban.*` Luau) · `ph2d-kanban-hooks` (**porta ÚNICA de escrita de um gesto**) · `ph2d-kanban-scenes` (cenas de smoke, `now` por argumento) · `ph2d-kanban-links` (Power-Ups cartão↔objecto/script/macro/trecho/Quadro) · `ph2d-kanban-import` (Trello JSON, export CSV/md) · `ph2d-kanban-sync` (ao vivo, LAN+QUIC; `port`) · `ph2d-kanban-views` (Tabela, Calendário, Linha do tempo, Painel) · `ph2d-app-kanban` (aba + vista no Quadro) · `ph2d-app-planner` (Calendário, Planner, Inbox, My cards, `.ics`).

`ph2d-app-registry-init`: +features `app-kanban`, `app-planner` e dois `reg.push(..::FAMILY)` (`crates/ph2d-app-registry-init/src/lib.rs`) — o integrador confere que o registo não colide com as linhas que também o editam.

## 4. Contratos e formatos (valores lidos do código no HEAD; ⛔ reconte na integração)

| Item | Base `c3662cd8e4` | Linha (HEAD) | Nota |
|---|---|---|---|
| `PROJECT_SCHEMA` (`shells/desktop/src/project_schema.rs:499`) | 193 | **194** | `kanbans` no fim do `ProjectFile` (`da655e8075`). O main está em 193 → **reconte**: `python3 scripts/schema-recount.py <valor no main>` (exige o argumento) |
| `KanbanSet` `FORMAT_VERSION` (`ph2d-kanban-model/src/set.rs:19`) | — (novo) | **8** | v2 C · v3 F · v4 U · v5 P · v6 K · v7 P2 (excepções das séries) · v8 D11 (`Background` do Kanban, `background.rs`; fixtures v7 e v8). Antigas leem-se por degrau com fixture |
| Quadro `FORMAT_VERSION` (`ph2d-board-model/src/board.rs:28`) | 17 | **19** | 18 = `ElementKind::Kanban` (bytes do 17 = os do 18 sem a variante); 19 = rolagem por lista na vista do elemento |
| `PROTOCOL` (`ph2d-board-sync/src/session.rs:27`) | 11 | **15** | 14 = J7 (`9bba7079d4`/`c3b7fa1ba5`); 15 = D11 (`8ef6bdf648`, o fundo viaja na sala). Outras linhas do Quadro/MiroClone também o sobem → CONTAR |
| Contratos §6 (`NodeOp`, `Tool`…) | — | intactos | nenhum gate `architecture_*_contract_surface` tocado |

## 5. Tokens, i18n, dependências

- **Tokens** (`crates/ph2d-tokens/src/`): `kanban.rs` (etiquetas, superfícies, sombras, prazos), `faint_row.rs`, `contrast.rs` (pares medidos pelo gate), `skin_body.rs`, `color.rs`, `derive.rs`. Renomeados na W2: `kanban-<hue>-normal`→`kanban-<hue>`, `kanban-header-btn-on-fg`→`kanban-btn-on-fg`, `kanban-shadow-flight-near`→`kanban-flight-near`.
- **i18n** (`crates/ph2d-i18n/src/`): `kanban*.rs` (back/front/auto/hooks/import/links/share), `kanban_views/*`, `board_kanban.rs`, `planner.rs`; editados `lib.rs`, `familias.rs`, `shell_media.rs` (pontos de colisão).
- **Dependências externas novas** (`Cargo.lock`; a conferir no `cargo deny`/`stack-audit`):

- `jiff` 0.2.38 (+`-core`/`-static`/`-tzdb`/`-tzdb-platform`; Unlicense OR MIT; `ph2d-kanban-dates`, `-views`, `-sync`; features `tz-system`, `tzdb-zoneinfo`, `tzdb-bundle-platform`) · `pulldown-cmark` 0.13.5 (+`unicase`; MIT; `ph2d-kanban-md`) · `sha2` 0.10 (já no lock; ids de ficheiro por conteúdo) · `parley` 0.11.1 (já no workspace; só o `Cargo.lock`). Conferir no `cargo deny`/`stack-audit`.

## 6. Superfície de colisão (ficheiros que outras linhas também tocam)

`Cargo.lock` · `ph2d-i18n/src/{lib,familias,shell_media}.rs` · `ph2d-tokens/src/{color,derive,lib,skin_body}.rs` · `ph2d-editor-core` (`documents.rs` `SharedKanbans`, `DocumentSurface::{family_commands,family_command,chrome_over,wants_frame}`, widgets `calendar`/`date_picker`/`rich_editor`, `kanban_links`, `board_live.rs`, avisos de aba) · `shells/desktop` (`project_schema.rs`, `project.rs`, `render_loop/fase_*`, `it/`) · `ph2d-board-{model,sync,net,render,edit,excalidraw,video,geom}` · `ph2d-app-{board,code,components,macros,host}` · `ph2d-script` + `runtime/luau` · `ph2d-panel-{macros,tokens,registry-init}` · `ph2d-text` · `ph2d-code-help`.

O que um merge pode partir:
- **Quadro** (MiroClone W24+): `FORMAT_VERSION`/`PROTOCOL`/`ElementKind` — a variante `Kanban` entra no enum; um segundo merge de variante exige renumerar os dois e as fixtures de bytes.
- **`PROJECT_SCHEMA`** e a escada `project_schema.rs`: o degrau 194 é por campo `kanbans` no fim; outra linha com 194 → recontar e re-ordenar o campo.
- **`ph2d-app-registry-init`**, `ph2d-panel-registry-init`, `i18n/lib.rs`: adições em linhas vizinhas (Mergiraf pode apagar uma remoção em array — confira por `git diff`).
- **`ph2d-app-board`**: já NÃO depende de `ph2d-app-kanban` (porta `ph2d_kanban_sync::port`, J7); `Cargo.lock` muda por isso.
- **`ph2d-board-net`** (`redial.rs`, retrocesso único) e `ph2d-board-sync` (`replica_kanban`, `session`): o MiroClone toca os mesmos ficheiros.
- **`ph2d-script`/luau**: registo `ph2d.kanban`; a linha CodeEditor mexe no mesmo registo e em `ph2d-code-help`.

## 7. Para o integrador

- **Catraca `the_shell_only_shrinks`** (`crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs:112`): `TETO_LOC = 189_041`, intacto na linha. Medido no HEAD (mesma contagem do teste, `.rs` sob `shells/desktop`): **189 026**; o teste passa por nome. Reconte na árvore combinada (integrar → `cargo fmt --all` → medir → escrever).
- **ADR 0195 PROVISÓRIO.** Último no main: `0191`; há `0192..0194` noutras linhas por contar. Renumere pelo código: `git ls-tree main docs/architecture/decisions/`, depois `grep -rn "0195" docs crates` (o ADR, `decisions/README.md`, handoffs, doc-comments `[ADR-0195]`).
- **Cenas de smoke** (contadas nos roteadores): `PH2D_KANBAN_SMOKE=1..9` (`CENAS: u8 = 9`, `ph2d-app-kanban/src/smoke.rs:34`) · `PH2D_BOARD_SMOKE=30` (`max_level: 30`, `ph2d-app-board/src/smoke.rs:115`; o main chega a 29 → **recontar** se o MiroClone somou) · `PH2D_PLANNER_SMOKE=1` · `PH2D_CODE_SMOKE=10` (`CENAS: u32 = 10`, `ph2d-app-code/src/smoke.rs:11`; o main anuncia 1..9).
- **Suíte completa**: `nextest-impacted` + clippy `--all-targets` + `ship.sh` são do integrador (`PH2D_SUITE=fecho`).
- **Flakes de carga** (confirme com `loadavg`; [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)): timeline, traço de máscara, timeout do painter (fora do Kanban). Medições a `load` 28–33 são cota SUPERIOR.
- **Portões**: UI/arquitectura 25/25 verdes por nome depois do J7 (reconfirme na suíte). Nenhum vermelho conhecido.
- **Docs**: decisões de PRODUTO tomadas sem o dono (para rever no smoke): [`DECISOES_SEM_O_DONO.md`](../loop/DECISOES_SEM_O_DONO.md). Medições (`MAX_*` e tectos): [`MEDICOES.md`](../loop/MEDICOES.md).

## 7b. Smoke do dono 28/10 (`6739befe3b`, `8ef6bdf648`, `f5d9dbcde6`, `6fdc67ca7f`) — mecanismo em [SMOKE_DONO_2026-10-28_D10_a_D12.md](SMOKE_DONO_2026-10-28_D10_a_D12.md)

- **D10**: o teclado era engolido (o verso, o filtro e a automação respondiam «tomada» à tecla morta); a Mesa agora passa pela composição como a shell (`escrever_no_verso.rs`; mutação: 4+3+2 vermelhos). Menu «Aa» medido e por cima.
- **D12**: capa por upload («Upload a cover image»), Attachments com estado vazio, largar imagem = capa (`verso_capa.rs`).
- **D11**: fundo do quadro (cores, degradês, 8 padrões, imagem; `bg/`, `background.rs`, `kanban_bg.rs`; `d11_fundo.rs`, `d11_fotos.rs`); `KanbanSet` v8, `PROTOCOL` 15.
- **Vistas sobre o fundo** (`6fdc67ca7f`): as quatro vistas escrevem com a tinta das listas e eram pintadas sobre o fundo do quadro (ilegíveis num fundo claro); agora `header::other` pinta-as num painel `KanbanList` inset `BOARD_PAD` (o mesmo rect serve o rato); Tabela e Painel deixam de pintar `KanbanBoardBg`. Fotos `D11/vista_*`.
- Fecho depois do smoke: fmt + clippy `--all-targets -D warnings` limpos nas crates tocadas; censo de i18n do `ph2d-editor-core` verde (o `Debug` do `RichEditor` sem literal); `the_shell_only_shrinks` e `no_dependency_climbs_a_layer` verdes; `censos-da-arvore-combinada.sh` deu 120/121 antes de `f5d9dbcde6` (o único vermelho, o censo de i18n, re-corrido por nome e verde depois); ⛔ o integrador re-corre os censos inteiros.
- Merge pode partir: esses dois números; `ph2d-i18n/src/kanban{,_back}.rs`; `ph2d-board-render/src/lib.rs`; `export_json.rs`.

## 8. Prova de fecho e smokes

Verdes por nome (ver `ESTADO.md`): seam tests do hero (`ph2d-app-kanban/tests/it/`, `j7_porta.rs`), modelo (degraus v1→v7 com fixture), `ph2d-kanban-sync` `loopback::two_processes_converge_*` (127.0.0.1), gate das duas janelas (`2603d3023c`), tokens (contraste por par), catraca. Fotos `docs/TrelloClone/smoke/W2..W10/`. Tutorial do smoke final: `docs/TrelloClone/tutorial/01_kanban.pdf` (27 páginas, 12 partes; regenerado depois do D10–D12: parte 2 com descrição/«Aa»/capa e parte 12 «Fundo do quadro»); as fotos 8.2/9.8 e a parte 6 entraram em `9bd14fa500`; a foto `cena30_aba` perdeu-se no J7.

Smokes (o `cd` inteiro):
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TrelloClone && PH2D_KANBAN_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke` (trocar 1 por 2..9).
2. `cd … && PH2D_BOARD_SMOKE=30 cargo run -p ph2d-host-desktop --profile smoke` (Kanban no canvas do Quadro).
3. `cd … && PH2D_PLANNER_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`.
4. `cd … && PH2D_CODE_SMOKE=10 cargo run -p ph2d-host-desktop --profile smoke` (script `ph2d.kanban` + Ctrl+S).
Errado = tela vazia, cartão sem título, ou o quadro vazio ao abrir um projecto antigo (degrau de formato).

## 9. ABERTO

**Bloqueios** ([`BLOQUEIOS.md`](../loop/BLOQUEIOS.md)):
- W10: JSON REAL do Trello do dono (Menu → Export as JSON) vira fixture e gate; hoje só fixture sintética.
- W5: furo de NAT entre DUAS máquinas reais — provado só em 127.0.0.1 (LAN e QUIC sem ponto de encontro); mesma pendência do Quadro (ADR-0187).

**Pendências** (resto da ligação):
- J4: avatares esmaecidos ao reconectar. (Join `ph2d-kanban://`, retrocesso, presença e ids das versões: FEITOS no J7.)
- W8 L2: app correr scripts Kanban com handlers `on()` activos; UI do Butler completa.
- W11, fiação: `frame.rs` `serve(hero)`; menu do Quadro «Convert to card»; shell `publish_scene` + drenar `take(Scene/Timeline)`; app-code `take(Script)→CodeDocs::open`; Macros `take(Macro)`; secção «Scene object» no verso; nota nova do cartão no tamanho do texto.
- P2: teste do lado Kanban de `request_open_card` (`frame.rs`).
- W4: desfazer «inserir Kanban» deixa o documento vazio órfão (decidir); `board_nav.rs` = 700 linhas (no tecto). Menores: tracking das maiúsculas nos títulos; `back/dates.rs` → `datepick.rs` das views; gancho `front::cover::pending()`.
- Revisor final por fazer; `rm -rf target/*/incremental` e `agent-loop-profile.sh` ao fechar.
- D11: «Photos» lista só as imagens subidas, não as do projeto; a reserva do canvas sem a família Kanban pinta azul liso para fundo de imagem (nunca no app real). Ver [`BLOQUEIOS.md`](../loop/BLOQUEIOS.md).
- Smoke FINAL do dono: do PDF; [decisões sem o dono](../loop/DECISOES_SEM_O_DONO.md) por rever.

## Entrada §5.1 proposta (≤ 700 bytes; o integrador aplica)

- **Kanban (TrelloClone)** — o Kanban é um documento do projeto ([ADR-0195](docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md)): aba como no Trello (verso, vistas, fundo e padrões, Butler, `ph2d.kanban` Luau), o mesmo no Quadro, ao vivo, import do Trello, Planner; `ph2d-kanban-*` + `ph2d-app-kanban`/`-planner`. Smoke `PH2D_KANBAN_SMOKE=1..9` · `PH2D_BOARD_SMOKE=30`; tutorial PDF. Último: [W2b–W11, 28/10](docs/TrelloClone/handoffs/HANDOFF_INTEGRACAO_line_TrelloClone_2026-10-28_W2b_a_W11.md) · [plano](docs/TrelloClone/01_plano.md) · [docs](docs/TrelloClone/README.md)

## 10. A integração (29/10)

- **Antes do rebase** (suíte completa na linha, 28/10): 16 vermelhos desta linha, todos curados na linha (`15f03dbab4` `83d734fa7d` `e492145fd5` `890fb5ceb5` `213dc3b510` `1e9a99416f` `80e1764cc4`): tectos de LOC (`copy_move`, `choices`), o fundo de imagem pela porta que grava, conteúdo de cena declarado, oráculo do LSP com `ph2d.kanban`, biblioteca de macros relê o disco, Planner da cena no relógio injectado, widgets novos pelos censos da fundação, DAG da fundação.
- **Rebase** sobre `be35cb386a` (+401): 137 commits (o `3a266d3a70` caiu, já no upstream) + curas da combinação; conflitos pelos estágios (`Cargo.lock` só adições; `project*.rs` com `jogo`/`mesa` do main e `kanbans` por último; README dos ADR 0192–0195; registry-init regenerado). Comparação commit a commit (multiconjunto ±, controlo positivo): 126 idênticos, 11 = conflitos resolvidos.
- **Recontagem**: `PROJECT_SCHEMA` 207 → **208** (tripla `(208, 13, 25)`; `PROJECT_SCHEMA_SEM_KANBANS = 207`); PROTOCOL 15, board FORMAT 19, BOARD smoke 30, CODE smoke 10, `KanbanSet` 8 (o main não os tinha mexido); ADR 0195 livre.
- **Suíte combinada** (2 corridas, 20 211 testes): 1.ª — 3 vermelhos da combinação curados em `cd71958f24` (`board_kanban_notes` nomeia as fracções; `fase_snapshots_publish` 201 > 200 → `kanban_families`; a lista do `ph2d.` num sítio só, `api_table::ph2d_members`) + `schema-recount.py` lê os irmãos da tripla (`049da68afb`). 2.ª — 20 209/20 211: `a_cena_3_conduzida_passo_a_passo` = o defeito BIMODAL aberto do CodeEditor (FLAKES_DE_CARGA, «NÃO é carga»; 3/3 verde sozinho a load ~16) e `um_print_longo_…` = orçamento de relógio de 16,7 ms sob carga (sozinho verde) → promovido. Flakes promovidas (tabela + faixa isolada): `as_bordas_nao_esbranquicam_…`, `o_estado_inicial_leva_as_ferramentas_do_trilho`, `um_print_longo_…`.
- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` verde; censos da árvore combinada 122/122; `the_shell_only_shrinks`, `fn_loc_caps`, `workspace_src_files_under_loc_cap` verdes.
- `docs/PixelArt/` por rastrear no checkout primário (cópias dos docs da `line/PixelArt`, 2 com diferenças) — NÃO é desta linha; deixado intacto, e por isso o `foundational-integrate.sh` (que recusa primário sujo) foi corrido passo a passo.
- §5.1 do CLAUDE.md: entrada **Kanban (TrelloClone)** nova; Quadro passa a `PH2D_BOARD_SMOKE=1..30`.
