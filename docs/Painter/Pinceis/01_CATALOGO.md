# 01 — O CATÁLOGO: os pincéis mais usados e amados, juntos por identidade

> Leitor: as janelas 1–5 da missão ([`00_MISSAO.md`](00_MISSAO.md)) e o agente do `03_MOTORES.md`. Escrito a
> 2026-10-19 (JANELA 1, entrega B). Fontes (URLs) e o que se leu de cada app: [`01b_FONTES.md`](01b_FONTES.md).
> ⚠️ É a LISTA DE CANDIDATOS: um pincel só entra na fábrica depois do EXAME (§3 do contrato). Motor, prioridade e
> «falta» são a melhor leitura de hoje — o `03_MOTORES.md` confirma cada lacuna pelo CÓDIGO e por sonda.

## §0 — Resumo

**112 pincéis em 13 categorias** — 28 já na fábrica · 2 a refazer (Filbert, Palette Knife) · 82 novos. P1 = os mais
amados (contagem de fontes, `01b` §5) ou que o Rebelle corre (o teste expira ~06/11 — as fotos dele vêm primeiro);
P2 = muito usados; P3 = completam a gama.

| Categoria | Total | já existe | P1 | P2 | P3 |
|---|---:|---:|---:|---:|---:|
| A. Básicos digitais | 6 | 0 | 4 | 2 | 0 |
| B. Esboço / Lápis | 11 | 6 | 9 | 1 | 1 |
| C. Tinta / Caneta | 15 | 3 | 8 | 5 | 2 |
| D. Caligrafia / Lettering | 4 | 0 | 3 | 1 | 0 |
| E. Marcadores | 4 | 2 | 4 | 0 | 0 |
| F. Carvão / Giz / Pastel | 10 | 5 | 6 | 3 | 1 |
| G. Óleo / Acrílica | 14 | 4 (+2 refazer) | 11 | 3 | 0 |
| H. Guache | 4 | 1 | 3 | 1 | 0 |
| I. Aguarela / Sumi-e | 13 | 6 | 9 | 3 | 1 |
| J. Aerógrafo / Spray | 6 | 1 | 3 | 2 | 1 |
| K. Texturas / Elementos | 9 | 0 | 1 | 6 | 2 |
| L. Mistura / Borrador / Borracha | 9 | 0 | 6 | 2 | 1 |
| M. Especiais / Efeitos | 7 | 0 | 1 | 3 | 3 |
| **Total** | **112** | **28 (+2)** | **68** | **32** | **12** |

**Top-15 P1** (o mais amado primeiro; contagem de fontes no `01b` §5): Nikko Rull (G-13) · Gesinski Ink (C-14) ·
Studio Pen (C-4) · Monoline (D-1) · Technical Pen (C-6) · Graphite 8B/6B (B-4) · Hard Round (A-1) · Round Pressure
(A-3, Krita Basic-5) · Tilted Shade (B-6, Krita Quick Shade) · Rough Sketch (B-7, CSP Rough pencil / Pc Narinder) ·
Brush Pen (C-10, CSP Turnip pen) · Airbrush (J-1) · Soft Pastel/Chalk (F-5) · Smudge Finger (L-3) · Wet Blend Round
(G-12, Sargent/Wet Acrylic/Live Oil).

**Ordem de ataque sugerida** (alimenta o `ESTADO.md`): (1) as fotos do Rebelle de TODO P1/P2 com `Rb` — as que já
existem em `NewFeaturesWetPaint/fixtures/rebelle/` vão marcadas **†** (refotografar na BATERIA nova); (2) as lacunas
CAP-22 (presets dos 4 meios no navegador), CAP-01 (ponta que gira) e CAP-05 (modos de mistura) — juntas bloqueiam
≈ 55 pincéis; (3) as famílias por categoria, P1 primeiro.

**Saídas pelo veredito do dono (19/10):** Flat Oily sai (o G-6 Flat Bristle ocupa o lugar) · Filbert (G-5) e Knife
Smooth (→ G-7 Palette Knife) ficam **a refazer** com a ponta que gira (CAP-01; `26_ABERTOS` C38).

**Suspeitos de duplicado DENTRO da fábrica** (decide a matriz da bancada D, §3.3 do contrato; nenhum declarado sem
medição, mas cada um tem de MOSTRAR a diferença na prancha): Graphite 2B × HB (irmãos do mesmo ficheiro,
`NASCE_DE`) · Ink Bullet × Marker Bullet (o mesmo `Bullet` de duas ferramentas: a tinta escurece por cima, o marcador
multiplica) · Dabber × Oil Bristle (nasce dele) · Square Hard × Soft Pastel (mesmo vizinho) · Wet Round × Wet Wash ·
Express Oil × Acrylic Round. ⚠️ O Oil Bristle foi medido contra `Oil/Bristle/Bristle Flat` SEM a rotação: até a
CAP-01 existir, o G-6 Flat Bristle seria o mesmo pincel — G-6 só entra com a lâmina que gira.

## §1 — Capacidades de motor que o catálogo exige (→ `03_MOTORES.md`)

«Existe?» = o campo no código (`ph2d-painter-brush/src/spec.rs`, `blend.rs`; `ph2d-tool-painter/.../paint_mode.rs`),
**a confirmar por sonda** — campo existir ≠ pintar como o oráculo (a lição do P53).

