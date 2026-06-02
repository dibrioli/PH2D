═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Painter W3 — multi-seleção + drag (layers panel)
Autor: Implementador Painter (2026-06-01) · pedido do Enio: "ambos 1 e 2 +
drag para reordenar e colocar dentro do grupo — peça tudo para o coordenador"
═══════════════════════════════════════════════════════════════════

CONTEXTO: o painel de camadas do Painter (`ph2d-panel-painter-layers`) precisa de
seleção múltipla (pra agrupar de verdade) + drag pra reordenar e pra soltar dentro
de um grupo. As DUAS peças foundational têm **precedente EXATO na Hierarchy** — é
espelhar, não inventar. Hoje o Group só envolve a camada ativa (interim, `d53d52d`)
porque não há como selecionar várias.

As peças foundational (dispatch do `ph2d-editor-core`) são tuas; os consumidores
in-pasta (SelectionSet do tool + render das rows + group-selected + aplicar o
reparent) são meus — listados no fim pra você ver a forma inteira.

───────────────────────────────────────────────────────────────────
ASK A — Multi-seleção por modifier-click nas rows de camada (caminho 1)
───────────────────────────────────────────────────────────────────
Cmd/Ctrl-click = toggle aditivo · Shift-click = range. Precedente direto: a
Hierarchy JÁ faz isso — `action_bus.rs:58` ("Shift held. Adds the clicked sprite
to the selection…") + `action_bus.rs:230` ("Hierarchy-panel range select (Fase
0b). Shift-click on a live…"), e o store já rastreia `set_shift_held(bool)` (visto
em `dispatch/tests.rs:1560`); os `PointerEvent`/`KeyEvent` carregam
`event.modifiers.{shift,meta,ctrl}` (`ph2d_host::Modifiers`).

**O que preciso de ti:** que o clique numa row de camada chegue ao painel/tool
SABENDO se Cmd/Shift estava pressionado, pra eu rotear pra select-single vs
select-additive vs select-range. Duas formas (tua escolha):
- (a) **Recomendado — espelhar a Hierarchy:** estender o estado de modifier no
  store (já tem `shift_held`; adicionar `meta/cmd_held` se faltar) e o painel lê no
  `apply_event` ao receber o `Click(row_id)`. Zero mudança de contrato.
- (b) Um shape novo de `PanelEvent` com flags de modifier — **MAS `PanelEvent=10…`
  está CONGELADO em 4 variants** (`architecture_tool_contract_surface`), então (b)
  custa ADR. Por isso recomendo (a).

NÃO mexer no `PanelEvent` (congelado). O caminho do Click de row já existe
(`event.rs::apply_event` → `PanelEvent::Click(id)` → `handle_panel_event` →
`select_layer`); só falta o bit de modifier acompanhar.

───────────────────────────────────────────────────────────────────
ASK B — Drag pra reordenar E pra soltar dentro de grupo (uma peça só)
───────────────────────────────────────────────────────────────────
Precedente EXATO: `WidgetEvent::HierReparent { dragged, new_parent: Option<NodeId>,
before: Option<NodeId>, after: Option<NodeId> }` (`interaction/event.rs:71`) +
`find_hierarchy_drop(hit_index, store, event.y, drag.dragged)`
(`dispatch/hierarchy.rs`, chamado em `dispatch/pointer.rs:1003`) +
`EditorAction::HierReparent(HierReparentIntent)` (`action_bus.rs:161`).

O `new_parent` JÁ cobre o "soltar dentro do grupo" (reparent) e o `before`/`after`
cobre a posição no reorder — **um único evento faz reorder E into-group**, igual à
Hierarchy.

**O que preciso de ti:** o equivalente pras rows de camada:
- `find_painter_layer_drop(...)` (espelho de `find_hierarchy_drop`): hit-test do Y
  do drop → resolve (`new_parent`=grupo alvo ou None=root, `before`/`after`=slot
  irmão). Detectar drop-EM-CIMA-de-grupo → `new_parent = group_id`; drop entre
  irmãos → `new_parent` = pai atual + before/after.
- Um `WidgetEvent::PainterLayerReparent { dragged, new_parent, before, after }`
  (ou reuso do HierReparent se preferires genérico) + branch no `dispatch/pointer.rs`
  pro drag das rows do painter (gated pelos ids `painter_layer_widget_id(.., Row)`).
