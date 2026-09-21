---
name: feedback_a_naming_convention_that_an_oracle_harvests_cuts_both_ways
description: Um prefixo que um oráculo COLHE (`fase_*`) obriga tanto a usá-lo como a NÃO o usar — e a regra escrita só guarda metade
metadata:
  type: feedback
---

**Quando um oráculo colhe funções por PREFIXO, o prefixo passa a ser uma AFIRMAÇÃO — e ela pode ser
falsa nos dois sentidos.**

Medido em 2026-09-21 (`shells/desktop/render_loop`). A regra que este repo tinha escrita, da wave da
FÁBRICA, diz: *«a fase-filha teve de se chamar `fase_*`: o texto emendado do quadro colhe só essas,
e com outro nome ela desaparece do oráculo de toda lei de ordem desta shell, em silêncio»*. Eu segui-a
— cortei uma fase de `201` para `~150` linhas extraindo um bloco, chamei-lhe `fase_ponte_do_sculpt3d`
— e o gate `the_frame_text_is_the_whole_frame_and_every_phase_is_called` **reprovou**: *«fases ÓRFÃS
— definidas e nunca chamadas pelo quadro»*.

**Why:** o emendador não colhe só pelo nome; ele **substitui cada fase no sítio onde o QUADRO a
chama** (`self.fase_x()`). A da FÁBRICA era chamada pelo quadro; esta é chamada de **dentro de outra
fase**. ⇒ o prefixo afirma *«o quadro chama-me»*, e numa função-filha isso é **mentira** — ela
apareceria para sempre como órfã, ou alguém calaria o gate para a acomodar. *A regra escrita guardava
a metade que tinha sido paga, e a outra metade só aparece quando se paga.*

**How to apply:** antes de dar a uma função o nome que um oráculo colhe, pergunte **o que o oráculo
faz com ela**, não só *se ele a vê* — e para um prefixo, *quem tem de a chamar para a afirmação ser
verdadeira*. A cura aqui foi o nome SEM prefixo mais a razão escrita no doc da função, e os censos
que a medem passaram a ler o **FICHEIRO** em vez do texto emendado do quadro. ⭐ E a mesma pergunta
protege o caso oposto: um censo que varre por prefixo de nome passa a varrer ZERO quando alguém
renomeia, e fica **verde a medir nada** ([[feedback-a-gate-that-reads-the-shell-from-another-crate-escapes-the-line-that-moves-the-code]]).
