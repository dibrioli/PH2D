# 01 — O que o Rebelle tem e nós não, porque o traço dele é superior, e o plano para o alcançar

> **Leitor:** a próxima LLM / engenheiro que vai implementar. Técnico, com endereços. O resumo para o dono
> está no fim do relatório da sessão, não aqui.
>
> **Como se chegou a isto (método, [00_o_metodo](../../_ComoInvestigarApps/00_o_metodo.md)):** (1) inventário
> do nosso Painter (código, `file:line`); (2) o manual do Rebelle 8.3.4 instalado (81 páginas, lidas por
> subagente); (3) os **dados** dos 281 presets de fábrica (doc [02](02_presets_para_wet_painter.md));
> (4) o Rebelle **corrido** pela porta oficial *WebSocket Control* (`-websocket-server-enable`, eventos
> Motion IO `POINTER_PRESS/MOVE/RELEASE` com pressão e inclinação) sobre **os mesmos cinco gestos** que a
> nossa sonda pinta nos quatro meios, fotografado e **medido com a mesma régua** (doc
> [03](03_tracos_medidos_rebelle_x_ph2d.md)). Nada do binário foi lido; as texturas dele não se copiam;
> a mistura por pigmentos dele é Mixbox (CC BY-NC) e não é porta.

---

## §0 — Resumo (uma tela)

**O traço do Rebelle é superior por sete leis, e nenhuma delas é «uma textura melhor»:**

1. **Fios, não carimbos** (Bristle Brushes, motor de partículas): cada fio tem a sua pressão, largura
   ondulante e cor com ruído; metade dos fios *constrói* relevo e a outra *escava*. Medido: óleo com
   textura **5,9 %** e uniformidade **CV 0,011** ao mesmo tempo (o nosso Impasto: 1,8 % / 0,007 — liso como
   plástico; o nosso Wet Paint: 14 % mas em **carris paralelos que não variam**).
2. **Esborrato entre carimbos** (`splat_smudge` + `smudge_density`, em 112/281 presets): mata o «colar de
   contas». Medido: o nosso Impasto tem um período de carimbo de **5,6 px a 42 % da potência** do perfil;
   o óleo dele, 26 px a 9 %.
3. **Forma × grão sequenciados pela pressão/inclinação** (até 4 pares, `texture_order=PRESSURE` em 44/62
   óleos): pouca pressão = cerdas secas, muita = cheio. O mesmo pincel é dois pincéis.
4. **A lei do papel é uma curva da pressão** (`paper_structure_influence_curve (0,0,7)→(1,0)` em quase
   todos): pouca pressão deixa o papel furar, muita cobre; + contraste + «raspar claros/escuros» + uma
   **textura de fundo** que fura nos vales (`HEIGHT`/`INV_MULTIPLY`). Nós: um `paper_depth` constante.
5. **A mistura é uma lei da pressão e da sua QUEDA** (Paint & Mix: enquanto a pressão sobe pinta; quando
   cai mais do que o limiar do máximo do traço, mistura), com **tinta finita** (`Loading` + `Length`) e
   **pincel sujo** entre traços. Nós: `wet_charge` constante, só na aquarela, sem limiar, sem fim de tinta.
6. **Inclinação com duas curvas** (altura E largura do carimbo) e **rotação da caneta**: o lápis deitado a
   55° cresce; a espátula estreita com a inclinação; o flat de lado é uma linha. Medido: `penciltilt`
   37 px contra 30 px direito, e mais liso (textura 5,2 % contra 9,1 %), com a mesma pressão.
7. **Água como volume que se espalha e seca** com `Absorbency/Re-wet/Edge darkening/Diffusion speed/Drips`
   por papel: a aquarela a opacidade 50 seca com a borda mais escura (`borda 1,04`) e orla de 1 px; a
   água 80 torna-a transparente (cobertura 0,37) e o cruzamento vermelho vira roxo (mistura molhada).
   ⚠️ A largura **não** muda com a água (31 px nos três gestos). Nós temos as peças (aguada óptica +
   fluido) mas em dois meios separados e sem a água como volume do pincel.

**O que falta, por família (os 10 que mudam de patamar):** presets em ficheiro · esborrato entre carimbos ·
pares forma/grão por pressão · lei do papel por pressão + textura de fundo · Paint & Mix/Blend por curvas +
tinta finita + pincel sujo · inclinação em 2 eixos + rotação da caneta · motor de fios · pincel duplo ·
painel de volumes (5 sliders) · papéis como height-map com comportamento próprio. Ondas em §4.

