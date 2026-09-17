---
name: feedback-mergiraf-silently-drops-a-deletion-in-a-list-and-says-solved
description: O Mergiraf resolve sozinho um conflito entre duas REMOÇÕES vizinhas num array Rust e pode deixar a remoção de um lado de fora — o rebase diz «Successfully», o índice fica limpo, e só uma comparação de patch commit a commit o vê.
metadata:
  type: feedback
---

Medido na integração de 2026-09-13 (`line/input-dispatch` + `line/render-bodies` + `line/loc-caps`):
as três linhas apagavam entradas vizinhas do `FN_OVERAGE_OK` (`shells/desktop/tests/it/fn_loc_caps.rs`).
No rebase da terceira, o git levantou 4 conflictos textuais (resolvidos pelos estágios) e o **Mergiraf
resolveu outros sozinho** — em dois deles (`172e47e8e` `build_initial_state`, `c52895917` `lay_out`)
o resultado **manteve a entrada que a linha apagava**. Rebase «Successfully», zero marcadores, índice
limpo; o `fn_loc_caps` só reprovaria mais tarde, no censo de obsolescência.

**Why:** o driver só corre nas regiões em conflito e escolhe uma forma sintacticamente válida do array;
«válido» não é «a união das duas mudanças». A linha `INFO Mergiraf: Solved N conflict` sai no stderr
do rebase e some num `grep` que filtra por `CONFLICT|error` — foi o que aconteceu.

**How to apply:**
- Depois de TODO rebase de integração, compare **cada commit rebaseado com o original**: por ficheiro, o
  multiconjunto das linhas `+`/`-` (contexto e números de linha mudam; o que o commit acrescenta/tira,
  não). Um script com controlo positivo (dois patches diferentes TÊM de dar assinaturas diferentes) —
  foi ele que achou os 2 de 130; `git range-diff` também serve, mas é ruidoso.
- Para ficheiros-lista partilhados (as listas numeradas de LOC, catracas), ponha o merge em **texto**
  durante a integração (`$GIT_COMMON_DIR/info/attributes`: `<path> merge=text`, precedência máxima) e
  resolva pelos estágios com um resolvedor por CHAVE de entrada com `assert` de contagem
  ([[feedback-resolve-conflicts-from-index-stages-not-markers]]). Tire a linha no fim.
- Nunca filtre a saída de um rebase para `CONFLICT` só: guarde-a inteira e procure `mergiraf`.
- Esta verificação por patch correu primeiro sob zsh com uma lista não partida e deu «0 diferenças»
  sobre caminhos vazios ([[feedback-a-pastable-bash-loop-never-iterates-under-zsh]]) — escreva-a em
  ficheiro `bash` com arrays e o controlo, nunca inline.
