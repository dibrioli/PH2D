---
name: a-surface-that-only-counts-is-usually-missing-a-datum-not-a-widget
description: Um painel que diz «há 3» onde o critério pede «quais 3» quase nunca é buraco de UI — é um dado que ninguém guardou, e a janela para o guardar costuma ser de um passe
metadata:
  type: feedback
---

O cartão de instância dizia **`3 unused override(s)`** com o botão que apaga as três ao lado, e o
critério do plano pedia que cada excepção *«apareça»*. Durante uma semana isso leu-se como um item
de UI por fazer.

**Não era.** Cada uma dessas excepções pertencia a uma peça que o mestre **apagou**, e a chave que
sobrava era um `StableId` que já não nomeava nada. Não havia como escrever a linha: **o nome nunca
tinha sido guardado**. E a janela para o guardar era de um passe — quem sepulta a excepção corre com
a peça **ainda viva**, e o `despawn` é o laço a seguir; um passe depois não há onde o ir buscar.

⭐ **E o argumento que autorizava guardá-lo já estava escrito, para outro campo.** A refutação
original — *«guardar o valor cria duas fontes para o mesmo facto»* — vale **enquanto a coisa
existe**. Sobre uma coisa apagada não há segunda fonte: há a única. Isso já tinha justificado
guardar os *bytes* ali ao lado, e ninguém tinha reparado que **cobre todos os factos sobre a coisa
morta**, o nome incluído.

**Why:** «contagem» e «lista» parecem o mesmo trabalho com granularidades diferentes, e não são: a
lista precisa de um dado **por item** que a contagem nunca precisou. Ler o défice como UI leva a
abrir o painel e descobrir que não há o que pintar.

**How to apply:** perante um `N` onde o requisito pede *«quais N»*, **pergunte primeiro o que a
linha diria** e vá procurar cada metade no modelo. Se alguma não existe: (1) ache o instante em que
ela ainda é legível — costuma ser o passe que destrói a coisa; (2) veja se a recusa que proibiria
guardá-la é uma recusa sobre coisas **vivas** (aí ela não se aplica); (3) guarde-a **só para
mostrar**, com a chave de procura a continuar a ser a identidade. Relacionado:
[[a-measured-refusal-answers-one-question-recheck-it-when-yours-is-another]],
[[the-representation-can-delete-the-special-case]].
