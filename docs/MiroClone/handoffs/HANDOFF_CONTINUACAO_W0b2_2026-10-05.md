# HANDOFF DE CONTINUAÇÃO — `line/MiroClone`, W0b (1.ª metade) fechada → resto da W0b / W1 (2026-10-05)

> Para a PRÓXIMA JANELA desta linha (não é handoff de integração). Antecessor:
> [`HANDOFF_CONTINUACAO_W0b_2026-10-05.md`](HANDOFF_CONTINUACAO_W0b_2026-10-05.md) (§1 decisões do dono,
> §2 o que a W0a deixou, §4 o que custou — continuam a valer). Plano: [`../02_plano.md`](../02_plano.md).

## §0 — Identidade

- Branch `line/MiroClone`, worktree `Worktrees/line-MiroClone`. Rebase sobre `main` em `a46c4c200` no início
  desta janela (sem conflito). Commits desta janela: `d5b8c11df` (gestos da aba) · `bbee3a001` (régua da
  placa + plano §2.1) · `f210b792d` (literal declarado) · o deste handoff.
- ⏳ **Smoke da W0b: por fazer pelo dono** (passos no fim deste ficheiro).

## §1 — O que esta janela fez

| peça | onde | prova |
|---|---|---|
| **Renomear NO LUGAR**: duplo-clique (ou *Rename* do menu) troca a aba pelo campo `DOC_TAB_RENAME_INPUT` com o nome seleccionado; `Enter`/clicar fora grava (aparado; vazio não grava); `Esc` desiste (`Cancel` chega ANTES do `Blur` — largar `renaming` no `Cancel` é o que impede o `Blur` de gravar). A aba mede pelo maior de nome/buffer (`tab_rects(.., editing, ..)`) | `screens/hero/document_tabs_menu.rs` (`begin_rename`, `commit_rename`, `apply_event`) · pintura `document_tabs::paint_rename_field` | 4 testes |
| ⚠️ Decisão técnica: **no lugar, não diálogo** (o handoff antecessor dizia «diálogo»; é o idioma das abas de planilha, e é UMA porta para duplo-clique e menu) | — | — |
| **Menu do botão direito**: `ContextMenuKind::BoardTab { board: u64 }` → `BOARD_TAB_ROWS` (*Rename · Duplicate · Delete…*); *Duplicate* põe a cópia (`"{name} copy"`) à direita e abre-a | `menu_tables.rs`, `menu_rows.rs`, `types_menu.rs` | 2 testes |
| **Apagar pergunta antes**: `ContextMenuKind::ConfirmDeleteBoard { board }`, pintado por `document_tabs_menu::paint_confirm_delete` (precisa do NOME, que o `store` não tem ⇒ chamado no `paint.rs` logo a seguir ao overlay, que salta esta variante); `click_belongs_to_the_open_menu` mantém-na aberta no Down dos dois botões. Apagar o quadro aberto abre a vizinha (direita › esquerda › `Cena`) | `documents.rs::remove`, `pointer_down_menus.rs` | 3 testes |
| **Reordenar arrastando**: `Documents::tab_drag` (âncora + cursor), limiar `TAB_DRAG_THRESHOLD_PX` (o das abas de painel); a largada é julgada contra o `HitIndex` do quadro ANTERIOR (lei de `slot_tabs_drag`), com a marca pintada pela MESMA `drop_slot`; largar fora da faixa da barra (±1 altura) desiste; a aba largada abre | `document_tabs_menu::{pointer, drop_slot, drop_index, drop_caret}` | 2 testes |
| ⚠️ **O botão direito e o arrasto entram ANTES do despacho**: `document_tabs_menu::pointer` chamado no topo de `HeroScreen::handle_pointer` e `handle_pointer_with_text` (o despacho não sabe que um id derivado é uma aba). Só o botão direito sobre uma aba de quadro é CONSUMIDO; o arrasto só observa (o clique continua do despacho, e o empurrão de 2 px ainda troca de aba) | `screens/hero.rs` (697 L — 3 doc-comments reembrulhados para caber) | — |
| **`board_view::pointer` larga o foco** antes de tomar o Down na área do quadro (`blur_focus` + `apply_event`): sem isto o nome escrito nunca gravava ao clicar no quadro (a lei de `forwarding::forward_blur_to_hero`) | `board_view.rs` | 1 teste |
| **Régua da PLACA**: `VelloPass::render_to_intermediate` do produto, `TIMESTAMP_QUERY` com enchimento na fila (senão conta a placa ociosa durante o encode do Vello), controlo de cobertura lida de volta. 10 mil rects = **0,364 ms**, 100 mil = **1,465 ms** (mín., RTX 5060 Ti/Vulkan, loadavg 10–22; a coluna da placa não mexe com a carga) | `crates/ph2d-board-render/tests/it/measure_gpu_raster_cost.rs` (`#[ignore]`) · plano §2.1 | 3 corridas |