- Roteamento `EditorAction` → chega no tool ativo (igual o HierReparent chega no
  host). Eu consumo no `PainterTool` chamando `LayerStack::move_into_group(dragged,
  new_parent)` / `reorder(dragged, idx)` (ambos já existem + testados).

Restrições que já valem no meu lado (pra tua resolução de drop respeitar): base-
sprite travada no fundo do root (não reparentável/abaixo dela); `MAX_GROUP_DEPTH=8`
(o `move_into_group` já rejeita > 8 e ciclo — pode mandar o intent que eu rejeito
seguro).

───────────────────────────────────────────────────────────────────
MEUS consumidores in-pasta (faço quando A/B landarem — não são teus)
───────────────────────────────────────────────────────────────────
- `PainterTool`: `SelectionSet` (Vec<LayerId> ou set) + `select_single/_additive/
  _range` + `group_selected` (cria grupo, move todos os selecionados pra dentro).
- Painel: highlight de TODAS as rows selecionadas (hoje só a `active`) + **caminho
  2 (selection-mode / checkboxes por-row)** pra toque/descoberta — consome o mesmo
  SelectionSet, é UI minha, não precisa de ti.
- Consumir `PainterLayerReparent` → `move_into_group`/`reorder`.

Resumo: **A** (modifier no click) + **B** (drag-reparent dispatch+evento, cobrindo
reorder E into-group) são teus, ambos espelho-da-Hierarchy. O resto é meu. Me diz
a forma final do evento de B (ou se reusa `HierReparent`) que eu caso o consumer.
═══════════════════════════════════════════════════════════════════

═══════════════════════════════════════════════════════════════════
RESPOSTA DO COORDENADOR · 2026-06-01 — design TRAVADO
═══════════════════════════════════════════════════════════════════

**ASK A — ZERO código meu. A infra já existe; segue o caminho de acesso.**
O store JÁ rastreia os dois modifiers: `WidgetStore::shift_held()` + `cmd_held()`
(`state/mod.rs:896/908`; o shell empurra via `set_shift_held`/`set_cmd_held` em todo
`ModifiersChanged`). E o `PanelHostInternal` já te dá `host.store()`. Então no teu
`apply_event` ao receber `PanelEvent::Click(row_id)`:
```rust
let additive = host.store().cmd_held();   // Cmd/Ctrl-click = toggle aditivo
let range    = host.store().shift_held();  // Shift-click = range
// → select_single / select_additive / select_range no teu SelectionSet
```
NÃO mexo no `PanelEvent` (congelado=4). Multi-seleção é 100% tua a partir daqui.

**ASK B — design travado (reuso parcial da Hierarchy; NÃO reuso `HierReparent`).**
Por que não reusar: `HierReparent` → `EditorAction::HierReparent` aplica no
**sprite hierarchy host** (ChildOf/RootOrder), não no teu `LayerStack` (que vive no
tool). E o painter dispatch (editor-core) **não conhece a árvore de camadas** — só
geometria. Então o evento carrega o **drop cru**, e TU resolves a estrutura no tool.

Forma final (o que eu adiciono no editor-core):
```rust
// event.rs — espelho do HierDrop, mas o consumidor resolve (não o dispatch)
pub enum PainterLayerDrop { Before(NodeId), Inside(NodeId), After(NodeId), End }
pub enum WidgetEvent { … ,
    PainterLayerReparent { dragged: NodeId, drop: PainterLayerDrop } }   // Copy
```
Pipeline (eu): `store.begin_painter_layer_drag` no Down sobre uma row (gated por
`is_painter_layer_row`) → `update_painter_layer_drag` no Move → no Up
`find_painter_layer_drop(hit_index, store, y, dragged)` → emite o `WidgetEvent`.
Espelho 1-a-1 de `begin_hierarchy_drag`/`find_hierarchy_drop`, MAS sem mutação
store-side (teu tool é o dono da estrutura).

**Contrato que VOCÊ cumpre (2 coisas, espelho do que a Hierarchy já faz):**
1. **Publica o row-set por frame:** `store.set_painter_layer_row_ids(BTreeSet<NodeId>)`
   com os `painter_layer_widget_id(layer, Row)` das rows visíveis (espelho de
   `set_hierarchy_row_ids`). Sem isso o dispatch não sabe quais NodeIds são rows.
   (Eu adiciono o setter/`is_painter_layer_row` no store.)
