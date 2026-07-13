---
name: feedback-widget-is-done-when-a-test-clicks-it
description: Um widget novo não está pronto quando PINTA — está pronto quando um teste headless CLICA nele e o estado muda
metadata:
  type: feedback
---

Ao entregar **qualquer widget novo** (chip, checkbox, swatch, segmento), o gate obrigatório é um teste
headless que **CLICA nele** e afirma que o estado do tool mudou. Pintar não é entregar.

**Why:** o seam painel↔tool tem 4+ elos (pintar · registrar hit-rect · registrar no `WidgetStore` ·
encaminhar o `Click` em `event.rs` · rotear no tool), e **cada um deles é silencioso quando quebra**. Um
widget que pinta perfeitamente e não faz nada passa em *todo* teste de unidade, em clippy, no CI. No
Impasto (2026-07-12) entreguei o rig de 4 luzes com **6 gates na matemática e 3 mutações vermelhas** — e
**zero gates no seam**. Os chips pintaram, com o estado certo (o print do Enio mostrava as lâmpadas
apagadas marcadas corretamente!), e **nenhum respondia ao clique**. O Enio descobriu abrindo o app. Um
teste que clicasse o chip teria saído vermelho **antes**.

**How to apply:** use o `ph2d-ui-testkit` (seam headless, `docs` da Blindagem Fase 0). Padrão:

```rust
#[test]
fn clicking_the_chip_selects_that_lamp() {
    let mut h = testkit::panel_with_painter();
    h.paint();                                   // popula hit-index + store
    h.click(core_ids::PAINTER_IMPASTO_LIGHT_2);  // o seam INTEIRO
    assert_eq!(h.tool().paint.impasto_rig.selected, 1);
}
```

Escreva-o **ANTES** do fix quando o widget já está morto: ele nasce vermelho e prova o diagnóstico.
Escreva-o **junto** com o widget quando é novo: custa 6 linhas.

Família: [[feedback_painted_is_not_populated_paint_gate]] (nenhum gate rodava `paint`) ·
[[feedback_tool_unit_green_integration_dead]] (tool passa unit+CI e está morta) ·
[[feedback_panel_populate_register]] (botão novo exige register) ·
[[feedback_disabled_button_still_dispatches]]. Todas dizem a mesma coisa por ângulos diferentes — esta é
a regra operacional que as resume: **o widget está pronto quando um teste clica nele.**
