TROCA DE AGENTE — line/PainterWatercolor: o **plano 27**, ondas **W3** (a lei B do #47) e **W4** (o painel por
mídia). W0, W1 e W2 feitas (27 §6, até §6.7).

Leia SÓ: [27 §6](27_plano_shape_casa_dos_pinceis.md) · o handoff
`docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_W0_W1_SHAPE_CASA.md` (R1–R9, §4b, §4c).
⚠️ «faça como achar melhor e pare de me fazer perguntas» (dono, 11/10). Só o Wet Paint evolui.

## Fase 0
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current` →
`line/PainterWatercolor`; `git status --short` limpo; `git cherry main HEAD`; `bash scripts/btrfs-health.sh`.

## W3 — a lei B onde ela ainda falta
Depois da W2 a família do #47 pinta (o padrão já não multiplica as cerdas). O portão ainda corta o padrão quando o
`texv` é o GRÃO (Shape procedural + Grain armado — os pastéis com um Shape Noise): `stamp = env·sv·grão·I − g`.
Meça primeiro (a sonda `sonda_shape_casa_leis_do_dente` com o grão como `texv`) e, se cortar, implemente a lei B
(o `sv` multiplica DEPOIS do portão) por um terceiro canal na `stamp_row_shaped` do motor — o redondo e o Bristles
ficam `× 1` ao bit (portão: as 12 miniaturas de fábrica ao byte).

## W4 — o painel por mídia (26 C1, C2)
- «Use Texture Colors» esmaecido onde a cor da textura não pinta (Wet Paint, Watercolor): `Dependente` em
  `inercia.rs` + a pré-condição no `censo_dos_controlos` (esmaecida ⇔ inerte).
- C2 (o Solid no Wet Paint lê a ponta?) — meça; decida.
- A ponta LISA do Wet Paint deposita 4,7× a tinta das cerdas (27 §6.7): confira no smoke se é a ponta lisa que o
  dono espera; se não, a calibração é do `texv = 1` (um único número, medido contra o Round do Rebelle —
  fixtures `ponta_textura_round_*`).

## Fecho
R1 ~150 chamadas · `verificador` (fmt, clippy `-D warnings --all-targets`, `nextest-impacted`) · handoff ≤ 8 KB ·
atualizar 26 e 27 · commit por caminho, nunca push · smoke ao dono em passos numerados. Depois: W5 (Flat/Fan do motor
saem, doc do Shape).
