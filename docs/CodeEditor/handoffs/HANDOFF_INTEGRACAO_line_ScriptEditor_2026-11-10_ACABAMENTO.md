# HANDOFF DE INTEGRAÇÃO — line/ScriptEditor: acabamento 10/11 (tabelas da casa, caixa de marcar, molduras, divisor, pesos da maquete 14)

> Leitor: o agente integrador. Antecessor: [Onda B + plano 14 Q1–Q4](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_PLANO14_Q1_A_Q4.md)
> (já no main desde `bc4a6ce930`, a base desta linha). Mecanismo e diário:
> [handoff de continuação do acabamento](HANDOFF_line_ScriptEditor_2026-11-10_ACABAMENTO.md) (§A e o smoke de lá estão
> **superados** por este: a pílula saiu). Decisões do dono: [00_decisoes_do_dono.md](../00_decisoes_do_dono.md).
> História das rondas (cortada com prova sha256 por `scripts/doc-split.py`): [_historia](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-10_ACABAMENTO_historia.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor` · branch `line/ScriptEditor` · HEAD de código `628dfddfa8` (5.ª ronda, 11–12/11)
  (o commit deste handoff vem a seguir: só docs).
- Merge-base com o `main` = `bc4a6ce930`. **168 commits** (`git rev-list --count bc4a6ce930..HEAD`, sem o commit deste handoff), **318 ficheiros, +15 792 −2 112** (com os docs e as fotos)
  (`git diff --shortstat bc4a6ce930..HEAD`). A 4.ª ronda = `39d36f2ca6..9811817c39` (base `96c5bdcb74`), 30 commits; a **5.ª = `56d4d0f315..628dfddfa8`** (base `6269005a4e`), 11 commits.
- Smoke do dono 10/11: as rondas até `e082d45b6d` **APROVADAS («Smoke OK»)**. Smoke do dono 11/11 sobre `4718a9e536` (a mira
  do submenu na diagonal, a pilha da aba curta e o resto da 3.ª ronda): **APROVADO («smoke OK»)**. **4.ª ronda (ordem do dono depois
  desse smoke: «resolver tudo antes de integrar»): smoke feito 11/11 e PARTIU com um pânico (curado na 5.ª). 4.ª + 5.ª ronda: smoke PENDENTE** (§6, passos 8–12).

## 2. Superfície de colisão

`git diff --name-only bc4a6ce930..HEAD`, por crate (**A** = ficheiro novo):

| Crate / pasta | Ficheiros |
|---|---|
| `ph2d-panel-code` (47) | `Cargo.toml` · `src/`: **A** `peso.rs`, `find_state.rs`; M `splitter.rs`, `bancada/{grid,mod,outline,phrase,popover}`, `cards`, `chrome`, `console`, `console/problems`, `debug_card/{boxes,head,live,live_marks,mod,sheet}`, `debug_marks`, `drop_mark`, `help/{paint,view}`, `lib`, `line_cache`, `metrics`, `options_card`, `paint`, `rename_box`, `script_list`, `status`, `table_text` · `tests/it/`: **A** `peso_das_pecas.rs`, `peso_na_placa.rs`, `bancada/acabamento.rs`; M `bancada{,/grelha,/puras}`, `clique`, `estado`, `main`, `medicoes_do_fecho`, `opcoes_do_script`, `pagina`, `problemas`, `quadro_do_depurador`, `valores_ao_vivo` |
| `ph2d-app-code` (17) | `src/`: **A** `column.rs`; M `console`, `doc`, `docs`, `find`, `folds`, `lib`, `note`, `play_problems`, `problem_list`, `strict`, `surface`, `tab_menu` (vários só encolhem: código movido) · `tests/it/`: **A** `divisor.rs`; M `barra_da_aba`, `main`, `onda_b` |
| `ph2d-code-assist` (4) | `src/`: **A** `folds.rs`, `play_problems.rs`, `strict.rs`; M `lib.rs` (`ClosedFolds`/`PlayProblem`/strict vieram do app-code) |
| `ph2d-code-bancada` (3) | `src/{act,lib}.rs`, `tests/it/acto.rs` |
| `ph2d-text` (4, foundational) | `src/`: **A** `peso_exato.rs`, `peso_exato_tests.rs`; M `lib.rs`, `system.rs` |
| `ph2d-editor-core` (6, foundational) | `src/`: **A** `paint_text_weight.rs`; M `paint.rs`, `paint_text.rs`, `widget/{mod,toggle_classic}.rs` · `tests/it/architecture_the_shell_only_shrinks.rs` (1 linha, abaixo) |
| `ph2d-tokens` (4, foundational) | `src/{lib,state_layer}.rs`, `tests/it/{main,the_hover_layer_lightens_a_dark_surface}.rs` |
| `ph2d-ui-table` (8, foundational) | `src/`: **A** `measure.rs`, `look.rs`; M `lib.rs`, `marks.rs`, `paint.rs` · `tests/it/`: **A** `pesos.rs`, `reserva_e_fios.rs`; M `main.rs` |
| **`Cargo.lock`** | **TOCADO:** `ph2d-panel-code` ganha dev-deps `ph2d-gpu` e `ph2d-render` (teste `peso_na_placa`: tinta medida no píxel). `no_dependency_climbs` e `cycle_prevention` **passaram**. Merge de lockfile: refaça por `cargo` se conflitar. |
| Hook / scripts / roteador | `.claude/hooks/prazo-do-agente.sh`, `scripts/processo.prova.d/I.sh`, `docs/Components/ferramentas/fotografa_cena.sh`, `CLAUDE.md` (só a linha §5.1 do módulo: o link do handoff) |
| **`ph2d-code-scene-arena` (crate NOVA, 2)** | **A** `Cargo.toml`, `src/lib.rs` (a cena 15 e o `STEPS_15`, vindos de `ph2d-code-scenes`/`ph2d-code-smoke`); consumidores `ph2d-app-code` (`smoke_drive.rs`, `Cargo.toml`) e `ph2d-app-components` (`code_smoke.rs`, `Cargo.toml`); linha nova em `TETOS_DAS_CRATES_DA_FAMILIA`; `Cargo.lock` tocado |
| «stem darkening» (`07d722387f`) | `ph2d-tokens/src/{typography,lib}.rs` (`TextRendering::ALL` = 4 ⚠️ `match` exaustivo noutra linha parte) · `ph2d-editor-core`: `published.rs`, `paint.rs`, `paint_text.rs` (692/700), `paint_text_rotated.rs`, `ids/menus_settings.rs`, `screens/hero/{chrome/settings_text,menu_row_mark,menu_tables_settings,pre_populate}.rs`, **A** `published_embolden_tests.rs` · `ph2d-i18n/src/chrome_menus.rs` · `ph2d-vector/src/lib.rs` (re-export; `ph2d-vector-doc` intocado) · `ph2d-panel-code/src/glyphs.rs`, `tests/it/nitidez.rs` |
| **11/11** (`89ac2897e9..c67465ff6e`) | `ph2d-panel-code`: **A** `src/tab_bar.rs`, `src/tab_bar/menu.rs`, `tests/it/{submenu,fundo_real}.rs`; M `mono_label` (⚠️ `paint` → `paint_on(.., bg, ..)`), `palette`, `code_text`, `help/paint`, `script_list`, `bancada/{grid,outline}`, `lib`, `tests/it/main` · `ph2d-app-code`: `tab_menu`, `commands`, `surface`, `smoke_drive`, `tests/it/{barra_da_aba,divisor,ajuda,arrastar,bancada,main,onda_b}` · **`ph2d-editor-core` (foundational, ADITIVO):** `doc_surface.rs` **`PointerInput.primary_held`** (⚠️ campo novo num literal: quem construa `PointerInput` noutra linha não compila), `documents.rs`, `documents_surface.rs`, `widget/lista_flutuante.rs`, `screens/hero/surface_input.rs`, `published_embolden_tests.rs` · `shells/desktop/src/input_dispatch.rs` (+2) · `shells/game-scenes/tests/it/` (5 ficheiros, o campo) · `ph2d-i18n/src/code_bancada.rs` |
| **3.ª** (`e040ad604c..491e4e1294`; sem foundational) | `ph2d-panel-code`: **A** `src/stack.rs`, `tests/it/{pilha,barras_e_caminho}.rs`; M `src/{tab_bar,console,script_list,view,paint,lib}.rs`, `tab_bar/menu.rs`, `debug_card/mod.rs`, `bancada/{grid,outline}.rs`, `tests/it/{submenu,problemas,virtualizacao,main}` · `ph2d-app-code`: `src/{doc,surface,debug_card,tab_menu}.rs`, `tests/it/{depurador,linha_longa}` (⚠️ `CodeDoc.bench_h` → `Want`; `TabBar::hover_move_at(x,y,now)`) · `docs/CodeEditor/ferramentas/smoke/`: `cena14/{fotos_q3.sh,caminho_diagonal_card_style.txt}`, `ax_*.png`, `ay_*.png` |
| **4.ª** (`39d36f2ca6..9811817c39`) | `ph2d-panel-code`: **A** `src/doc_rules.rs`; M `window.rs`, `stack.rs`, `tab_bar{,/menu,/keys}.rs`, `help/*`, completação, `debug_card` · `ph2d-app-code`: `docs`/`surface`/`tab_menu` (encolhem) + testes · `scripts/luau-lsp-fetch.sh` (**A**), `docs/CodeEditor/ferramentas/xvfb_livre.sh` (**A**) + 21 scripts Xvfb em `-displayfd` · `ph2d-i18n`: `code.menu.more{,_tip}` · **FOUNDATIONAL `ph2d-editor-core` (ADITIVO):** `doc_surface.rs` (métodos novos COM omissão no trait: `paint_floating`, `floats_at`, `pointer_leave`), `documents_surface.rs`, `paint_text.rs`, `published.rs`, `published_embolden_tests.rs` → **`paint_text_embolden_tests.rs`** (rename ⚠️), `screens/hero.rs`, `screens/hero/board_view.rs`, `screens/hero/paint.rs` (**700/700**), `screens/hero/publish_text_tests.rs` (**A**), `screens/hero/surface_input{,_tests}.rs`, `cascade.rs`, `palette.rs` · shell: `input_dispatch.rs`, `despacho_metodos_janela_e_vetor.rs`, `main.rs` (`CursorLeft`; 1 031 = tecto) |
| **Ronda 11–12/11 (5.ª)** (`56d4d0f315..628dfddfa8`) | `ph2d-panel-code`: `doc_rules.rs` (`tail_start` partilhado), `caret_column`, `stack.rs` (`Want.asked`), `paint.rs` (→ `CodeView::view_columns`, 600/600), `window.rs`, `tab_bar*`; `ph2d-app-code`: `goto.rs`, `context_menu.rs`, `code_least`/`cards::code_well`; `ph2d-code-explain` `read.rs` (corte a 120 B). **FOUNDATIONAL `ph2d-editor-core` (ADITIVO):** `doc_surface.rs` (`DocumentSurface::dismiss_floating`, omissão vazia), `screens/hero/board_view.rs` (1 linha, `route_pointer`), `screens/hero/surface_input.rs` (`dismiss_floating`, `dismiss_on_chrome_press`), `screens/hero/na_escala.rs` (`premido_fisico` chama-o no Down, antes dos ramos Preview F19/Pixel/áudio), `screens/hero/surface_input_tests.rs` · shell `main.rs` (`Focused(false)` → `on_focus_lost`; 1 031 = tecto) e `input_dispatch/despacho_metodos_janela_e_vetor.rs` (+8, `on_focus_lost`). ⚠️ `ph2d_code_explain` e `doc_rules::tail_start` são APIs novas; nenhuma assinatura existente mudou. |
| Docs / fotos | 2 handoffs em `docs/CodeEditor/handoffs/`; fotos em `docs/CodeEditor/ferramentas/smoke/` (+`cena13/`); `cena14/fotos_q3.sh` (`ARRASTA=`, `DUPLO=`) |

**Provas de contrato:**

- §6 (Nodes, Tools, Vector): `git diff --stat bc4a6ce930..HEAD -- crates/ph2d-nodegraph crates/ph2d-tool-api crates/ph2d-vector` = **vazio**.
- `PROJECT_SCHEMA`: `git diff bc4a6ce930..HEAD -- . ':!*.md' | grep -c PROJECT_SCHEMA` = **0**. Nenhum registo nem ADR; uma crate nova (`ph2d-code-scene-arena`, §8).

**Foundational tocado — ADITIVO** (consumidores dos removidos = 0 em `crates/` e `shells/`):

- `ph2d-text`: escopo **`com_peso_exato`** (guarda thread-local). Dentro dele `effective_weight` **não** soma o reforço do preset de
  nitidez; o desvio Light/Normal/Strong do estilo continua a valer. A chave do cache de layout é o peso efectivo (nada vaza). Repõe ao
  sair, aninhado e em pânico (`peso_exato_tests.rs`). Fora do escopo: byte-idêntico.
- `ph2d-editor-core`: pintores com peso (`paint_text_in_weight` e irmãos, `paint_text_weight.rs`); `title_tracked_width` +
  `TITLE_TRACKED_WEIGHT` (mesmo SEMI_BOLD de antes: Kanban/cartões inalterados). ⚠️ `paint_text.rs` **693 / 700**.
  `paint_switch`/`switch_inks` REMOVIDOS (sem outro consumidor; quem os use parte aqui).
- `ph2d-tokens`: `HOVER_LAYER_ALPHA` (Material 3, 8 %), usado pelo hover da linha de opções.
- `ph2d-ui-table`: `Metrics::twist`/`rules_bottom`; `Column::snug()` (largura fixa, sem puxador); `Look::House` (omissão, pesos da casa)
  / `Look::Exact` (aba de scripts, via `peso::metricas()`); medidas em `measure.rs`. Consumidores: `ph2d-panel-code` e `ph2d-code-debugcard`.
- **Catraca** `crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs`: `ph2d-code-assist` **3 925 → 4 188** (regra
  «a isenção viaja com o código», mesma forma de `8281ac0a43`). ⚠️ **PARA O INTEGRADOR:** o tecto de fonte `ph2d-app-code` 26 628 **NÃO**
  foi baixado (baixar 263 ficaria abaixo do contado) e uma lente de auditoria notou que o teto total da família cresceu: **o integrador
  decide no recontar**.

**O que um merge pode partir:** quem importe `paint_switch`/`switch_inks`/`switch_knob`; quem use `FindState`, `ClosedFolds`,
`PlayProblem` ou o «strict» por `ph2d-app-code` (agora panel/assist); `PH2D_CODE_SMOKE=1..14` (`CENAS`, inalterado); os tectos de LOC
(§5); `paint_bench`/`paint_popover`/`ConsoleView::paint` (levam `&mut CodeView`); `Cargo.lock`. ⚠️ **Hook `prazo-do-agente.sh`:** o que
corre é o do primário; a cura só vale depois da integração.

## 4. Prova de fecho (1×; sem suíte nem nextest — regra do dono 23/10, são do integrador)

- **Testes por nome e mutações das rondas até 10/11:** verdes/vermelhas, lista em [a história, apêndice](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-10_ACABAMENTO_historia.md). Última ronda: §8.
- **Portões por nome, verdes:** `file_loc_caps`, `fn_loc_caps`, `arch_safe_clamp_only`, `architecture_the_shell_only_shrinks`,
  `architecture_workspace_file_loc_cap`, `every_frame_goes_through_the_theme_door`, `no_magic_numeric`, `no_literal_color`,
  `hr15_no_hardcoded_ui_strings`, `architecture_widget_showcase_coverage`, `architecture_claude_md_cabe_no_orcamento`,
  `architecture_panel_loc_cap`, `architecture_widget_loc_cap`, **`no_dependency_climbs`, `cycle_prevention`** (Cargo.lock). `fmt --check` ✓ ·
  `clippy -D warnings` nas crates tocadas ✓ (`PH2D_SUITE=fecho`).
- **Auditoria, lentes independentes:** sem violação de regra; correcção curada (item F); a lente de ratchet gerou o aviso do §2.
- **Binário do smoke:** ver o fim do §8.

## 5. Folgas medidas (recontar na integração; HEAD `628dfddfa8`, 12/11, 5.ª ronda — só as linhas marcadas «5.ª» foram remedidas; o resto é de `491e4e1294`/`9811817c39`)

| Ficheiro / crate | Medido / tecto | Folga |
|---|---|---|
| crate `ph2d-app-code` (catraca `the_shell_only_shrinks`) — 5.ª | **26 618** / 26 628 | **10** (o integrador reconta) |
| `screens/hero/paint.rs` (editor-core) — 5.ª | 700 / 700 | **0** |
| `shells/desktop/src/main.rs` — 5.ª | 1 031 / 1 031 | **0** (tecto) |
| `ph2d-panel-code/src/paint.rs` — 5.ª | 600 / 600 (`architecture_panel_loc_cap`) | **0** |
| maiores ficheiros tocados na 3.ª ronda | `doc.rs` 668 · `surface.rs` 646 · `script_list.rs` 591 · `ph2d-panel-code/src/paint.rs` 575 / 700 | — |
| crate `ph2d-code-smoke` | 614 / 614 | **0** (não tocada) |
| crate `ph2d-panel-code` | 22 129 | sem tecto de crate |
| `ph2d-editor-core/src/paint.rs` | 700 / 700 | **0** (não tocado) |
| `ph2d-editor-core/src/paint_text.rs` | 692 / 700 | 8 (não tocado) |
| `ph2d-editor-core/src/screens/hero/paint.rs` | 700 / 700 | **0** (não tocado; o teste do `publish_text` vive em `published_embolden_tests.rs`) |
| `ph2d-app-code/src/surface.rs` | 643 / 700 | 57 |
| `ph2d-panel-code/src/tab_bar/menu.rs` · `tab_bar.rs` · `palette.rs` · `glyphs.rs` | 387 · 276 · 331 · 354 / 700 | — |
| `ph2d-editor-core/src/documents_surface.rs` · `widget/lista_flutuante.rs` | 134 · 267 / 700 | — |
| `shells/desktop/src/input_dispatch.rs` | 596 / 700 | 104 |
| crate `ph2d-code-assist` | 4 088 × isenção 4 188 | — |

`the_shell_only_shrinks` e `architecture_workspace_file_loc_cap` verdes (por nome, 11/11). O tecto 26 628 da `ph2d-app-code` não foi
baixado (o integrador reconta).

## 6. Smokes (comando exacto)

Os passos 1–7 vêm das rondas aprovadas (ficam para o próximo testador); **8–10 são da 4.ª ronda e 11–12 da 5.ª: PENDENTES do dono**.

Smokes 1–7 (cenas 11, 12, 13, 15 e divisor/peso; todos APROVADOS pelo dono) — texto, comandos e fotos em [a história, apêndice](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-10_ACABAMENTO_historia.md). Comando-base: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=<11|12|13|15> cargo run --profile smoke -p ph2d-host-desktop`.

8. **Janela pequena (4.ª ronda, NOVO).** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=4 ./target/smoke/ph2d-host-desktop` →
   encolha a janela até ~900×560. (1) a barra de menus manda os títulos que não cabem para um **«…»** antes do caminho e da engrenagem;
   «…» ▸ View/Run abre o mesmo que o menu do título (atalhos, inactivas a cinzento). (2) os menus abrem **por cima de tudo** (Run
   inteiro, coluna Scripts e Inspector tapados), com ▲▼ quando não cabem: roda e **rato parado na seta** rolam sozinhos e pararam no
   limite. (3) teclado: ↑↓ saltam as inactivas, → abre o submenu, ← fecha, Enter corre, Esc fecha um nível. (4) com pouca altura o
   **código guarda as linhas** e os cartões caem ao cabeçalho (Bancada, depois a faixa, o depurador por último). Errado se: menu cortado
   pela aba, texto do código a zero, auto-rolagem que não pára ao tirar o rato da janela. Fotos:
   [bd_menus_estreito](../ferramentas/smoke/bd_menus_estreito.png) · [bd_menus_transbordo](../ferramentas/smoke/bd_menus_transbordo.png) ·
   [bd_menus_view_aninhado](../ferramentas/smoke/bd_menus_view_aninhado.png) · [bc_janela_900x560](../ferramentas/smoke/bc_janela_900x560.png) ·
   [bc_janela_1280x720](../ferramentas/smoke/bc_janela_1280x720.png).
9. **Popups (4.ª ronda, NOVO; precisa do `luau-lsp`: `bash scripts/luau-lsp-fetch.sh`).** Mesma janela pequena, cena com o assistente: a lista de
   autocomplete e a página de ajuda com **barra** (arrastável); balão e exemplo da ajuda cortados acabam em «…». Fotos `bb_*` em
   [smoke/](../ferramentas/smoke/) (autocomplete e ajuda antes/depois; `bb_janela_900x560_editor_sem_texto` = o defeito).
10. **Linha longa (4.ª ronda, NOVO).** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=4 ./target/smoke/ph2d-host-desktop` com a linha colada de 64 KB, vá ao fim (End): o
   texto à vista pinta (antes: vazio). Foto [ba_linha_longa_fim](../ferramentas/smoke/ba_linha_longa_fim.png).

11. **Pânico de fronteira de char (5.ª, NOVO).** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=4 cargo run --profile smoke -p ph2d-host-desktop` → ponha o cursor numa palavra logo depois de «»» ou «—» (texto com separador de vários bytes) e mexa o cursor/abra o go-to e o menu de contexto: **nada cai**, a coluna acompanha. Errado se: o app fecha com `not a char boundary`.
12. **Menu da barra e faixa Problems (5.ª, NOVO).** Mesmo comando, cena 4: (1) com um menu da barra aberto, clicar no Inspector (ou noutro painel) **fecha-o**, e ↓/Enter já não mexem no menu; tirar o foco da janela também o fecha. (2) Janela ~900×560: clicar nos contadores ⊗/⚠ abre Problems e a lista **fica** (cabeçalho + 1 linha; 113+37+44+16 = 210 ≤ 225). Errado se: o menu continua aberto depois do clique fora, ou a lista de problemas some.

## 7. Aberto

1. A cura do hook só vale depois da integração (corre o do primário).
2. `ph2d-editor-core/src/paint.rs` 700/700 e `screens/hero/paint.rs` 700/700 — **não tocados de propósito** (o `hero/paint.rs` só encolhe/igual): partir foundational só fabrica conflitos; quem precisar de crescer lá cura movendo. `paint_text.rs` 692/700.
3. O integrador reconta as catracas (`ph2d-app-code` 26 618 × 26 628, folga **10**; `ph2d-code-assist` 4 188; `main.rs` 1 031/1 031; `ph2d-panel-code` `paint.rs` 600/600).
4. **Memória no primário `project-memory/`, NÃO comitada** (as do dono de 10/11) — o integrador funde-as.
5. **Smoke do dono:** rondas até `4718a9e536` APROVADAS; 4.ª ronda feita 11/11 e partiu no pânico (curado); **4.ª + 5.ª PENDENTES** (§6, 8–12).

Fecharam na 5.ª ronda: (c) faixa Console/Problems que saía a 900×560 (agora fica, `Want.asked`) e (d) >250 colunas de chars de 4 bytes na linha longa (tecto efectivo = max(`LINE_RENDER_BYTES_MAX`, bytes do degrau + colunas à vista), `8db611383e`). Abertos:

- **(a)** Sem teste na `ph2d-app-code` do caminho completo hero→`CodeDocs` (`dismiss_floating`, setas/Enter por `CodeDocs::key`); cobertos em `ph2d-panel-code` e `ph2d-editor-core`.
- **(b)** `WindowEvent::CursorLeft`/`Focused(false)` e os três ramos da shell (Preview/Pixel/áudio) só provados por compilação + o teste do `premido_fisico`: smoke de mão (§6, 8 e 12).
- **(c)** PRÉ-EXISTENTE, fora do diff desta linha: `shells/desktop/src/input_dispatch/despacho_clique_flip.rs` `ramo_preview_e_fechos` — com o Preview do Vector ligado e uma aba de scripts ativa, `over_canvas_or_gizmo` pode tomar cliques dentro da área da aba (por verificar; a dona do Preview é UI/UX).
- **(d)** Com uma fonte CJK do sistema de 2 colunas, a conta mono do editor assume 1 coluna por grafema (a mono embutida desenha 1: não é defeito hoje).
- **(e)** Oráculos: 10 dos 12 só com `bash -n`; I10 cena 6 clica em coordenada fixa; a cor `#264654` por confirmar; a sonda da grade de camadas grava `{}`.
- **(f)** `agent-loop-profile.sh` não corrido na 5.ª (o último: paralelismo 1,15 ✗, test:check 2,9× ✗).

## 8. Rondas anteriores e binário do smoke
- **Ronda 11–12/11 (5.ª)** (`56d4d0f315..628dfddfa8`, base `6269005a4e`, 11 commits; ordem do dono: «Auditoria final · Corrija tudo · escreva handoff · Não integre»). Pânico `not a char boundary` em `caret_column` curado (`56d4d0f315`, `910ca5f7fc`); auditoria F1–F6 curada (menu da barra que não fecha, faixa Problems, selecção cortada, `pointer_left`, `code_least`, submenu invisível). Achados, commits e testes: [história, apêndice](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-10_ACABAMENTO_historia.md).
  (3) **Prova de fecho (HEAD `628dfddfa8`):** fmt ✓ (7 crates) · clippy `-D warnings --all-targets` ✓ (panel-code, app-code, editor-core, host-desktop, code-assist, code-explain, i18n) · crates inteiras: panel-code lib 22 + it 185 (2 ign.) · app-code lib 9 + it 273 (3 ign.) · code-assist 22 · code-explain 7 · editor-core lib 2 146 (6 ign.) + it 630 (12 ign.) + `no_alloc` 1+1 · shell `input_dispatch` por nome 21 ✓ · check host-desktop ✓ · `the_shell_only_shrinks`, `architecture_workspace_file_loc_cap`, `the_foundation_modules_form_a_dag`, `arch_safe_clamp_only`, `no_magic_numeric` ✓ · `commit-barato` ✓. Suíte completa e nextest: do integrador.

Binário de smoke depois de `rm -rf target/*/incremental` (12/11, 5.ª ronda, HEAD `628dfddfa8`); a 2.ª:

```
    Finished `smoke` profile [optimized] target(s) in 0.30s
```