---

## §1 — Inventário paralelo

Status PH2D: ✅ tem · ◐ parcial · ❌ não tem. Endereços do nosso lado: `crates/ph2d-painter-brush/src/spec.rs`
(`BrushSpec`), `stroke.rs`, `dynamics.rs`, `texture.rs`; `crates/ph2d-tool-painter/src/tool/paint/media.rs`
(`PaintMedia`), `watercolor_*.rs`, `impasto_*.rs`; `crates/ph2d-wet-paint/`; `crates/ph2d-panel-painter-layers/`.

| família | Rebelle 8 Pro | PH2D |
|---|---|---|
| **Mídias** | 8 ferramentas de tinta (Oils & Acrylics, Express Oils, Watercolors, Inks, Pencils, Pastels, Markers, Airbrushes) + Erase/Blend/Smudge/Clone/Water/Dry/Blow — **um esquema de pincel para todas** | 4 meios mutuamente exclusivos (Digital · Watercolor óptica · Impasto · WetPaint fluido) + 12 modos (Paint, Smear, Blur, Clone, Fill, Mask, Inpaint, Selection, Deform, Sculpt, Knife, WetPaint) ◐ — lápis/pastel/tinta/marcador/airbrush são **Digital com outros valores**, sem lei própria ❌ |
| **Motor de carimbo** | forma × grão (4 pares) + sequência (seq/random/pressão/tilt) + fundo 1/2 + dual brush + shape border + smudge entre carimbos + AA opcional | 1 forma + 1 grão (`shape`, `texture`), 9 falloffs + curva, `dab_flatten/angle`, `jitter_*`, `spray_count`, `symmetry`, linhas procedurais (Sketchy/Wire/Ribbon/Rough/Dash) ✅ — sequência ❌ fundo ❌ dual ❌ smudge entre carimbos ❌ |
| **Motor de fios** | Bristle Brushes (partículas; Thin/Thick/Edges; Add/Remove impasto; Perlin por fio; Brush Form; oscilação; cor por fio) | `ph2d-wet-paint` tem `Bristle Size/Count` (fios Poisson só no fluido) ◐; Digital/Impasto/Aquarela por carimbo ❌ |
| **Dinâmicas** | pressão→size/opacity (curvas, piso, tecto, ganho), tilt→altura e largura (2 curvas), pen rotation, follow trajectory, pressure smoothing, position smoothing (média móvel / corda puxada), **sem velocidade** (só *Smudge on Paper Bumps*) | pressão/tilt/velocidade→size/opacity (4 curvas) ✅, `stabilizer`, `input_samples`, `taper` (nós temos; ele não), heading/rake ✅ — tilt-largura ❌, pen rotation ❌, pressure smoothing ❌, corda ❌ |
| **Mistura** | Paint / Paint & Mix / Paint & Blend / Blend / Erase por curvas de pressão + limiar da queda; Loading + Length (tinta finita); Dirty Brush, MultiColored; Oiliness; Average smudge color (por fio) | `BrushBlend` 24 modos ✅, K–M `pigment` ✅, `wet_charge/pull/dilution/smudge/rewet` (aquarela) ◐, Smear/Blend/Knife ✅ — lei por pressão ❌ tinta finita ❌ sujo entre traços ❌ |
| **Papel** | papéis-loja como height-map 256² (texture scale 25–400 %, deckled edges), Paper Texture Strength (curva)/Contrast/Scratch Auto·Light·Dark, Legacy mode, grão com DPI, **comportamento por papel** (Absorbency, Re-wet, Texture influence, Edge darkening, Diffusion speed, Drips, Granulation strength/density/textura) | papel do documento (cor, procedural/imagem, `paper_depth`, `paper_edge`, `granulation`), `Paper Gate` no fluido, vidro K–M ✅ — curva por pressão ❌ contraste ❌ scratch ❌ DPI ❌ presets de comportamento por papel ❌ (há `visual_settings`-like só nos knobs do fluido) |
| **Água** | *Water* como volume do pincel (0–100) em watercolor/ink/acrylic; Water/Dry/Blow tools; Wet the Layer / Wet All Visible / Dry / Fast Dry / Pause; Tilt panel + acelerómetro; drips | WetPaint com 6 sub-ferramentas (Paint·Smear·Blend·Wet·Dry·Blow(def.)) ✅, Dry Time, Rewet ✅ — água como volume na AQUARELA ❌, Blow ❌, tilt dial ◐ |
| **Relevo/luz** | Impasto Depth/Gloss por layer, Max Impasto Height/Smudge/Thick por pincel, **RealShader** (env-maps), **SoftShadows** (sombras), Shadow altitude, Metallic (reflectivity map), Heightmap EXR export | height field + N·L + especular + wrap (`impasto_*` 13 campos), 10 luzes ✅ — env-map ❌ sombras projectadas ❌ (doc 27 já o planeia) metálico ◐ (`impasto_metallic` é cor do highlight) |
| **Cor** | roda HSV/HSL/HSLuv/RGB + **RYB**, harmonias, gradiente, HEX, color sets (pigmentos nomeados), Mixing Palette, Pigments (Mixbox, Pro), Natural Colors (RYB), Color Noise per bristle/stamp/stroke | RGB sliders + swatch, `color_jitter` por carimbo, K–M próprio ✅ — roda/harmonias/paleta de mistura ❌ ruído por fio/traço ❌ |
| **Presets** | 329 de fábrica, 1 ficheiro cada (PNG + JSON + texturas), grupos, import/export/partilha/favoritos/recentes, Volume Presets, Save Volumes policy, Reset to default | só snapshots em memória por modo ❌ (doc 02 §5) |
| **UI** | Properties (5 volumes + modos), Brush Creator (Stroke / Shape & Grain / Paint), Brushes panel, Visual Settings, Tilt, Navigator, Layers (wet/dry/pigment/metallic), Assets | secções Brush/Stroke/Dynamics/Jitter/Line/Shape/Grain/Watercolor/Impasto/WetPaint (censo doc 45) ✅ — painel de volumes ❌ criador separado ❌ |
| **Ferramentas** | Fill (color/gradient/pattern, tolerance, wet), Pick (mix), Selection (6 modos, feather, roughness, quick fill), Transform (deform/perspective/warp, fractal), Liquify 7 modos, Clone, Rulers/Perspective/Symmetry, Stencils/Structures/Paths, Tracing, Masking fluid, Filter layers, Record time-lapse, Reference images | Fill, Mask, Inpaint, Selection (4 modos), Deform (Push), Sculpt, Clone ◐, symmetry ✅ — réguas/perspectiva ❌ stencils ◐ tracing ❌ masking fluid ❌ time-lapse ❌ |
| **I/O** | REB, PSD (Pro), NanoPixel export ×20, EXR heightmap, WebSocket control, Photoshop plug-in | `.ph2dproj`, PNG… — WebSocket/automação ❌ (a porta que nos faltou para o smoke!) |

