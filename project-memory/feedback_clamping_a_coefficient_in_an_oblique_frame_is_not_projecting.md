---
name: feedback-clamping-a-coefficient-in-an-oblique-frame-is-not-projecting
description: "Um max(coeficiente, 0) só é a projecção no cone quando a base é ORTOGONAL — num referencial oblíquo ele acerta no vértice e desloca a face, e o gate do vértice não o vê"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-02T23:56:18.023Z
---

Fórmulas de distância com `max(x, 0)` em cada componente (o padrão de toda SDF publicada) escondem
uma suposição: **o `max(0)` é a projecção no cone só porque a base é ortonormal**. Generalizar a
fórmula para uma base **oblíqua** sem generalizar o recorte dá uma função que acerta onde todos os
termos estão activos e **mente onde algum foi cortado**.

Caso medido (PH2D, `line/3DModeling`, 2026-09-02): o filete de duas faces a ângulo `2α`. Trocar
`√(u⁺² + v⁺²)` por `√((u⁺² + v⁺² − 2c·u⁺·v⁺)/(1−c²))` dá o **arco exacto no vértice** — e onde só
uma face está activa (`v⁺ = 0`) devolve `u/√(1−c²)` em vez de `u`, ou seja **desloca a face plana**
de toda a peça. Medido: `0,0 % → 3,3 %` de superfície sobre um vinco, pior giro `67,6°`.

A cura é recortar nas **coordenadas duais**: com `Δ = s·n_a + t·n_b` (e `u = s + c·t`),
`s⁺ = max(s + c·min(t, 0), 0)` degenera exactamente em `u` quando a outra face deixa de contar, e
`s = u` na fronteira ⇒ as regiões coincidem. ⭐ E `s + c·min(t,0)` é `min(s, u)` num diedro obtuso e
`max(s, u)` num agudo — uma operação, escolhida ao **compilar**.

**How to apply:**
- ⭐⭐⭐ Ao levar uma fórmula de distância a um referencial não-ortogonal, **generalize o RECORTE junto
  com a norma**. Um `max(componente, 0)` transportado tal e qual é a suposição antiga a viajar
  escondida.
- ⭐⭐ **Um gate sobre o VÉRTICE não vê uma face deslocada**: ele mede um ponto, e a face é todo o
  resto da peça. Toda lei de canto precisa de **duas** réguas — o recuo no vértice **e** o valor
  longe dele, onde a resposta tem de ser a distância à face, ao dígito.
- ⚠️ O sintoma é traiçoeiro: a primeira versão passou o gate novo (o do arco) e reprovou **três**
  gates antigos que ninguém associaria a um canto.
