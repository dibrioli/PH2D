# 50 — PROMPT: a W3b — o relevo de cada fio SEM ESCADA (18/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo; `bash scripts/btrfs-health.sh`. Leia SÓ: este prompt ·
o [handoff P49](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P49_W3_RELEVO.md) · o
[49_medicao](49_medicao_o_relevo_de_cada_fio.md) §3, §4, §7 e as Recusas · a [foto do dono](49_smoke_dono_pixelado.png).
Regras de 13–14/10 (teste POR NOME, clippy só no fecho), R8 (sondas em `--profile smoke`), §0.9 (o Rebelle corre-se).

## De onde vimos

A W3 pôs o relevo de cada fio (`ph2d-wet-paint/src/bristles_relevo.rs`, `Grid::altura` preguiçoso, a luz do Impasto
por `impasto_light_porta.rs`) e o Oil Bristle bateu a textura do Rebelle no recto da bancada (5,87 contra 5,86). **O
smoke do dono reprovou-o** («ficou pior, muito pixelado, talvez porque as cerdas tenham ficado mais finas») e o relevo
saiu da fábrica (a lei fica, desligada). A bancada é horizontal: não via a escada.

## Pedido — o produto final

1. **A régua primeiro, um lote só (§0.10):** a escada medida onde o dono a viu — traço INCLINADO (a curva em S da linha
   1 da bancada, e um recto a 20–30°) ao tamanho do preset (103 px) e a 40 px, no nosso e na foto do Rebelle (a curva
   em S existe nas duas). Uma métrica que distinga o relevo de 18/10 do preset de ontem E do Rebelle (ex.: os saltos de
   1 px na NORMAL do traço; a energia a 1 px ÷ a 5 px), com o controlo ao lado. Confira-a contra a foto do dono.
2. **A lei sem escada:** a altura CONTÍNUA através dos fios (o alvo interpolado pela posição dentro da faixa, ou um
   perfil de crista suave por fio — nunca um degrau), o declive abaixo da saturação da luz, e a largura dos fios do
   relevo medida contra o Rebelle a 103 px (o «cerdas mais finas»: hoje 1,5 px fixos). A W2 também tem a cor por fio em
   degrau — meça se a escada é só do relevo antes de lhe mexer.
3. **A barra:** a textura do recto no vale do Rebelle (5,86) E a métrica da escada no inclinado do lado do Rebelle
   (controlo = o preset sem relevo). Portões ao lado de `o_relevo_dos_fios.rs`; mutação nas leis novas; a identidade das
   outras 21 cenas e 19 miniaturas ao byte; regere o Oil Bristle no commit da lei.
4. **Fecho:** suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; smoke ao dono em passos numerados com um
   traço INCLINADO e GRANDE (o dele), ao lado do de ontem — e fotografe-o antes de mandar.

## Decidido pelo dono (18/10)

- **B15: NÃO** — o Acrylic Round fica com as estrias dele; não o alise (lista 26).

## Na fila (do dono — NÃO junte)

C35 (o leve do Oil Bristle) · A13–A15 · C29, C30, C32 · C33. Depois: W4 (papel), W5 (mistura), W7, W8 (o painel).