| Id | Capacidade | Existe? (leitura do código) | Pincéis que a pedem |
|---|---|---|---|
| CAP-01 | Ponta CHATA que gira com a direção do arrasto (cerda chata, lâmina, bisel) | Digital: `needs_heading`/`dab_angle_deg`; Wet Paint: **não** (C38) | A-4, B-9, D-3, E-3, F-2, F-7, G-5, G-6, G-7, G-8, H-2, I-3, I-8 |
| CAP-02 | Largura/opacidade pela VELOCIDADE (Pc Dynamics Speed, CSP Tapered pen, CP Pencils) | `LineKind::Speed`? confirmar | C-6, D-1, D-2, B-1 |
| CAP-03 | Inclinação → sombra com o LADO da mina (Pc Tilt, AR Tilt Angle, CSP Design pencil) | `pen_tilt` (Digital) | B-6, B-7, F-2, F-7 |
| CAP-04 | Ponta que SE GASTA (PS erodible: Softness, Sharpen Tip) | não | B-11, F-4 |
| CAP-05 | Modos de mistura do PINCEL no meio molhado (Multiply, Screen/Add, Overlay, Dodge, Burn, Color, Hue) | Digital: `BrushBlend` (24 modos); Wet Paint: não | E-1, E-3, C-14, K-4, M-1, M-2, M-3 |
| CAP-06 | Dual brush: 2.ª ponta dispersa que RECORTA a 1.ª por um modo (PS, CSP «2-Brush shape», Pc dual) | não (há 2 slots: forma + grão) | C-9, C-11, F-3, G-11, G-13, K-9 |
| CAP-07 | Dinâmica de COR por carimbo/traço (H/S/V, FG↔BG; PS Color Dynamics, Pc Color Dynamics) | Digital: `color_jitter_*` | K-2, K-3, K-4, M-5 |
| CAP-08 | Dispersão + contagem de carimbos (PS Scattering, Pc Scatter/Count ≤ 16) | Digital: `spray_count`, `jitter_*` | I-11, J-2, J-3, J-5, J-6, K-1..5 |
| CAP-09 | Borrador que ARRASTA a cor (Kr Smudge Length/Dulling, PS Strength/Finger Painting, CSP Color stretch) | `PaintMode::Smear`; Wet: `wet_smudge`/`wet_pull` | L-1..4, G-12 |
| CAP-10 | Mistura molhada: carga que ACABA ao longo do traço + puxa a cor da tela (Pc Wet Mix Dilution/Charge/Attack/Pull/Grade; PS Mixer Wet/Load/Mix/Flow; CSP Amount/Density of paint; Fr Paint Mix; AR Loading/Thinners) | Digital: `wet_*`; Wet Paint: carga | G-12, G-13, G-11, H-1, H-3 |
| CAP-11 | GLAZE por traço: tecto de opacidade dentro do traço, acumula só entre traços (Pc Light/Uniform/Intense/Heavy Glaze; CP Glazing) | doc 07 (modos de render) | A-6, E-1..4, I-12 |
| CAP-12 | Escorridos por gravidade (Pc Spraypaints Drip, Water Drip) | Wet Paint: gravidade (Wet Drip) | J-4, I-6 |
| CAP-13 | A tinta EXPANDE onde a caneta pára e funde-se (AR Gloop Pen; poça do aparo) | não medido | C-1, C-7, C-12 |
| CAP-14 | Capilaridade pelas FIBRAS / orla escura (Pc Ink Bleed, CSP Watercolor edge Opacity/Darkness, CP Real Watercolor) | Watercolor: orla; Wet: papel | C-13, I-9, I-10 |
| CAP-15 | Backrun / «couve-flor»: água nova numa lavagem a secar | Wet Paint: `wet_rewet`? confirmar | I-10, L-6 |
| CAP-16 | Pixel-perfect sem anti-alias (Pc Alpha Threshold, CSP Anti-aliasing None) | não | A-5 |
| CAP-17 | Carimbo de imagem COLORIDA (RGBA) com rotação/escala aleatória e sequência (CP Image Hose, AR Sticker Spray, Kr ponta RGBA) | `shape` é máscara; RGBA não | K-2, K-3, K-4, M-6 |
| CAP-18 | Metálico / especular / glitter (Pc Materials Metallic/Roughness, AR Glitter Tube) | Impasto: `impasto_metallic`, `impasto_shine` | M-4, M-5 |
| CAP-19 | Cerdas individuais com carga que ESGOTA (PS Bristle tips: Bristles/Length/Stiffness; Kr Bristle ink depletion; CP RealBristle) | Wet Paint: fios (Oil Bristle) | G-1, G-8, G-9, G-11, I-8, I-13, C-11 |
| CAP-20 | Padrão no espaço da TELA (retícula, hachura; Pc Grain «Texturized») | grão `Tiled` | K-6, K-7 |
| CAP-21 | Espessura física + mistura molhada no MESMO traço (CP Thick Paint, AR Oil Brush, Fr Live Oil) | Impasto e Wet Paint são exclusivos | G-10, G-7 |
| CAP-22 | Presets dos meios Digital/Watercolor/Impasto no MESMO navegador e formato `.ph2d-brush.png` | confirmar (a fábrica hoje é toda Wet Paint) | todo pincel de motor ≠ WP (≈ 40) |
| CAP-23 | Borracha com o carácter do pincel (macia/parcial, com grão; CP Bleach/Darkener) | `EraseAlpha`; Wet: confirmar | L-8, L-9 |
| CAP-24 | Partículas com física (CP Particles Gravity/Flow/Spring, Kr Particle) | não | M-6 |
| CAP-25 | Linha que liga aos pontos anteriores (Kr Sketch/Curve, Harmony) | `LineKind::Sketchy` | M-7 |
| CAP-26 | Secar / levantar tinta (Rb Dry, Fr «Dry Layer», CP Sponges) | Wet Paint: secagem | L-6, L-7 |
| CAP-27 | Grão «Moving» (segue o traço) × «Texturized» (fica na tela) — Procreate Grain | `texture.mapping` | B-*, F-*, K-8 |
| CAP-28 | Estabilização forte (Pc StreamLine/Stabilization/Motion Filtering, CSP Stabilization, PS Pulled String) | `stabilizer`, `input_samples` | D-1, D-2, D-4, C-4 |

## §2 — O catálogo por categoria

Origens: **Pc** Procreate (Classic Library, nome exacto) · **PS** Photoshop · **Fr** Fresco · **Kr** Krita · **CSP**
Clip Studio Paint · **CP** Corel Painter · **AR** ArtRage · **MP** MyPaint · **Rb** Rebelle. Oráculo que CORRE:
`Rb Ferramenta/Grupo/Nome` (exacto da fixture `rebelle_presets_281.json`, espaços duplos incluídos; **†** = já
fotografado em `NewFeaturesWetPaint/fixtures/rebelle/`) · `MP pasta/ficheiro` (`/usr/share/mypaint-data/2.0/brushes/`,
CC0) · `Kr preset` (bundle `Krita_4_Default_Resources`, CC-0, ou `/usr/share/krita/paintoppresets/`). CSP/AR/CP: sob
Wine pela entrega C (`02_ORACULOS.md`); até lá «doc.». Motor: **Dg** Digital · **Wc** Watercolor · **Im** Impasto ·
**WP** Wet Paint. ✅ = já na fábrica.

### A. Básicos digitais (o topo de todo app)

| # | Nome nosso | Origem | Oráculo que corre | Identidade (o que se vê) | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| A-1 | Hard Round | PS Hard Round · Kr Basic-1 · Pc Hard Brush · MP basic_digital_brush | Kr `b)_Basic-1` · MP `deevad/basic_digital_brush` · Rb `Pencil/Digital/Round Hard` | disco duro opaco, borda limpa, tamanho pela pressão | Dg | bloquear formas; o «cavalo de batalha» | CAP-22 | P1 |
| A-2 | Soft Round | PS Soft Round · Pc Soft Brush · Kr Basic-3 Flow | Kr `b)_Basic-3_Flow` · Rb `Pencil/Digital/Round Soft` | disco gaussiano, sem borda, acumula pelo fluxo | Dg | luz e sombra suaves sem textura | CAP-22 | P1 |
| A-3 | Round Pressure | PS Hard Round Pressure Opacity · Kr Basic-5 Size Opacity · MP brush | Kr `b)_Basic-5_Size_Opacity` · MP `deevad/brush` | redondo cujo tamanho E opacidade seguem a pressão | Dg | o mais votado do Krita; pintura de painel | CAP-22 | P1 |
| A-4 | Flat Block-in | Kr Basic-4 · MP fill/basic_digital_knife · Rb Digital Flat | MP `deevad/basic_digital_knife` · Rb `Pencil/Digital/Flat` | ponta chata dura, aresta que vira com o gesto | Dg | planos de cor rápidos (concept, Polycount) | CAP-01, 22 | P1 |
| A-5 | Pixel 1px | Kr Pixel Art · MP 1pixel · PS Pencil · CSP Mapping pen sem AA | Kr `u)_Pixel_Art` · MP `experimental/1pixel` | um pixel exacto, sem anti-alias | Dg | pixel art | CAP-16 | P2 |
| A-6 | Glaze Round | Pc Light Glaze · CP Glazing Soft · MP detail_brush_large_glazing · Kr Bristles-4 Glaze | MP `deevad/detail_brush_large_glazing` · Kr `f)_Bristles-4_Glaze` | translúcido; a opacidade sobe devagar e nunca passa um tecto no traço | Dg | velaturas por camadas | CAP-11, 22 | P2 |

