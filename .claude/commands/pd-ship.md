---
description: Só quando eu mandar. Verde antes de push, sempre.
---
Ship da jornada.

1. `./scripts/ship.sh` — paridade EXATA com lint+test do CI (fmt, clippy --all-targets
   +features, machete, deny, audit, nextest --cargo-profile ci-test, typos).
   Corrija TODO ✗. NÃO pushe antes de verde.
   ⚠️ Um ✗ pode ser AMBIENTE, não código (tmpfs que evaporou) — verifique o ESTADO.
2. `git push origin main`
3. Babysit até success (polling 15min, `gh run watch`). Vermelho: fix + re-push.
   Escalone para mim após 3 falhas do mesmo job.
4. Me dê SEMPRE o link: https://github.com/dibrioli/PH2D/actions/runs/<id>
