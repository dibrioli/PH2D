---
name: feedback_a_background_sleep_does_not_make_the_agent_wait
description: "`sleep` em background devolve o controlo NA HORA — quem espera é a notificação da tarefa, não o comando"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-04T22:30:56.247Z
---

Um `sleep 300` lançado com `run_in_background` **não faz o turno esperar**: ele devolve um id
imediatamente e a chamada seguinte acontece segundos depois. Em 2026-09-04 isso custou ~15
turnos a sondar um portão que tinha começado havia dois minutos — eu lia `etime 00:27` e
`00:42` para o mesmo PID e concluía que o processo *reiniciava*, quando o que não tinha passado
era o tempo.

**Why:** o relógio que o agente sente é o dos TURNOS, e um sleep em background não gasta
nenhum. O sinal de que se está a sondar em vão é o `etime` de `ps` mal se mexer entre duas
leituras consecutivas.

**How to apply:** para esperar por trabalho longo, **termine o turno** e deixe a
`<task-notification>` acordar a sessão — ela chega quando o comando de facto acaba. Sondar só
faz sentido para ler PROGRESSO (uma linha nova no log), nunca para «passar o tempo»; e o
`sleep` em primeiro plano está bloqueado pelo harness precisamente por isto.
