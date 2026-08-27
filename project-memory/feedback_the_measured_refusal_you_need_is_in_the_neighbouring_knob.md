---
name: feedback-the-measured-refusal-you-need-is-in-the-neighbouring-knob
description: A recusa medida que responde a' sua pergunta esta' no doc do knob VIZINHO — o que se repete e' a metrica, nao o botao.
metadata:
  type: feedback
---

Ao pegar numa métrica (dobras, furos, resíduo, `χ`), consulte as recusas medidas de **todos
os knobs que a mediram**, não só a do knob que vai mexer. *O que se repete entre waves é a
métrica; o botão é diferente de cada vez.*

**Why:** medido no `ph2d-gridmap` (2026-08-27). Escrevi *«é a dobra que parte o `χ`»* e
gastei uma wave a construir o tecto de grupo para a apagar. A medição desmentiu-o (dobras
de volta ao valor do controlo, `χ` na mesma em `−6`) — e a frase
*«**«Dobra» e «furo» não são o mesmo defeito**, e só uma medição a podia derrubar»* já
estava escrita, com tabela, **no doc do `RETRY_ON_FOLD`, no mesmo ficheiro que eu editei
nesse dia**, dois dias antes.

**How to apply:** antes de inferir «apagar A cura B», `grep` a métrica B nos docs do
módulo. Uma recusa medida é indexada pelo knob que a pagou, mas *responde por toda a
métrica* — e o índice pela métrica não existe, então é o `grep` que o faz.
Parente de [[feedback-a-measured-refusal-answers-one-question-recheck-it-when-yours-is-another]]
e de [[feedback-documented-decision-chesterton-fence]].
