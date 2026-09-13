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
   já morreu — quem re-roda é você. ⛔ E a coluna `base:` é o MERGE-BASE da linha:
   ela NÃO anda com o `main`. O valor do `main` lê-se no ficheiro
   (`git show main:<arq>`), e o degrau da linha conta-se como DELTA sobre ele.
   Uma linha `✗ SONDA CEGA` (saída 2) = a const mudou de ficheiro: corrija o script antes.
1. Meça a sobreposição par-a-par e me diga a ORDEM de integração antes de mergear
   (a ordem se mede, não se escolhe).
2. `--ff-only` + `scripts/foundational-integrate.sh` (gate da árvore combinada).
   Mergiraf funde o resíduo textual.
3. Resolva conflito pelos ESTÁGIOS do índice (`:1` base, `:2` ours, `:3` theirs),
   nunca pelos marcadores. Varra marcadores em CADA commit.
3b. **Rebase sem conflito NÃO é prova.** Listas numeradas/catracas: `merge=text` em
   `$(git rev-parse --git-common-dir)/info/attributes` enquanto integra (tire no fim), com
   assert de contagem; e compare CADA commit rebaseado com o original (multiconjunto `+/-`
   por ficheiro, num ficheiro `bash` com controlo positivo). O Mergiraf largou 2 remoções em
   130 commits e disse «Solved» (13/09). Grave a saída inteira do rebase; nunca a filtre só
   por `CONFLICT`.
3c. **Rodada de várias linhas:** rebase em cadeia num `integ/<rodada>`, gate da árvore
   combinada no tip, `--ff-only` do `main` no fim. Um `dead_code` que só nasce na fusão cura-se
   NO commit rebaseado em que o último uso morre (edit-rebase) — o `bisect` não pode atravessar
   árvore vermelha. Duas linhas que fizeram a MESMA coisa de formas diferentes: fica um nome e
   um corpo. Mover corpos prova-se com `scripts/moved-proof.py`.
3d. Vermelho sem uma linha de Rust mudada: reproduza sob contenção (N cópias `--exact`, com o
   `loadavg` ao lado) ANTES de culpar o commit (CLAUDE.md §5.0, família de flakes).
4. Números que SOMAM entre linhas se CONTAM (PROJECT_SCHEMA, registro de componentes,
   número de ADR): o valor certo pode não estar em nenhum dos dois lados do conflito.
5. Gate da árvore combinada COMPLETO — inclusive os arch-gates de shell, que só correm
   na varredura impactada e já chegaram vermelhos ao tip de uma linha.
6. §5 do CLAUDE.md: **uma linha por linha integrada**, nunca a narrativa (§1.5.9 item 8).
   Os registos da rodada (blocos de abertura, briefings, handoffs de integração, estado)
   nascem em `docs/archive/integracao-jornadas/` — a `docs/IntegracaoMultiAgente/` é só
   processo, e o gate `the_live_process_folder_holds_no_dated_record` reprova um registo lá.
   Ship/push só por ordem EXPLÍCITA do Enio (CLAUDE.md §0.7).
   Orce 2-4 iterações: o ship do integrador drena latentes.
