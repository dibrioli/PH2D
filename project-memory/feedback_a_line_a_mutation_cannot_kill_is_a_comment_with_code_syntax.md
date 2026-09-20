---
name: a-line-a-mutation-cannot-kill-is-a-comment-with-code-syntax
description: Mutação sobrevivente sobre uma heurística — julgue-a no corpus inteiro, nunca numa peça; se trocar de sinal entre peças, apague-a com a tabela ao lado.
metadata:
  type: feedback
---

Uma linha que a prova de mutação não consegue matar é **um comentário com sintaxe de
código** — mas a cura depende do que ela é. Se for **LEI**, falta o gate. Se for
**HEURÍSTICA**, a régua é o **corpus inteiro**, e uma amostra de UMA peça produz as duas
conclusões opostas da mesma medição.

**Why:** medido em 2026-09-20 (o corte do atlas UV): uma heurística de dez linhas leu
`227 → 226` peças numa escultura e `122 → 129` noutra — apagá-la ganha numa e perde
noutra. Com uma peça só eu teria escrito *«é inerte»* ou *«vale 5 %»*, as duas erradas.

**How to apply:** ao ver uma mutação sobreviver, pergunte *lei ou heurística?* · lei ⇒
escreva o gate · heurística ⇒ corra o corpus **todo**, e se ela trocar de sinal, **apague**
e deixe a tabela no comentário para quem a quiser reconstruir saber o que compra. Ver
[[reference_topic_mutation_proofs]].