### B. Esboço / Lápis

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| B-1 | ✅ Graphite HB | Pc HB Pencil / Procreate Pencil · PS KYLE Ultimate Pencil Hard · Kr Pencil-2 · CSP Real pencil · AR Pencil | Rb `Pencil/Graphite Pencil/HB` † · Kr `c)_Pencil-2` | cinza médio, grão do papel nos picos | WP | o esboço do dia-a-dia | — | P1 |
| B-2 | ✅ Graphite 2H | Kr Pencil-1 Hard · MP 4H_pencil · CSP Lighter pencil | Rb `Pencil/Graphite Pencil/2H` † · MP `deevad/4H_pencil` | claro, fino, quase sem mancha | WP | linhas de construção que somem | — | P1 |
| B-3 | ✅ Graphite 2B | MP 2B_pencil (deevad, ramon, tanda) · CP 2B Pencil | Rb `Pencil/Graphite Pencil/2B` † · MP `deevad/2B_pencil`, `tanda/pencil-2b` | escuro macio, grão cheio com pressão | WP | esboço expressivo | — (suspeito × HB) | P1 |
| B-4 | ✅ Graphite 8B | Pc 6B Pencil · CSP Darker pencil · MP 8B_Pencil · Kr Pencil-3 Large 4B · CP Greasy Pencil | Rb `Pencil/Graphite Pencil/8B` † · MP `Dieterle/8B_Pencil#1`, `tanda/pencil-8b` | negro brilhante, largo, grão saturado | WP | valores escuros, desenho de tom | — | P1 |
| B-5 | Mechanical 0.5 | CSP Mechanical pencil · Pc Technical Pencil · AR Pencil «Precise» | Rb `Pencil/Graphite Pencil/Pentell` | linha fina CONSTANTE, sem textura | WP | limpar linha, desenho técnico | — | P1 |
| B-6 | Tilted Shade | Kr Pencil-6 Quick Shade / Pencil-5 Tilted · Rb Tilted Hard · AR Tilt Angle · CSP Design pencil | Kr `c)_Pencil-6_Quick_Shade`, `c)_Pencil-5_Tilted` · Rb `Pencil/Digital/Tilted Hard` · Rb † `penciltilt` (8B deitado) | caneta deitada = mancha larga do lado da mina | Dg | sombrear sem trocar de pincel («o mais parecido com grafite real», krita-artists) | CAP-03 | P1 |
| B-7 | Rough Sketch | CSP Rough pencil (o favorito) · Pc Narinder Pencil · MP Sketch_1 | MP `ramon/Sketch_1` · Kr `c)_Pencil-4_Soft` | muita textura; do claro ao escuro pela pressão, borda que treme | Dg | rough solto, gesto | CAP-03, 22 | P1 |
| B-8 | ✅ Colored 4B | Pc Derwent · CSP Colored pencil · CP Colored Pencils | Rb `Pencil/Colored Pencil/4B Color` † | cor cerosa, deixa o papel branco nos vales; escurece por passadas | WP | lápis de cor | — | P1 |
| B-9 | Craft Flat | Rb Craft (lápis de carpinteiro) · CP Cover Pencil | Rb `Pencil/Graphite Pencil/Craft Flat` | mina chata: fino num sentido, largo no outro | WP | letras, hachura de largura variável | CAP-01 | P2 |
| B-10 | ✅ Scratch | Rb Scratches | Rb `Pencil/Scratches/Scratch` † | riscos finos e nervosos, multi-linha | WP | arranhões, pêlo | — | P1 |
| B-11 | Erodible Pencil | PS erodible tips · CP Real Pencils | doc. (PS) | a ponta gasta-se: engrossa e perde o fio | Dg | o lápis «que se afia» | CAP-04 | P3 |

