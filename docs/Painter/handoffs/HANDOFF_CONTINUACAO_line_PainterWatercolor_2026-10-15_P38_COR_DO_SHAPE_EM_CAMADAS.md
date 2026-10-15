# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 38: a cor do Shape em duas camadas (2026-10-15)

## §1 — Coordenadas

Linha sobre o `main` `0aab2c1cb6` (14/10). Sem push. Prompt:
[38](../NewFeaturesWetPaint/38_PROMPT_rampa_do_shape_como_a_do_grain.md) · lista
[26](../NewFeaturesWetPaint/26_ABERTOS.md) (A6 feito; C23, C24 novos) · prancha
[38](../NewFeaturesWetPaint/38_prancha_cor_do_shape_em_camadas.png). Anterior: [P36](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-14_P36_COR_DO_SHAPE.md).
Commits: `36b554f315` (código + portões) · o de docs + portão do painel.

## §2 — O pedido mudou a meio (o dono, 15/10)

O prompt pedia trocar a lei da rampa do Shape (cor pela silhueta → cor pela textura). A meio da janela
o dono: *«o modo como está pode ser preservado como falloff color»* e *«o novo seria a cor da textura
de shape. Ambos na mesma seção com possibilidade de organizar que é o primeiro e com a opção de Blend
entre eles»*. Fez-se isso.

## §3 — A lei (uma porta: `ph2d-painter-brush/src/ramp_alpha.rs`)

- `RampIndex::{Grain, Shape}` (o `Silhouette` do P36 saiu); `RampPaint.shape: ShapeRampLayers`.
- **Texture Color** = `lut[valor da textura do Shape]` (padrão procedural, ou o valor da imagem). Um
  padrão PROCEDURAL deixa de recortar (a silhueta é a queda), a lei do Grain em cor; uma **imagem**
  continua a ser a ponta (o valor dela é a cobertura). Sem Shape activo o valor é a silhueta.
- **Falloff Color** = `lut_queda[cobertura]` — a lei do P36 ao byte (o padrão recorta).
- As duas: `compose_shape_ramp_layers` — a de cima (`falloff_first`) sobre a de baixo por
  `blend::blend_over(modo, baixo, cima.rgb, cima.a × mix)`. A escolha «o padrão vira cor» contra
  «recorta e colore» (o desenho A) foi medida na prancha (coluna 4): o A esconde a ponta baixa da rampa,
  e exprime-se na lei escolhida com a rampa em Alpha → Strength; o inverso não.
- **B&W ligado**: a rampa do Shape é o TOM da silhueta, como antes; as camadas não entram.

**Rotas (varridas por consumidor):** por pixel — `dab.rs::stamp_dab_pixels` separa `shape` (silhueta)
de `shape_cor` (o valor, amostrado por `shape_value_at` em `bands.rs`); carimbo assado —
`render_ramp_color_stamp` (agora também sem Grain); carimbo em cache sem Grain — `blit_stamp_ramped`
com `CamadasDoShape.lut_combinada` (só quando os dois índices SÃO a cobertura: sem Shape, imagem, ou só
Falloff); o roteador (`stamp_route.rs`) manda a Texture Color sobre padrão procedural ao assado
(`padrao_do_shape_vira_cor`) ou ao por pixel; a aguada — `rampa_da_aguada` indexa a Texture Color pela
DENSIDADE do carimbo (o valor do Shape) e a Falloff pelo peso (antes ela seguia SÓ a queda, mesmo com
ponta dura). Wet Paint não lê a rampa de cor (inalterado).

**Ferramenta:** `tool/paint/shape_camadas.rs` (estado `CamadasDoShape`, setters, LUTs, eventos);
`ramp_lut::rampa_de_cor` (uma porta para as rotas); `state.rs` (700/700) ganhou um campo por troca de um
comentário. **Painel:** `paint_ramp_widget::paint_ramp_body` (o corpo partido da secção), a secção em
`paint_shape_ramp.rs` (B&W + Alpha no topo, as camadas na ordem, Swap na 1.ª linha, Blend + Opacity com
as duas), `state_shape_cor.rs`, o seletor de cor `event/shape_ramp_picker.rs` serve as duas rampas.
Ids em `ids/painter_shape_cor.rs`. Portas de macro: 13 setters em `INTERNAS`;
`set_shape_falloff_ramp_mode` em `BENIGN_SET_MODE`.

