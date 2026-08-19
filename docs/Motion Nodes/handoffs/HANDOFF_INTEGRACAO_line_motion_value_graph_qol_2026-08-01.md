# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (QoL do editor de nós)

**Status:** FECHADO 2026-08-01 · no `main` em `cd9646734` (o commit que trouxe este arquivo).

**Data:** 2026-08-01 · **Branch:** `line/motion-value` · **HEAD:** `879d9703c` · **Base:** `main`
**Ordem do Enio:** *"Smoke OK. Handoff para outro agente fazer integração ao MAIN"* — todos os smokes aprovados.

> ⚠️ Esta é a **continuação pós-integração** da `line/motion-value` (a jornada de 2026-07-30 já
> integrou; ver CLAUDE.md §5). São **10 commits novos**, todos de **usabilidade (QoL) do editor de
> nós** do Motion — teclado, clipboard, menus de contexto e seleção de fio. **Nenhum schema, nenhum
> contrato congelado, nenhum ADR.**

---

## 1. O que a linha entrega (10 commits, `main..HEAD`)

| Commit | Fatia | Superfície |
|---|---|---|
| `0b76efc64` | **Copy/Paste** de nós (Ctrl+C / Ctrl+V) | clipboard `GraphClip` (transiente) |
| `f46e951f5` | Fix: Ctrl+V colava **DUAS vezes** (double-dispatch não é inócuo p/ verbo não-idempotente) | `dedup_double_dispatch` |
| `b19aa0383` | **Cut** (Ctrl+X) = copy-then-delete, 1 undo | `GraphKey::Cut` (foundational) |
| `89727ced1` | **Select Inverse** (Ctrl+I) — fecha a família de seleção | `GraphKey::SelectInvert` + `KEY_KEY_I` (foundational) |
| `93b3247e4` | Ctrl+V de um **grupo** cola COMO grupo (paste preserva nesting) | shell `motion_bridge_subgraph::paste_nesting` |
| `aeef7535d` | **Menu de contexto do nó** (R-click) — os verbos ganham uma mão | `MenuBody::NodeActions` |
| `88d3c280f` | Fix smoke: **Rename fora de multi-seleção** + **Mute funciona em grupos** | fold card.bypassed + SetBypass expande cards |
| `bbb37bac1` | Menu do **card de grupo** ganha **Enter + Ungroup** | `NodeAction::requires_group` |
| `19297de3e` | Menu do **backdrop** ganha **Rename + Delete** (não só as cores) | `BACKDROP_ACTIONS` |
| `879d9703c` | **Clicar num fio o SELECIONA; Delete o remove** | `MotionGraphPanelState.selected_wire` |

**Resultado:** o editor de nós ganhou o conjunto completo de idiomas de um editor profissional —
clipboard (Copy/Cut/Paste com nesting), seleção (All/Inverse/Linked/box), os menus de contexto de
**todo** sujeito de R-click (nó · card de grupo · backdrop · fio via seleção), e a seleção de fio
com Delete. Cada verbo passa por **UMA porta** (`apply_key`), então o menu nunca faz algo diferente
do atalho.

---

## 2. ⚠️ Toque FOUNDATIONAL (`ph2d-editor-core`) — precisa do gate da árvore combinada

Esta linha **toca a superfície de `GraphKey`** (a foundation da interação), de forma **aditiva**:

