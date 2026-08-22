---
name: feedback-a-collapsed-field-does-not-go-neutral-it-takes-over
description: O campo adaptativo colapsou numa constante 2,5× o alvo — o knob deixou de adaptar e passou a GROSSEIRAR a peça
metadata:
  type: feedback
---

Quando um campo por-elemento colapsa num valor único, ⛔ **ele não vira o caso
neutro**: ele vira **um segundo alvo**, que substitui o que o utilizador pediu.
*"O knob não faz nada"* e *"o knob faz outra coisa"* leem igual no report e pedem
consertos diferentes.

**Why:** medido em 2026-08-21 (quad remesher). O `Follow Curvature` alimentava um
campo de tamanho por vértice, recortado por `lo`/`hi` derivados de um **piso**. O
piso era o do outro motor; com um alvo mais fino que ele, `lo == hi` e todo vértice
recebia o mesmo número:

```text
campo de tamanho: min 0.2301  mediana 0.2301  max 0.2301   |  alvo 0.0910
```

⚠️ **A constante valia `2,5×` o alvo.** Então o artista mexia no knob e a peça saía
com **451 quads em vez de 1 336** — não *"igual"*, **mais grosseira**. O report que
chegou foi *"Follow curvature não funciona"*, e a caçada começou no sítio errado
(a ligação nova) em vez do sítio certo (o recorte do campo).

⭐ **E o `adapt = 0,5` e o `adapt = 1,0` davam saída IDÊNTICA.** Esse é o sinal que
nomeia a espécie: um knob sem consumidor dá saída idêntica **ao neutro**; um knob
colapsado dá saída idêntica **a si mesmo em todos os valores não-neutros**. *São
dois sintomas diferentes e o segundo é mais caro, porque a peça muda.*

**How to apply:**

1. ⭐ **Imprima `(min, mediana, max)` de todo campo antes de o consumir.** Uma
   linha. É a diferença entre horas de bissecção e um olhar.
2. **Um `clamp(lo, hi)` cujos limites vêm de outra fase é um colapso à espera:**
   pergunte *"existe entrada legítima em que `lo == hi`?"*. Se existir, ou a
   assinatura leva o limite do chamador, ou o colapso tem de ser um erro nomeado —
   nunca uma constante silenciosa ([[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]).
3. ⚠️ **Teste DOIS valores não-neutros do knob, não só ligado/desligado.** `0` vs
   `1` teria passado por um campo colapsado que difere do uniforme; `0,5` vs `1,0`
   idênticos é o que o denuncia.
4. **Um controlo SINTÉTICO e brutal separa *"não chega"* de *"a peça não pede"*.**
   Aqui um campo de `9×` de contraste realizou `2,2×` na saída — o mecanismo
   funcionava e quem comprimia era a estrutura a jusante. *Sem esse controlo, o
   `1,30×` do campo real leria como «continua partido».*

Irmã de [[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] (o
sintoma parecido, a causa oposta) e de
[[feedback_a_label_must_promise_what_the_model_delivers]].
