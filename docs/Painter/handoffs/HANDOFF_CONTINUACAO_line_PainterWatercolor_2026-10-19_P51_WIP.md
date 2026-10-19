# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 51 (WIP): Spacing 0,1 + Continuous em todos os pincéis (2026-10-19)

## §1 — Coordenadas

Sobre o `main` `ad5f8fe39c` (merge-base, sem rebase). Sem push. [Prompt 51](../NewFeaturesWetPaint/51_PROMPT_spacing_continuous_em_todos_os_pinceis.md).
Commits `4d430227f8` (WIP 1) + o commit deste handoff. A régua e as medições gravadas em
[`51_regua/`](../NewFeaturesWetPaint/51_regua/) (o resumo `resumo.py`, o custo `tabela_custo.py`). Anterior: [P50](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-18_P50_W3b_SEM_ESCADA.md).
**Decisão do dono (18/10):** o pincel SEM preset entra na regra (feito: o slot do Wet Paint em `state_default.rs`).

## §2 — A lei (medida; o porquê de cada peça)

O pigmento SOMA por dab na janela do rasto (`trail/deposit.rs`): o Spacing 0,01 punha 2,2× a tinta do 0,025, o 0,1 sem
Continuous 0,25×. E o Continuous do Wet Paint são sub-dabs a 0,025 do diâmetro (`dab_route.rs`): para os 17 pincéis a
0,025 a regra não dava velocidade; para os 3 óleos (0,01) sim. O que entrou, cada um contra a régua:
1. **Pigmento por COMPRIMENTO** — knob oculto `pigmentSpacing` (o Spacing de afinação; `0` = a lei de antes): cada dab
   pousa `opacidade × Δarco ÷ passo_ao_longo_px(pigmentSpacing)` (Δarco desde o dab anterior da LANE, `Lane::arco` — a
   corda entre centros dava 1,82× no Charcoal, que espalha). Sem cápsula nunca > 1. Referência pelo raio do PINCEL
   (`brush`, não `spec` do dab): o do dab traz a pressão (+27 % na rampa do Wet Round) e a caneta deitada (×3,3).
2. **Sub-dabs nos pontos do traço DENSO** (`capsula::sub_dabs`): múltiplos do sub-passo no ARCO; o sub-passo é um
   número INTEIRO de células (`sub_passo_sem_batimento`: até 1,25 → 1, depois o mais perto) — um passo fraccionário
   bate com a grelha num período `1/(1 − 1/passo)` (Oil Bristle a 41 px 4,2 px, Express 8,1 px: os medidos; a 103 px
   a escada do Oil Bristle 3,06 a 2,06 px contra 2,24 a 2,0). O sub-passo pelo raio do pincel.
3. **Cada sub-dab o seu sorteio** (`sorteio_do_sub_dab`, splitmix sobre o fluxo do dab): repetido, dava o período do
   Spacing.
4. **A janela do rasto recebe o sub-passo NATURAL** (antes de arredondar): ela conta dabs (`trail.rs::on_segment`); o
   arredondado a 2 células virava-a para 1 e as três de camadas falhavam por um fio.
5. **Pente do relevo por comprimento** (`penteia(h, alvo, w, passos)`, `1 − (1 − TAXA·w)^n`): a borracha (n = sub ÷
   célula) — ela deixava 1,9e-3 px. ⚠️ Também ligado na TINTA (n = Δarco ÷ célula): ver §4.1.
6. **Por preset** (receitas): os 3 óleos `pigmentSpacing` 0,01 (afinados a 0,01); **Oil Bristle** `pigmentSpacing 0`
   (satura: tinta 0,985 sem o peso; o peso só lhe dava escada) e `capsuleMaxPx 2` (os fios têm largura em px: a 3 px a
   escada lê 4,71); **Acrylic Round** `capsuleSpacing` 0,01 (as cerdas dela andam com o dab: a 0,025 a escada dobrava
   6,2 → 12,5). Knobs novos, ocultos, no fim do registo: `pigmentSpacing`, `capsuleSpacing`, `capsuleMaxPx` (KNOB_COUNT 88).