- `crates/ph2d-editor-core/src/interaction/types.rs` — `enum GraphKey` ganhou **`Cut`** e
  **`SelectInvert`** (variants apendados). **GraphKey NÃO é contrato congelado** (§6 congela
  `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/…; GraphKey é enum interno de interação, evolui
  livre). **Não há gate que pine a contagem de GraphKey.**
- `crates/ph2d-editor-core/src/interaction/dispatch/keymap.rs` — `pub const KEY_KEY_I = 0x49`.
- `crates/ph2d-editor-core/src/interaction/dispatch/key.rs` — a **ONE map** `graph_key_for`
  ganhou `Cut` (Ctrl+X), `SelectInvert` (Ctrl+I).
- `crates/ph2d-editor-core/src/interaction/dispatch/mod.rs` + `interaction/mod.rs` — as **duas**
  listas de re-export ganharam `KEY_KEY_I` (explícitas, não glob).

⚠️ **A regra da ONE map:** o shell consulta o MESMO `graph_key_for` (seu router roda no CURSOR, não
no focus gate). Por isso o shell foi tocado também (§3) e há o gate
**`..._through_the_shells_normalizer`** (shell) provando que **todo** GraphKey — incluindo os dois
novos — sobrevive ao normalizer do shell.

**Conflito possível na integração:** se outra linha da janela também apendou um `GraphKey` variant
ou mexeu em `graph_key_for`/`keymap.rs`, é add/add no MESMO enum/map → resolver mantendo **as duas**
adições (é aditivo, não há colisão semântica). Não há número pinado a reconciliar.

---

## 3. Toque no SHELL (`shells/desktop`) — bridge do Motion + normalizer

- `keymap.rs` (+ `input_handlers.rs`) — a tabela do normalizer + o router do cursor ganharam
  `Cut`/`SelectInvert` (o segundo leitor da ONE map).
- `motion_state.rs` — **`GraphClip`** (o clipboard do grafo, **transiente, NÃO serializado**)
  estendido para carregar `nodes`/`edges`/`subgraphs` + `pastes` (portável entre níveis / após os
  originais sumirem).
- `render_loop/motion_bridge_edit.rs` — `copy_selection(motion, nodes, cards)` captura o nesting;
  `paste()` delega a `motion_bridge_subgraph::paste_nesting`.
- `render_loop/motion_bridge_subgraph.rs` — `paste_nesting` (funde grupos colados como grupos).
- `render_loop/motion_bridge_intents.rs` — os consumidores `CopySelection`/`DeleteSelection`/
  `DuplicateSelection`/**`SetBypass`** expandem os **cards** de grupo para os nós-membro.
- `render_loop/motion_bridge_fold.rs` — `card.bypassed` = **todos** os membros mutados (fez o Mute
  de grupo aparecer/desligar; era `false` fixo).
- `splice_smoke.rs` — doc + bullets das cenas (o smoke `PH2D_SPLICE_SMOKE=1`).

---

## 4. Schema / contrato / registro — **TUDO INTACTO**

- `PROJECT_SCHEMA` / `VEC_SCENE` / `DOC_VERSION` / `FLIP_SCHEMA` — **não tocados** (conferido: o
  diff não altera nenhuma dessas constantes; o grafo do Motion viaja como **TEXTO** e carrega a
  própria versão, e o `GraphClip` é transiente).
- Contrato congelado `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — **intacto** (o diff não os toca; o
  shell compila).
- **Nenhum ADR** (é QoL puro — sem schema, sem contrato, sem crate nova, sem dep nova).
- **Nenhuma dep nova**, **nenhum `Cargo.toml` tocado**.

---

## 5. ⚠️ LOC caps — o gate que NÃO roda com `cargo test -p`

Durante a linha, o `ph2d-panel-motion-graph` cruzou o cap de **600** do painel **em silêncio** (o
gate `architecture_panel_loc_cap` mora na `ph2d-editor-core` e não roda numa varredura por-crate —
a mesma família do miss do `file_loc_caps` que outras linhas documentaram). Corrigido por SPLITS
(nada de allowlist):

- Os testes de menu de contexto saíram para **`interact_context_menu_tests.rs`** (novo).
- Os testes de fio saíram para **`interact_wire_tests.rs`** (novo).
- `dedup_double_dispatch` e `select_on_press` (lógica de teclado/seleção) mudaram-se de `interact.rs`
  para **`interact_key.rs`**, onde pertencem.

**Estado agora (todos < 600):** `interact.rs` 598 · `interact_key.rs` 232 · `interact_menu_tests.rs`
365 · `interact_context_menu_tests.rs` 263 · `interact_wire_tests.rs` 97.
⚠️ `interact.rs` está **a 2 linhas do teto (598/600)** — a próxima adição a ele obriga um split real.

**O integrador DEVE rodar os dois gates de LOC na árvore combinada** (eles não saem no `cargo test
-p`): `architecture_panel_loc_cap` (600) e `architecture_workspace_file_loc_cap` (700).

---

## 6. Bill of health (verde no tip, antes do rebase)

- **Painel** (`cargo test -p ph2d-panel-motion-graph --lib`): **106 passed**.
- **Shell** (`cargo test -p ph2d-host-desktop --bins motion_bridge`): **124 passed, 2 ignored**.
- **Normalizer** (`..._through_the_shells_normalizer`): **1 passed** — os 2 GraphKey novos chegam.
- **LOC caps** (painel 600 + workspace 700): **verdes**.
- **clippy** `-p ph2d-panel-motion-graph --all-targets`: **limpo**.
- **Shell compila** com as adições foundational de `GraphKey` (`cargo check -p ph2d-host-desktop`).

Cada fatia de QoL tem **gate mutação-testado** (RED→GREEN sobre algo antes VERDE, cada mutação
isolando uma defesa distinta — `feedback_layered_defenses_need_per_layer_gates`).

---

## 7. Passos de integração (o integrador)

1. `cd` na worktree `line/motion-value` (ou faça o merge a partir de onde preferir) e
   **`git rebase main`** — a rota "linha reaberta". Vigie conflitos em:
   - `ph2d-editor-core/src/interaction/types.rs` (add/add de `GraphKey` — mantenha ambas as adições)
     e `dispatch/key.rs` / `keymap.rs` / os dois `mod.rs` de re-export.
   - `shells/desktop/src/keymap.rs`, `input_handlers.rs`, `render_loop/motion_bridge_*` (se outra
     linha mexeu no bridge do Motion).
2. Rode `scripts/foundational-integrate.sh` (gate da árvore combinada — é foundational).
3. Rode a suíte de gates que o `cargo test -p` **não** alcança, na árvore combinada:
   `architecture_panel_loc_cap`, `architecture_workspace_file_loc_cap`,
   `architecture_contract_surface`, `architecture_panel_wiring_parity`, o normalizer do shell.
4. `./scripts/ship.sh` (paridade CI) → corrija todo `✗` → **só então** push (ordem do Enio; §3).

---

## 8. Smokes (todos aprovados pelo Enio, `--release`)

`env PH2D_SPLICE_SMOKE=1 cargo run -p ph2d-host-desktop --release` — a cena-guarda-chuva:

- **Clipboard:** Ctrl+C / Ctrl+V (cola com offset, vira a nova seleção) · Ctrl+X (copy-then-delete,
  1 undo) · Ctrl+V de um **grupo** cola como grupo.
- **Seleção:** Ctrl+A / Ctrl+I / Ctrl+L / box-select.
- **Menu do nó** (R-click): Cut/Copy/Dup/Delete/Mute/Rename; multi-seleção **esconde Rename**.
- **Menu do card de grupo**: Enter (topo) + Ungroup (fim) + as edições; **Mute** apaga/reacende o
  grupo inteiro.
- **Menu do backdrop**: 8 cores + **Rename** + **Delete**.
- **Fio:** clique-esquerdo **seleciona** (realce) → **Delete** remove; clicar nó/backdrop tira o
  realce; alt-click ainda desconecta direto.

---

## 9. Notas / gotchas

- ⚠️ **A ONE map é a espinha:** todo verbo do grafo passa por `graph_key_for` (foundational) **e**
  pelo normalizer do shell. Um verbo novo que só entra num dos dois é a doença que já fez `Ctrl+G`
  alternar a grade da cena (doc no `key.rs`). Os 2 GraphKey novos passam pelos dois (gate provado).
- ⚠️ **`GraphClip` é transiente:** não entra em `ProjectState`, não serializa — por isso nenhum
  bump de schema. Se um dia o clipboard for persistido, aí sim há schema.
- ⚠️ **`interact.rs` a 598/600:** sem folga. A próxima fatia que o tocar deve **splitar** (candidato
  natural: extrair os handlers `apply_node`/`apply_background`/`apply_socket_*` para um irmão).
- **Aberto (não-bloqueante), de outros donos / decisões de produto:** mutar um **grupo** inteiro
  (card de subgrafo) por um caminho que não expanda os membros (hoje o Mute expande — é o certo) ·
  seleção de **múltiplos fios** (hoje `selected_wire` é um único) · o realce do fio reusa o visual
  de hover (v1 — um realce distinto seria mudança de `draw_wire`).
