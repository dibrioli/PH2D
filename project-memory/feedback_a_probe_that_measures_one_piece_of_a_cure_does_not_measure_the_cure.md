---
name: a-probe-that-measures-one-piece-of-a-cure-does-not-measure-the-cure
description: "Medi a DECISÃO de um LOD a 0,17 ms e a cura inteira custava 8 ms — quem me acusou foi a FOTO da app, não a sonda"
metadata:
  type: feedback
---

Ao ligar o LOD da forma (2026-09-22), escrevi uma sonda para o preço da decisão no quadro em que
ela **não arma**: `geometrias_para_lod` deu `0,46 ms` e, depois de eu a cortar com uma saída cedo,
`0,17 ms`. Dei-a por barata e segui.

A **FOTO da app** — o `fotografa_cena.sh`, lendo a barra de estado — deu outro número: com a cura
ligada `43 raw · 24,6 ms`, com `PH2D_LOD_DA_FORMA=0` **`73 raw · 16,5 ms`**. A cura custava **8 ms**
no zoom onde ela não compra nada.

**Why:** a cura tinha **três** peças (decidir · particionar · manter o despejo honesto) e a sonda
apontava a uma. A culpada era a terceira: `vivas_com_o_lod` reconstruía o conjunto de gids das
`90 000` instâncias — num `BTreeSet` de UMA chave — ao lado de um chamador que já o tinha
construído, e depois varria `instances` outra vez. Três passagens de `90 000` por quadro, nenhuma
delas na sonda.

*Uma sonda mede o que eu lhe aponto. A foto mede o PRODUTO.*

**How to apply:** ao medir o preço de uma cura no caminho de omissão, a régua é o **produto a
correr** (a barra de estado, o `raw`), e a sonda serve para **atribuir** depois de o produto ter
dito que há um preço. E o A/B tem de ser a porta de bissecção da própria cura, na mesma máquina e
na mesma condição — nunca um número escrito num roteiro noutro dia
([[feedback-a-calm-ruler-that-does-not-name-the-build-profile-reads-as-a-verdict]]).

⚠️ E o cheiro da terceira peça é reconhecível: **um conjunto derivado que o chamador já tem**.
Quando uma função nova aceita `&[T]` e constrói um `Set` que o sítio de chamada acabou de
construir, ela está a pagar a travessia duas vezes — e o custo é invisível numa fixtura pequena.

[[reference-topic-measurement-discipline]] · [[a-geometric-ruler-and-a-pixel-ruler-decides]]
