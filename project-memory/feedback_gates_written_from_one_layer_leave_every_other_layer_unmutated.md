---
name: gates-written-from-one-layer-leave-every-other-layer-unmutated
description: Uma feature que atravessa camadas (fórmula → API do documento → painel) ganha gates escritos da camada onde ela foi PENSADA — e toda mutação nas outras sobrevive; conte as camadas antes de contar os gates
metadata:
  type: feedback
---

3DModeling / W101 (formas novas: cone, cápsula, prisma), 2026-08-29. Escrevi 9 gates e corri 10
mutações. **Cinco sobreviveram — e as cinco estavam fora da camada de onde eu tinha escrito os
gates.**

A feature atravessa **três** camadas:

| camada | o que é | gates que eu escrevi |
|---|---|---|
| **fórmula** (`ops.rs`) | o campo que sai da forma | **todos** |
| **API do documento** (`round_limit`, `bounding_radius`, `set_round`, `validate`) | o que a peça aceita e diz de si | zero |
| **painel** (o piso/teto do slider) | o que a mão alcança | zero |

As mortas foram todas de fórmula (a parede não normaliza, a cápsula não prende o `z`, o prisma põe
a parede na quina). As sobreviventes foram: o `bounding_radius` trocado por uma hipotenusa · o
`round_limit` sem o termo da inclinação · o `set_round` a esquecer as formas novas · o piso do
slider a voltar ao literal `1` · e o recuo exacto do filete — este último porque **todos** os meus
gates de cone usavam `round = 0`.

**Why:** eu escrevo os gates **de onde estou a pensar**. A wave era *«qual é a fórmula certa?»*, e
por isso as afirmações que me ocorreram foram sobre o campo. As outras camadas não são «detalhe»:
elas são onde o defeito é **silencioso** — um `bounding_radius` pequeno demais corta a peça nas
pontas e o artista culpa a forma; um `set_round` que cai num braço vazio devolve `Ok` e não escreve
nada; um piso de slider errado faz o controle saltar debaixo do dedo.

⚠️ E o quinto é o mais instrutivo: **gatear a forma sem o filete é gatear metade de um módulo cujo
argumento é o filete.** O parâmetro que a wave acrescentou (`round·√(1+m²)`) não era tocado por
gate nenhum.

**How to apply:**
- Antes de contar gates, **conte as camadas** que a feature atravessa e escreva uma linha por
  camada. Se uma delas não tem nome de gate, ela não está coberta.
- Para cada **parâmetro novo**, pergunte: *que gate corre com ele diferente do default?* Se todos
  os gates usam o valor neutro, o parâmetro está por medir.
- ⭐ A cura barata é um **censo derivado da enumeração** (aqui `PrimitiveKind::ALL`, com um
  `match` exaustivo a dar um representante): uma pergunta por camada, e a próxima forma herda-as
  todas por erro de compilação.
- ⛔ Cinco sobreviventes com o mesmo mecanismo não são cinco buracos — são **um**, e remendá-los um
  a um deixa o sexto para a wave seguinte.

## ⚠️ O caso irmão, no MESMO dia: entradas de input em vez de camadas

W100, e o smoke do Enio: *«o modal não funciona, não fecha. Os modelos do modal não são criados.»*
**Um mecanismo, dois sintomas.** O módulo 3D reclama o ponteiro **antes** do despacho de chrome, e
reclama todo gesto que começa dentro do rectângulo que ele desenhou — a paleta cobre-o. O clique
morria ali, e é o handler da paleta que regista o pick **e** que a fecha.

Eu tinha curado a metade do **teclado** na mesma wave (achei-a a raciocinar sobre a ordem do
roteador) e **shipei a do ponteiro partida**. As entradas eram quatro — tecla, clique, movimento,
roda — e eu gateei a que estava a construir.

⚠️ **E a guarda que parecia cobrir isto respondia «não»:** o `cursor_over_hero_chrome` pergunta *«há
um PAINEL por cima?»*, e um painel publica um rect. **Um modal de tela cheia não é um painel** — não
publica rect nenhum. *Uma guarda que faz a pergunta quase certa é mais perigosa do que nenhuma,
porque parece cobrir o caso.*

⇒ **Sempre que um módulo agarra input antes do despacho de chrome, ele deve uma pergunta ao modal**,
e a pergunta vive numa **porta com nome** que todas as entradas leem (aqui `field3d_yields_to_modal`)
— com o **soltar** deliberadamente de fora, porque um gesto em curso tem de poder acabar.

Ver [[feedback-i-write-the-right-guard-and-do-not-gate-it]] ·
[[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]] ·
[[reference-topic-mutation-proofs]] · [[reference-topic-gate-discipline]]
