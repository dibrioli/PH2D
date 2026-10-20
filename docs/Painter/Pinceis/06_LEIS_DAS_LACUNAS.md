# 06 — As LEIS medidas das lacunas L3 · L7 · L4 · L11 · L5+L12 (+ L6 · L8 · L19)

> JANELA 2, agente medidor, 2026-10-20. Só fotos que JÁ existem (`fixtures/`: Rebelle 281, ArtRage 35, MyPaint 196,
> Krita 144), nenhum oráculo corrido. Régua: [`ferramentas/leis_lacunas.py`](ferramentas/leis_lacunas.py) (um sub-comando
> por lacuna; a MESMA conta serve para medir a nossa pintura no portão). Cada número diz preset · cena · gesto.
> D = o diâmetro pedido (40 px na cena `gestos`). Ruído = o do oráculo (`04_BANCADA` §4(c)), escrito ao lado.

## §0 — As leis numa frase

| lacuna | lei medida | portão (valor · tolerância = ruído) |
|---|---|---|
| **L3** | Não há lâmina que gira com o arrasto no Rebelle: **FOLLOW_TRAJECTORY = largura constante** (46/47 presets, sd 0,6 px em todas as direcções, sem atraso medível: S nos picos de curvatura ÷ recto = 0,98–1,03, λ ≤ ~0,3·D); **PEN_ROTATION/PEN_TILT = ponta FIXA na tela** (26 presets): w(θ) = L·\|sin(θ−α)\| + T·\|cos(θ−α)\|, rms 0,8–2,4 px; caneta sem rotação ⇒ α ≈ 90° (ROTATION) ou ≈ 0° (TILT). Flat Oily: α 86°, L 39, T 8 (0,98·D × 0,2·D) | `a_lamina_fixa_segue_a_projeccao` (w0, w90, S±40 ± 2 px) · `o_seguidor_e_largo_em_qualquer_direccao` (w(θ) sd ≤ 1 px; arranque da vertical ≥ 0,97) |
| **L7** | Rebelle Effect = **fórmula canónica W3C em sRGB codificado**, misturada a α por passagem (0,056 Color/Hue/Lum/Sat · 0,088 Burn/Dodge · 0,216 Overlay); **no papel sem tinta o efeito pousa em NORMAL com o MESMO α**. Erro ≤ 0,5/255 em Color, Additive, Overlay; Linear Burn 5/255. Krita: Color, Multiply, Overlay, Lighten, Color Dodge com erro ≤ 0,9/255 a α ≈ 0,90 | `o_efeito_do_pincel_compoe_no_seu_modo`: \|nosso − fórmula(base, azul, α)\| ≤ 2/255 por canal, α_papel = α_tinta ± 0,01 |
| **L4** | Spatter do Rebelle (Splats/Star): dabs **isolados**, 5–8 por 100 px, dispersão ⊥ sd 0,61–0,92·D ≈ 3× a dispersão ao longo; diâmetro cv 0,31–0,61 (p90/p10 2,2–5,0) e ×2,8 da pressão < 0,4 à > 0,7; opacidade por dab cv 0,08–0,20 (média 0,33–0,37) | `o_jitter_espalha_como_o_spatter` (⊥ sd 0,6–0,9·D, diâm cv 0,3–0,6, opac cv ≤ 0,2; média de cobertura preservada ± 0,023) |
| **L11** | Rebelle: o ruído de cor é **POR TRAÇO**, nunca por dab: Bristle Ribbon ΔH −15…+74° entre 8 traços (sd 34°), V 0,53–0,95, dentro do traço 0° ao longo e 0° através; nenhuma cor muda com a pressão (0/281 visível). Por dab e pela mão só o MyPaint: puantilism2 ΔH p10–p90 ±55°/dab; long_grass S 0,29→0,61 e V 0,62→0,91 da pressão 0 a 0,9 (matiz ± 3°) | `a_cor_por_traco_fica_no_traco` (p90−p10 do ΔH ao longo ≤ 2°) · `a_pressao_leva_a_cor` (curva por decil ± 0,05) |
| **L5** | Rebelle a pressão constante **não afila o fim**: cauda ≈ cabeça ≈ tampa redonda (p50 0,07–0,23·D) em 9 de 10 famílias; excepções: Knife Dry 0,65·D × cabeça 0,07·D, Knife Dry 2 0,72 × 0,05; Ribbon/Stripes = envelope nas DUAS pontas (6,6–7,3·D). Fall-off: no recto de 880 px a cobertura do fim ÷ início tem p50 0,96–1,00; só 5 de 228 caem ≥ 25 % (Charcoal Wide 0,63, Flat Filbert Dry 0,75, os 2 «ribbon») — e `paint_fadeout_length` = 1 nos 281 (o parâmetro não os distingue) | `a_cauda_redonda_e_a_da_cabeca` (\|cauda − cabeça\| ≤ 0,15·D) · `o_fall_off_esmaece_ao_longo` (Charcoal Wide fim 0,63 ± 0,05) |
| **L12** | Rebelle **não responde à velocidade** (vertical 2500 ÷ 300 px/s: largura p10–p90 0,96–1,03, cobertura 0,96–1,06, 221 presets). Krita/MyPaint: a velocidade só ESTREITA: Ink-2 Fineliner 0,85–0,90 · Ink-1 Precision 0,93 · MyPaint pen 0,74–0,78 · pen-note 0,70–0,75 · Thin Pen 0,80 (cobertura 0,64) · Marker 0,37–0,42 | `a_velocidade_estreita_como_o_fineliner` (0,85–0,90 ± 0,04) e o neutro (1,00 ± 0,04) |

