# Plano — desempenho total do Painter (ordem do dono 11–12/11)

«Se esforce para resolver todos os problemas de performance do módulo a todo custo.» Censo dos abertos de desempenho
(BACKLOG.md + BACKLOG_AUDITORIA.md + FAIXAS_SEGURAS.md, 12/11), agrupado por FAMÍLIA (CLAUDE.md §0.10): cada onda
cura a família inteira e mede-se numa matriz calma única. Curas sem mudar pixel (alarmes de pincel: a mesma lista
de vermelhos antes × depois). Do DONO, fora do plano: B14.2 (luz cheia no gesto), B13.3 (sono da água), B43.6 (cgu1).

## Ondas

| onda | família | itens | número de partida | régua |
|---|---|---|---|---|
| 1 | toque e Ctrl+Z 4K | envio sem cópia dos planos da luz · Wet: undo sem memset do fluxo, preparo no evento, `poe` dos tiles · B17.5 | Filbert toque ~76 · Ctrl+Z ~62 · Wet toque ~62 · Wet Ctrl+Z ~45 ms | `mede_pen_up_4k.sh matriz` |
| 2 | cerdas largas | B14.3 · W5 (PA-52) · W9 (PA-56) | Flat Bristle 240: dab 1 037 ms p50 · Bristle 16·4000: soltar 1 340 ms · traço p95 190 ms | `mede_o_dab` + perf · matriz com `KNOBS`/`TECLAS` |
| 3 | mistura de cor | B28.1 · B28.2 · B28.3 · B28.4 · B32.A1 | Wet blend 21,6 ms/q (alvo 16,7) · Wash Mix 24 ms/q | `sonda_custo_no_app`, sondas da `mistura_opaca` |
| 4 | água do Wet | B11.3 · B13.5 · B13.6 · B13.7 (= B13.A2) | 34 → 3,6–5 img/s · 26 → 8 img/s do 4.º traço · ms/passo 4,3 → 35 | matriz (img/s), sondas do passo |
| 5 | camadas e compositor | B19.10 · B18.1–18.3 · B39.A3 · B01.A1 · B20.6 · B20.11 · B40.A3–A5 | 16 UHD ~126 ms [E] · troca 66 MB 7–10 ms [E] · salvar 130–190 ms [E] · máscara 17–28 ms/q [E] | matriz `CAMADA_*` · sondas novas |
| 6 | cadência e caneta | B21.1 (Mailbox) · B27.1 · B22.1/22.4 · B23.2 (resto do laço a pedido) | caneta→fotão −8..−25 ms [E] · INPUT p95 61,7 ms | régua G10 · matriz |
| 7 | memória | B42.3 · B17.9 · B17.6–17.10 | undo 4K 533,8 MB ≈ 21 traços | `mede_o_passo_de_undo_a_4k` |

## Recusas MEDIDAS que não se reconstroem

- `planos_de_tamanho` (crescer os planos por páginas zeradas): 62,2 × 61,7 — sem ganho (REGRESSOES 11/11).
- `write_texture` em paralelo: o wgpu-core 30 copia com `pending_writes.lock()` preso.
- SilMemo para o Flat 240: 1,072 → 0,995 ms (B14.3). Paralelizar o pouso do Drag: é Gauss-Seidel.
- `omp critical` à libmypaint na mistura (B28.3). Fifo + latência 1 a 4K: 30 q/s (B21.1).
- `clear()` + resize nos pools: 17,6 → 47,5 ms (RC-10, B19.12). Quantizar `wall_dt` (C9). `pre_present_notify` (B23.6).
- Handler da caneta que só enfileira: o custo migrou, total igual (B27.1).

## Estado

(cada onda acrescenta aqui o commit, a tabela A×B e o que ficou aberto, com o porquê)

**Onda 1 (12/11, feita):** `d9bd722622` envio sem cópia (Filbert Ctrl+Z 63,6 → 50,1 · Wet toque 61,8 → 50,6) ·
`b5f4a90128` `62bb7c2efa` `3848f195aa` curas do Wet (o preparo vale 15 ms no 1.º toque, mesmo binário; controlo
vermelho) · portão `the_shell_folds_the_window_the_tool_reports_dirty` reancorado (3 mutações). Tabelas r12/r13/p1 em
`medicoes/REGRESSOES_TOQUE_E_UNDO_2026-11-11.md`. Aberto: Wet Ctrl+Z ~46–48 (o `poe` serial dos tiles em
`Registo::restaura`, `reconta_teto`/`reconta_real`, `FotoDoMotor::de/repoe`); Filbert toque ~62.

