---
description: O formato que faz a LLM procurar o mecanismo, não remendar o sintoma.
argument-hint: [Passos] [Esperado] [Visto]
---
Smoke reprovado.

O que eu fiz: $1
O que eu esperava: $2
O que aconteceu: $3

Antes de propor fix:
- REPRODUZA (harness ou sonda render-and-look, no caminho REAL do produto — não um
  espelho). Se não reproduzir, diga isso; não-repro ≠ fix.
- MEÇA o mecanismo e me dê o número.
- Escreva o gate RED-FIRST: ele tem de nascer vermelho sobre o defeito e a fixture
  precisa CONTER o fenômeno.
- Depois do fix, prove por mutação: reinstale o defeito e mostre o gate sangrando.

Se algum gate já existente estava VERDE sobre este bug, me diga qual e por quê — isso
é achado, não rodapé.
