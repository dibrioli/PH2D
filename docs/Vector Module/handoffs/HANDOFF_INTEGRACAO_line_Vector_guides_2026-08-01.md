# Handoff de integração — `line/Vector`: as GUIAS e a RÉGUA (W6.2) + o MIRROR (W6.3)

**Status:** FECHADO 2026-08-01 · no `main` em `f9346fad5` (o commit que trouxe este arquivo).

> **2026-08-01.** Fecha o **único item `G`** da tabela do plano 25 §9 (W6.2) e o **mirror
> vivo** (W6.3). Nove commits sobre
> `3197c5c9e` (o `main` de 2026-08-01, já com a integração anterior desta linha).
>
> ⚠️ **A linha NÃO integra e NÃO pusha.** Este documento existe para o agente integrador, sob
> ordem explícita do Enio (CLAUDE.md §0.7).

---

## §1 — O que entra

**Uma wave, quatro peças:** o modelo, a 5ª espécie de alvo do snap, o desenho (guias + régua) e
o gesto. Mais a UI (duas linhas na seção Snap) e o arquivo.

| commit | o quê |
|---|---|
| `0d02b2c83` | O MOTOR: crate `ph2d-guides`, a espécie `Guide` no snap, `draw_document_guides`, o módulo `ruler` |
| `db5ddaa0e` | O PRODUTO: o gesto, as duas linhas de UI, `ProjectState.guides`, `PROJECT_SCHEMA` 48→49 |
| `7fa42c1d9` | A cena de smoke `=45` e o plano 25 §9 documentado |
| `0ede804b7` | ⚠️ **O 1º fix de auditoria** — a régua vive com a ferramenta Vector, por UMA porta (§4.4) |
| `df9a58bb3` | ⚠️ **O FIX do 1º smoke** — o movimento estava ligado no handler que não entrega movimento (§8.3) |
| `9802b7be0` | **O MIRROR** (W6.3): a simetria VIVA — `PathEffect::Mirror`, `MAX_FX_KINDS` 21→22, o split de `paint.rs` |

---

## §2 — Os números que a integração tem de CONTAR, não copiar

| | esta linha escreveu | o que o integrador faz |
|---|---|---|
| **`PROJECT_SCHEMA`** | **48 → 49** | ⚠️ **CONTAR contra o `main` do dia.** Se outra linha na mesma janela também bumpou, o valor certo não está em nenhum dos dois lados |
| `VEC_SCENE_SCHEMA` | **13, intocado** | conferir por `git diff` |
| registro do `ph2d-ecs` | **intocado** | nenhum componente novo — uma guia **não** é entidade |
| ADR | **nenhum** | a linha fica fora de toda disputa de número desta janela |
| contrato congelado | **intacto** | rodado, não auto-relatado: `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` verdes |
| `MAX_FX_KINDS` (W6.3) | **21 → 22** | o teto do menu Add; **conferir contra o `main`** — se outra linha acrescentou um efeito, o valor se CONTA |

⚠️ **A armadilha que a `line/FLIP` documentou em 2026-08-01 vale aqui e é MUDA:** se outra linha
escrever o **mesmo** literal 49 no `project.rs`, o git **não conflita** — os dois lados escreveram
o mesmo texto, e ele não tem opinião sobre o que o número SIGNIFICA. O bump de uma das duas
evaporaria com a suíte inteira verde. **O sinal é o conflito do `project_schema_tests.rs` ao
lado** (a tripla `(49, 13, 13)`), e o valor se CONTA a partir do `main`.

---

## §3 — Superfície nova (tudo aditivo)

**Crate nova `ph2d-guides`** (leaf, só `serde` + `postcard` em dev-dep):
`GuideAxis::{Horizontal,Vertical}` · `GuideAxis::locked_axis()` · `Guide{axis,pos}` com
`horizontal`/`vertical`/`distance_to` · `GuideSet` com `len`/`is_empty`/`iter`/`get`/`push`/
`remove`/`set_pos`/`clear`/`nearest`.

**`ph2d-vec-edit::snap`** — `SnapConfig.to_guides: bool` (default `true`) · `SnapTargets.guides:
Vec<Guide>` · `SnapSource::Guide`.

**`ph2d-vec-render`** — `GuideKind::GuideHit` (5ª marca) · `draw_document_guides`.

**`ph2d-editor-core::ruler`** (módulo novo) — `RULER_PX` · `RulerAxis::{Top,Left}` +
`spawns()` · `top_band`/`left_band`/`hit` · `label_step`/`label_text` · `world_per_px` ·
`world_at` · `Tick` + `ticks` · `paint_rulers`.
Mais `HeroScreen.last_canvas` e `ViewState.rulers_visible`.
E, **dentro** de `grid.rs`, `world_bounds`/`world_to_screen_x`/`_y` passaram a `pub(crate)` e
ganharam as inversas `screen_to_world_x`/`_y`.

**`ph2d-panel-vector`** — `set_current_guides(snap, rulers)`.

