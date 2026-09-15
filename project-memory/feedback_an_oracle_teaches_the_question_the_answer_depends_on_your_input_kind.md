---
name: feedback-an-oracle-teaches-the-question-the-answer-depends-on-your-input-kind
description: Portar a lei de um oráculo à letra falha quando a NATUREZA da entrada difere — um NÍVEL que fica verdadeiro e um EVENTO que acontece precisam de regras de paragem opostas.
metadata:
  type: feedback
---

A máquina de estados do Godot (MIT, corrida sem interface) atravessa uma cadeia inteira num avanço e
**não pendura** num ciclo: ela pára ao voltar a um estado já visitado nesse avanço. Portada à letra
para o `StateMachine` do PH2D, uma porta `Aberta` ia a `Fechada` **e logo a `A abrir`** com UM
toque — achado por um gate vermelho sobre o meu próprio desenho.

**Why:** a entrada dele é um **NÍVEL** (`advance_condition` é um booleano que FICA verdadeiro, logo
re-satisfaz a transição seguinte, e por isso ele precisa de uma regra de paragem própria). A nossa é
um **EVENTO** — um sinal acontece e é **gasto** por quem o ouve. Com o consumo, a lei observável do
oráculo fica intacta (cadeia num tique, ciclo que nunca pendura) e nenhum `MAX_DEPTH` é preciso: o
laço termina porque a lista de eventos é finita.

**How to apply:** ao colher uma lei de um oráculo, pergunte *de que NATUREZA é a entrada dele?* —
nível, evento, amostra contínua. O oráculo ensina a **pergunta** e o formato da resposta; a resposta
é de quem conhece a própria entrada. E escreva a divergência como **declarada**, com o mecanismo,
ao lado do gate que a defende. Ver [[reference_topic_oracle_discipline]].