---

## §2 — Porque o traço dele é superior: as leis, com a medição

A régua é [`ferramentas/mede_traco.py`](ferramentas/mede_traco.py) (largura mediana, taper 5/50/95 %,
orla 10→90 %, CV ao longo do traço, período dominante do perfil e a sua fracção, textura = desvio do
resíduo 5×5 no miolo, borda/miolo, cobertura). Os gestos são os da sonda. Tabela completa no doc 03;
aqui só o que decide.

### L1 — Fios individuais com física própria (o que faz «óleo» parecer óleo)

Rebelle Bristle Flat (40 px): `textura 5,9 % · CV 0,011 · orla 2 px`. Olhando: 20–30 riscos paralelos de
largura e brilho variáveis, cristas e sulcos (Add/Remove impasto), e o RealShader a iluminar cada crista.
PH2D Impasto (mesma largura): `textura 1,8 % · CV 0,007 · orla 3 px` — um tubo liso com um especular
único. PH2D Wet Paint: `textura 14 %` mas o resíduo é **periódico** (carris) — os fios nossos são pontos
Poisson fixos que arrastam a mesma lane durante o traço inteiro; os dele oscilam (`bristles_oscillation`
0…12 px), têm largura ondulante (`thick_wave_length` 400–1000 px, `jitter` 0,2–0,96), pressão própria
(Perlin `scale` 0,75–2 + `random_ratio`), e `Variation` baralha quem constrói e quem escava.
**Mecanismo:** doc 02 §2.4. **A nossa lacuna:** não há motor de fios fora do fluido.

### L2 — O esborrato entre carimbos

