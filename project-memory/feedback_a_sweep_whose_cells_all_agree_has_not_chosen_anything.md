---
name: feedback-a-sweep-whose-cells-all-agree-has-not-chosen-anything
description: Varri 30 combinações para escolher duas constantes, todas leram o mesmo, e eu quase escrevi o número como se a varredura o tivesse escolhido
metadata:
  type: feedback
---

Uma varredura de parâmetros existe para **escolher**. Se todas as células da grade
lerem praticamente o mesmo número, ela **mediu saturação, não a lei** — e a cura é trocar
a **GRANDEZA**, nunca refinar a grade.

**Why:** medido em 2026-08-25 (`motion.wave`, a borda absorvente da folha 06). Para
escolher a largura e a mordida da esponja varri `5 × 6` combinações medindo *a energia
que sobra ao tique 400*. Saiu isto:

```
  cells\str  0.15    0.25    0.35    0.50    0.75    1.00
        2   0.052%  0.205%  0.145%  0.047%  0.086%  0.227%
        4   0.031%  0.184%  0.127%  0.006%  0.141%  0.219%
        8   0.074%  0.036%  0.049%  0.064%  0.008%  0.077%
```

Trinta células entre `0,003 %` e `0,24 %`, **sem monotonia em eixo nenhum** — o que varia
de célula para célula é ruído. Aquela grandeza responde *"qualquer esponja mata a
ressonância a longo prazo?"* — **sim**, e essa é a pergunta do GATE, não a da escolha.
A grandeza que distingue é o **ECO**: o que volta da parede ao miolo depois de a frente
de saída já o ter deixado. Com ela a mesma grade passou a dizer uma lei:

```
  cells\str  0.02    0.05    0.10    0.15    0.25    0.50    1.00
        2   55.81%  45.36%  37.95%  30.10%  31.54%  37.49%  46.49%
        6   50.72%  44.00%  27.77%  25.99%  32.30%  33.97%  40.16%
```

⭐ Um **U** na mordida, com as duas pontas ruins por razões **diferentes** (fraca mal
absorve · forte reflecte na própria escada de impedância), e a largura a pagar até um
joelho. E ⚠️ **a 1.ª grade útil tinha o mínimo na PRÓPRIA BORDA dela** (`0,15`, a coluna
mais à esquerda) — isso não é «a borda é a resposta», é **a grade acabou cedo demais**;
estendê-la para `0,02` revelou o outro braço do U.

**How to apply:** antes de escrever a constante que uma varredura deu, olhe a
DISPERSÃO da grade. Sem espalhamento e sem monotonia, você não tem um óptimo — tem uma
régua saturada, e o número que escrever será o da célula que o ruído sorteou. E se o
melhor par cair na borda da grade, **estenda a grade** antes de o adoptar. Irmãs:
[[feedback_ask_what_number_the_opposite_answer_would_print]] ·
[[feedback_a_global_extreme_is_not_a_per_face_ruler]] ·
[[feedback_one_ruler_measures_one_clock]].