## §4 — Medições (régua `tests/rampa_do_shape.rs::regua_a_rampa_do_shape_segue_a_textura`)

Um carimbo, rampa vermelho → azul, matiz `t = b/(r+b)` contra o padrão e contra a queda; «cobertura» =
α médio ÷ o da queda sozinha.

| meio / forma / queda | antes (= Falloff Color hoje) corr padrão · queda · cob. | Texture Color | Grain Color (ref.) |
|---|---|---|---|
| Digital Checker macia | 0,141 · 0,932 · 0,500 | 1,000 · 0,000 · 0,999 | 1,000 · 0,000 · 0,999 |
| Digital Noise macia | 0,459 · 0,239 · 0,281 | 0,982 · −0,568 · 0,999 | 1,000 · −0,568 · 0,999 |
| Digital Checker dura | 0,974 · — · 0,500 | 1,000 · — · 1,000 | 1,000 · — · 1,000 |
| Impasto Checker macia | 0,455 · 0,790 · 0,500 | 1,000 · 0,000 · 1,000 | — |
| Watercolor Noise dura | 0,215 · 0,911 · 0,637 | 0,925 · 0,011 · 0,637 | — |
| Watercolor Checker dura | −0,104 · 0,819 · 0,680 | 0,634 · −0,360 · 0,680 | — |

A Falloff Color reproduz a coluna «antes» número a número (é o código de antes). Na aguada o xadrez
fica em 0,63: a aguada espalha a cor entre quadrados vizinhos (o Noise, suave, dá 0,92).

## §5 — Portões e mutações

- `rampa_do_shape::a_texture_color_segue_a_textura_do_shape` — Digital e Impasto × Checker/Noise ×
  dura/macia × View (assado)/Tiled (por pixel): corr ≥ 0,95 e cobertura ≥ 0,99; aguada Noise ≥ 0,85;
  controlo: Falloff só, macia, segue a queda (≥ 0,9) e não o padrão (≤ 0,5).
- `rampa_do_shape::as_camadas_misturam_na_ordem_escolhida` — Mix 1: manda a de cima (nas duas ordens);
  Mix 0: aparece a de baixo; sem Shape, Texture só e Falloff só pintam o mesmo byte (LUT combinada).
- `cor_do_shape::a_rampa_do_shape_em_cor_nao_desliga_o_grain` (P36, reescrito): Falloff só → α igual a
  sem rampa; Texture → α igual a sem rampa e SEM Shape; o Grain morde nos dois.
- `paint_shape_ramp::tests::as_duas_camadas_pintam_os_controlos_de_cada_estado` (painel): cada estado
  (Texture só · as duas · B&W) regista os controlos dele, e os das camadas não se sobrepõem.
- Mutações 7/7 sangram (G1 = segue a textura, G2 = camadas, G3 = D2 do P36): M1 `ramp_texel` indexa a
  Texture por `w` → G1+G2 · M2 o por pixel volta a recortar → G1 (Tiled)+G3 · M3 o assado volta a
  recortar → G1+G2+G3 · M4 ignora a ordem → G2 · M5 ignora o Mix → G2 · M6 o roteador manda o padrão
  ao cache por cobertura → G1+G2 · M7 a aguada indexa pelo peso → G1 (aguada).

## §6 — ABERTO

- **Smoke do dono** (passos na mensagem de fecho; prancha acima).
- C23: Shape de imagem com silhueta pelo ALFA — a Texture Color indexa o alfa (o Krita usaria a
  luminância); pede o plano de luminância nas rotas.
- C24: no Continuous o valor do Shape lê-se no centro do dab (não por sub-dab); no Impasto o relevo
  mantém o padrão enquanto o pigmento já não recorta (a mesma assimetria do Grain em cor).
- C22 continua para «sem Shape» e para a Falloff Color (a borda AA indexa a ponta baixa da rampa).
- Do P36 seguem: B8–B12, C21, D2 (Rebelle não corrido).