## §1 — L3: a ponta que gira — o que o Rebelle faz de facto

**Oráculos:** Rebelle `OIL_AND_ACRYLIC/flat_flat_oily` · `knife_knife_smooth` · `flat_flat_dry` + os 165 presets com
`brush_rotation_mode` ≠ NONE (cena `gestos`, foto A) · MyPaint `classic/knife`, `classic/calligraphy`,
`deevad/basic_digital_knife` · Krita `f) Bristles-2 Flat Rough`. Comando: `leis_lacunas.py lamina <png>`.

⛔ **A premissa do doc 53 §4 («a lâmina gira com o arrasto; recto fino em qualquer direcção, curva larga») é REFUTADA
pela bateria.** O Flat Oily é uma ponta FIXA na tela, larga no horizontal e fina no vertical:

| Flat Oily (Rb) | recto 0° | rápido 0° | vertical 90° | vert. rápida | S +40° | S −40° | ajuste fixo | constante |
|---|---|---|---|---|---|---|---|---|
| largura (px) | 40 | 40 | 11 | 11 | 35 | 35 | α 86°, L 39,3, T 7,8, **rms 0,8** | rms 9,7 |

Com a espiral, w(θ) por 15° = 40 · 36 · 35 · 30 · 23 · 15 · 11 · 23 · 30 · 35 · 36 · 36 (0°→180°); ajuste com atraso
(β′ = (θ − β)/λ, λ 0–320 px): melhor λ finito rms 8,5 × fixo 1,8 ⇒ **dentro do traço a ponta não roda**.

**A tabela cruzada** (Rebelle, foto A, sem as fotos velhas do §7; FIXA = ajuste fixo ≤ ⅓ do rms da constante):

| modo do preset | FIXA | CONSTANTE | outro (textura/scatter: régua não decide) |
|---|---|---|---|
| FOLLOW_TRAJECTORY (force True) | 1 (Express Flat Knife Soft) | **46** | 40 |
| PEN_ROTATION | **17** | 5 (pontas redondas) | 17 |
| PEN_TILT | **9** | 1 | 19 |

Eixo longo das 26 FIXAS: PEN_ROTATION 78–100° (Flat 84°, Bristle Wide 96°, Knife Textured 86°); PEN_TILT 170–10°
(Knife Dry 170°, Knife Soft 178°, Marker Chisel/Liner 4°). Comprimento 0,85–1,05·D, espessura 0,12–0,35·D.

