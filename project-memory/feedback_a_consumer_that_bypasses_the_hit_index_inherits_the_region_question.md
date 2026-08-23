---
name: feedback_a_consumer_that_bypasses_the_hit_index_inherits_the_region_question
description: "Um handler que faz o próprio hit-test em coordenadas de MUNDO herda a pergunta que o hit-index respondia — onde ele vale; sem a guarda de região ele engole cliques de painéis inteiros, e o sintoma é «não consigo ligar um fio», nunca «o gizmo pegou»"
metadata:
  type: feedback
---

O gizmo de canvas novo consumia o `Down` no topo do despacho, como os irmãos (Flip pose,
Flip selection, field). O Enio reportou: *"se colocar transform antes, não é possível
conectar transform em Bezier Warp"*.

**Why:** os irmãos consomem perguntando ao **HIT-INDEX** (`GizmoTarget::…`), e o hit-index
já sabe das regiões — ele só devolve um alvo quando o cursor está sobre o canvas. O meu
fazia o **próprio** hit-test, convertendo o ponteiro para MUNDO com a câmara da cena. Um
clique no **painel do grafo** também converte para um ponto de mundo; se calhar de cair no
raio de uma alça, o `Down` é consumido e o gesto de ligar um fio **nunca começa**.

⚠️ **E o sintoma não aponta para o culpado.** Ninguém relata *"o gizmo agarrou uma alça
invisível"* — relata *"não consigo conectar"*. O handler que engole vive num arquivo que
não tem nada a ver com fios, e a busca natural vai para a validação de portas (que estava
correcta: a conexão passa e valida no grafo, medido).

**How to apply:**
1. ⚠️ **Ao consumir um evento com hit-test PRÓPRIO, a guarda de região é sua** — o
   `on_canvas` da casa (nenhum painel sob o cursor **e** nenhum widget no hit-index) já
   existe e é a resposta; não invente outra. *Quem decide sozinho tem de saber sozinho
   onde vale.*
2. **Copiar a POSIÇÃO no despacho não copia a PRÉ-CONDIÇÃO.** Eu pus o meu ao lado dos
   irmãos e herdei a ordem deles sem herdar o que os torna seguros ali — aquilo era o
   hit-index, e ele não estava no código que copiei, estava no que o método deles chama.
3. **Quando um relato diz *"não consigo fazer X"* e X passa no nível do modelo, procure
   quem CONSOME o evento antes de X** — a lista de handlers na ordem do despacho, não a
   validação de X. Aqui a conexão foi medida a passar (`connect` → `Ok`, `validate` →
   `Ok`) em dez segundos, e isso apontou o resto da busca para o sítio certo.
4. O gate é de FONTE e barato: a condição do `if` que precede a chamada tem de conter a
   guarda. [[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]

*Terceiro defeito seguido deste gizmo em que a causa era o FRAME e não a geometria —
irmão de [[feedback_paint_and_hit_test_must_project_through_one_door]]: ali as duas
superfícies projectavam diferente, aqui uma delas não sabia onde parava.*
