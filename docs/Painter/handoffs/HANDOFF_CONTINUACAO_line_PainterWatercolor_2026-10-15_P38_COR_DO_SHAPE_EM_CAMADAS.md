# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, prompt 38: a cor do Shape em duas camadas (2026-10-15)

## §1 — Coordenadas

Sobre o `main` `0aab2c1cb6` (14/10). Sem push. Prompt:
[38](../NewFeaturesWetPaint/38_PROMPT_rampa_do_shape_como_a_do_grain.md) · lista
[26](../NewFeaturesWetPaint/26_ABERTOS.md) (A6 feito; C23, C24 novos) · prancha
[38](../NewFeaturesWetPaint/38_prancha_cor_do_shape_em_camadas.png). Anterior: [P36](HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-14_P36_COR_DO_SHAPE.md).
Commits: `36b554f315` (código + portões) · `d3a3bc2962` (docs + portão do painel) · `9d0fb6c8c8`
(clippy) · `fc1ab9fc1b` (o 1.º smoke do dono, §2) · `a8538169fa` (gate: Texture Color sem textura com
as duas ligadas) · `cec6b6b59d` (2.º smoke: prévia da ponta + cartão inativo).

## §2 — O pedido mudou a meio e os dois smokes do dono (15/10)

O prompt pedia trocar a lei da rampa do Shape (silhueta → textura). O dono: o modo antigo fica como
*falloff color*, o novo é a *cor da textura de shape*, «ambos na mesma seção… com Blend entre eles».

**1.º smoke** (`fc1ab9fc1b`): mesma rampa para textura e falloff, a do falloff sem atuação, opacidade
para os dois, layout bagunçado. Sem textura no Shape a Texture Color indexava a
silhueta: as duas faziam o mesmo e a de cima tapava a outra. Cura: sem textura a Texture Color NÃO pinta e
o painel esmaece-a (`Dependente::TexturaSemForma`); as duas nascem ligadas (Texture em cima, opaca);
**Opacity por camada sobre a cor do pincel** (substitui o «Mix»); a secção em **cartões** (B&W e Alpha
como linhas, «Move Up» no título do cartão de baixo, Blend no de cima), fotografada:
[foto](../NewFeaturesWetPaint/38_foto_secao_shape_color.png) (`foto_shape_color_tests`, vello na placa,
`PH2D_FOTO_DIR` + `--ignored`).

**2.º smoke** (`cec6b6b59d`): com Texture em none a rampa da Texture transmitia cor (devia ficar inativa)
e a Falloff «não chegava ao pincel». O MOTOR pintava certo; mentia a **prévia da ponta** (`paint_shape.rs`:
`rampa_da_Texture[cobertura]`). Cura: a prévia pinta a mesma pilha (`paint_shape_previa.rs`, portão
`a_previa_da_ponta_pinta_as_camadas_sem_textura`, vermelho sob mutação); o cartão da Texture Color
esmaecido e sem clique sem textura (recorte de hit vazio; portão em `paint_shape_ramp_tests.rs`; o cartão
numa função, `paint_layer_card`).

## §3 — A lei (uma porta: `ph2d-painter-brush/src/ramp_alpha.rs`)

- `RampIndex::{Grain, Shape}` (o `Silhouette` do P36 saiu); `RampPaint.shape: ShapeRampLayers`.
- **Texture Color** = `lut[valor da textura do Shape]` (padrão procedural, ou o valor da imagem). Um
  padrão PROCEDURAL deixa de recortar (a silhueta é a queda: a lei do Grain em cor); uma **imagem**
  continua a ser a ponta (o valor é a cobertura). Sem textura ela não pinta.
- **Falloff Color** = `lut_queda[cobertura]` — a lei do P36 ao byte (o padrão recorta).
- A pilha: `compose_shape_ramp_layers` — sobre a cor do pincel, a de baixo em Normal com a sua Opacity,
  a de cima (`falloff_first` ⇒ a Falloff) com o Blend e a sua; exacta nas pontas. O alfa da rampa segue
  o `RampAlphaMode`. Cor do pincel à vista ⇒ só a rota por pixel (`camadas_mostram_o_pincel`). «O padrão
  vira cor» contra «recorta e colore» (desenho A), medido na prancha (coluna 4): o A esconde a ponta
  baixa da rampa; a lei escolhida exprime-o com Alpha → Strength; o inverso não.
- **B&W ligado**: a rampa do Shape é o TOM da silhueta; as camadas não entram.

**Rotas:** por pixel — `dab.rs::stamp_dab_pixels` separa `shape` (silhueta)
de `shape_cor` (o valor, amostrado por `shape_value_at` em `bands.rs`); carimbo assado —
`render_ramp_color_stamp` (agora também sem Grain); cache sem Grain — `blit_stamp_ramped` com
`CamadasDoShape.lut_combinada` (só quando os dois índices SÃO a cobertura: sem Shape, imagem, ou só
Falloff); o roteador (`stamp_route.rs`) manda a Texture Color sobre padrão procedural ao assado
(`padrao_do_shape_vira_cor`) ou ao por pixel; aguada — `rampa_da_aguada` indexa a Texture Color pela
DENSIDADE do carimbo e a Falloff pelo peso (antes SÓ a queda). Wet Paint não lê a rampa (inalterado).