**O seguidor** (FOLLOW, 62 presets dos 4 oráculos com rms constante ≤ 1,5): largura constante sd p50 0,6 px; S nos
picos de curvatura (κ ≈ 0,016/px) ÷ recto = 0,979 · 1,000 · 1,026 (p10/50/90). Com T ≈ 0 isso limita o atraso a
Δ ≤ 13° ⇒ **λ ≤ ~12 px ≈ 0,3·D** (o Follow de hoje, L = 0,1·D, já cabe). MyPaint `classic/knife` (`direction_filter`
2,99) e Krita Bristles-2 também constantes (40/40 e 35/35 em 0°/90°).

**Memória entre traços:** o arranque da vertical (depois da `rampa_afunila`, horizontal) ÷ a sua mediana = 0,97 · 1,00 ·
1,00 nos seguidores ⇒ o 1.º dab já está alinhado com a direcção inicial, sem herdar o traço anterior. ⚠️ Mas o
Rebelle recebeu `SET_BRUSH` antes de CADA traço (`bateria_rebelle.py`): a memória do Rebelle não é medível nesta bateria.

**A excepção Knife Smooth** (PEN_ROTATION, force False): nem fixa-rectangular nem seguidora — mínimo nos EIXOS e
máximo nas DIAGONAIS: w(θ) = 9 · 18 · 21 · 22 · 20 · 15 · 12 · 21 · 24 · 22 · 19 · 13 (0°→180°, faixas de 15°);
recto 9, vertical 12, S ±40° 21–22. É a pegada de uma ponta não-convexa fixa (o canto «quadrado» vê-se na espiral);
nenhum rectângulo a reproduz (ajuste rms 2,5 × constante 5,1).

**Lei para o motor:** (a) ângulo FIXO por preset (existe: `dab_angle`) com o eixo que o oráculo mostra — o nosso Flat
Oily do P53 estava a 0°, o do Rebelle está a 86°; (b) FOLLOW instantâneo (existe) com λ ≤ 0,3·D; (c) inércia/memória
NÃO têm oráculo que as peça — só se o dono as quiser como produto. **Portão** (`gestos`, 40 px): w0, w90, S±40 de
cada preset FIXO = os do oráculo ± 2 px (rms do ajuste 0,8–1,4 + promessa da largura); seguidor: sd de w(θ) ≤ 1 px e
arranque ≥ 0,97. Ruído do oráculo: recto lento × rápido e vertical × vertical rápida iguais ao px (0 a 1 px).

## §2 — L7: os modos de mistura

**Oráculos:** Rebelle `MARKER/effect_{burn,color,dodge,hue,luminosity,overlay,saturation}` (cena `sobre`, base
Marker Bullet vermelho) e `marker_bullet`; Krita `l) Adjust_{Color,Dodge,Lighten,Multiply,Overlay_Burn}` (base
`b) Basic-1`). Comando: `leis_lacunas.py modos <png> <MODO>`. Conta: resultado = base + α·(fórmula(base, azul) − base),
α por mínimos quadrados; erro = máx \|previsto − foto\| em níveis de 8 bit (ruído: 1 nível de quantização).

| preset (modo do ficheiro) | ΔE na faixa | α papel | α tinta | erro sRGB | erro linear | NORMAL | veredito |
|---|---|---|---|---|---|---|---|
| Rb Effect/Color (COLOR) | 10 | 0,056 | 0,053 | **0,2** | 3,0 | 1,6 | ✓ |
| Rb Effect/Dodge (ADDITIVE) | 15 | 0,088 | 0,085 | **0,4** | 1,9 | 9,8 | ✓ (Screen 1,5) |
| Rb Effect/Overlay (OVERLAY) | 11 | 0,216 | 0,211 | **0,5** | 2,1 | 5,3 | ✓ |
| Rb Effect/Burn (LINEAR_BURN) | 20 | 0,088 | 0,107 | 5,1 | 5,1 | 14,2 | ≈ (Multiply 6,4); o azul cai 39 % e o vermelho 10 % |
| Rb Effect/Hue (HUE) | 10 | 0,056 | 0,055 | 0,9 | 2,0 | 1,3 | ambíguo: Hue = Color sobre vermelho saturado |
| Rb Effect/Luminosity · Saturation | 1,0 · 1,5 | 0,056 · 0,063 | 0,04 · 0,07 | 0,0 · 0,3 | — | 1,2 · 0,6 | **não medível** (ΔE ≤ 1,5 níveis) |
| Rb Marker/Bullet (vê-se MULTIPLY) | 171 | 1,00 | 0,98 | 2,3 | 5,7 | 97 | o marcador é multiplicativo |
| Kr Adjust Color · Multiply · Overlay | 173 · 162 · 47 | 0 · 0,91 · 0 | 0,90 · 0,90 · 0,90 | **0,9 · 0,2 · 0,5** | 56 · 4,8 · 6,5 | — | ✓ |
| Kr Adjust Lighten · Dodge (Lighten · Color Dodge) | 146 · 38 | 0 | 0,91 · 0,90 | **0,4 · 0,0** | 0,4 · 14 | — | ✓ |

