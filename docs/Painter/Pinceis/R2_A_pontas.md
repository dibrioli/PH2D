# R2-A — as pontas de Charcoal, Wax Scrape e Square Hard (31/10)

Régua (`tamanhos` A, recto de 40 em y 190 e de 200 em y 440; o eixo ±35 % da largura ÷ o corpo; `ini`/`fim` = a
fracção do corpo nas faixas ½–¼ · ¼–0,1 · 0,1–0 da largura a partir da borda; canto = a altura a 0,08 W da borda ÷ a do
corpo). Sonda por nome no processo de teste (a bancada inteira é pesada: fila).

## O que mudou (commit `2620df8f6d`)

1. **Entrada recta pelo EIXO** (`wetpaint/esbate.rs`): com `pontas_rectas` a entrada esbate por `u/R` e não pela corda
   redonda `u/√(R²−v²)`, que arredondava a esquina. Só muda pincéis com rectas + esbate (Square Hard, **Marker Chisel**).
2. **As pontas que CARREGAM** — `pontas_esbate < 0`: a MESMA lei `1 − e·((1−a)/2)³` a subir, e a janela do rasto
   POUSA ao soltar (`ciclo.rs`, `land_direct_remainders`) em vez de cair. Faixa `−4..8`
   (`ids::PAINTER_BRUSH_PONTAS_ESBATE_MIN`): setter, JSON (o negativo agora viaja; antes `≤ 0` não se gravava) e o
   campo «Ends fade» do cartão Shape. `e ≥ 0`: ao bit (a pousada só corre com `e < 0`).
3. Receitas (`familias/pastel_carvao.rs`): Charcoal `pontas_esbate −1`, Wax Scrape `−2,5`.

| pincel | medida | antes | depois | Rebelle |
|---|---|---|---|---|
| Charcoal | fim 200 | 0,21·0,07·0,03 | **0,32·0,15·0,08** | 0,31·0,18·0,06 |
| Charcoal | fim 40 | 0,38·0,18·0,08 | 0,52·0,29·0,15 | 0,37·0,33·0,18 |
| Wax Scrape | fim 200 | 0,64·0,28·0,09 | **0,82·0,55·0,32** | 0,84·0,63·0,48 |
| Wax Scrape | fim 40 | 0,74·0,29·0,07 | 0,83·0,39·0,11 | 0,67·0,39·0,28 |
| Square Hard | canto da entrada 40 · 200 | 0,59 · 0,70 | **0,95 · 0,95** | 0,62 · (cortado pela tela) |

O miolo não muda (desvio das colunas do recto de 200: Charcoal 0,0238 → 0,0242, Wax 0,0496 = 0,0496; o gate pede a
tinta do miolo ± 3 %: 1,0014).

Gates (wetpaint/esbate_tests.rs): `as_pontas_carregam_com_o_esbate_negativo` (fim 0,503 → 0,680 a 0 R) ·
`a_entrada_das_pontas_rectas_tem_a_esquina_cheia` (altura a meia escuridão do eixo da coluna a 0,3 R: 106/114 com a
lei, 102/114 sem — M1 `eixo: false` VERMELHO). ⚠️ M2 (sem a pousada) SOBREVIVE no gate de 40 px: o peso negativo
sozinho já passa; a pousada mede-se na sonda (Charcoal 200 0,21 → 0,27 só com ela).

Pranchas (antes | depois | Rebelle; em baixo: entrada e fim do 200, fim e entrada do 40):
`pranchas/r2_charcoal.png`, `pranchas/r2_wax_scrape.png`, `pranchas/r2_square_hard.png`.

## Por fazer (fila)

- **Regerar** Charcoal, Wax Scrape, Square Hard e Marker Chisel (na `fila-pesada`): até lá `a_miniatura_de_fabrica…` e
  `cada_preset_de_fabrica_e_a_sua_receita` acusam os quatro. O Marker Chisel é do R2-C: a esquina dele também passa a
  recta — o R2-C/coordenador confirma na foto.
- A bancada (`exame.sh`) dos três, com a régua congelada: não corrida (o hook manda para a fila).
- `a_borda_da_mancha_molhada_esbate` (aguarela, Wet Liner) reprovou numa corrida por filtro `esbate` a meio do
  trabalho dos irmãos; o Wet Liner tem `pontas_esbate 0` (nada desta lei corre) — conferir na integração.
- Square Hard GRÃO (pontinhos brancos grossos × o grão fino e escuro dele, que adensa com o tamanho): já teve as duas
  rondas (J5 r4, J6 r2); só com uma lei de motor da carga/grão pelo DIÂMETRO — não começada.
- Wax Scrape: as bordas LATERAIS do Rebelle são duras e esfarrapadas, as nossas uma névoa de pó (dureza 0 + pó).

## Necessidade em `spec.rs` (não tocado)

A doc de `BrushSpec::pontas_esbate` diz «Faixa `0..PONTAS_ESBATE_MAX`»: passa a `−4..8` (abaixo de 0 as pontas
carregam). O chão vive hoje em `ph2d-tool-painter::ids`; o lugar dele é ao lado de `PONTAS_ESBATE_MAX` (`PONTAS_ESBATE_MIN`).

## ⛔ Recusas MEDIDAS

| recusa | medido |
|---|---|
| só pousar a janela do rasto (sem peso) | Charcoal 200 fim 0,27·0,13·0,07, Wax 0,73·0,43·0,22: metade do caminho |
| tampa REDONDA recortada por disco (o fecho sem a metade de trás, `alem` 0,6 · 0,85 · 1) | tampa de vector, dura e limpa contra a borda de pó: ponta 1,0–1,1 do corpo, pior à vista que o Rebelle |
| Charcoal `−2` | fim 40 0,58 (Rebelle 0,37) |
| Wax Scrape `−4` | a entrada esvazia (200: 0,54 → 0,39; canto 0,64 → 0,57) |
