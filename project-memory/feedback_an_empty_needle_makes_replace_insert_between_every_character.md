---
name: feedback_an_empty_needle_makes_replace_insert_between_every_character
description: "Uma fatia calculada por índices que sai VAZIA transforma `str.replace` num vandalismo silencioso — 581 linhas viraram 646 277"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-05T01:21:33.133Z
---

Ao cortar um bloco com `a = s[s.index(X):s.index(Y)]` e depois `s.replace(a, n)`: se `Y`
aparecer **antes** de `X`, a fatia sai **vazia** — e `str.replace("", n)` insere `n` **entre
cada carácter do ficheiro**. Medido em 2026-09-04: `sculpt3d_keys.rs` foi de `581` para
`646 277` linhas, e o compilador cuspiu `312` erros de *«unknown start of token»*.

**Why:** a disciplina da casa (`CLAUDE.md` §2) manda pôr `assert` de contagem em toda edição
por script — mas um `assert s.count(a) == 1` sobre uma agulha **vazia** não protege nada:
`"abc".count("")` é `4`, e o assert nem chega a disparar. *A guarda tem de ser sobre a AGULHA,
não sobre a contagem dela.*

**How to apply:** ao construir a agulha por índices, `assert a` (não-vazia) **antes** de
qualquer coisa; e prefira sempre a agulha **literal** escrita à mão, que é a forma que falha
alto. Se o estrago acontecer, o ficheiro ainda estava por commitar: `git checkout -- <path>`
devolve-o ao último commit — e é por isso que commits locais frequentes são a rede
([[feedback_python_replace_silent_noop_after_fmt]]).
