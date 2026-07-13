---
name: feedback_seam_gates_need_fractional_advance
description: Gate de emenda/interpolação com taxa 1:1 nasce CEGO — frac=0, o 2º frame nunca é lido, frame segurado é invisível
metadata:
  type: feedback
---

Qualquer gate sobre uma **emenda** (loop wrap, costura de stream, splice de grão) tem de rodar com
**avanço fracionário** — taxa da fonte ≠ taxa de saída.

Com 1:1 o cursor cai exatamente em cima dos frames da fonte, `frac` é sempre `0.0`, e o **segundo
frame da interpolação nunca é lido**. Então o bug clássico — segurar o último frame em vez de ler o
frame de destino do wrap — é **invisível**: o gate passa com o código quebrado.

**Why:** aconteceu 2× no ADR-0119 (regiões de loop), e o gate só ficou honesto depois de trocar o
sinal de teste para `OUT_RATE / 2` (avanço 0.5), onde um partner segurado vira um número simplesmente
errado (8.0 onde a verdade é 8.5). O 1º gate passou com a mutação aplicada; a mutação foi o único
motivo de eu ter descoberto.

**How to apply:** gate de emenda = fonte a `OUT_RATE/2` (ou 44.1k contra 48k) + stamps por frame
(`valor = índice`, escalado pra **abaixo de 1.0** — o master grampeia em unidade e todo stamp volta
como √2 se estourar). Depois **mute o código** e exija vermelho. Vale para [[feedback_derived_coordinate_seed_must_match_sample]]:
o mesmo tipo de coordenada derivada que mente sem que ninguém veja.
