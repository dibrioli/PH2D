---
name: a-per-texel-albedo-is-the-base-color-never-a-factor-at-the-end
description: "Multiplicar a resposta de um BSDF pela cor do texel TINGE o destaque especular — a cor entra como `base_color`, dentro da lei"
metadata:
  type: feedback
---

Um consumidor com albedo **por-texel** (um sprite, uma textura) é tentado a acender
uma vez com o material e **multiplicar** a resposta pela cor do texel. Está errado,
e não por pouco: um BSDF **não é linear no `base_color`** — só o lóbulo difuso
escala com ele, o especular não escala nada.

Medido (2026-09-20, OpenPBR de omissão, destaque de um dieléctrico):

```
  albedo              multiplicar depois       base_color = albedo     razão R/B
  [0,80 0,10 0,10]   [0,5096 0,0637 0,0637]   [0,6370 0,4251 0,4251]   8,00 → 1,50
  [0,05 0,05 0,90]   [0,0319 0,0319 0,5733]   [0,4099 0,4099 0,6673]   0,06 → 0,61
  [0,80 0,80 0,80]   [0,5096 0,5096 0,5096]   [0,6370 0,6370 0,6370]   1,00 → 1,00
```

⇒ multiplicar depois dá a um **plástico vermelho um destaque vermelho**, que é o
que um METAL faz.

**Why:** ⚠️ **a linha CINZENTA é o que faz isto passar despercebido** — num albedo
sem matiz as duas leis dão a mesma razão entre canais. *Uma régua corrida só em
cinzento aprova as duas*, e foi assim que o gate que eu escrevi primeiro ficou
verde sobre a composição errada. E nenhuma suíte de paridade a apanha: a paridade
mede a lei contra ela própria nos dois motores, e a composição errada está do lado
de fora dela, no consumidor.

**How to apply:** a cor de um texel entra **como `base_color`, dentro da lei** —
uma porta na crate da óptica (`Surface::at_base_color`), irmã da que já lá está
para a curvatura, e não uma multiplicação no consumidor. ⭐ Ela costuma ser
**barata**: das grandezas que um `prepare()` deriva, ver quantas dependem mesmo da
cor (no OpenPBR é **uma**, e ela é inerte sem verniz) ⇒ sem verniz é uma troca de
campo, medida a `~3 %`. E o gémeo em WGSL escreve a mesma ranhura do material
empacotado antes de compor, como o passe já faz com a curvatura. Ao escrever a
régua, **ponha a lei rejeitada DENTRO dela**: uma barra sem o lado errado ao lado
não separa nada. Ver [[a-law-written-in-one-place-is-not-a-law-only-a-door-is]].
