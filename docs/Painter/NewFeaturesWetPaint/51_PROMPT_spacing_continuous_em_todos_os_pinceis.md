# 51 — PROMPT: Spacing 0,1 + Continuous em TODOS os pincéis, sem que mudem de cara (18/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo; `bash scripts/btrfs-health.sh`. Leia SÓ: este prompt ·
o [handoff P50](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P50_W3b_SEM_ESCADA.md) §6 · o
[50_medicao](50_medicao_o_relevo_sem_escada.md) §4 (a auditoria do traço rápido) · o
[34](34_PROMPT_janela_continuous_capsula.md) e o [40](40_PROMPT_capsula_carrega_o_estado_do_dab.md) só no que dizem do
Continuous (o dab-cápsula) e das Recusas. Regras de 13–14/10 (teste POR NOME, clippy só no fecho), R8 (sondas em
`--profile smoke`; o relógio em `--release` com `PH2D_RELEASE="mede velocidade"`), §0.9 (o Rebelle corre-se), §0.10.

## De onde vimos

Smoke do dono 18/10: «um traço rápido o FPS cai para 30; antes não era». A bissecção (50 §4, a sonda
`tests/it/sonda_traco_rapido.rs`: Oil Bristle de fábrica a 103 px, 1024², 25 px por evento, 3 eventos por quadro):
main 17/10 **8,7 ms/quadro** → fios W2 13,6 → relevo 31,9 → hoje **13,7** (`1cf41ba2f3`, o que dava para curar ao byte).
O que resta é o NÚMERO de dabs: com o Spacing do pincel a **0,01** cada evento de 25 px pousa ~25 carimbos quase
sobrepostos, e cada um paga o depósito, a cor K–M por célula (`trail/deposit.rs::pousa_a_tinta`, ~3,8 ms) e o pente
do relevo (~2,3 ms). O carimbo corre no handler do ponteiro, sem orçamento por quadro (o «raw» do HUD não o vê).
**O dono testou (smoke): «ainda lento».**

## Pedido do dono (18/10) — o produto final

**Spacing 0,1 (em vez de 0,01) com o Continuous ligado, POR REGRA, em todos os pincéis de fábrica** — e cada pincel
reajustado para não ficar muito diferente do que é hoje.

⚠️ Dois «spacing» diferentes no preset: o do PINCEL (`brush.spacing` 0,01, o da regra) e o knob do motor
`engine.spacing` (2,0 no Oil Bristle) — confira no código qual é qual antes de mexer.

1. **A régua primeiro, um lote só (§0.10):** os 20 presets de fábrica (`receitas_de_fabrica::PRESETS_DE_FABRICA`),
   cada um a Spacing 0,01 (hoje) e a 0,1 + Continuous, no MESMO processo: as métricas que os portões já guardam
   (`midias_de_fabrica` — cobertura, textura, borda, orla, CV, largura, carris, período, pontas, rampa, leve/forte —, a
   escada e o contraste na luminância (`regua_escada`) para os óleos, a largura prometida) e o CUSTO do traço rápido
   (`sonda_traco_rapido`, por preset, a 40 px e ao tamanho do preset). A tabela diz que pincel muda e em quê.
2. **A lei antes do knob:** o que hoje é «por DAB» (o pigmento e a água de cada carimbo, o pente do relevo com
   `TAXA` 0,5 por cobertura, o ganho/oscilação dos fios por arco, a cor por célula que se mistura a cada carimbo, o pó
   do aerógrafo, o grão…) passa a ser por COMPRIMENTO de traço onde a física o pede — o padrão-ouro é o traço que não
   muda com o Spacing (a mesma régua a 0,01 · 0,05 · 0,1). Só o que sobrar se reajusta knob a knob, por preset, e cada
   reajuste é uma linha medida. O Continuous (a cápsula) tem de carregar o que o dab carregava (o 40).
3. **A regra:** a receita de fábrica põe Spacing 0,1 + Continuous em todos; um censo (portão) reprova o preset que
   não a cumpra. ❓ No arranque, pergunte ao dono numa linha se o pincel SEM preset (o padrão do `BrushSpec`) entra
   também.
4. **A barra:** (a) o traço rápido do Oil Bristle a 103 px **≤ 8,7 ms/quadro** (o main de 17/10) e nenhum preset acima
   de 16,7; (b) cada métrica de cada preset no vale do que é HOJE (ou do Rebelle, onde o portão já o compara) — o
   controlo é o preset a 0,1 sem o reajuste; (c) o relevo sem escada (`o_relevo_nao_tem_escada`) e os portões das
   mídias verdes. As cenas de `a_lei_da_tinta` e as miniaturas mudam TODAS (regere-as no commit da regra, e diga-o).
   Depois re-meça os três abertos do 50 §4 (a cor K–M por célula, o pente, o orçamento por quadro): podem ter deixado
   de importar — não os construa sem o número.
5. **Fecho:** suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; smoke ao dono em passos numerados: o
   `PH2D_WETPAINT_SMOKE=2` (o Oil Bristle a 103 px) com um traço RÁPIDO, e uma prancha antes/depois dos 20 pincéis —
   fotografe antes de mandar.

## Na fila (do dono — NÃO junte)

O smoke do relevo sem escada (P50) segue por fazer. C35 (o leve do Oil Bristle) · C36 (a borda a 103 px) · C37 (as
linhas escuras do Rebelle) · A13–A15 · C29, C30, C32 · C33. Depois: W4 (papel), W5 (mistura), W7, W8 (o painel).
