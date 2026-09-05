---
name: feedback_pkill_f_with_a_cargo_pattern_kills_your_own_shell_and_the_other_worktrees_build
description: "`pkill -f 'cargo build …'` casa com a PRÓPRIA linha de comando da shell que o corre (exit 144, nada do resto do script corre) e com o cargo de OUTRA worktree — esta máquina corre várias sessões ao mesmo tempo"
metadata:
  type: feedback
---

2026-09-05, `line/UIUX`: quis parar uma compilação de release obsoleta antes de uma prova de
mutação e escrevi `pkill -f "cargo build -p ph2d-host-desktop --release"; python3 - <<'PY' …`.
Resultado: **exit 144**, e a prova nunca correu — o `-f` casa com a linha de comando **da própria
shell** que contém aquele texto, e o `pkill` matou-se a si mesmo antes de chegar ao Python.

⚠️ **E o padrão também alcança as OUTRAS sessões.** O `ps` mostrou um `cargo build -p
ph2d-host-desktop --release` e dois `cargo test -p ph2d-field-eval` de outra janela do Claude
(directório de sessão diferente, outra worktree). Um `pkill -f` por padrão de cargo mata o
compilador de uma linha que não é a minha — é o irmão destrutivo do
[[feedback_a_probe_that_waits_on_pgrep_catches_the_other_worktrees_compiler]].

**Why:** nesta workstation correm várias linhas em paralelo (Modo L), cada uma com o seu `target/`
e as suas compilações; um processo identificado por **texto** não é um processo identificado por
**dono**. E a shell que executa o comando é, ela própria, um processo cuja linha de comando
contém o texto.

**How to apply:**
- ⛔ **Nunca `pkill -f`/`pgrep -f` com um padrão de cargo.** Se uma compilação minha está obsoleta,
  deixo-a acabar (o cargo re-compila só o que mudou) — o custo é minutos; o custo do `pkill` é a
  linha de outra pessoa.
- Se for MESMO preciso parar um processo meu: guardo o PID quando o lanço (o id da tarefa em
  background do harness já o tem), e mato **esse PID**, nunca um padrão.
- ⚠️ Uma prova de mutação nunca corre enquanto uma compilação **minha** da mesma árvore está em
  curso — o backup/restauro muda ficheiros que ela está a ler. Esperar pela notificação de fim
  é a única sequência honesta.
