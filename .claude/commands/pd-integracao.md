---
description: Só por ordem explícita. Ordem de integração se MEDE.
argument-hint: [Linhas]
---
Integre a(s) linha(s): $1

Você é o agente integrador. Munição: o handoff de cada linha.

0. **`bash scripts/collision-surface.sh` em CADA worktree, ANTES do primeiro grep.**
   Ele responde de uma vez a lista que a integração redescobre ~1.000 vezes
   (schemas, registro de componentes, contrato congelado, ADR, Cargo.lock,
   marcadores, tetos de LOC). ⚠️ O que está colado no handoff é **referência,
   não evidência**: se a linha fechou antes de outras integrarem, a tabela dela
   já morreu — quem re-roda é você.
1. Meça a sobreposição par-a-par e me diga a ORDEM de integração antes de mergear
   (a ordem se mede, não se escolhe).
2. `--ff-only` + `scripts/foundational-integrate.sh` (gate da árvore combinada).
   Mergiraf funde o resíduo textual.
3. Resolva conflito pelos ESTÁGIOS do índice (`:1` base, `:2` ours, `:3` theirs),
   nunca pelos marcadores. Varra marcadores em CADA commit.
4. Números que SOMAM entre linhas se CONTAM (PROJECT_SCHEMA, registro de componentes,
   número de ADR): o valor certo pode não estar em nenhum dos dois lados do conflito.
5. Gate da árvore combinada COMPLETO — inclusive os arch-gates de shell, que só correm
   na varredura impactada e já chegaram vermelhos ao tip de uma linha.
6. §5 do CLAUDE.md: **uma linha por linha integrada**, nunca a narrativa (§1.5.9 item 8).
   Orce 2-4 iterações: o ship do integrador drena latentes.
