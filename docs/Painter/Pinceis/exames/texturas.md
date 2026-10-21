# Exame da família `texturas` (J4, catálogo K + o reexame de J-2 Spray Can, J-3 Splatter e J-5 Stipple)

Receitas e portões: `tests/it/familias/texturas.rs` (MEIOS = aprovados, `EM_ESTUDO` = reprovados) e, para J-2/J-3/J-5,
`familias/aerografo.rs` (`EM_ESTUDO`). Imagens dos carimbos: desenhadas no próprio ficheiro, procedurais (nenhuma copiada
de app nenhum). Pranchas: `pranchas/texturas/` (+ `cegas/`, `chaves/`); réguas: `exames/texturas/`. Régua do portão:
`exames/texturas_foto.py`. Custo: `exame.sh --custo … 5,40,200`, perfil smoke (load 20–30: indicativo, a J5 re-mede calmo).
Os EM_ESTUDO examinam-se pelo caminho: `familias::texturas::escreve_os_em_estudo` (ignorado) os escreve em
`$PH2D_EXAME_DIR/estudo/` sem entrarem na fábrica.

**Aprovados: Halftone (K-6) · Stars (K-5) · Bokeh (K-4).** **Curados, NÃO aprovados:** Foliage (K-2), Grass (K-3),
Noise Grit (K-8), Spray Can (J-2), Splatter (J-3), Stipple (J-5). Não tentados: K-1 Clouds e K-7 Hatching (sem oráculo
que se compare: o MyPaint `Clouds` enche a tela à escala da bateria; nenhum preset de hachura do Krita fotografado),
K-9 Grunge (Wet, outro dono).

## K-6 Halftone — APROVADO (J4 1.ª onda, sem mudança)

Digital: o grão `Dots` preso à TELA (`Tiled`, 45°, ~7,8 px), redondo 0,8 com orla 3 px, a pressão no tamanho. Krita
`y) Screentones Regular`, régua 352/452, vizinho Soft Round 2,44. Razão de uso: a retícula do mangá. Revisões r1–r7 (o
nosso tomado pelo original 6/6, baixa). Custo p95 0,17 · 0,37 · 0,96 ms. Portão por foto, M1 SANGRA.

## K-5 Stars — APROVADO, com o tecto da estrela e o rasto da pressão

- **Receita** (Digital): a estrela NOSSA, 2 por passo a 0,85·D, através 0,9·D, `jitter_scale` 0,6, partícula 0,8,
  **`particle_max_px` 56** (o Rebelle a 200 px espalha as estrelas por ~234 px mas nenhuma passa de 56: a caixa da
  imagem dele — `marcas.py` na foto: lado p50 22/máx 39 a 40 px, 56 a 200 px), **`passo_pela_pressao`** (o rasto de
  estrelinhas da pressão baixa, r4/r6). 108 px = o Rebelle.
- **Porquê o tecto da partícula e não o `size_max_px`** (decisão 8): `leis_escala_rebelle.json` não tem o Spatter/Star;
  medido na foto, o tecto é da MARCA, não do pincel — o `size_max_px` 56 encolhia também o espalhamento (variante v2,
  `v_st1_tamanhos.png`: a faixa de 200 px vira uma de 56). Com o tecto da partícula as estrelas param a 56 e a faixa fica.
- **Régua** 348/454 (era 330). Vizinho Scratch 2,99. **r7** (prancha nova): o nosso pelo original em 4/6 (tamanhos
  alta: «a 200 px a imitação vira quadrados chapados» — o defeito do Rebelle, divergência decidida); no «sobre»
  achou o Rebelle (baixa: «estrelas vermelhas maiores»). **Custo** p95 0,32 · 0,34 ms; 27 MiB.

## K-4 Bokeh — APROVADO

- **Receita** (Digital): o aro por `custom_falloff` (miolo 0,28, aro 0,95), 3 por passo, **passo 0,75 pelo diâmetro à
  pressão**, partícula 0,6, `jitter_scale` 0,35, a pressão no tamanho pela curva 1,5 e na opacidade. Krita `z) Stamp Bokeh`.
- **A lei que faltava** (r3–r6): na pressão baixa o Krita deixa um rasto DENSO de bolinhas — o passo segue o diâmetro do
  dab. Com `passo_pela_pressao` a rampa 0→1 tem o rasto (`z_bokeh_rampa.png`).
