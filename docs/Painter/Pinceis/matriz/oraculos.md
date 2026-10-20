# Os oráculos pela matriz — de onde sai a lei (gerado por `exame_matriz.py --recalibra`)

População: 228 presets do Rebelle (os que pintam as 3 cenas) · 34 do ArtRage · 31 nossos. Escala congelada em `matriz/escala.json` (o MAD de cada traço sobre o Rebelle, com o chão = o ruído do recto repetido).

## (c) O ruído do oráculo — o mesmo recto a 40 pintado duas vezes (recto_lento × recto_40)

| família | p50 | p75 (o chão da escala) | p90 | zeros |
|---|---|---|---|---|
| largura | 0.0000 | 0.0250 | 0.1250 | 62% |
| cobertura | 0.0062 | 0.0226 | 0.0773 | 2% |
| opacidade | 0.0039 | 0.0225 | 0.0686 | 39% |
| textura | 0.7111 | 2.9022 | 10.3241 | 1% |
| var_local | 0.0015 | 0.0126 | 0.0719 | 0% |
| rugosidade | 0.0928 | 0.3504 | 0.8956 | 1% |
| cv | 0.0048 | 0.0187 | 0.0593 | 2% |
| periodo_fracao | 0.0302 | 0.0574 | 0.1086 | 2% |
| pontas | 0.0046 | 0.0455 | 0.0998 | 47% |
| alem | 0.0766 | 0.1939 | 0.4566 | 24% |
| perfil | 0.0056 | 0.0269 | 0.0816 | 22% |
| cor | 0.0035 | 0.0133 | 0.0678 | 12% |

Em DISTÂNCIA (só os traços do recto, as famílias acima): o mesmo preset repintado dá p10 · p25 · p50 · p75 · p90 = **0.10 · 0.18 · 0.33 · 0.70 · 1.50**; o mesmo recto entre um preset e o seu vizinho mais perto: 0.23 · 0.40 · 0.65 · 1.05 · 1.49.

## (a) A distância ao vizinho mais perto dentro do Rebelle — p10 · p25 · p50 · p75 · p90

| ferramenta | n | qualquer vizinho | fora da própria série |
|---|---|---|---|
| AIRBRUSH | 5 | 0.44 · 0.44 · 0.99 · 0.99 · 1.07 | 0.88 · 0.89 · 0.99 · 1.06 · 1.10 |
| EXPRESS_OIL | 32 | 0.49 · 0.64 · 0.80 · 0.99 · 1.25 | 0.50 · 0.71 · 0.82 · 0.99 · 1.25 |
| INK_PEN | 10 | 0.91 · 0.95 · 1.13 · 1.38 · 1.41 | 0.91 · 0.95 · 1.20 · 1.38 · 1.43 |
| MARKER | 4 | 0.95 · 0.95 · 0.96 · 1.16 · 1.49 | 0.95 · 0.95 · 0.96 · 1.16 · 1.49 |
| OIL_AND_ACRYLIC | 62 | 0.46 · 0.63 · 0.85 · 1.13 · 1.50 | 0.46 · 0.63 · 0.85 · 1.13 · 1.50 |
| PASTEL | 24 | 0.74 · 0.84 · 1.00 · 1.30 · 1.41 | 0.74 · 0.84 · 1.00 · 1.30 · 1.41 |
| PENCIL | 44 | 0.00 · 0.31 · 0.78 · 0.95 · 1.22 | 0.62 · 0.67 · 0.84 · 1.06 · 1.22 |
| WATERCOLOR | 47 | 0.63 · 0.80 · 1.13 · 1.52 · 1.95 | 0.72 · 0.82 · 1.13 · 1.52 · 1.98 |
| TOTAL | 228 | 0.44 · 0.67 · 0.90 · 1.19 · 1.53 | 0.60 · 0.72 · 0.91 · 1.19 · 1.53 |
| ArtRage (dentro dele) | 34 | 0.34 · 0.76 · 0.86 · 1.08 · 1.90 | — |

**Duplicados** (d < 0.05): 5 pares — Colored Pencil/2B Color = Graphite Pencil/2B (0.000); Colored Pencil/2H Color = Graphite Pencil/2H (0.000); Colored Pencil/4B Color = Graphite Pencil/4B (0.000); Colored Pencil/8B Color = Graphite Pencil/8B (0.000); Colored Pencil/HB Color = Graphite Pencil/HB (0.000). ⚠️ Os «X Color» e os «Graphite Pencil/X» do Rebelle dão fotos IGUAIS AO BYTE com a cor da bateria (sha1 conferido): só a cor de omissão os separa — o Rebelle vende duplicados.

Os 15 pares mais perto do Rebelle (fora da família: a série unida aos duplicados ao byte):