### C. Tinta / Caneta

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| C-1 | ✅ Dip Pen | CSP Real G-pen · CP Pens Croquil · MP soft-dip-pen | Rb `Ink/Pen/Dip Pen` † · MP `deevad/soft-dip-pen` | aparo flexível: fino→grosso pela pressão, tinta que poça | WP | arte-final com tinta de verdade | CAP-13 | P1 |
| C-2 | ✅ Ink Bullet | Rb Bullet · AR Ink Pen | Rb `Ink/Pen/Bullet` † | ponta redonda de largura quase fixa | WP | contorno uniforme a tinta | — (suspeito × E-1) | P1 |
| C-3 | ✅ Ink Brush | CSP India ink Brush pen · CP Sumi-e/Liquid Ink | Rb `Ink/Brush/Brush` † | pincel de tinta: grande variação, ponta afiada | WP | arte-final a pincel, manchas de preto | — | P1 |
| C-4 | Studio Pen | Pc Studio Pen / Syrup / Mercury · CSP G-pen · Kr Ink-3 Gpen · PS Kyle Basic Inker · MP pen | Kr `d)_Ink-3_Gpen` · MP `deevad/pen` | tinta lisa e chapada, engrossa com a pressão mas varia pouco | Dg | a arte-final limpa mais usada (BD, mangá) | CAP-22, 28 | P1 |
| C-5 | Mapping Pen | CSP Mapping pen · Kr Ink-1 Precision | Kr `d)_Ink-1_Precision` | fio fino e duro, nítido | Dg | detalhe, hachura fina | CAP-22 | P2 |
| C-6 | Technical Pen | Pc Technical Pen / Fine Tip · Kr Ink-2 Fineliner · MP liner · CSP Tapered pen (variante rápida) | Kr `d)_Ink-2_Fineliner` · MP `deevad/liner` | largura uniforme («Rapidograph»), ponta afinada | Dg | fineliner, quadrinhos, diagramas | CAP-02, 22 | P1 |
| C-7 | Ballpoint | MP ballpen · Pc Gel Pen | MP `deevad/ballpen` | esferográfica: falhas no traço leve, gota no fim | Dg | esboço de caderno | CAP-13 | P2 |
| C-8 | Fountain Pen | Rb Fountain Round · MP Fountain_SF | Rb `Ink/Pen/Fountain Round` · MP `Dieterle/Fountain_SF#1` | caneta-tinteiro: fluxo contínuo, translúcido | WP | escrita e esboço a tinta | — | P2 |
| C-9 | Reed Pen | Rb Reed Pen | Rb `Ink/Pen/Reed Pen` † | cana: borda áspera, seca e falha | WP | tinta rústica | CAP-06 | P2 |
| C-10 | Brush Pen | CSP Turnip pen (o favorito) · Kr Ink-7 Brush Rough · Pc Brush Pen | Kr `d)_Ink-7_Brush_Rough` | fino→grosso suave com pouca pressão, afina a zero | Dg | contorno de BD com peso | CAP-22 | P1 |
| C-11 | Dry Ink | Pc Dry Ink / Thylacine / Tinderbox · Kr Ink-4 Pen Rough · CP Dry Ink · CSP Bit husky · MP textured_ink | Kr `d)_Ink-4_Pen_Rough` · MP `classic/textured_ink` | tinta seca: borda quebrada, falhas de cerda | Dg | arte-final com textura | CAP-06, 19 | P1 |
| C-12 | Gloop Pen | AR Gloop Pen · CSP Wet blotting ink | doc. (AR, Wine na entrega C) | gota de tinta que EXPANDE parada e funde | WP | lettering «gordo», manchas vivas | CAP-13 | P3 |
| C-13 | Ink Bleed | Pc Ink Bleed · CSP Watery ink | doc. (Pc) | a tinta sangra a cada traço: borda áspera pelas fibras | WP | tinta em papel absorvente | CAP-14 | P2 |
| C-14 | Gesinski Ink | Pc Gesinski Ink | doc. (Pc) | pena/caneta-pincel translúcida que ESCURECE onde se sobrepõe | WP | hachura e arte-final orgânica (2.º mais amado do Pc) | CAP-05, 11 | P1 |
| C-15 | Wood Stick | Rb Wood | Rb `Ink/Pen/Wood` | pau mergulhado em tinta: borrões irregulares | WP | esboço a tinta solto | — | P3 |

### D. Caligrafia / Lettering

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| D-1 | Monoline | Pc Monoline · AR Ink Pen | doc. (Pc) | largura constante sem pressão, pontas redondas, liso | Dg | contorno e lettering limpo | CAP-28, 22 | P1 |
| D-2 | Script | Pc Script / Brush Pen · CSP Calligraphy | doc. (Pc) | finos a subir, grossos a descer, muito estabilizado | Dg | brush lettering | CAP-02, 28 | P1 |
| D-3 | Broad Nib | Rb Fountain Square · MP calligraphy · CP Calligraphy | Rb `Ink/Pen/Fountain Square` † · MP `classic/calligraphy` | bico chato a ângulo FIXO: largura pela direção | WP | caligrafia clássica (itálica, gótica) | CAP-01 | P1 |
| D-4 | Shale | Pc Shale Brush · Pc Calligraphy Chalk | doc. (Pc) | pincel de letras macio, muito estabilizado, borda com grão leve | Dg | lettering liso com grão | CAP-28, 27 | P2 |

### E. Marcadores

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| E-1 | ✅ Marker Bullet | Pc Marker · CP Round Tip/Felt · AR Felt Pen · CSP Felt pen · MP marker_fat | Rb `Marker/Marker/Bullet` † · MP `classic/marker_fat` | redondo transparente; acumula e poça onde se sobrepõe | WP | o marcador de ilustração | CAP-05, 11 | P1 |
| E-2 | ✅ Marker Liner | Pc Fine Tip · MP marker_small | Rb `Marker/Marker/Liner` † · MP `classic/marker_small` | marcador fino de largura fixa | WP | contorno a marcador | — | P1 |
| E-3 | Marker Chisel | Rb Chisel · Kr Marker Chisel Smooth · CP Flat Rendering | Rb `Marker/Marker/Chisel` † · Kr `e)_Marker_Chisel_Smooth` | bisel: faixas largas com riscos de sobreposição | WP | preencher áreas a marcador | CAP-01, 05 | P1 |
| E-4 | Dry Marker | Rb Dry · Kr Marker Dry · CP Dirty Marker | Rb `Marker/Marker/Dry` · Kr `e)_Marker_Dry` | marcador a acabar: falhas e listras | WP | textura de marcador gasto | CAP-11 | P1 |

### F. Carvão / Giz / Pastel

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| F-1 | ✅ Charcoal | Pc Vine/Willow Charcoal · Kr Charcoal Rock Soft · CP Soft Vine · MP charcoal | Rb `Pencil/Charcoal/Charcoal` † · MP `classic/charcoal`, `tanda/charcoal-01` | negro poroso, grão forte, pó | WP | figura, valor | — | P1 |
| F-2 | Compressed Charcoal | Pc Charcoal Block / 6B Compressed · CP Square Conte · Rb Charcoal Flat | Rb `Pencil/Charcoal/Charcoal Flat` | barra deitada: faixa larga e densa | WP | massas escuras, fundos | CAP-01, 03 | P1 |
| F-3 | Charcoal Pencil | Kr Charcoal Pencil Large/Medium · CP Hard Charcoal Pencil · Pc Carbon Stick | Rb `Pencil/Charcoal/Charcoal Liner` · Kr `h)_Charcoal_Pencil_Medium` | carvão em lápis: linha negra granulosa | WP | detalhe a carvão | CAP-06 | P2 |
| F-4 | Charcoal Powder | Rb Powder · Pc Burnt Tree | Rb `Pencil/Charcoal/Powder` | pó de carvão espalhado, sem linha | WP | tons de fundo | CAP-04 | P2 |
| F-5 | ✅ Soft Pastel | Pc Soft Pastel / Bonobo Chalk · CP Soft Pastel · Kr Chalk Soft · AR Chalk · MP chalk · PS KYLE Pastel Palooza | Rb `Pastel/Pastel Dry/Dry Thin` † · Kr `h)_Chalk_Soft` · MP `deevad/chalk` | cor opaca só nos picos do papel | WP | pastel e giz (o «chalk» de 99 % do trabalho, Polycount) | — | P1 |
| F-6 | ✅ Square Hard | Rb Square Hard · CP Square Chalk | Rb `Pastel/Pastel Dry/Square Hard` † | pastel duro de aresta: linha nítida com grão | WP | pastel de detalhe | CAP-01 | P1 |
| F-7 | Pastel Side | Rb Flat Soft · CP Square Soft Pastel · Pc Charcoal Roll | Rb `Pastel/Pastel Dry/Flat Soft` | o LADO do bastão: faixa larga, só os picos | WP | fundos e massas em pastel | CAP-01, 03 | P2 |
| F-8 | ✅ Oil Pastel | Pc Oil Pastel / Artist Crayon · CP Variable Oil Pastel | Rb `Pastel/Pastel Oil/Oil Pastel 2` † | gordo, empasta e arrasta a cor de baixo | WP | cor cremosa e vibrante | — | P1 |
| F-9 | ✅ Wax Scrape | Rb Wax Scrape | Rb `Pastel/Scrape/Wax Scrape` † | raspa a cera e mostra a camada de baixo | WP | sgraffito | — | P1 |
| F-10 | Wax Crayon | AR Crayon · CP Waxy Crayon · Rb Wax Pencil | Rb `Pastel/Pastel Oil/Wax Pencil` | ceroso, salta o grão, não mistura | WP | ilustração infantil | — | P3 |

