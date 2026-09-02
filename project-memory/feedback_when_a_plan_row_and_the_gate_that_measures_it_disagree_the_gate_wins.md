---
name: feedback-when-a-plan-row-and-the-gate-that-measures-it-disagree-the-gate-wins
description: "Linha de plano diz «faça»; o gate que mede o assunto traz a RECUSA no doc-comment — manda o gate, que foi escrito por quem mediu"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-09-01T21:22:52.566Z
---

O `spec/02` da linha UI/UX listava o degrau **B** como *«fundir os 16 apelidos de cor — mudança de
zero pixels, independente de tudo»*, na tabela **«o que pode começar HOJE»**. Re-medi a premissa
(0 divergências em 64 comparações, os quatro temas), construí a fusão inteira — 40 sítios
re-apontados, 16 variantes apagadas, 57 entradas do `tokens.json` — e só então o `cargo check`
partiu num gate cujo nome eu não tinha lido: `the_sixteen_timeline_slots_are_pure_aliases`.

O doc-comment dele dizia, escrito pela mesma linha:

> *«equivalência não é desejabilidade, e é por isso que a fusão não foi feita aqui … Fundir troca
> 16 nomes por 58 sítios de chamada directos … Essa é uma decisão de design system, do Enio — não
> uma dedução que este gate autorize.»*

Revertido.

**Why:** uma linha de tabela de plano é um **resumo**, escrito quando o assunto era uma intenção;
o gate é escrito por quem foi lá **medir**, e é onde a conclusão real fica. Quando a medição
descobre que a equivalência existe **mas não autoriza a mudança**, essa nuance não cabe numa
célula de tabela — e a célula fica a dizer «faça».

⚠️ E o modo de falha é caro de uma forma específica: eu **confirmei a premissa** (zero pixels) e
li isso como confirmação da **obra**. *Provar que uma mudança é inócua não é provar que ela é
desejada.*

**How to apply:** antes de pegar num item de plano, **procure o gate do assunto e leia o
doc-comment dele** — não só o nome.

- `grep -rl "<o assunto>" crates/*/tests/` custa segundos e responde *«alguém já mediu isto?»*.
- Um gate cujo nome descreve a PREMISSA (`..._are_pure_aliases`) e não a obra é o sinal mais forte
  de que a obra foi considerada e **não** feita.
- Se o plano e o gate discordarem, **corrija o plano** no mesmo trabalho: a linha vai continuar a
  mandar construir enquanto ninguém a reescrever.
- ⛔ E quando a decisão é declaradamente do dono, **pergunte** — é o caso raro em que parar com
  nada entregue é o certo, porque avançar em qualquer direcção é uma mudança de produto.

Irmãos: [[feedback_documented_decision_chesterton_fence]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_a_dead_control_and_an_absent_one_read_the_same_and_building_is_the_wrong_cure]] ·
[[feedback_archiving_without_indexing_the_refusals_deletes_them]]
