---
name: an-exact-invariance-gate-needs-an-exact-transformation
description: Um gate de invariância exacta tem de usar uma transformação exacta em vírgula flutuante — potência de dois —, senão ele reprova por um ULP amplificado
metadata:
  type: feedback
---

Um gate que afirma *«escalar a entrada e o alvo juntos tem de dar a mesma saída»* mede a LEI — mas
só se a transformação de escala for **exacta em vírgula flutuante**. Use `s = 2^k`.

**Why:** medido em 2026-08-30 (`a_energia_nao_tem_opiniao_sobre_a_densidade`, `ph2d-gridmap`). Com
`s = 7` o gate reprovou sobre produto **correcto**: `√(49·a)` e `7·√a` diferem por **um ULP**, e a
descida de uma energia de barreira **a partir de um estado emaranhado é caótica** — o ULP no
referencial de repouso virou `0,15` na saída (a saída inteira vale `~0,5`). Com `s = 8` o
achatamento isométrico é **exactamente** escalado (a raiz de `4^k·a` só desloca o expoente), o
referencial sai bit a bit igual, e o gate volta a medir a lei.

**How to apply:** ao escrever qualquer gate de invariância por escala/translação sobre um
optimizador iterativo, escolha o factor entre `2`, `4`, `8`, `16`. E escreva **porquê** no gate —
quem lá voltar com `s = 3` vai ver um vermelho e procurar o defeito no produto. ⚠️ A regra vale
com força dobrada quando o método é caótico (descida a partir de estado inválido, guloso,
recozimento): aí um ULP não se dilui, amplifica-se. Irmã de
[[a-term-with-a-unit-bearing-minimum-imposes-its-own-scale]], que é o defeito que este gate apanha.