### G. Óleo / Acrílica

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| G-1 | ✅ Oil Bristle | Pc Old Brush / Oil Paint / Turpentine · CP RealBristle · Kr Bristles-1..3 · MP oil-01-paint | Rb `Oil/Bristle/Bristle Flat` † (sem rotação) · MP `tanda/oil-01-paint` | sulcos de cerda, cor que mistura com a de baixo | WP | o óleo a pincel | CAP-19 | P1 |
| G-2 | ✅ Acrylic Round | Pc Round Brush / Acrylic · CP Acrylics · MP acrylic-04-paint | Rb `Oil/Round/Round Hard` † · MP `tanda/acrylic-04-paint` | redondo opaco, borda firme, seca rápido | WP | acrílica de cobertura | — (suspeito × G-3) | P1 |
| G-3 | ✅ Express Oil | Rb Express Oil | Rb `ExpressOil/Round/Hairy` † | óleo rápido e fluido, mistura sem secar | WP | estudos alla prima | — | P1 |
| G-4 | ✅ Dabber | Rb Dabber · CP Impressionist | Rb `Oil/Dabber/Dabber` † | toques curtos de cerdas | WP | folhagem, impressionismo | — (suspeito × G-1) | P1 |
| G-5 | Filbert (REFAZER) | Rb Filbert · Fr Live Oil Filbert · CP Thick Paint Filbert | Rb `Oil/Flat/Filbert` † | chato de ponta oval: largo de lado, fino de canto | WP | o pincel de retrato | CAP-01 | P1 |
| G-6 | Flat Bristle | Pc Flat Brush · Fr Live Oil Flat · Kr Bristles-5 Flat · CP Opaque Flat | Rb `Oil/Flat/Flat` · Kr `f)_Bristles-5_Flat` | chato: largura pela direção, sulcos paralelos | WP | planos, arquitetura (no lugar do Flat Oily) | CAP-01, 19 | P1 |
| G-7 | Palette Knife (REFAZER) | AR Palette Knife (Flat/Edge Blade) · CP Palette Knives · Kr Wet Knife · MP knife | Rb `Oil/Knife/Knife Oily` · Rb † `knife` (Knife Smooth) · Kr `i)_Wet_Knife` · MP `classic/knife` | lâmina que raspa, empurra e alisa; aresta nítida | WP | empaste, pedra, céu raspado | CAP-01, 21 | P1 |
| G-8 | Fan Brush | Rb Fan · MP Fan#1 · CP Thick Paint Fan | Rb `Oil/Dabber/Fan` · MP `Dieterle/Fan#1` | leque: riscos paralelos espaçados | WP | erva, cabelo, nuvens | CAP-01, 19 | P2 |
| G-9 | Rake | Rb Rake Sharp · Kr Blender Rake · CP Depth Rake · PS Kyle Rake | Rb `Oil/Rake/Rake Sharp` † | cerdas separadas: linhas paralelas nítidas | WP | cabelo, madeira | CAP-19 | P2 |
| G-10 | Thick Impasto | CP Thick Paint · AR Oil Brush (Loading alto) · CSP Thick oil paint | Rb `Oil/Flat/Thick` | tinta GROSSA com relevo e luz | Im | óleo «que se sente» | CAP-21 | P1 |
| G-11 | Dry Brush | Pc Dry Brush · Rb Flat Dry · Kr Dry Bristles · AR Oil Loading baixo | Rb `Oil/Flat/Flat Dry` · Kr `g)_Dry_Bristles` | quase seco: só toca os picos, riscado | WP | scumble, luz de grão | CAP-06, 10, 19 | P1 |
| G-12 | Wet Blend Round | Pc Wet Acrylic · CP Sargent Brush · Fr Live Oil Round · PS Kyle Paintbrush Wet Blender · Kr Wet Paint · MP acrylic-04-with-water | Kr `i)_Wet_Paint` · MP `tanda/acrylic-04-with-water` | carregado: deposita e PUXA a cor de baixo no mesmo gesto | WP | «mistura no quadro», retrato alla prima | CAP-09, 10 | P1 |
| G-13 | Nikko Rull | Pc Nikko Rull · Pc Blackburn | doc. (Pc) | cor opaca com textura de rolo, falhas de carga | Dg | o pincel de pintura mais amado do Procreate | CAP-06, 10, 22 | P1 |
| G-14 | Rigger | Rb Liner · MP rigger_brush · CP Thick Paint Rigger | Rb `Oil/Round/Liner` · MP `deevad/rigger_brush` | fio longo e fino que dura | WP | galhos, cordas | — | P2 |

### H. Guache

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| H-1 | Gouache Round | Pc Gouache · PS Kyle's Gouache · CSP Gouache · CP Wet Gouache Round | Rb `Watercolor/Gouache/Gouache Round` † | opaco-mate; forte = sólido, fraco = áspero; seca plano | WP | ilustração editorial, fundos de animação | CAP-10 | P1 |
| H-2 | ✅ Gouache Flat | Rb Gouache Flat · CP Thick Gouache Flat | Rb `Watercolor/Gouache/Gouache Flat` † | faixas mates de guache | WP | planos | CAP-01 | P1 |
| H-3 | Gouache Dry | Rb Gouache Dry · CSP Dry gouache | Rb `Watercolor/Gouache/Gouache Dry` † | quase seco: arrasto granuloso | WP | texturas de guache | CAP-10 | P1 |
| H-4 | Gouache Wash | Rb Gouache Wash · CSP Thin gouache | Rb `Watercolor/Gouache/Gouache Wash` | diluído: mancha larga semitransparente | WP | fundos de guache | — | P2 |

