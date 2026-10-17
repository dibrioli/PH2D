---
name: project_agent_time_split_2026_10_17
description: "Para onde vai o tempo das linhas (medido 17/10): 61 % espera de fundo (ajudantes), 23 % gerar (56 % pensar, xhigh), ~8 % testes — e o que foi medido e RECUSADO (tecto de CPU, os 10 min)"
metadata:
  type: project
---

**Medido 17/10/2026** com `python3 scripts/agent-time-split.py --desde 2026-10-14T16:00` (14/10 16h → 17/10,
7 linhas, ~100 h ativas das janelas principais; a espera pelo dono NÃO conta):

| parte | % | o que é |
|---|---|---|
| espera de FUNDO | 61–63 | a janela pára (100 % das vezes o turno acaba em texto) até um ajudante/comando avisar: ajudantes gerais 29 h (são pedaços de implementação, Opus xhigh), verificador 9 h (fechos repetidos — curado), comandos longos 9 h (oráculos), mutação 3 h, integrador 2 h |
| gerar | 23 | **56 % disso é PENSAR** (`effortLevel: xhigh` no `~/.claude/settings.json`); custo fixo ~4,2 s/passo, 90 % dos passos fazem 1 só chamada |
| testes | ~8 | depois das regras R5–R7 de 14/10 |

**Why:** as regras de 13–14/10 (suíte a cada 10, teste por nome, clippy só no fecho) acertaram no que
miravam e mexeram ~13 % — porque os testes já eram a parte pequena. Medir a DIVISÃO antes de mirar.

**Curado 17/10:** fecho de ONDA ≠ fecho de LINHA (`/pd-onda-fechar`; Luz2D/Painter fechavam 14× em 3 dias,
metade repetidos por fmt/clippy depois da suíte) · R8 (sonda em `--profile smoke`, não `--release`: 114 s ×
3,1 s a remontar a motion) · tecto de memória do primário soma o tmpfs (morria calado).

**⛔ Medido e RECUSADO (não refaça):**
- **Tecto de CPU 50 % por linha** (pedido do dono, 15/09): custa ~3 % — release 118 × 115 s; smoke depois de
  editar a `ph2d-core` 5,2 × 5,2 s; `ci-test` 27,4 × 26,6 s (2.ª rodada; a 1.ª, 34,8 s, era o arranque). Os
  contadores `nr_throttled` (21–39 %) parecem alarmantes e não viram relógio.
- **Comando que passa de 10 min NÃO se perde:** o arnês passa-o para fundo e avisa.

**How to apply:** pedido de «mais velocidade» ⇒ corra o `agent-time-split.py` primeiro e mire a maior
fatia. A alavanca grande que sobra é decisão do dono: `xhigh` → `high` (proposto testar numa linha, 1 dia).
Ligado: [[reference_session_transcripts_are_a_measurable_instrument]] · [[project_dev_speed_audit_2026_09_10_w0_applied]].
