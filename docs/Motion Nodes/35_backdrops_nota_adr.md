# 35 — Backdrops (grupos no editor de nós) — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** editor **F2**
**Status:** implementado, testado, **pendente smoke do Enio**
**Contratos congelados encostados:** **nenhum** · **Foundational tocado:** **nenhum**

---

## 1. O que era, e o que faltava

O `Backdrop` **já existia inteiro** desde o M0 e nunca teve UI:

| peça | estado antes |
|---|---|
| `ph2d_motion_doc::Backdrop {id, x, y, w, h, color, title}` | ✅ existia |
| seção `[backdrop]` no formato textual (save/load) | ✅ existia — **sem nenhum produtor** |
| tokens `graph-backdrop-1..8` | ✅ existiam |
| `GraphHitKind::Backdrop` / `BackdropResize` | ✅ existiam |
| `IconId::Backdrop` | ✅ existia |
| **a UI** | ❌ **nada** — o `interact.rs` dizia literalmente *"backdrops land later"* |

Ou seja: os trilhos do M0 estavam postos e o trem nunca passou. Com **71 nós** no registro, um
grafo real vira sopa de letrinhas sem regiões nomeadas. Esta fatia é só a UI — **zero foundational,
zero contrato**.

## 2. Pesquisa: qual modelo de "grupo"?

Há dois modelos na indústria, e a escolha muda o documento:

| | **Nuke** (BackdropNode) / **Houdini** (network box) | **Blender** (Frame node) |
|---|---|---|
| Posse | **Geométrica**: o backdrop não guarda filhos; o que ele carrega é decidido **na hora do arrasto** (quem está dentro) | **Parentesco**: os nós viram FILHOS do frame no documento |
| Custo | zero estado extra; nada para dessincronizar | precisa manter `parent` em todo nó, re-parentear no drop, e o doc pode discordar do que o olho vê |

**Escolhido: o modelo do Nuke** — e não por gosto: o nosso `Backdrop` **já é** `{id, rect, color,
title}` e nada mais. Adotar Blender exigiria um campo `parent` em cada nó (mudança de documento) para
resolver um problema que a geometria já resolve. *A decisão foi tomada pelo dado que já existia.*

**Consequência boa:** não existe "re-parentear". Arrastou o header → carrega quem estiver dentro
**agora**. Redimensionou → o grupo muda de conteúdo sozinho.

## 3. As duas decisões que fazem a feature funcionar (ou não)

### 3.1 O corpo é CLICK-THROUGH (a que mata a feature se errada)

Só o **header** e o **gripper** registram hit rect. O **corpo não registra nada**. Se o corpo
capturasse cliques, todo nó que o backdrop emoldura ficaria **inselecionável** e o box-select morreria
por cima dele — a ferramenta de agrupar tornaria o grafo *menos* usável. É o bug clássico, e tem gate
executável: `the_backdrop_body_registers_no_hit_rect` (o centro do corpo não é coberto por nenhum
rect; exatamente 2 rects registrados).

Ordem de hit (último vence, como o resto do painel):
`background → header/gripper do backdrop → wires → nós → sockets`.
Logo um nó por cima do backdrop **ganha** o clique, e o header **perde** para um wire que passe nele.

### 3.2 O arrasto carrega quem ele emoldura — capturado no GRAB

Arrastar o header emite, no mesmo frame, `MoveBackdrop` **+** `MoveNodes` (reusando o intent que já
existia), tudo dentro de UM `BeginDrag`/`EndDrag` → **um passo de undo** para o grupo inteiro.

O conjunto emoldurado é capturado **no Begin**, não re-testado por frame: um nó que ficasse na borda
entraria/sairia do grupo no meio do gesto. Contenção = **CENTRO** do card dentro da região (não
"totalmente dentro"): um card com um canto para fora pertence ao grupo que o olho vê.

## 4. Superfície nova (toda aditiva)

**Painel `ph2d-panel-motion-graph`** (módulo irmão novo `backdrop.rs` — o `paint.rs` estava a 528/600):
- `GraphBackdropView` no `GraphViewSnapshot` (campo `backdrops`).
- 6 intents novos: `AddBackdrop` · `MoveBackdrop` · `ResizeBackdrop` · `DeleteBackdrop` ·
  `SetBackdropTitle` · `SetBackdropColor`.
- Canal de seleção `set/current_graph_backdrop_selection` — **mutuamente exclusivo** com a seleção de
  nós (o painel de params mostra UM sujeito, e o Delete nunca fica ambíguo).
- Chip novo na toolbar: `CHROME_BACKDROP = 3` (`IconId::Backdrop`).
- Consts públicas `BACKDROP_MIN_W/MIN_H` — o shell aplica o clamp do resize com **o número do painel**,
  não com uma cópia que poderia derivar dele.
- `Interaction::{DragBackdrop, ResizeBackdrop}`; `MotionGraphPanelState.selected_backdrop`.

**Shell** (módulo irmão novo `motion_bridge_backdrops.rs` — o `motion_bridge_params.rs` bateu 622/600 e
foi **dividido**, não allowlistado):
- aplica os 6 intents no doc; publica `doc.backdrops` no snapshot.
- rows do painel de params para um backdrop selecionado: **Title** (`TextRow`) + **Color** (`EnumRow`
  de 8). **O painel de params não mudou uma linha** — reusar o vocabulário de rows que já existia
  (o mesmo `TextRow` da fórmula do `motion.expression`) foi o pagamento.
