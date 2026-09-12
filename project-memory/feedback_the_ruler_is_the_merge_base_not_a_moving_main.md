---
name: the-ruler-is-the-merge-base-not-a-moving-main
description: "`git diff main` mistura o trabalho da linha com a deriva do main — a régua de uma linha é sempre o merge-base"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: cbcca673-89ac-4ca6-be35-ef2308fe97cb
  modified: 2026-09-12T03:12:34.860Z
---

Num Modo L, o `main` anda enquanto a linha trabalha. `git diff main -- <path>` compara a árvore
de trabalho com o `main` **de agora**, então tudo o que o `main` ganhou depois do fork aparece
como **remoção tua**.

Medido 2026-09-11 (W2/L3-B): o `git diff main -- docs/` acusou-me de apagar 15 linhas do
`HOWTO_partir_uma_familia_da_shell.md` — um parágrafo que a `line/app-motion` tinha escrito. O
`main` estava **um commit** à frente do meu merge-base e foi ele que as escreveu; nenhum commit
meu tocara `docs/`.

**Why:** a acusação lê-se como dano próprio e leva a "restaurar" código alheio que não foi tocado
— ou, pior, a não notar a deriva ao contar ficheiros tocados.

**How to apply:** `BASE=$(git merge-base main HEAD)` e depois `git diff $BASE`. Vale para o censo
de *«que ficheiros fora do meu alvo é que eu toquei?»* e para qualquer contagem de linhas. ⚠️ É a
mesma família do aviso do `collision-surface.sh` no CLAUDE.md §1 (a coluna `base:` dele é o
merge-base e envelhece a partir da 2ª fusão de uma rodada) — a diferença é que ali o erro é ler
um número velho, e aqui é **atribuir a si o trabalho de outra linha**.
Relacionado: [[a-rename-by-name-cannot-tell-an-address-from-a-memory]].
