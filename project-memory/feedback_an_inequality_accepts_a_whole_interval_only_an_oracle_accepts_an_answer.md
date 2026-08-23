---
name: feedback_an_inequality_accepts_a_whole_interval_only_an_oracle_accepts_an_answer
description: Um gate que só afirma "o meio está ENTRE as pontas" deixa passar implementações erradas — e uma soma de áreas absolutas não é régua de REGIÃO
metadata:
  type: feedback
---

Três réguas caíram na mesma auditoria (`line/Vector`, 2026-08-23), todas provadas por mutação:

1. **A desigualdade.** O gate da receita afirmava *"o meio está entre a união e a chegada"*. Um
   mutante que fazia o cozimento devolver a **entrada crua** sobreviveu: as duas formas num
   composto medem 336, que cai no intervalo. ⇒ a barra passou a ser o **cozimento DIRETO** como
   oráculo. *Uma desigualdade aceita um intervalo inteiro; só um oráculo aceita uma resposta.*
2. **A soma de áreas absolutas não é uma régua de REGIÃO.** `Σ|área(peça)|` lê **400** e **272**
   para dois desenhos que cobrem **exactamente a mesma região** — um vem em duas peças, o outro num
   composto. A régua de região é a **diferença simétrica**.
3. **E a diferença simétrica tem de colapsar cada lado primeiro.** Encadear os dois num `Exclude`
   só dá `a ⊕ b ⊕ c`, que não é `(a∪b) ⊖ c`, e devolve **0** quando um lado é vazio (o motor recusa
   uma booleana de um operando) — ou seja, diz *"idênticos"* no caso em que eles mais diferem.

**Why:** as três ficam verdes sobre produto errado, e as duas últimas ficam verdes **por
construção** — elas nem chegam a medir o que o nome delas diz. É a mesma família de
[[reference_topic_quad_remesh_rulers]] (extremo global ≠ régua por-face) e de
[[feedback_an_unlabelled_probe_column_gets_read_backwards]].

**How to apply:** antes de aceitar um gate de geometria, pergunte *"que implementação ERRADA
passaria nisto?"* — e se a resposta existir, troque a desigualdade por um oráculo independente.
Para "estes dois desenhos são o mesmo?", a resposta é sempre **tinta** (diferença simétrica com
cada lado colapsado), nunca área somada. E `.is_some()` sobre uma projeção de painel não distingue
a face CHEIA da **face VAZIA**.
