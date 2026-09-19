---
name: feedback-a-cure-on-a-path-the-product-does-not-execute
description: Uma cura gateada ao bit pode estar fora do caminho que o produto corre, e todo gate dela fica verde
metadata:
  type: feedback
---

**Antes de declarar uma cura ao dono, prove que o produto EXECUTA o código curado
no caminho de OMISSÃO dele.** Uma cura pode estar certa, gateada ao bit, com
prova de mutação — e viver num ramo que o artista nunca alcança. Todos os gates
ficam verdes, e o sintoma dele continua exactamente igual.

**Caso medido (2026-09-19, `line/3DModeling`).** Duas waves seguidas (a sombra de
borda mole; depois o raio do borrão por material) foram entregues ao dono com
tabela, gates e mutação. Ele respondeu, pela terceira vez, que o sintoma
continuava — e exigiu auditoria. O achado:

```text
let pintado = if pelo_dispositivo && !p.refinar { gpu_frame::paint(…) } else { None };
if let Some(pintura) = pintado { …manda a imagem…; return; }   ⇐ DEVOLVE AQUI
…
sh.set_soft(… sss_shadow::blur_por_material …)                 ⇐ nunca alcançado
```

O `!p.refinar` era o valor de **fábrica**. ⇒ as duas curas existiam e o produto
**não as corria**.

⚠️ **E a segunda metade é o que o escondeu:** todas as colunas que aquele módulo
rotulava «DISPOSITIVO» eram a **CPU** com um campo (`mole: false`) desligado — um
**sucedâneo**, e a placa nunca tinha sido corrida por gate nenhum. O sucedâneo
até acertava no número (`9,21` contra `9,20` medidos na placa), o que é o pior
caso possível: *ele dava confiança sobre um programa que ninguém tinha observado,
e a RAZÃO que eu escrevia ao lado do número estava errada*.

**Why:** um gate prova que a LEI está certa; nenhum gate deste repositório
pergunta *«o produto chega aqui?»*. As duas perguntas leem-se iguais num relatório
verde, e a diferença só aparece no smoke do dono — que é a superfície mais cara
de todas.

**How to apply:** ao fechar uma cura, siga o caminho do **pen-down até ao pixel**
lendo as condições reais (`enabled()`, `!flag`, `matches!(modo, …)`) e escreva os
valores de FÁBRICA de cada uma numa tabela. Se alguma delas desvia antes do
código curado, a cura não shipou. E **um gate estrutural barato fecha isto**: uma
asserção textual sobre a ORDEM (`o ramo devolve antes da chamada`) corre sem
placa, sem cena e sem device, e reprova no dia em que o gémeo chegar.

⚠️ **O gate estrutural tem de nomear o CORPO do ramo, não o resto do ficheiro:** a
1.ª redacção procurava o `return;` seguinte em toda a fonte e uma mutação
**sobreviveu** — apagado o `return` do ramo pintado, ela achava o `return` de
outro caminho, que também vinha antes da chamada. *O gate afirmava «há ALGUM
return pelo caminho» e o nome dele prometia «ESTE ramo devolve».*

Irmão de [[feedback_a_probe_that_arms_a_module_by_env_var_measures_another_program_than_the_pill]]
e de [[feedback_the_inner_channel_fixture_is_below_the_break]].