**ids novos** (4, no irmão `ids/chrome/vector_snap.rs`, bloco append-only):
`VECTOR_SNAP_GUIDES_OFF`/`_ON` · `VECTOR_RULERS_OFF`/`_ON`.

**Deps novas (arestas para a crate leaf, nada de terceiro):** `ph2d-guides` entra em
`ph2d-vec-edit`, `ph2d-vec-render`, `ph2d-editor-core` e na shell. **Nenhuma dep externa.**

---

## §4 — Os pontos de merge sensíveis

1. **`ProjectState`** (`shells/desktop/src/undo.rs`) ganhou um 4º campo, e `capture` um 4º
   argumento. Cinco fixtures de teste foram atualizadas. Se outra linha tocou o mesmo struct, o
   conflito é textual e a resolução é manter os dois campos.
2. **`ViewState`** (`hero.rs`) ganhou `rulers_visible`. Literal de construção atualizado.
3. **`input_dispatch.rs`** ganhou um bloco de despacho **no topo da cadeia**. A ORDEM é a
   afirmação — há arch-gate (`the_guide_gesture_runs_before_any_tool_claims_the_pointer`) que a
   pina contra o canvas do Flip, o arrasto de joint e os dois pickers.
4. **`hero/paint.rs`** ganhou o `hero.last_canvas = layout.canvas;` e a chamada da régua. O
   arch-gate `the_ruler_is_painted_with_the_canvas_the_layout_resolved` pina os três fatos.
   ⚠️ **`HeroScreen::rulers_live()` é a PORTA ÚNICA** (`view.rulers_visible` **e** a ferramenta
   vetorial em mãos), perguntada pelo paint e pelo gesto — a segunda metade é uma correção
   achada auditando a wave: sem ela a faixa comeria o pen-down do **Painter** nos 20 px de cima.
5. **`PathEffect` ganhou um variant no FIM** (`Mirror`) — postcard é posicional, então a ordem
   é a afirmação. Se outra linha também apendou um efeito, os dois vão para o fim e o
   `KINDS`/`from_kind`/`kind_index`/`label` de cada um sobe de índice: **conferir o gate
   `every_effect_kind_is_reachable_from_the_add_table`**, que fecha a volta.
6. **`lib.rs` da `ph2d-vec-scene` foi SPLIT** (`paint.rs`): `Rgba8`/`GradientStop`/`GradientPoint`/
   `Paint` mudaram de arquivo com **re-export na raiz** ⇒ nenhum caminho muda, mas um merge que
   traga edições àqueles tipos vai querer o arquivo novo.
7. **Os dois gates de fiação de snap** (`the_snap_toggles_are_not_crossed` e
   `each_pending_snap_toggle_lands_on_its_own_field`) foram **estendidos de 4 para 8 entradas**.
   Um merge que perdesse as novas deixaria o par novo livre para cruzar.

---

## §5 — A bateria de fechamento (rodada, não auto-relatada)

```
ph2d-guides        7 ok    ph2d-vec-render    30 ok    ph2d-panel-vector  107 ok
ph2d-vec-edit    149 ok    ph2d-editor-core  885 ok    shell            1788 ok
```

- **As TRÊS réguas de LOC** — nenhuma alcançada por um `cargo test -p` filtrado:
  `architecture_workspace_file_loc_cap` (crates, 700) ✅ · `shells/desktop/tests/file_loc_caps`
  (shell, 600) ✅ · `architecture_panel_loc_cap` (painéis, 600) ✅.
- Contratos congelados (4 gates) ✅ · `node_id_collisions` ✅ · `architecture_panel_wiring_parity` ✅.
- `cargo clippy --all-targets` nas seis crates: **limpo**.
- `cargo machete` na crate nova: **sem dep morta**.
- **Debug E release** — a lição da `line/FLIP` (um gate que reprovava só em debug) e a do
  `ph2d-flip-colorize` (um pânico que só o debug via).

**15 mutações, 15 sangram:**

| # | o que muta | quem sangra |
|---|---|---|
| M1 | a guia perde o empate (`<=` → `<`) | `a_guide_wins_a_tie_against_a_shape_point` |
| M2 | sem a cláusula das duas guias | `two_crossing_guides_retire_the_position_claim` |
| M3 | eixos trocados no `locked_axis` | **6 gates**, nas duas crates |
| M4 | o passe das guias ignora o toggle | `the_guide_toggle_governs_only_guides` |
| M5 | `remove` → `swap_remove` | `removing_a_guide_keeps_the_order_of_the_others` |
| M6 | o press usa a coordenada da PRÓPRIA régua | os dois gates de spawn |
| M7 | o *lock* some (o plano ignora as réguas) | `with_the_rulers_hidden_nothing_is_grabbable` |
| M8 | soltar sobre a régua não apaga | `releasing_over_a_ruler_deletes_…` |
| M9 | a régua recebe o canvas de FACHADA | `the_ruler_is_painted_with_the_canvas_the_layout_resolved` |
| M10 | o press de guia cai depois do Flip | `the_guide_gesture_runs_before_any_tool_claims_the_pointer` |
| M11 | a porta esquece a ferramenta (a régua volta a comer o pen-down do Painter) | `the_rulers_are_live_only_with_the_vector_tool_and_the_toggle_on` |
| M12 | o gesto recompõe a condição em vez de perguntar a porta | `the_paint_and_the_gesture_ask_the_same_door_about_the_rulers` |
| M13 | o movimento perde o chamador | `each_phase_of_the_guide_drag_is_wired_to_the_door_that_delivers_it` |
| M14 | **o defeito do 1º smoke, verbatim** (o braço de Move de volta ao `on_mouse_input`) | idem — e **só ele**: os outros 14 gates do arquivo passam |
| M15 | a guia anda depois do traço do Painter | idem (a metade de ORDEM) |

