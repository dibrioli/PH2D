# R2-B — o óleo que derrete o grão fino (31/10)

Sonda `sonda_r2b_grao_do_oleo` (`familias/oleo_acrilica.rs`, ignorada, ~13 s): cena `tamanhos` A por caso (`Desligado`, `Sempre`, `SemLuz`, knobs a 0) e variante de receita; no recto de 200 px a mediana da escuridão, os furos e o grão (|px − caixa 3×3| ×1000). Ganchos neutros: `SEM_LUZ_NA_FOTO`, `AJUSTA_OLEO`.

## Knife Stain — o grão morre NA ÓPTICA, no entalhe
| caso | mediana | furos | grão |
|---|---|---|---|
| Real Oil desligado | 0,269 | 0,438 | 186,9 |
| pousa com óleo, foto sem óptica | 0,269 | 0,438 | 186,9 (= o depósito é o mesmo) |
| Real Oil ligado (hoje) | **0,610** | 0,382 | **119,0** |
| ligado, profundidade 0 / sombra 0 / brilho 0 / céu 0 / fenda 0 | 0,60–0,62 | ≈0,39 | 119–127 (a luz não é a causa) |
| **simulado: o entalhe sem alfa** | 0,263 | 0,424 | 177,5 |
| Rebelle | 0,086 | 0,471 | 202,3 |

Mecanismo: `sem_entalhes` (`oleo_borda.rs`) enche a célula rala até à mediana das 3×3 em ALTURA e ALFA. No grão do
Stain cada célula rala entre grãos cheios é um «entalhe»: o alfa enchido pinta de azul o papel entre os grãos
(13,8 % dos px que eram papel ficam com tinta). A cura J6 (`borda_suave`) não tocava isto.

Ronda 1: alfa preso à crua, altura enchida — Stain volta ao grão (prancha), mas `a_orla_rasgada_nao_tem_pontinhos_escuros`
reprova (Estúdio 5). Ronda 2 (≥ 3 ralas nas 3×3): pontinhos 1, o grão reprova. Motor intacto.

Gate vermelho registado (ignorado, com o número): `o_grao_ralo_nao_engorda_no_oleo` (+0,85 de cobertura sobre a crua
nas células ralas). `medida_r2b_crua_dos_entalhes`: na orla do Dabber 171/492 células enchidas têm crua < 0,1.

**Necessidade (para o coordenador):** separar o entalhe da ORLA RASGADA (o alfa enche, contra os pontinhos) do do GRÃO
(o alfa fica na crua). Depois: regerar Oil/* (as miniaturas mudam) pela fila pesada.

**Ronda pedida pelo coordenador (31/10, «borda × grão»), 4 critérios no MESMO processo** (seletor de teste num
`GRAO_CRITERIO`, revertido): o entalhe de GRÃO guarda o alfa cru, o de BORDA enche como hoje.
| critério de GRÃO | grão ralo (≤ 0,01) | pontinhos Clássico (soltos·todos) | Estúdio |
|---|---|---|---|
| 0 = lei de hoje | +0,845 | 0 · 0 | 0 · 0 |
| 1 = sem papel nu nas 3×3 | +0,0005 | 0 · 0 | 0 · **5** |
| 2 = sem papel nu nas 5×5 (≤ 2 da borda) | +0,0005 | 0 · 0 | 0 · **5** |
| 3 = ≤ 2 papéis nas 5×5 | +0,0005 | 0 · 0 | 0 · **5** |
| 4 = nenhuma vizinha 3×3 < 0,02 | +0,0005 | 0 · 0 | 0 · **5** |
Os 5 pontinhos do Estúdio são os MESMOS nos quatro: células ralas do INTERIOR da orla do Dabber (sem papel por perto),
que com o alfa cru ficam escuras — a distância ao papel não separa o pontinho do grão. Nenhuma variante passou ⇒
revertido. **Última tentativa (critério 1 + papel sem sombra):** com a sombra a 0 os 5 somem (são sombra); tirar a sombra do papel
na célula de grão deixa 3, e nas 3×3 dela 2 — o resto é a sombra na própria TINTA vizinha ao furo novo ⇒ revertido.

## Knife Grainy — o grão morre NO DEPÓSITO (receita), não no óleo
| caso | mediana | furos | grão |
|---|---|---|---|
| desligado = sem óptica | 0,660 | 0,009 | 16,7 |
| ligado (hoje) | 0,646 | 0,001 | 8,9 |
| ligado + entalhe sem alfa | 0,645 | 0,005 | 17,9 |
| gate 2,0 | 0,626 | 0,080 | 94,5 (pontilhado de papel de 1 px) |
| gate 2 + Clouds 60 (c 1, b 0,42) | 0,306 | 0,294 | 116,7 (furos, um Stain) |
| Rebelle | 0,449 | 0,101 | 85,6 |

paperContrast 2→4 não muda nada. O Rebelle é TONAL (flocos claros/escuros dentro da tinta, a luz num relevo granulado),
não furos até ao papel: nenhuma das 4+4 variantes o dá ⇒ curado-não-aprovado, receita intacta. Fila: o grão de
profundidade por dab (`ba68f2166f`) com Clouds largo, medido pela mesma sonda.

## Grunge
Sem receita em `oleo_acrilica.rs`; não comecei.

## Pranchas (Real Oil ligado; antes | depois simulado | Rebelle, ×1 e ×2)
`pranchas/r2_knife_stain.png` · `pranchas/r2_knife_grainy.png`.

Fila pesada: o exame do Stain, para quando a lei entrar.
