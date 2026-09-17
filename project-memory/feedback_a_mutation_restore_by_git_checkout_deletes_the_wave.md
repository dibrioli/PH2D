---
name: feedback_a_mutation_restore_by_git_checkout_deletes_the_wave
description: Restaurar uma mutação com `git checkout -- <ficheiro>` numa árvore SUJA não desfaz a mutação — desfaz a FATIA, devolvendo o ficheiro ao HEAD.
metadata:
  type: feedback
---

Medido em 2026-09-17 (`line/UIUX`, 7.ª fatia do HR-15). O arnês de mutação fazia
`restaura() { git checkout -- "$1" && touch "$1"; }`. A wave ainda **não estava comitada**, logo o
`checkout` não devolveu o ficheiro ao estado de ANTES DA MUTAÇÃO — devolveu-o ao **`HEAD`**, e
**três ficheiros da fatia foram apagados** (o `Panel::TITLE` tipado do painel, o pintor do cabeçalho
e as nove entradas novas da tabela de strings, o acessor `TextKey::key` incluído).

**Why:** `git checkout -- <path>` não conhece «mutação»; ele conhece o índice. Num fluxo de mutação
o ficheiro tem **duas** diferenças empilhadas — a da wave (por comitar) e a da mutação — e o
`checkout` deita as duas fora. ⚠️ **O modo de falha é MUDO no sítio onde acontece:** a corrida
seguinte compila menos, ou não compila, e o relatório diz *«sobreviveu»* ou *«ARNÊS-PARTIDO»* sobre
um produto que já não é o da wave.

**How to apply:** a cópia de segurança é do ficheiro **COMO ESTÁ**, e o restore é dela —
`cp -p f f.bak` antes de mutar, `mv -f f.bak f && touch f` depois (⚠️ o `touch`:
[[feedback_a_mutation_restore_by_mv_leaves_cargo_with_the_mutated_build]]). ⭐ **E quem me disse foi
o CONTROLO DO FILTRO** ([[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]]): as três
mutações seguintes casaram **zero** testes porque a árvore deixara de compilar, e o arnês recusou-se
a chamar-lhes «sobreviveu». *Sem esse controlo eu teria lido três falsos sobreviventes e ido curar
gates que estavam certos.* ⛔ A alternativa — comitar a wave antes de mutar — também serve, e é a
que deixa o `git checkout` honesto; escolha uma, nunca as misture.
Irmãs: [[reference_topic_mutation_proofs]] · [[reference_topic_git_hazards]]