- **Revisões** r7–r14 (8 novos, cada cura a sua prancha): r7 e r9 tomaram o nosso pelo original; r8, r10–r12 acharam o
  Krita (baixa, média em r10), cada um com o seu par de diferenças — e elas CONTRADIZEM-SE de ronda para ronda (o nosso
  «mais denso» r7/r8, «mais ralo» r9/r11; discos «menores» r12, «maiores» r13; micro-pontos de mais r12, de menos
  r13/r14). **r13 e r14** (a prancha final): o nosso pelo original em 6/6, baixa. Decisão 12: não reprova.
- **Régua** 292/454. Vizinho Airbrush 2,33. Razão de uso: luzes de fundo desfocadas — discos de aro, de todos os tamanhos.
- **Custo** p95 0,13 · 0,20 · 0,44 ms; 25–27 MiB (load 25–29). **Portão** (Contraste[1] · Pontos[0] · CvArea[0], os
  irmãos); mutação B1 (o disco sem o aro, `Constant`): SANGRA; o preset volta ao byte.

## Curados, NÃO aprovados (receita da última ronda em `EM_ESTUDO`)

- **K-2 Foliage** (Krita `z) Stamp Leaves`): pela ÁREA a 5 px (a linha contínua do Krita — o pontilhado acabou: r10–r13
  «nenhuma visível» a 5 px), `particle_min_px` 3,5, 5 por passo, partícula 0,42, pressão 0,8 na opacidade (a rampa 0→1:
  Krita 0,165 → 0,341 de cobertura, a nossa 0,161 → 0,337). Régua 420/454. r11 e r13 tomaram o nosso pelo original
  (6/6); r10 e r12 acharam o Krita (6/6, baixa). Repetido em r12 e r13 (visível): **na pressão baixa o Krita é um tom
  claro liso e comprido, o nosso esfiapado** (a contagem sorteada 0,4 parte-o) e **a 200 px o Krita é ralo, de folhas
  soltas, o nosso uma massa em cachos**. Custo p95 1,7 · 8,3 · 11,6 ms (load 22).
- **K-3 Grass** (Krita `z) Stamp Grass`): os tufos redesenhados (lâminas largas 0,025–0,05, ¼ pálidas, de pé a inclinar
  à direita), a área, passo 0,75. Régua 414/454 (era ~350). r7 e r8 tomaram o nosso pelo original (6/6, baixa); **r9**
  (prancha com a área de hoje) achou o Krita 6/6 — e os três dão a MESMA diferença: **o Krita é uma fila contínua de
  tufos iguais com as pontas em gancho; o nosso moitas com buracos e lâminas rectas**. Saiu da fábrica. Custo p95
  0,40 · 0,67 · 0,97 ms.
- **K-8 Noise Grit** (Krita `y) Texture Noise`): o carimbo redesenhado — MANCHAS largas e pálidas + RABISCOS finos que
  curvam (r5: o Krita são «rabiscos curvos, grossos e esbatidos», não palha), pela área, fluxo 0,6. Régua 422/454
  (era 414). r6 achou o Krita 6/6 (média): **o Krita tem grumos escuros e densidade desigual (nós espaçados), o nosso é
  homogéneo; a 5 px o Krita é mais escuro**. Custo p95 1,0 · 1,1 · 2,1 ms.
- **J-2 Spray Can** (Krita `y) Texture Spray`): gotas (partícula 0,09, 16 por passo) + o **VÉU** (`veu` 0,2 ×1,8, tom
  0,2) + o spray **PARADO** (`parado_s` 0,0035: o Krita parado 0,235 · 0,918 · 0,996 a 10 · 60 · 200 eventos; o nosso
  ~0,30 · 0,89 · 1,00 — `varparado.sh`), dureza 0,85, fluxo 0,4. Régua 426/454 (era 430 na J3 sem véu nem parado). r1
  tomou o nosso pelo original (baixa); r2 e r3 acharam o Krita (r3 média). Repetido (visível): **a 5 px o nosso é uma
  linha escura, o Krita pó quase invisível** (a marca mínima de 0,5 px de raio concentra a tinta) e **discos do véu
  grandes e iguais soltos acima da faixa**. Custo p95 1,5 · 12,0 ms a 40 · 200 (a receita da ronda r2, load 22).
- **J-3 Splatter, J-5 Stipple**: como a 1.ª onda os deixou (o Splats é aguarela: dono `aguarela`).

## Motor J4 (o agente «motor da família texturas»)

