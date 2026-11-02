---
name: feedback_a_field_that_is_not_self_evident_gets_a_label
description: Dono 01/11 — no layout Figma dos painéis, todo campo que não é PERFEITAMENTE inteligível sozinho leva rótulo pequeno acima (Mix, Chain length, Softness…); ícone sozinho não basta
metadata:
  type: feedback
---

Dono, 01/11 (maquete do painel Bones em cartões Kanban + campos Figma, linha de teste `line/TrelloUITeste`):
na secção IK os campos tinham só ícone + valor, enquanto Skin tinha o rótulo «Deform by». Ele marcou a
diferença e fixou como REGRA: **todo campo que não é perfeitamente inteligível sem nome ganha um rótulo
pequeno acima dele** (como o Figma faz com Position / Rotation / Dimensions). Dois campos lado a lado =
um rótulo por coluna. Dispensa só o que o próprio texto já diz (segmentado «Create · Transform · Weight»).

**Why:** o ícone do Figma é atalho para quem já conhece a propriedade; o PH2D tem propriedades próprias
(IK Mix, Softness, Bend, Curve in/out) que nenhum ícone explica — e o dono aprende a ferramenta pelo ecrã.

**How to apply:** ao desenhar qualquer painel no idioma Figma, liste cada campo e pergunte «sem o nome,
o dono sabe o que é?» — na dúvida, rótulo. O ícone continua (é a pega de arrastar), o rótulo não o
substitui. Relacionado: [[project_vector_effects_filters_figma_layout_2026_11_01]],
[[feedback_app_ui_english_only]].
