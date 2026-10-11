# CONTINUAÇÃO — `line/motion-value`: ZERAR as pendências, família B (B4 feito, B3/B6 fechados, B5 é a próxima onda) (11/10)

> Ordem do Enio (11/10): *«vamos até zerar as pendências»*. Regras R1–R9 do topo do
> [`…_SO_O_ESTADO.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_SO_O_ESTADO.md). Caminhos de saída:
> [`…_ZERAR_PENDENCIAS.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_ZERAR_PENDENCIAS.md). Família A:
> [`…_ZERAR_FAMILIA_A.md`](HANDOFF_CONTINUACAO_line_motion_value_2026-10-11_ZERAR_FAMILIA_A.md). **Este ficheiro
> substitui o da família A como fila viva.**

## Onde a linha está

- Worktree `Worktrees/line-motion-value`, branch `line/motion-value`, base `8f20c60de` (= `main`), B4 em
  `e0de5f41d9`. **Nada integrado nem enviado.**
- ⏳ Re-smoke das estrelas a girar (cena `=70`; o dono disse *«as estrelas estão paradas»*, cura em `f0370a2786`,
  gate `the_stars_orbit_and_stay_on_screen`).
- ⏳ Smoke do B4: cena `=135`, passos (10) Gradient Map arco-íris e (11) Multiply (`PH2D_ACABAMENTO_PASSO=10|11`);
  fotografados (discos ciano→azul→violeta; o Multiply escurece). Tutorial 13 com os dois passos (PDF 7→9 páginas).
  Os atalhos de predefinição do editor são tirinhas de cor SEM texto: o roteiro diz «a 1.ª tirinha».
  binário do smoke: <colar>

## B4 — FEITO (`fx.filter`: lei de mistura + degradê inteiro)

- **Cartão:** linha `Blend` (Enum de 20 leis, chaves `node.opts.node_fx_filter.blend_labels.N`, palavras iguais às de
  `BlendMode` — gate) só nos 6 tipos `takes_blend` (Inner Shadow, Inner Glow, Bevel, Color Overlay, Duotone, Gradient
  Map); linha `Gradient` = param de texto `ramp` com `ParamWidget::Gradient` (editor do doc 85: paragens arrastáveis,
  OKLCH por paragem, 5 interpolações, 5 espaços, até 32) só no Gradient Map. `ramp_mode` saiu; as duas cores já não
  alimentam a rampa. Nenhuma cena usava Gradient Map e o `fx.filter` chegou ao main em 10–11/10 ⇒ sem migração.
- **Placa:** `FxOpGpu::ramp_lut: Option<Arc<[[f32;4]]>>` (perdeu `Copy`), binding 5 `ramp_lut`, `Globals.ramp =
  [base,len,0,0]` (320→336 B, stride 512 intacto); sem tabela o caminho das paragens é byte-idêntico (105 gates
  `fx_stack` verdes). `FxOp::RAMP_LUT_TEXELS = 4096`, MEDIDO no encoding de SAÍDA (sRGB 8 bits): pior erro `0,0327`
  @512, `0,0117` @2048, `0,0038` @4096 (< 1/255); gate `the_baked_ramp_is_invisible_and_half_of_it_is_not`.
  ⛔ Recusado (medido): guardar a tabela em sRGB — o erro vem dos JOELHOS das paragens, não da curvatura
  (`0,0478` @512). Sonda `measure_ramp_lut_in_the_screen_encoding`.
- **Cache:** `RampasAssadas` (`ph2d-node-fx-filter/src/rampa.rs`, 693 linhas), uma cozedura por texto entre quadros;
  gate `the_ramp_bakes_once_per_text`. Seam: `the_cards_blend_and_gradient_reach_the_step_the_pass_runs`.
- **Oráculo (§0.9):** ImageMagick 7.1.2-32 (licença ImageMagick; liblqr GPL não usada), 49 pares nossos, PFM
  flutuante: 12 leis iguais (≤ 1,2e-7); ColorBurn/ColorDodge/VividLight só diferem no canto 0/0; SoftLight = IM
  `min(1, W3C+fundo)`; Hue/Saturation/Color/Luminosity diferem (IM ≠ W3C SetLum/SetSat). A nossa lei fica W3C;
  divergências fixadas em `crates/ph2d-blend-mode/tests/oraculo/` + `tests/it/oraculo_imagemagick.rs`.
- **Custo (sonda `ph2d-render/tests/it/fx_stack_mistura_custo.rs`, release, 9 blocos intercalados, mínimo, load ~5;
  colunas = ÁREAS: camada 1600×972 · 4K · atlas 4096 · atlas 32 761 formas, ms):** pilha vazia
  `0,080/0,357/0,120/0,764` · Color Overlay Normal `0,104/0,484/0,135/1,046` · pior lei (17 Saturation)
  `0,111/0,570/0,164/1,270` · Gradient Map 2 paragens `0,114/0,520/0,167/1,104`, 8 paragens
  `0,123/0,564/0,167/1,230`, tabela 4096 `0,120/0,502/0,157/1,061`, tabela+Hue `0,127/0,561/0,172/1,229`.
  ⇒ nenhuma lei recusada: pior `+0,09 ms` a 4K, `+0,22 ms` a 32 761.

