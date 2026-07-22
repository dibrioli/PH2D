---
name: feedback_a_restored_file_keeps_its_old_mtime_and_cargo_reuses_the_mutant
description: "Script de mutação: restaurar por `shutil.move` do .bak devolve o mtime ANTIGO ⇒ o cargo pula o rebuild e reusa o artefato MUTADO — a linha seguinte roda contra o mutante e o baseline fica vermelho sem causa"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b464385a-3461-49b7-a757-c9961ffd5f30
  modified: 2026-07-22T00:28:27.516Z
---

Um script de prova de mutação que faz `shutil.copy(f, f+".bak")` → escreve o mutante → roda → `shutil.move(f+".bak", f)` **restaura o conteúdo certo com o mtime ORIGINAL**. O `copy` preserva mtime, e o `move` o traz de volta — então o arquivo restaurado fica **mais velho que o artefato compilado a partir do mutante**, e o cargo o considera atualizado.

Aconteceu 2026-07-21 (linha `line/anim-fixes`, 10 mutações sobre `ph2d-panel-timeline` + `ph2d-timeline`). A mutação 3 tirava o teto de `add_container` na `ph2d-timeline`; as mutações 4–10 só tocavam o painel, então **a `ph2d-timeline` parecia intocada e o cargo reusou a rlib mutada** — o gate do teto aparecia como "morto" em TODAS elas, e o baseline pós-script ficou **vermelho** com `left: 16, right: 15` sobre um código-fonte correto.

⚠️ **Não é só sobre mutação.** Vale para qualquer restore de arquivo por cópia/backup (`cp -p`, `rsync -t`, `tar -x` preservando timestamps, um `git stash pop` não): **conteúdo certo + mtime velho = build velho**.

**Why:** a fingerprint do cargo é por-crate e usa mtime. Uma crate cujo arquivo mais novo é anterior ao artefato não é recompilada — e o `cargo` compila a crate como UNIDADE, então enquanto *alguma* fonte dela parece nova o mutante some sozinho; o veneno só fica quando a crate inteira parece intocada, que é exatamente o caso de "mutei outra crate desta vez".

**How to apply:**

> Depois de **todo** restore de arquivo num loop de build, `os.utime(p, None)` / `touch p`.
> E antes de acreditar num baseline pós-script, `touch` em tudo que o script tocou e rode de novo.

Sinais de que você está nisto: um gate **que não tem relação com a mutação** aparece na lista de mortos (e reaparece em toda mutação seguinte); o mesmo gate passa sozinho (`--test <nome>`) e falha no run combinado; e o número da asserção é o do MUTANTE, não do fonte que você está lendo. Irmão do [[feedback_a_negative_search_needs_a_positive_control]] — o verde/vermelho estava sendo produzido por algo que não era o código sob teste. Vide também [[feedback_mutation_undo_with_cp_never_git_checkout]] e [[reference_topic_mutation_proofs]].