- OIL_AND_ACRYLIC Metallic/Flat 2 ~ OIL_AND_ACRYLIC Metallic/Round 2: 0.393
- OIL_AND_ACRYLIC Round/Round soft ~ PENCIL Digital/Round Hard: 0.407
- EXPRESS_OIL Glaze/Liner ~ PENCIL Digital/Round Hard: 0.418
- OIL_AND_ACRYLIC Metallic/Round ~ PENCIL Digital/Round Hard: 0.422
- OIL_AND_ACRYLIC Metallic/Flat ~ OIL_AND_ACRYLIC Metallic/Round 2: 0.448
- OIL_AND_ACRYLIC Knife/Knife Smooth 3 ~ OIL_AND_ACRYLIC Round/Hairy: 0.456
- OIL_AND_ACRYLIC Dabber/Dabber 2 ~ OIL_AND_ACRYLIC Metallic/Round 3: 0.457
- OIL_AND_ACRYLIC Round/Round Hard ~ PENCIL Digital/Round Hard: 0.463
- EXPRESS_OIL Grunge/Grunge ~ OIL_AND_ACRYLIC Metallic/Round: 0.494
- EXPRESS_OIL Flat/Soft ~ EXPRESS_OIL Round/Rough Wet: 0.494
- EXPRESS_OIL Flat/Flat thick ~ OIL_AND_ACRYLIC Flat/Flat Dry Soft: 0.528
- OIL_AND_ACRYLIC Metallic/Round 2 ~ OIL_AND_ACRYLIC Round/Round soft 2: 0.546
- OIL_AND_ACRYLIC Knife/Knife Smooth 2 ~ OIL_AND_ACRYLIC Metallic/Flat: 0.554
- OIL_AND_ACRYLIC Round/Hairy ~ PENCIL Colored Pencil/8B Color: 0.562
- OIL_AND_ACRYLIC Round/Hairy ~ PENCIL Graphite Pencil/8B: 0.562

## (b) As séries graduadas do Rebelle — a distância entre membros VIZINHOS da série

