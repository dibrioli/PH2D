---
name: feedback-an-oracle-that-shares-the-law-of-what-it-judges-is-a-mirror
description: Oráculo que usa a mesma lei do produto concorda no ERRADO — a prova de mutação é quem revela
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T00:50:14.872Z
---

Um oráculo tem de ser correcto **por construção**, e não por partilhar a lei de quem ele
julga. Se ele herda a mesma regra, com a mutação aplicada os dois falham **juntos** e o
gate passa — *concordam no errado*.

Caso medido (PH2D, `the_picture_matches_an_honest_march`, 2026-08-30): a marcha de
referência andava `t += f` — **a lei do produto**. Com o divisor do campo removido, o
oráculo herdava o mesmo campo desonesto, atravessava a superfície pela mesma razão, e as
duas imagens concordavam; o gate só morria pela cláusula de **controle** (*«a peça não
está a ser desenhada»*), não pela régua que ele diz medir. Com `t += f · 0,1` — correcto
sobre um campo que exagere a distância até `10×`, quando o pior do módulo é `2×` — a
mutação passa a matar pela régua certa: `1 186` de `2 308` pixels fora de 12°, pior `77°`.

**Why:** a prova de mutação não testa só o gate; ela testa se o **oráculo** é independente.
Um oráculo lento e burro (sem JIT, sem cache, sem especialização, passo minúsculo) vale
mais do que um esperto que partilhe uma linha com o produto.

**How to apply:**
- Ao escrever um oráculo, liste o que ele **partilha** com o produto. O aceitável é o
  input (o campo, a câmera, a cena); a **lei** nunca.
- ⚠️ Se a mutação morre pela cláusula de **controle** e não pela régua principal, o gate
  ainda não prova o que diz — ajuste o oráculo, não a barra
  ([[reference_topic_mutation_proofs]], [[reference_topic_oracle_discipline]]).
- ⚠️ E o oráculo tem de usar a **mesma tolerância de paragem** do produto: um oráculo `10×`
  mais apertado mede a TOLERÂNCIA, não a lei — foram `10` pixels a `28°` numa peça correcta.
