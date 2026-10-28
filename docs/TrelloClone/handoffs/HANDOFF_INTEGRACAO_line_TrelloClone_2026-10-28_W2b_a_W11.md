# Handoff de INTEGRAÇÃO — `line/TrelloClone` W2b → W11 + ligação J1–J6 (28/10/2026)

Leitor: o agente integrador e a próxima LLM. Plano: [`../01_plano.md`](../01_plano.md) · ADR: [0195](../../architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md) (provisório) · loop: [`../loop/`](../loop/) (`ESTADO`, `MEDICOES`, `BLOQUEIOS`, `DECISOES_SEM_O_DONO`) · antecessor: [W2→W3](HANDOFF_CONTINUACAO_W2_para_W3_2026-10-28.md).

## 1. Base e HEAD

| | |
|---|---|
| Branch / worktree | `line/TrelloClone` · `Worktrees/line-TrelloClone` |
| Base (merge-base com o main) | `c3662cd8e4` (main nessa data; 0 atrás) |
| HEAD no momento de escrever | `00401bbc49` (99 commits desde a base; 683 ficheiros, +88 370 / −537) |
| Árvore **suja** (J7 em curso, ver §9) | 22 `M` + 3 `??` por comitar: `ph2d-app-board/src/live*.rs`, `ph2d-app-kanban/{header,share}`, `ph2d-board-net/src/{lib,redial}.rs`, `ph2d-board-sync/src/{replica_kanban,session}.rs`, `ph2d-kanban-sync/src/{lib,port}.rs`, `editor-core board_live.rs`, `tests/it/j7_porta.rs`, `Cargo.lock` |

⚠️ O integrador lê o merge-base e o `main` NO FICHEIRO (`collision-surface.sh` pelo caminho absoluto); a base acima é o que a linha mediu em 28/10.

## 2. Commits por onda (99; hashes = os que fecham cada lote)

| Onda | O que entrou | Commits |
|---|---|---|
| W0–W1 (pré) | pesquisa/plano, decisões do dono, `ph2d-kanban-model` + `FracKey`, `PROJECT_SCHEMA 193→194`, ADR-0195, maquete | `622f11da11` `b30313cebb` `86ab320490` `da655e8075` `a09c623648` `bf893b2729` |
| W2 | tokens `kanban-*`, `ph2d-app-kanban` (aba), seam tests, fecho/fidelidade | `86a83f07d1` `0deede112f` `c644b6d738` `e0cc013232` `a227acc576` |
| W2b | títulos de lista em MAIÚSC. 15 px, lista em voo | `b7b46b7555` |
| W3 | richdoc, kanban-md, kanban-dates (A) · widgets calendar/date_picker/rich_editor (B) · modelo v2 + tracking de letras (C) · verso do cartão + frente + menção + ligações + anexos (D1–D3) · catraca/wants_frame/anexos grandes/`ph2d_app_host::open` (E) | `f94679ce9c` `ba73d150f5` `4771ddc2a5` · `97de715bf7` · `3e5dd5b05a` `6b4ff2373e` · `251b2d2652` `59d6c6828b` `b011d604b1` `3033008f8f` `553bf822e3` `da79404217` `ea80f10a4a` · `2403db62e1` `a0a562322b` `5961b5f80a` `6a80664997` |
| W4 | `ElementKind::Kanban` no Quadro (Quadro v18, PROTOCOL 12), vista numa `Affine`, Show/Place on board, Open in tab, roda/IME/cursor (v19, PROTOCOL 13), cena 30 | `4f06569b3a` `36d3fd8c1e` `1a59f46c7c` `2603d3023c` `13459e3837` `e8eb96649e` `4cbbbb532e` `f29cef9136` |
| W5 | `ph2d-kanban-sync` (LAN+QUIC, loopback 2 processos), `set_session`, retrato em blocos | `258968e864` `7871d1c4a6` `d9e9b202b3` |
| W6 | desfazer (F), modelo v3 (modelos/cópia/espelhos), `ph2d-kanban-filter`, UI (filtro, `/`, `E`, atalhos), cenas 5–6 | `50426ea408` `0efbd1ed18` `65f3ea9fed` `c6db55359b` `8d9871cba8` |
| W7 | `ph2d-kanban-views` (Tabela, Calendário mês/semana/Dia/Agenda, Linha do tempo, Painel) | `fb27d29542` `3be4ca777f` `7525ed9173` `b4349d438c` |
| W8 | Butler `ph2d-kanban-auto` (modelo v4), `ph2d.kanban.*` Luau (`ph2d-kanban-script`) | `d8ed63cb74` `35373e3520` |
| W9 | agenda no `KanbanSet` v5, `ph2d-app-planner`, Inbox/My cards/sino/.ics; «To Kanban», séries recorrentes (v7); Planner sem dep. de app-kanban | `1fd42be44d` `b74e72502f` `97f371f9de` `29427ab3c6` |
| W10 | `ph2d-kanban-import` (Trello JSON, CSV, markdown); `export_csv` anti-injecção de fórmulas | `c1d6f02b14` `56b1bb9ce6` |
| W11 | `KanbanSet` v6 `ProjectLink`, `ph2d-kanban-links`, canal `ph2d_editor_core::kanban_links`, nota adesiva ↔ cartão | `53883fefd8` `246d58ecfe` `66fc6afca1` `b98af4d1a6` |
| Ligação J1–J6 | cabeçalho (vistas, Share, presença, filtro fixo), verso+Butler, `ph2d-kanban-hooks` (porta única do gesto), `ph2d-kanban-scenes` + `PH2D_CODE_SMOKE=10`, Import/Export no menu File, sala do Quadro leva Kanbans (`ph2d-kanban://`, reconexão) | `d38bee1f6c` `9c088afccc` `ec4784635d` `efd1e1ac7d` `55ffc8a654` `3d2401eed4` `07051c5317` `a1861d1992` `590f1c8ae6` `02039c9c7a` |
| R1 (revisor) | `FracKey` só válida, desfazer esvaziado salta, agenda nunca-dispara recusada, religar não dispara o passado, ordenar O(n log n) | `9b23dfbd98` `f2a1692477` `033d8da3eb` `d8fa198708` `df078a6f5b` `7fddaf74ac` `b89fb8d42c` |
| Fecho/docs | clippy/fmt, ADR-0195 segue o construído, tutorial PDF, T1 (seam do pedido de abrir cartão; sugestões `@`/`#` sem tecto fixo) | `84e4debf44` `6230d6c770` `9bdce8b5bf` `e11d1eb99c` `aa75d50ab1` `00401bbc49` |

