---
name: feedback_a_ratio_between_two_sick_channels_is_green_by_construction
description: "Gate que compara canal A com canal B fica verde quando os DOIS têm o mesmo defeito — ancore no que o usuário tem direito a (o pincel, a spec), não no vizinho"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1ad2e828-0576-4788-947a-5980948e93be
---

Um gate que afirma *"o relevo acompanha o pigmento"* comparando **relevo ÷ pigmento** é verde por
construção quando os dois canais compartilham o kernel doente. Caso real (Smear, 2026-07-18): o transporte
de cor e o de corpo tinham a estrutura idêntica (`h·wⁿ` por dab), então mediram
**`relief_w == pigment_w` ao texel em toda estação da trilha** — razão perfeita, produto quebrado. O
handoff até avisava *"a COR tem a mesma estrutura"*, e eu escrevi a razão mesmo assim.

**Why:** um oráculo relativo mede *concordância*, não *correção*. Dois canais errados concordam. A
pergunta do artista nunca é "o corpo acompanha a tinta?" — é "arrastei uma faca de 32 px, cadê a trilha de
32 px?".

**How to apply:** ancore a asserção numa quantidade que o produto **promete** e que não pode adoecer junto
— o raio do pincel, a spec, a unidade do usuário. Se você só tem canais derivados para comparar, ainda não
tem oráculo. E confira: se o gate ficaria verde com AMBOS os lados quebrados, ele não é um gate.

Irmão de [[feedback_test_with_product_numbers_not_convenient_ones]] e de
[[reference_topic_oracle_discipline]]; o parente de fixture é
[[reference_topic_fixture_discipline]] (o mesmo gate também cravava `hardness=1.0`, que tornava o defeito
**inalcançável** — um fixture que exclui o fenômeno que ele existe para pegar).