**Onda 2 (em curso):** p1 mostrou que o W9 (1 340 ms) e o D1 (Color mixing largo) já não passam de 60 ms; o que resta
é o W5 (Flat Bristle `]`×10 = Size 240): um quadro de **1 055 ms** com o fio da UI a 0 % (espera o depósito no pool).
`c572c293e2`: mistura espectral com memo + `ph2d-pigment` num pedaço no smoke + pow da opacidade fora do laço =
662 → 590 G instruções por ronda do `mede_o_dab … Flat Bristle 240` (−10,9 %), hash `fe1a4c3dc2e7193a` igual.
Em voo: agente β (vetorizar a mistura ao bit, worktree `.ph2d-ag-beta`), agente α (cortes ao bit no caminho das
sub-amostras, worktree `.ph2d-ag-alfa`). Por decidir: tirar o depósito do quadro (a UI deixa de congelar; a tinta
aparece com atraso) — arquitectura do `offthread`.

**Onda 2, 02:10:** β → `ac4730824c` `61c3ceb0bc` (a mistura em SSE, ao bit; oráculo verbatim + prova exaustiva dos
2³² bits do log2/exp2): −10 % · α → `f67aadf902`…`7babb40720` (6 cortes ao bit no caminho das sub-amostras): −18,5 % ·
`065e4300a9` a miniatura viva num pool próprio de ¼ dos núcleos (W6: CPU parado 451 → 269 %, traço p95 29 → 16,
mesmo binário). Alarmes de pincel = a lista de sempre (largura: Oil_Wash, Smear_Wet; miniatura: os 74).
`ea81d00445`: o vermelho herdado era o teste a pedir Medium 0,5 depois do degrau Water · Oil (`03df88ea8c`).
Em voo (worktrees `.ph2d-ag-<x>`, base `7babb40720`): α2 (os próximos cortes exatos das cerdas), ε (Ctrl+Z do Wet),
δ (camadas: B39.A3, B19.10, B18.1–18.3), γ (água com Óleo Real: B11.3, B13.5–13.7).
**02:55 — na linha:** α2 `9ce5682066` `e9233800b9` `2d207f37c6` (corte por linha do motor + miolo + `CerdasDoDab`;
Flat 240 427,5 → 388,5 G; acumulado 662 → ~388 G, −41 %, hash `fe1a4c3dc2e7193a`) · δ `e5ebe3f068` (current_layers
devolve o Arc) `d648f295f9` (cópias de plano em par_clone) · ε `22010692a6` (restauro do histórico do Wet no pool,
ADR-0204: desfazer do motor 28,9 → 22,5 ms na sonda) · γ `c7c3e2a9c2` `3602c2b884` (laços do Óleo Real fora do fio
do quadro: 325 → 141 G instruções nele) · δ2 `d872d3811c` `3230d3ba5f` (trocar de camada sem recompor quando a figura
é provada igual: o quadro seguinte 47 → 0 ms; a troca 12 → 9 ms). Em voo: ε2 (fases do Ctrl+Z do Wet no app, trocar
tiles em vez de copiar 2×), bissecção dos 5 vermelhos herdados (verdes no main, vermelhos em `4381ee6d4a`:
smooth_edges_no_zero…, arrastar_um_numero_do_oleo…, o_modo_blender_casa…, o_oleo_pouco_diluido…,
a_subsuperficie_do_pincel…), verificação de 3 vermelhos novos vistos pelo δ2 (trocar_um_knob_de_render…, os dois do
formato do documento da macro). `a_divida_cheia_acaba_com_a_agua_a_correr`: flake de carga (verde 2× no HEAD a load 175).
Por fazer no fecho: matriz calma única base `4381ee6d4a` × HEAD (Wet/Filbert/Impasto 4K, W5, W6, W9, D1, trocar de
camada), alarmes, fmt/clippy das crates tocadas, tectos, handoff, §5.1, binário smoke 2×, limpar as worktrees
`.ph2d-ag-*`, `.ph2d-bis-4k`, `.ph2d-perf-target` e os targets.

