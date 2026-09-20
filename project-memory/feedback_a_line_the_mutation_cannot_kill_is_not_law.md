---
name: feedback-a-line-the-mutation-cannot-kill-is-not-law
description: Um guarda cujo ramo é inalcançável, e uma cerca a jusante do sítio onde o estrago acontece — as duas leem-se como cuidado e não protegem nada
metadata:
  type: feedback
---

Na lei nova das alças escrevi dois guardas defensivos e **as duas mutações que os apagavam
sobreviveram**:

- o **determinante** do sistema `2×2` — a matriz depende só dos `t` das amostras, logo é a MESMA em
  todo segmento de toda forma: uma **constante**, e o ramo `if det.abs() < 1e-12` é inalcançável;
- uma cerca de **`NaN`** sobre o resultado — tudo o que chegasse assim já teria passado pela lei
  ingénua, que corre **antes** e escreve o `NaN` no desenho. *Uma cerca a jusante do sítio onde o
  estrago acontece protege o quê?* (Medido: com uma mancha de centro `NaN` a forma desaparece **com
  e sem** a cerca.)

**Why:** os dois lêem-se como cuidado numa revisão, e nenhum custa um teste vermelho quando é
apagado. É a forma mais barata de código que finge ser lei.

**How to apply:** ao escrever um guarda, pergunte **de que variável ele depende**. Se ela é
constante por construção, o ramo é inalcançável — apague-o e escreva porquê. Se o estrago acontece
antes, mova a cerca para lá ou não a escreva. ⭐ E o inverso: quando uma mutação sobrevive, a
primeira hipótese não é *«falta um gate»* — é *«isto não é lei»*. Relacionado:
[[reference-topic-mutation-proofs]] · [[feedback-a-ruler-that-is-the-law-approves-any-law]]
