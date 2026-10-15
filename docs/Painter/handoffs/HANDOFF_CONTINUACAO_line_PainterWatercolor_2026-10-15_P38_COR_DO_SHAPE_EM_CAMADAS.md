# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 38: a cor do Shape em duas camadas (2026-10-15)

## §1 — Coordenadas

Sobre o `main` `0aab2c1cb6`. Sem push. [Prompt 38](../NewFeaturesWetPaint/38_PROMPT_rampa_do_shape_como_a_do_grain.md) · lista
[26](../NewFeaturesWetPaint/26_ABERTOS.md) (A6 feito; C23/C24 novos) · prancha
[38](../NewFeaturesWetPaint/38_prancha_cor_do_shape_em_camadas.png). Anterior: [P36](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-14_P36_COR_DO_SHAPE.md).
Commits: `36b554f315` (código) · `d3a3bc2962` (docs + painel) · `9d0fb6c8c8`
(clippy) · `fc1ab9fc1b` (1.º smoke, §2) · `a8538169fa` (gate: sem textura, as duas ligadas) ·
`cec6b6b59d` (2.º smoke: prévia + cartão inativo) · `9dac2e0d98` (o Alpha das rampas do pincel sai, §2-bis)
· `da7be2f73b` (portões do diff).

## §2 — O pedido mudou a meio e os dois smokes do dono (15/10)

O prompt pedia trocar a lei da rampa do Shape (silhueta → textura). O dono: o antigo fica como *falloff
color*, o novo é a *cor da textura de shape*, «ambos na mesma seção… com Blend entre eles».

**1.º smoke** (`fc1ab9fc1b`): mesma rampa para textura e falloff, a do falloff sem atuação, opacidade
para os dois, layout bagunçado. Sem textura a Texture indexava a silhueta: as duas faziam o
mesmo e a de cima tapava a outra. Cura: sem textura a Texture Color NÃO pinta e
o painel esmaece-a (`Dependente::TexturaSemForma`); as duas nascem ligadas (Texture em cima, opaca);
**Opacity por camada** (substitui o «Mix»; hoje sobre o transparente, §2-bis); a secção em **cartões**
(B&W e Alpha como linhas, «Move Up» no cartão de baixo, Blend no de cima):
[foto](../NewFeaturesWetPaint/38_foto_secao_shape_color.png) (`foto_shape_color_tests`, placa, `PH2D_FOTO_DIR` + `--ignored`).

**2.º smoke** (`cec6b6b59d`): com Texture em none a rampa da Texture transmitia cor (devia ficar inativa)
e a Falloff «não chegava ao pincel». O MOTOR pintava certo; mentia a **prévia da ponta** (`paint_shape.rs`: `rampa_da_Texture[cobertura]`). Cura: a prévia pinta a mesma pilha (`paint_shape_previa.rs`, portão
`a_previa_da_ponta_pinta_as_camadas_sem_textura`, vermelho sob mutação); o cartão da Texture Color
esmaecido e sem clique sem textura (hit vazio; portão em `paint_shape_ramp_tests.rs`; `paint_layer_card`).

## §2-bis — 3.º pedido do dono (15/10): simplificar o Alpha

«Vamos simplificar o Alpha: que o alpha não precise desses modos mas que obedeça o alfa de cada cor do
color ramp.» As rampas do Grain e do Shape usam sempre `BRUSH_RAMP_ALPHA` (= `RampAlphaMode::Strength`,
re-exportada de ph2d-tool-painter): o alfa da cor multiplica a cobertura (lei Krita).
As camadas do Shape empilham sobre o TRANSPARENTE (`compose_shape_ramp_layers`, W3C; alfa da
camada = alfa da cor × Opacity): cor transparente em cima mostra a de baixo; onde nenhuma tem cor não há
tinta; o pincel já não fica por baixo (`shows_brush_color`/`camadas_mostram_o_pincel` saíram, as
rotas em cache voltam a servir todos os casos). A camada de TEXTURA (feature) mantém o seu menu Alpha
(reusa o widget do Grain via `AlphaRow`). Portões: `brush_panel::ramp_stop_alpha_scales_the_painted_coverage` e
`rampa_do_shape::o_alfa_de_cada_cor_e_a_opacity_sao_a_tinta` (troca o da Opacity sobre o pincel).

## §3 — A lei (uma porta: `ph2d-painter-brush/src/ramp_alpha.rs`)

- `RampIndex::{Grain, Shape}` (o `Silhouette` do P36 saiu); `RampPaint.shape: ShapeRampLayers`.
- **Texture Color** = `lut[valor da textura do Shape]` (padrão procedural, ou o valor da imagem). Um
  padrão PROCEDURAL deixa de recortar (a silhueta é a queda: a lei do Grain em cor); uma **imagem**
  continua a ser a ponta (o valor é a cobertura). Sem textura ela não pinta.
