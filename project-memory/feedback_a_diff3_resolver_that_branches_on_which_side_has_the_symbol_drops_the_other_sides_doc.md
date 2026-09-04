---
name: feedback_a_diff3_resolver_that_branches_on_which_side_has_the_symbol_drops_the_other_sides_doc
description: "Resolver conflito por script: um `if` que escolhe o lado pelo símbolo apaga em silêncio o que o outro lado acrescentou no MESMO hunk"
metadata:
  type: feedback
---

Integração de 2026-09-04, a renumerar `PROJECT_SCHEMA` em 6 degraus de 3 linhas. Escrevi um
resolvedor de conflito diff3 com esta forma:

```python
if any('const PROJECT_SCHEMA' in l for l in lado):
    return [l.replace('= 108;', '= 109;') for l in head]   # ⛔ o doc da linha EVAPORA
```

⛔ **Num commit o bloco de conflito continha o doc E a const juntos**, noutro vinham em blocos
separados. Na versão junta o ramo do `const` ganhou e **o degrau da escada desapareceu**: o número
ficou certo (`109`) e a escada de documentação perdeu a entrada que explica *porquê* ele subiu.

⭐ **Só se viu ao resolver o commit SEGUINTE** — o `head` dele já não tinha o doc que eu julgava
ter escrito. Nada falhou: compila, o gate da tripla passa, e a perda é de PROSA.

**How to apply:**
1. ⭐⭐ **Nunca escolha um LADO — recomponha o hunk por PARTES.** Separe o que é declaração do que é
   documentação e junte as duas: `head_sem_const + doc_novo_da_linha + const_renumerada`.
2. ⭐ **Verifique o que ACRESCENTOU, não só o que compila.** Depois de cada resolução, um `grep` do
   título do degrau novo custa uma chamada e é o único sinal.
3. ⚠️ **Um `git add` de ficheiro com marcadores + `rebase --continue` não falha alto** — o rebase
   avança e o `parse conflict hunks` só aparece no commit seguinte, já com dois estragos
   sobrepostos. Ou o script tem `assert` de que nenhum marcador sobreviveu, ou não se faz `add`.
4. ⚠️ **Um corte por ÍNDICE dentro de um hunk trunca o vizinho:** levei metade de um braço de
   `match` e o compilador disse `unclosed delimiter` — o sintoma barato. Sem sorte, seria código
   silenciosamente mutilado ([[feedback_an_index_splice_between_two_markers_deletes_the_neighbour_in_silence]]).

E o parser em si: `splitlines(keepends=True)` deixa o `\n`, então `linha == '======='` **nunca**
casa. Compare com `.rstrip('\n')`.