`splat_smudge=true` em todos os óleos e metade das aquarelas: cada carimbo é esborratado contra o
anterior com densidade 0,9–9,9 (90 = marcas ≤ 1 px). Medido no perfil ao longo do traço: PH2D Impasto
**período 5,6 px a 42 %** da potência (o carimbo vê-se), Digital 28 px a 13 %; Rebelle óleo 25,8 px a 9 %,
aquarela 7 px a 23 % (mas com `textura 1 %` — é o grão, não o carimbo). ⚠️ O Express *Hairy* dele dá 6 px a
**47 %**: cerdas desenhadas na FORMA do carimbo (sem fios) também mostram o carimbo — o smudge e os fios é que
o apagam. **Mecanismo plausível (a
confirmar na bancada, red-first):** depois de carimbar, um *smear* de raio `density` do carimbo n−1 para
o n ao longo da tangente — é o que o nosso `smear_field.rs` já sabe fazer **entre eventos**, ainda não
entre carimbos.

### L3 — Pares forma/grão sequenciados pela pressão ou inclinação

`texture_order=PRESSURE` com faixas (`texture_0_end 25`, `texture_1 25–50`, …): a pressão escolhe o par e
**mistura** pares vizinhos nas zonas de transição. Óleo *Round Hard*: par 1 até 50 %, par 2 de 50–62 %,
par 3 a partir de 75 %. É assim que um traço leve é «seco» e um pesado «cheio» sem trocar de pincel.
Nós: um par só. A rampa de pressão do Rebelle muda de textura ao longo do traço (fotografado: óleo e
express); a nossa só muda de largura.

### L4 — A lei do papel

Três controlos que não temos: (a) **força do papel em função da pressão** (curva, cai para 0 com pressão
máxima em 258/281 presets — a tinta cobre o grão quando se carrega); (b) **contraste** da textura; (c)
**raspar claros ou escuros** (`AUTO/LIGHT`). E a **textura de fundo** (`background_texture`, 86 presets)
que fura o carimbo nos **vales** do height-map (modo `INV_MULTIPLY` nos lápis — vales brancos; `HEIGHT`
nos óleos — o fundo é relevo). Medido: lápis 8B do Rebelle `textura 9,1 %` com `borda/miolo 0,32` (o grão
abre nas bordas, fecha no miolo); o nosso Digital `0,7 %` (não há papel no Digital) e a aquarela nossa
`6,9 %` mas uniforme (o `paper_depth` é constante).

### L5 — A mistura como lei da pressão, da sua queda, da tinta que acaba e do pincel sujo

Paint & Mix (136/281): enquanto a pressão sobe só pinta; quando desce `paint_mix_threshold` (70–100 %)
do máximo do traço entra a mistura, com `paint_mix_curve_2` a dizer quanto tempo a cor carregada
persiste. `apply_loading_to_paint` (168/281) + `Length`: a tinta ESGOTA-SE ao longo do traço e o pincel
passa a Blend (preferência *Automatically switch to Blend when the brush runs out of paint*). Dirty Brush
guarda a cor apanhada ENTRE traços. ⚠️ **Ainda NÃO medido:** o gesto de cruzamento da bancada é um flat
de lado (4 px) e não distingue `oildirty` de `oil` (doc 03 §3.8); a lei vem do manual e de 136/281 presets
em Paint & Mix — a bancada que a mede está no 03 §4 (traço redondo a cruzar + 3.º traço fora). O que se
mediu: o nosso Impasto cobre a 100 % (`cob 0,89`, cruzamento sem mistura) e a nossa aquarela K–M não
mistura no cruzamento (vermelho sobre azul = vermelho fora). Nós: `wet_charge` (constante, aquarela),
Smear (modo à parte), sem fim de tinta, sem sujo persistente.

### L6 — Inclinação em dois eixos e rotação da caneta

`tip_tilt_curve` (altura) e `tip_tilt_width_curve` (largura) + `PEN_ROTATION/PEN_TILT` como modo de
rotação do carimbo + `precise_tip_tilt`. Medido: `penciltilt` (8B a 55°) **37 px** de largura e
`textura 5,2 %` contra **30 px / 9,1 %** direito — é o lado do grafite. A espátula: `tip_tilt_width
(0,0),(1,−5)` estreita ao deitar. O flat `PEN_ROTATION`: o traço vertical com rotação 0 é uma linha de
4 px (fotografado em todas as pranchas: o cruzamento vermelho fino). Nós: tilt só escala o raio.

### L7 — A água é um volume do pincel, e o papel decide como corre