## 3. Crates NOVAS — 16 (contagem por `git diff --name-status c3662cd8e4..HEAD -- 'crates/*/Cargo.toml'`, `A`)

| Crate | É |
|---|---|
| `ph2d-frac` | `FracKey` (índice fraccionário), partilhado pelo Quadro (z) e pelo Kanban |
| `ph2d-kanban-model` | o documento: `KanbanSet` (registos, ops, `History`, vistas, agenda, `ProjectLink`) |
| `ph2d-richdoc` | documento rico em blocos + marcas inline, edição pura (descrição/comentário) |
| `ph2d-kanban-md` | markdown ↔ `RichDoc` (`pulldown-cmark`) |
| `ph2d-kanban-dates` | «hoje/atrasado/esta semana» num fuso (`jiff`), `due_tone` |
| `ph2d-kanban-filter` | filtro, pesquisa `/`, ordenação, envelhecimento |
| `ph2d-kanban-auto` | Butler: regras, botões, agenda, executor determinístico |
| `ph2d-kanban-script` | `ph2d.kanban.*` (Luau) |
| `ph2d-kanban-hooks` | **a porta ÚNICA de escrita de um gesto** (alvos antes → aplicar → Butler/scripts reagem) |
| `ph2d-kanban-scenes` | cenas de smoke como modelo puro (`now` por argumento) |
| `ph2d-kanban-links` | Power-Ups: cartão ↔ objecto/script/macro/trecho/Quadro; nota ↔ cartão |
| `ph2d-kanban-import` | Trello JSON → Kanban; export Trello-JSON/CSV/md |
| `ph2d-kanban-sync` | Kanban ao vivo, mesma lei de fusão do Quadro (LAN + QUIC) |
| `ph2d-kanban-views` | Tabela, Calendário, Linha do tempo, Painel; `DatePicker` partilhado |
| `ph2d-app-kanban` | a família: a aba do Kanban + a vista do elemento no Quadro |
| `ph2d-app-planner` | família pessoal: Calendário do projecto, Planner, Inbox, My cards, `.ics` |

