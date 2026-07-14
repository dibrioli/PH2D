---
name: feedback-a-shared-list-is-merged-against-todays-main
description: Uma branch NÃO pode consertar uma lista compartilhada (MEMORY.md, allowlist, registry) a partir da própria base — a "limpeza" apaga o que a main ganhou depois do fork
metadata:
  type: feedback
---

Na linha `line/audio` eu vi 4 linhas de índice no `project-memory/MEMORY.md` que **não eram
minhas**, concluí que tinham vazado pelo symlink de memória (que é compartilhado entre as
worktrees da máquina) e as **removi** — com uma mensagem de commit segura de si.

**Elas eram da `main`.** Minha branch tinha forkado ANTES de elas pousarem. No rebase a deleção
aplicou **limpa** (a main de fato as tinha) e as 4 sumiram de verdade: os arquivos ficaram, sem
linha de índice, e **memória não-indexada nunca é recuperada** — viraram memória morta. Quem achou
foi o integrador, olhando; **nenhum gate pega isso** (o texto funde sem conflito).

A ironia, registrada: **fui eu quem escreveu, no meu próprio handoff, que "lista que SOMA resolve
por UNIÃO, nunca escolha um lado" — e caí exatamente nisso.**

**Why:** a regra que eu escrevi estava pela metade. Não basta *"resolva por união"*: falta **contra
o quê**. Uma lista compartilhada (`MEMORY.md`, allowlist de gate, registry, tabela `KINDS`) não tem
dono, e a sua base é uma **foto velha** dela. Ausência na sua base tem DUAS causas indistinguíveis
daqui: (a) alguém adicionou por engano depois, ou (b) **a main ganhou aquilo enquanto você
trabalhava**. Você não consegue diferenciar sem olhar a main de HOJE — e no caso (b) "limpar" é
**apagar trabalho alheio, em silêncio, com o merge verde**.

**How to apply:** antes de REMOVER qualquer linha de uma lista compartilhada:

1. `git fetch` / `git log <base>..main -- <o arquivo>` — **a main andou?** Se andou, a sua base
   mente.
2. A linha que você quer tirar **está na main de hoje**? Então ela **não é lixo**: é conteúdo que
   você não tinha. Não remova.
3. Só **ADICIONE** o que é seu. Deixe o merge fazer a união. Remoção numa lista compartilhada é
   uma operação de INTEGRAÇÃO (com a main na mão), não de linha.
4. Antes de fechar: todo item que a sua branch ADICIONA está indexado/registrado? (o meu bug
   gêmeo: criei uma memória e **esqueci a linha de índice dela** — o arquivo sozinho é inerte).

Corolário para o `MEMORY.md` especificamente: o symlink `~/.claude/.../memory` → `project-memory/`
aponta para a árvore **primária**, não para a sua worktree. Escreva memória **direto na worktree,
por caminho absoluto**; copiar de lá para cá importa o WIP das outras linhas
([[feedback_sed_relative_path_hits_primary_cwd]]).

Parente de [[feedback_numbers_that_sum_across_lines_count_dont_pick]] (o valor certo não existe em
nenhum lado do conflito — lá você CONTA, aqui você UNE) e de
[[feedback_resolve_conflicts_from_index_stages_not_markers]].
