---
name: feedback-a-never-used-warning-measures-visibility-not-the-law
description: O `warning: function is never used` apanhou um roteiro de smoke sem chamador — e teria ficado mudo se a função fosse `pub`
metadata:
  type: feedback
---

Uma cena de smoke nova nasceu com o roteiro de seis passos escrito e **nenhum
chamador**: o despacho que os imprime enumera as cenas **à mão**, e acrescentar um
ficheiro não acrescenta uma linha lá. O artista abriria a cena e veria a peça sem
uma palavra sobre onde clicar — que é a metade do smoke em que ele *aprende* a
ferramenta (`CLAUDE.md` §0.8).

O que o apanhou foi `warning: function is never used`.

**Why:** isso é **sorte que não se repete**. O `dead_code` alcançou-a porque ela é
`pub(crate)`; bastava ser `pub`, ou ser citada por um único teste, e o aviso
desaparecia com o roteiro na mesma mudo. *Um aviso do compilador mede
visibilidade, não a lei.* A mesma forma vale para toda "lista escrita à mão ao
lado de uma população derivável" — o despacho de cenas, um registo de painéis, um
`match` de níveis.

**How to apply:** quando um aviso do compilador apanhar uma lacuna de PRODUTO,
escreva o censo que a apanharia sem ele — varrer a população (os ficheiros, o
`enum`, o directório) e exigir que cada membro apareça na lista. ⚠️ Com **piso de
população**: uma varredura partida devolve lista vazia e `vazio.is_empty()` lê-se
como aprovado. Ver [[reference_topic_gate_discipline]].