### I. Aguarela / Sumi-e

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| I-1 | ✅ Wet Round | Pc Watercolor · Fr Live Watercolor Round Detail · CSP Opaque watercolor · CP Real Watercolor · AR Watercolor | Rb `Watercolor/Watercolor/Round` † | redondo molhado: escorre, orla escura ao secar | WP | o pincel de aguarela | — | P1 |
| I-2 | ✅ Wet Wash | Pc Water/Wash · Fr Wash Soft · CSP Wet wash | Rb `Watercolor/Watercolor/Wash Round` † | lavagem larga e uniforme, com orla | WP | céus, fundos | — (suspeito × I-1) | P1 |
| I-3 | ✅ Wet Flat | Fr Wash Flat · CSP Flat watercolor | Rb `Watercolor/Watercolor/Flat` † | chato molhado: faixas planas | WP | lavagens em faixa | CAP-01 | P1 |
| I-4 | ✅ Wet Liner | Rb Liner | Rb `Watercolor/Watercolor/Liner` † | fio de aguarela | WP | detalhe molhado | — | P1 |
| I-5 | ✅ Wet Dry Brush | CP Digital Watercolor Dry Brush · AR Watercolor Loading baixo | Rb `Watercolor/Watercolor/Flat Dry` † | falhas brancas no grão | WP | brilho na água | — | P1 |
| I-6 | ✅ Wet Drip | Pc Water/Water Drip | Rb `Watercolor/Watercolor/Round` † (vizinho) | escorre por gravidade | WP | escorridos | CAP-12 | P1 |
| I-7 | Mop | Rb Mop · CP Wet on Wet Paper | Rb `Watercolor/Watercolor/Mop` | trincha carregadíssima, bordas difusas | WP | molhado-no-molhado em área | — (suspeito × I-2) | P2 |
| I-8 | Bristle Watercolor | Rb Bristle Flat · Kr WaterC Flat Big-Grain Tilt | Rb `Watercolor/Bristle/Bristle Flat` · Kr `j)_WaterC_Flat_Big-Grain_Tilt` | aguarela com sulcos de cerda | WP | aguarela com gesto | CAP-01, 19 | P2 |
| I-9 | Granulation | Rb Granulation · Kr WaterC Round-Grain | Rb `Watercolor/Granulation/Granulation` † · Kr `j)_WaterC_Basic_Round-Grain` | o pigmento assenta nos vales do papel | WP | pigmentos granulantes | CAP-14 | P1 |
| I-10 | Fringe Bloom | CSP Soft bleed · Kr Watercolor Fringe · MP large_watercolor_fringe · Pc Water Bleed · CP Light Fringe | Kr `j)_Watercolor_Fringe` · MP `deevad/large_watercolor_fringe` | orla viva e «couve-flor» onde a água encontra a tinta | WP | o «acidente» da aguarela | CAP-14, 15 | P1 |
| I-11 | Spatter Watercolor | Rb Spatter · Fr Wet Spatter · PS Kyle's Spatter · CSP Watercolor splash | Rb `Watercolor/Spatter/Spatter` · MP `tanda/splatter-02` | salpicos molhados que se fundem | WP | salpicos, textura solta | CAP-08 | P1 |
| I-12 | Optical Wash | CP Digital Watercolor Simple Water · Pc Wet Glaze | Kr `j)_Waterpaint_Soft_Edges` | aguarela ÓPTICA (sem fluido): véu com orla instantânea | Wc | aguarela leve em tela grande | CAP-11, 22 | P2 |
| I-13 | Sumi-e | Rb Sumi-e · Kr Ink-8 Sumi-e (hairy) · CP Coarse Bristle Sumi-e | Rb `Watercolor/Sumi-e/Sumi-e  soft` † · Kr `d)_Ink-8_Sumi-e` | cerdas longas com tinta: cinzas e secos | WP | sumi-e, bambu | CAP-19 | P3 |

### J. Aerógrafo / Spray

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| J-1 | ✅ Airbrush | Pc Soft/Medium Airbrush · PS Airbrush · Kr Airbrush Soft · CP Digital Airbrush · AR Airbrush · MP airbrush | Rb `Airbrush/Default/Spray 2` † · Kr `b)_Airbrush_Soft` · MP `deevad/airbrush` | névoa fina que acumula enquanto se segura | WP | degradês, luz | — | P1 |
| J-2 | Spray Can | Pc Fat/Medium/Fine Nozzle · CP Coarse Spray · MP spray · Kr Texture Spray | MP `deevad/spray` · Kr `y)_Texture_Spray` · Rb `Airbrush/Default/Spray 3` | núcleo denso com gotículas em volta | Dg | graffiti, efeitos | CAP-08 | P1 |
| J-3 | Splatter | Pc Splatter / Flicks · PS Kyle's Spatter · CSP Droplet · MP spray2 | MP `deevad/spray2`, `tanda/splatter-04` | gotas opacas de tamanhos vários | Dg | salpicos, poeira, estrelas | CAP-08 | P1 |
| J-4 | Spray Drips | Pc Spraypaints Drip | doc. (Pc) | spray que escorre em fios | WP | graffiti, sujidade | CAP-12 | P2 |
| J-5 | Stipple | MP puantilism · Kr Texture Pointillism · CP Seurat · CSP Pointillism | MP `classic/puantilism` · Kr `v)_Texture_Pointillism` | pontos discretos | Dg | pontilhismo | CAP-08 | P3 |
| J-6 | Grainy Shader | Pc Bonobo Chalk · Pc Noise Brush (sombra) | doc. (Pc) | aerógrafo bem GRANULADO: sombra com ruído, sem banding | Dg | sombrear com textura | CAP-08, 27 | P2 |

### K. Texturas / Elementos / Carimbos

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| K-1 | Clouds | Pc Elements Clouds · PS Kyle's Concept · MP Clouds | MP `ramon/Clouds` | massas fofas de borda irregular | Dg | céus rápidos | CAP-08 | P1 |
| K-2 | Foliage | Kr Stamp Leaves · CSP Background Leaves · MP leaves | Kr `z)_Stamp_Leaves` · MP `experimental/leaves` | folhas dispersas com cor variada | Dg | árvores, arbustos | CAP-07, 08, 17 | P2 |
| K-3 | Grass | Kr Stamp Grass · Pc Wildgrass · MP long_grass | Kr `z)_Stamp_Grass` · MP `classic/long_grass` | lâminas de erva verticais | Dg | relva | CAP-07, 08, 17 | P2 |
| K-4 | Bokeh | Pc Bokeh Lights · Kr Stamp Bokeh · CSP Dappled light | Kr `z)_Stamp_Bokeh` | discos de luz desfocados, aditivos | Dg | luzes de fundo | CAP-05, 07, 17 | P2 |
| K-5 | Snow / Stars | Pc Driven Snow · Kr Texture Starfield | Kr `y)_Texture_Starfield` | pontos de luz dispersos | Dg | neve, céu noturno | CAP-08 | P3 |
| K-6 | Halftone | Kr Screentones · MP HalfTone · PS Kyle's Screentones · CSP tones | Kr `y)_Screentones_Regular` · MP `Dieterle/HalfTone#1` | retícula de pontos alinhada à tela | Dg | mangá, retro | CAP-20 | P2 |
| K-7 | Hatching | Kr Hatching · CSP Hatching · PS Kyle Crosshatchers | Kr `y)_Screentone_Moire` | hachuras paralelas automáticas | Dg | sombra a traço | CAP-20 | P2 |
| K-8 | Noise Grit | Pc Noise Brush · Pc Grit Roll · Kr Texture Noise | Kr `y)_Texture_Noise` | grão sujo sobreposto | Dg | textura de superfície (13.º mais citado do Pc) | CAP-27 | P2 |
| K-9 | Grunge | Rb Grunge · Pc Industrial/Textures Grunge | Rb `Oil/Grunge/Grunge` | manchas rasgadas, ferrugem | WP | sujidade, desgaste | CAP-06 | P3 |