`Water` 0–100 na aquarela/tinta/acrílico (`volume_params.water`) + `length_water_reduction` (acaba ao
longo do traço) + os parâmetros por papel (Absorbency, Re-wet, Diffusion speed, Edge darkening, Drips,
Granulation). Medido: *Round* a água 80 e opacidade 60: cobertura 0,37, `borda 0,83`, cruzamento roxo
(mistura molhada); a largura **não** depende da água (31 px nos três gestos — «a água alarga» foi
refutada nesta bancada, doc 03 §3.6). A `Flat 2` a opacidade 50 / água 30: `cob 0,43`,
**`borda 1,04`** (a orla escurece ao secar) e orla de **1 px** (a aguada seca com borda dura). A nossa
aquarela: `cob 0,40`, `borda 1,23` ✅ (a orla escura nossa é mais forte), orla 3 px, mas **a largura não
depende da água** (não há água como volume) e o Wet Paint (fluido) é outro meio.

### Explicações REFUTADAS (não reconstruir)

| hipótese | medição que a derruba |
|---|---|
| «É a resolução das texturas (2048²)» | marcador e airbrush têm `paper 0` e grão plano e continuam «certos»: a lei está no carimbo e na mistura, não na imagem |
| «É dinâmica de velocidade» | `speed_size/opacity` são **neutros em 281/281**; a única lei de velocidade é *Smudge on Paper Bumps* |
| «É o antialias» | `use_AA=false` em 87 presets (lápis/pastel) de propósito; a orla de 1–2 px vem do carimbo + smudge |
| «O traço dele é mais uniforme» | não: CV é igual (0,001–0,01 nos dois); o que difere é **textura com uniformidade** (fios) |
| «É o Mixbox» | a mistura por pigmentos é opcional (Pro, por layer) e estava **desligada** (`pigment_color_mixing=false`) em todas as medições |

---

## §3 — O que o Rebelle tem que nós não temos (lista de funcionalidades)

Marcado **[PRO]** o que o manual reserva à edição Pro. Agrupado por «leva o app a outro patamar».

**Motor (o traço):** Bristle Brushes [PRO] · smudge entre carimbos (+ on paper bumps) · 4 pares forma/grão
+ Texture Order + Image Sequence · grão Follow/Stretched/Tiled + DPI · Background Texture 1/2 (+ Use Shape
Texture, Invert Paper Texture) · Dual Brush · Shape Border · Random Flip X/Y · Opacity Multiplier ×100 +
Non-linear Opacity · Pressure Smoothing + Pulled String · Tip Tilt (2 curvas) + Precise · Pen Rotation ·
Force Rotation Mode · Cursor modes.

**Tinta (a mistura):** Paint & Mix / Paint & Blend / Blend por curvas + Paint Pressure Threshold · Loading +
Length (tinta finita) · Dirty Brush · MultiColored Brush · Oiliness + Length Oiliness Fadeout · Average
Smudge Color · Glaze/Normal por pincel · Paint Blending (modo de blend da cor) · Default Color · Color
Noise per Bristle/Stamp/Stroke (HSLuv) · Glaze mode Transparent/Semi/Opaque na aquarela.

**Papel e água:** loja de papéis (height-maps, deckled edges, texture scale) · Paper Texture
Strength/Contrast/Scratch · Watercolor Behavior por papel (Absorbency, Re-wet, Texture influence, Edge
darkening, Diffusion speed, Drips size/length, Granulation 3 texturas + própria) · Water tool / Dry tool
(Absorbency, Keep wet) / Blow (Strength) · Wet the Layer / Wet All Visible / Fast Dry / Pause Diffusion ·
Tilt panel + acelerómetro + Lock to Canvas · Masking Fluid Layer.

**Relevo e luz:** RealShader [PRO] (env-maps 1/2/Legacy) · SoftShadows + Shadow Altitude [PRO] · Impasto
Depth/Gloss por layer · Metallic materials + Reflectivity map + custom [PRO] · Heightmap EXR export [PRO] ·
NanoPixel ×20 + Fractal [PRO] · Smooth Scaling [PRO].

**Cor:** roda RYB + harmonias (lock) · HSLuv · Mixing Palette (com undo) · Color Sets de pigmentos nomeados ·
Pick Color com Mix · Natural Colors · Pigments [PRO, Mixbox — **não é porta**].

**Ferramentas/UI:** Properties com 5 volumes + Volume Presets + Save Volumes policy · Brush Creator com
preview de traço · painel de pincéis com grupos/favoritos/recentes/partilha · Rulers (line, ellipse,
perspective 1/2/3) · Symmetry (ângulo e eixo) · Stencils (tile/border) · Structures · Paths (SVG) ·
Tracing layer · Selection (6 modos, feather, roughness, expand/contract, quick fill) · Fill gradient/pattern
+ wet · Liquify 7 modos (+ fractal) · Transform Deform/Perspective/Warp · Filter layers (gradient map…) ·
Reference images · Record time-lapse · PSD [PRO] · WebSocket Control [PRO] · Photoshop plug-in [PRO].

