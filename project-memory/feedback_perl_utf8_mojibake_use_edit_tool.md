---
name: feedback-perl-utf8-mojibake-use-edit-tool
description: perl/sed -i com literais não-ASCII corrompe arquivos UTF-8 (mojibake) — texto acentuado só via Edit tool
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 63a4a831-e323-4bd7-9ba3-274c614260cb
---

`perl -0pi -e 's/…—…·…/…/'` num `.md` pt-BR reescreveu o ARQUIVO INTEIRO em mojibake
(Ã/â€/Â por toda parte) e o commit levou a corrupção junto (2026-07-11, plano da Timeline;
sintoma no terminal: "Wide character in print"). O perl sem `-CSD` trata o arquivo como
bytes, mas o literal do `-e` chega em wide chars → re-encoda tudo errado.

**Why:** os docs canônicos do projeto são pt-BR cheios de acento/travessão/·; um sed/perl
"inocente" destrói o arquivo silenciosamente e o `git add` empacota.

**How to apply:** mutação por shell (perl/sed) SÓ quando padrão E substituição são 100%
ASCII (ok pra código Rust). Qualquer edição envolvendo texto acentuado/em prosa → Edit
tool. Se escapar: `git show <commit-anterior>:<path> >` restaura, refaz com Edit,
`git commit --amend`. Detectar: `grep -c "Ã\|â€"` (0 = são; cuidado com NÃO/AÇÃO legítimos).
Vide [[feedback_sed_relative_path_hits_primary_cwd]] (a outra armadilha de mutação via shell).
