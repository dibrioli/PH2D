# Exame da família `basicos` (J3, catálogo A — os primeiros presets do meio Digital)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/basicos.rs`. Pranchas: `pranchas/basicos/` (cegas e
chaves em `cegas/`, `chaves/`). Régua: `exames/<slug>_regua.txt`. Régua do portão por foto: `exames/basicos_foto.py`
(as contas do `Identidade::de`; a orla em DUAS fases da grelha — o recto lento e o topo da curva em S). Custo:
`exame.sh --custo … 5,40,200`, perfil smoke, load 20–80 (o p95 é o número). Pasta `Digital/Basic/<Hard|Soft|Flat|
Pixel|Glaze>` (rótulos registados).

**Aprovados: Hard Round (curado na 2.ª onda) · Soft Round · Flat Block-in (2.ª onda) · Pixel 1px · Glaze Round.
Reprovado: Round Pressure.**

## Motor — 1.ª onda

`dab/bands.rs`: a `dab_quina` chega ao kernel de faixas do Digital (portão `a_ponta_reta_do_digital_pinta_um_rectangulo`).

## Motor — 2.ª onda (três leis, aditivas, DESLIGADAS por omissão; neutro = o código de antes ao bit)

1. **`TracoComposto::tecto_por_dab`** (`ph2d-painter-brush/src/traco.rs`; arma o `stroke_cover_wanted` em
   `stamp_route.rs`): com o Accumulate desligado cada texel pára na opacidade do dab que o cobre (a da mão), também à
   Strength 1 — a *Opacity* do Krita/PS. Antes os dabs somavam (o Mapping Pen das canetas arma o mesmo tecto com a
   Strength 0,99: esta é a porta sem o número mágico). Portão `o_tecto_por_dab_para_na_opacidade_da_mao` (p 0,4: pico
   0,401 · sem ele 1,000). Mutação: tirar o `|| tecto_por_dab` → 1,000, SANGRA.
2. **`BrushSpec::sub_passo_fino`** (`spec.rs`, `capsula.rs` `sub_passo_fino_px`, `spec/queries.rs`
   `sub_passo_da_capsula`): o piso do sub-passo da cápsula desce de 1 texel a 1/16 — a lâmina de 2 px a 5 px leva as
   mesmas 16 passagens que a de 16 a 40 (o acumular por fracção do diâmetro, a lei do Rebelle). Portão
   `o_sub_passo_fino_acumula_a_lamina_fina_como_a_larga` (5 × 40 px: 0,991 × 0,996 · sem 0,483 × 0,996). Mutação:
   ignorar o campo → 0,483, SANGRA.
