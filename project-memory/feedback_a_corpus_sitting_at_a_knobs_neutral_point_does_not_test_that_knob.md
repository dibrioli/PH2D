---
name: a-corpus-sitting-at-a-knobs-neutral-point-does-not-test-that-knob
description: Uma mutação sobrevive quando o fenómeno que a guarda defende não existe no corpus — construa a fixtura pelo mecanismo.
metadata:
  type: feedback
---

Quando uma mutação **sobrevive** a uma suíte inteira, a primeira pergunta não é *«falta um
gate?»* — é ***«o fenómeno que aquele código defende existe no corpus?»***. Muitas vezes não
existe, e nenhum gate escrito sobre o corpus o encontraria.

**Why:** L-System, 2026-08-31. Duas mutações sobreviveram a 125 testes:
- tirar o `step_scale` da escada de tamanhos — **os oito moldes deixam-no no default `1,0`**, onde
  `powf` devolve `1,0` ao bit e o factor é inerte;
- apagar a rede que descarta um degrau plano — **nenhum molde tem um patamar mais largo que a
  densidade da escada**.

As duas fixturas que as mataram foram construídas **pelo mecanismo**, não por afinação: pôr o
knob fora do neutro (`Step Scale = 0,5`), e desenhar uma planta cujos galhos laterais (`0,9·s`)
são mais compridos do que o rebento de uma geração (`0,8·s`), de modo que ele **nunca** os
ultrapassa ⇒ o patamar cobre a geração inteira. ⭐ A segunda fixtura também revelou que o defeito
do `step_scale` era **pré-existente** (`0,3737` de desvio com a lei antiga).

**How to apply:** ao escrever uma guarda, pergunte *que valor de que param faz isto disparar?* — e
se nenhum membro do corpus o tem, a fixtura faz parte da mesma wave. Derive-a do mecanismo (a
desigualdade que produz o fenómeno), nunca varrendo números até um teste ficar vermelho. Ver
[[a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]] e
[[a-declaration-with-a-default-is-decoration-until-something-reads-it]].
