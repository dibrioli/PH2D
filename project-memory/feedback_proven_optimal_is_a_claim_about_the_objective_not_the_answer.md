---
name: feedback-proven-optimal-is-a-claim-about-the-objective-not-the-answer
description: `gap = 0` prova que a resposta é ótima PARA O CUSTO ESCRITO — e um custo linear torna "esmagar" e "espalhar" indistinguíveis
metadata:
  type: feedback
---

Quando um solver devolve **ótimo demonstrado** e o resultado ainda é mau, ⛔ não
procure o bug no solver: **leia o custo**. `gap = 0` é uma afirmação sobre o
**objectivo**, não sobre a qualidade — e um objectivo que não distingue duas
respostas deixa o ótimo escolher qualquer uma delas.

**Why:** medido em 2026-08-21 (quad remesher, `ph2d-quantize`). O custo era
`w·|x − t|`. Uma função de valor absoluto tem **marginal constante** de cada lado
do alvo ⇒ tirar a 1.ª unidade custa o mesmo que tirar a 10.ª ⇒ **esmagar um arco e
espalhar o erro por vários custam exactamente o mesmo**. O solver provava o ótimo
e devolvia um arco que pedia 4,1 segmentos com **1** — uma corda recta de
comprimento `1,105` numa peça de raio `1,0`, e a grade construída sobre ela nascia
do lado errado da forma.

⛔ **E a "correcção relativa" piorou o sinal.** Pôr `peso = 1/alvo` sobre um custo
**linear** — para exprimir *"a qualidade de uma grade é uma razão"* — tornou
esmagar um arco LONGO **4× mais barato** que espalhar:

| escolher | `w·|x−t|`, `w = 1/t` | `w·(x−t)²` |
|---|---|---|
| esmagar `t=4,1` até 1 | `3,1/4,1 =` **0,76** | `(1−4,1)² =` **9,6** |
| espalhar por 3 arcos `t=1` | **3,0** | **3,0** |
| o ótimo escolhe | ⛔ esmagar | ✅ espalhar |

⭐ **A máquina para o custo certo já estava na crate**: `step_cost(k) =
cost(k) − cost(k−1)` e um fatiador que agrupa marginais iguais — a redução clássica
de custo convexo para fluxo linear, com um doc-comment a dizer *"é esta função que
faz o custo convexo caber num fluxo"*. **Só o custo tinha ficado linear.**

**How to apply:**

1. **Pergunte se o objectivo SEPARA as respostas que você quer separar.** Escreva
   as duas à mão e some. Se derem o mesmo, nenhum solver o vai salvar.
2. ⚠️ **Marginal constante = indiferença.** Todo custo linear por partes tem esse
   defeito no regime de deviação grande — que é exactamente o regime onde o
   resultado dói.
3. **Um princípio correcto pode ser implementado com o sinal invertido.** *"É uma
   razão"* vira `((x−t)/t)²`, não `|x−t|/t`: a razão tem de estar dentro do
   quadrado, senão o peso só desconta o preço de estragar o que é grande.
4. **Se há uma referência permissiva, leia o custo DELA antes de inventar** — aqui
   `CostFunction.hh` do libSatsuma (MIT) trazia as três formas prontas, e o
   pipeline escolhia a quadrática com peso que **não olha o alvo**.

Irmã de [[feedback_two_proofs_of_the_same_optimum_cannot_disagree]] e de
[[feedback_a_conserved_invariant_cannot_grade_quality]] — a mesma família: *o
instrumento respondia com rigor a outra pergunta.*
