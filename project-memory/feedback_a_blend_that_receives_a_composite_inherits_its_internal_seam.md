---
name: feedback-a-blend-that-receives-a-composite-inherits-its-internal-seam
description: Mistura que recebe uma COMPOSTA põe o vinco interno dela na superfície — construa em ETAPAS e paga na aresta
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T01:28:33.226Z
---

Um sólido montado em **etapas** entrega à 2.ª mistura a 1.ª **já composta**, e ela **herda o
vinco interno dessa composta e põe-no na superfície visível**. Vale para qualquer
`smooth-min`/`smooth-max`: o operador alcança `r` para dentro de cada peça, e se houver uma
costura de `max`/`min`/`abs` a menos de `r` da superfície, ela aflora.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30, 3.º report do Enio — *«algumas arestas não
arredondam»*). Uma causa, três disfarces:
- **prisma**: paredes fecham num sítio, aro noutro ⇒ os pontos duros ficam **no aro, a ~12° de
  uma quina lateral** (o ponto triplo). `4,2° → 21,1°`.
- **caixa**: o `box_at` é um `max` cujo vinco é a bissetriz da quina, a `c/2` da faceta do
  chanfro ⇒ com filete `> c/2` o raio efectivo **COLAPSA** em vez de crescer (`0,086` a `r=0,04`,
  `0,006` a `r=0,12`) — *um slider que piora ao subir*.
- **cilindro**: o plano do chanfro carrega o `|z|` da laje dobrada ⇒ costura no **equador**.

Cura: **as peças do sólido E os planos de chanfro das arestas que elas formam, numa mistura só.**

**Why:** decompor não é elegância — é a única forma de a mistura não ver uma costura.

**How to apply:**
- Ao compor arredondamentos, pergunte de cada argumento: *isto é uma superfície ou uma
  composta?* Se for composta, onde está o vinco dela e a que distância da superfície?
- ⛔ **Mais peças NÃO é sempre melhor:** a mistura encolhe `≈ r(√k − 1)` com `k` peças
  **activas**, então separar uma dobra cujos dois lados estão activos ao mesmo tempo come material
  a dobrar — uma viga fina ficou com **`0` de `64 000`** células dentro. Separe só quando os dois
  lados não podem estar activos (`chanfro + filete < 2·meia`).
- ⭐ **O tecto de `‖∇f‖` é `√(ACTIVAS)`, não `√(total)`**: 19 peças com 3 activas mediram `0,695`
  contra `1,085` de duas misturas encaixadas — *mais peças saiu mais barato*.
- ⚠️ A régua é o **giro da normal**, nunca o volume: um chanfro deslocado tira o mesmo volume que
  um arredondado ([[feedback_a_fidelity_ruler_has_two_directions_and_only_one_sees_an_amputation]]).
  E a barra tem de ser uma **RAZÃO** contra a mesma peça sem o recurso — uma barra absoluta ou
  branqueia a forma fácil ou reprova a difícil.
- ⚠️ Uma função que recebe **coordenadas** tem de as usar: construir a partir de `Tree::x/y/z`
  quebrou o chamador que passava expressões dobradas, e a peça **desapareceu sem erro nenhum**.
