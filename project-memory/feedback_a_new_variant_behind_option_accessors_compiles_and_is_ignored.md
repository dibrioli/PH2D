---
name: feedback_a_new_variant_behind_option_accessors_compiles_and_is_ignored
description: "Variante nova num enum que o resto do código lê por ACESSORES Option (`shape()`, `connector()` com `_ => None`) compila em todo o lado e é ignorada em silêncio — procure os leitores de cada acessor"
metadata:
  type: feedback
---

MiroClone W4 (07/10): `ElementKind::Ink` (o traço da caneta) entrou no fim do enum e a família
inteira compilou à primeira — **zero erros fora do próprio modelo**. Mas o desenho (`el.shape()` →
`continue`), o clique (`geom::hit` → `false` para quem não é forma), a moldura (`retain(shape)`) e a
barra de estilo (`selected_kind` só via formas e setas) liam o elemento pelos acessores
`shape()`/`connector()`, cujo braço `_ => None` engole a variante nova: o traço existia no documento,
não se desenhava, não se clicava, não se movia — sem um aviso.

**Why:** o `match` exaustivo só protege quem faz `match` no enum; quem lê por um acessor `Option`
herdou um `_ =>` escrito antes de a variante existir, e o `None` lê-se como «não é comigo».

**How to apply:** ao acrescentar uma variante, `grep` cada ACESSOR do enum (`fn shape(`, `.shape()`,
`.connector()`) e decida em cada leitor o que a variante nova faz ali — desenhar, acertar, emoldurar,
estilizar, copiar. Só os `match` directos são o compilador; os acessores são uma lista à mão. Ver
[[feedback-an-exhaustive-match-does-not-guard-the-list-a-loop-iterates]] e
[[feedback_a_gesture_written_in_two_halves_accepts_a_new_variant_in_only_one]].