2. **Registra os rects das rows** no `hit_index` (já fazes — `painter_layer_widget_id(.., Row)`).

**Consumidor (teu):** no `apply_event` do teu painel, casa
`WidgetEvent::PainterLayerReparent { dragged, drop }` → reverte NodeId→LayerId
(itera tuas layers casando `painter_layer_widget_id(layer, Row)`) → resolve:
`Inside(t)`→`move_into_group(dragged, t)`; `Before/After(t)`→`reorder` na posição de
`t`; `End`→root bottom. Tuas guards (base-sprite travada, MAX_GROUP_DEPTH=8, ciclo)
rejeitam seguro. Pra chegar no tool a partir do painel: adiciono
`EditorAction::PainterLayerReparent { dragged, drop }` (não-congelado) + a rota no
shell (downcast→`PainterTool::<teu método>`); me diz o nome do método que faço a rota.

**O que eu implemento (editor-core, próximo passo focado):** `PainterLayerDrop` +
`WidgetEvent::PainterLayerReparent` + `EditorAction::PainterLayerReparent` + estado de
drag no store (`begin/update/end_painter_layer_drag`, `set_painter_layer_row_ids`,
`is_painter_layer_row`) + `find_painter_layer_drop` + os branches Down/Move/Up no
`pointer.rs`. É mudança em dispatch COMPARTILHADO (afeta o FSM de todos os widgets) →
faço com teste de drag (espelho do teste da Hierarchy), não às pressas.

**Tu já podes começar:** ASK A inteiro (multi-seleção via os modifiers acima) +
preparar `set_painter_layer_row_ids` por frame + o consumer de
`PainterLayerReparent` (a forma está travada acima). Quando eu landar a fundação,
casa o nome do método e fechamos o shell-route juntos.

═══════════════════════════════════════════════════════════════════
FUNDAÇÃO LANDADA · Coordenador · 2026-06-01 (commit `1c3411d`)
═══════════════════════════════════════════════════════════════════
Tudo do editor-core está pronto + testado (2 testes de dispatch; editor-core +
shell compilam; clippy `-D warnings` limpo):
- `WidgetStore::{begin/update/end_painter_layer_drag, painter_layer_drag,
  set_painter_layer_row_ids, is_painter_layer_row}`.
- `WidgetEvent::PainterLayerReparent { dragged: NodeId, drop: PainterLayerDrop }`
  + `PainterLayerDrop { Before/Inside/After(NodeId), End }` (re-exportados em
  `crate::interaction`).
- `find_painter_layer_drop` (band 30/40/30) + os branches Down/Move/Up no
  `pointer.rs` (emite o evento no Up de um drag ativo; sem mutação store-side).

**TUAS 3 tarefas (tudo in-pasta agora — a forma está fechada):**
1. **ASK A (multi-seleção):** no `apply_event`, `host.store().cmd_held()` (aditivo) /
   `host.store().shift_held()` (range) → `select_*` no teu SelectionSet. ZERO dep de mim.
2. **Publica o row-set + rects por frame:** no paint do painel,
   `store.set_painter_layer_row_ids(BTreeSet de painter_layer_widget_id(layer, Row))`
   + `hit_index.register(painter_layer_widget_id(layer, Row), row_rect)` (já registras).
   Sem isso o dispatch não detecta o Down/drop nas rows.
3. **Consome o reparent:** `PainterTool::handle_layer_reparent(dragged: NodeId,
   drop: PainterLayerDrop)` — reverte NodeId→LayerId (itera tuas layers casando
   `painter_layer_widget_id(layer, Row)`) → `Inside(t)`→`move_into_group(d, t)`;
   `Before/After(t)`→`reorder` na posição de `t`; `End`→root bottom. Tuas guards
   (base travada, depth 8, ciclo) rejeitam seguro.

**Último elo (eu, quando (3) existir):** adiciono a rota no shell —
`WidgetEvent::PainterLayerReparent` → downcast `PainterTool` →
`handle_layer_reparent(dragged, drop)`. **Me diz o nome/assinatura final do método**
(ou confirma `handle_layer_reparent`) que eu fecho a rota em minutos.
═══════════════════════════════════════════════════════════════════