`ph2d-app-registry-init`: +features `app-kanban`, `app-planner` e dois `reg.push(..::FAMILY)` (`crates/ph2d-app-registry-init/src/lib.rs`) — o integrador confere que o registo não colide com as linhas que também o editam.

## 4. Contratos e formatos (valores lidos do código no HEAD; ⛔ reconte na integração)

| Item | Base `c3662cd8e4` | Linha (HEAD) | Nota |
|---|---|---|---|
| `PROJECT_SCHEMA` (`shells/desktop/src/project_schema.rs:499`) | 193 | **194** | `kanbans` no fim do `ProjectFile` (`da655e8075`). O main está em 193 → **reconte**: `python3 scripts/schema-recount.py <valor no main>` (exige o argumento) |
| `KanbanSet` `FORMAT_VERSION` (`ph2d-kanban-model/src/set.rs:19`) | — (novo) | **7** | v2 C · v3 F · v4 U · v5 P · v6 K · v7 P2b (excepções das séries). Antigas leem-se por degrau com fixture |
| Quadro `FORMAT_VERSION` (`ph2d-board-model/src/board.rs:28`) | 17 | **19** | 18 = `ElementKind::Kanban` (bytes do 17 = os do 18 sem a variante); 19 = rolagem por lista na vista do elemento |
| `PROTOCOL` (`ph2d-board-sync/src/session.rs:27`) | 11 | **13 comitado** | **14 na árvore suja (J7)**; só vale depois de comitado. Outras linhas do Quadro/MiroClone também o sobem → CONTAR |
| Contratos §6 (`NodeOp`, `Tool`…) | — | intactos | nenhum gate `architecture_*_contract_surface` tocado |

## 5. Tokens, i18n, dependências

- **Tokens** (`crates/ph2d-tokens/src/`): `kanban.rs` (etiquetas claro/escuro, superfícies, sombras, prazos; 62 cores da W2 + os de W3–W7), `faint_row.rs`, `contrast.rs` (pares medidos pelo gate), `skin_body.rs`, `color.rs`, `derive.rs`. Renomeados na W2: `kanban-<hue>-normal`→`kanban-<hue>`, `kanban-header-btn-on-fg`→`kanban-btn-on-fg`, `kanban-shadow-flight-near`→`kanban-flight-near`.
- **i18n** (`crates/ph2d-i18n/src/`): `kanban.rs`, `kanban_back.rs`, `kanban_front.rs`, `kanban_auto.rs`, `kanban_hooks.rs`, `kanban_import.rs`, `kanban_links.rs`, `kanban_share.rs`, `kanban_views/{calendar,dashboard,table,timeline}.rs`, `board_kanban.rs`, `planner.rs`; editados `lib.rs`, `familias.rs`, `shell_media.rs`. `lib.rs` e `familias.rs` são pontos de colisão (outras linhas acrescentam módulos).
- **Dependências externas novas** (`Cargo.lock`; a conferir no `cargo deny`/`stack-audit`):

| Crate | Versão | Licença | Onde |
|---|---|---|---|
| `jiff` (+`jiff-core`, `jiff-static`, `jiff-tzdb`, `jiff-tzdb-platform`) | 0.2.38 | Unlicense OR MIT | `ph2d-kanban-dates` (e `-views`/`-sync` com `std`); features `tz-system`, `tzdb-zoneinfo`, `tzdb-bundle-platform` |
| `pulldown-cmark` (+`unicase`) | 0.13.5 | MIT | `ph2d-kanban-md` |
| `defmt` 1.2.0 / `defmt-macros` / `defmt-parser` | — | MIT OR Apache-2.0 | transitiva (entrou no lock com o `jiff`); confirme a origem |
| `sha2` 0.10 | já no lock | MIT OR Apache-2.0 | `ph2d-kanban-model` (ids de ficheiro por conteúdo) |
| `parley` 0.11.1 | já no workspace | Apache-2.0 OR MIT | `ph2d-editor-core` (rich_editor); só o `Cargo.lock` mexeu |

## 6. Superfície de colisão (ficheiros que outras linhas também tocam)

