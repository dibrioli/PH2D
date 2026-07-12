# 33 — Nota-ADR: UI de texto no painel de params (editar a fórmula da expression)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Autorizado por:** Enio ("abre a UI de texto"). **Escopo:** **EDITOR/UI** — fecha o follow-up da fatia 32: a
fórmula da `motion.expression` (um **text param**) agora é editável no **painel de params**, não só via
`set_text_param` no código. **Contratos congelados intocados** (`architecture_contract_surface` 2/1/8). **Zero
mudança no `ph2d-editor-core`** — reusei o widget `TextInput` compartilhado (regra-mãe da UI: espelhar o widget
existente, não improvisar chrome).

---

## 1. O problema

A fatia 32 deu o canal de text param + a `motion.expression`, mas a fórmula só era setável por código
(`Graph::set_text_param`) — sem campo de texto no painel. O `ParamSpec` é f32-only, então o painel (que itera
`manifest.params`) nem via o text param. Faltava a UI.

## 2. A pesquisa (mapear antes de codar — DIRETRIZ §3.B + regra-mãe da UI)

Mapeei a infra de text-input do editor (via Explore + leitura). **Achado: um widget de texto single-line JÁ
existe e é reusável** — não construir um novo:
- **`InteractiveState::TextInput { state, text: String, caret, selection_anchor }`** (o estado editável) +
  **`WidgetStore::text(id)`** (leitura).
- **`TextInput::new(id,label).placeholder(p).state(st)`** + **`paint_text_input_with_buffer(...)`** (o painter).
- O **dispatch global** (`interaction/dispatch/{text_input,key,text_ops}.rs`) já roteia teclado/caret/seleção/
  clipboard pro TextInput focado e dispara `WidgetEvent::{TextChanged, Submit(Enter), Blur, Cancel(Esc)}` —
  **escrevi ZERO código de teclado**.
- **Precedente end-to-end:** o campo de **rename da Hierarchy** (`HIER_RENAME_INPUT`) e a **busca** dela — um
  `TextInput` num painel docado, register→paint→read→commit. Espelhei esse padrão + o padrão das rows numéricas
  Angle/Seed (`mirror_number`/`paint_seed_row`).

## 3. O que foi adicionado (fatia)

**`ph2d-node-registry` (`ui.rs`, aditivo não-congelado):** `ParamWidget::Text` — declara que um `param` (a
chave do text param, ex. `expr`) rende um campo de texto. `min/max/step` inertes.

**`ph2d-node-motion-expression`:** hint `{ param: "expr", label: "Formula", widget: Text }` (primeiro, o
controle primário do nó).

**`ph2d-panel-motion-params`:**
- `snapshot.rs`: `ParamRow::Text(TextRow{name,label,value:String})` + **`MotionParamIntent::SetTextParam{node,
  param,value:String}`** (o `SetParam` é f64-only) + `param_text_id(slot)`.
- `text_rows.rs` (irmão, mold de `number_rows.rs`): `mirror_text` (sincroniza o doc no campo quando não-focado)
  + `paint_text_row` (label + campo full-width via `TextInput`+`paint_text_input_with_buffer`) + `text_value`/
  `text_is_typing`.
- `lib.rs`: registra o `TextInput` por slot em `populate`; pinta a Text row; **commita em `Submit`|`Blur`** →
  lê `store.text` → `SetTextParam`; `any_param_editing` inclui o campo focado (undo de uma sessão).
- **`rows_paint.rs` (NOVO, split):** o laço de pintura das rows saiu de `lib.rs::paint` pro irmão `paint_rows`
  — a fn `paint` estourava o cap de 200 LOC e o arquivo o de 600 com a Text arm. (Gate `architecture_panel_
  loc_cap` verde.)

**`shells/desktop/.../motion_bridge_params.rs`:**
- `build_params_snapshot`: laço novo que itera os hints `ParamWidget::Text` (não são `ParamSpec`) → `TextRow`
  lendo `graph.node_text_param_overrides(nid)`, **primeiro** (a fórmula no topo).
- `apply_param_edits`: o laço de drain virou `match` (SetParam | **SetTextParam** → `set_text_param` +
  `mark_dirty`), na mesma sessão de undo.

**Testes (2 unit/integração novos):** bridge `selected_expression_node_yields_a_formula_text_row` (o nó
expression → Text row com a fórmula do canal, primeiro; falsificável) · panel `text_row_and_set_text_param_
intent_round_trip` (o `SetTextParam` string-carrying round-trips no canal).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco |
|---|---|---|
| `ParamWidget::Text` | `ph2d-node-registry/ui.rs` (aditivo, non-frozen side-table) | variant novo; matches com `_ =>` absorvem |
| `ParamRow::Text`/`TextRow`/`MotionParamIntent::SetTextParam`/`param_text_id`/`text_rows.rs`/`rows_paint.rs` | `ph2d-panel-motion-params` | módulo Motion; baixo |
| Text-row build + `SetTextParam` apply | `shells/.../motion_bridge_params.rs` | módulo Motion; baixo |
| hint `expr` | `ph2d-node-motion-expression` | leaf |

**Zero `ph2d-editor-core` tocado** (TextInput reusado). Contrato de nó intocado (2/1/8). Sem dep nova.

## 5. O que fica

A `motion.expression` é **editável ponta-a-ponta na UI** (clicar no campo → digitar → Enter/clicar-fora
commita → re-cook). Follow-ups: multi-select nos params (uma seleção só hoje) · o campo é single-line (fórmulas
são single-line por contrato). A cauda M1 está **completa e editável**. Fronteiras inalteradas: **M4** · **M5**.
É hora de **integrar** as 18 fatias.
