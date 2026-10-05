# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, W0a fechada → W0b (2026-10-05)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Plano: [`../02_plano.md`](../02_plano.md)
> (§1 arquitectura, §2.1 medições, §3 ondas). Pesquisa: [`../01_pesquisa_miro_excalidraw.md`](../01_pesquisa_miro_excalidraw.md).
> Oráculo: [`../ferramentas/excalidraw_oracle/`](../ferramentas/excalidraw_oracle/README.md).

## §0 — Identidade

- Branch `line/MiroClone` (renomeada de `line/Components2` no mesmo dia), worktree `Worktrees/line-MiroClone`.
- Base (merge-base com `main`): `5d596eaaf`. HEAD à saída: ver `git log -1` (último commit desta sessão:
  o deste handoff, sobre `2238ff39a`).
- **Smoke W0a APROVADO pelo dono (2026-10-05):** abas, criar, alternar, zoom/arrastar, gravar/reabrir.

## §1 — Decisões do dono que mandam (não re-litigar)

1. Equipa em 2 etapas: Etapa 1 local (comentários/histórico NO ficheiro), Etapa 2 ao vivo (plano próprio).
2. Sticky **cresce na vertical**. 3. Botão **Rascunho ↔ Final** (traço à mão / limpo). 4. Barra curta.
5. ⛔ **Cada quadro é uma ABA na barra de cima e NUNCA aparece na Hierarquia** (substituiu o «modo no
   seletor Mode»; o `ObjectKind::Board`/`ModeFamily` da 1.ª versão do plano está RECUSADO, §6 do plano).
6. «As nossas formas e setas talvez não sirvam»: medido → reusar o roteador puro `ph2d-vec-connect`
   (desvia de obstáculos; o Excalidraw 0.18.1 NÃO desvia — fixture no oráculo) e a geometria `ShapeKind`;
   núcleo de quadro próprio, sem herdar o modo Vector.

## §2 — O que existe (W0a)

| peça | onde |
|---|---|
| modelo puro: `BoardSet`/`Board`/`BoardDoc`/`Element` (versão+nonce+lápide), `BoardOp` com inverso, `apply_batch`, `Camera` (ecrã↔mundo, zoom à volta do cursor), `FracKey` (esquema publicado do `fractional-indexing`, 17 vectores), bytes com `FORMAT_VERSION = 1` | `crates/ph2d-board-model/` |
| desenho: fundo, grelha de pontos de densidade constante (1 caminho), rects vivos em z, recorte | `crates/ph2d-board-render/` · régua `tests/it/measure_encode_cost.rs` (`#[ignore]`) |
| estado no editor: `HeroScreen.documents` (`Documents`: quadros + aba activa + âncora do arrasto) | `crates/ph2d-editor-core/src/documents.rs` |
| abas `Scene · Board n · +` na barra de menus (entre menus e abas de layout), ids derivados `XOR 0xb0a2_d7ab_0000_0003`, `register_board`, `load`; `DoubleClick` aceite como clique | `screens/hero/document_tabs.rs` (+`_tests.rs`, clique REAL) · ids fixos `ids/chrome/documents.rs` (`DOC_TAB_SCENE`, `DOC_TAB_NEW`) |
| vista: roda = zoom, arrastar (esq./meio) = vista; área = `draw_area ∪ tool_bar` | `screens/hero/board_view.rs` |
| overlays da cena (réguas, gizmos, selecção, prefab, etiquetas) MOVIDOS verbatim (moved-proof 195 L) e saltados com quadro activo | `screens/hero/paint_canvas_overlays.rs` |
| shell: `ProjectFile.boards: Vec<u8>` no FIM, `PROJECT_SCHEMA 183 → 184`; load recusa blob ilegível; roda/clique/movimento → `board_view` | `shells/desktop/src/{project,project_save,project_load,input_dispatch}.rs` |
| família `ph2d-app-board` (`FAMILY`, `PH2D_BOARD_SMOKE=1`), chamada no `init.rs` | `crates/ph2d-app-board/` |
| i18n `board.tab.{scene,default_name,new}` + `shell.project_load.project_refused_its_boards` | `crates/ph2d-i18n/src/{board,shell_media}.rs` |

## §3 — ⏳ W0b — o que falta (nesta ordem)

1. **Renomear** (duplo-clique na aba → diálogo; precedente `palette_rename.rs` + `CenteredDialog` em
   `context_menu_dialogs.rs`). ⚠️ O `DoubleClick` numa aba de quadro hoje só ACTIVA — é aqui que entra.
2. **Menu do botão direito** na aba: Renomear · Duplicar · Apagar (com confirmação). `BoardSet::{rename,
   duplicate, remove}` já existem e têm testes.
3. **Reordenar** por arrastar (precedente `slot_tabs_drag.rs`); `BoardSet::move_tab` existe.
4. **Overflow**: com muitos quadros as abas encolhem por igual (`tab_rects`); medir se precisa de setas
   como `slot_tabs_overflow.rs`.
5. **Undo por quadro** (plano §1.4): nasce com o 1.º gesto que cria elementos (W1); decidir (a) fila
   única filtrada pela aba × (b) histórico por quadro — conferir primeiro como Motion/Timeline fazem.
6. **Régua da PLACA** (raster do Vello) na mesma cena, regra do dono de 05/10 (A/B no mesmo processo,
   mínimo de 7×20, `pass_profiler::drain` ou soma por janela). Decide o kill-criterion da W1 (§2 do plano).
7. Depois: W1 (tela, formas, texto dentro da forma) — §3 do plano.

## §4 — O que custou e não se repete

- ⛔ Gates que leem `screens/hero/paint.rs` **por caminho** (5 deles) ficaram vermelhos quando o bloco se
  mudou — hoje leem `paint_canvas_overlays.rs` (ou os dois). Mover mais código de lá = varrer
  `git grep -n 'screens/hero/paint.rs'` antes.
- ⛔ O censo `a_fourth_branch_on_the_canvas_wakes_this_law` conta `on_canvas` no `input_dispatch.rs`: o
  ramo do quadro está declarado lá (`board_view::pointer(`). Um ramo novo do quadro que receba
  `on_canvas` acorda-o outra vez.
- ⛔ `hero.rs` (≈699) e `pre_populate.rs` (≈697) estão no tecto de 700 linhas: campo novo no hero = cortar
  antes (reembrulhar comentários a 100 col. foi a cura desta vez).
- ⛔ `z_on_top` O(n) e chave só-fracção: curados; não voltar a gerar chaves sem a parte inteira.
- Cliques sintéticos não chegam à janela virtual do `fotografa_cena.sh`: o clique prova-se no gate
  (`document_tabs_tests::click`, Down+Up pelo despacho real); a foto usa `PH2D_BOARD_SMOKE`.

## §5 — Prova à saída

`nextest-impacted` 16 588/16 588 · `clippy --workspace --all-targets -D warnings` verde ·
`check --workspace --all-targets` warnings=deny verde · machete · fmt · standalone-optional ·
workflow-packages · censos-da-árvore-combinada (114/114) verdes. Binário `smoke` quente
(`Finished … 0.26s`, zero `Compiling`).
