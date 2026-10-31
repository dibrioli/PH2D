# R2 — pincéis do Rebelle que faltam (janela reserva 02, 31/10)

Ramo `line/PainterWatercolor-r2` (base `6ba5d06493`). Três agentes em paralelo por família, método REGRAS_DE_CUSTO
(simular antes de construir, ≤ 2 rondas, teste por nome). Detalhe: [A pontas](R2_A_pontas.md) ·
[B óleo](R2_B_oleo_grao.md) · [C spray/marcador](R2_C_spray_marker.md).

## Resultado

| pincel | estado | o que mudou | prancha |
|---|---|---|---|
| Charcoal | melhorado, na fábrica | fim CARREGA (esbate negativo): fim do recto 200 0,21·0,07·0,03 → 0,32·0,15·0,08 (Rebelle 0,31·0,18·0,06); miolo igual | [r2_charcoal](pranchas/r2_charcoal.png) |
| Wax Scrape | melhorado, na fábrica | fim 0,64·0,28·0,09 → 0,82·0,55·0,32 (Rebelle 0,84·0,63·0,48); miolo igual | [r2_wax_scrape](pranchas/r2_wax_scrape.png) |
| Square Hard | melhorado, na fábrica | esquina da entrada recta pelo eixo: altura no canto ÷ corpo 0,59 → 0,95 (40 px), 0,70 → 0,95 (200 px) | [r2_square_hard](pranchas/r2_square_hard.png) |
| Marker Chisel (candidato) | muda de passagem | a mesma esquina recta (lei das pontas rectas) | — |
| Knife Stain | NÃO curado | mecanismo medido: `sem_entalhes` (`oleo_borda.rs`) sobe cobertura à mediana e pinta 13,8 % do papel entre grãos; com o grão salvo sobram 2–3 pontinhos de sombra na tinta (`a_orla_rasgada_nao_tem_pontinhos_escuros`) | [r2_knife_stain](pranchas/r2_knife_stain.png) |
| Knife Grainy | NÃO curado | o grão morre no DEPÓSITO (sem óleo já 16,7 × Rebelle 85,6); 7 variantes de receita, nenhuma dá as lascas claras/escuras | [r2_knife_grainy](pranchas/r2_knife_grainy.png) |
| Grunge | não começado | sem receita em `oleo_acrilica.rs` | — |
| Spray Can | NÃO promovido | lei `VEU_OPACIDADE_MAX = 4` (`c4564d3294`, neutra ≤ 1, teste `o_veu_acima_de_um_e_a_populacao_escura`) não escurece: a cobertura do dab já satura em 1 (opac 1 × pressão; o fluxo só entra na composição) | [r2_spray_can](pranchas/r2_spray_can.png) |
| Dry Marker | NÃO curado | Rebelle: riscas de 8 px ±0,018; no Wet grão de riscas e espaçamento não mudam nada; a receita Digital em estudo está mais perto (candidata a cega/dono) | [r2_dry_marker](pranchas/r2_dry_marker.png) |
| Splatter, Rake Sharp, Blender Grainy | não começados | prazo / rendimento | — |

Leis novas (neutras ao bit no ponto antigo): `pontas_esbate` aceita −4..8 (abaixo de 0 as pontas carregam e o resto
do rasto pousa ao soltar) · entrada das pontas rectas medida no eixo · `veu_opacidade` até 4.
Testes novos verdes: `as_pontas_carregam_com_o_esbate_negativo`, `a_entrada_das_pontas_rectas_tem_a_esquina_cheia`,
`o_veu_acima_de_um_e_a_populacao_escura`. Ignorado com o número: `o_grao_ralo_nao_engorda_no_oleo` (+0,85).

## Fábrica

Charcoal, Wax Scrape e Square Hard regerados (preset + miniatura). `cada_preset_de_fabrica_e_a_sua_receita` e
`the_factory_presets_are_counted` verdes. ⚠️ `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` VERMELHO em 23 presets
fora da R2 (Blend ×4, Smear ×5, Oil ×10, Ink Brush, Dip Pen, Wet Liner, Wet Wash) — o «miniaturas (24 mudam)» da J11
no ESTADO, anterior à R2; nenhuma lei da R2 corre neles. Fica para a integração. Idem `a_borda_da_mancha_molhada_esbate`
(Wet Liner, `pontas_esbate` 0), visto uma vez pelo R2-A sob edições dos irmãos.

## Precisa de ficheiro proibido / decisão

