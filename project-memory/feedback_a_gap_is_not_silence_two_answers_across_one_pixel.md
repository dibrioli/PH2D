---
name: feedback_a_gap_is_not_silence_two_answers_across_one_pixel
description: "Nada aqui" e "algo aqui com influencia zero" tem de dar a MESMA pose; discordar atraves de um pixel de regua e um salto
metadata:
  type: feedback
---

Um fade-in de strip saltava 3 unidades num frame. A causa não era o fade: **duas respostas
discordavam através de um pixel de régua.** Onde nenhum strip cobria, a lane era *silenciosa*
(ninguém escrevia, o objeto segurava a pose). No primeiro instante do fade-in — strip cobrindo
com peso **zero** — ela respondia *repouso*. O fade rampeava do repouso; o objeto não estava lá.

**Why:** eu tinha corrigido o peso-zero antes (silenciá-lo fazia a pose depender do lado de onde
o playhead chegava) e o gate daquela correção pinava "peso 0 = repouso" — path-independente,
determinístico, **e um salto na tela**. Duas propriedades certas (função pura do playhead) não
somam a uma terceira que ninguém escreveu: **continuidade**. O erro real estava do outro lado:
a lacuna nunca foi silêncio — o strip que acabou segue afirmando o último frame dele.

**How to apply:** quando um valor tem um caso "presente mas neutro" e um caso "ausente", exija
que os dois **coincidam no limite** — é uma fronteira, e uma fronteira onde a resposta pula é um
salto que o usuário vê. Ao consertar, pergunte qual dos dois lados está errado (aqui era o
"ausente", não o "neutro"). E gate a **aparência**: `passo por frame < 0.1` pega o teleporte que
qualquer asserção sobre a regra deixaria passar. Ver [[reference_topic_oracle_discipline]],
[[feedback_a_threshold_must_live_where_the_domain_is_empty]].
