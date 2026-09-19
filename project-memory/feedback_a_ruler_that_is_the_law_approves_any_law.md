---
name: feedback-a-ruler-that-is-the-law-approves-any-law
description: Um gate que mede com a mesma função que julga fica verde sobre qualquer mutação dela
metadata:
  type: feedback
---

O gate *«nenhum segmento fica acima do passo»* media o comprimento com a **mesma** função que a lei
usa para decidir onde cortar. Trocar o polígono de controlo pela CORDA — que subestima um arco —
deixava-o **verde**: *uma régua que é a própria lei aprova qualquer lei.* A mutação sobreviveu, e a
cura foi achatar o arco **dentro do gate**, mais uma fixtura com uma ELIPSE (numa aresta recta a
corda e o polígono de controlo são o mesmo número, logo o rectângulo não distinguia as duas).

**Why:** é a forma mais silenciosa de gate vácuo que existe — ele corre, mede, compara e não pode
falhar. Não aparece num relatório de cobertura nem num `grep`: só uma prova de mutação sobre a
função medidora o revela.

**How to apply:** ao escrever um gate sobre uma grandeza, pergunte *de onde vem o número do lado
ESQUERDO?* Se vier da mesma porta que o produto usa, calcule-o por outro caminho no teste — e
escolha a fixtura onde os dois caminhos DIVERGEM, senão eles coincidem por acaso. Relacionado:
[[feedback-a-mutation-proof-needs-a-control-on-its-own-filter]] ·
[[reference-topic-gate-discipline]]