**O que NÓS temos e ele não:** taper explícito, linhas procedurais (Sketchy/Wire/Ribbon/Rough/Dash),
Solid (preencher o cercado), 24 modos de blend do carimbo, 10 luzes com rig, Knife com `plow/push`,
Composite Brush de 5 camadas, undo por diff, o fluido completo (ADR-0134) com rewet/dry por poça,
Kubelka–Munk próprio com papel como filtro (vidro), sculpt do relevo, inpaint.

---

## §4 — O plano: o que implementar e como (ondas, cada uma com gate)

> ⚠️ **Decisão do dono (07/10, depois deste doc):** só o **Wet Paint** evolui; Digital, Impasto e Watercolor
> ficam como estão. As ondas abaixo descrevem o alvo em geral; a versão re-apontada ao Wet Paint (com a cerca
> «os outros meios byte-idênticos») é o [04_PROMPT_linha_wet_painter.md](04_PROMPT_linha_wet_painter.md).

Princípios: ECS/drop-crate (ADR-0075); **meça antes de limitar** (§0.0); o oráculo corre-se pela porta
WebSocket com `SET_RANDOM_SEED` e os gestos da sonda — **cada onda nasce com a sua fixture do Rebelle e a
régua já existente**; a barra sai do vale medido com os dois lados (método §3 passo 6). Ordem = dependência,
não importância.

### W1 — Presets em ficheiro (enabler) · crate nova `ph2d-brush-presets` · custo ~2 sessões

Formato e política no doc 02 §5. Entrega: ler/escrever/migrar; miniatura determinística; painel de pincéis
(grupos = pastas, favoritos, recentes); *Save as New Preset*, *Save as Default*, *Reset*; os presets de
fábrica do doc 02 §6 em `assets/brushes/`. **Gates:** round-trip byte-idêntico de cada preset; campo novo
sem default reprova a migração; `the_factory_presets_are_counted` (número que se conta); miniatura por hash
(o corpus de regressão de todo o motor). **Painel de volumes** (Size · Opacity · Water/Oiliness ·
Pressure · Length) no topo da secção Brush, com presets de volumes (`1,3,5,…100`) — `ph2d-panel-painter-layers`.

### W2 — O motor de carimbo cresce (`ph2d-painter-brush`) · ~3 sessões

1. **Smudge entre carimbos** (L2): após `stamp_dab(n)`, smear do carimbo n−1 → n ao longo da tangente com
   raio `smudge_density·r`; opt-in por preset (`splat_smudge`). Gate: período do perfil `≤ 10 %` da potência
   no Impasto a 40 px (hoje 42 %); controlo: desligado reproduz o byte actual.
2. **Opacity multiplier** (×1…×50) com `accumulate` e tecto — o «corpo» dos óleos sem saturar. Gate: cobertura
   do miolo em função do multiplicador replica a curva medida no Rebelle (`cob` 0,60 → 0,92 de ×1 a ×5).
3. **Pressure smoothing** (média móvel 0…9 eventos) e **Pulled String** (corda de comprimento L; a marca só
   anda quando a corda estica) ao lado do `stabilizer`. Gate: a rampa de pressão da sonda com ruído ±0,1
   dá largura monótona.
4. `pressure_max` (tecto, 0,25…1 — o grafite nunca é preto), ganho, **opacity jitter**, **scatter
   perpendicular**, `use_AA` por preset, **Random Flip X/Y**.
5. **Tilt em 2 eixos** (L6): `tilt_height_curve`, `tilt_width_curve` (escala do carimbo no eixo do traço e
   no normal), `rotation_mode {None, FollowTrajectory, PenTilt, PenRotation}` + `force`. Precisa do
   **barrel rotation** no `CanvasPointer` (foundational, append-only: `rotation: f32`) — Modo L. Gate: 8B a
   55° dá largura `≥ 1,2×` a direita e textura menor (medido 37/30 = 1,23; 5,2 % contra 9,1 %).

### W3 — Texturas: pares, sequência, fundo, dual (`ph2d-painter-brush/texture.rs`) · ~3 sessões

1. **Até 4 pares forma/grão** + `texture_order {Sequential, Random(ratio), Pressure(faixas), Tilt(faixas)}`
   com zonas de mistura entre pares (L3). O slot actual é o par 0 (byte-idêntico quando só há um).
