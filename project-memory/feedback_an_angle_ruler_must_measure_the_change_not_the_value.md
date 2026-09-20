---
name: an-angle-ruler-must-measure-the-change-not-the-value
description: Uma régua de ângulo que lê o VALOR acusa as quinas que o artista desenhou; a régua é a MUDANÇA contra uma lei que preserva o que a fonte tinha.
metadata:
  type: feedback
---

**Uma régua que lê o ÂNGULO de um nó não distingue uma quina AUTORADA de uma quina CRAVADA.** Num
rectângulo ela lê `90°` em repouso e acusa quatro defeitos que não existem.

⇒ a régua é a **MUDANÇA** contra uma referência que preserva, por construção, o que a fonte tinha —
e essa referência costuma já existir no produto (PH2D, 2026-09-19: a lei ingénua da pele, que aplica
um afim por vértice).

⚠️ **Duas armadilhas medidas na mesma régua:**

1. **Um nó cuja alça é degenerada não tem ângulo** — devolva `None` e **nunca** um salto: saltá-lo
   desalinha o emparelhamento com o outro estado do mesmo caminho, e a régua passa a comparar nós
   diferentes. Com um piso de população ao lado, senão ela lê zero por vácuo.
2. **A fixtura tem de ter tangentes a sério.** Um `RoundRect` desta casa parece curvo na tela e é
   **recto na FONTE** — o arredondamento é um `corner_radius` dentro do vértice, resolvido só no
   `cooked()`. Medido: a régua leu `0,000°` dos dois lados sobre o defeito, e a fixtura que contém o
   fenómeno é uma ELIPSE. *Uma forma que parece curva na tela pode ser recta na fonte.*
