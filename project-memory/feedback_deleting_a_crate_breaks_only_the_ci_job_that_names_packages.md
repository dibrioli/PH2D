---
name: feedback-deleting-a-crate-breaks-only-the-ci-job-that-names-packages
description: Apagar uma crate não acorda portão local nenhum (o ship.sh corre --workspace) e parte o job do CI que a nomeia com -p — o envio de 13/09 reprovou nos 3 sistemas antes de compilar por um shim apagado na véspera; portão check-workflow-packages.sh no ship
metadata:
  type: feedback
---

O envio de 2026-09-13 (`aceaa439a`, run 34757814212) reprovou em ubuntu, macOS e Windows em
segundos: o `spike.yml` corre `cargo nextest run -p <26 nomes>` e um deles, o shim deprecado
`ph2d-editor`, fora apagado em `d94e4155c` no dia anterior. O cargo recusa a especificação antes de
compilar, logo nenhum teste correu. O `ship.sh` estava 13 de 13.

**Why:** o nextest local corre `--workspace`, que não nomeia pacote nenhum, e o CI corre uma lista
escrita à mão. A linha que apagou a crate varreu os 2 454 usos no código e não os workflows — o
YAML não é compilado por ninguém localmente.

**How to apply:** quem apaga ou renomeia uma crate corre `git grep -n <nome> .github/`; o portão é
o `scripts/check-workflow-packages.sh` (passo do `ship.sh`), com os membros do `cargo metadata` e os
nomes dos workflows. Irmã de
[[feedback-a-workspace-build-unifies-features-and-hides-a-crate-that-cannot-build-alone]] — as duas
são diferenças entre o que a workspace compila e o que o CI pede por nome.
