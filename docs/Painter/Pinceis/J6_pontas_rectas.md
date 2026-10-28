# J6 — As PONTAS RECTAS do traço (lacuna de motor 2 do `01_CATALOGO.md` §3d)

**O defeito** (revisões cegas J4–J5 de Square Hard, Soft Pastel, Charcoal, Wax Scrape): o Rebelle começa e acaba o
traço cortado a direito, perpendicular ao rumo, e CHEIO até perto do corte; a nossa cadeia de dabs redondos acaba na
meia-lua do último e esmaece num raio. A `dab_quina` não serve (é a quina de CADA dab: endurece a borda lateral —
recusa medida em `exames/pastel_carvao.md`).

**A lei** (`ph2d-tool-painter/src/tool/paint/wetpaint/tampa.rs`; knobs `BrushSpec::pontas_rectas` + `pontas_alem`,
cartão Shape do Wet Paint: «Flat ends» · «Ends beyond»):
- cada dab pousa só a METADE DE TRÁS da pegada (semiplano atrás do centro, pelo rumo dele) × 2 pela porta do padrão
  (depois do portão do papel, onde a soma é linear) — o meio pousa a mesma tinta; o filme (máximo) não compensa;
- o INÍCIO corta no semiplano à frente de `c₀ − alem·r₀`; o 1.º dab (sem rumo) não pousa; com `alem > 0` a ABERTURA
  pousa os dabs que haveria antes de `c₀` logo que a lane conhece o rumo;
- o FIM: ao soltar, o FECHO continua o traço a direito além do fim, cortado em `c_N + alem·r_N`, e a janela do rasto
  que sobra POUSA (`Engine::land_direct_remainders`, porta nova; antes caía e o fim esmaecia);
- `alem` é em meias-larguras PROMETIDAS (o número do painel), convertido ao raio do motor (`largura.rs`).
- Sem a caixa: nada muda, ao bit (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`, `cada_preset_de_fabrica_e_a_sua_receita` verdes).

**A régua** (o recto de 40 px; `tampa_tests::pontas`, e `rampa.py`/`curva` sobre as fotos da bateria):

| | curva ini · fim | rampa 10→90 % ini · fim (px) |
|---|---|---|
| nosso redondo (gate, pincel de omissão) | 0,40 · 0,25 | 56 · 52 |
| nosso com as pontas rectas | 0,03 · 0,00 | 12 · 0 |
| Rebelle Square Hard · Dry Thin · Wax Scrape · Charcoal | 0,03·0,06 · 0,05·−0,10 · 0,28·0,33 · 0,50·0,30 | 11·35 · 15·13 · 21·25 · 24·29 |

O corte do Rebelle passa o 1.º ponto (ini, px a 40): Square Hard 12 · Dry Thin 3 · Wax Scrape 15 · Charcoal 2.

**Gates**: `as_pontas_rectas_cortam_o_traco_a_direito_e_pousam_a_mesma_tinta` (curva ≤ 0,08; rampa ≤ 0,35 da do
redondo; tinta do miolo ± 3 %; `alem` 0,5 = 20 px além, cheio) · `the_flat_ends_rows_land_on_the_tool` · censos
`mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` e `um_controlo_do_pincel_nao_recompoe_a_tela`. **Mutações** (todas
VERMELHAS): M1 sem o corte da frente (curva 0,18·0,27, tinta 1,73×) · M2 `COMPENSA` 1 (tinta 0,53×) · M3 sem o fecho
(curva do fim 0,16) · M4 `alem` no raio do motor (33 px em vez de 20).

## ⛔ Recusas MEDIDAS
| recusa | medido |
|---|---|
| só a metade de trás, sem o fecho | o fim esmaece num raio atrás do último centro (curva 0,16) |
| `alem` em raios do motor | o redondo de 40 px tem raio de motor 66,9: o além saía 33 px em vez de 20 |

Não medido, só desenho (não reconstruir sem medir): atrasar o depósito um raio para ver o fim antes (a tinta seguiria
o cursor um raio atrás) e fechar com UM dab de peso grande (uma amostra do padrão em vez de muitas).
