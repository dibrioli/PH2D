# Regras de TODO agente da missão Kanban (leia inteiro antes do primeiro comando)

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-TrelloClone` · branch `line/TrelloClone`.
Primeiro comando: `pwd && git branch --show-current` (o cwd já é a worktree; use caminhos absolutos ou relativos a ela).
O dono NÃO está presente; ninguém responde a perguntas. Decida pelo padrão-ouro; dúvida de PRODUTO →
`docs/TrelloClone/01_plano.md` §8, a maquete `docs/TrelloClone/maquete/project/*.dc.html`, e o que o Trello faz.
Toda decisão de produto tomada sem o dono: devolva-a ao coordenador numa linha (ele regista).

## Formas PROIBIDAS (um guarda recusa ou a missão congela num pedido de permissão)
- `cd X && …` encadeado, `bash -c`/`sh -c`, `rm -rf` com variável ou glob, `kill`/`pkill` de PID que não lançou
  (NUNCA o pai de um órfão = logout do dono), escrever em `.git/`, `.claude/` ou rc da shell.
- `git push`, merge/rebase no `main`, `git add -A`/`git add .`, `git stash`, `git commit` sem `-- <paths>`.
- `spectacle` (fotografa o ecrã REAL do dono). Foto: `docs/Components/ferramentas/fotografa_cena.sh` ou
  `import -window` dentro de `kwin --virtual`.
- `sed -i`/`perl -i` sem `grep -c` na MESMA linha; script python que grava sem `assert` de contagem. Edite com `Edit`.
- Vigia de fundo sem `timeout`; `until ! pgrep -f …` (auto-apanha-se) — espere pelo PID.
- Suíte completa, `nextest-impacted`, `PH2D_SUITE=fecho`, crate inteira sem nome (salvo crate NOVA:
  `PH2D_SUITE="motivo: crate NOVA ph2d-…"`), clippy a meio do trabalho, `cargo test --release`.

## Como trabalhar
- Máquina PARTILHADA: comando pesado por `bash scripts/ph2d-run.sh <cmd>`. Antes de acusar um processo de
  pendurado, leia `utime+stime` de `/proc/<pid>/stat` DUAS vezes.
- Inner loop: `bash scripts/cargo-check-narrow.sh <crate>`. Teste por NOME: `bash scripts/cargo-test-narrow.sh <crate> <nome>`
  (exit 0 verde · 1 vermelho · 2 não compilou · 4 nome não casou).
- Smoke/sondas: `--profile smoke`. Ex.: `PH2D_KANBAN_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`.
- UI canónica (HR-15): zero hex, zero `f32` literal de UI, zero texto fixo → tokens `kanban-*`
  (`crates/ph2d-tokens`, contraste medido pelo gate) e chaves em `crates/ph2d-i18n/src/kanban.rs`.
- Código da família em `crates/ph2d-app-kanban` ou drop-crate `ph2d-kanban-*`. A shell (`shells/desktop`) só
  compõe; nunca suba a catraca `the_shell_only_shrinks`.
- Tetos: 700 linhas por `.rs`, 600 nos testes — parta antes. Comentário curto (o código diz a lei).
- Nenhum `MAX_*`/limite sem medir: escreva a tabela de medição em `docs/TrelloClone/loop/MEDICOES.md`.
- Seam tests pelo hero, no estilo de `crates/ph2d-app-kanban/tests/it/`.
- Agentes em PARALELO na mesma worktree: toque SÓ os caminhos que o seu briefing lhe dá. Ficheiro partilhado
  fora deles (workspace `Cargo.toml`, `lib.rs` de outra crate, shell…) → devolva o trecho ao coordenador.
  Se outro agente deixar a árvore sem compilar fora dos seus caminhos, espere/repita o check; não conserte o dele.
- Commit SÓ dos seus caminhos: `git add -- <paths>` e `git commit --no-verify -m "…" -- <paths>`
  (ficheiro novo: `git add -- <path>` ANTES, senão o commit não o leva). Fim da mensagem:
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Resposta final ao coordenador: NO MÁXIMO 25 linhas
O que fez · commits (hash curto) · testes por nome verdes · foto(s) (caminho) · decisões de produto · o que
ficou por fazer e onde. Nada de diffs nem de logs.
