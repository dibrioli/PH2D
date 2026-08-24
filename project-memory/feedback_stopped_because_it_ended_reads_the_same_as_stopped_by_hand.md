---
name: feedback-stopped-because-it-ended-reads-the-same-as-stopped-by-hand
description: "«Acabou» e «foi pausado» colapsam no mesmo bool, e o gesto de religar precisa da distinção — senão ele é MORTO, não lento"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d971358c-b4ab-4ed0-ab84-65cd6d892c68
  modified: 2026-08-23T15:46:00.574Z
---

Num transporte (animação, playback, simulação), **«parou porque chegou ao fim» e «parou porque
alguém pausou» leem-se iguais num `playing: bool`** — e o gesto de religar precisa de os distinguir.

**Why:** medido na §11 Animation (2026-08-23). Numa tag com `repeat: Some(1)` já gasta, pôr
`playing = true` deixa a imagem na ponta do intervalo **com o contador de ciclos cheio**: o primeiro
passo do avanço vê `at_end`, `will_continue == false`, e fecha o ciclo outra vez — no MESMO tique. A
caixa ficava marcada por um quadro e desmarcava-se sozinha. ⚠️ **Não é «lento», é MORTO**, e um gate
que só verifica o estado logo após o commit fica verde: é preciso **correr o relógio depois de
ligar**.

O irmão do mesmo dia: **rebobinar tem de mover a IMAGEM**, não só os contadores. Repor
`elapsed`/`repeat_count`/o flag de ping-pong não se vê — e, com um `repeat` finito, um rebobinar que
não reposiciona o frame é um botão que não faz nada **duas** vezes (o passo seguinte re-fecha o
ciclo a partir da ponta).

**How to apply:**
- Dê um predicado ao estado terminal (`is_finished(tag)` = `ciclos_permitidos.is_some_and(|n| feitos >= n)`);
  uma lei infinita **nunca** acaba, por mais que corra.
- *Tocar uma reprodução que se esgotou rebobina-a; retomar uma pausa continua de onde estava.* Uma
  lei, e todas as portas que «tocam» passam por ela (o interruptor **e** escolher outro item na
  lista).
- O gate afirma as **duas** metades, senão a cura vira «rebobina sempre» e apaga a pausa.

Família: [[reference_topic_authored_state_and_clocks]] ·
irmã de [[feedback_paint_and_dispatch_must_read_the_same_source]] (o mesmo report do Enio destapou as duas).
