---
name: feedback_a_ceiling_written_from_price_may_be_double_what_the_march_allows
description: Um teto de contagem escrito a partir do CUSTO passa calado por outro recurso que amarra antes — e o representante típico de um gate nunca corre o teto.
metadata:
  type: feedback
---

Quando uma forma tem uma **contagem** com teto (lados, pontas, dentes, bossas), a régua óbvia é o
**preço**. ⚠️ Mas outro recurso pode amarrar muito antes, e o teto escrito pelo primeiro fica ao
dobro do que o segundo permite.

**Why:** medido em 2026-09-05 (formas 3D). O `MAX_STAR_POINTS = 16` saiu de uma tabela de custo
(*«a estrela chega ao preço do prisma às 16 pontas»*). Corrida a **marcha**, o joelho está entre `8`
e `9` — `passo × ‖∇f‖` vai de `0,88` a `1,07`, e acima de `1` o traçador **atravessa a superfície**.
A mesma armadilha ia repetir-se na nuvem: o preço dizia `12`, a marcha diz `7`.

⛔ **E nenhum censo o via**, porque o representante de cada gate usa um valor **típico**: o
`every_primitive_honours_the_march` media a forma que o artista cria e **nunca** a que ele alcança
arrastando o slider até ao fim. *Um teto que ninguém corre é uma promessa.*

**How to apply:** (1) toda constante de contagem entra com um gate que a corre **no próprio teto**,
derivado da faixa declarada (`Span::Count`) e não de uma lista à mão — ver
[[feedback_a_literal_corpus_count_in_a_gate_makes_every_new_feature_edit_someone_elses_test]];
(2) subir uma contagem **encolhe** o filete que a forma comporta, então o gate tem de chamar o
`clamp_round` como o produto faz, senão ele mede uma peça que o documento recusa;
(3) ⚠️ **não baixe um teto que já shipa só com um gate de gradiente** — ele diz *«pode furar»*, e só
a IMAGEM diz *«fura»*. Declare o número com a tabela e devolva a decisão a quem vê.
