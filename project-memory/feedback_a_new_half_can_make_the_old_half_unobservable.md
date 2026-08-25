---
name: feedback-a-new-half-can-make-the-old-half-unobservable
description: Acrescentar uma segunda via que cobre o mesmo caso apaga o gate da primeira — re-corra as mutações da metade antiga depois de acrescentar a nova
metadata:
  type: feedback
---

O laço de seleção 3D do PH2D passou a apanhar por **superfície OU por origem**. Ao acrescentar a
segunda via, **três mutações que estavam vermelhas passaram a sobreviver**: sabotar a amostragem da
superfície ficava verde, porque a via da origem apanhava tudo na mesma nas fixturas existentes.

**Why:** um gate mede o **resultado**, não o caminho. Duas vias que cobrem o mesmo caso tornam cada
uma individualmente inobservável — e uma metade que nenhum gate observa é uma metade que se apaga
sem ninguém reparar.

**How to apply:** depois de acrescentar uma via alternativa, **re-corra as provas de mutação da via
antiga**. As que sobreviverem apontam para a fixtura que falta: uma em que **só** a via antiga possa
responder (aqui, formas cuja origem cai FORA do rectângulo e cujo corpo entra nele). ⚠️ E confira o
que a mutação de facto faz: uma rotulada *"só amostra o canto"* deixava uma **coluna** inteira de pé,
porque o laço tem dois `while` — [[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]].
