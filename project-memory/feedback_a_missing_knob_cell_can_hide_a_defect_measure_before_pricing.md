---
name: feedback_a_missing_knob_cell_can_hide_a_defect_measure_before_pricing
description: Uma célula de auditoria classificada como «knob ausente, só a referência X tem» pode estar por cima de um DEFEITO — a classificação vem de quem leu a referência, não de quem mediu o produto; e uma sonda que corre com um param no default mede o param desligado, não a lei
metadata:
  type: feedback
---

⭐⭐⭐ **A natureza de um gap — *omissão* contra *defeito* — é uma afirmação sobre o
PRODUTO, e quase sempre foi escrita a olhar para a REFERÊNCIA.** Meça antes de a
aceitar, mesmo (sobretudo) quando ela parece a célula mais fraca da folha.

**Why:** medido em 2026-08-26, folha 08 do doc 89. A célula do `motion.duplicator`
dizia *"modos de transferência de atributo (Copy/Mult/Add/Sub) — **omissão** (só
Houdini tem; Cavalry e Blender não)"*. Lida assim, é a última da fila: um knob de
nicho que duas das três referências nem oferecem.

⛔ **A sonda mediu outra coisa.** O carimbo replicava as colunas da FORMA e somava
`P`/`rot`; **toda coluna autorada só nos PONTOS desaparecia sem aviso** —

```text
nos pontos:  Count · Index · P · tint
na saida:    Count · Index · P          ⚠️ tint sumiu
```

— e uma rampa de cor sobre um arranjo é o gesto mais comum que existe. Não era um
knob de nicho: era perda silenciosa de trabalho do artista, e a ordem do plano
(*«dentro de cada folha, primeiro o que descreve comportamento errado»*) tinha-a posto
no fim **porque a célula não sabia o que estava a descrever**.

⚠️ **E havia uma cerca por cima, que a leitura ingénua respeitaria:** o doc do nó
declarava *"a per-point tint would be a future extension — the shape is the template"*.
Uma cerca de Chesterton legítima diz *«escolhemos que A vence B»*; esta dizia isso e o
código fazia outra coisa — **B não perdia, B evaporava**. *Uma cerca que descreve uma
precedência não cobre uma perda.*

**How to apply:**

1. ⭐⭐ **Antes de precificar uma célula pela natureza declarada, corra a sonda que
   pergunta o que o produto FAZ.** Aqui foram duas linhas: cozer o grafo e imprimir as
   colunas dos dois lados da costura. *A ordem «defeito antes de knob» não se lê da
   célula — lê-se da sonda.*
2. ⚠️ **Uma sonda que corre com um param no DEFAULT mede o param desligado.** Na mesma
   jornada, a 1.ª leitura de outra sonda concluiu *«o afunilamento perde-se na rota por
   composição»* — e ela corria com `point_scale = 0`, cujo próprio doc diz que o `0` é
   *"a escala do ponto fica de fora, que é o que sempre aconteceu"*. Eu tinha medido uma
   tautologia e ia escrever uma célula em cima dela. **Varra o param antes de o declarar
   insuficiente**; com `point_scale = 1` a rota reproduz o arranjo *ao valor*.
3. ⭐ **A cura de uma perda entra DESLIGADA e a pergunta vai com o smoke.** A referência
   transfere por omissão; ligar isso por default mudaria arte já autorada, que nesta casa
   é decisão do dono do produto. O default é *«o que sempre aconteceu»*, byte a byte, e o
   gate do controlo prova-o.
4. ⛔ **Quando a cura cai, REESCREVA a cerca no mesmo commit.** O doc-comment que dizia
   *"future extension"* passou a dizer o que de facto havia (uma perda) e o que passou a
   existir. Um comentário velho ao lado de um param novo é a próxima nota errada.
5. ⚠️ **E não deixe a grandeza que já tinha porta entrar na lei nova.** O `size` tem o
   `point_scale`, medido e decidido; a transferência salta-o **pelo nome**, com gate.
   *Uma grandeza com duas portas é como as duas divergem.*

Irmãs: [[feedback_ask_what_number_the_opposite_answer_would_print]] ·
[[feedback_documented_decision_chesterton_fence]] ·
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] ·
[[feedback_stale_comment_and_dead_code_lie]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]]
