---
name: reproduce_with_the_real_constructors_and_look_at_the_image
description: Duas curas raciocinadas sobre a foto foram REAIS e não eram a causa; reproduzir a cena com os construtores do app e OLHAR a imagem achou-a à primeira
metadata:
  type: feedback
---

2026-09-01, o retrato de um prefab, três fotos do Enio. Da 1.ª e da 2.ª raciocinei sobre a
imagem e curei coisas verdadeiras (a forma da minoria · a janela de UV e o pivô) — nenhuma era o
que a 3.ª foto mostrava. Na 3.ª montei a cena dele com os construtores REAIS
(`Sprite::atlas`, `ChildOf`, `MasterRoot`), compus, gravei o retrato ampliado 4× com
`PH2D_PORTRAIT_DUMP=<dir>` e **li a imagem com o `Read`**: era a foto dele ao pixel. A cura
saiu em minutos, e depois de a aplicar li a imagem outra vez: era a cena.

**Why:** uma foto é uma projecção; várias causas produzem a mesma projecção, e a cabeça escolhe a
que já conhece. Um gate que grava o resultado e um olho a vê-lo eliminam o palpite. É a
[[render_and_look_when_a_green_gate_is_contradicted]] levada ao ponto de partida — ANTES de
propor a causa, não depois de o gate falhar.

**How to apply:** um report com foto sobre uma coisa que se DESENHA começa por um gate que
reproduz a cena com os construtores do produto e grava a imagem (uma env var, PNG ampliado).
Leia-a. Só depois nomeie a causa. O gate fica: ele é o que carrega o fenómeno.
