---
name: feedback-mutate-the-code-not-just-the-test
description: Um gate só vale se você mutar o CÓDIGO que ele guarda — e a mutação pode revelar que o comentário do gate está errado, não o código
metadata:
  type: feedback
---

Ao escrever um gate, **mute o código que ele guarda e confirme o vermelho**. Mas o resultado tem
três desfechos, não dois — e o terceiro é o valioso:

1. **Fica vermelho** → o gate presta.
2. **Fica verde e o código está errado** → o gate é frouxo ([[feedback_loose_oracle_hides_systematic_bias]]).
3. **Fica verde e o CÓDIGO está certo** → **a sua explicação do código é que estava errada.**

O caso (3) aconteceu duas vezes no W5 do áudio (2026-07-12), e as duas vezes o comentário mentia:

- Mutei a normalização WOLA do STFT pra constante do livro (1.5) esperando vermelho nas bordas. Ficou
  **verde** — porque o *padding* já garante cobertura COLA completa em toda amostra real, e a constante
  simplesmente **acerta** naquele hop. O ganho real da WOLA é outro (funcionar em QUALQUER hop; a crate
  usa dois). O comentário afirmava um benefício que não existia.
- Mutei o downsample do spectrogram de peak-hold pra média esperando o bipe sumir. Ficou **verde** —
  porque o fixture tinha 1 s desenhado em 200 px, o que é *up*sample: o downsample nunca rodava. O
  teste "provava" uma propriedade que jamais exercitou.

**Why:** um gate verde que você não sabe explicar é indistinguível de um gate que não testa nada. A
mutação não valida só o gate — valida o **modelo mental** que você escreveu no comentário. E o
comentário é o que a próxima LLM vai ler como verdade.

**How to apply:** ao mutar, PREVEJA o vermelho e o **motivo**. Se vier verde, não relaxe o gate nem o
descarte: descubra por que o código sobreviveu. Ou o gate está frouxo (aperte-o a partir da MEDIÇÃO —
meça o algoritmo certo e o errado, ponha a barra entre os dois), ou a sua explicação está errada
(corrija o comentário, e escreva o gate que prova o benefício REAL). Nos dois casos a mutação pagou.
