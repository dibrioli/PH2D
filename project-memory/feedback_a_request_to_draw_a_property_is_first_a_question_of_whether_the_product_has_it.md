---
name: feedback-a-request-to-draw-a-property-is-first-a-question-of-whether-the-product-has-it
description: "Pedido de \"faça o desenho expressar X\" — MEÇA se o produto faz X antes de desenhar; desenhar X sobre um produto que não o faz é um desenho que mente."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: fa8d010b-41ed-4c6b-a35d-1ff0035f12da
  modified: 2026-08-01T13:06:29.608Z
---

Quando o Enio pede que **o desenho expresse** uma propriedade ("a curva deve expressar essa
continuidade"), a primeira coisa é **medir se o produto tem a propriedade** — não abrir o
painel.

**Why:** em 2026-08-01 o pedido era desenhar a curva da costura do loop como UMA curva
contínua. Medindo a VELOCIDADE antes de desenhar: ela subia a −0,299, caía a **0,000
exatamente na volta** e recomeçava — o objeto PARAVA na costura. Cada ponta corria um S
inteiro na própria janela. Um desenho contínuo sobre isso seria um desenho que **mente**, e
a divergência apareceria só numa screenshot, onde ninguém lê número. O trabalho real era no
MOTOR (uma travessia, uma curva, cada ponta com a sua fatia); o desenho virou consequência.

**How to apply:** antes de tocar no painel, escreva a sonda que mede a propriedade pedida no
caminho do PRODUTO (velocidade, não pose, quando o que se pede é continuidade — a pose já era
contínua e um gate de pose ficaria verde sobre a parada). Se a propriedade não existe, o
pedido é de motor. Se existe, aí sim o desenho é que está errado. Corolário: quando o motor
muda, o desenho **não** precisa de fiação nova se o snapshot já publica o valor EFETIVO — foi
assim que a direção do easing chegou à tela com zero linha no painel.

Relacionado: [[feedback_oracle_must_model_appearance_not_implementation]] ·
[[reference_topic_oracle_discipline]] · [[feedback_stale_comment_and_dead_code_lie]]