- `MotionCookPump::is_dirty()` (getter novo, aditivo) — só para tornar a regra abaixo **executável**.

**Gestos:** chip = criar (envolvendo a seleção, senão bloco default no centro) · header = selecionar +
arrastar o grupo · **qualquer um dos 2 cantos de baixo** = redimensionar · Delete = apagar a região
(**os nós ficam**) · Esc = limpar.

### 4.1 Correções do smoke do Enio (2026-07-12)

1. **Toolbar do grafo foi do topo-esquerdo para o CANTO INFERIOR ESQUERDO.** No topo ela **embolava
   com o rail de botões do editor** (undo/redo) — duas toolbars empilhadas na mesma quina. O rodapé é
   a única quina permanentemente livre do grafo: o topo carrega a banda do divisor do split, e a
   direita é onde um grafo panado transborda.
2. **O gripper era chrome INVENTADO** (um quadradinho `Border`) e virou **o do app**: o mesmo
   `paint_panel_corner_dot` + `panel_resize_handle_rect` que todo painel usa — e, como os painéis,
   **nos DOIS cantos de baixo**, redimensionável por qualquer um deles. O canto **não** agarrado fica
   **ancorado**: puxar o da esquerda move `x` e encolhe `w` segurando a borda direita, e o clamp de
   tamanho mínimo ancora **nessa mesma borda** (senão a região inteira sairia arrastada junto com o
   cursor ao passar do mínimo — gate `a_left_corner_resize_holds_the_right_edge`). O canto agarrado
   viaja no handle opaco do `BackdropResize` (mesmo truque do `wire_handle`). Lição:
   [[feedback_ui_source_of_truth_gallery_inspector]] — procure o precedente ANTES de desenhar chrome.

## 5. A regra que um gate protege: **decoração NÃO recozinha**

Backdrop é estado de **documento** (é undoable, e serializa), mas é **UI-only**: nenhuma edição dele
chama `mark_dirty`. Se chamasse, arrastar um grupo recozinharia o grafo de 71 nós **a cada frame do
gesto**. Gate: `no_backdrop_edit_ever_re_cooks_the_graph` — add/move/resize/rename/re-tint/delete e o
pump segue limpo; **com controle** (um `mark_dirty` de verdade ainda suja, senão a asserção seria vazia).

## 6. Verificação

**Painel (24 testes, 24 verdes)** — os falsificáveis:
- `dragging_a_backdrop_header_carries_the_nodes_it_frames` — compara a **lista exata** de intents:
  `[BeginDrag, MoveBackdrop, MoveNodes{[1]}, EndDrag]`. Vermelho se a região deslizasse sozinha (o
  backdrop sai de baixo do grupo) ou se varresse o nó de fora.
- `the_backdrop_body_registers_no_hit_rect` — o click-through (§3.1).
- `dragging_the_gripper_resizes_without_moving_the_nodes` · `the_backdrop_chip_wraps_the_selection` ·
  `delete_with_a_backdrop_selected_removes_only_the_backdrop` ·
  `selecting_a_node_clears_the_backdrop_selection`.

**Shell (8 testes)** — `no_backdrop_edit_ever_re_cooks_the_graph` (§5) ·
`deleting_a_backdrop_leaves_every_node_standing` · `a_resize_clamps_to_the_panels_minimum` ·
`a_backdrop_is_undoable` · `a_selected_backdrop_yields_its_title_and_colour_rows` (a UI de rename
existe — sem ela o grupo nasceria "Group" para sempre) · `a_selected_node_still_wins_the_params_panel` ·
**`an_authored_backdrop_survives_a_text_round_trip`** — a seção `[backdrop]` existia desde o M0 **sem
produtor**; esta é a **primeira vez** que um backdrop autorado na UI é provado sobreviver a save/load.

**Um buraco honesto:** o roteamento `GraphIntent → backdrops::*` no `apply_graph_intents` é garantido
pelo **match exaustivo** (o compilador barra um variant não tratado), mas nenhum teste injeta intents
pelo canal real — `push_intent` é `pub(crate)` do painel. É o que o **smoke** cobre.

## 7. Smoke (o Enio roda)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && cargo run -p ph2d-host-desktop
```
Na tool Motion: selecione alguns nós → **chip Backdrop** (4º da toolbar) → a região nasce **envolvendo
a seleção**, nomeada e tintada. Arraste o **header**: o grupo inteiro viaja (um Ctrl+Z desfaz tudo).
Arraste o **canto**: só a região cresce. Clique num nó POR CIMA do backdrop: ele é selecionado
(corpo click-through). Com o backdrop selecionado, o painel de params mostra **Title** + **Color**.

## 8. Aberto (o resto do F2)

**Duplicate (Ctrl+D)** · **knife** (cortar fios) · **probe + sparkline** · **smart-connect popup** ·
waypoints/branches · readouts inline. As teclas `GraphKey::{Duplicate, Knife, Probe}` **já existem** no
editor-core — é a mesma história dos backdrops: trilho posto, trem faltando.
