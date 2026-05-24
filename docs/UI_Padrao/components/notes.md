# Notes + section outlines

**Status:** canon de posicionamento + Hierarchy-exclusion definido 2026-05-24. Broadcast para os outros painéis Inspector-type pendente — vide §"Pendências".
**Fontes vivas:** [Widget Gallery](../../../crates/ph2d-panel-widget-gallery/) (Showcase: NOTES section faz isso correto), [Inspector](../../../crates/ph2d-panel-inspector/) (corrigido 2026-05-24).
**Implementação:** [`WidgetStore::notes_for_panel`](../../../crates/ph2d-editor-core/src/interaction/state/mod.rs) + [`paint_one_note`](../../../crates/ph2d-editor-core/src/widget/showcase/mod.rs) + macro `live_section!` em cada panel orquestrador.

---

## O que são

**Notas** = post-its 5-cores anchoradas a seções de um painel. Criadas via right-click → "Create note". O usuário escreve título + corpo; persistem no `WidgetStore`.

**Section outlines** = highlight retangular colorido envolvendo uma seção inteira. Criados via right-click no header da seção → "Section outline" → escolhe uma das 5 cores. Visual: stroke `Thick` em volta do bloco da seção.

Os dois compartilham a paleta [`HIGHLIGHTER_RGBA`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs) (5 cores: yellow / pink / green / blue / orange).

## Onde aparecem (escopo canon)

| Painel | Notas? | Outlines? |
|---|---|---|
| Widget Gallery | ✅ | ✅ |
| Inspector | ✅ | ✅ |
| BgRemoval, Padding, CEQ, Upscale, Equalize Sizes, Grid Snap | ⏳ pendente (vide §Pendências) | ⏳ pendente |
| **Hierarchy** | ❌ **NÃO** | ❌ **NÃO** |

**Hierarchy é exceção declarada:** rows são entidades, não parágrafos. Right-click vai pra `HierarchyRow` menu (Duplicate / Delete / Reset / Add Child); right-click em espaço vazio dentro do painel **não** abre menu (pré-2026-05-24 abria "Create note" spuriamente — corrigido em [`dispatch/pointer.rs`](../../../crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs) com filter `panel != HIER_PANEL`).

## Onde a nota aparece (posição visual)

**Canon:** nota é pintada **no FIM da seção** onde o usuário clicou, **ANTES do separador**.

```
┌─────────────────┐
│ SECTION HEADER  │
├─────────────────┤
│ param 1         │
│ param 2         │
│ param 3         │
│ ┌─────────────┐ │  ← nota anchorada a esta seção
│ │ Note title  │ │
│ │ note body   │ │
│ └─────────────┘ │
├─────────────────┤  ← separador (depois das notas, fim da seção)
│ NEXT SECTION    │
```

Não:

```
│ param 3         │
├─────────────────┤  ← separador
│ ┌─────────────┐ │  ← nota DEPOIS do separador (pre-canon, ERRADO)
│ │ Note title  │ │
```

A nota pertence visualmente à seção que o usuário clicou — agrupada com seus params, antes da linha que fecha aquela seção. Pre-canon (Inspector + Showcase) a nota era pintada ANTES do header da seção anchor, o que efetivamente colocava ela DEPOIS do separador da seção anterior.

## Modelo de dados

```rust
NoteData {
    color_idx: u8,                   // 0..4 (HIGHLIGHTER_RGBA)
    title: String,                   // single line
    body: String,                    // multi-line
    before_section: Option<u8>,      // anchor: índice da seção (mesma escala que SECTION_IDS do painel)
}
```

`before_section`:
- `Some(i)` → nota anchorada à seção `i` (pinta dentro dela, no fim, antes do separador).
- `None` → nota trailing (pinta no fim do painel, depois de todas as seções).

Nome do field é histórico (era "before this section" = pinta antes do header). Semântica atualizada — agora é "ENDS this section". Renomear pra `inside_section` é follow-up.

## Right-click resolution (dispatch/pointer.rs)

Quando o usuário right-click dentro de um painel, o dispatch resolve em ordem:

1. Hierarchy row → menu `HierarchyRow` (Duplicate, Delete, etc.)
2. Em cima de uma nota existente → menu `NoteBackground` (5 cores de fundo)
3. Em cima de um header de seção → menu `SectionOutline` (5 cores de outline)
4. Em espaço vazio dentro de painel **diferente de Hierarchy** → menu `CreateNote` (single item)
5. Em espaço vazio FORA de painel → nada

A chave do canon: passo 4 explicitamente filtra `HIER_PANEL`.

Quando "Create note" é clicada, `apply_event` calcula `before_section` via [`section_index_below_body_y`](../../../crates/ph2d-editor-core/src/widget/showcase/state.rs) (screen-y do click → body-y → índice da seção que contém aquele y).

## Pendências

- **Broadcast pros outros painéis Inspector-type:** BgRemoval, Padding, Color Equalization, Upscale, Equalize Sizes, Grid Snap não chamam `notes_for_panel` nem renderizam notas. Requer:
  1. Cada painel orquestrador adota o mesmo macro `live_section!` do Inspector.
  2. `pre_populate` (ou panel populate) registra os section ids de cada painel (`SECTION_IDS` por painel).
  3. `section_index_below_body_y` precisa ser per-panel (atualmente usa thread-local — pode haver wiring extra pra distinguir painel ativo).
- **Outline + nota broadcast em Inspector-type.** Análogo ao acima.
- **Renomear `before_section` → `inside_section`** para refletir a nova semântica.

## Checklist quando adicionar notes a um painel novo

- [ ] `let all_notes = store.notes_for_panel(<PANEL_ID>).to_vec();` no topo do paint.
- [ ] Particionar em `notes_per_section[N]` + `trailing_notes`.
- [ ] Macro `live_section!` no orquestrador:
  - Paint section body.
  - Paint optional outline.
  - `paint_one_note` pra cada nota anchorada (notes_per_section[i]).
  - `paint_section_separator`.
- [ ] Trailing notes paintadas no fim do painel.
- [ ] `pre_populate` (ou panel populate) declara os section ids do painel.
- [ ] Right-click handler garante panel != HIER_PANEL pra CreateNote (já no dispatch global desde 2026-05-24).

## Anti-padrões

- **Pintar nota antes do section header** — pre-canon Inspector errado. Notas vão ao FIM da seção.
- **Pintar separador antes da nota** — separador é o ÚLTIMO elemento da seção, depois das notas.
- **Show CreateNote em Hierarchy** — Hierarchy NÃO tem notas. Dispatch já filtra.
- **Hardcode `panel_under == INSP_PANEL`** em algum dispatch — generalize pra "panel que NÃO é Hierarchy" usando filter.
