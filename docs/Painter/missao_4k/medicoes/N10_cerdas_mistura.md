# N10 — famílias 2 e 3: cerdas largas e mistura de cor (agente F2, 14/11)

Uma linha por id: estado · antes → depois · sha · porquê. A janela principal dobra-o no BACKLOG.

Régua desta rodada: `mede_pen_up_4k.sh matriz 3840x2160 papel <bin> "Flat Bristle:wet"` com `TECLAS=10` (Size 240),
`CURTO=1`, Xvfb `:92`, Radeon (RADV sw WSI), base `target/ph2d-base-36a312` × HEAD `3132b3da13` (+ a árvore suja dos
outros agentes, `target/n10/f2/bin-head`), intercalados base1 head1 base2 head2, load 17 → 23 (o relógio é indicativo;
a parada de centenas de ms não é ruído). Quadros crus: `target/n10/f2/res/quadros/*.csv`.

| corrida | pior intervalo (custo do quadro) ms | traço p95 | soltar | Ctrl+Z | parado CPU % |
|---|---|---|---|---|---|
| base1 | **628,3** (131,2) | 628,3 | 342,1 | 319,6 | 193 / 59 |
| base2 | **796,0** (273,4) | 796,0 | 223,1 | 50,0 | 114 / 56 |
| head1 | **1 434,8** (16,0) | 49,9 | 0 (a parada caiu no +1 s) | 52,4 | 203 / 59 |
| head2 | **583,0** (110,5) | 583,0 | 268,0 | 97,4 | 114 / 53 |

`mede_o_dab 1 preset "WetPainter/Oil/Flat/Flat Bristle" 240` (smoke, `perf stat -e instructions:u`, load 24):
**382,8 G** instruções por ronda (o plano: 662 → ~388 G na onda 2), depósito 2 441 ms por 3 dabs (≈ 0,8 s/dab a load 24),
hash **`5e697cf298bc63ee`** — ≠ `fe1a4c3dc2e7193a` do plano: este agente não mudou código; o hash moveu-se por commits de
depois do `7babb40720` que tocam o Wet (integ 13/11 `755e86cd05` tecto u16, `24ff377cb6`/`cb09e97587` papel em duas
leituras, `ac5b3bb566` B13.9) e/ou pela árvore suja do F4d (`ph2d-wet-paint` flow/solver). Nova referência de hash para
os cortes ao bit desta família: `5e697cf298bc63ee` (no MESMO binário antes × depois).

| id | estado | antes → depois | sha | porquê / o que falta |
|---|---|---|---|---|
| B14.3 / W5 | parcial (medido; sem cura nesta rodada) | quadro 1 055 ms (12/11) → hoje pior intervalo 583–1 435 ms no HEAD, 628–796 na base; o custo do quadro nesses intervalos é 16–273 ms ⇒ a UI fica PRESA FORA do quadro, no evento do ponteiro (`stamp_dabs_wetpaint` → `no_pool(stamp_dabs_wetpaint_corpo)`, `wetpaint/dab_route.rs:32`, síncrono) | este commit (medição) | a condição da decisão do plano (> 50 ms) está cumprida: o depósito tem de sair do evento. Ver «depósito fora do quadro» |
| depósito fora do quadro | aberto (desenho fixado, não implementado: não cabe em 45 min e toca ficheiros em edição por F1d/F5c) | — | — | desenho (estado da arte, Krita `KisStrokesQueue`/MyPaint): `stamp_dabs_wetpaint` passa a ENFILEIRAR `(dabs, BrushSpec, DabRng, sub_k)` num canal para o worker que JÁ possui o `Engine` (`wetpaint/offthread.rs`, o motor viaja por canal); o worker pousa na ordem de chegada (a mesma ordem determinística ⇒ o mesmo bit no documento final); o quadro compõe o que já assentou. O `LoteDoDab` empresta do `PainterTool` (imagens da ponta/grão, `copia`, `groups`, `rampa`) ⇒ passo 1 = tornar o `LoteDoDab` DONO (`Arc` das imagens, cópia do resto) sem mudar a aritmética, gate ao bit «lote dono × lote emprestado» (Flat Bristle 70 + Thick Impasto 60 × 3 fundos); passo 2 = a fila e o `drain` antes de toda leitura do motor (undo/`FotoDoMotor`, conta-gotas, óptica, `wet_compoe_ou_adia`, pen-up, troca de camada); gate «a tela com a fila = a tela síncrona ao bit» + «nenhuma leitura sem drain» (mutação: tirar o drain do undo → vermelho) |
| B28.x memo da SAÍDA da mistura espectral | ⛔ recusa medida | `mistura_no_meio` com a última (base, tinta, `fac`) → saída por fio, chave `PH2D_MISTURA_MEMO=0/1` no MESMO binário, `mede_o_dab 1 preset Flat Bristle 240`, A B B A, `perf stat -e instructions:u` (load 10–14): sem 389,14 / 389,45 G × com 390,41 / 390,28 G (**+0,3 %**), hash `5e697cf298bc63ee` nas 4 | — (revertido) | o `fac` (cobertura) muda de célula a célula no miolo do Flat 240: o memo quase nunca acerta e paga a chave de 7 palavras. Perfil (smoke + símbolos, `perf record -k CLOCK_MONOTONIC -F 997`, 29 148 amostras, sem `-g`): `Trail::tip_rows` 7,8 % · `rgb4_do_log` 7,6 % · `visibilidade_e_ceu` 4,6 % · `PontaDasCerdas::recorta_no_fio` 4,5 % · `penteia_o_relevo_do_dab` 4,4 % · `mistura_no_meio` 4,3 % · closures de `pousa_o_dab` 2–4 % cada · `rebote_no_papel` 3,2 % · `crossbeam_epoch` pin 2,8 % · `mistura_255` 2,2 % · `libm cos` 2,0 % + `rem_pio2` 1,2 % (um `cos` por célula: candidato a tirar do laço, conferir onde) — custo espalhado, nenhum corte ao bit > ~5 % à vista: a alavanca é o depósito fora do evento |
| B28.3 | feito via B28.6 | Wet blend 21,6 ms/q → 9,4–13,8 ms/q (alvo 16,7 a 300 px/q) | `c1f01b5429` (B28.6) · `814b5552df` (B28.3 parcial) | o «falta ~4 ms/q» do B28.3 foi fechado pela onda 3 (pegada e película ao bit); não remedido nesta rodada |
| W6 / W9 / D1 | feito (onda 2) | W9 soltar 1 340 → < 60 (p1) e toque 55,9 → 30,7 (rf); W6 CPU parado 468 → 160 % (`065e4300a9`); D1 < 60 (p1) | ver PLANO §Estado | não remedidos nesta rodada (a matriz foi gasta no W5, que é o aberto) |