**Lei:** sRGB CODIFICADO (o linear erra 2–56 níveis); o Rebelle aplica o modo sobre a tinta e **NORMAL com o mesmo α
sobre o papel** (α papel = α tinta ± 0,005); o Krita aplica-o ao papel branco também (Color/Overlay/Lighten/Dodge sobre
branco = branco). Faixa SECA = MOLHADA ao nível. ⇒ o motor já mistura (03 §1.1); falta o preset com α 0,05–0,22 por
passo. **Não chegam** para Hue/Saturation/Luminosity (vermelho saturado: Hue ≡ Color; Lum/Sat ≤ 1,5 níveis) nem para
Multiply/Screen no Rebelle. Estas pintam (fraco), ao contrário da cena do doc 52 (0 px).

## §3 — L4: o jitter por dab

**Oráculos:** Rebelle `WATERCOLOR/spatter_splats` (scatter 1, size jitter 1), `spatter_splat_2`, `spatter_spatter_2/3`,
`OIL_AND_ACRYLIC/metallic_splats`; Krita `v) Texture_Pointillism`, `y) Texture_Spray`. Comando: `leis_lacunas.py dabs`.
Componentes conexas da cobertura > 0,12 (área ≥ 3 px), atribuídas ao traço mais perto.

| oráculo · traço | n/100 px | ⊥ sd ÷ D | passo média · cv | diâmetro p10/50/90 (px) · cv | opacidade p10/50/90 · cv |
|---|---|---|---|---|---|
| Rb Splats · recto_lento | 5,6 | 0,66 (p90 1,10) | 17,0 · 0,75 | 5,3 / 14,7 / 24,7 · 0,52 | 0,30 / 0,36 / 0,41 · 0,14 |
| Rb Splats · 7 traços | 5,0–8,2 | 0,61–0,92 | — | p50 11–18 · 0,31–0,61 | p50 0,31–0,37 · 0,08–0,20 |
| Rb Splats · rampa_0_1 (pressão < 0,4 × > 0,7) | | | | 7,6 × 21,6 | 0,27 × 0,38 |
| Rb Splat 2 · recto_lento | — | ⊥ 21 px | 9,9 · 1,01 | — | — |
| Kr Pointillism · recto_lento | 2,6 | 0,28 | 42,5 · 0,60 | 3,9 / 30,7 / 48,6 · 0,63 | 0,48 / 0,78 / 0,82 · 0,19 |
| Kr Spray · recto_lento | 41,8 | 1,19 | 3,5 · 0,99 | 2,0 / 2,3 / 4,1 | 0,19 / 0,27 / 0,39 |

**Lei:** (1) dispersão **anisotrópica** no Splats: ⊥ 28 px ≈ 0,7·D, ~3× o ao longo (passo cv 0,75 ⇒ σ∥ ≈ 9 px); o
Spray do Krita é Poisson (passo cv 0,99), ⊥ 1,2·D; (2) **tamanho por dab** largo (p90/p10 4–5 com size jitter 1) e ×2,8
pela pressão; (3) **opacidade por dab** quase fixa (cv ≤ 0,2); (4) 5–8 dabs/100 px a D 40, spacing 0,7·D. Ruído: ⊥
0,61–0,81·D entre os 5 traços horizontais do mesmo preset. Os spatters finos (`spatter_2/3/6`, `splat`, `metallic_splats`:
dabs de 2–3 px a opacidade 0,14–0,22) ficam abaixo da régua — compare-os por textura. **Portão:** o nosso Splats no
`recto_lento` com ⊥ sd 0,6–0,9·D, diâm cv 0,3–0,6, opac cv ≤ 0,2, cobertura média = a sem jitter ± 0,023.