`Cargo.lock` · `crates/ph2d-i18n/src/{lib,familias,shell_media}.rs` · `crates/ph2d-tokens/src/{color,derive,lib,skin_body}.rs` · `crates/ph2d-editor-core` (55 ficheiros: `documents.rs` `SharedKanbans`, `DocumentSurface::{family_commands,family_command,chrome_over,wants_frame}`, widgets `calendar`/`date_picker`/`rich_editor`, `kanban_links`, `board_live.rs`) · `shells/desktop` (18 ficheiros: `project_schema.rs`, `project.rs`, `render_loop/fase_*`, testes `it/`) · `crates/ph2d-board-{model,sync,net,render,edit,excalidraw,video,geom}` · `crates/ph2d-app-board` · `crates/ph2d-app-code` · `crates/ph2d-script` + `runtime/luau` (binding `ph2d.kanban`) · `crates/ph2d-app-{components,macros,host}` · `ph2d-panel-{macros,tokens,registry-init}` · `ph2d-text` (espaçamento entre letras, `6b4ff2373e`) · `ph2d-code-help`.

O que um merge pode partir:
- **Quadro** (MiroClone W24+): `FORMAT_VERSION`/`PROTOCOL`/`ElementKind` — a variante `Kanban` entra no enum; um segundo merge de variante exige renumerar os dois e as fixtures de bytes.
- **`PROJECT_SCHEMA`** e a escada `project_schema.rs`: o degrau 194 é por campo `kanbans` no fim; outra linha com 194 → recontar e re-ordenar o campo.
- **`ph2d-app-registry-init`**, `ph2d-panel-registry-init`, `i18n/lib.rs`: adições em linhas vizinhas (Mergiraf pode apagar uma remoção em array — confira por `git diff`).
- **`ph2d-app-board` → `ph2d-app-kanban`** (dep família → família, vermelho J7 conhecido no gate de arquitectura; ver §9).
- **`ph2d-script`/luau**: registo `ph2d.kanban`; a linha CodeEditor mexe no mesmo registo e em `ph2d-code-help`.

## 7. Para o integrador

- **Catraca `the_shell_only_shrinks`** (`crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs:112`): `TETO_LOC = 189_041`; sem alteração do tecto na linha. Última medição da linha: **189 023** (lote E1, `2403db62e1`) — a W2 tinha chegado a 189 086 e foi curada MOVENDO. Reconte na árvore combinada (integrar → `cargo fmt --all` → medir → escrever). Nenhuma linha lhe toca.
- **ADR 0195 PROVISÓRIO.** Último no main: `0191`; há `0192..0194` noutras linhas por contar. Renumere pelo código: `git ls-tree main docs/architecture/decisions/`, depois `grep -rn "0195" docs crates` (o ADR, `decisions/README.md`, handoffs, doc-comments `[ADR-0195]`).
- **Cenas de smoke** (contadas nos roteadores, não escolhidas): `PH2D_KANBAN_SMOKE=1..9` (`CENAS: u8 = 9`, `ph2d-app-kanban/src/smoke.rs:34`; 1–2 W2, 3 verso, 4 verso/datas, 5–6 filtro, 7 cabeçalho/Share, 8 Butler/verso ligado, 9 import do Trello) · `PH2D_BOARD_SMOKE=30` (`ph2d-app-board/src/smoke.rs`; o main chega a 29 → a 30 é a do Kanban no canvas; **recontar** se o MiroClone somou) · `PH2D_PLANNER_SMOKE=1` · `PH2D_CODE_SMOKE=10` (`CENAS: u32 = 10`, `ph2d-app-code/src/smoke.rs:11`; o main anuncia 1..9).
- **A suíte COMPLETA NÃO foi corrida na linha** (ordem do dono 23/10): só testes por NOME, clippy das crates estáveis (`84e4debf44`, `6230d6c770`) e fmt. `nextest-impacted` + clippy `--all-targets` + `ship.sh` são do integrador. Precisam de `PH2D_SUITE="motivo: crate NOVA …"` ou `fecho`.
- **Flakes de carga conhecidos** (confirme com `loadavg` antes de culpar um commit; [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)): profundidade da timeline, traço de máscara, timeout do painter (fora do Kanban; herdados do W2). Medições da linha tomadas a `load` 28–33 são cota SUPERIOR.
- **Portões vermelhos conhecidos** no HEAD (23/25 UI/arquitectura verdes): `app-board → app-kanban` (família → família, cura = J7/porta no `ph2d-kanban-sync`, §9). O `ph2d-app-planner` já não depende do `ph2d-app-kanban` (`29427ab3c6`).
- **Memória/docs**: dono 28/10 — W2b..W11 em loop autónomo, smoke só no fim (`project_kanban_autonomous_loop_owner_order_2026_10_28`). Decisões de PRODUTO tomadas sem o dono (para rever no smoke): [`DECISOES_SEM_O_DONO.md`](../loop/DECISOES_SEM_O_DONO.md). Medições (`MAX_*` e tectos): [`MEDICOES.md`](../loop/MEDICOES.md).

