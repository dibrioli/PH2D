---
name: feedback-resolve-conflicts-from-index-stages-not-markers
description: "Resolva conflito pelos estágios do índice (:1 base, :2 ours, :3 theirs) — os marcadores mentem (o Mergiraf emite 2 vias, sem base) e o Edit deixa `<<<<<<<` órfão"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6316633f-521c-4b1d-a255-7662e2fda363
---

Dois tropeços na integração das 6 linhas (2026-07-12), ambos por tratar conflito como **texto**:

**1. Marcador órfão commitado (meu erro, 11 commits).** Resolvi um hunk com o Edit tool e o
`old_string` **não incluía a linha `<<<<<<< HEAD`** — só o miolo. O `=======`/`>>>>>>>` saíram, o
`<<<<<<<` **ficou**. Pior: meu próprio script imprimiu `marcadores: 4` e eu **segui assim mesmo**.
Salvou-me a varredura por-commit ([[feedback_sweep_conflict_markers_every_commit]]) e o fato de o
Modo L só fundir **depois** do gate — o `main` nunca foi contaminado.

**2. Sem base no arquivo.** O `.gitattributes` do repo registra o **Mergiraf**, que emite conflito
de **2 vias** (sem a seção `|||||||`). Um resolvedor que espera diff3 fica cego e, se "decidir"
mesmo assim, escolhe errado.

**Why:** o arquivo em conflito é uma *renderização*; a verdade (base, ours, theirs) está no
**índice**. E `ours` num rebase é o main-em-crescimento, não "a minha versão" — a intuição inverte.

**How to apply:**
- Leia os três lados de onde eles existem: `git show :1:<path>` (base) · `:2:` (ours) · `:3:`
  (theirs). Semântica de rebase = **aplicar o delta de theirs sobre ours**.
- **Portão duro antes de `git add`:** `grep -qE '^(<<<<<<< |>>>>>>> |\|\|\|\|\|\|\||={7}$)' <file>`
  → recusa. Um `git add` de arquivo com marcador é um commit que não compila.
- Nunca resolva um hunk pegando `ours` **cego**: nesta jornada isso **engoliu um teste** do Flip
  (o gate do par `PROJECT_SCHEMA` × `FLIP_SCHEMA_VERSION`). Confira o que o lado descartado trazia.
- Cargo.lock / gerados: `git checkout main -- Cargo.lock` + regenerar. Nunca à mão.
