---
name: feedback_the_hard_edge_the_eye_sees_is_the_dark_rim_inside_the_blot
description: Num meio com escurecimento de orla, a «borda dura» que o olho vê é o contorno escuro POR DENTRO da mancha — alisar a silhueta (AA, forma) não se vê; amaciar a TINTA numa faixa da orla sim
metadata:
  type: feedback
---

O Smooth Edges do Wet Paint passou por três leis que mexiam só na SILHUETA da mancha (`pa ≥ SS0`): o AA 3×3
da Aquarela, o slider a misturá-lo, e um raio a alisar a FORMA (contorno −55 % medido) — o dono: «no máximo
não vi benefício». O que se via duro era o contorno ESCURO (o `EdgeDarkening`), por dentro da mancha, com
escadinha de pixel. A lei aprovada («maravilhoso», 10/10) desfoca a tinta premultiplicada por uma tenda de
raio `r` numa faixa à volta da orla, com peso `4·B·(1−B)` (`B` = a mancha desfocada).
E a borda de uma tinta SOBRE outra (4.º smoke) também é um salto de tom dentro da mancha: detecta-se pela
tinta desfocada a `r` px de cada LADO (no grão os dois lados são iguais; o declive local confundia o grão
com um degrau e borrou-o −38 %), com limiar medido contra «o S aprovado não muda».
E uma LINHA fina (a da Heavy Edge, 5.º smoke) não é um degrau — os dois lados são iguais; detecta-se pelo
meio contra a média dos lados, e só se for COMPRIDA (as células da aguada têm o mesmo contraste mas são
curtas: sem esse critério o S aprovado mudava 6 713 px) e só na direção da GRAVIDADE (as listras das
cerdas também são compridas, mas seguem o traço: em todas as direções o miolo de um traço deitado mudava −26 %).
Doc: `docs/Painter/NewFeaturesWetPaint/14_smooth_edges_a_borda_suave.md`.

**Why:** uma régua de forma (contorno, área) aprovou a 3.ª lei e ela era invisível; a métrica que acompanhou o
olho foi a ASPEREZA (soma das diferenças de escuro entre vizinhos).
**How to apply:** antes de construir um «suavizar bordas», fotografe a borda a 8× e pergunte ONDE está o
salto de tom que se vê — na silhueta ou num contorno interno; meça pela aspereza, não pelo contorno.