- **Falloff Color** = `lut_queda[cobertura]` — a lei do P36 ao byte (o padrão recorta).
- A pilha: `compose_shape_ramp_layers` — sobre o transparente; Opacity × alfa da cor = alfa da tinta; a de
  baixo em Normal, a de cima (`falloff_first` ⇒ a Falloff) com o Blend; exacta nas pontas. «O padrão vira
  cor» contra «recorta e colore» (desenho A), medido na prancha (coluna 4): o A esconde a ponta baixa
  da rampa; a lei escolhida exprime-o com o alfa da cor; o inverso não.
- **B&W ligado**: a rampa do Shape é o TOM da silhueta; as camadas não entram.

**Rotas:** por pixel — `dab.rs::stamp_dab_pixels` separa `shape` (silhueta)
de `shape_cor` (o valor, amostrado por `shape_value_at` em `bands.rs`); carimbo assado —
`render_ramp_color_stamp` (agora também sem Grain); cache sem Grain — `blit_stamp_ramped` com
`CamadasDoShape.lut_combinada` (só quando os dois índices SÃO a cobertura: sem Shape, imagem, ou só
Falloff); o roteador (`stamp_route.rs`) manda a Texture Color com padrão procedural ao assado
(`padrao_do_shape_vira_cor`) ou ao por pixel; aguada — `rampa_da_aguada` indexa a Texture Color pela
DENSIDADE do carimbo e a Falloff pelo peso (antes SÓ a queda). Wet Paint não lê a rampa (inalterado).

**Ferramenta:** `tool/paint/shape_camadas.rs` (`CamadasDoShape`, setters, LUTs, eventos);
`ramp_lut::rampa_de_cor` (porta das rotas); `state.rs` (700/700): um campo por troca de um comentário. **Painel:** `paint_ramp_widget::paint_ramp_body` (o corpo partido da secção), a secção em
`paint_shape_ramp.rs` (B&W + Alpha no topo, camadas na ordem, Swap na 1.ª linha, Blend + Opacity),
`state_shape_cor.rs`, `event/shape_ramp_picker.rs` serve as duas rampas.
Ids em `ids/painter_shape_cor.rs`. Macro: 13 setters em `INTERNAS`; `set_shape_falloff_ramp_mode` em `BENIGN_SET_MODE`.

## §4 — Medições

Tabela da régua: [38_medicoes_regua_cor_do_shape.md](../NewFeaturesWetPaint/38_medicoes_regua_cor_do_shape.md).

## §5 — Portões e mutações

- `rampa_do_shape::a_texture_color_segue_a_textura_do_shape` — Digital/Impasto × Checker/Noise ×
  dura/macia × View (assado)/Tiled (por pixel): corr ≥ 0,95, cobertura ≥ 0,99; aguada Noise ≥ 0,85;
  controlo: Falloff só, macia, segue a queda (≥ 0,9), não o padrão (≤ 0,5).
- `rampa_do_shape::as_camadas_misturam_na_ordem_escolhida` — Opacity 1 da de cima manda (nas duas ordens);
  0 aparece a de baixo; sem textura a Texture só = sem rampa, e a Falloff pinta.
- ⛔ Lição: `brush_panel::accumulate_off_caps_the_stroke_even_with_a_colour_ramp` estava VERMELHO desde
  `fc1ab9fc1b` (Falloff ligada por omissão ⇒ sem textura ela pinta, com a rampa de fábrica) e só a suíte
  do diff o apanhou — teste por nome depois de mudar um DEFAULT não basta; cura: a Falloff leva a rampa vermelha.
- Mutações do smoke 5/5 (M8 sobreviveu: faltava «as duas ligadas, sem textura»; o portão ganhou-o); M9–M11 → o da Opacity, M12 → o da ordem.
- `cor_do_shape::a_rampa_do_shape_em_cor_nao_desliga_o_grain` (P36, reescrito): Falloff só → α igual a
  sem rampa; Texture → α igual a sem rampa e SEM Shape; o Grain morde nos dois.
- `paint_shape_ramp::tests::as_duas_camadas_pintam_os_controlos_de_cada_estado` (painel): Texture só · as duas · B&W.
- Mutações 7/7 sangram (G1 = segue a textura, G2 = camadas, G3 = D2 do P36): M1 `ramp_texel` indexa a
  Texture por `w` → G1+G2 · M2 por pixel volta a recortar → G1 (Tiled)+G3 · M3 assado idem → G1+G2+G3 ·
  M4 ignora a ordem → G2 · M5 ignora o Mix → G2 · M6 roteador manda o padrão ao cache por cobertura →
  G1+G2 · M7 aguada indexa pelo peso → G1 (aguada).

**4.º (`4ecd67d726`):** B&W mantém os dois cartões; tom = luminância da pilha (`ensure_shape_ramp_lut`).

## §6 — ABERTO

- Smoke do dono: OK até ao Alpha (15/10); o do B&W pendente.
- C23: Shape de imagem com silhueta pelo ALFA — a Texture Color indexa o alfa (Krita: a luminância).
- C24: no Continuous o valor do Shape lê-se no centro do dab; no Impasto o relevo mantém o padrão.
- C22 segue para «sem Shape» e a Falloff Color (a borda AA indexa a ponta baixa da rampa).
- Do P36: B8–B12, C21, D2 (Rebelle não corrido).
