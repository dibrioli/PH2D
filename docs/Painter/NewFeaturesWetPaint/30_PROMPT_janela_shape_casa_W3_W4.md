TROCA DE AGENTE — line/PainterWatercolor: o **plano 27**, ondas **W3** (a lei B do #47) e **W4** (o painel por
mídia). W0, W1 e W2 feitas (27 §6, até §6.7).

Leia SÓ: [27 §6](27_plano_shape_casa_dos_pinceis.md) · o handoff
`docs/Painter/handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_W0_W1_SHAPE_CASA.md` (R1–R9, §4b, §4c) ·
o [handoff de integração de 14/10](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-14.md) (o estado no
`main`) · a lista [26](26_ABERTOS.md). ⚠️ «faça como achar melhor e pare de me fazer perguntas» (dono, 11/10). Só o
Wet Paint evolui.

## Fase 0
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current` →
`line/PainterWatercolor`; `git status --short` limpo; `git cherry main HEAD` (a linha foi reaberta em 14/10 sobre o
`main` `0aab2c1cb6`, zero commits à frente); `bash scripts/btrfs-health.sh`.

## O que mudou desde que este prompt foi escrito (14/10)
- **O Continuous é o dab-CÁPSULA** ([31 §5](31_spacing_por_meio_o_pontilhado.md), `ph2d-painter-brush/src/capsula.rs`):
  no Wet Paint os sub-dabs chegam ao motor (`wetpaint/dab_route.rs`), cada um pelo `sil`/`texv` de sempre — uma lei
  nova na `stamp_row_shaped` (W3) vale para eles sem mais nada. A varredura do ladrilho (`TexDabBasis::sweep`,
  `ShapeFrame::Stroke.step`) já não existe.
- **Tectos de LOC (C15 do 26):** `tool/paint/stamp_cache.rs` 700/700 e `painter-brush/src/stroke.rs` 698 — tocar ali
  pede CORTE primeiro.

## Testes e portões — as regras de 13–14/10 (CLAUDE.md §2; há guardas que recusam o resto)
- Por item: `bash scripts/cargo-check-narrow.sh <crate>` e os testes do item POR NOME
  (`bash scripts/cargo-test-narrow.sh <crate> <nome>`) — a crate inteira sem nome é recusada (R5).
- A suíte (`nextest-impacted`) só a cada **10** commits ou no fecho (`PH2D_SUITE=fecho`); **clippy só no fecho** (R7).
- Edite pelo `Edit`; um script que grava sem `assert`/`grep -c` é recusado (R6).

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
R1 ~150 chamadas · `verificador` com `PH2D_SUITE=fecho` (fmt, clippy `-D warnings --all-targets`, `nextest-impacted`)
· `bash scripts/agent-loop-profile.sh` + `rm -rf target/*/incremental` · handoff ≤ 8 KB ·
atualizar 26 e 27 · commit por caminho, nunca push · smoke ao dono em passos numerados. Depois: W5 (Flat/Fan do motor
saem, doc do Shape).