Testes: `document_tabs::menu_tests` (12, pelo caminho real: ponteiro/teclado pelo despacho do hero, cada
evento pelo `apply_event`, repintado entre gestos). **Prova de mutação 10/10** (M1 duplo-clique · M2
`Cancel` · M3 `blur` no quadro · M4 botão direito · M5 lado da largada · M6 vizinha ao apagar · M7 Down no
botão da pergunta · M8 apagar sem perguntar · M9 faixa da largada · M10 nome vazio). ⛔ A M6 SOBREVIVEU
à 1.ª versão: com três quadros «a da direita» e «a última» coincidiam; o teste passou a quatro.

## §2 — ⏳ O que falta (nesta ordem)

1. **Smoke do dono** (fim deste ficheiro). Um gesto que só o binário real mostra: a shell tem ramos que
   podem devolver antes do `forward_to_hero` (`ramo_preview_e_fechos` com menu aberto) — o caminho do
   teste é `HeroScreen::handle_pointer`, o mesmo das abas de painel, mas o botão direito numa aba de quadro
   COM outro menu aberto não está provado no binário.
2. **Transbordo das abas** (item 4 do antecessor): medir com 15–30 quadros se as abas encolhidas ficam
   legíveis/clicáveis; precedente `slot_tabs_overflow.rs`.
3. **Undo por quadro** (plano §1.4): nasce com o 1.º gesto que cria elementos (W1). Renomear/duplicar/
   apagar/reordenar quadros **não têm undo hoje** — a pergunta antes de apagar é a única guarda. Decidir
   com o undo da W1 se as ops de aba entram na fila do projecto.
4. **W1** (tela, formas, texto dentro da forma). ⚠️ O kill-criterion (formas + texto ≤ 8 ms na placa a 10
   mil) decide-se lá com a régua desta janela: a cena de medida (`measure_encode_cost::board_with`, agora
   partilhada pelas duas réguas) ganha formas e texto.

## §3 — O que custou e não se repete

- ⛔ `no_magic_numeric_in_widget_or_screens` reprova um `3.0` (contagem de linhas de um diálogo) sem
  `// LITERAL-PX-OK:` — os irmãos em `context_menu_dialogs.rs` declaram-no na mesma linha.
- ⛔ O id da aba de um quadro é uma bijecção XOR do `BoardId`: TODO `NodeId` corresponde a algum `BoardId`,
  logo «este id é uma aba?» só se responde contra os quadros que EXISTEM (`board_at`), nunca pelo id.
- `hero.rs` volta a 697/700: o próximo campo/gancho lá precisa de corte antes.

## §4 — Prova à saída

Gate batched sobre o diff acumulado (merge-base `a46c4c200`, 127 ficheiros): `nextest-impacted`
16 598/16 599 com o único vermelho (`no_magic_numeric`) curado em `f210b792d` e re-corrido verde ·
`clippy --workspace --all-targets -D warnings` (flags do `ship.sh`) · `check --workspace --all-targets`
warnings=deny · fmt · machete · standalone-optional · workflow-packages · censos-da-árvore-combinada 12/12.
`target/*/incremental` reclamado (41 GB). Binário `smoke` quente, 2.ª corrida depois do reclamo:
`Finished \`smoke\` profile [optimized] target(s) in 0.20s`, zero `Compiling`.

## §5 — Smoke do dono (W0b)

`cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && ./target/smoke/ph2d-host-desktop`

1. Na barra de cima, clique no **+** duas vezes → aparecem **Board 1** e **Board 2**.
2. **Duplo-clique em «Board 1»** → a aba vira uma caixa de texto com o nome seleccionado. Escreva
   `Ideias` e carregue **Enter** → a aba passa a dizer **Ideias**. (Errado: o duplo-clique só abre o quadro,
   ou as letras não aparecem na aba.)
3. Duplo-clique outra vez, escreva qualquer coisa e carregue **Esc** → o nome volta a **Ideias**.
4. **Botão direito em «Ideias»** → menu com **Rename · Duplicate · Delete…**. Clique em **Duplicate** →
   aparece **Ideias copy** logo à direita, já aberta.
5. Botão direito em **Ideias copy** → **Delete…** → abre a pergunta **Delete “Ideias copy”?** com
   **Cancel** e **Delete board**. Clique em **Cancel** → nada some. Repita e clique em **Delete board** → a
   aba some e abre a do lado.
6. **Arraste a aba «Board 2»** para a esquerda de **Ideias** → enquanto arrasta aparece uma barrinha de
   cor onde ela vai cair; ao soltar, a ordem muda. (Errado: nada muda, ou a aba muda sem a barrinha.)
7. Grave (**Ctrl+S**), feche e reabra o projecto → os nomes e a ordem continuam lá.
