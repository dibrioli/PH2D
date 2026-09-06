---
name: feedback_a_hard_max_is_an_intersection_without_a_radius_and_that_is_a_live_edge
description: Compor duas peças de um SDF com `max`/`min` cru deixa a aresta VIVA — o filete e o chanfro entram pela junta, e uma composição sem junta não tem por onde os receber.
metadata:
  type: feedback
---

Num modelador de campo implícito, `max` é a intersecção e `min` a união — **sem raio**. Toda aresta
que nasce de um `max`/`min` cru fica **viva**, e nem o filete nem o chanfro lhe chegam: os dois
entram pela **junta** (`intersection_joint` / `union_joint`), e uma composição que não usa junta não
tem por onde os receber.

**Why:** medido três vezes na mesma jornada (formas 3D, 2026-09-05). A **faixa** subtraía o entalhe
com `corpo.max(entalhe)` e as quinas onde ele encontra o contorno liam `75,2°` de vinco. A **chave**
unia os arcos com a união crua e o chanfro cortava `54,8 %` das arestas — os pontos que ficavam
estavam **todos no nariz**. E o **arco** fechava o sector com `max`, deixando as duas pontas vivas.

**How to apply:** pergunte de cada `max`/`min` do seu perfil *«a aresta que isto cria está na
FRONTEIRA da peça?»*. Se está, é junta. ⚠️ **E a junta tem duas rotas**: com o chanfro a zero use a
binária (ou a união n-ária **crua**), porque os planos de corte que a n-ária acrescenta passam pela
própria aresta, não cortam nada e **contam à mesma** para o `length` da mistura — o tecto dela é
`√(activas)`; com chanfro use a n-ária, e entregue as peças INTEIRAS, porque um perfil já composto
leva a costura dele para dentro do aro.

⚠️ **Duas juntas ENCAIXADAS têm o mesmo defeito**: a segunda recebe a composta da primeira. Um
triângulo fechado por `joint(joint(a,b), c)` lia `37,2 %` da superfície sobre um vinco; os três
cantos numa mistura só levaram-no a `13,4 %`, e o pior vinco de `48,3°` para `31,0°`.
