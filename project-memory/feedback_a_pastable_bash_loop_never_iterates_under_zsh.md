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
