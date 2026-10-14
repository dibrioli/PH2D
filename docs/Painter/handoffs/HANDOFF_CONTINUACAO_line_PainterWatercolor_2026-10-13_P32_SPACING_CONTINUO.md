# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 32: o Spacing contínuo (2026-10-13)

> As regras R1–R9 continuam as do topo do
> [handoff W0/W1](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-13_W0_W1_SHAPE_CASA.md). Relato do dono (13/10):
> *«Spacing em wet paint produz traços mais uniformes que em digital (que fica pontilhado)»*. Medida e porquê:
> [31 §1–§3](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md); o que esta janela fez: [31 §4](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md).

## §1 — Coordenadas

Sobre `048e52d69a` (merge-base com o `main` `757afebd58`; o `main` não andou). Dois commits desta janela (o 2.º, o smoke do dono). Sem push.
Próxima janela: o [prompt 34](../NewFeaturesWetPaint/34_PROMPT_janela_continuous_capsula.md) (3.ª rodada do smoke: o Continuous faz retângulos — o mecanismo e a lei estão lá), depois o [prompt 30](../NewFeaturesWetPaint/30_PROMPT_janela_shape_casa_W3_W4.md) (W3/W4 do plano 27).

## §2 — Item 1: as cerdas que ARRASTAM (feito)

- `Dab::step` (campo novo, `ph2d-painter-brush/src/stroke.rs`): o centro menos o do dab anterior, no caminho onde a
  tinta cai (pré-jitter). Escrito UMA vez por ponto do caminho em `Stroke::advance_step` (`stroke/dab_build.rs`,
  chamado de `dab_at` e `stamp_at`), lido por `dab_from` ⇒ as cópias do spray e as passadas do Rough partilham-no.
  Zero no 1.º dab e onde `!allows_jitter()` (Drag Dot, Anchored, Grid Stamp). O preenchimento do arremesso
  (`speed.rs::fill_thrown_gap`) reparte-o (devolve o sub-passo); a simetria reflecte/roda-o.
- **Sem segunda porta:** `ShapeFrame::Stroke { arc_len, unit_px, step }` — as 13 rotas que constroem o frame
  passam `d.step` (o enum continua sem `Default`). `shape_basis` resolve `TexDabBasis::sweep` (o passo em texels,
  só para Bristles) 1× por dab — medido: recalculá-lo por píxel custava 1,6×.
- **A lei** (`texture/bristles.rs::sample_dab`): a MÉDIA do tapete sobre os `n = máx|D|` texels do passo,
  centrada; memo por thread (8 sweeps, preenchidos texel a texel, marcados por geração). `n ≤ 1`, Tiled, Flow e o
  frame Static ⇒ a leitura de antes AO BIT. `bristles_at` (o Wet Paint) NÃO varre. `ph2d-wet-paint` ganhou
  `bristle_index`/`bristle_round` (a mesma lei de índice do `sample_bristle`, agora uma só).
- Medido pelo produto (CV ao longo, Digital Bristles, raio 20): 0,1 **0,206 → 0,014** · 0,25 **0,513 → 0,052** ·
  0,5 0,763 → 0,102; o Wet Paint idêntico número a número. ⛔ Recusado medido: o MÁXIMO ao longo (tracejado,
  2–5× a tinta) e o varrimento fechado — [31 §4.1](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md).
- Custo (variantes intercaladas no mesmo processo, mínimo de 9, `load` 10–16): recto 0,96–0,99×; curva ≤ 1,00×
  salvo Spacing 0,25: **1,15×** (C11 do [26](../NewFeaturesWetPaint/26_ABERTOS.md)). Aberto: o 1.º dab em pontos (C10).

## §2b — Smoke do dono (2.º commit): a caixa **Continuous**, também no Wet Paint

O dono (captura de tufos repetidos, pincel grande): *«não parece funcionar»* e *«não pode substituir a antiga mas
ser uma opção a mais»*. Causas: o Wet Paint não varria; o 1.º dab, sem passo, ficava em pontos (~100 px a raio 45).
- `BrushSpec::continuous_spacing` (OFF ⇒ `Dab::step` zero em `advance_step` ⇒ o traço de antes ao bit). Caixa no
  Stroke abaixo do Spacing, todo meio (`ids::PAINTER_BRUSH_CONTINUOUS_SPACING`, `paint_stroke.rs`, `populate.rs`,
  `event.rs`, `trait_impls.rs`, `seccoes::STROKE`, i18n); esmaece por `Dependente::ContinuoSemCerdas` = `!drags_bristles` (o censo reprovou esmaecê-la onde a água não lê as cerdas: o aquecimento muda a tinta); o preset do
  Wet Paint leva-a (`CAMPOS_DO_PINCEL` 30, `json.rs`).
