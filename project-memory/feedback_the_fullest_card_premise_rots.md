---
name: feedback_the_fullest_card_premise_rots
description: "Sweep de UI ancorado num 'card mais cheio' apodrece — pergunte a CADA modo qual pinta o widget"
metadata:
  node_type: memory
  type: feedback
---

O sweep do seam do Sculpt varria `PAINTER_SCULPT_CLICKS` **armando o Chisel**, com o comentário *"o card
mais cheio"*. A W5b deu ao **Smooth** um botão (Filter Layer) que o Chisel **não pode ter** (verbo de plano:
o alvo é ajustado à pegada do dab) — e o sweep quebrou. Não havia mais um card que mostrasse tudo.

**Why:** um card cujo conteúdo é **função do modo** não tem superset garantido — e "o modo X mostra tudo" é
uma premissa que envelhece silenciosamente a cada feature. Ela não estava errada quando foi escrita; ficou
errada. O consumidor do gate não recebe erro de compilação: o teste fica **vermelho por um motivo que não é
o bug** (o widget novo está ótimo; a premissa é que morreu). É primo de
[[feedback_a_condition_that_enumerates_its_readers_rots]]: uma âncora que enumera *quem mostra o quê*
apodrece quando chega o próximo.

**How to apply:** varra **cada modo** e exija que **ALGUM** pinte o widget (`for verb in 0..N { … continue }`
+ um `reached` no fim). Isso afirma exatamente a propriedade que importa — *um widget que nenhum card pinta
é morto; um que algum card pinta tem de ser clicável LÁ* — e **não precisa de tabela id→modo escrita à mão**,
que seria uma 2ª cópia das regras do painel e driftaria delas ([[feedback_two_doors_to_the_same_question_diverge]]).
Corolário: quando um gate quebra ao adicionar feature, pergunte **se o gate perdeu a premissa** antes de
podar a lista — podar teria escondido o widget novo do sweep, que é exatamente o que ele existe pra pegar.