## §4 — L11: a cor por dab, por traço e pela mão

**Oráculos:** Rebelle `OIL_AND_ACRYLIC/bristle_bristle_ribbon` (`bristle_noise` h 0,5 l 0,2), `bristle_bristle_stripes`
(h 0,2 s 0,2 l 0,5), `bristle_bristle_flat` (h 0,08: controlo); MyPaint `classic/puantilism2`, `impressionism`,
`long_grass`; Krita `v) Texture_Impressionism`. Varridos os 602 `gestos` dos 4 oráculos (`leis_lacunas.py cor`):
só 17 mudam de matiz (entre traços > 15°, ao longo ou através > 12°). ΔH = matiz − 210,4° (o azul pedido); só px com tinta e S > 0,2.

| oráculo | ΔH por traço (8 traços) | ao longo p90−p10 | através sd | S · V | pressão |
|---|---|---|---|---|---|
| Rb Bristle Ribbon | +12 −9 −15 +3 −13 +65 +52 +74 | **0°** | 0° | S 0,84–0,88 · V 0,53–0,95 por traço | nada (−13° em todos os decis) |
| Rb Bristle Stripes | +5 +13 −4 −1 +14 +3 +12 −5 | 0–9° | 0–8° | S 0,58–0,93 · V 0,39–0,97 | nada |
| Rb Bristle Flat (controlo) | −3…−1 | 0–1° | 1–2° | constantes | — |
| Mp puantilism2 | p50 +2…+38 | **85–115°** (±55°/dab) | 13–21° | S 0,76–0,95 · V 0,61–0,78 | — |
| Mp impressionism · imp_details | — | 52° · 29° | 32° · 16° | — | — |
| Mp long_grass | −3…+3 | 20° | 1–7° | — | **S 0,29→0,61, V 0,62→0,91** (decil 0 → 0,9) |
| Kr Texture_Impressionism | +12…+18 (desvio fixo) | 13° | 6° | — | nada |

**Lei:** no Rebelle o `bristle_noise` sorteia UMA cor por traço (matiz, valor) e o traço inteiro fica nela — o p90−p10
ao longo é 0° no Ribbon. Por dab (puantilismo) e pela pressão (long_grass escurece e dessatura com a mão leve; não é
a cobertura sobre o branco, que clarearia) só o MyPaint mostra. **Cor secundária (fg/bg): nenhuma foto** — a bateria
tem uma cor só. Portões: `a_cor_por_traco_fica_no_traco` (Ribbon: ao longo ≤ 2°, entre traços sd ≥ 25°) e
`a_pressao_leva_a_cor` (long_grass: S e V por decil ± 0,05 = o ruído entre decis vizinhos).

## §5 — L5 + L12: o fim do traço e a velocidade

**Comando:** `leis_lacunas.py fim <gestos> <tamanhos>` (corrido nos 687 presets). Cauda/cabeça = do último (1.º) ponto
com largura ≥ 0,8·mediana até ao fim (início) da tinta, ÷ D, no `recto_40` a pressão 1. Tampa redonda = 0,2·D.

| grupo | n | cauda ÷ D p50 · p90 | cabeça ÷ D p50 | fall fim÷início p10/50/90 | s50 < 880 px | rápido÷lento larg. · cob. (verticais) |
|---|---|---|---|---|---|---|
| Rb OIL_AND_ACRYLIC | 62 | 0,10 · 0,32 | 0,07 | 0,89 · 0,99 · 1,02 | 4 | **0,96–1,03 · 0,96–1,06** (221 presets) |
| Rb WATERCOLOR | 47 | 0,20 · 0,45 | 0,17 | 0,90 · 0,96 · 1,08 | 3 | |
| Rb PENCIL · PASTEL · INK · MARKER | 89 | 0,07–0,23 | 0,07–0,23 | 0,96–1,00 (p50) | 1 | |
| Krita | 144 | 0,15 · 0,30 | 0,15 | 0,87 · 1,00 · 1,03 | 6 | p10 0,94 · 0,96 (10/113 > 15 %) |
| MyPaint | 196 | 0,15 · 0,60 | 0,10 | 0,72 · 0,99 · 1,11 | 19 | p10 **0,66 · 0,69** (32/141 > 15 %) |
| ArtRage | 35 | 0,07–0,53 | 0,05–0,69 | ≥ 0,94 salvo Oil ThinAndDry (0,00, s50 540 px) | 2 | p10 0,92 |

