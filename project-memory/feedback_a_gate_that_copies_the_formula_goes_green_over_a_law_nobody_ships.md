---
name: feedback-a-gate-that-copies-the-formula-goes-green-over-a-law-nobody-ships
description: "Copiar a fórmula para dentro do teste, para a régua não usar a função sob teste, deixa o gate VERDE depois de o produto mudar de lei — contra um oráculo analítico, a régua tem de atravessar o produto"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-03T16:57:25.614Z
---

Há um instinto certo — *«se eu chamar a função sob teste, o gate mede-se a si próprio»* — e ele tem
um modo de falha caro: **copiar a implementação para dentro do teste congela o teste na lei de
ontem.** Quando o produto troca de lei, o gate continua verde, e ele agora afirma uma coisa que
nenhum código executa.

Caso medido (PH2D, `line/3DModeling`, 2026-09-03). O gate do chanfro construía o plano à mão —
`(a + b + c)·√½` — com o comentário a dizer, por extenso, *«se fosse importado, a régua usaria a
função sob teste»*. Quando o operador passou a construir o plano de outra maneira, o gate **passou**
na mesma: ele media uma fórmula que já não existia no produto.

⭐ **A distinção que resolve:** quando o oráculo é **analítico** (trigonometria, uma conservação, um
invariante), a régua já não corre risco de circularidade — o oráculo não vem do código. ⇒ nesse caso
a régua **tem** de atravessar a porta do produto, senão ela deixa de medir o produto. Copiar a
implementação só é certo quando o oráculo **é** a implementação de referência (paridade CPU/GPU,
porte contra original).

**How to apply:**
- ⭐⭐ Pergunte *de onde vem o valor esperado*. Se vem de fora do código (uma conta), chame o produto.
  Se vem de outra implementação nossa, aí sim escreva a segunda por fora.
- ⚠️ Um gate que testa uma fórmula copiada devia dizê-lo no nome — `..._that_is_the_shipped_error`
  é honesto no dia em que se escreve e mentiroso no dia em que o produto muda. Prefira gatear o
  **produto** e pôr a lei antiga como **controlo** (*«e este número, o da lei antiga, tem de ser
  diferente»*), que é a forma que sobrevive à mudança.
- ⛔ Irmão de [[feedback_stale_comment_and_dead_code_lie]] e de
  [[feedback_a_registry_cannot_tell_a_missing_feature_from_a_typo_ask_the_tree]]: em todos, a segunda
  cópia de um facto é a que envelhece sem avisar.
