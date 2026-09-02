---
name: feedback-an-env-read-inside-a-library-function-reaches-every-caller
description: "Liguei uma feature por env DENTRO do remalhador; ela alcançou o motor legado e quebrou um gate — a escolha é do CHAMADOR, sempre"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T15:52:31.279Z
---

Em 2026-08-31 (`line/quadextract`) liguei a densidade adaptativa da fase zero lendo
`PH2D_ISO_ADAPT` **dentro** de `ph2d_remesh_iso::remesh_with`. Medi a cadeia de extracção,
ficou tudo melhor, e o portão de fecho reprovou noutro sítio:
`the_ear_does_not_ship_an_edge_across_the_piece` — um gate que corre o motor **legado**
(preenchimento por patch, o `Fast` do menu), que chama o **mesmo** `remesh_isotropic`.

⭐⭐ **E o doc da própria função já escrevia a lei que eu violei**, três linhas acima, sobre
outro parâmetro: *«por argumento e não por variável de ambiente: os testes desta crate correm em
paralelo no mesmo processo, e uma env lida lá dentro faria um gate decidir o resultado do outro.
Uma bandeira global é uma corrida escrita à mão.»*

**Why:** uma env dentro de uma função de biblioteca é um **acoplamento invisível a todos os
chamadores**, presentes e futuros. Quem a lê não vê a decisão no sítio onde ela é tomada, e
quem chama de outro caminho herda-a sem a pedir. O modo de falha é o caro: a medição que
justifica a mudança corre **num** caminho, e o dano aparece **noutro**.

**How to apply:**
- Uma feature nova entra por **porta separada** (`remesh_isotropic_graded` ao lado de
  `remesh_isotropic`), com a antiga byte-idêntica. ⭐ A env fica, mas é lida **no chamador**,
  onde a decisão é visível e alcançável por um gate.
- ⚠️ Antes de pôr um `env::var` dentro de uma crate, **conte os chamadores**. Se houver mais de
  um caminho de produto, a resposta é um parâmetro.
- ⭐ Se a função já tem um parâmetro que existe *precisamente* por esta razão (aqui o
  `rim_law`), o doc dele é a resposta escrita — leia-o antes de acrescentar o vizinho.

Relacionadas: [[feedback_a_cure_written_in_one_of_two_lowering_routes_makes_every_gate_lie]] ·
[[feedback_a_private_second_door_rots_while_the_shared_one_does_not]] ·
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]