| série | passos (membro → membro: d (o grupo que mais muda)) |
|---|---|
| AIRBRUSH Default/Spray (numero) | Spray → Spray 2: 1.15 (textura) · Spray 2 → Spray 3: 0.44 (forma) · Spray 3 → Spray 4: 1.03 (textura) · Spray 4 → Spray 5: 1.72 (textura) |
| EXPRESS_OIL Flat/Knife Dry (numero) | Knife Dry → Knife Dry 2: 1.59 (forma) |
| EXPRESS_OIL Grunge/Grunge (numero) | Grunge → Grunge 2: 1.43 (textura) · Grunge 2 → Grunge 3: 0.66 (textura) · Grunge 3 → Grunge 4: 1.27 (textura) |
| EXPRESS_OIL Grunge/Scatter (numero) | Scatter → Scatter 2: 2.05 (textura) |
| EXPRESS_OIL Grunge/Shred (numero) | Shred → Shred 2: 1.02 (textura) · Shred 2 → Shred 3: 0.44 (forma) |
| EXPRESS_OIL Round/Hairy (numero) | Hairy → Hairy 2: 1.67 (textura) |
| INK_PEN Brush/Brush (numero) | Brush → Brush 2: 1.22 (textura) · Brush 2 → Brush 3: 1.80 (dinamica) |
| INK_PEN Pen/Wood (numero) | Wood → Wood 2: 1.81 (mistura) |
| OIL_AND_ACRYLIC Bristle/Bristle Flat (numero) | Bristle Flat → Bristle Flat 2: 0.91 (forma) |
| OIL_AND_ACRYLIC Bristle/Bristle Round (numero) | Bristle Round → Bristle Round 2: 0.97 (dinamica) |
| OIL_AND_ACRYLIC Dabber/Dabber (numero) | Dabber → Dabber 2: 0.98 (forma) · Dabber 2 → Dabber 3: 0.93 (textura) · Dabber 3 → Dabber 4: 1.21 (forma) |
| OIL_AND_ACRYLIC Flat/Flat Dry (numero) | Flat Dry → Flat Dry 2: 1.37 (forma) |
| OIL_AND_ACRYLIC Flat/Flat Oily (numero) | Flat Oily → Flat Oily 2: 1.82 (forma) |
| OIL_AND_ACRYLIC Grunge/Grunge (numero) | Grunge → Grunge 2: 1.22 (forma) · Grunge 2 → Grunge 3: 1.34 (forma) · Grunge 3 → Grunge 4: 1.25 (textura) |
| OIL_AND_ACRYLIC Grunge/Scatter (numero) | Scatter → Scatter 2: 2.05 (textura) · Scatter 2 → Scatter 3: 2.31 (textura) |
| OIL_AND_ACRYLIC Knife/Knife Dry (numero) | Knife Dry → Knife Dry 2: 1.17 (forma) |
| OIL_AND_ACRYLIC Knife/Knife Smooth (numero) | Knife Smooth → Knife Smooth 2: 1.34 (forma) · Knife Smooth 2 → Knife Smooth 3: 0.64 (dinamica) |
| OIL_AND_ACRYLIC Metallic/Flat (numero) | Flat → Flat 2: 0.26 (textura) · Flat 2 → Flat 3: 0.81 (dinamica) |
| OIL_AND_ACRYLIC Metallic/Round (numero) | Round → Round 2: 0.43 (dinamica) · Round 2 → Round 3: 0.61 (dinamica) |
| OIL_AND_ACRYLIC Round/Round Hard (numero) | Round Hard → Round Hard 2: 1.32 (dinamica) |
| OIL_AND_ACRYLIC Round/Round soft (numero) | Round soft → Round soft 2: 0.74 (dinamica) |
| PASTEL Pastel Dry/Oval (numero) | Oval → Oval 2: 2.01 (textura) |
| PASTEL Pastel Oil/Oil Pastel (numero) | Oil Pastel → Oil Pastel 2: 2.15 (dinamica) · Oil Pastel 2 → Oil Pastel 3: 1.25 (textura) |
| PASTEL Scrape/Wax Scrape (numero) | Wax Scrape → Wax Scrape 2: 1.35 (textura) |
| PENCIL Charcoal/Charcoal (numero) | Charcoal → Charcoal 2: 1.24 (dinamica) · Charcoal 2 → Charcoal 3: 1.81 (dinamica) |
| PENCIL Charcoal/Charcoal Flat (numero) | Charcoal Flat → Charcoal Flat 2: 1.65 (dinamica) · Charcoal Flat 2 → Charcoal Flat 3: 1.02 (textura) |
| PENCIL Colored Pencil/Color (dureza) | 2H Color → H Color: 1.08 (dinamica) · H Color → HB Color: 1.10 (dinamica) · HB Color → 2B Color: 0.41 (textura) · 2B Color → 4B Color: 0.43 (forma) · 4B Color → 8B Color: 0.45 (forma) |
| PENCIL Digital/Round Soft (numero) | Round Soft → Round Soft 2: 0.37 (dinamica) |
| PENCIL Dry Media/Powder (numero) | Powder → Powder 2: 2.23 (textura) |
| PENCIL Dry Media/Textured (numero) | Textured → Textured 2: 1.11 (textura) · Textured 2 → Textured 3: 1.74 (textura) |
| PENCIL Graphite Pencil/· (dureza) | 2H → H: 0.17 (forma) · H → HB: 0.13 (forma) · HB → 2B: 0.41 (textura) · 2B → 4B: 0.43 (forma) · 4B → 8B: 0.45 (forma) |
| PENCIL Graphite Pencil/Craft (numero) | Craft → Craft 2: 0.67 (dinamica) |
| PENCIL Graphite Pencil/Craft Flat (numero) | Craft Flat → Craft Flat 2: 0.84 (textura) · Craft Flat 2 → Craft Flat 3: 0.77 (dinamica) |
| WATERCOLOR Bristle/Bristle Round (numero) | Bristle Round → Bristle Round 2: 1.13 (dinamica) |
| WATERCOLOR Gouache/Gouache Filbert (numero) | Gouache Filbert → Gouache Filbert 2: 2.23 (mistura) |
| WATERCOLOR Gouache/Gouache Flat (numero) | Gouache Flat → Gouache Flat 2: 1.12 (dinamica) |
| WATERCOLOR Gouache/Gouache Liner (numero) | Gouache Liner → Gouache Liner 2: 1.61 (dinamica) |
| WATERCOLOR Gouache/Gouache Round (numero) | Gouache Round → Gouache Round 2: 1.51 (textura) |
| WATERCOLOR Granulation/Granulation (numero) | Granulation → Granulation 2: 1.69 (textura) |
| WATERCOLOR Spatter/Spatter (numero) | Spatter → Spatter 3: 2.55 (textura) · Spatter 3 → Spatter 4: 2.43 (textura) · Spatter 4 → Spatter 5: 1.57 (forma) · Spatter 5 → Spatter 6: 2.16 (textura) |
| WATERCOLOR Spatter/Splat (numero) | Splat → Splat 2: 1.68 (dinamica) |
| WATERCOLOR Watercolor/Flat (numero) | Flat → Flat 2: 1.47 (dinamica) |
| WATERCOLOR Watercolor/Liner (numero) | Liner → Liner 2: 0.96 (dinamica) |
| WATERCOLOR Watercolor/Round (numero) | Round → Round 2: 1.09 (dinamica) · Round 2 → Round 3: 1.54 (textura) · Round 3 → Round 4: 1.40 (textura) |

Passos de série: 0.43 · 0.81 · 1.22 · 1.65 · 2.05 (n = 77).

## A lei (a proposta está no 04 §4)

limiar = p10 do vizinho mais perto no Rebelle, fora da série e dos duplicados = **0.596** (o recto repintado, só os traços do recto: mediana 0.329)
