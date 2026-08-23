---
name: reference-topic-git-hazards
description: Perigos e armadilhas de git/edição multi-agente — stash · reset alheio · fence · worktree-base · mojibake · fmt -p · str.replace · sed -i · rewrite de token · mover doc (12)
metadata: 
  node_type: memory
  type: reference
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-08-23T00:59:08.535Z
---

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
