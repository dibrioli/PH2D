# J6t — As pontas que ESBATEM ao comprido (o traço largo)

**O defeito** (cega J6 r2 do Square Hard, Wet Liner, Wet Wash: «traço largo chapado com tampa dura»): o Rebelle
esbate o traço ao longo do comprimento nas duas pontas, e a rampa cresce com o tamanho; o nosso ia de nada a cheio em
< 15 px.

**A lei** (`ph2d-tool-painter/src/tool/paint/wetpaint/esbate.rs`; knob `BrushSpec::pontas_esbate` = `e`, cartão Shape do
Wet Paint: «Ends fade», `0..8`): a `a` raios do motor PARA DENTRO do centro da ponta o traço pousa
`alvo = 1 − e·((1 − a)/2)³` do corpo (`1 − e/8` no centro da ponta, cheio a um raio para dentro, `1 − e` na borda da
tampa). Pesa pela porta do padrão (depois do portão do papel: o depósito e o alvo do filme são lineares nela).
- INÍCIO (conhecido): cada célula pesa o `alvo` da sua posição ao longo do 1.º rumo desde o 1.º centro — exacto na soma e
  no filme. O 1.º dab (sem rumo) não pousa.
- FIM sem as pontas rectas: cada dab pesa a pegada pelo `alvo` ao longo do SEU rumo (cheio atrás). No FILME (máximo) a
  célula da ponta fica com o último dab; na SOMA pesa a parte ÍMPAR, `1 + (alvo(−û) − alvo(û))/2` (o corpo exacto para
  qualquer pegada simétrica).
- FIM com as pontas rectas (soma, metade de trás): a cauda viva é a rampa `a`; o FECHO pousa com carga
  `1 − (3e/8)(d/R)²` a `d` além do fim — a soma dá o `alvo` (pegada chata ao longo: a ponta dura do pastel).
- Posições na CORDA da pegada redonda (`√(R² − v²)`). `e = 0` = nada corre: ao bit.

**Medido** (perfil do eixo ±35 % da largura ÷ o meio, nos rectos de 40 e 200 da `tamanhos` A; `perfil.py`/`erro.py` da
bancada J6t; RMS dos quatro perfis das pontas ao Rebelle, escala 1):

| pincel | sem | variantes `e` → RMS | escolhido |
|---|---|---|---|
| Square Hard | 0,224 | 1,5 → 0,136 · **2 → 0,123** · 3 → 0,161 · 4 → 0,204 | 2 (fábrica) |
| Wet Wash (cura J6) | 0,182 | **1 → 0,096** · 1,3 → 0,135 · 2 → 0,214 | 1 (`CURAS_J6_PINCEL`) |
| Wet Liner (cura J6) | 0,084 | 0,5 → 0,086 · 1 → 0,102 · 1,6 → 0,196 | ⛔ nenhum |

Rampa 10 → 90 % (px, ini · fim) do Square Hard: Rebelle 23 · 21 (40 px), 123 · 64 (200 px); o nosso antes 0 · 0 e 0 · 0;
J6t (escala 0,8) 10 · 8 e 46 · 44. Wash a 200 px, o centro da ponta do fim: Rebelle 0,84, antes 0,98, J6t 0,79.

**Exame** (biblioteca fixa J6): Square Hard régua 428 → 430/454, vizinho Oil Pastel 0,764; Wet Liner 429 → 426, vizinho
Soft Pastel 0,858; Wet Wash 384 = 384, vizinho Wet Round 1,003. Pranchas `pranchas/J6t_<nome>{,_cega}.jpg` + `_chave.txt`.

**Gates**: `as_pontas_esbatem_ao_comprido_e_o_miolo_pousa_a_mesma_tinta` (rectas, filme, redondo; e 1,6 e 4: a ponta cai
≥ 0,06/0,14 no eixo, 1,5 raio para dentro igual, miolo ± 3 %) · `as_contas_do_esbate_fecham` · censos do painel
(`mudar_o_painel…`, `um_controlo_do_pincel_nao_recompoe`, `DEPOSITO_MOLHADO`). **Mutações** (todas VERMELHAS): M1 alvo 1 ·
M2 sem a entrada · M3 fecho a carga cheia · M4 a soma com o perfil inteiro.

## ⛔ Recusas MEDIDAS
| recusa | medido |
|---|---|
| a soma pelo perfil inteiro renormalizado `2 ÷ ∫alvo` | supõe a pegada chata: a redonda macia pousava 1,14× no miolo |
| `e` acima de 2 no Square Hard | o corte da entrada esvazia (0,12 a −40 px a 200, Rebelle 0,63) |
| esbate no Wet Liner | a 40 px o Rebelle Liner é cheio até à tampa (0,93): o nosso esvazia-a (RMS 40 0,06 → 0,15) |

## Ainda se vê (para a cega seguinte)
- Square Hard a 200 px: o Rebelle sobe devagar até ~0,6 D para dentro da entrada (0,76 no 1.º centro, 0,87 a 1 R); o
  nosso enche a um raio — e a esquina da entrada sai arredondada (a corda redonda; com as pontas rectas a entrada
  devia medir no eixo).
- Wet Wash: a orla escura contorna a tampa (o Rebelle esfuma sem contorno) — é a orla, não a rampa.
