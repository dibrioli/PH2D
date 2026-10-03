---
name: feedback-a-same-look-between-two-8bit-paths-closes-per-dab
description: "«O traço numa camada nova = o traço na base, ±1» só fecha por DAB: num traço cada dab arredonda a 8 bits — a base guarda a COR, a camada o ALFA — e aparecem 2s raros que nenhuma lei de composição tira"
metadata:
  type: feedback
---

Medido 2026-10-03 (ADR-0177, doc Painter 45 §8): com as camadas já em tons de ecrã, o gémeo 2D deu 1 dab `≤1` sempre e, com 2+ dabs sobrepostos, 2s em `0,006–0,03 %` dos canais, nunca 3. Um modelo SÓ das duas recursões arredondadas (`c ← round(c·(1−w))` × `a ← round(a + w·(255−a))`, composto no fim) reproduz o número sem lei nenhuma. Na peça 3D fechou ao bit.

**Why:** o critério de desistência do plano dizia «se o seam não fechar a ±1, o desvio não é só o espaço — PARE». O resíduo era o armazenamento, não o espaço; ler o 2 como reprovação mandaria refazer a lei certa.

**How to apply:** uma igualdade entre dois caminhos que arredondam a cada passo mede-se POR PASSO (um dab) com a tolerância de um degrau, e o traço inteiro com o tecto que o modelo das recursões dá; antes de declarar «não é só X», ablate X num modelo sem X. Família: [[reference_topic_measurement_discipline]].
