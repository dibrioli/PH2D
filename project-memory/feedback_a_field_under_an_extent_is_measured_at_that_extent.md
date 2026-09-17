---
name: feedback-a-field-under-an-extent-is-measured-at-that-extent
description: "Um campo que varia e é aplicado sobre uma EXTENSÃO mede-se ao tamanho dessa extensão — lê-lo num ponto pode deixar o resultado PIOR do que não corrigir nada"
metadata:
  node_type: memory
  type: feedback
---

Medido em 2026-09-14 (W12 do plano `docs/Skeleton/03`). O pincel corrige a forma do dab pela
deformação da malha, para a marca sair redonda no ecrã. A deformação era lida **no ponto** debaixo do
centro do dab — e uma malha é **afim por triângulo**, logo um dab que se estende por vários recebia,
por inteiro, a deformação de um pedaço dele.

⛔⛔ **O resultado não era só «menos bom»: com um pincel grande sobre uma malha grossa a correcção
deixava a marca MENOS redonda do que não corrigir nada** (`1,383` contra `1,188`). Medir a
deformação **sobre o disco que o dab ocupa** (o melhor afim por mínimos quadrados, 8 amostras)
devolve `1,18`, e nunca piora: nas 24 células medidas é sempre melhor ou igual.

⭐ E ela **degenera no de sempre**: se todas as amostras caem no mesmo triângulo do centro, a porta
devolve a facete **sem tocar num float** — porque um ajuste sobre um afim exacto devolve o mesmo
número com ruído de `f32` por cima, e há consumidores que exigem o bit.

**Why:** ler um campo num ponto é barato e parece inócuo, e o defeito que ele produz é
**proporcional à razão entre a extensão do consumidor e a escala em que o campo varia** — invisível
nas fixturas pequenas, e capaz de inverter o sinal da cura nas grandes. O dono leu isto como
*«talvez artefato inevitável devido à natureza das deformações do mesh»*: metade certa (o resíduo de
UMA elipse por dab é inerente), metade errada (a maior parte tinha cura, com número).

**How to apply:**
- quando um consumidor tem **tamanho** (um dab, um kernel, um tile, uma janela de filtro) e o que ele
  lê é um **campo**, a pergunta à porta leva o tamanho — `mesh_uv(.., footprint)`, não `mesh_uv(..)`;
- gateie com o consumidor **grande** e com o **pequeno**: o pequeno é o que pina o tamanho de
  amostragem (amostrar a metade do raio melhora os grandes na mesma e dá ganho **zero** nos pequenos);
- e escreva a degeneração exacta como um `if`, não como esperança
  ([[feedback-an-axis-aligned-fixture-cannot-measure-a-basis]] tem a mesma forma um nível ao lado).