Cada lei aditiva, desligada por omissão, gravada no preset só ligada (os presets de hoje ao byte: `volta_ao_byte`,
`a_miniatura`, `counted` verdes); portão red-first e mutação em cada uma. Commits `10cb5825b3`, `3c1d6c6da8`, `fc9738ef4d`.

| lei | onde | o que faz | portão (mutação) |
|---|---|---|---|
| `GrainLaw::area` | `texture/shape.rs` | a Shape de imagem lida pela ÁREA do píxel (grelha de ½ texel/toque, ≤ 8×8); a máscara em cache recusa-a | `area_tests` (A1, A2, U1 SANGRAM) |
| `BrushSpec::passo_pela_pressao` | `stroke/walk.rs` | o passo × o diâmetro que a pressão dá | `o_passo_segue_o_diametro_a_pressao` (P1, P2) |
| `DabJitter::particle_max_px` / `particle_min_px` | `marca.rs` | o tecto / o piso da marca do spray em px | `o_tecto_da_particula_corta_so_as_maiores` (T1, T2, Pmin, Pmin2) |
| `DabJitter::veu` (+ escala, opacidade) | `stroke/spray.rs` | uma 2.ª população das marcas (fluxo próprio) | `o_veu_e_uma_segunda_populacao_maior_e_palida` (V1, V2) |
| `BrushSpec::parado_s` | `stroke/spray.rs` | o Space parado pousa uma nuvem a cada `parado_s` | `o_spray_parado_pousa_pelo_tempo` (Pa1, Pa2) |
| `DabJitter::po` (+ alcance, px, opacidade) | `stroke/spray.rs` | o PÓ SOLTO fora da borda (pedido do `pastel_carvao`) | `o_po_solto_espalha_graos_a_volta_do_traco` (PO1–PO3) |

- **O custo da área** (medido): com um toque por texel (grelha ≤ 16) o Foliage a 40 px dava p95 33 ms (load 26); com
  ½ texel por toque e ≤ 8, 8,3 ms (load 22). Um mipmap verdadeiro (O(1) por píxel) precisa da pirâmide junto do
  `ImageMask` (25 construções em 12 ficheiros, 3 crates) — fila, se um pincel de imagem grande o pedir.
- **O pó solto, medido no Rebelle** (`po.py`, o recto de 200 e o de 40 da cena tamanhos): Soft Pastel `Dry Thin` 86 grãos
  /100 px a 200 e 15,7 a 40 (a densidade POR ÁREA é a mesma, ~0,009/px²), área p50 3 px, distância à borda p50 0,33 ·
  p90 0,90 da meia-largura, tom 0,18; Charcoal 46/100 px, 0,20 · 0,40; Flat Soft 60/100 px, 0,96 · 2,64. **No Digital**
  (Soft Round + `po` 0,9, alcance 0,75, 3 px, 0,5): 36–46/100 px acima do limiar, 0,52 · 1,10, tom 0,20.
  **No Wet NÃO serve ainda**: os grãos entram no sub-passo do lote (`wetpaint/lote.rs`) como se fossem o traço e pintam
  NÓDOAS (Soft Pastel: o miolo 92 → 109 px, os grãos 86 → 19/100 px; Charcoal: manchas de 136 px²) — o `dab_spec` desliga
  o `po` fora do Digital. ⇒ fila: a rota do Wet pousar os grãos À PARTE (sem lanes nem sub-passo).

## Portões vermelhos de outros (por nome, no fim)

21/10, ~19h: `cada_preset_de_fabrica_e_a_sua_receita`, `a_largura_e_a_prometida`, `cada_preset_parece_a_sua_midia`
verdes; `a_lei_da_tinta` vermelho só no **Wet Liner** (fnv da cena; Wet, não desta família).

## Para a fila

1. **O pó solto no Wet** (acima) — a rota do Wet; depois o reexame dos pastéis (Soft Pastel, Pastel Side, Charcoal).
2. Foliage: a pressão baixa lisa (sem a contagem sorteada na pressão baixa) e o ralo a 200 px. Grass: a fila contínua de
   tufos em gancho. Noise Grit: os grumos (densidade desigual). Spray Can: a marca a 5 px (a cobertura da marca menor que
   meio píxel) e o véu menor. Clouds/Hatching: um oráculo que se compare (Krita com preset de hachura; MyPaint `Clouds` a
   outra escala).
3. As leis sem UI (cartões Jitter/Spray): `particle_px/max/min`, `veu`, `po`, `parado_s`, `passo_pela_pressao`, `area`.
