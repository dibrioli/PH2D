---
name: feedback_a_correct_gate_that_blocks_a_new_feature_is_not_the_thing_to_weaken
description: Um gate correto que barra uma forma nova — a forma e que sai, e a recusa fica com a tabela
metadata:
  type: feedback
---

Quando um gate **certo** reprova uma feature nova, a pergunta não é como o afrouxar: é se a
feature paga o que o afrouxamento custa a **tudo o resto**.

Caso medido (`line/3DModeling`, W125, 2026-09-06). A **escada** por fórmula foi construída
**quatro** vezes; a última passava marcha, chanfro e arestas. Ela morreu no
`the_biggest_fillet_still_leaves_a_body`, que exige com `<` **estrito** que o maior filete
que o documento aceita **encolha** a peça. A escada mede `20 139` amostras dentro **com e sem**
filete — exactamente o mesmo número — porque tem **tantas quinas côncavas como convexas, e do
mesmo raio**: o que o filete come numa aresta devolve na vizinha.

⚠️ **O gate não distingue *equilibrado* de *inerte*, e não devia ter de o fazer.** Foi esse `<`
estrito que apanhou, em Agosto, o `round` **inerte** do cone e do prisma (`+0,0 %` de volume,
campo bit a bit igual). Trocar `<` por `<=`, ou isentar a forma, seria trocar um gate que
apanhou **dois defeitos reais** por **uma** forma.

**Why:** um gate é um instrumento partilhado por todas as features futuras; a feature que ele
barra é uma. O afrouxamento paga-se em todo o corpus, para sempre, e a conta nunca aparece no
diff em que foi feito.

**How to apply:** ao ver um gate correto a reprovar obra nova — (1) meça **o que ele apanhou**
antes (o histórico do gate está no doc-comment dele, e é a metade da conta que falta); (2) se
ele já apanhou defeito real, **a obra é que sai**, com a tabela da recusa escrita ao lado dela
(doc do módulo + tabela de *Recusas MEDIDAS*); (3) só considere mudar o gate se a medição
mostrar que ele **descreve mal** o que quer proibir — e aí a mudança é uma wave própria com
prova de mutação, não uma linha no commit da feature.

⚠️ E quando a feature recusada já está **ligada em N ficheiros**, recupere por **reversão**
(`git checkout -- <dirs>`, docs preservados) e **reaplique só o que fica**: a remoção cirúrgica
em 29 ficheiros é a operação em que uma linha esquecida **compila**.

Relacionado: [[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_perfection_no_deferrals]] · [[reference_topic_gate_discipline]] ·
[[reference_topic_implicit_field_laws]]