---

## §6 — O smoke

**`env PH2D_BUILD_SMOKE=45 cargo run -p ph2d-host-desktop --release`**

A cena **imprime o que montou** e o roteiro de 7 passos. ⚠️ Se a linha `[guides] cena montada:`
não aparecer, **pare** — o resto não significa nada.

Julgar: criar (arrastar da faixa) · mover · **apagar arrastando de volta** · o **ímã** (a guia
vence o vértice a partir de `x=0,970`, medido) · o **lock** (desligar `Rulers` deixa as guias
visíveis e magnéticas e imóveis) · o **zero** (a régua conta da origem da grade) · o **arquivo**
(Ctrl+S / Ctrl+O).

---

## §7 — Aberto, com o preço nomeado

- **Guia INCLINADA** — é uma **terceira espécie** de reivindicação (a *linha*: 1-D que não se
  decompõe em eixos), com gesto e matemática próprios. Não é um campo a mais.
- **Origem móvel do zero** — a régua já lê a origem da GRADE (porta única), mas `GridSnapState`
  **não é persistido**: uma origem movida seria um ajuste que ESQUECE. Fazê-la direito é levar o
  estado da grade ao arquivo — decisão sobre uma struct foundational com 9 configs.
- **O consumo é do Vector** — o gesto já é tool-agnóstico, o ímã ainda não: quem encaixa nas
  guias é o motor de snap vetorial. Levá-las ao gizmo de sprite é wave própria.
- **Restam na W6** (a ordem da tabela §9): **mirror vivo** · **rótulo de distância** nos smart
  guides.

---

## §8 — Erros de processo desta sessão, registrados

1. **Crase em `git commit -m` executou** (`press_plan`, `hit_plan`, `gfx`… viraram substituição
   de comando e a mensagem saiu mutilada). Corrigido por `--amend -F <arquivo>`. É exatamente o
   perigo que a memória `feedback_backticks_in_commit_message_are_command_substitution` descreve
   — **toda mensagem com crase vai por `-F`**.
2. **Dois gates meus nasceram errados, os dois reprovando código correto** (a escada 1/2/5
   enumerada em vez de construída; um arch-gate ancorado na *definição* de um `fn` em vez da
   *chamada*). Estão descritos no plano 25 §9 e nos próprios doc-comments.
3. ⚠️ **E o 1º smoke REPROVOU o gesto**, por um defeito que os 36 gates não podiam ver: o braço
   `PointerKind::Move` nasceu dentro do `on_mouse_input`, que **só produz `Down` e `Up`** — o
   braço era **estruturalmente inalcançável** e `guide_pointer_move` ficou **sem chamador
   nenhum**. Quem entrega movimento é o `on_cursor_moved`.
   - **Um defeito, os dois sintomas do report:** a guia nascia sob o cursor e ficava lá (*"cria
     a linha mas ela não segue o mouse"*), e pegar uma guia posta armava um arrasto que nunca
     andava (*"mover linha não é possível"*).
   - **Por que nada viu:** os seis gates de política afirmam o que a guia deve fazer *quando
     alguém a move*, e são **cegos a qual porta chamou** — a lição de que *um gate de unidade
     é CEGO à fiação do shell*, aqui na forma mais barata de cometer. O `dead_code` também não
     ajuda: a função **era** chamada, de um braço que nunca roda.
   - **O gate que faltava já existia para outro gesto:** `the_move_advances_the_hand` (W-Grab)
     afirma exatamente isto para a mão da física, com a prosa certa — *"sem isto a mão não
     segue o cursor: ela pega e fica onde estava."* O irmão agora existe para as guias
     (`each_phase_of_the_guide_drag_is_wired_to_the_door_that_delivers_it`), e afirma as **três
     fases contra as duas portas** mais a ordem do movimento.
   - **Higiene junto:** o `Up` deixou de ser gateado por `kind` dentro de um `match` e passou a
     ser uma condição direta, **sem exigir Primary** — o mesmo motivo pelo qual o
     `on_mouse_input` abre soltando a mão da física: *um arrasto que sobrevive ao release fica
     colado no cursor para sempre*, e um botão secundário não é modificador de gesto.
