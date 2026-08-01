---
name: feedback-a-capability-without-a-door-passes-every-gate
description: "Motor honra o campo ponta a ponta, gateado e com paridade — e NADA no produto consegue pedir. Todo gate fica verde."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 185146ca-f166-44a7-ad8e-b32c532ff022
  modified: 2026-08-01T13:06:13.354Z
---

Uma capacidade pode estar **completa e inalcançável**, e nesse estado ela **passa em TODOS os
gates** — porque cada peça dela está certa.

**Caso real (Flip, 2026-08-01).** `FlipStroke::cap` era honrado ponta a ponta: o bit no `pack`, o
semi-plano na silhueta, o ramo do `flip.wgsl`, tudo gateado e com paridade CPU×device provada. E
**nenhum traço do artista jamais foi reto**: o `build_stroke` não escrevia `s.cap` e o snapshot de
estilo não tinha o campo. `Cap::Flat` era alcançável só de dentro de um teste.

**Why:** é a falha mais silenciosa que existe. Não há erro, warning, gate vermelho nem widget morto
— não há widget nenhum. É **upstream** de [[feedback_painted_is_not_populated_paint_gate]]: lá o
controle existe e não responde; aqui ele nunca foi desenhado, e o campo até viaja serializado.

**How to apply:**
- Ao auditar um módulo, a pergunta não é *"isto funciona?"* e sim ***"o artista consegue PEDIR
  isto?"*** — grepe quem ESCREVE o campo, não quem o lê.
- Enum de modelo com variante que nenhum caminho de produto produz é o sintoma: `grep -rn "Kind::X"
  --include=*.rs | grep -v test` devolvendo só a definição.
- O gate certo atravessa a porta INTEIRA (estilo autorado → modelo → o bit/estado que o motor lê),
  porque o elo roto está no MEIO — gates das pontas ficam verdes dos dois lados.
- Corolário na hora de ESTENDER: acrescentar variante a um enum muda o que copiar significa. Todo
  choke point que faz `dst.campo = src.campo` tem de ser reconferido
  ([[feedback_a_condition_that_enumerates_its_readers_rots]] pelo outro lado).
