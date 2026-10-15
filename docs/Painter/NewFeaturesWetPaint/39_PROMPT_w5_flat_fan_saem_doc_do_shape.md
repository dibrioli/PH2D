# 39 — PROMPT: a W5 do plano 27 — Flat/Fan saem do motor, o doc do Shape, o smoke ao dono (15/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current` → `line/PainterWatercolor`; `git status --short` limpo; `bash scripts/btrfs-health.sh`.
Leia SÓ: este prompt · o [plano 27](27_plano_shape_casa_dos_pinceis.md) §3 (as ondas), §4 (riscos), §6.4 (censo
Flat/Fan) e §6.7–6.9 (W2–W4 feitas) · a [lista 26](26_ABERTOS.md) A1 e B6 · o §1 do
[handoff P38](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P38_COR_DO_SHAPE_EM_CAMADAS.md) (o
estado da linha). Regras de 13–14/10: teste POR NOME, suíte a cada 10 commits (ou `PH2D_SUITE="motivo: …"`), clippy
só no fecho, edição provada. «Faça como achar melhor e pare de me fazer perguntas» (dono, 11/10). Ordem de 07/10:
só o Wet Paint evolui.

## De onde vimos

Os ajustes da cor do Shape (prompts 36 e 38) fecharam com o smoke do dono OK em 15/10: duas camadas (Texture Color +
Falloff Color), ordem, Blend, Opacity por camada, o alfa de cada cor é o alfa da tinta (sem menu Alpha), B&W com as
duas camadas. O dono: *«Finalizamos os ajustes. Voltemos ao plano.»* O plano é o 27 (A1 da lista 26): **W0–W4
feitas**, falta a **W5**.

## A W5 (plano 27 §3: «limpeza (Flat/Fan se mortos), doc do Shape, smoke ao dono»)

1. **Flat/Fan saem do motor.** Censo da W0 (27 §6.4): `BrushShape::{Flat, Fan}` do `ph2d-wet-paint` estão MORTOS no
   produto — só `Round` é construído (`painter.rs:173`); o ramo elíptico (`brush.rs:283-348`) só corre sem silhueta e
   o hospedeiro passa sempre `Some(&sil)` (`dab_route.rs:419,433`). ⚠️ **Reconfira o censo no código de HOJE antes
   de cortar** (as linhas andaram desde 13/10): quem constrói, quem lê, que presets/ficheiros gravados os nomeiam
   (formato gravado = suíte inteira por `motivo:`). Corte com prova: o traço de fábrica dos 12 presets do Wet Paint
   AO BIT antes/depois (as miniaturas e a impressão da sessão — os portões da W2 já existem; use-os). Se algum
   chamador vivo aparecer, PARE essa parte e registe na lista 26 com a medida.
2. **O doc do Shape** — o que o Shape É depois das W1–W4 + P36/P38, para o próximo leitor: as 30 formas (com
   Bristles), o papel de cada slot (silhueta × textura do Wet Paint, a lei B), o painel por mídia, a secção Shape
   Color (as duas camadas, a lei do alfa, o B&W, o «Texture Color sem textura não pinta»). Um doc novo na pasta,
   com links para os portões que provam cada lei; corte docs grandes com `python3 scripts/doc-split.py`.
3. **Smoke ao dono** em passos numerados (CLAUDE.md §0.8), com a prancha ao lado, cobrindo: o Shape Noise e
   Bristles no Wet Paint (o #47 fechado), o painel por mídia, e a pergunta **B6** (a ponta LISA do Wet Paint deposita
   4,7–6,2× a tinta das cerdas de fábrica — é a lisa que o dono espera? A calibração, se não, é um número só,
   `texv = 1`, medido contra o Round do Rebelle, `fixtures/rebelle/ponta_textura_round_*`). Ponha as duas pontas
   lado a lado na prancha para ele decidir vendo.

## Portões (plano 27 §3)

Suíte afetada + clippy no fecho + **mutação nas leis novas** (agente `mutacao`). Tectos de LOC: `stroke.rs` e
`stamp_color_cache.rs` perto de 700 — corte antes de crescer (lista 26 C15).

## Fecho

Handoff ≤ 8 KB (`HANDOFF_CONTINUACAO_line_PainterWatercolor_<data>_P39_W5.md`), lista 26 (A1 → feito ou o que
restar; B6 com a resposta do dono quando vier), o §5 do plano 27 atualizado, e a mensagem ao dono com o smoke.
