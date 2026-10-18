---
description: Fim de uma ONDA (o dono faz o smoke, a janela troca) — a linha CONTINUA. Sem a bateria inteira.
argument-hint: [Nome da linha] [Nome da onda]
---
Feche a onda `$2` da linha `$1`. **A linha continua** — o fecho da LINHA (`/pd-linha-fechar`, a bateria
inteira) é só antes de integrar.

⛔ **Por que isto existe (medido 14–17/10):** as linhas fechavam CADA onda com a bateria da linha —
Luz2D e PainterWatercolor 14 fechos cada em 3 dias, ondas de 3 a 13 commits, metade deles repetidos
(«second pass», «round 3») porque o fmt/clippy reprovava DEPOIS da suíte. O `PH2D_SUITE=fecho` virou a
cadência de 5 commits por outra porta (CLAUDE.md §2, DIRETRIZ §6.2).

1. **Arrumação, o barato primeiro** (segundos):
   - `cargo fmt --all` (corrige; não é só `--check`)
   - `PH2D_SUITE=onda bash scripts/ph2d-run.sh cargo clippy -p <cada crate que a onda tocou> --all-targets -- -D warnings`
2. **Os testes da ONDA, por nome** — os que ela escreveu ou mudou:
   `bash scripts/cargo-test-narrow.sh <crate> <nome>` (gates de placa: `-- --ignored <nome>`; a vez da placa pede-se sozinha).
3. **A suíte só se a cadência mandar:** `PH2D_SUITE=onda bash scripts/nextest-impacted.sh` — corre com ≥ 10
   commits desde a última verde; antes disso recusa em ~2 s (exit 3), e isso é o esperado, não um ✗.
4. **O smoke do dono compilado** — o passo 6 do `/pd-linha-fechar`, igual.
5. **Commit + handoff de CONTINUAÇÃO** (`HANDOFF_CONTINUACAO_<onda>`, ≤ 15 KB, pelo `documentador`):
   o que a onda fez, os números medidos, o smoke em passos numerados, o próximo item.
6. PARE e entregue o smoke ao Enio.

Depois de um conserto: re-corra **só o que falhou, por nome** — nunca a onda inteira de novo.
