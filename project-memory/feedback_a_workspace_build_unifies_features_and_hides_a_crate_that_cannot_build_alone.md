---
name: feedback-a-workspace-build-unifies-features-and-hides-a-crate-that-cannot-build-alone
description: Crate que nomeia uma dependência OPCIONAL fora do cfg dela compila verde em toda build da workspace (a shell liga a feature) e não compila sozinha — o ph2d-app-flip tinha 75 erros com o ship.sh a 12/12; o portão é `cargo check -p` por crate (13/09)
metadata:
  type: feedback
---

A descida dos ids da A5b moveu para `ph2d-panel-flip`/`-frames` os ids que o `ph2d-app-flip` lê
sem `cfg`, e esses dois eram dependências **opcionais** dele. Na integração de 2026-09-13 o
`clippy --workspace`, o `nextest --workspace` e o `ship.sh` inteiro passaram (12/12, 22 720 testes);
um `cargo nextest run -p ph2d-app-flip` isolado deu 75 erros E0433. Varridas as 48 crates tocadas,
cada uma sozinha: só essa falhava.

**Why:** numa build da workspace o cargo unifica as features de todos os membros, e a shell liga
`panel-flip` por omissão — a crate nunca é compilada com a feature desligada em portão nenhum. É a
§2.4 do HOWTO («as features não viajam») pela porta de trás: não foi o código a mudar de crate, foi
a CONSTANTE a mudar para trás de uma dependência opcional.

**How to apply:** mover um símbolo lido sem `cfg` para uma crate exige que o leitor dependa dela
sem condição (ou que a leitura ganhe o `cfg`). O portão é o `scripts/check-standalone-optional.sh`
(passo do `ship.sh`), que pergunta ao compilador crate a crate, com a lista derivada dos
manifestos. Irmã de [[feedback_an_orphaned_cfg_attaches_to_the_next_item_and_the_default_build_is_blind]].
