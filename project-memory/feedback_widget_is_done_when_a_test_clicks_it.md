---
name: feedback-widget-is-done-when-a-test-clicks-it
description: Um widget novo não está pronto quando PINTA — está pronto quando um teste headless CLICA nele e o estado do tool muda; sem registro no WidgetStore o ponteiro nunca vira Click
metadata:
  type: feedback
---

Ao entregar **qualquer widget novo** (chip, checkbox, swatch, segmento), o gate obrigatório é um teste
headless que **CLICA nele** e afirma que o estado do tool mudou. Pintar não é entregar.

**Why:** o seam painel↔tool tem 4+ elos (pintar · registrar hit-rect · **registrar no `WidgetStore`** ·
encaminhar o `Click` em `event.rs` · rotear no tool), e **cada um é silencioso quando quebra**. Um widget
que pinta perfeitamente e não faz nada passa em *todo* teste de unidade, em clippy, no CI.

**O MECANISMO — decore, porque torna o diagnóstico mecânico** (2026-07-13):
`dispatch_pointer` só torna um hit `active` no Down se ele for **focusable**, e
`is_focusable(store, id)` faz `None => false` — **id que o store nunca viu não ativa, então o Up nunca
chama `apply_click`, então o `Click` NUNCA EXISTE.** E `paint_checkbox_row` /
`paint_segmented_group_adaptive` recebem o store por **`&`** (imutável): eles **não podem** se
auto-registrar. Só o `populate` fecha o fio. Corolário: *pintar + hit rect = morto*.

**A armadilha de segunda ordem:** registrar, mas com o `InteractiveState` errado. `Checkbox`/`Toggle`
emitem `WidgetEvent::Toggled`, que o `event.rs` **não encaminha** — fica **registrado e ainda morto**.
Use `Button` mesmo no que *pinta* como checkbox (o estado mora no TOOL; a pintura espelha o snapshot).
É a convenção da casa (as checkboxes de Symmetry são `Button`).

**Histórico:** no Impasto (2026-07-12) o rig de 4 luzes foi entregue com **6 gates na matemática e 3
mutações vermelhas** — e **zero gates no seam**. Os chips pintaram com o estado certo (o print do Enio
mostrava as apagadas marcadas!) e **nenhum respondia**. Não eram só os chips: os 13 ids de clique da
seção estavam mortos, inclusive o **Enable mestre** — o smoke (`PH2D_IMPASTO_SMOKE`) arma
`toggle_brush_impasto()` **em código**, e foi isso que fez os cards aparecerem e mascarou o resto.
*Um scaffold que arma o estado por baixo do pano esconde exatamente o seam que ele deveria exercitar.*

**How to apply:** o `ph2d-ui-testkit` agora **sabe clicar** (`MockPanelHost::click_at` — Down+Up pelo
`dispatch_pointer` REAL, sobre o hit index que a pintura construiu; `hit_at` diz quem roubou o hit).
Padrão real, do pixel ao estado do tool:

```rust
let painted = host.paint::<PainterLayersPanel>(&mut st, viewport());   // hit rects reais
let (_, r) = painted.iter().find(|(w, _)| *w == LIGHT_2).unwrap();     // pintado?
for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {            // o seam INTEIRO
    host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
}
for a in host.drained_actions() {
    if let EditorAction::ToolPanelEvent(pe) = a { tool.handle_panel_event(pe); }
}
assert_eq!(tool.brush_settings().impasto_rig.selected, 1);
```

Escreva-o **ANTES** do fix quando o widget já está morto: nasce vermelho e **prova** o diagnóstico
(matou em minutos dois candidatos que o handoff dava como certos). Escreva-o **junto** com o widget
quando é novo: custa 6 linhas. E varra a seção inteira — o sintoma reportado quase nunca é a extensão
do estrago (o header não colapsava e o dot de cor não abria o picker, e ninguém tinha notado).

Família: [[feedback_painted_is_not_populated_paint_gate]] (nenhum gate rodava `paint`) ·
[[feedback_tool_unit_green_integration_dead]] (tool passa unit+CI e está morta) ·
[[feedback_panel_populate_register]] (botão novo exige register) ·
[[feedback_disabled_button_still_dispatches]] · [[feedback_harness_reproduces_mechanism_not_context]]
(o mock não roda o `pre_populate` compartilhado — `insp_*`/scrollbar aparecem "mortos" e não estão).
Todas dizem a mesma coisa por ângulos diferentes — esta é a regra que as resume:
**o widget está pronto quando um teste clica nele.**
