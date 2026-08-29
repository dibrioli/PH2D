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

Ver [[feedback-i-write-the-right-guard-and-do-not-gate-it]] ·
[[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]] ·
[[reference-topic-mutation-proofs]] · [[reference-topic-gate-discipline]]