**06:10 — FECHO da janela:** + ε2/ε3 (o histórico TROCA os tiles, densos e esparsos), α3b (5 cortes), os 7 vermelhos
herdados curados (B33.1, B32.1, B14.2/C3, formato da macro v3), os 4 tectos cortados, a bateria de fecho verde. Matriz rf
(base × HEAD, 72 corridas): Wet toque 62,9 → 50,7 · Wet Ctrl+Z 49,9 → 34,0 · Filbert Ctrl+Z 61,9 → 48,1 · Óleo Real toque
68,6 → 55,6 e Ctrl+Z 64,7 → 46,8 · W5 toque 44,3 → 31,0 e Ctrl+Z 88,4 → 47,9 · W6 CPU parado 468 → 160 % · W9 toque 55,9 →
30,7. Handoff: [DESEMPENHO_TOTAL](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md)
(§8 = o que fica aberto, por ordem).

**Achado transversal (não é do Painter):** parado, cada quadro gasta ~38 M ciclos no fio da UI — estados dos widgets
e a11y em BTreeMap (~12 %), o modelo do Quadro (`BoardId`, 4,4 %) numa cena de pintura, `format` por quadro. Vai ao
handoff para a linha da UI/Editor.

**Achados a seguir:** W6 parado depois de `]`×10 com CPU 492 % (pool, sem entrada; perfilar com binário de
símbolos) · vermelho HERDADO `o_meio_viaja_do_controlo_ao_deposito` (ph2d-wet-paint): verde no `main` `bc4a6ce930`,
vermelho em `4381ee6d4a` — bissecção em curso.

**Ferramentas desta onda:** binários com símbolos em `CARGO_TARGET_DIR=/home/enio/Documentos/Projetos/.ph2d-perf-target`
com `CARGO_PROFILE_SMOKE_STRIP=none CARGO_PROFILE_SMOKE_DEBUG=line-tables-only` (o perf não grava pilha aqui: paranoid
2; usar `perf annotate -l` para a linha). Matriz por bash (zsh não parte `"a b"`): roteiros no scratch da janela.

**Rodada n.º 10 (14/11, base `main` `36a312482a`, 109 commits; ordem do dono: atacar todo o plano, integrar e fazer ship no fim, smoke só depois):**
agentes por família, estado em `medicoes/N10_*.md` (a matriz final: [`N10_MATRIZ_FINAL.md`](medicoes/N10_MATRIZ_FINAL.md), base em
[`N10_BASE_MATRIZ.md`](medicoes/N10_BASE_MATRIZ.md)); BACKLOG dobrado nos ids tocados. Fechado: **depósito do dab fora do evento**
(Flat Bristle 240 4K, traço máx 629/538 → 68/91 ms; composição no quadro 72–85 → 10–35) e pen-up adiado ligado · **Ctrl+Z e troca do
Wet** (fluxo no pool 5,4–8,4 → 2,7–3,0 ms; troca_os_planos 23,0 → 16,1; Ctrl+Z da tela e do relevo do Impasto no lugar, 0 cópias) ·
**memória 4K** (1.º traço Impasto 100 407 872 → 1 528 256 B = 1024²; figura de aguada 58 060 800 → 131 871 B/quadro, quadro Drag Dot
1,08 → 0,15 ms; 13 dos 15 `None` do fork viram caixa; Digital até 1024) · **água** (difusão Jacobi paralela 33 × 104 ms; secagem
por tile; freio na grade fina; cedência sem Max velocity) · **camadas/placa** (Repeat Image 8 ids → 1; premul opaca 10,05 → 2,08 ms;
anel de subida 2,68 → 1,28 ms CPU; chip do meio e troca de ferramenta aquecem o fundo; Compositor com Smooth Edges: on_tick 50,0 →
0,02 ms) · **UI/quadro** (Mailbox ritmado, parado 48–70 % → 3–4 % de um núcleo a pedido; Moves em lote) · **build/medição** (libm 23 →
0, hugetlb: minflt 2 955 k → 65 k, hash dos pincéis no CI, 93 cabeçalhos `--release` → 0, sondas dizem perfil e raio). Recusas medidas:
reuso do acumulador (−12–17 ms), pool próprio da compressão (23,9 × 18,7), cache f32 e views em cache da placa, FilmLut paralela
(1,09×), `cos` por célula (−0,1 %), memo da mistura (+0,3 %), cura 4 `de_e_repoe`. Aberto: B23.6 (weston), B44.11–13, B08.6, B26.1–3,
B25.8 (cura), B17.6 (2 casos), B18.3, soltar +1 s do Flat Bristle 240, B13.3/B14.2/B30.5 (dono). Handoff:
[N10_DESEMPENHO](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-14_N10_DESEMPENHO.md).
