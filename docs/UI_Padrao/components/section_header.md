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
| Título | centro-esquerda | **ALL CAPS** sempre. Font `TypeToken::Sm` em **`FontWeight::SEMI_BOLD` (600 = "quase negrito")**, cor `Text1`. Mesmo peso que o título do painel (`paint_panel_title`). `String` é guardada case-original; uppercase só no paint via `.to_uppercase()`. Painter usa `paint_text_title` (não `paint_text`). |
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

## Collapse — wiring completo (2026-05-24)

Click esquerdo em qualquer lugar do header band toggla a seção. Estado persiste em [`WidgetStore::collapsed`](../../../crates/ph2d-editor-core/src/interaction/state/chrome_ops.rs) (já existia pre-canon; o trabalho de 2026-05-24 só plugou no dispatch + orquestrador).

**Fluxo:**

1. `pre_populate` (ou panel `populate`) chama `store.mark_collapsible_section($section_id)` pra cada seção do painel — registra qual id é "section header" elegível pra toggle.
2. Orquestrador (ex: Inspector `live_section!` macro) hit-registra `$section_id` cobrindo o header band.
3. Dispatch (`apply_click`) checa `store.is_collapsible_section(id)` antes do match de InteractiveState — se true, chama `store.toggle_collapsed(id)` + emite `WidgetEvent::Click(id)`.
4. Section painter lê `store.is_collapsed($section_id)`. Passa `!collapsed` pro `SectionHeader::collapsible(...)` (chevron `▼`/`▶`). Se collapsed, retorna depois de pintar só o header — pula o body.
5. `is_focusable` retorna `true` pra qualquer id `is_collapsible_section`, mesmo sem InteractiveState — caso contrário o dispatch nem chegaria a `apply_click`.

## Vertical spacing — single source of truth (canon 2026-05-24)

Toda seção respeita o mesmo ritmo vertical, definido por constantes
em [`widget/panel_chrome.rs`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs):

| Constante | Valor | Onde aplica |
|---|---|---|
| `SECTION_LABEL_TO_CONTROL_PX` | 4 px (Xxs) | Gap entre **label** (ex: "Position (px)") e o **control imediatamente abaixo** (chip row, segmented group, etc.) quando o label está em sua própria linha. |
| `SECTION_INNER_ROW_GAP_PX` | 8 px (Sm) | Gap entre **rows consecutivas DENTRO** de uma seção (ex: Position → Rotation → Scale; Strategy → Pixel format → Reimport). |
| `SECTION_BOTTOM_PAD_PX` | **0 px (2026-05-24)** | Gap extra entre **última row da seção** e a `paint_section_separator`. Zerado porque o separador já contribui `SEPARATOR_PAD_Y` (≈10 px) acima E abaixo da própria linha; adicionar pad-extra criava assimetria visível (top-gap=10 px, bottom-gap=18 px → seção "encostada" no separador de cima). Com 0, ambos os gaps = 10 px, seção visualmente centralizada entre seus separadores. Mantido como constante (não removido) pra estabilidade da API. |

`SEPARATOR_PAD_Y` (definido em [`widget/showcase/mod.rs`](../../../crates/ph2d-editor-core/src/widget/showcase/mod.rs)) controla o espaço acima E abaixo da linha separadora (Md = 10 px cada lado). Não confundir com `SECTION_BOTTOM_PAD_PX`, que é o gap ANTES do separador.

**Regra:** seções declaradas em qualquer panel-crate (não só Inspector) **DEVEM** consumir essas constantes, não hardcodar Spacing tokens. Se a constante não couber, levante issue — não improvise.

## Pendências (próximos checkpoints)

- **Click no color dot → abrir color picker.** [`color_circle_hit_rect`](../../../crates/ph2d-editor-core/src/widget/section_header.rs) já expõe o rect; falta dispatch wiring + handler que abre BlenderColorPicker associado.
- **Broadcast pros outros painéis Inspector-type** (BgRemoval, Padding, CEQ, Upscale, EqSizes, Grid Snap) — adotar o mesmo padrão de `paint_section_header` + `mark_collapsible_section`.

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
