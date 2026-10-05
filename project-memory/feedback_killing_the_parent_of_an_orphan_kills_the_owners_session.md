---
name: feedback-killing-the-parent-of-an-orphan-kills-the-owners-session
description: «matar o PAI» de um processo órfão mata o `systemd --user` (o subreaper) — a sessão gráfica INTEIRA do dono acaba (05/10, 18:30, parecia «o PC reiniciou»)
metadata:
  type: feedback
---

Em 2026-10-05 18:30:03 a sessão gráfica do Enio terminou (ecrã de login, VS Code e todas as janelas
do Claude mortos; o PC NÃO reiniciou — uptime contínuo). Causa medida no journal + transcript: um
agente (janela da `line/motion-value`) quis parar um `bash scripts/ph2d-run.sh` e correu
`pp=$(ps -o ppid= -p <pid>); kill $pp`. O processo era ÓRFÃO — o shell que o lançou já tinha saído —
e um órfão é adoptado pelo **subreaper** da sessão, o `systemd --user` (PID 1271). O `kill` mandou
SIGTERM ao gestor da sessão, e 26 ms depois o journal regista `systemd[1271]: Activating special unit
Exit the Session` → logout ordenado de tudo.

**Why:** «matar o pai» só é seguro quando o pai é um embrulho teu; num órfão o pai é o dono da
sessão. A casa já sabia metade (*«matar quem lançou NÃO mata o teste — ele reparenta-se ao `systemd
--user`»*, DIRETIVA_IMPLEMENTACAO §5), e foi exactamente essa reparentação que transformou o gesto
de limpeza num logout.

**How to apply:** NUNCA `kill $(ps -o ppid= …)`. Para parar uma corrida: mate o PID dela e o grupo
(`kill -- -<pgid>`), ou pare o scope da linha (`systemctl --user stop 'ph2d-run-*.scope'` /
`systemctl --user list-units 'ph2d*'`). Antes de qualquer `kill` de um PID calculado, confira
`ps -o comm= -p <pid>` ≠ `systemd`. Sintoma para a próxima vez: «o PC reiniciou» com `uptime`
contínuo ⇒ procure `Exit the Session` no `journalctl -b 0` e o comando do minuto anterior nos
transcripts. Ver [[feedback_a_pgrep_watcher_catches_its_own_shell]].
