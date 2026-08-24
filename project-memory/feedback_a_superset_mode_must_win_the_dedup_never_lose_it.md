---
name: a-superset-mode-must-win-the-dedup-never-lose-it
description: "Quando dois modos de desenho disputam a mesma entidade, quem sai é o SUBSET — desistir do superset apaga o que só ele desenha (o destaque, as alças)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d971358c-b4ab-4ed0-ab84-65cd6d892c68
  modified: 2026-08-23T13:21:43.706Z
---

Duas passagens de desenho reclamaram a mesma entidade. Eu vi o problema certo — *desenhar duas
vezes soma o alfa e finge destaque* — e curei ao contrário: fiz o modo **rico** (`Editing`: todas
as âncoras + realce da linha aberta + alças) desistir para o modo **pobre** (`AlwaysVisible`:
todas esmaecidas). Efeito: marcar «Always show anchors» **apagava o destaque da âncora
selecionada**. E o gate afirmava esse comportamento, com a justificação certa ao lado.

**Why:** a dedup entre modos não é simétrica. Se um modo desenha um *superset* do outro, remover o
superset perde informação; remover o subset não perde nada. A pergunta não é «quem chegou
primeiro» nem «quem é mais específico» — é **qual dos dois contém o outro**.

**How to apply:** antes de escrever uma dedup entre modos de render, liste o que cada um desenha e
verifique a inclusão. Se A ⊇ B, o skip vai em B, e o gate afirma `A` — nunca `B`. E quando o modo
rico depende de uma condição (aqui, a §12 estar aberta), decida-o **primeiro** e faça as outras
passagens saltarem-no, em vez de as deixar correr e tentar desfazer depois.

⚠️ Corolário medido: *um gate verde pode pinar um defeito de produto*. Três defeitos desta linha em
dois dias foram apanhados por smoke do Enio, nenhum por suíte — os três eram decisões de produto
que eu tinha escrito no gate. Ver [[feedback_inherited_affordance_must_be_rederived]] e
[[feedback_ergonomics_verdict_is_a_design_bug]].
