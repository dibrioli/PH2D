---
name: feedback_sweep_conflict_markers_every_commit
description: "Ao resolver conflito de rebase, varra marcadores em TODO commit do branch (não só na árvore) — um `<<<<<<< HEAD` órfão commitado não compila e envenena o bisect"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ac6fba2f-c694-4c47-a142-9e06671dae88
---

Resolver conflito à mão deixa marcador para trás com facilidade: o `=======` e o `>>>>>>>`
são visíveis no meio do diff, mas o **`<<<<<<< HEAD` fica dezenas de linhas acima** e passa
batido. Ele entra no commit, aquele commit **não compila**, e como o gate só testa o TIP do
branch, ninguém percebe — até um `git bisect` futuro cair nele.

**Como mordeu (rebase da `line/Vector` sobre a `line/FLIP`, 2026-07-11):** resolvi um bloco em
`render_loop/mod.rs` com dois Edits (removi o `|||||||`/`=======` num, o `>>>>>>>` no outro) e
**esqueci o `<<<<<<< HEAD`**. Um `grep -c` me devolveu `1` e eu li como ruído. Dois commits
depois o marcador ainda estava lá, já commitado.

**Como aplicar — duas varreduras, sempre, antes de `rebase --continue` e ao fim do rebase:**
```bash
# 1. a working tree
grep -rn '^<<<<<<< \|^=======$\|^||||||| \|^>>>>>>> ' --include='*.rs' --include='*.toml' crates/ shells/
# 2. CADA commit do branch (a que salva o histórico)
git log --oneline main..HEAD | while read -r sha _; do
  n=$(git show "$sha" --format="" --unified=0 | grep -c '^+<<<<<<< \|^+>>>>>>> ' || true)
  [ "$n" -gt 0 ] && echo "⚠ $sha introduziu marcador"
done
```
**Conserto sem `-i` interativo** (que trava esperando editor): rebase roteirizado —
`GIT_SEQUENCE_EDITOR="sed -i 's/^pick <sha>/edit <sha>/'" GIT_EDITOR=true git rebase -i <sha>~1`
→ corrige → `git commit --amend --no-verify --no-edit` → `git rebase --continue`.

Ver [[feedback_pipe_masks_script_exit_code]] (mesma família: não confie no sinal fácil, cheque
o estado) e [[feedback_integration_only_enio_command_end_of_all_lines]].
