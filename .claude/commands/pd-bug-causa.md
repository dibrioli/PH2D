---
description: Mecanismo, medição, e o gate que estava verde.
argument-hint: [Sintoma] [Módulo]
---
Bug: $1

Não remende. Ache o MECANISMO:
1. Reproduza no caminho real (não num espelho da sequência).
2. Isole com medição — números, não teoria. Se sua hipótese cair, diga que caiu.
3. Antes de trocar um componente: suspeite do CHAMADOR (trocar o componente esconde a
   causa e costuma trazer um 2º defeito).
4. Verifique se algum gate existente estava VERDE sobre isto — e por quê.
5. Fix + gate red-first + prova de mutação.
6. Registre em docs/$2/BUGS_*.md se a causa enganava.

Se o comentário/doc ao lado do código afirma algo que o código não faz, o comentário
é parte do bug: corrija os dois.
