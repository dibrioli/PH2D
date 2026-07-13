---
name: feedback-mutate-the-code-not-just-the-test
description: Um gate só vale se você mutar o CÓDIGO que ele guarda — e a mutação pode revelar que o comentário do gate está errado, não o código
metadata:
  type: feedback
---

Ao escrever um gate, **mute o código que ele guarda e confirme o vermelho**. Mas o resultado tem
**quatro** desfechos, não dois — e os dois últimos são os valiosos:

1. **Fica vermelho** → o gate presta.
2. **Fica verde e o código está errado** → o gate é frouxo ([[feedback_loose_oracle_hides_systematic_bias]]).
3. **Fica verde e o CÓDIGO está certo** → **a sua explicação do código é que estava errada.**
4. **Fica verde e a MUTAÇÃO é que era cega** → você cortou um termo **inerte**. O gate presta, o
   comentário presta; o seu bisturi é que errou o alvo.

O caso (3) aconteceu duas vezes no W5 do áudio (2026-07-12), e as duas vezes o comentário mentia:

- Mutei a normalização WOLA do STFT pra constante do livro (1.5) esperando vermelho nas bordas. Ficou
  **verde** — porque o *padding* já garante cobertura COLA completa em toda amostra real, e a constante
  simplesmente **acerta** naquele hop. O ganho real da WOLA é outro (funcionar em QUALQUER hop; a crate
  usa dois). O comentário afirmava um benefício que não existia.
- Mutei o downsample do spectrogram de peak-hold pra média esperando o bipe sumir. Ficou **verde** —
  porque o fixture tinha 1 s desenhado em 200 px, o que é *up*sample: o downsample nunca rodava. O
  teste "provava" uma propriedade que jamais exercitou.

O caso (4) aconteceu no Multiband (2026-07-13). O crossover de 3 vias compensa a fase da banda grave
rodando-a pelo **allpass** do 2º crossover (`LP4 + HP4`). Mutei removendo **só a metade high-pass** e
esperei o dip. Veio **verde** — e quase concluí "o gate é cego". Não era: uma década abaixo de f2 essa
metade está ~−96 dB abaixo, então tirá-la muda a soma em **0,001 dB**. O que falta à banda grave na
árvore ingênua é a **FASE** daquele estágio, não a energia — a mutação tinha que bypassar o **estágio
inteiro**. Feito isso: −0,1135 dB, vermelho como previsto.

**Why:** um gate verde que você não sabe explicar é indistinguível de um gate que não testa nada. A
mutação não valida só o gate — valida o **modelo mental** que você escreveu no comentário. E o
comentário é o que a próxima LLM vai ler como verdade. Mas a recíproca também vale: **"a mutação não
mordeu" não prova que o gate é frouxo** — pode ser que você tenha removido algo que nunca carregou o
peso.

**How to apply:** ao mutar, PREVEJA o vermelho **e a magnitude**. Se vier verde, três perguntas, nesta
ordem: (a) *o que eu removi realmente carregava o efeito?* (senão, mute o estágio inteiro, não um
termo dele); (b) *o gate está frouxo?* (aperte a partir da MEDIÇÃO — meça o algoritmo certo e o
errado, ponha a barra entre os dois); (c) *a minha explicação está errada?* (corrija o comentário e
escreva o gate que prova o benefício REAL). Nos três casos a mutação pagou.
