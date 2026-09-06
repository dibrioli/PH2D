---
name: feedback_a_deep_copy_that_copies_every_component_also_copies_the_identity_link
description: "Uma cópia profunda que leva TODO componente registado leva também o ELO — e duas entidades a reclamar a mesma identidade são um sósia que não se deixa mover"
metadata:
  type: feedback
---

Uma cópia profunda genérica (*«copia todo componente registado, verbatim»*) está certa para
**valores** e errada para **elos de identidade**. Um elo diz *«eu sou a peça X do mestre»*; copiá-lo
produz **duas** entidades a dizer a mesma coisa, e o consumidor a jusante escolhe uma.

Medido em 2026-09-06 (`line/components`). O *Duplicate* de uma peça dentro de uma cópia de
componente levava o `InstanceOf` verbatim. O passe de sincronização indexa as peças num
`BTreeMap<StableId, Entity>` — o segundo **tapa** o primeiro — e depois reescreve **os dois** com os
bytes do mestre. ⇒ o duplicado era um **sósia que não se deixa mover**: o artista arrastava-o e ele
voltava, e nada na tela dizia porquê.

⚠️ **O sintoma não aponta para a cópia.** Ele lê-se como *«o arrastar está partido»* ou *«a
sincronização está agressiva»*, e as duas pistas levam para o passe — que está correcto. O que
estava errado nasceu um gesto antes.

⛔ **E a cura NÃO é «tirar o elo sempre».** A mesma porta serve dois sujeitos: duplicar a **raiz** de
uma cópia tem de dar uma segunda cópia (o elo fica), e duplicar uma **peça** dela tem de dar um
objecto novo (o elo sai). Um `remove` incondicional transforma o gesto num *Detach* silencioso — o
objecto fica igual na tela e deixa de seguir a receita para sempre. A pergunta certa já existia como
porta (`is_a_recipe_given_piece`), e a **largura** dela é load-bearing.

⚠️ **A cópia RASA que existia antes acertava nisto por acidente** — ela levava quatro componentes e
nenhum era um elo. *Uma generalização correcta pode reintroduzir um defeito que o caso particular
evitava sem saber.*

**Why:** o modo de falha é mudo em toda a cadeia — compila, corre, não estoura, e o gate que existia
media a peça original. O que o apanhou foi um **smoke** que pedia ao gesto para produzir um estado
que nenhuma fixtura tinha.

**How to apply:** ao escrever ou alargar uma cópia profunda, liste os componentes que carregam
**identidade** (elos, ids de documento possuído, marcadores de papel) e decida um a um — nunca por
omissão. E quando a decisão depender do sujeito, ela é uma **porta com nome**, não um `if` no sítio
da cópia. Ver [[feedback_the_representation_can_delete_the_special_case]] e
[[feedback_the_content_of_an_asset_is_shared_only_which_asset_is_per_object]].
