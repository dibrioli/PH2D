---
name: panel-populate-register
description: Botão novo num panel typed precisa ser registrado em populate() do panel crate OU o dispatcher dropa o click silenciosamente; o paint_section sozinho não basta.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7d5e6481-e38a-41fd-b4ce-ae6413dd4bc6
---

Ao adicionar um botão novo num panel typed (e.g., BgRemoval, Inspector, Hierarchy), o `paint_*` que pinta o botão + chama `hit_index.register(id, rect)` **não é suficiente** — o dispatcher só reconhece o id como clickable se ele tiver entrada no `WidgetStore` via `populate()` do crate do panel.

**Why:** sintoma 2026-05-26 — adicionei `BGR_REMOVE_BRUSH` no `paint_eyedropper_swatches` mas esqueci de adicionar em `populate.rs`. O botão pintava, o hit-test acertava o rect, mas o dispatch tratava como NodeId desconhecido e dropava silenciosamente. O Enio falou "o botão não funciona" sem mais info; achei rápido procurando "register" em populate.rs e vendo que tinha um comentário do `BGR_AUTO_PROTECT_SUBJECT` avisando exatamente isso: "Without this register the dispatcher doesn't recognise the id as a clickable button and the click is silently dropped."

**How to apply:** ao adicionar NodeId novo de botão/toggle:
1. Criar a const no módulo `ids` da crate que a LÊ (o `src/ids.rs` do painel ou da tool). ⚠️ **Desde 2026-09-12 (A5b) os ids NÃO vivem mais em `ph2d-editor-core/src/ids.rs`** — lá (`src/ids/`) só fica o que a própria fundação lê; 1 944 ids desceram para as crates donas.
2. Se outra crate também o lê, ela importa do dono — nunca uma 2.ª cópia do literal.
3. Pintar em `paint_sections.rs` + `hit_index.register(id, rect)`.
4. **Adicionar em `populate.rs`** com `InteractiveState::Button { state: ButtonState::Normal }` no loop principal.
5. Rotear em `event.rs::is_bgr_click` (ou equivalente).
6. Mapear o NodeId → UiEdit em `tool.rs::handle_panel_event`.

Sliders + chips têm seu próprio bloco de register em populate; padrão similar.

**Update 2026-06-19 (cycler "Filter:" inerte, W2.14):** um **cycler/dropdown** do Brush Studio precisa de DOIS registros que o compilador não cobra, e eu errei UM de cada vez por NÃO ter traçado o caminho inteiro antes: (1) `button(store, ids::X)` em `populate.rs` (hit-test reconhecer) **E** (2) `|| id == ids::X` em `is_studio_button` (`event.rs`) — sem o (2) o painel reconhece o clique mas **descarta antes de emitir o `PanelEvent::Click`**. O slider tinha gate (`architecture_studio_slider_wiring`) e passou; o cycler não tinha rede e morreu calado em DOIS sites. **Lição-mãe:** a prosa deste arquivo JÁ listava o event.rs (passo 5) e mesmo assim falhei — *checklist verbal não morde*. Correção definitiva = gate executável novo **`architecture_studio_cycler_wiring`** (ancorado no site de pintura `cycler_row`: "pintou ⟹ wirado"; prova as 3 pernas populate+event+dispatch; **provado mordendo** — fica vermelho ao remover a linha do event.rs). Diagnóstico certo da próxima vez: **grep comparativo do widget que FUNCIONA vs o quebrado** (`RENDERING_MODE` vs `SHAPE_FILTERING`) revela TODAS as lacunas de uma vez. Widget de tipo novo sem gate → escreva o gate junto.

**Update 2026-09-13 (verificado contra o código):** os dois gates do Brush Studio citados acima (`architecture_studio_slider_wiring` / `_cycler_wiring`) **foram apagados junto com o Brush Studio** (ADR-0099) — a lição fica, a rede não. A rede viva para o passo 4 é `hit_indexed_ids_are_registered` + `table_driven_chips_are_registered_too` (`crates/ph2d-editor-core/tests/it/architecture_panel_wiring_parity.rs`): o primeiro só lê a forma `.register(ids::LITERAL, …)`, o irmão cobre os chips guiados por tabela. Os passos 5–6 (roteamento no `event.rs` e o `handle_panel_event`) **seguem sem rede geral** — e mesmo o clique que chega não prova que a escrita chega a um efeito (CLAUDE.md §5.0, os controlos mortos).

Linka com [[feedback-hier-companion-dispatch-allowlist]] (sintoma análogo em hierarchy: precisa em 2 sites de dispatch/pointer.rs) e [[feedback-tool-unit-green-integration-dead]] (unit-verde ≠ vivo no produto).
