---
name: a-runtime-machine-must-be-seeded-by-the-world-it-drives
description: Uma máquina não-serializada que escreve um componente PERSISTENTE tem de nascer semeada pelo mundo — nascer no estado inicial faz «já estou aí» recusar o voo que o artista pediu
metadata:
  type: feedback
---

Vector / Morph States, 2026-08-26. A `MorphMachine` **não é serializada** (ela é *onde a forma está
agora*; o documento guarda *quais são as setas*) e morre sempre que a pré-visualização se desliga.
Mas o `VecMorph` que ela escreveu **fica** — o ledger da `preview_drive` larga a condução e a
`settle` promove o vivo a documento no quadro seguinte.

⇒ a máquina seguinte nascia em `graph.start()` **com a cena noutra forma**. E como a máquina recusa
`travel` quando `st.shape == self.current` (*«chegar onde já se está não é chegar»* — uma regra
certa), o voo que o artista pedia era recusado sobre um «onde» que **só a máquina acreditava**.

**Why:** as duas metades eram individualmente corretas — a máquina ser efémera é uma decisão
defendida, e a recusa do voo degenerado também. O defeito nasce só da **composição** delas com um
componente que sobrevive. E a nota do módulo dizia o contrário (*"ao largar, a cena volta ao que o
artista desenhou"*), o que fazia o defeito parecer impossível de existir.

**How to apply:**
- Toda máquina/estado de runtime que **escreve** um componente persistente tem de ter um construtor
  **semeado pelo mundo** (aqui `MorphMachine::seeded(graph, showing)`), e o dono chama-o por uma
  porta só.
- ⚠️ Semente **fora** do domínio cai no início — a forma pode ter sido desconectada entre duas
  sessões do modo.
- Antes de confiar em «sair restaura», **meça**: com o ledger da `preview_drive`, sair **compromete**
  (é o *«desfaz a corrida»*, `settle`). Ver [[feedback_stale_comment_and_dead_code_lie]] e
  [[a-map-the-tick-clears-must-be-opened-not-looked-up]].
