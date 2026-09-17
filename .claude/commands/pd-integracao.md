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
   ⭐ **O degrau do `PROJECT_SCHEMA` reconta-se por SCRIPT, não por juízo:**
   `python3 scripts/schema-recount.py` (lê os estágios `:1`/`:2`/`:3` do índice, preserva a escada
   do main, renumera o degrau da linha para `main+1` e sobe a tripla do ficheiro irmão). ⚠️ Cada
   passo tem `assert` — a 1.ª redacção supôs que a âncora era igual dos dois lados (verdade na
   escada, **falso na tripla**, que CONTÉM o número) e parou alto em vez de escrever lixo.
4b. ⛔ **Tecto de LOC vermelho: dimensione o corte pela MEDIÇÃO, não pela generosidade.** A catraca
   imprime o excesso. Em 17/09 ela pedia **711** linhas e o corte levou **4 470** — seis vezes mais,
   e o excesso custou **uma corrida inteira de portão a mais** (arrastou um gate que lia um ficheiro
   por caminho em runtime e quatro ficheiros de teste que o `git mv` deixou para trás).
   ⚠️ **`cargo check --all-targets` é CEGO aos testes de uma DEPENDÊNCIA** — quem apanha um ficheiro
   de teste deixado para trás é `cargo nextest list`, nunca o `check`.
4c. ⚠️ **Antes de inventar uma cura de ARQUIVO, `grep` o padrão.** A cura do tecto da escada do
   schema (arquivar uma faixa de degraus) **já existia com três irmãos**, e a 1.ª tentativa de 17/09
   escreveu por cima de um deles — **490 linhas de história verbatim**. *Uma cura que parece óbvia ao
   integrador costuma já ter sido paga.* Onde as horas foram, medido:
   [ANATOMIA](../../docs/archive/integracao-jornadas/ANATOMIA_DE_UMA_RODADA_2026-09-17.md).
5. Gate da árvore combinada COMPLETO — inclusive os arch-gates de shell, que só correm
   na varredura impactada e já chegaram vermelhos ao tip de uma linha.
6. §5 do CLAUDE.md: **uma linha por linha integrada**, nunca a narrativa (§1.5.9 item 8).
   Os registos da rodada (blocos de abertura, briefings, handoffs de integração, estado)
   nascem em `docs/archive/integracao-jornadas/` — a `docs/IntegracaoMultiAgente/` é só
   processo, e o gate `the_live_process_folder_holds_no_dated_record` reprova um registo lá.
   Ship/push só por ordem EXPLÍCITA do Enio (CLAUDE.md §0.7).
   Orce 2-4 iterações: o ship do integrador drena latentes.
