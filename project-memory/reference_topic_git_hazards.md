---
name: reference-topic-git-hazards
description: Perigos e armadilhas de git/edição multi-agente — stash · reset alheio · fence · worktree-base · mojibake · fmt -p · str.replace · sed -i · rewrite de token · mover doc · commit -- paths ignora `??` (13)
metadata: 
  node_type: memory
  type: reference
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-08-23T00:59:08.535Z
---

- [[feedback_a_commit_with_paths_never_picks_up_an_untracked_file]] — ⛔⛔ `commit -- <paths>` ignora ficheiros `??`: o commit «tem sucesso» e a árvore dele NÃO COMPILA (11 ficheiros novos de fora)
- [[feedback_git_stash_multiagent_danger]] — pop com índice sujo injeta marcador em arquivo alheio
- [[feedback_destructive_git_outside_pasta]] — nunca reset/checkout em path alheio
- [[feedback_destructive_reset_collision_2026_05_28]] — `git add` cedo cria fence
- [[feedback_worktree_agent_stale_base]] — ramifica do HEAD de início; só audit read-only
- [[feedback_perl_utf8_mojibake_use_edit_tool]] — texto acentuado só via Edit tool
- [[feedback_an_unanchored_replace_renames_english_identifiers_inside_pt_br_prose]] — troque a FRASE; depois `git diff | grep "^[-+].*fn "`
- [[feedback_cargo_fmt_p_reformats_foreign_wip]] — `cargo fmt -p` reformata WIP alheio; use `rustfmt <arquivos>`
- [[feedback_python_replace_silent_noop_after_fmt]] — `str.replace()` sem casar é no-op SILENCIOSO; `assert old in s`
- [[feedback_sed_relative_path_hits_primary_cwd]] — `sed -i` relativo erra de repo (a cwd volta ao primário); caminho absoluto
- [[feedback_a_token_rewrite_scopes_to_changed_files_not_the_whole_tree]] — rewrite de token = só arquivos MUDADOS; `git grep` corrompeu um .ttf
- [[feedback_moving_a_doc_means_resolving_links_not_matching_strings]] — mover doc = RESOLVER link, não casar string; `ls-files` pós-`mv` mente
- [[feedback_mutation_undo_with_cp_never_git_checkout]] — desfaça mutação com `cp` do backup, nunca `git checkout`
- ⛔ **Crase numa mensagem de `git commit -m "…"` é SUBSTITUIÇÃO DE COMANDO** (zsh/bash, 2026-09-14): `` `fase_*` `` executou e a mensagem foi gravada com duas palavras **em falta**, sem erro nenhum. ⇒ mensagem densa vai por **`-F ficheiro`** (ou `<<'MSG'` com o delimitador entre plicas), nunca por `-m` com crases.

## ⛔⛔⛔ PROSA DENTRO DE UM HEREDOC `<<EOF` É CÓDIGO — as crases EXECUTAM (2026-09-19)

Ao acrescentar um bloco de comentário explicativo **dentro** de um `cat > ficheiro <<EOF` (heredoc
**sem aspas**), as crases do meu próprio texto viraram **substituição de comando**. O comentário
dizia *«a mesma família do `spectacle` a fotografar o ecrã real»* — e o bash **correu o
`spectacle`**, que é precisamente o programa proibido nesta máquina porque fala com o KWin pelo
D-Bus da sessão do DONO. Ele ficou vivo **449 s** e pendurou o roteiro; a varredura de `~/Imagens`,
`~/Pictures`, `~/Desktop` e `$HOME` por ficheiros novos deu **zero** (abriu e esperou, não gravou).

O sintoma visível foi outro braço da mesma causa: `linha 59: /home/enio/.ph2d/layout.txt: Permissão
negada` — o bash a tentar EXECUTAR o caminho que as outras crases delimitavam. ⚠️ **`bash -n`
passa**: a sintaxe está correcta, o defeito é semântico.

**How to apply:** prosa fica **FORA** do heredoc (ali as crases são inertes); dentro dele só código.
E ponha um guarda sobre o ficheiro gerado — `grep -q '\`' "$gerado" && exit`. *Um heredoc sem aspas
é um programa, e escrever texto lá dentro é escrever um programa sem saber.*

⚠️ Da mesma corrida: um `target/smoke/ph2d-host-desktop` **órfão de 46 minutos** estava a segurar a
placa (a armadilha do `CLAUDE.md` §2 — um binário reparenta-se ao `systemd --user` e sobrevive a
quem o lançou). *Depois de uma fotografia, confira `pgrep -f ph2d-host-desktop`.*
- ⛔⛔ [`git mv` de um ficheiro com edições por encenar grava o blob de HEAD no destino](feedback_git_mv_stages_the_index_blob_not_the_worktree.md) — o commit tem sucesso e a árvore dele **NÃO COMPILA** (sinal: `M <destino>` depois do commit).
## ⛔ Um APÓSTROFO numa mensagem de commit inline PENDURA a consola (2026-09-18)
Duas tentativas de `git commit -m "…que e' maximizar…"` ficaram presas **sem `index.lock` e sem
carga**, e a segunda sobreviveu ao próprio `timeout 90`. **Why:** esta consola embrulha o comando
num `eval '…'` de aspa **simples**; um `'` no texto fecha-a e o shell fica à espera de mais
entrada — não é o git que está lento, é o shell que nunca recebeu o comando inteiro. **How to
apply:** mensagem de commit vai **sempre** por `-F <ficheiro>` (ou heredoc citado), nunca por `-m`
inline quando o texto tem apóstrofos — e o sintoma que a distingue de um lock é **não haver
`index.lock`**: `ls .git/index.lock` antes de culpar o git. ⚠️ Os processos presos matam-se pelo
PID (`pgrep -af "git commit"`) e saem com **144**, que se lê como falha do comando e não é.

## ⛔⛔⛔ Numa WORKTREE, uma memória escrita pela ferramenta aterra na árvore PRIMÁRIA e o ponteiro aterra na worktree (2026-09-20)
O symlink `~/.claude/projects/<key>/memory` aponta para o `project-memory/` da árvore
**PRIMÁRIA**. Uma linha em Modo L que escreva uma memória pela ferramenta põe o FICHEIRO lá e a
linha do `MEMORY.md` — editada por caminho relativo — **cá**: o par separa-se, e só o lado de cá é
medido. Medido nesta sessão: `1` de `155` ponteiros órfão na worktree (o ficheiro existia na
primária, com o `originSessionId` desta janela) e mais **três** ficheiros na primária **sem**
ponteiro nenhum — a outra direcção do mesmo defeito, que gate nenhum vê. **Why:** o gate
`o_indice_da_memoria_conta_o_que_aponta` corre na árvore da linha, logo apanha o ponteiro órfão e
**nunca** o ficheiro órfão; e um ficheiro de memória por rastrear na primária entra na rodada de
outra pessoa ou perde-se. **How to apply:** ao fechar uma linha em worktree, corra o censo dos dois
sentidos (`ponteiros do MEMORY.md` contra `project-memory/*.md` **das DUAS árvores**) e traga para a
worktree o que a sessão escreveu; escrever a memória por caminho RELATIVO a partir da worktree
(`cat >> project-memory/…`) aterra no sítio certo à primeira.
Ver [[reference_topic_integration_discipline]] · [[feedback_the_ruler_is_the_merge_base_not_a_moving_main]].
