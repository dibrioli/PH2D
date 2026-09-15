---
name: feedback-the-ruler-and-the-product-shared-the-same-defect
description: Quando o número ESPERADO de um assert vem da mesma varredura que o produz, o assert não protege de nada
metadata:
  type: feedback
---

Ao acrescentar um campo a `PlayerInput`, contei os literais exaustivos com um regex, obtive **44**, e
escrevi `assert total == 44` no script que os editava. O assert disparou — mas o número esperado
**vinha do mesmo regex partido** que fazia a edição: ele contava `-> PlayerInput {` (o corpo de uma
função) como um literal, e lia `(40..48)` como sintaxe de actualização de struct. O verdadeiro era
**37** mais dois que o filtro de `..` escondia.

**Why:** um assert de contagem só protege quando o número vem de uma **régua independente**. Vindo da
mesma varredura, ele confirma que a varredura é consistente consigo mesma — que é sempre verdade.

**How to apply:** o número esperado de uma edição em lote sai de **outra fonte**: o compilador (os
erros de campo em falta), um `grep` com outra forma, ou a contagem à mão de uma amostra. E depois da
edição, a régua independente corre outra vez: aqui foi `cargo check --workspace --all-targets`, que
nomeou os dois que faltavam e o que tinha sido inserido fora de um literal. Ver
[[reference-topic-measurement-discipline]] e [[feedback-python-replace-silent-noop-after-fmt]].
