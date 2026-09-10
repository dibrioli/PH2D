---
name: feedback-a-blend-never-rounds-an-edge-that-lives-inside-one-piece
description: "Um plano de chanfro construído por SOMA de campos herda o EIXO MEDIAL do que somou — e a cumeeira que nasce vive DENTRO de uma peça, onde nenhum filete lhe pega"
metadata:
  type: feedback
---

⭐⭐⭐ **Uma mistura arredonda ENTRE peças. Uma aresta que viva DENTRO de uma peça não é alcançável
por filete nenhum, por maior que ele seja.**

E há uma construção desta casa que produz exactamente isso: o plano do chanfro de um aro é
`(tampa + paredes + c)·√½` — ele **SOMA** o campo do contorno, logo herda **toda crista desse
campo**. E o campo de um contorno 2D tem sempre uma: o **EIXO MEDIAL**. Numa quina convexa
arredondada de raio `r`, o eixo medial começa no **centro do arco**, isto é, à profundidade `r`
abaixo da superfície.

⇒ **a faceta do chanfro atravessa o eixo medial sempre que `chamfer > round`**, e ali ela deixa de
ser um plano: vira um **telhado**, com uma cumeeira. A cumeeira é uma peça só ⇒ o filete não lhe
pega, e o artista reporta *«o fillet não pega todas as arestas»*.

**Why:** medido na estrela em 2026-09-08 (W142). Os vincos que sobravam ficavam **todos** no raio
`outer − round/sin α` — `0,3541` medido contra `0,3537` pela conta, e `0,4019` contra `0,4019` com
outro filete. O campo das paredes vira `141,7°` ao atravessar a bissectriz da ponta, a toda
profundidade abaixo de `round/sin α`, e é **liso** acima dela.

**How to apply:**
1. Ao ver uma aresta que o filete não alcança, pergunte primeiro *ela está ENTRE duas peças da
   mistura, ou DENTRO de uma?* — a segunda não tem cura por raio.
2. Se a peça que a contém foi construída **somando** outro campo, a crista é do campo somado. Cure
   **a montante**: dê-lhe um campo cujo nível na profundidade em causa seja liso (na estrela, um
   segundo contorno com as quinas a `round + chamfer` — aí a fronteira interior da faceta fica com
   raio `round` em planta, que **é** o filete pedido).
3. ⛔ A cura não serve a todo contorno: o eixo medial de uma **faixa** é a linha do meio dela e o de
   um **sector** parte de um ápice — ali a saída é **decompor o perfil** e entregar as peças.
4. ⚠️ Ponha um controlo que morra se a cura for *«comer material»*: o alcance da peça tem de ficar
   **igual ao bit**.

Irmãs: [[feedback_a_field_can_be_wrong_exactly_where_no_surface_ruler_looks]] ·
[[feedback_two_nearly_coincident_pieces_in_a_blend_need_the_angle_not_a_fence]] ·
[[feedback_a_worst_angle_gate_is_blind_to_the_fraction]]