### L. Mistura / Borrador / Borracha

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| L-1 | Blender Soft | Rb Blend Soft · Kr Blender Basic · Pc Soft Blend · MP blending | Rb `Blend/Default/Soft` · Kr `k)_Blender_Basic` · MP `deevad/blending` | funde as cores sem as arrastar longe | WP | pele, céu | CAP-09 | P1 |
| L-2 | Blender Grainy | Rb Blend Textured · Kr Blender Textured Soft · CP Grainy Water · CSP Textured blender · MP grainy_blending | Rb `Blend/Default/Textured` · MP `deevad/grainy_blending` | funde seguindo o grão do papel | WP | carvão e pastel esfumados | CAP-09 | P1 |
| L-3 | Smudge Finger | PS Smudge (Finger Painting) · CSP Finger tip · Kr Blender Smear · CP Smudge · Rb Smudge | Rb `Smudge/Default/Soft` · Kr `k)_Blender_Smear` | dedo: arrasta a cor ao longo do gesto | WP | esfumar, puxar cor | CAP-09 | P1 |
| L-4 | Smudge Spiky | Rb Spikey / Shattered | Rb `Smudge/Default/Spikey` | arrasto em riscos pontiagudos | WP | pêlo, erva puxada | CAP-09 | P2 |
| L-5 | Blur | Kr Filter Blur · CSP Blur · Rb Wet Blurred | Kr `x)_Filter_Blur` · Rb `Wet/Default/Blurred` | desfoca onde passa | Dg | profundidade | — | P2 |
| L-6 | Water Brush | CP Just Add Water · CSP Watery brush · Pc Water Pen · Rb Wet · MP water-01 | Rb `Wet/Default/Soft` · MP `tanda/water-01` | só água: reabre e dilui a tinta | WP | puxar aguarela/guache | CAP-15, 26 | P1 |
| L-7 | Dry / Lift | Rb Dry · CP Sponges · Fr Dry Layer | Rb `Dry/Default/Sponge` | seca ou levanta a tinta | WP | parar o escorrer, clarear | CAP-26 | P3 |
| L-8 | Kneaded Eraser | CSP Kneaded · MP kneaded_eraser · Kr Eraser Soft · CP Bleach | Rb `Eraser/Default/Round Soft` · MP `deevad/kneaded_eraser` · Kr `a)_Eraser_Soft` | macia: clareia em parte | Dg/WP | levantar luz em grafite | CAP-23 | P1 |
| L-9 | Hard Eraser | CSP Hard · Kr Eraser Circle · MP large_hard_eraser | Rb `Eraser/Default/Square Hard` · Kr `a)_Eraser_Circle` | dura, borda limpa | Dg/WP | corrigir, recortar | CAP-23 | P1 |

### M. Especiais / Efeitos

| # | Nome nosso | Origem | Oráculo que corre | Identidade | Motor | Razão de uso | Falta | P |
|---|---|---|---|---|---|---|---|---|
| M-1 | Glow | Pc Lightpen / Lightbrush (Luminance) · CP F-X Glow · MP glow, Glow_Airbrush | MP `experimental/glow`, `ramon/Glow_Airbrush` | luz aditiva: o centro estoura para branco | Dg | neon, magia | CAP-05 | P1 |
| M-2 | Dodge / Burn | Rb Marker Effect · Kr Adjust Dodge · CP Erasers Darkener | Rb `Marker/Effect/Dodge`, `Marker/Effect/Burn` · Kr `l)_Adjust_Dodge` | clareia/escurece o que já está pintado | WP | luz e sombra por cima | CAP-05 | P2 |
| M-3 | Color Glaze | Rb Marker Effect · Kr Adjust Color/Overlay · CP Tinting | Rb `Marker/Effect/Color`, `Marker/Effect/Overlay` · Kr `l)_Adjust_Color` | muda a cor sem mexer no valor | WP | ajustar tons | CAP-05 | P2 |
| M-4 | Metallic | Rb Metallic · Pc Materials Metallic | Rb `Oil/Metallic/Round` | tinta com brilho metálico | Im/WP | dourados, metais | CAP-18 | P2 |
| M-5 | Glitter | AR Glitter Tube · CP Fairy Dust · CSP Glitter | doc. (AR, Wine na entrega C) | flocos brilhantes multicor | Im | brilhos, decoração | CAP-07, 18 | P3 |
| M-6 | Particles | CP Particles Gravity/Flow/Spring · Kr Experimental Webs · MP DNA_brush | Kr `v)_Experimental_Webs` · MP `experimental/DNA_brush` | fios emitidos que caem, fluem ou ricocheteiam | Dg | efeitos orgânicos, fumo | CAP-17, 24 | P3 |
| M-7 | Sketchy Web | Kr Sketching Chrome · Kr Sketching Leaky (curve) | Kr `v)_Sketching-1_Chrome_Thin` | linhas que ligam aos pontos anteriores | Dg | rough «teia» | CAP-25 | P3 |

## §3 — Fusões por identidade (o que NÃO é linha própria)

| Fica como | Absorve (o mesmo traço noutro app) |
|---|---|
| A-1 Hard Round | Pc Hard Brush / Hard Airbrush / Medium Hard Brush · Rb Digital Round Hard |
| A-3 Round Pressure | PS Soft Round Pressure Size · Kr Basic-6 Details · MP detail_brush_thin · Pc Tamar |
| B-1 Graphite HB | Pc Procreate Pencil / Peppermint (contorno limpo) · Rb H · CP Pencils 2B |
| B-4 Graphite 8B | Pc 6B Pencil · CSP Darker pencil · Rb 4B (entre 2B e 8B: não justifica linha) |
| C-4 Studio Pen | Pc Syrup (chapado sem estria) · Pc Mercury · CSP G-pen · Kr Ink-3 Gpen · PS Kyle Basic Inker · MP pen |
| C-11 Dry Ink | Pc Thylacine / Tinderbox · CP Dry Ink · CSP Bit husky / Rough brush · Rb Ink Brush 3 |
| F-5 Soft Pastel | todo «Chalk» (Kr Chalk Soft/Grainy, AR Chalk, MP chalk, CP Blunt/Variable Chalk) — pastel seco e giz pintam igual |
| F-1 Charcoal | Pc Vine/Willow Charcoal · Kr Charcoal Rock Soft · Rb Charcoal 2/3 |
| G-1 Oil Bristle | Pc Old Brush / Oil Paint / Turpentine · Rb Bristle Hairy/Round · Kr Bristles-1..3 |
| G-12 Wet Blend Round | Pc Wet Acrylic · CP Sargent Brush / Real Wet Oil · Fr Live Oil Round · Kr Wet Bristles · PS Mixer «Wet» |
| I-1 Wet Round | Fr Live Watercolor Round Detail · CP Real Watercolor · Rb Round 2/3/4 |
| J-1 Airbrush | Pc Soft/Medium Airbrush · PS Airbrush Soft · Kr Airbrush Soft · MP BigAirbrush |
| L-3 Smudge Finger | PS Smudge · CSP Finger tip · Kr Blender Smear · MP smudge · Pc Smudge com Soft Airbrush |

