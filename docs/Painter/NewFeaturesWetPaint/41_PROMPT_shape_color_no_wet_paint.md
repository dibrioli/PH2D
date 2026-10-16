# 41 — PROMPT: a Shape Color pinta no Wet Paint (lista 26 A7, plano 27 W6) (16/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo. Leia SÓ: este prompt · lista [26](26_ABERTOS.md) A7 ·
[doc 37 §5](37_rampas_de_cor_shape_grain_papel.md) · [plano 27](27_plano_shape_casa_dos_pinceis.md) linha W6 · §9 do
[handoff P40](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-15_P40_CAPSULA_COM_ESTADO.md).
Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada).

## De onde vimos

P40 fechado e aprovado no smoke (16/10): a cápsula do Continuous carrega o estado do dab. A ordem do dono de 15/10:
*«quero a Shape Color no Wet Paint»* — as duas camadas (Texture Color · Falloff Color) que já pintam no Digital,
Impasto e Watercolor (A6, A9, A10) não chegam à água: hoje a secção some no Wet Paint e o motor recebe UMA cor por dab.

## Pedido

1. **Medir primeiro** (um lote só): onde o motor (`ph2d-wet-paint`) deposita a cor por célula; o custo por dab de
   uma cor por célula; o traço de hoje AO BYTE com a secção desligada (o portão de identidade nasce aqui).
2. A lei: a cor de cada célula pela porta única da rampa (`ramp_alpha::ramp_texel`), Texture Color pelo valor da
   textura do Shape, Falloff Color pela queda — e o que a água faz com ela (mistura, Glaze, Charge) segue a lei do
   Rebelle (a claridade é o pigmento). O Continuous (`capsula::sub_dabs`) e a prévia do Shape têm de concordar.
3. Painel: a secção Shape Color aparece no Wet Paint; os controlos que não agem na água esmaecem (censo).
4. Portões vermelhos primeiro; mutação por lei; prancha (Digital × Wet Paint, as duas camadas).
5. Fecho: suíte afetada + clippy; handoff ≤ 8 KB; smoke ao dono em passos numerados.