- `spec.rs`: doc de `pontas_esbate` passa a −4..8; o chão devia morar ao lado de `PONTAS_ESBATE_MAX` (hoje em `ids`).
- Knife Stain: o pontinho é a sombra da tinta à volta do furo novo (por eliminação) — curar na luz do óleo; depois
  regerar Oil/* pela fila pesada.
- Spray Can: tinta na FORÇA (opac 0,4 · fluxo 1) ou fluxo próprio do véu — muda a acumulação, medir antes.
- Dry Marker: glaze por traço (CAP-11) com carimbo visível; por que o Wet ignora o grão (`wetpaint/pousa_o_dab.rs`).
- Square Hard: grão fino e denso pelo diâmetro (lei de carga/grão por tamanho).
- Charcoal: o Rebelle tem estrias horizontais no miolo; o nosso é granulado.

## Rendimento

~1 h 40 de agentes (3 × ~30 min + 3 retomas): 3 pincéis melhorados, 0 aprovados por cega (não corrida), 3 leis de
motor neutras. Abaixo de 1/h ⇒ reavaliar com o dono antes de outra ronda.

## Ronda 2 (coordenador 31/10): Thick Impasto · Oil Grunge · Blender Grainy · Splatter

- **Blender Grainy** (R2-F, `c4b060c5bd`, [relatório](R2_F_blender_grainy.md), [prancha](pranchas/r2_blender_grainy.png)): os grumos vinham do
  grão da máscara (ruído de lado 24 ⇒ manchas de ~24 px com buracos); grão 4 + toque 0,35 tira-os. ⛔ NÃO promovido
  (coordenador R2, olhando a prancha): o «depois» é uma nuvem pálida e larga (1,5× a do Rebelle, miolo 80 × 350 px
  opacos); o Rebelle tem LASCAS escuras recortadas — o «antes» tinha mais o carácter dele. Candidato para o smoke do
  dono com as duas receitas; a próxima tentativa é grão médio (lado ~10) com o toque cheio, não mais fino.
- **Splatter** (R2-F, `61c9ec83cc` `b419869382`, [relatório](R2_F_splatter.md), [prancha](pranchas/r2_splatter.png)): o de hoje é a mancha
  do Rebelle a 55 % (pálido, um tamanho). s5 (disco opaco + 2.ª população do véu a 1 + tecto 36 px por gota) dá gotas
  opacas de vários tamanhos sem motor novo. ⛔ NÃO promovido (coordenador R2, na prancha): círculos perfeitos = bolinhas,
  sem coroa nem cauda. A próxima: a MANCHA recortada do «antes» (que já tem coroa) opaca, com a 2.ª população e o tecto
  da s5. Blender Grainy: o preset de estudo ficou na v7; a J6 volta por `git checkout 0319d50dcf -- docs/Painter/Pinceis/pranchas/mistura/presets/blender_grainy.ph2d-brush.png`.
- **Oil Grunge** (R2-E, `5ee08084ee` `ed6c053a7a`, [relatório](R2_E_grunge.md), [prancha](pranchas/r2_oil_grunge.png)): o Grunge do
  Rebelle não tem relevo ⇒ Real Oil desligado (o `sem_entalhes` não entra). Receita `OIL_GRUNGE` (cerdas «Grunge C» +
  Shape de ruído no Flow), 2 rondas: furos 0,057 × 0,054, mas o corpo é opaco e chapado (contraste das estrias 0,056 ×
  0,117; o vermelho vê-se através no Rebelle), fendas duras, traços quase pretos do `bristleStreakCut`. ⛔ Fora da
  fábrica. Próximo a simular: pigmento 40–60 com menos opacidade, Shape a contraste 0,3–0,5, sem `bristleStreakCut`;
  senão, motor: Shape que modula a CARGA em vez do alfa.
- **Splatter** ENTROU na fábrica (`3efdecda4f`, `Digital/Splatter/Splatter`): m4 da 2.ª volta do R2-F — a mancha opaca
  com coroa, 6 por dab, 2.ª população do véu a 1, tecto 72 px por gota (sem motor novo). Nos gestos finos fica ralo.
- **Thick Impasto** ENTROU na fábrica (R2-D `adf77b190a`, gerado no preset seguinte; [relatório](R2_D_thick.md),
  [prancha](pranchas/r2_thick_impasto.png)): composição sobre o Filbert (Real Oil 0,9, brilho 0,5, pigmento 600, emboss
  0,3, dureza 0,3, Clouds 48 px); sai o estudo homónimo do meio Impasto. Mais relevo e luz que o Rebelle; a ver no smoke:
  faixa cinza de sombra (~10 px) sob os traços finos (1.º botão: Profundidade 0,6–0,7) e a textura algo carregada.
  ⚠️ Custo: sonda 8 → 85 s por traço com a Clouds (load 26–82) — medir no app. Geração 9 min (load 47).
  `the_factory_presets_are_counted`, `cada_preset_de_fabrica_e_a_sua_receita`, `os_reprovados_do_oleo_nao_estao_na_fabrica` verdes.
- ⚠️ Hook `prazo-do-agente.sh:47` recusa qualquer comando com `PH2D_REGERA_`, até o `fila-pesada.sh pôr` que ele manda usar.
- **Total da R2:** 5 presets de fábrica mudados ou novos (Charcoal, Wax Scrape, Square Hard melhorados; Splatter e Thick
  Impasto novos), nenhum por revisão cega — para o smoke do dono.