2. **Grão Follow / Stretched(velocidade) / Tiled** + `grain_dpi` (escala com o DPI do documento) +
   random start/stamp offset + follow shape size/rotation.
3. **Background Texture 1/2**: um slot que FURA o carimbo, com `blend {Normal, Multiply, InvMultiply,
   HardMix, Overlay, Height}` e `paper_strength` (curva por pressão) — `Height` lê o relevo do papel (vales
   por defeito, invert = picos). Gate: lápis com `INV_MULTIPLY` deixa os vales do papel brancos (medir
   `textura` ≥ 8 % e `borda/miolo` < 0,5 — o 8B dele: 9,1 % / 0,32).
4. **Dual Brush** (stamp/stroke, count 1…15, size, opacity, spacing, scatter, curvas, follow): o pó do
   carvão e do pastel. Gate: *Charcoal* com 14 carimbos a 0,3 dá halo de pó medível (cobertura fora da
   silhueta > 0).
5. **Shape Border** (start/mid/end + curva por pressão) — o Rake. Última, só se o smoke pedir.

### W4 — A lei do papel (`ph2d-tool-painter` substrate + `ph2d-painter-brush`) · ~2 sessões

`paper_strength_curve` (por pressão), `paper_contrast`, `paper_scratch {Auto, Light, Dark}`, papel como
**height-map** (as nossas texturas procedurais já dão altura: `substrate_relief.rs:118`), `texture_scale`
25–400 %, deckled edges opcional; e **comportamento por papel** (`PaperBehaviour {absorbency, rewet,
texture_influence, edge_darkening, diffusion_speed, drips{on,size,length}, granulation{strength,density,
texture}}`) presets Default / Hot Pressed / Cold Pressed / Rough / Japanese — a ponte para os knobs do fluido
(`WetKnobs`) e da aguada (`edge_gain`, `granulation`, `wet_rewet`). Gate: a mesma aguada em Hot Pressed e
Rough difere em `textura` e `borda` nos sentidos medidos no Rebelle (`PaperInfluence` 3 vs 6).

### W5 — A lei da tinta (todos os meios) · ~3 sessões

1. **Paint & Mix / Paint & Blend / Blend** por curvas de pressão com **limiar da queda** relativo ao máximo
   do traço (L5) — generaliza `wet_charge/pull` para Digital/Impasto. O Smear deixa de ser só um modo: é o
   extremo `Blend` da mesma lei.
2. **Loading + Length**: tinta finita por traço (`paint_fadeout_curve`), passa a Blend quando acaba
   (preferência). **Oiliness** e `length_oiliness_fadeout` no Impasto (`impasto_plow` deriva dela).
3. **Dirty Brush** (a cor apanhada persiste entre traços; `C`/clique na cor limpa) e **MultiColored**.
4. **Glaze/Normal** por preset (já é `accumulate`); **Paint Blending** (modo do carimbo) por preset; **Default
   Color**; **Color Noise per Bristle (altura) / Stamp / Stroke** em HSLuv.
Gate: o cruzamento vermelho sobre azul em `PAINT_MIX` com `dirty` apanha azul (`cor_miolo` do traço vermelho
seguinte desvia-se do vermelho puro) — a barra sai PRIMEIRO da bancada do Rebelle (doc 03 §4, pincel redondo),
que ainda não correu.

### W6 — O motor de fios para os meios por carimbo (`ph2d-painter-bristles`, crate nova) · ~5 sessões

O maior salto (L1). Desenho: um pincel de partículas gera N fios dentro da silhueta (`Random Span`,
`Brush Form` 0…1 decide se os fios se reposicionam ou se escondem com a pressão), cada fio com pressão
própria (Perlin + ruído), largura ondulante (`wave_length`, `jitter`, `min_width`), tipo Add/Remove impasto,
cor com ruído HSLuv; três populações (Thin, Thick, Edges com curvas de pressão e tilt). O fio escreve cor e
altura no height field existente (`height.rs`), o Impasto ilumina. Reutiliza o `ph2d-wet-paint` (fios
Poisson) como ponto de partida e traz-lhe a **oscilação** e a **onda de largura** (cura dos carris).
Gate (o do L1): a 40 px, `textura ≥ 5 %` com `CV ≤ 0,02` e **período do resíduo não periódico** (fracção
do pico < 15 %) — o Rebelle dá 5,9 % / 0,011 / 9 %.

### W7 — Mídias com lei própria (presets + pequenos motores) · ~2 sessões

