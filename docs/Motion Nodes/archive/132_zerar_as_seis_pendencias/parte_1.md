# ARQUIVO — 132_zerar_as_seis_pendencias.md (história, 70 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`132_zerar_as_seis_pendencias.md`](../../132_zerar_as_seis_pendencias.md) em 2026-10-13, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-215,286-486` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### P1 — o degrau de FORA: a causa é o custo POR PIXEL do borrão

**Decomposição** (`relogio_c10_decomposicao`, 4 096 discos de 12 px a 1600×972, ms de placa por quadro; o
relógio por PASSE da pilha é novo — `fx.*`):

| pilha | A · L | célula · saída | pág. | Mpx | placa | pilha (H · V · resolve · ingest) | formas | volta | colagem |
|---|---|---|---:|---:|---:|---|---:|---:|---:|
| Inner Shadow (dentro) | 0 · 23 | 38 · 14 | 2 | 6,0 | 2,15 | 1,39 (0,64 · 0,55 · 0,07 · 0,07) | 0,30 | 0,06 | 0,08 |
| Outer Glow σ 5,3 | 16 · 17 | 48 · 46 | 3 | 9,5 | 3,60 | 2,29 (1,03 · 0,87 · 0,16 · 0,11) | 0,54 | 0,19 | 0,14 |
| Drop Shadow 6/6 (fora) | 16–22 · 23 | 60 · 52 | 4 | 14,8 | 4,88 | 3,45 (1,60 · 1,36 · 0,17 · 0,17) | 0,49 | 0,24 | 0,16 |
| camada inteira (qualquer) | — | — | 1 | 1,56 | 0,37 | (0,17 · 0,14 · 0,02 · 0,02) | | | |

A pilha é linear na área (`0,24 ms/Mpx`, as células e a camada inteira igual), a área já é ≈ Σ das saídas (Glow:
`46²` de `48²`), a colagem é 4 % ⇒ hipótese 1: o custo por pixel, e ele é o BORRÃO (H + V = 83 % da pilha).

**A cura:** o borrão por GRUPO (`ph2d-render/src/fx_stack_borrao_grupo.rs`) — 64 pixels de uma linha (coluna)
carregam UMA vez os `64 + 2h` taps e os `2h + 1` pesos na memória do grupo; o de sempre fazia, por pixel,
`2h + 1` leituras E `2h + 1` exponenciais. Os mesmos taps, pesos e ordem: o MESMO byte em 8 pilhas
(`o_borrao_por_grupo_da_o_mesmo_byte`, `2cfacfe11f`), e `the_c10_variants_give_the_same_pixels` (com as pilhas de
fora) igual ao byte.

**Régua intercalada** (`relogio_c10_intercalado`, placa por quadro, o MÍNIMO de 7 rodadas; `load 9,1 → 7,9`):

| discos | Inner Shadow: produto → grupo | Outer Glow | Drop Shadow (fora) |
|---:|---:|---:|---:|
| 256 | 0,38 → 0,36 | 0,54 → 0,50 | 0,61 → 0,55 |
| 1 024 | 0,69 → 0,60 | 1,02 → 0,88 | 1,31 → 1,07 |
| 2 048 | 1,08 → 0,90 | 1,95 → 1,71 | 2,47 → 2,01 |
| 4 096 | 2,18 → 1,83 | 3,57 → 2,96 | 4,92 → 3,93 |
| 8 192 | 3,98 → 3,28 | 6,81 → 5,65 | 9,46 → 7,70 |

A camada inteira (1600×972): `0,37 → 0,28 ms`. **O borrão por grupo é o único** (`4609c19ec6`).

| ⛔ Recusas MEDIDAS | porquê |
|---|---|
| o borrão por PIXEL (`blur_h_at` + `cs_op_v` de sempre) | perde nas 15 linhas da régua (o grupo tira até `19 %` a 8 192) e dá o mesmo byte |
| colar/desenhar só a caixa da saída | a saída já é `92 %` da célula (Glow `46²`/`48²`); a colagem é `4 %` da placa |
| o borrão na pirâmide abaixo de `MAX_HALF/3` | não construído: o borrão por grupo ganha com paridade AO BYTE; a pirâmide paga a caixa+tenda contra o oráculo Φ |

