---
name: feedback_a_pastable_bash_loop_never_iterates_under_zsh
description: "`for p in $VAR` é idioma BASH e NÃO divide em zsh — um bloco colável de runbook itera 1× com a string inteira, e um portão que enumera passa SEMPRE, calado"
metadata:
  type: feedback
---

**O shell interativo desta máquina é `zsh`, e o zsh NÃO faz *word splitting* em
expansão não-citada.** `VAR="a b c"; for p in $VAR` dá **uma** iteração com `p="a b c"`
— em bash daria três. Um script com shebang `#!/usr/bin/env bash` está a salvo; **um
bloco COLÁVEL de runbook não está**, e é justamente o formato que alguém executa.

**Como mordeu (fim de dia, 2026-08-19):** o portão **duro e global** da
`DIRETIVA_FIM_DE_DIA.md` §4 — *"build ativo aborta a limpeza inteira"* — rodou
`pgrep -x "cargo rustc mold cc1 ld rustdoc"` (31 chars). O `pgrep` **recusa padrões
acima de 15 caracteres**, avisou em **stderr** e devolveu zero. ⇒ o portão que protege
197 GB de `rm -rf` **passava sempre, por avaria**. A limpeza daquele dia foi segura
porque os construtores tinham sido medidos **num comando à parte** — *um resultado
correto obtido com instrumento morto não valida o instrumento*.

**Como aplicar:**
- **Array, sempre, e expandido CITADO:** `BUILDERS=(cargo rustc mold)` +
  `for p in "${BUILDERS[@]}"` — imune ao IFS **e** ao shell.
- **Portão que ENUMERA exige CONTROLE POSITIVO.** Ele não pode provar a própria
  negativa: pergunte se o instrumento vê algo que você SABE que existe (ex.: o próprio
  shell, `pgrep -x "$(basename "$(readlink -f /proc/$$/exe)")"`) e aborte se não vir.
  Sem isso, *"ninguém está a construir"* e *"eu não consigo ver ninguém"* leem igual.
- **Prova vermelha antes de confiar:** ponha na lista um processo vivo e confirme que
  o portão ABORTA. Um portão que nunca foi visto a reprovar não é um portão.

⚠️ **A espécie é diferente de uma sonda que lê errado** ([[feedback_a_silenced_instrument_reads_as_a_result]]):
aqui não há resposta errada a inspecionar — **a pergunta não chegou a ser feita**. Um
laço que nunca itera é um portão que sempre passa. Irmã de
[[feedback_pipe_masks_script_exit_code]] (o veredito está no ESTADO, não no `$?`) e de
[[feedback_a_tool_is_adopted_only_when_a_written_step_names_it]] (o defeito mora no
formato que alguém de facto executa).

## ⛔ RECORRÊNCIA — 2026-09-12, e a forma mais cara: um `git add -- $LISTA` (integração da W2 Fase D)

O integrador pôs sete caminhos numa variável (`C1="a.rs b.rs …"`) e correu `git add -- $C1 && git
commit …` três vezes. Sob zsh a variável **não se dividiu**: o `git` recebeu **um** caminho com
espaços, respondeu `fatal: pathspec … did not match any files`, e **nenhum commit foi gravado** —
com o `&&` a garantir que nada se seguia, o que foi a metade boa.
⚠️ **Esta memória já existia e não foi lida antes de o escrever.** *Uma lição num ficheiro que
ninguém consulta no momento de agir não impede nada.*
⇒ **escreva os caminhos LITERAIS no comando** (`git commit -- a.rs b.rs`), ou use um array
(`arr=(a.rs b.rs); git add -- "${arr[@]}"`), que se comporta igual em bash e zsh.
