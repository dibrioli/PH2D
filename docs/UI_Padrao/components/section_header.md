# Section header

**Status:** canon definido 2026-05-24. Wiring de interação (collapse + color picker click) pendente — vide §"Pendências".
**Fonte viva:** [Widget Gallery](../../../crates/ph2d-panel-widget-gallery/) — INPUTS, SLIDER, SWITCHES, LISTS são exemplos canônicos.
**Implementação:** [`crates/ph2d-editor-core/src/widget/section_header.rs`](../../../crates/ph2d-editor-core/src/widget/section_header.rs) (`SectionHeader` + `paint_section_header`).

---

## Anatomia

```
▼  TÍTULO EM MAIÚSCULAS                                    ●
└chev      └label                                      └color dot
```

| Elemento | Onde | Notas |
|---|---|---|
| Chevron `▼/▶` | esquerda | Sempre pintado. `▼` = aberto, `▶` = fechado. Click no chevron OU no header inteiro = toggle. |
| Título | centro-esquerda | **ALL CAPS** sempre. Font `TypeToken::Sm`, cor `Text1`. `String` é guardada case-original; uppercase só no paint via `.to_uppercase()`. |
| Color dot | direita | Círculo `7 px` raio, fill = RGBA escolhido pelo usuário, ring de 1 px `Border` pra contraste. Click abre color picker da seção. Quando ausente, fallback é o count chip legado. |

## Regras absolutas (canon)

1. **TODA seção usa `paint_section_header`.** NÃO chame `paint_text_title` para títulos de seção. Inspector errava (pintava título via `paint_text_title`); corrigido 2026-05-24.
2. **TODA seção é colapsável.** `SectionHeader::collapsible(true)` (ou `false` quando inicialmente fechada). Chevron sempre presente.
3. **Título em MAIÚSCULAS.** O painter cuida da conversão — você passa o label em case normal ("Transform"), ele pinta "TRANSFORM".
4. **Color dot opcional, mas SLOT sempre reservado.** Quando o usuário não escolheu cor, slot fica vazio (afordância: ainda clicável pra abrir picker — wiring pendente, vide §Pendências).
5. **NUNCA pinte separador entre header e parâmetros.** Separador é coisa do ORQUESTRADOR do painel, entre uma seção e a próxima (vide §Separator abaixo). Pré-2026-05-24 Inspector pintava separador interno errado.

## Separator (linha horizontal entre seções)

Separador é **entre** seções, não dentro. Painted por [`paint_section_separator`](../../../crates/ph2d-editor-core/src/widget/showcase/mod.rs) após o conteúdo da seção (e após as notas anchoradas àquela seção — vide [`notes.md`](notes.md)). Ordem canônica do orquestrador:

```
section! { paint_section_X(); outline_if_any(); paint_pending_notes!(); paint_section_separator(); }
```

Notas vão DENTRO da seção (no fim), separador depois das notas. Pré-canon, notas iam ANTES do header da próxima seção — visual errado.

## Estrutura de dados

```rust
SectionHeader {
    id: NodeId,                          // pra hit-test (clique = collapse + color picker)
    label: String,                       // case normal; painter faz uppercase
    count: Option<u32>,                  // legacy chip (fallback quando color é None)
    collapsible: Option<bool>,           // Some(true)=aberto, Some(false)=fechado, None=não-colapsável
    color: Option<[u8; 4]>,              // RGBA do color dot
}
```

`SectionHeader::is_open()` retorna `true` quando `collapsible == None || Some(true)`. Painter usa pra escolher chevron + fundo:
- aberto: fundo transparente, chevron `▼`
- fechado: fundo `Bg3` tinted, chevron `▶`

## Pendências (wiring de interação)

Pintura está canônica. O que falta:

- **Click no chevron / header → toggle collapsible.** Dispatch ainda não tem handler. Cada painel orquestrador precisa: hit-register o header rect com `$section_id`, drenar click via `apply_event`, mutar estado de visibility da seção.
- **Click no color dot → abrir color picker.** [`color_circle_hit_rect`](../../../crates/ph2d-editor-core/src/widget/section_header.rs) já expõe o rect; falta dispatch wiring + handler que abre BlenderColorPicker associado.
- **Persistir collapse/color state.** Provavelmente em `WidgetStore::section_outline_color` (já existe pra outline cor) — análogo `section_color_dot` + `section_collapsed`.

Implementação prevista pra checkpoint subsequente.

## Checklist quando criar seção nova

- [ ] `SectionHeader::new($section_id, "Label Em Case Normal")`.
- [ ] `.collapsible(true)` (sempre — toda seção é colapsável).
- [ ] `.color(rgba)` se já tem cor padrão; senão omitir.
- [ ] `paint_section_header(&header, Rect::new(x, y, w_minus_actions, header_h), ...)`.
- [ ] NÃO pinte separador dentro do `paint_X_section` — orquestrador cuida.
- [ ] hit_register o `$section_id` no rect do header (pra collapse handler futuro).

## Anti-padrões

- `paint_text_title("MyTitle", ..., TypeToken::Md, ...)` — pinta título do JEITO ERRADO. Use `paint_section_header`.
- `paint_section_separator()` dentro de `paint_X_section` — pinta separador entre header e params. Remova; orquestrador cuida.
- Hardcode `label.to_uppercase()` no call-site — painter já faz.
- Esquecer `collapsible(true)` — header não mostra chevron, regride a "título estático".