**A vala POR LADO** (fechada na mesma sessão, `plano::Vala`): `leitura` passa a ser `[l, t, r, b]`
(`ph2d_render::stack_read_por_lado`: o pixel lê em volta de `p − off`, o lado para onde o deslocamento aponta
paga-o, o oposto desconta-o) e a vala entre vizinhas em `x` é `max(A_r + L_r, A_l + L_l, A_l + A_r) + 1`
(idem em `y`). Gates: `the_tight_trench_keeps_every_read_off_the_neighbour` (por lado, e as saídas disjuntas)
e `the_c10_variants_give_the_same_pixels` com as pilhas de fora + a Drop Shadow LONGA `14/0` — tudo na faixa da
referência, `0` texels da silhueta.

| pilha (A · L por lado) | célula de um disco de 14 px | 4 096 discos: simétrica → por lado | 8 192 |
|---|---|---:|---:|
| Inner Shadow 6/6 (0 · 23/23/11/11) | 38×38 = 38×38 | 1,79 → 1,80 | 3,35 → 3,36 |
| Outer Glow (16 · 17) — o controlo | 48×48 = 48×48 | 3,13 → 3,07 | 6,01 → 5,99 |
| Drop Shadow 6/6 (16/16/22/22 · 23/23/11/11) | 60×60 → 54×54 | 4,18 → 3,35 (4 → 3 pág.) | 7,93 → 6,67 |
| Drop Shadow longa 14/0 (16/16/30/16 · 31/17/3/17) | 76×76 → 62×48 | 6,53 → 3,38 (7 → 3 pág.) | 12,48 → 6,75 |

(`relogio_c10_intercalado`, as variantes no mesmo processo, 7 rodadas com a ordem rodada, o MÍNIMO da placa em
ms; `load 27,8 → 24,1`.) Ganha onde há deslocamento e empata onde não há ⇒ **a vala por lado é a lei**; a
simétrica sai (não há variante: a `dupla` do W3b continua como a régua de referência dos gates).

**A `=138` no PRODUTO** (app `--profile smoke`, 1600×944, `PH2D_FLUID_PROFILE=1`): a sonda dizia 3 páginas e o app
fazia 10–11 (`fx_cells.pilha` 11 ms ×11). Mecanismo, medido no app: `caixas 8192 · vivas 8138` — cada disco pagava
DUAS células, a dele e a da sprite CALADA que a placa põe por cada linha de forma (a cura é a do `001077631a`, a
linha calada não é célula). Régua no caminho do produto (a variante trocada a cada 120 quadros no MESMO processo,
15 janelas, o mínimo; `load 26 → 68`): placa `14,05 → 9,34 ms`, pilha `9,93 → 6,07 ms`, `10 → 7` páginas. O que
SOBRA é o custo verdadeiro da cena: discos de ~18 px com o Glow da cena (raio `0,1` ⇒ σ `9,05` px: alcance 28,
leitura 29, célula ~78 px) — 4 096 × ~6 kpx = 25 Mpx, `0,24 ms/Mpx`; fora do viewport só ~30 discos. ⇒ a sonda de
12 px / σ 5,3 não era a cena. Gate `a_138_paga_as_paginas_dos_discos` (4 096 discos de 18 px + 4 096 caladas da
placa, o Glow da cena): 7 páginas e 4 096 células = só os discos; o controlo sem a regra dá **11 páginas e 8 192
células** (o número do app). ⚠️ E o `render.vello` de 15 ms não eram discos no Vello: o span abria ANTES da camada
filtrada e somava o filtro inteiro (=137 sem filtro: `0,68 ms`); fechado só sobre o render, a `=138` mostra
`render.vello 0,30 ms` · `fx_cells.pilha 6,14 ms (×7)` · `gpu-busy 9,7 ms`. O roteiro da `=138` diz estes números.

