# R2-B — o óleo que derrete o grão fino (31/10)

Sonda: `sonda_r2b_grao_do_oleo` (`tests/it/familias/oleo_acrilica.rs`, ignorada; ~13 s). Pinta a cena `tamanhos`
(foto A) por pincel e caso (`Desligado`, `Sempre`, `SemLuz` = pousa com o óleo e fotografa sem a óptica, `Sombra0`/`Prof0`/
`Fenda0`/`Ceu0`/`Brilho0`) e variantes de receita (`PH2D_SONDA_VARIANTES`). Mede no recto de 200 px (y 355..530,
x 150..850): a mediana da escuridão, os furos (< 35 % da mediana) e o grão (|px − caixa 3×3| ×1000). Ganchos neutros na
bancada: `SEM_LUZ_NA_FOTO`, `AJUSTA_OLEO` (por omissão a foto de sempre).

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

Ronda 1 (simulada no motor): alfa preso à crua (`borda.min(1)`), altura enchida. Stain volta ao grão fino (prancha);
gate novo verde; **mas** `a_orla_rasgada_nao_tem_pontinhos_escuros` reprova (Estúdio: 5 pontinhos) e os dois
`o_smooth_edges_*` comparam com a lei de antes. Ronda 2: só no GRÃO (≥ 3 ralas com pigmento nas 3×3): pontinhos 1 e o
grão sintético reprova. Duas rondas ⇒ PAREI; o motor ficou como estava (nada comitado nele).

Gate vermelho registado (ignorado, com o número): `o_grao_ralo_nao_engorda_no_oleo` (+0,85 de cobertura sobre a crua
nas células ralas). Medida: `medida_r2b_crua_dos_entalhes` — na orla do Dabber, 171 das 492 células que o entalhe enche
têm crua < 0,1 (o limiar pela crua não separa orla de grão).

**Necessidade (para o coordenador):** separar o entalhe da ORLA RASGADA (o alfa enche, contra os pontinhos) do do GRÃO
(o alfa fica na crua). Pista não medida: na orla a célula rala tem papel nu nas 3×3; no grão sintético não. Ficheiros
`oleo_borda.rs`/`oleo_campo.rs` (livres). Depois: regerar Oil/* (as miniaturas mudam) pela fila pesada.

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
Não há receita de «Oil Grunge» em `oleo_acrilica.rs` (nem em `EM_ESTUDO_W2`); não comecei.

## Pranchas (Real Oil ligado; antes | depois simulado | Rebelle, ×1 e ×2)
`pranchas/r2_knife_stain.png` · `pranchas/r2_knife_grainy.png`.

## Fila pesada
A bancada não corre em subagente (hook): pus na fila o exame de Stain e Grainy para quando a lei entrar.