Com W2–W6 no lugar, as mídias são presets (doc 02 §6) + três leis pequenas: **tinta** (água que escorre no
molhado: `wet_dilution` × água do papel), **marcador** (grão preso à tela + glaze a 0,3 + modos de efeito),
**airbrush** (fluxo por pressão, cone por tilt). Gate: a prancha dos 8 meios reproduz a do Rebelle nas
sete métricas dentro do vale medido (doc 03), com `SET_RANDOM_SEED` fixo dos dois lados.

### W8 — Luz do relevo (doc 27 + env-map + sombras) · ~3 sessões

RealShader-like: env-map (2 ambientes), sombras projectadas de baixa altitude (SoftShadows), metálico com
reflectivity map. O doc [27](../27_plano_luz_sombra_propria.md) já tem o plano de sombras; o que entra
de novo é o env-map e o material metálico com mapa próprio.

### W9 — Ferramentas e UI de pintura · ~3 sessões

Mixing Palette · roda RYB + harmonias · Tilt panel/Blow strength · Water/Dry como ferramentas com presets ·
Wet the Layer / Fast Dry / Pause · réguas (linha, elipse, perspectiva) · stencils com tile/border ·
tracing layer · masking fluid · Filter layers · time-lapse. Cada um é drop-crate; prioridade pelo smoke.

### Ordem e custo

| onda | sessões | desbloqueia |
|---|---|---|
| W1 presets + volumes | 2 | tudo (os presets são o corpus) |
| W2 carimbo | 3 | L2, L6 |
| W3 texturas | 3 | L3, L4 (fundo) |
| W4 papel | 2 | L4, L7 |
| W5 tinta | 3 | L5 |
| W6 fios | 5 | L1 |
| W7 mídias | 2 | «todas as mídias» |
| W8 luz · W9 UI | 3 + 3 | acabamento |

---

## §5 — Recusas medidas (⛔ não reconstruir)

| recusa | porquê |
|---|---|
| ⛔ Portar o Mixbox / a mistura «por pigmentos» dele | CC BY-NC 4.0; o nosso K–M já dá azul+amarelo=verde |
| ⛔ Copiar as 31 formas / 17 grãos / papéis / granulações dele | assets do alvo (método §3.3); os nossos são procedurais + imagens nossas |
| ⛔ Ler o executável/DLLs para a lei | EULA §3(d); e o método diz que ler é o pior caminho — tudo aqui saiu de DADOS e de CORRER |
| ⛔ Dinâmica de velocidade como «a» explicação | neutra em 281/281 presets |
| ⛔ Injectar rato pelo `SendInput` para o smoke do Rebelle | **regrediu** nesta máquina (clique pinta, arrasto não; 5 variantes e 3 reinícios, `use_ink` on/off, Wine limpo — nada); a porta é o WebSocket, que ainda dá pressão e inclinação |
| ⛔ `SAVE` por WebSocket | «aren't supported via Websockets yet» (8.3.4) — fica a fotografia da janela a 100 % |
| ⛔ Um `SET_BRUSH` por conjunto de gestos | só vale para o 1.º traço; reenvia-se `SET_BRUSH_PARAMS` antes de cada gesto |

---

## §6 — A porta do oráculo (como repetir)

```
~/Apps/rebelle/oraculo/inicia.sh              # kwin virtual 1920×1080 (+ sessao.sh); NUNCA DISPLAY=:0
~/Apps/rebelle/oraculo/arranque.sh            # Rebelle com -websocket-server-enable -websocket-port 8265
                                              #   -websocket-allowed-ip-addresses 127.0.0.1,::ffff:127.0.0.1,::1
                                              #   passa Registration (Try → Rebelle 8 Pro) e cancela New Artwork
python3 gera_eventos.py WATERCOLOR "Watercolor/Flat 2" Z:/… nome [--size_px 40 --opacity 50 --water 30 --tilt 55 0 --dirty --paint_type PAINT_MIX]
node ws_eventos.mjs ev_x_A.json 6000 2        # UM evento por mensagem (o documento inteiro é ignorado)
bash inj.sh "click 1853 273; key ctrl+a; type 100; key enter"   # zoom 100 % (campo do Navigator)
bash foto.sh nome; magick nome.png -crop 1000x640+480+220 +repage c.png   # a tela 1000×640 a 1:1
python3 mede_traco.py c.png --crop 20,60,580,120 --label …
```

Cópias dos scripts em [`ferramentas/`](ferramentas/). Os eventos são os mesmos gestos de
`tests/sonda_tracos_por_meio.rs`; `SET_RANDOM_SEED 7` dos dois lados quando a nossa semente existir (W1).
