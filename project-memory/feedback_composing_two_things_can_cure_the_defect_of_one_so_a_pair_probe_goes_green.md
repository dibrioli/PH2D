---
name: feedback-composing-two-things-can-cure-the-defect-of-one-so-a-pair-probe-goes-green
description: Sonda de PARES dá verde ao que a de UM só reprovaria — compor pode curar o defeito de um dos membros
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T16:13:51.637Z
---

Uma sonda que varre **pares** (ou combinações de N) pode ser **cega ao membro sozinho**: compor duas
coisas às vezes **cura** o defeito de uma delas, e aí o par mede-se seguro.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30): o gate varria todos os pares de modificadores nas
duas ordens — e `[Bend]` **sozinha**, a um clique do nascimento, rasgava o campo
(`‖∇f‖ = 1,72`). O par `[Bend, Bend]` tem **envelope maior** ⇒ divisor maior ⇒ mede-se seguro, e o
gate dava verde.

⭐ E a nota que descrevia a dívida do par acusava a causa errada — *«dois deformadores encadeados: o
segundo lê um envelope que o primeiro já deformou»*. Não era o encadeamento: era uma **parede medida
contra a bola errada**, e ela mordia com **um** deformador só. Corrigi-la curou o membro sozinho, o
par (`44,6 → 0,27`) e duas combinações com a repetição (`245,8 → 0,23`).

**Why:** um gate de combinações mede a combinação. O membro sozinho é uma **população própria**, e a
mais comum — é o que o artista faz primeiro.

**How to apply:**
- Ao escrever um gate de pares/combinações, escreva o irmão de **um só**, varrendo a **faixa do
  parâmetro** de cada membro (não o valor de nascimento —
  [[feedback_a_declaration_with_a_default_is_decoration_until_something_reads_it]]).
- ⚠️ Quando uma dívida tolerada tem uma causa **escrita** na nota, meça-a antes de acreditar: a
  causa aqui estava errada e a cura verdadeira era um nível acima
  ([[feedback_a_correct_mechanism_can_prescribe_the_wrong_cure]]).
- ⭐ Uma **cerca calculada** tem de usar a região onde o consumidor de facto avalia — aqui a caixa de
  recorte da marcha (o envelope da pilha), não a bola da peça. *A cerca fica onde o perigo está, e o
  perigo está onde o avaliador olha* ([[feedback_the_fence_goes_on_the_dangerous_side_not_on_both]]).
- ⭐⭐ Uma saturação medida contra uma bola **fixa** deixa o controlo **morto** acima de certo ponto
  (aqui `0,25`, `0,50` e `1,00` davam a mesma peça); medida contra a que **acompanha** o efeito, o
  controlo continua a responder ([[feedback_a_knob_whose_range_is_derived_from_the_object_it_rewrites_is_not_idempotent]]).