- `BrushSpec::drags_bristles` (opção + Bristles) arma o aquecimento do Rake (`warmup.rs`): o dab de abertura recebe
  `heading · dab_spacing_px`.
- `bristles_at` varre também (escala pela razão da grelha), pela lei `Varre::Maximo`: o depósito do Wet Paint
  subtrai um portão de papel e a média esvaziava o traço (medido). O carimbo continua `Varre::Media`.
- Portões: `o_spacing_continuo_risca_no_digital_e_no_wet_paint` (pela caixa; Digital CV 0,559 → 0,055, Wet Paint
  0,762 → 0,081, nenhum esvazia; CONTROLO a ponta lisa igual) · `o_passo_continuo_e_uma_opcao_desligada_por_omissao`
  · a lei ao bit agora confere a média (carimbo) e o máximo (Wet Paint). Censos do painel 181/181 (o nome curto
  «Continuous» cabe na coluna). Aberto: a costura do Wet Paint a 0,25 (C12). Prancha:
  [`32_prancha_continuous_desligada_ligada.png`](../NewFeaturesWetPaint/32_prancha_continuous_desligada_ligada.png). Detalhe: [31 §4.4](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md).

## §3 — Item 2: o dab-cápsula da redonda — NÃO entra

Medido: a redonda já é contínua até 0,25 (CV 0 / 0 / 0,005); só a 0,5–1 há contas, onde o pontilhado é muitas
vezes pedido. [31 §4.2](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md).

## §4 — Item 3: o Spacing do Wet Paint (medido; a lei PROPOSTA espera o dono — B5)

Passo efetivo ÷ pedido nos 12 presets: 1,0–2,6× (= raio do motor ÷ raio do painel; a fábrica mascara-o no piso de
1 px; o Oil Bristle 0,33× é a ponta achatada, lei à parte). Lei proposta: o Spacing escalado por `raio painel ÷
raio motor` no `dab_spec.rs`, SÓ para o caminhador (o `spacing` também é lido por Accumulate, Space Attenuation,
`solid_carimbos`, `speed.rs`). Muda 5 presets de fábrica. Prancha:
[`32_prancha_spacing_wet_hoje_proposta.png`](../NewFeaturesWetPaint/32_prancha_spacing_wet_hoje_proposta.png).
Tabela: [31 §4.3](../NewFeaturesWetPaint/31_spacing_por_meio_o_pontilhado.md).

## §5 — Portões

- `texture::bristles::sweep_tests::o_arrasto_e_a_media_ao_longo_do_passo_centrado` — a lei AO BIT contra uma
  referência escrita à parte (a média); sem passo = a leitura de antes; `bristles_at` nunca varre; Tiled/Flow
  ignoram o passo; CONTROLO: > 200 píxeis mudam.
- `…::as_cerdas_do_digital_riscam_em_vez_de_pontilhar` — Stroke → dabs → carimbo por píxel: CV < 0,2 e < ¼ do de
  hoje a 0,1/0,25; CONTROLO o passo zerado pontilha (> 0,3); a tinta ± 3 %; a 0,025 a MESMA tinta.
- `#[ignore]`: `measure_custo_do_arrasto` (o custo), `sonda_passo_do_wet_paint`, `sonda_prancha_passo_wet`.
- Por nome nesta janela: `ph2d-painter-brush --lib` 458 ✓ · `ph2d-tool-painter --lib` (shape_bristles, symmetry,
  stroke, dab_route, texture) 182 ✓ · `presets_de_fabrica` (as miniaturas ao byte) 5 ✓.
- Bateria de fecho (`verificador`): `fmt --check` ✓ · clippy `-D warnings --all-targets` nas 5 crates que constroem
  `Dab`/`ShapeFrame::Stroke` ✓ · `PH2D_SUITE=fecho nextest-impacted` 18 297/18 300 em 571 s: 1 real — o tecto de LOC
  (`stroke.rs` 712/700; cura por CORTE: o `anchored_dab` mudou para `stroke/dab_build.rs`, 690; o portão sozinho ✓)
  e 2 de relógio do `ph2d-light2d-gpu` (fora do diff; `load` 50–94; sozinhos 3/3 ✓ — família de carga).

## §6 — Smoke do dono

Ver a mensagem de entrega (Digital ▸ Shape ▸ Bristles, Spacing 0,1 e 0,25: riscas, não pontos). Prancha:
[`32_prancha_digital_cerdas_antes_depois.png`](../NewFeaturesWetPaint/32_prancha_digital_cerdas_antes_depois.png).
