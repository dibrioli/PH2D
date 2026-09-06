---
name: feedback-continuous-at-one-seam-is-not-continuous-at-every-seam
description: Provei que o ângulo desenrolado de uma espiral é contínuo no corte do `atan2` — e ele salta `2π` na OUTRA costura, a da volta mais próxima: `‖∇f‖ = 2596,5`.
metadata:
  type: feedback
---

Medido em 2026-09-05 (W123). A espiral usa `θ = φ + 2πk` com `k` arredondado a partir do raio.
Escrevi — e demonstrei — que `θ` é **contínuo no corte do `atan2`**: ali `φ` salta `−2π` e `k` salta
`+1`, e a soma não se mexe. ⛔ E usei-o para cortar as pontas da fita, que é onde ele **salta**: na
costura onde a **volta mais próxima muda**, `k` muda e `φ` não. `passo × ‖∇f‖` foi de `0,99` para
**`2596,5`**.

⚠️ **A demonstração estava certa e respondia a uma costura das duas.** A outra estava a duas linhas
de distância no mesmo ficheiro — era o `.round()` que eu próprio tinha escrito.

**Why:** uma expressão construída a partir de um `round`/`floor`/`clamp` tem **uma costura por
operador de escolha**, e a continuidade tem de ser verificada em cada uma. O `abs` que envolve a
distância à volta escolhida esconde a segunda costura (`|ρ − r_k|` é contínuo ali porque as duas
candidatas empatam) — mas qualquer outro uso de `k`, ou de `θ`, **não é**.

**How to apply:** ao dizer *«isto é contínuo»* de uma expressão com escolha, **enumere as costuras**
(uma por operador de escolha) e diga o que acontece em cada. E a régua barata que apanha tudo é o
`‖∇f‖` sobre uma grelha: um salto lê-se como um número absurdo, não como um erro subtil. Irmãs:
[[reference_topic_implicit_field_laws]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]]
