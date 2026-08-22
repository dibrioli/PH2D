---
name: feedback-a-new-features-gate-can-expose-a-pre-existing-bug-check-the-control-first
description: "Antes de culpar a feature nova pelo gate vermelho, corra o gate com a feature DESLIGADA — o defeito pode ser antigo e nunca medido"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: edbb014f-4ffb-40ff-bd89-2200158288ca
  modified: 2026-08-22T12:30:53.761Z
---

Liguei o termo de alinhamento do campo cruzado e o gate de Euler reprovou: o toro
saía com `χ = 2` onde a topologia exige `0`. Ia escrever *"o alinhamento quebra a
topologia"*. Antes disso, varri a **mesma peça em vários tamanhos com o peso a
zero** — e o toro 48×24 falhava **também**, no produto, sem alinhamento nenhum.

**Why:** a feature nova não causou nada; ela **perturbou** um sistema caótico e mudou
*qual* caso cai. O defeito era pré-existente e invisível porque **nenhuma fixtura
gateada o continha** — o único toro do corpus era o 32×16, e nele ele não aparece. E
o defeito passava em **todas** as outras réguas: 100 % de quads, zero arestas de
bordo, zero não-manifold, contagem de irregulares na ordem certa. *Uma peça pode
passar em toda asserção e ter deixado de ser um toro.*

**How to apply:** quando um gate fica vermelho ao ligar algo novo, **corra o mesmo
gate com a coisa desligada e sobre mais uma variação da fixtura** antes de atribuir
a causa. E quando a causa se confirmar antiga: separe em dois testes — um que corre
verde com a afirmação verdadeira, e um `#[ignore]` com o vermelho pré-existente e o
mecanismo no doc. Um vermelho escondido dentro de um gate verde é pior que nenhum.
Irmã de [[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]].