3. **`BrushSpec::borda_px`** — a orla em PÍXEIS (`spec/queries.rs` `dureza_do_dab` + `alcance_da_borda`,
   `spec_frame.rs` `at_dab`, `dab/bands.rs`, `stamp_route.rs`): a queda nunca é mais estreita que `b` px na tela e, onde
   a orla fixa é a mais larga, a caixa do AA CENTRA-SE na borda (o dab cresce `b/2`, até 1 texel); sob o tecto do traço
   a orla é ÁREA (o tecto da Smooth Edges, BUGS #16) e, na cápsula, o MAIOR sub-dab (a união enchia-a). Fora da cache
   de máscara e do device (a dureza é de cada dab). Portão `a_orla_em_pixeis_e_a_mesma_a_40_e_a_200` (orla 1,0 × 1,0 px,
   a tinta da linha de 5 px 4,85 = a área; o Hard Round da 1.ª onda 0,5 × 2,0 e 4,00). Mutações, todas SANGRAM:
   dureza sem a orla (tinta 6,00) · união em vez do maior sub-dab (orla 0,5) · sem centrar (tinta 3,97) · sem o tecto
   de área (orla 0,5 × 0,7).

As três só no Digital (`authored_spec` zera-as nos outros meios; `leis_do_digital` na aguada e no relevo, que leem o
pincel cru); no preset só gravadas ligadas (`json.rs`/`json/meio.rs`; censo `CAMPOS_FORA` com `DIGITAL`). Verdes por
nome: `os_outros_meios_nao_mudam`, `presets_de_fabrica::` (13), `presets_dos_meios::`, `a_ponta_de_pixel::`,
`as_formas::`, `wetpaint_preset`, `stamp_device`, e no `ph2d-painter-brush` `capsula`/`spec`/`stroke_cover`/`dab`/
`traco`/`falloff`. Sem UI ainda (só pelo preset).

## A-1 Hard Round — APROVADO (curado na 2.ª onda)

- Receita (Digital): `hardness 1` + **`borda_px 1`** (a caixa de 1 px centrada na borda), cápsula, `pressureSize 0,98`
  curva **1,4**, opacidade fixa; **84 px** = o preset do Rebelle (83,87): com a orla centrada a largura casa a 40 e a
  200 à escala 1 (a 1.ª onda precisava de 1,025 e falhava a 200).
- Oráculo Rebelle `PENCIL/Digital/Round Hard`. Régua **450/454** (a ponta do fim da rampa; o perfil do recto de 5 px).
  Origem 1.ª da matriz a **0,140** (era 0,198). Vizinho: Soft Pastel **0,723**.
- O aro a 200: 0,15 · 0,59 · 0,91 (2–3 px) → um píxel, como o Rebelle (0,22 → 0,95).
- Revisão cega 2.ª onda: r1 escolheu o NOSSO como original nas 6 linhas (na espiral/«sobre» achou o oráculo «não
  claramente outro»); listou a orla mais macia (era 1,3 px) e o terço fraco da rampa mais gordo. r2 achou o oráculo em
  todas, só com zoom: o terço fraco da rampa (curado: curva 1,4, a largura casa coluna a coluna 2·2·4·6·8·10·12·14) e a
  linha de 5 px. **r3** (final): *«ORIGINAL: o nosso nas 6 linhas — confiança baixa … diferença média de 2–3 em 255 …
  quase uma moeda ao ar»*; sobra no limite: o Rebelle 0,3–0,5 px mais largo (o 5 px dele pinta 6), as pontas 2 px.
- Custo p95 (final): 0,12 · 0,59 · 1,10 ms; RSS 23–25 MiB (load 36–46).

## A-2 Soft Round — APROVADO (1.ª onda)

`hardness 0`, queda `Sharp`, `pressureOpacity 1` curva 2,6, tecto da mão 0,8, 163 px. Rebelle `Round Soft`, régua
448/454, vizinho Graphite 8B 0,918. Revisão cega r2: *«Não consegui separar o ORIGINAL da IMITAÇÃO»*. p95 0,17 · 0,27 ·
1,74 ms.

## A-4 Flat Block-in — APROVADO (2.ª onda)

- Receita (Digital): a lâmina FIXA do Rebelle (`dab_flatten 0,6` = T 0,4·D, `dab_angle_deg 90`, `dab_quina 0,9`),
  `hardness 0,8` (a queda relativa dele: 2 px a 40, 7 a 200), **`borda_px 1`** (o AA de área a 5 px), Accumulate a
  `flow 0,28`, cápsula + **`sub_passo_fino`**; sem pressão; **35 px** = o preset do Rebelle (35,1) × 0,995.
- Oráculo Rebelle `PENCIL/Digital/Flat` (escala 0,995). Régua **452/454** (o cv do recto de 5 px). Origem a **0,210**.
  Vizinho: Pixel 1px 0,664 (família, degrau ≥ 0,40); fora: Mapping Pen **1,063**.
- A 5 px: 0,48 → 1,00 com 0,42 de AA nas orlas (Rebelle 0,95, 0,75). O esbatido do fim a 200 px já casava (0,1→0,9 em
  25 px nos dois) — a queixa da 1.ª onda era da receita anterior.
- Razão de uso: bloquear massas com a lâmina fixa — larga no horizontal, fina no vertical, pontas quadradas que esbatem.
- Revisão cega: r1 escolheu o nosso como original nas 6 linhas e listou a linha de 5 px mais leve (curado: orla em px),
  r2 achou o oráculo só com zoom (o nosso 1 px mais grosso — curado: a orla só centra onde é a mais larga), r3 de novo o
  nosso nas 6, lendo a borda do nosso «mais seca» (curado: dureza 0,85 → 0,8). **r4** (final) achou o oráculo nas 6
  pela assimetria da borda dele (o registo de meio píxel da bancada, fila 3) — *«os dois são quase o mesmo pincel, e só a
  medição das bordas os separa»*: o nosso 0,4 px mais estreito, o vertical 1 px mais fino e de ponta mais quadrada.
  Medido e recusado: quina 0,8/0,7 arredonda a ponta mas põe vieiras de carimbo nos rectos (régua 438 e 430/454).
- Portão: `cada_basico_se_reconhece_pela_foto` com `M::Chato` (recto ÷ vertical 2,375 × foto 2,294). Mutação: regerar o
  preset com `dab_flatten 0` → Chato 1,0 e Borda 0 fora, «pinta o irmão» Pixel 1px — SANGRA (preset reposto).

## A-5 Pixel 1px — APROVADO (1.ª onda)

Ponta Pixel-perfect (`pixel 2`), 1 px. MyPaint `1pixel` (e Krita Pixel Art). Portão
`o_pixel_1px_e_o_unico_que_pinta_o_pixel_exacto` (0 parciais, 460/460 colunas). p95 0,16 · 0,24 · 0,49 ms.

## A-6 Glaze Round — APROVADO (1.ª onda)

Dabs a `flow 0,03` que somam, `hardness 0,8`, tecto do traço 0,37 (L8), 14 px. MyPaint `detail_brush_large_glazing`,
régua 444/452, vizinho Wet Wash 1,188. r2: *«Não consigo separar o original da imitação»*. p95 0,19 · 0,53 · 2,79 ms.

## A-3 Round Pressure — REPROVADO (2.ª onda, de novo)

O motor que ele pedia existe e mede a lei do Krita `b) Basic-5 Size Opacity`: com `tecto_por_dab` a rampa sobe desde o
1.º píxel sem somar (0,02 · 0,07 · 0,13 · 0,19 contra 0,02 · 0,06 · 0,10 · 0,15; a 1.ª onda 0,00 · 0,01 · 0,04 · 0,06).
Régua 448/454. Reprova por: (1) **não é distinto** — a matriz põe-no a 0,286 do Hard Round (degrau < 0,40) e a 0,30 do
Studio Pen das canetas, e o revisor cego (r1) não separou o Hard Round dele nos tamanhos e no «sobre» («o terceiro não é
claramente outro»); (2) a curva de potência + tecto da mão dá um JOELHO aos 0,86 de pressão que o Krita (curva em S)
não tem — o revisor achou o Krita e listou-o em primeiro. Saiu da receita e da fábrica. Para voltar: a curva em S da mão
(o agente das canetas) e uma decisão de produto sobre a série Hard Round ↔ Round Pressure.

## Custo (2.ª onda, `exame.sh --custo`, smoke)

Finais: Hard Round p95 0,12 · 0,59 · 1,10 ms (RSS 23–25 MiB, load 36–46) · Flat Block-in 0,37 · 0,53 · 0,94 ms
(23–24 MiB, load 26–33). Round Pressure (antes de sair) 0,16 · 0,71 · 3,28 ms. As três leis só custam onde ligadas (a
orla e o tecto por dab tiram o pincel da cache de máscara: a rota por píxel já servia a cápsula).

## Para a fila (coordenador)

1. A curva em S da mão (G-pen, Basic-5) — destrava o Round Pressure, com a decisão da série.
2. As três leis novas não têm linha no painel (só o preset as liga): «Edge (px)», «Fine sub-step», «Opacity per dab».
3. O registo de meio píxel entre a bancada e o Rebelle (os revisores viram o traço todo 0,4–0,5 px deslocado) — é da
   bancada/entrada, não de um pincel.
