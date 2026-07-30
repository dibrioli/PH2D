---
name: feedback_a_per_pass_gain_becomes_a_product_gain_only_through_the_cadence
description: Ganho por-passe só vira ganho de produto depois da cadência; e uma razão medida numa fixture não se transporta para outra de composição diferente
metadata:
  type: feedback
---

Otimizei três passes do solver do Wet Paint e anunciei **1,56×**. O produto ganhou
**1,10×**, e os dois números estavam certos: o passo não roda todo passe em todo
passo. Amortizada pela cadência real (um passe a cada 2, 3 ou 4 passos), a
decomposição mostrou que os três que eu acelerei são **6,5% do passo**, não os 46%
que a soma-sem-cadência sugeria.

**Why:** um perfil por-passe mede *quanto um passe custa quando roda*. O produto paga
*quanto ele custa em média por passo*, e entre os dois há a **frequência**. Sem
multiplicar por ela, toda porcentagem de perfil é sobre um passo que não existe. A
mesma sessão me pegou uma segunda vez pelo outro lado: extrapolei uma razão
diagonal÷horizontal (1,8-2,0×) para uma cena com **14× mais células ativas**, e o
efeito que a razão media valia 5% ali — *a razão estava certa, a extrapolação não*.

**How to apply:** antes de anunciar um ganho, (1) leia a cadência do laço e amortize
cada passe por ela; o modelo tem de **prever** o custo medido pela porta do produto
(o meu fechou com 0,03 ms de erro em 62 — e é esse acerto que autoriza a conclusão);
(2) meça na fixture do **PRODUTO**, não na da crate — as minhas duas cenas "grandes"
custavam 10,34 e 62,05 ms/passo, seis vezes, e só a segunda vira decisão; (3) ao
transportar uma razão entre cenas, ajuste um modelo com os dois eixos
(`custo = a·janela + b·ativas`) em vez de reusar o quociente.

Ver [[feedback_two_quantities_that_should_differ_can_coincide_by_fixture_phase]] e
[[reference_topic_repro_discipline]].