## 8. Prova de fecho e smokes

Verdes por nome na linha (ver `ESTADO.md`): seam tests do hero (`ph2d-app-kanban/tests/it/`), modelo (degraus v1→v7 com fixture), `ph2d-kanban-sync` `loopback::two_processes_converge_*` (127.0.0.1), gate das duas janelas (aba e canvas do MESMO Kanban, `2603d3023c`), tokens (contraste por par). Fotos em `docs/TrelloClone/smoke/W2..W10/`. Tutorial do smoke final: `docs/TrelloClone/tutorial/01_kanban.pdf` (18 p.; 22 passos ainda SEM foto → lote FOTO depois do J7, depois regenerar o PDF).

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

**Pendências do `ESTADO.md`** (ligação por fechar):
- J4: `is_link` do Join do Quadro aceitar `KANBAN_SCHEME` (editor-core `board_live.rs`); avatares esmaecidos ao reconectar; Quadro convidado com retrocesso exponencial; presença do Kanban dentro da sessão do Quadro; histórico de versões do Quadro guarda ids locais.
- W8 L2: mover `short_film`/`sky_png` para crate baixa (feito em `ph2d-kanban-scenes`); app correr scripts Kanban com handlers `on()` activos; UI do Butler completa.
- W11: fiação por fazer — `frame.rs` `serve(hero)`; menu do Quadro «Convert to card»; shell `publish_scene` + drenar `take(Scene/Timeline)`; app-code `take(Script)→CodeDocs::open`; Macros `take(Macro)`; secção «Scene object» no verso; nota nova do cartão no tamanho do texto.
- W7: `back/dates.rs` deve usar o `datepick.rs` das views (uma lei).
- W2b: tracking das maiúsculas (ligar `paint_text.rs`/`text_elide.rs`/`list_title`).
- W4: desfazer «inserir Kanban» deixa o documento vazio órfão (decidir: apagar se intocado); medir limiar do LOD (feito na cena 30; reconferir); `board_nav.rs` = 700 linhas (no tecto).
- W6: teste no elemento do Quadro; painel do filtro cortava o topo (J1 curou — reconferir em foto).
- W3: gancho «quer outro quadro» para `front::cover::pending()`.
- Fecho batched por fazer: `cargo fmt`/clippy `--all-targets` sobre as crates novas de W7–W11 + revisor final.
- Smoke FINAL do dono: do PDF; decisões sem o dono (tabela acima) por rever.

## J7 (em curso)

> A completar pelo coordenador: o que o J7 comita (porta `ph2d-kanban-sync::port`, `redial.rs` no `ph2d-board-net`, `PROTOCOL 14`, `j7_porta.rs`), cura do vermelho `app-board → app-kanban`, hash final, `Cargo.lock`, HEAD definitivo, recontagens (PROTOCOL, catraca).

## Entrada §5.1 proposta (≤ 700 bytes; o integrador aplica)

- **Kanban (TrelloClone)** — o Kanban é um documento do projeto ([ADR-0195](docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md)): aba como no Trello (verso, vistas, filtro, Butler, `ph2d.kanban` Luau), o mesmo no canvas do Quadro, ao vivo, import do Trello, Planner; `ph2d-kanban-*` + `ph2d-app-kanban`/`-planner`. Smoke `PH2D_KANBAN_SMOKE=1..9` · `PH2D_BOARD_SMOKE=30`. Último: [W2b–W11, 28/10](docs/TrelloClone/handoffs/HANDOFF_INTEGRACAO_line_TrelloClone_2026-10-28_W2b_a_W11.md) · [plano](docs/TrelloClone/01_plano.md) · [docs](docs/TrelloClone/README.md)
