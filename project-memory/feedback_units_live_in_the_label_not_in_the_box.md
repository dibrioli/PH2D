---
name: feedback_units_live_in_the_label_not_in_the_box
description: Dono 01/11 — no layout Figma dos painéis a UNIDADE (px, %, °) sai da caixa numérica e vai para o rótulo («Length (px)»); a caixa tem só ícone + número, para estreitar no iPad sem embolar
metadata:
  type: feedback
---

Dono, 01/11 (maquete Bones + Inspector em cartões Kanban + campos Figma, `line/TrelloUITeste`): a
unidade dentro da caixa («120 px») sai — vai para o rótulo acima: **«Length (px)», «Rotation (°)»,
«Opacity (%)»**. A caixa numérica tem só o ícone (pega de arrastar) e o número.

**Why:** o PH2D quer painéis ESTREITOS que funcionem no iPad, e caixas numéricas que possam estreitar
sem o número e as letras da unidade se atropelarem. Uma unidade dentro da caixa come largura justamente
quando ela falta.

**How to apply:** em todo painel neste idioma, unidade = sufixo entre parênteses no rótulo; campo sem
unidade não leva parênteses. Um par X/Y partilha o rótulo do grupo («Position (px)»). Isto pressupõe o
rótulo — ver [[feedback_a_field_that_is_not_self_evident_gets_a_label]]. A caixa deve degradar bem até
estreita: o número nunca colide com o ícone.
