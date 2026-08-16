---
name: feedback-a-rule-only-exists-if-it-is-on-the-path-of-who-executes-it
description: Escrever a regra no doc certo para o agente ERRADO é o mesmo que não a escrever
metadata:
  type: feedback
---

Uma regra operacional só existe se estiver **no caminho de quem a executa, no
momento em que ela vale**. Um documento correto que ninguém abre naquele
instante é indistinguível de regra nenhuma.

**Caso medido (2026-08-16):** depois de medir que 54% do `target/` é
`incremental/`, escrevi as regras de higiene de disco na
`DIRETIVA_FIM_DE_DIA.md` — o doc de disco, aparentemente o lugar certo. Medido
em seguida: aquele arquivo era citado por **ZERO** documentos (só por duas
memórias), não estava no roteador da §1 do `CLAUDE.md` nem nos modelos de
abertura/troca de linha. **Um implementador que fecha uma linha nunca o
abriria.** A regra estava no lugar certo **para o agente errado**.

A cura não foi escrever mais, foi **redistribuir por momento**: a regra do gate
foi para o bullet do gate batched (`CLAUDE.md §2`, que todo agente lê
automaticamente); a de reclamar o `incremental/` foi para o passo em que a linha
PARA (`DIRETRIZ §1.5.9`); e o doc órfão ganhou a linha de roteador.

⚠️ **E o mesmo diagnóstico achou uma nota FALSA no caminho quente:** a
`DIRETRIZ §6` afirmava que o `[profile.dev] debug = true` *"só afeta `cargo
check` (irrelevante — não linka) e builds ad-hoc (que evitamos)"*. Medido: o
`cargo test -p` — o gate de fechamento que a própria `CLAUDE.md §2` prescreve —
**linka** binários de teste sob o `dev`, e eles somavam **8,3 GB em 40
binários**. *Uma nota que declara "irrelevante" um número que outra pessoa pode
mover é o §0 mordendo em casa.*

**Why:** docs crescem por assunto (*"disco"*, *"velocidade"*) e agentes leem por
momento (*"estou abrindo"*, *"estou fechando"*, *"estou no gate"*). As duas
taxonomias divergem, e a regra cai no vão.

**How to apply:** antes de dar por escrita qualquer regra para agentes,
pergunte **quem a executa e o que essa pessoa tem aberto nesse instante** — e
meça a alcançabilidade (`grep -rl <doc>` para ver quem o cita). Se o doc é
órfão do roteador, a regra ainda não existe. E antes de acrescentar a mesma
regra a um segundo doc, meça se ele já **aponta** para o canônico: um ponteiro
não diverge, uma segunda cópia diverge.

Irmão de [[feedback_a_condition_that_enumerates_its_readers_rots]] e de
[[feedback_stale_comment_and_dead_code_lie]].
