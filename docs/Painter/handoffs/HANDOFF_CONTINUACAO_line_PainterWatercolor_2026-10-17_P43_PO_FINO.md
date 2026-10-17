# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 43: o pó fino do aerógrafo (2026-10-17)

## §1 — Coordenadas

Sobre o `main` `0aab2c1cb6` (merge-base, sem rebase). Sem push. [Prompt 43](../NewFeaturesWetPaint/43_PROMPT_o_po_fino_aerografo.md)
· medição e lei: [43_medicao](../NewFeaturesWetPaint/43_medicao_o_po_fino.md) · [prancha 43](../NewFeaturesWetPaint/43_prancha_po_fino.png)
· doc [18](../NewFeaturesWetPaint/18_mapa_das_midias.md) §5 bloco B ✅. Anterior: [P42](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-16_P42_LEI_DA_TINTA.md).
Commits `2284bf9349` (identidade das 17, ANTES da lei) · `bea4bad67d` (oráculo) · `b94cc74f3f` (lei + Airbrush) ·
`6f9c19c60c` (portões, mutação, docs) · o deste fecho.

## §2 — Medido primeiro

- **A célula não era o limite** (18 §3 corrigido): a razão da grelha é 1; o candidato do 18 tinha o grão de lado
  0,02/0,05, abaixo do corte `TEX_SIZE_MIN` 0,1 — os dois liam 0,1 (manchas de 2 560 px).
- **A composição de hoje não chega**: grão `Tiled` lado 100 = o mesmo desenho em todo traço; `Random` = média dos
  dabs sobrepostos (2–11 %). Melhor 5/8.
- **O oráculo corrido** (`airbrush20`, `airbrush40b`, `airbrush80`, `gera_eventos.py` + `gotas_fecha.sh`): o grão do
  pó do Rebelle é **~1,3 px a 20, 40 e 80 px**; textura ~23 % em todos; o «período» não distingue (repetição 0,005).
- ⚠️ **Armadilha**: uma receita com `felt` no mapa `engine` RECRIA as cerdas por cima do Shape «None»
  (`wet_preset_aplicar`, a migração pré-W2). As receitas da sonda devem omitir as 4 chaves de `BRISTLE_UNIT_KNOBS`.
- ⚠️ A sonda de mídias mede à lei de largura do preset em memória (Wet Round); o preset de fábrica traz a SUA lei —
  o portão pinta o Airbrush no raio 15 (a sonda: 12) para a mesma largura 26.

## §3 — A lei

- **Motor** (`ph2d-wet-paint`): `Knob::Dust` (chave `dust`, Paint, 0..1, omissão 0, último do registo,
  `KNOB_COUNT` 56). Em `DepositLaw::cell` (os dois chamadores, o do produto e o serial do motor): `Po` — a gota é um
  quadrado de `GOTA_DO_PO` = 2 células com a fase do dab; cai com `p = 1 − POEIRA_MAXIMA(0,95)·dust` e leva a dose
  `÷ p` (a média de sempre, antes do portão do papel); a gota que não cai não pousa nem a umidade. Pó 0 → `None`, o
  carimbo de antes ao byte.
- **A semente**: o hospedeiro (`dab_route.rs`) dá um passo no fluxo do pincel (`tex_rng`, splitmix64) por dab SÓ com
  o pó ligado e escreve `Sim::semente_do_po`; o motor mistura-lhe o centro do dab (`Params::semente_do_po`).
- **Recusados (medido)**: a densidade seguindo a queda (L3: borda 0,71–0,80), gotas de 1, 1,5, 2,5 e 3 células, o
  grão `Random`/`Tiled` da composição — tabela no 43 «⛔ Recusas».
- **Formato**: o pó vai no mapa `engine` do preset, por nome; um preset sem a chave lê 0. **`SCHEMA` fica 1** (o
  leitor ignora chaves que não conhece — um build antigo pinta o Airbrush sem pó).
- **Painel**: Tuning › Paint ganhou «Brush dust» (i18n; a linha nasce do registo, como o «Brush glaze»).
- **Fábrica 17 → 18**: `assets/brushes/WetPainter/Airbrush/Spray/Airbrush` — seco, sem cerdas (Shape None),
  dureza 0,3 (Smooth), pó 0,8, pigmento 9. Contra o Rebelle (40 px): cob 0,359 · 0,381, tex 21,9 · 22,7, borda
  0,596 · 0,537, orla 12 · 12, cv 0,068 · 0,088, cruz 0,523 · 0,583, larg 26 · 26 (7/7). Rótulos das pastas
  «Airbrush» e «Spray» em `rotulo_da_pasta` (ph2d-app-painter) + i18n.

## §4 — Portões

`ph2d-tool-painter` `it/a_lei_da_tinta.rs`: a identidade das **17** (cenas ao byte, os 5 de tinta pinados em
`ddc782e9f0`). `it/o_po_do_aerografo.rs` (4): o 2.º traço no mesmo sítio cai noutras gotas (correlação < 0,3) · o
pincel gravado leva o pó · o grão é do píxel (textura a 40 e 80 px ± 3,5 da do Rebelle) · sem pó o fluxo do pincel
não anda (Soft Pastel com grão `Random`, ao byte). `it/midias_de_fabrica.rs`: 14 mídias; o Airbrush com Cobertura,
Textura, Borda, Orla, **Cv**, Cruzamento e **Largura** ± 2 px (a régua portada `regua_midia` ganhou `cv` e `largura`,
as do `mede_traco.py`). `ph2d-wet-paint` `it/a_lei_do_po.rs` (5): pó 0 ao bit · a massa (24 sementes, ± 3 %) · gotas
de 2 células na fracção 0,24 (correlação ½ a 1 célula, 0 a 2) · a semente escolhe · o centro do dab (¼ de célula).

## §5 — Mutação

**9/9 sangram** (1.ª rodada 7/9: M5 «a semente ignora o centro» e M8 «sorteia sem pó» ganharam portão — o deslocamento
de ¼ de célula e o Soft Pastel `Random` pinado). Lista: 43 §5.

## §6 — Aberto

- **A 20 px a textura lê 28,7 contra 22,5** do Rebelle (40 e 80 batem). A largura do traço é 13 px: a janela 5×5 da
  régua vê a orla — não investigado.
- **A pressão**: no Rebelle (Spray 2: `pressure_size_influence` 0, `pressure_opacity_influence` 1) a rampa ESMAECE;
  a nossa ESTREITA (a prancha, linha 2). Medir se o pincel do Wet Paint tem a pressão → opacidade antes de construir.
- **A gota é da CÉLULA**: com a razão da grelha > 1 (o slider do artista) a gota mede 2·razão px.
- **Custo**: um hash por célula por dab só com o pó ligado — não medido em intercalado (o traço do portão corre em
  0,3 s); medir antes de qualquer tecto.
- **B13/B14** (do P42), perguntas do smoke.

## §7 — Fecho

`cargo fmt --all --check` ✓ · clippy `-D warnings --all-targets` (`PH2D_SUITE=fecho`; wet-paint, tool-painter,
panel-wet-tuning, panel-painter-layers, i18n, app-painter com `panel-painter-layers`) ✓ · `nextest-impacted` (base
`0aab2c1cb6`) **18 517/18 519** em 518 s (`load` 2,4 → 8,5): os 2 eram contagens desta linha, curadas e refeitas por
nome — `rows::every_visible_knob_has_a_row` 29 → 30 (Paint 11, o `dust`) e `shape_color_tests::a_cor_sobrevive_a_agua_
nos_pinceis_de_fabrica` 17 → 18 (o Airbrush passa a rampa).