**Fall-off forte (Rb):** Bristle Stripes 0,18 e Ribbon 0,61 (envelope), Charcoal Wide 0,63 (s50 860 px), Grunge Scatter 2
0,71, Flat Filbert Dry 0,75; ArtRage Oil ThinAndDry acaba aos 540 px. (Gouache Liner 0,36 e WC Bristle Round 0,31 eram
fotos velhas, §7: ⛔ não usar.) **Cauda ≠
cabeça:** Rb Knife Dry 0,65 × 0,07, Knife Dry 2 0,72 × 0,05; ArtRage Pencil 0,50 × 0,05; MyPaint Tail_Feathers 1,68 × 0,38.

**Velocidade** (2500 ÷ 300 px/s nas verticais de 170 px; o recto rápido é mais curto e mistura o comprimento): Krita
Ink-2 Fineliner 39→33 (0,85) · Ink-1 Precision 0,93 · MyPaint pen 0,78 · pen-note 0,75 · Thin_Pen 20→16 e cobertura
0,885→0,57 · slow_ink cobertura 0,90→0,51 · ramon/Marker 0,42. Nenhum alarga. Rebelle: 0 (confirma «Rb 0/281»).

**Portões:** `a_cauda_redonda_e_a_da_cabeca` (\|cauda − cabeça\| ≤ 0,15·D no neutro; Knife Dry 0,65 ± 0,15·D) ·
`o_fall_off_esmaece_ao_longo` (Charcoal Wide: fim 0,63 ± 0,05; Flat Filbert Dry 0,75 ± 0,05) · `a_velocidade_estreita_como_o_fineliner`
(0,85 ± 0,04) e o neutro `sem_velocidade_a_largura_e_a_de_hoje` (1,00 ± 0,04 = p10–p90 do Rebelle).

## §6 — Em uma linha

- **L6 dual brush** — os 11 do Rebelle têm `dual_brush_layer_blend_mode` NORMAL. Charcoal 2 a 200 px: **estrias ao
  longo** (½-autocorrelação 8 px ao longo × 1 px através), cobertura 0,65, resíduo 8,9 %; Dry Thin: grão isótropo de 1–2
  px, cobertura 0,81–0,82 (indistinguível do grão do papel). Todos os carvões e pastéis secos são dual ⇒ não há controlo
  sem dual na mesma família: **foto nova** = o mesmo preset com `dual_brush_enabled false` (§8).
- **L8 tecto por traço** — a espiral a 40 px sobrepõe braços a 36 px DENTRO do traço; k = ln T(sobreposição) ÷ ln T(recto):
  Rebelle Watercolor Round 2 **1,09**, Wash Round 1,11, os 7 Glaze 1,00–1,15, Airbrush Spray 1,06 ⇒ **tecto por traço**;
  Airbrush Spray 2–5 **1,8–4,5** e Marker Dry 2,66 ⇒ acumulam; Krita Marker Medium 1,09, Airbrush Soft 1,04;
  ArtRage Felt Pen 1,03–1,08, Watercolor 1,88. (Os opacos saturam a 0,91 = o azul: k = 1 não informa.)
- **L19 image hose** — Krita `z) Stamp_*` pinta carimbos de forma variada (Leaves: folhas de 6–10 px, opacidade pela
  pressão) e os RGBA trazem matiz própria (Stamp_Hearts +30°, Shoujo +22°, RGBA_04 −10°); o sorteio/sequência de N
  imagens não se mede com dabs sobrepostos ⇒ foto nova com espaçamento ≥ 1,5·D.