7. **A regra**: `SPACING_DA_FABRICA` 0,1 + `continuous_spacing` no gerador; censo `a_regra_do_spacing_da_fabrica`
   (os 20 e o pincel sem preset); o portão da lei `o_pigmento_por_comprimento::a_tinta_nao_muda_com_o_spacing`
   (controlo `pigmentSpacing=0`; 21 s sozinho).

## §3 — O que mediu (antes da rodada §4; a máquina com carga 5–30, as RAZÕES valem)

- **Custo, traço rápido** (`sonda_spacing_continuo::sonda_o_custo_do_spacing`, release): Oil Bristle a 103 px
  **13,8 → 4,1 ms/quadro** (807 → 308 dabs; com o tecto de 2 px ~403 dabs, 6,2 ms com carga); Express 18,0 → 5,0;
  a 40 px os pincéis de grão 11,2 → 15,6 (Graphite, Charcoal: o sub-passo vai a 1 px) — todos ≤ 16,7.
- **Tinta a 103 px** (antes × fábrica): os 20 a ±2,5 %. Métricas do portão das mídias: iguais salvo Graphite Textura
  8,77 → 7,74 (R 9,09) e Oil Pastel 13,1 → 11,0 (R 17,35).
- **Oil Bristle a 41 px: IDÊNTICO ao de antes** em todas as métricas (o arco a 1 px é o traço denso).

## §4 — ABERTO (próxima janela, nesta ordem; a bateria de 19/10 em `51_regua/bateria_2026-10-19_falhas.txt`)

1. ⛔ **Escada do Oil Bristle a 103 px voltou** (3,05 · 2,94 · 2,87; tecto Rebelle+0,3 = 2,78 · 2,60 · 2,50) depois do
   última rodada (§2.1 raio do pincel, §2.4 janela natural, §2.5 pente na tinta — entraram juntos; antes dela
   2,23 · 2,39 · 2,17). Suspeito principal, a MEDIR primeiro (variantes no mesmo processo):
   `passos_do_pente = 1` na tinta (o pente por comprimento só na borracha). Sonda: `PH2D_SONDA_PRESETS="Oil Bristle"
   … sonda_spacing_continuo::sonda_a_regua_do_spacing` + `o_relevo_dos_fios::o_relevo_nao_tem_escada`.
2. `a_lei_da_tinta::a_terceira_camada_da_mesma_tinta_nao_clareia`: Dip Pen, medianas 0,1396 · 0,0989 · 0,0971 ·
   0,0950. Medir com a lei por partes (o diagnóstico de 18/10 só tinha o Marker, curado pela janela natural).
3. `midias_de_fabrica`: Graphite Textura 7,60 e Oil Pastel 10,80 fora do vale (controlo 7,70); Airbrush Pontas 0,596
   (R 0,8, controlo 0,31). Medir se é o sub-passo a 1 px (mais dabs sobre o grão) ou o sorteio por sub-dab.
4. Repinar no FIM (a lei fechada): `a_lei_da_tinta` (22 cenas), `opcoes_tests` (6), `shape_color_tests::
   a_secao_desligada_…` (6), `o_po_do_aerografo::sem_po_…` (1) — e dizê-lo no commit. A miniatura e a largura já passam.
5. Re-medir os três abertos do 50 §4 (a cor K–M por célula, o pente, o orçamento por quadro) com o custo novo.
6. Fecho: `nextest-impacted` + clippy (`PH2D_SUITE=fecho`), o doc `51_medicao` (as tabelas de `51_regua/`, a lei §2,
   ⛔ Recusas: o sub-passo fraccionário; o peso no Oil Bristle; a 0,025 na Acrylic; o tecto de 3 px; a corda entre
   centros; o «para cima» de 4,01 → 5; o pente por comprimento na tinta se §4.1 o confirmar), handoff ≤ 8 KB, e o
   smoke ao dono: `PH2D_WETPAINT_SMOKE=2` com um traço RÁPIDO + prancha antes/depois dos 20 (fotografada).

⚠️ Flakes de carga vistos na bateria (passam sozinhos): `offthread_tests::the_water_runs_at_a_usable_rate…`,
`parallel_rows::the_parallel_walk_is_actually_faster`.