**Ferramenta:** `tool/paint/shape_camadas.rs` (estado `CamadasDoShape`, setters, LUTs, eventos);
`ramp_lut::rampa_de_cor` (porta das rotas); `state.rs` (700/700) ganhou um campo por troca de um
comentário. **Painel:** `paint_ramp_widget::paint_ramp_body` (o corpo partido da secção), a secção em
`paint_shape_ramp.rs` (B&W + Alpha no topo, camadas na ordem, Swap na 1.ª linha, Blend + Opacity),
`state_shape_cor.rs`, `event/shape_ramp_picker.rs` serve as duas rampas.
Ids em `ids/painter_shape_cor.rs`. Portas de macro: 13 setters em `INTERNAS`;
`set_shape_falloff_ramp_mode` em `BENIGN_SET_MODE`.

## §4 — Medições (régua `tests/rampa_do_shape.rs::regua_a_rampa_do_shape_segue_a_textura`)

Um carimbo, rampa vermelho → azul, matiz `t = b/(r+b)`; «cobertura» = α médio ÷ o da queda sozinha.

| meio / forma / queda | antes (= Falloff Color hoje) corr padrão · queda · cob. | Texture Color | Grain Color (ref.) |
|---|---|---|---|
| Digital Checker macia | 0,141 · 0,932 · 0,500 | 1,000 · 0,000 · 0,999 | 1,000 · 0,000 · 0,999 |
| Digital Noise macia | 0,459 · 0,239 · 0,281 | 0,982 · −0,568 · 0,999 | 1,000 · −0,568 · 0,999 |
| Digital Checker dura | 0,974 · — · 0,500 | 1,000 · — · 1,000 | 1,000 · — · 1,000 |
| Impasto Checker macia | 0,455 · 0,790 · 0,500 | 1,000 · 0,000 · 1,000 | — |
| Watercolor Noise dura | 0,215 · 0,911 · 0,637 | 0,925 · 0,011 · 0,637 | — |
| Watercolor Checker dura | −0,104 · 0,819 · 0,680 | 0,634 · −0,360 · 0,680 | — |

A Falloff Color reproduz a coluna «antes». Na aguada o xadrez fica em 0,63 (espalha a cor entre
quadrados vizinhos; o Noise dá 0,92).

## §5 — Portões e mutações

- `rampa_do_shape::a_texture_color_segue_a_textura_do_shape` — Digital/Impasto × Checker/Noise ×
  dura/macia × View (assado)/Tiled (por pixel): corr ≥ 0,95, cobertura ≥ 0,99; aguada Noise ≥ 0,85;
  controlo: Falloff só, macia, segue a queda (≥ 0,9), não o padrão (≤ 0,5).
- `rampa_do_shape::as_camadas_misturam_na_ordem_escolhida` — Opacity 1 da de cima manda (nas duas
  ordens); 0 aparece a de baixo; sem textura a Texture só = sem rampa, e a Falloff pinta.
- `rampa_do_shape::cada_camada_tem_a_sua_opacidade_sobre_a_cor_do_pincel` — 1 = rampa, 0 = pincel, ½ = meio.
- Mutações do smoke 5/5 (M8 sobreviveu à 1.ª corrida: faltava «as duas ligadas, sem textura»; o portão
  ganhou-o e ela sangra); M9–M11 → o da Opacity, M12 → o da ordem.
- `cor_do_shape::a_rampa_do_shape_em_cor_nao_desliga_o_grain` (P36, reescrito): Falloff só → α igual a
  sem rampa; Texture → α igual a sem rampa e SEM Shape; o Grain morde nos dois.
- `paint_shape_ramp::tests::as_duas_camadas_pintam_os_controlos_de_cada_estado` (painel): cada estado
  (Texture só · as duas · B&W) regista os seus controlos.
- Mutações 7/7 sangram (G1 = segue a textura, G2 = camadas, G3 = D2 do P36): M1 `ramp_texel` indexa a
  Texture por `w` → G1+G2 · M2 por pixel volta a recortar → G1 (Tiled)+G3 · M3 assado volta a recortar →
  G1+G2+G3 · M4 ignora a ordem → G2 · M5 ignora o Mix → G2 · M6 roteador manda o padrão ao cache por
  cobertura → G1+G2 · M7 aguada indexa pelo peso → G1 (aguada).

## §6 — ABERTO

- **Smoke do dono** (passos na mensagem de fecho).
- C23: Shape de imagem com silhueta pelo ALFA — a Texture Color indexa o alfa (o Krita usaria a
  luminância); pede o plano de luminância nas rotas.
- C24: no Continuous o valor do Shape lê-se no centro do dab (não por sub-dab); no Impasto o relevo
  mantém o padrão e o pigmento já não recorta (assimetria do Grain em cor).
- C22 continua para «sem Shape» e para a Falloff Color (a borda AA indexa a ponta baixa da rampa).
- Do P36: B8–B12, C21, D2 (Rebelle não corrido).