## §7 — ⚠️ Achado: 29 fotos do Rebelle são a foto ANTERIOR (cena que não repintou)

`leis_lacunas.py velhas fixtures/rebelle_bateria` compara cada foto com a anterior (ordem do `log.csv`). Iguais ao byte,
com preset ou cena DIFERENTE (a mais tardia é a velha): `dabber_dabber` gestos = Bristle Wide (vê-se uma trincha chata:
**a matriz leu-a como Dabber**) · `grunge_scatter` gestos · Express `grunge_grunge_3` · `texture_fragments` · WC
`bristle_bristle_soft` · `spatter_spatter_5` · 15 cenas `tamanhos`/`sobre` (Knife Dry sobre, Knife Oily tamanhos, Scratch
Thin, Charcoal 3, Pen Fountain Round, WC Bristle Wide/Round/Soft, Gouache Fan Line/Filbert/Liner/Wash, Spatter 4, Spatter
Star, Watercolor Flat 2) · 2 do Blend = **23**. Quase iguais (Δ ≤ 12): Gouache Filbert 2 sobre, Watercolor Round 3 sobre,
Spatter 6 tamanhos, Spatter Splat gestos, Spatter Star gestos, Sumi-e Soft gestos = **6** (por decidir: Graphite HB ← H
Δ 15, Gouache Filbert 2 ← Filbert Δ 14). ⇒ **re-fotografar essas 29 cenas**; as leis acima não as usam.

## §8 — O que as fotos NÃO deixam medir (e a foto nova que o mede)

| o que falta | foto nova (preset · cena · gesto) |
|---|---|
| L3 memória (Rebelle) | Flat Dry, Flat Oily: horizontal e depois vertical SEM `SET_BRUSH` entre os dois |
| L3 atraso | Flat Dry e MyPaint knife a 20 px num gesto novo `canto_90` (recto → vertical) e num zigue-zague de 60 px |
| L7 Hue/Sat/Lum | os 7 Effect sobre base CINZA 50 % e PASTEL (S 0,3), 10 passagens; Multiply/Screen só no Krita |
| L4 contagem/eixo | Splats com spacing 3·D: a nuvem 2-D de cada ponto de emissão |
| L11 secundária | Krita com cor de fundo ≠ da frente; o Ribbon em 20 traços iguais (a distribuição do sorteio) |
| L12 curva | rectos a 3 · 6 · 12 · 25 · 50 px/evento no Krita Fineliner e no MyPaint pen |
| L6 2.ª ponta | Charcoal 2 e Dry Thin com `dual_brush_enabled false` |
| L19 N imagens | Krita Stamp_Leaves/Herbals com espaçamento 1,5·D |

## §9 — Como refazer

`python3 -I ferramentas/leis_lacunas.py <lamina|modos|dabs|cor|fim|tecto|textura|velhas> <png…>` (a docstring diz os
argumentos), da pasta `docs/Painter/Pinceis`; sobre a pintura da bancada (`/tmp/ph2d_exame/…`) dá o lado «nosso» de
cada portão. Os modos dos presets do Rebelle (rotação, `splat_*`, `bristle_noise`, `dual_brush_*`, `glaze_rgb_blend_mode`)
vieram de `NewFeaturesWetPaint/fixtures/rebelle_presets_281.json` só para ESCOLHER os oráculos; a lei é a da foto.

## ⛔ Recusas MEDIDAS

| proposta | porque não |
|---|---|
| «a lâmina do Flat Oily gira com o arrasto, com inércia» (53 §4, L3/C38) | Flat Oily = ponta fixa a 86°, rms 0,8 px; o melhor atraso finito dá rms 8,5 (o nosso do P53 estava fixo a 0°) |
| inércia de rotação como lei do seguidor | 62 seguidores sem atraso visível: λ ≤ 0,3·D |
| velocidade→largura no Rebelle | 221 presets: 0,96–1,03 (ruído) |
| `paint_fadeout_length` como chave do fall-off | vale 1 nos 281; o esmaecer medido varia de 0,18 a 1,08 |
| compor os modos em luz linear | erro 2–56 níveis contra 0,0–0,9 em sRGB codificado |
