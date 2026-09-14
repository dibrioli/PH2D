---
name: feedback-a-parity-measured-upstream-of-a-conversion-says-nothing-about-it
description: 51 de 61 fixturas do oráculo fechavam sobre uma ponte que invertia a curva do pincel — o corpus corre a lei DIRECTAMENTE e nunca atravessa a conversão
metadata:
  type: feedback
---

O pincel de contorno fechou `51 de 61` traços do oráculo dentro da barra, e o
que o artista via era o **contrário** da lei: a beirada ficava parada e o miolo
dobrava.

**O mecanismo:** duas casas escrevem a mesma curva com argumentos **opostos**, e
as duas estão certas em casa — para a lei o argumento é *quanto FALTA* (vale `1`
na borda) e para a curva do pincel é *quanto já se ANDOU* (vale `1` no centro do
carimbo). A ponte ligava-as sem a inversão, e o peso na borda lia `curva(1) = 0`.

**Why:** a bancada de paridade constrói a lei **directamente**, com a convenção
dela — ela nunca passa pela conversão que o produto usa. *Uma paridade medida a
montante de uma conversão não afirma nada sobre a conversão*, por mais fixturas
que tenha.

**How to apply:** toda adaptação de unidade, orientação, base ou convenção entre
duas casas precisa de **um gate no caminho do produto**, não no da bancada — e
ele tem de afirmar a FRASE do domínio (*«a borda é quem mais se move, e o efeito
morre para dentro»*), medida ponta a ponta. ⚠️ Escreva a tabela das duas
convenções no sítio da conversão: foi lê-las lado a lado que tornou a linha
óbvia. Irmão de
[[feedback_an_attested_spec_is_refutable_by_the_corpus_and_the_blind_spot_is_the_corpus]].
