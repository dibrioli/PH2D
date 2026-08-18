---
name: feedback_archiving_without_indexing_the_refusals_deletes_them
description: Cortar um doc gigante sem indexar as recusas MEDIDAS equivale a apagá-las — e a cura de um doc inchado pode REALOCAR a doença em vez de curá-la.
metadata:
  node_type: memory
  type: feedback
---

Duas leis que o corte de 2026-08-18 pagou para aprender.

## 1. Arquivar sem indexar as recusas é APAGÁ-LAS

Um doc gigante guarda dois tipos de coisa: narrativa (dispensável) e **recusas medidas** —
*"tentei X, medi Y, REJEITADO, e o mecanismo é Z"*. A segunda é o conteúdo mais caro do repo: é a
única que impede alguém de refazer trabalho já pago.

Medido: `docs/Painter/28_otimizacoes_o_que_funcionou.md` guardava **47** recusas; o `CLAUDE.md §5`
citava **cinco**. Mover as 47 para `docs/archive/` sem mais nada teria sido, na prática, apagá-las —
ninguém procura no arquivo o que não sabe que existe.

**How to apply:** todo doc cortado leva no fim uma tabela **`⛔ Recusas MEDIDAS`**, *derivada* do
arquivo (uma linha por recusa, com link para a linha exata). ⚠️ **Indexe também os CABEÇALHOS** que
carregam a marca — as recusas mais duras («MEDIDO E REJEITADO — não refaça») são títulos de seção, e
um extrator que só olha o corpo perde **31 de 126**. Verifique no fim: *zero cabeçalhos de recusa
inalcançáveis a partir do doc vivo.*

## 2. A cura de um doc inchado pode REALOCAR a doença

A regra *"a narrativa da jornada vai para o handoff; o `CLAUDE.md §5` recebe UMA linha"* curou o
`CLAUDE.md` (917 KB → 44 KB) e **criou** o `HANDOFF_line_physics.md` a **710 KB** — 77% do que o §5
chegou a ser. A regra redirecionou o append; não o limitou. E o §5 mandava ler o destino.

⚠️ **O teto se MEDE, não se escolhe.** O joelho medido neste repo está entre **80 e 110 KB**: a
`DIRETRIZ.md` (86 KB) é o doc mais lido e ainda cabe num `Read`; acima disso o `Read` desaparece e o
acesso vira raspagem por shell — o tracker de física teve **1 `Read` para 407 comandos**, e **89%
dele nunca entrou em contexto nenhum**. Uma regra na linha 8.000 não é "difícil de achar": ela não é
lida (667 marcadores lá dentro, 558 além da linha 2.000).

**How to apply:** ao escrever uma regra anti-inchaço, pergunte **para onde o append vai** — e ponha
teto lá também. Corte por `python3 scripts/doc-split.py` (aborta se as duas metades não remontarem o
original por sha256), nunca à mão. O doc vivo vira **roteador**; a história vai **verbatim**.

⚠️ E o que fica vivo tem de ser **endereçável**: medido, o agente não lê estes docs — ele reconstrói
o sumário (`grep '^## '`), salta para um endereço (`HR-5`, `Bug #17`, `§1.5.9`) e lê ~70 linhas. É
por isso que o `SKILL_Stack` (consultado por `HR-N`) funciona e um diário numerado fora de ordem não.

Irmãs: [[feedback_stale_comment_and_dead_code_lie]] · [[feedback_numbers_that_sum_across_lines_count_dont_pick]] ·
[[feedback_documented_decision_chesterton_fence]].
