---
name: feedback_close_and_reopen_cures_it_is_the_signature_of_a_feedback_loop
description: «Fecho e reabro e volta ao normal» não é um transitório de arranque — é a assinatura de um CICLO com dois pontos fixos, em que a saída errada de um quadro é a entrada que o mantém errado; procure a grandeza que o quadro N+1 deriva do que o quadro N publicou
metadata:
  type: feedback
---
Report do dono, 2026-09-11: *«Inspector e Hierarchy saíram da lateral e foram para uma posição
estranha na altura da timeline do Flip. Quando fecho o Flip e reabro, tudo volta para posição
normal.»*

⭐⭐⭐ **A frase «fecho e reabro e volta ao normal» é o achado, não a atenuação.** Ela diz que o
estado correcto é alcançável com os MESMOS dados — logo o defeito não é uma conta errada, é um
**ciclo**: a saída errada do quadro *N* é exactamente a entrada que mantém o quadro *N+1* errado.
Um toggle cura porque muda a população e **parte o ciclo**, não porque «recalcula».

⇒ **Procure a grandeza que um quadro DERIVA do que o quadro anterior PUBLICOU.** No caso: o
`DockSides::from_published` lê os rects publicados para decidir se uma coluna está ocupada; uma
faixa de abas empurrava os rects das colunas para fora delas; e no quadro seguinte a cobertura lia
~20 % contra uma barra de 50 % ⇒ *«coluna vazia»* ⇒ a faixa alargava-se outra vez. **Dois pontos
fixos, e o primeiro quadro decide em qual se cai.**

⚠️ **Como se distingue de um transitório de arranque:** um transitório converge sozinho no quadro
seguinte e o dono nunca o vê; um ciclo **persiste** e responde a um gesto que muda a população.
*Se o report contém um gesto que cura, não procure a conta — procure a realimentação.*

⚠️ **E a lei de desenho que o causou:** *tocar uma faixa não é MORAR no encaixe dela.* A regra era
«todo rect docado que TOCA a faixa começa onde ela acaba» — derivada de propósito contra uma lista
de nomes que caduca, e a derivação estava certa na intenção e **larga demais** no predicado. A cura
manteve-a derivada e trocou a propriedade: *um rect pertence à banda cuja coluna horizontal ele É*
(todo rect docado nasce com o `x`/`w` do encaixe dele; as colunas só se partem na VERTICAL).
⛔ **Contenção não bastaria** — `0..220` cabe dentro de `6..1914`, e só não cabe por causa dos 6 px
de reserva da alça: uma cura presa a esse número parte-se no dia em que ele for a zero.

**Why:** o ciclo tornava o defeito **estável**, e um defeito estável lê-se como uma decisão de
desenho. Procurar a conta errada naquele layout teria dado dias sem achado — a conta estava certa
em todos os quadros; o que estava errado era o que ela recebia.

**How to apply:** ao receber um report com «fecho e reabro e fica bom», pergunte primeiro *que
número deste quadro veio do quadro anterior?* e escreva o teste sobre UM quadro com a entrada que
o ciclo produz — foi o que reproduziu este ao número (`y=862`, o mesmo da foto).
[[reference_topic_ui_seam_discipline]] [[reference_topic_measurement_discipline]]