## B3 e B6 — FECHADOS por medição

- **B3:** o retângulo da camada usa `scene_viewport`; `cena_visivel` (só a lente, `na_escala.rs`) é ~17 % menor só com
  painéis por cima do mundo (doc 129 parte_1 l.89–95). Rect maior = superconjunto ⇒ sem erro visível, só custo:
  ≈ `0,03 ms` por passo de ponto a 4K e ≤ 17 % da pilha de `0,77 ms` ⇒ ≈ `0,13 ms`. Ligar `cena_visivel` acoplaria a
  app à geometria do chrome. Mantido.
- **B6:** fechado antes (doc 129, item 4): a recusa é sobre SAÍDAS, não fios.

## ⚠️ Item NOVO aberto (mesma família de B4)

`fx.glow` `HALO_LUT_TEXELS = 512` foi medido em luz LINEAR (`ph2d-node-fx-glow/src/ramp_tests.rs`, `DISPLAY_STEP`
sobre valores lineares). Na saída sRGB os mesmos joelhos provavelmente passam de 1/255 ⇒ **re-medir a régua do glow
no encoding de saída** (como em B4) e subir o número se a tabela o mandar.

## B5 — PRÓXIMA ONDA (factos levantados, não refazer)

`Multiplica`/mistura por instância na placa. As formas na placa são um passe de raster
(`ph2d-shape-gpu/src/pass.rs:228-243`) com UM `PREMULTIPLIED_ALPHA_BLENDING` fixo; não há mistura por instância ⇒
Everything/Copies e Blend por fileira vão para o Vello na camada (+`0,4–1,9 ms`). Add/Screen cabem em estados fixos
(como `band_blit.rs` `PASSES`); o Multiply exacto por instância sobreposta pede `DUAL_SOURCE_BLENDING`
(`out = s·(1−αb) + b·(s+1−αs)`: factor src `OneMinusDstAlpha`, dst `Src1`). Os dois adaptadores da estação dão
`dualSrcBlend=true` (RTX 5060 Ti + RADV Raphael). Nada pede a feature (`ph2d-gpu/src/context.rs:69-80`).
Predicados de rota: `MisturaDoSink::pousa_na_camada` (`ph2d-eval-motion/src/sink_style.rs:130`),
`sink_mistura_em_grupo` + `RECUSA_MISTURA_EM_GRUPO` (`ph2d-app-motion/src/motion_bridge_gpu.rs:135-143,298-300`),
`GeometriasDaPlaca` (`motion_shape_placa.rs`). **Plano:** pedir a feature onde o adaptador a tem; variantes de
pipeline por mistura no passe das formas; Vello fica de reserva por adaptador; gates de paridade contra a rota Vello;
tabela de custo contra os `0,49–2,25 ms` do Vello na camada.

## A fila que resta

**C — Desempenho** (máquina partilhada a `load 26–272` hoje: nenhum relógio valeu; medir com
`MEDIR_VELOCIDADE.md`, variantes no mesmo processo, intercaladas, o mínimo):
7. `scan` por subgrupos na cadeia pequena (`1,7–6×` a CPU) + o passe recria grupos de ligação e
uniforms por cozimento ([doc 130 §3.1](../130_o_passe_na_placa.md)).
8. `Boundaries` (rota híbrida) e `cook_taps_only` que coze todas as tomadas.
9. `opt-level` da `ph2d-contact`.
10. (W3b) rota de placa do `SpriteRenderer` na silhueta; `4 096` formas = 4 páginas; U2.
11. `Bezier`/`Tint` OKLCH caros na CPU de reserva; a fita acima de `32 MiB`.
12. Engasgo de `0,115 ms`/tique na troca de rota a meio — medir se se vê.

**D — Produto:** 13. uma linha de COR inteira num gesto; vida e dano pedem dois objectos ·
14. colisor escrito PELO NOME (`RECUSA_COLISOR`) ainda na CPU.

**E — Documentação:** 15. `33` docs do Motion acima de 40 KB (o maior `115_…`, 170 KB) — lote
mecânico para o `documentador` com `python3 scripts/doc-split.py --keep … --archive …`.

## Fecho desta janela (B4)

Portão §1.5.9 pelo `verificador` sobre o diff acumulado: `nextest-impacted` `21 435/21 437`, 2 falhas
determinísticas curadas e reconferidas (tecto de LOC `ph2d-node-fx-filter/src/lib.rs` 747>700 ⇒ cozedura movida para
`rampa.rs`; censo de chaves de enum ⇒ as etiquetas de Blend ganharam `node.opts`). `cargo check --workspace
--all-targets` com warnings=deny, GPU `fx_stack` 105/105, `fx-cells` 5/5, `motion_filter` 4/4 (load 25), fmt, machete,
typos (2 «librarys» renomeadas) e clippy `-D warnings` verdes. ⚠️ `FxOpGpu` perdeu `Copy`: ~19 literais de teste e 5
`from_ref` do clippy tocados — um merge que traga um literal novo de `FxOpGpu` parte aqui.