**Recusados como linha** (ferramentas, não pincéis, ou fora do 2D de pintura): Clone/Cloners (já `PaintMode::Clone`),
Distort/Liquify (`PaintMode::Deform`), Fill/Shapes Fill (balde/laço), Normal Map (Kr), Paint Tube/Roller (AR: é
gesto de faca, cabe em G-7), Pattern Pens e Image Hose (cabem em K-* com CAP-17), Highlighter (= E-1 com CAP-11),
Lino/Dry Media (Rb, impressão), Procreate Materials/Abstract/Vintage/Organic (carimbos temáticos ou de pintura 3D; os
nomes estão no `01b` §1 se a janela 5 quiser alargar), Pixel Dithering (A-5 + padrão).

## §3b — O estado do EXAME, categorias A–E e J (JANELA 3, 2026-10-21)

✅ aprovados (na fábrica): A-1 Hard Round · A-2 Soft Round · A-5 Pixel 1px · A-6 Glaze Round · A-4 Flat Block-in ·
C-5 Mapping Pen. ◐ pré-existentes curados, não aprovados (decisão 3): B-1..B-4 grafites · B-8 Colored 4B · B-10
Scratch · C-1 Dip Pen · C-2 Ink Bullet · C-3 Ink Brush · E-1 Marker Bullet · E-2 Marker Liner · J-1 Airbrush.
✗ em estudo (sem ficheiro; o motor que falta em [`HANDOFF_JANELA_3`](HANDOFF_JANELA_3.md)): B-6 · B-7 · B-9 · B-11 ·
C-4 · C-6 · C-7 · C-9 · C-11 · C-12 · C-13 · C-14 · C-15 · D-2 · D-3 · D-4 · E-3 · E-4 · J-2 · J-3 · J-4 · J-5.
🔀 fundidos: A-3 → A-1 · B-5 → A-1 · C-8 → C-2 · C-10 → C-4 · D-1 → A-5 · J-6 → J-1. Por pincel (receita, prancha,
régua, vizinho, revisão cega, custo, portão): `exames/<família>.md`.

## §3c — O estado do EXAME, categorias F–I, K–M (JANELA 4, 2026-10-21)

✅ aprovados: G-5 Filbert · L-1 Blender Soft · L-8 Kneaded Eraser · L-9 Hard Eraser · K-4 Bokeh · K-5 Stars ·
K-6 Halftone · M-2 Dodge · M-2 Burn · M-3 Color Glaze · C-7 Ballpoint. 🔀 fundidos: G-2 → A-1 (saiu) · G-14 → A-1 ·
F-3 → F-5 · L-4 → L-3 · Hue/Overlay → M-3. ◐/✗ o resto, com o que falta, em [`HANDOFF_JANELA_4`](HANDOFF_JANELA_4.md)
e `exames/<família>.md`.

## §4 — Licenças por ARTEFACTO (medido 19/10 nesta máquina)

| Artefacto | Versão | Licença (fonte) | O que se pode fazer |
|---|---|---|---|
| `mypaint-brushes` (7 packs, 196 `.myb`) | 2.0.2-2 | **CC0** — `pacman -Qi`: custom:CC0; `/usr/share/licenses/mypaint-brushes/COPYING` = CC0 1.0; upstream `Licenses.dep5`: `brushes/*` CC0-1.0 («by policy, MyPaint's brush settings are released into the public domain») | **PORTAR** os valores (são paramétricos, sem texturas) |
| — `deevad/` (36) | v6 (2012) | CC0: `readme.txt` + `notes` de 35 ficheiros («license: CC-Zero/Public-Domain») | portar |
| — `Dieterle/` (19) | — | CC0: `notes` dos 19 + dep5 | portar |
| — `kaerhon_v1/` (20) | v1.1 | CC0: `ReadMe.txt` («licensed under the CC0») | portar |
| — `classic/` 35 · `experimental/` 23 · `ramon/` 28 · `tanda/` 35 | — | CC0: sem nota no ficheiro; o dep5 cobre `brushes/*`, `ramon`, `tanda` | portar |
| `libmypaint` | 1.6.1-2.1 | **ISC** (`pacman -Qi`) | correr E portar a lei — o harness da entrega C usa-a |
| `mypaint` (app) | 2.0.1-8.1 | GPL-2.0-or-later | só correr; código ⇒ `/pd-cleanroom` |
| `krita` (app, motores) | 6.0.4.1-1.1 | GPL3 | só correr (Python/linha de comando) |
| `Krita_4_Default_Resources.bundle` (117 `.kpp`, 80 pontas, 82 padrões) | — | **CC-0** no `meta.xml` (autor: Deevad + Ramon Miranda, Razvanc, Radian, Wolthera, Storm, Scottyp) | valores, pontas e padrões portáveis pela declaração; ⚠️ é declaração do bundle inteiro — conferir a origem de cada ponta antes de a copiar |
| `Krita_3_Default_Resources.bundle` (131 `.kpp`) | — | CC-0 (`meta.xml`) | idem |
| `RGBA_brushes.bundle` (6) | — | «Same as default brushes in Krita» (Ramon Miranda) | tratar como CC-0; confirmar com o autor antes de copiar |
| `Krita_Artists_SeExpr_examples.bundle` | — | **CC-BY-SA** | só correr (share-alike) |
| Krita presets soltos `/usr/share/krita/paintoppresets/` (32: `j)_WaterC_*`, `(mypaint)`) | 6.0.4 | sem declaração própria; vêm no pacote GPL3 | só correr até haver prova |
| Rebelle 8.3.4 (teste) | 8.3.4 | EULA proprietária | só correr; nunca copiar presets/texturas |
| Procreate · Photoshop · Fresco · CSP · Corel Painter · ArtRage | — | proprietárias (EULA) | correr (avaliação sob Wine, entrega C) ou «doc.»; copia-se o COMPORTAMENTO |

Motor de cada um dos 150 presets do Krita (lido no `paintopid` de cada `.kpp`): pixel 95 · color smudge 16 · spray 4 ·
deform 3 · sketch 2 · filter 2 · um de cada: round marker (`Basic-1`), hairy (`Ink-8 Sumi-e`), hatching, grid,
particle, curve, experiment (shape), duplicate (clone), tangent normal.
