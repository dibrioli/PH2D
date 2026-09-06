---
name: a-replacement-surface-must-copy-what-the-old-one-resolves
description: Uma superfície nova que vai substituir outra tem de copiar o que a antiga RESOLVE (faixas, unidades), não só o que ela MOSTRA — senão o defeito já curado volta calado no dia em que a antiga sair.
metadata:
  type: feedback
---

Quando uma superfície NOVA vai substituir uma antiga (o cartão do nó a substituir o painel
lateral, 2026-09-05), o critério de aceitação óbvio — *«a nova oferece todos os controlos que a
antiga oferecia»* — é **fraco**, e um gate que só o mede fica verde sobre uma regressão.

**Medido:** o gate `no_param_the_panel_offers_falls_off_the_card` estava verde (713 params contra
700, e as 13 diferenças explicadas). E ainda assim, para o mesmo param, o cartão mostrava OUTRO
número que o painel em **109 de 454** rows escalares — porque o painel **resolve** três coisas
que o cartão copiava do hint:

| o que a antiga resolvia | o que a nova fazia |
|---|---|
| a faixa depende do **canal** que o nó conduz (graus numa Rotation, unidades de mundo num X/Y) | lia `hint.min`/`hint.max` |
| a faixa de um `value.*` é a de **quem ele alimenta**, e tem de CONTER o valor vivo | idem |
| a **FACE**: `mostrado = guardado × escala` (metros guardados, px mostrados) | mostrava o valor cru |

⛔ **A primeira delas já tinha sido curada na antiga por um report do dono** (*«Scale não aceita
mais que 4 em sua caixa de texto e 4 não é quase nada para rot»*). Copiar o hint teria reposto
esse defeito **no dia em que o painel saísse**, sem uma linha de código mudar nesse dia — o
report seria de outra pessoa, meses depois, sem ninguém ligar as duas coisas.

⇒ **Antes de construir a superfície de substituição, faça o CENSO do que a antiga resolve.** A
pergunta não é *«que controlos ela mostra?»* mas *«que perguntas ela responde antes de mostrar?»*
— e cada resposta é uma porta que a nova tem de CHAMAR, nunca reimplementar.

**Como se apanha:** um gate que compara **row a row, sobre todo o catálogo**, os NÚMEROS das duas
superfícies (`the_card_shows_and_drags_the_same_numbers_the_panel_does`) — faixa, limites
digitáveis, passo, valor e face. A mutação que apaga a face acusa as 109 pelo nome.

⚠️ E o sinal de alarme que eu quase perdi: **eu ia construir a caixa de escrever números sobre a
faixa errada**, ou seja, curar o report do dono e shipar no MESMO commit o defeito que a
superfície antiga já não tinha. *Uma feature nova sobre um substrato incompleto herda o buraco do
substrato.*

Relacionado: [[feedback_the_measured_refusal_you_need_is_in_the_neighbouring_knob]] ·
[[feedback_a_door_the_neighbour_does_not_call_is_not_a_door_yet]] ·
[[feedback_alive_reachable_and_in_the_wrong_place_are_three_questions]]
