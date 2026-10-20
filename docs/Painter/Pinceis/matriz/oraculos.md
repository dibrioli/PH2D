# Os oráculos pela matriz — de onde sai a lei (gerado por `exame_matriz.py --recalibra`)

População: 228 presets do Rebelle (os que pintam as 3 cenas) · 34 do ArtRage · 28 nossos. Escala congelada em `matriz/escala.json` (o MAD de cada traço sobre o Rebelle, com o chão = o ruído do recto repetido).

## (c) O ruído do oráculo — o mesmo recto a 40 pintado duas vezes (recto_lento × recto_40)

| família | p50 | p75 (o chão da escala) | p90 | zeros |
|---|---|---|---|---|
| largura | 0.0000 | 0.0250 | 0.0750 | 65% |
| cobertura | 0.0053 | 0.0150 | 0.0425 | 2% |
| opacidade | 0.0039 | 0.0186 | 0.0510 | 42% |
| textura | 0.6150 | 2.4539 | 8.4292 | 1% |
| var_local | 0.0012 | 0.0101 | 0.0517 | 0% |
| rugosidade | 0.0791 | 0.2793 | 0.7181 | 2% |
| cv | 0.0044 | 0.0143 | 0.0450 | 2% |
| periodo_fracao | 0.0294 | 0.0561 | 0.1141 | 2% |
| pontas | 0.0011 | 0.0455 | 0.1010 | 49% |
| alem | 0.0656 | 0.1653 | 0.3872 | 25% |
| perfil | 0.0048 | 0.0224 | 0.0669 | 23% |
| cor | 0.0031 | 0.0102 | 0.0303 | 13% |

Em DISTÂNCIA (só os traços do recto, as famílias acima): o mesmo preset repintado dá p10 · p25 · p50 · p75 · p90 = **0.10 · 0.18 · 0.32 · 0.66 · 1.39**; o mesmo recto entre um preset e o seu vizinho mais perto: 0.26 · 0.42 · 0.67 · 1.12 · 1.55.

## (a) A distância ao vizinho mais perto dentro do Rebelle — p10 · p25 · p50 · p75 · p90

| ferramenta | n | qualquer vizinho | fora da própria série |
|---|---|---|---|
| AIRBRUSH | 5 | 0.44 · 0.44 · 1.00 · 1.00 · 1.08 | 0.84 · 0.90 · 1.00 · 1.07 · 1.11 |
| EXPRESS_OIL | 32 | 0.51 · 0.63 · 0.81 · 0.96 · 1.25 | 0.51 · 0.73 · 0.82 · 0.96 · 1.25 |
| INK_PEN | 10 | 0.91 · 1.03 · 1.17 · 1.33 · 1.64 | 0.91 · 1.03 · 1.17 · 1.40 · 1.64 |
| MARKER | 4 | 0.94 · 0.94 · 1.01 · 1.27 · 1.58 | 1.19 · 1.33 · 1.51 · 1.66 · 1.74 |
| OIL_AND_ACRYLIC | 62 | 0.46 · 0.64 · 0.86 · 1.12 · 1.42 | 0.51 · 0.64 · 0.89 · 1.12 · 1.42 |
| PASTEL | 24 | 0.75 · 0.81 · 0.99 · 1.24 · 1.47 | 0.75 · 0.81 · 0.99 · 1.24 · 1.47 |
| PENCIL | 44 | 0.00 · 0.22 · 0.78 · 0.93 · 1.14 | 0.63 · 0.67 · 0.83 · 0.96 · 1.17 |
| WATERCOLOR | 47 | 0.61 · 0.74 · 0.86 · 1.46 · 2.08 | 0.69 · 0.77 · 0.87 · 1.58 · 2.11 |
| TOTAL | 228 | 0.45 · 0.66 · 0.87 · 1.13 · 1.62 | 0.59 · 0.73 · 0.90 · 1.15 · 1.63 |
| ArtRage (dentro dele) | 34 | 0.37 · 0.78 · 0.96 · 1.10 · 1.93 | — |

**Duplicados** (d < 0.05): 5 pares — Colored Pencil/2B Color = Graphite Pencil/2B (0.000); Colored Pencil/2H Color = Graphite Pencil/2H (0.000); Colored Pencil/4B Color = Graphite Pencil/4B (0.000); Colored Pencil/8B Color = Graphite Pencil/8B (0.000); Colored Pencil/HB Color = Graphite Pencil/HB (0.000). ⚠️ Os «X Color» e os «Graphite Pencil/X» do Rebelle dão fotos IGUAIS AO BYTE com a cor da bateria (sha1 conferido): só a cor de omissão os separa — o Rebelle vende duplicados.

Os 15 pares mais perto do Rebelle (fora da família: a série unida aos duplicados ao byte):

- OIL_AND_ACRYLIC Round/Round soft ~ PENCIL Digital/Round Hard: 0.407
- EXPRESS_OIL Glaze/Liner ~ PENCIL Digital/Round Hard: 0.416
- OIL_AND_ACRYLIC Metallic/Round ~ PENCIL Digital/Round Hard: 0.455
- OIL_AND_ACRYLIC Round/Round Hard ~ PENCIL Digital/Round Hard: 0.459
- OIL_AND_ACRYLIC Knife/Knife Smooth 3 ~ OIL_AND_ACRYLIC Round/Hairy: 0.462
- EXPRESS_OIL Flat/Soft ~ EXPRESS_OIL Round/Rough Wet: 0.505
- OIL_AND_ACRYLIC Dabber/Dabber 2 ~ OIL_AND_ACRYLIC Metallic/Round 3: 0.507
- EXPRESS_OIL Grunge/Grunge ~ OIL_AND_ACRYLIC Metallic/Round: 0.511
- OIL_AND_ACRYLIC Metallic/Round 2 ~ PENCIL Digital/Round Hard: 0.517
- OIL_AND_ACRYLIC Knife/Knife Smooth 3 ~ OIL_AND_ACRYLIC Metallic/Flat 2: 0.534
- EXPRESS_OIL Flat/Flat thick ~ OIL_AND_ACRYLIC Flat/Flat Dry Soft: 0.538
- OIL_AND_ACRYLIC Dabber/Dabber ~ PENCIL Colored Pencil/8B Color: 0.547
- OIL_AND_ACRYLIC Dabber/Dabber ~ PENCIL Graphite Pencil/8B: 0.547
- OIL_AND_ACRYLIC Metallic/Flat ~ PENCIL Digital/Round Hard: 0.552
- OIL_AND_ACRYLIC Knife/Knife Smooth 2 ~ OIL_AND_ACRYLIC Metallic/Flat: 0.561

## (b) As séries graduadas do Rebelle — a distância entre membros VIZINHOS da série

| série | passos (membro → membro: d (o grupo que mais muda)) |
|---|---|
| AIRBRUSH Default/spray (numero) | Spray → Spray 2: 1.18 (textura) · Spray 2 → Spray 3: 0.44 (forma) · Spray 3 → Spray 4: 1.04 (textura) · Spray 4 → Spray 5: 1.72 (textura) |
| EXPRESS_OIL Flat/hairy (forma) | Hairy → Flat Hairy: 1.58 (forma) |
| EXPRESS_OIL Flat/knife dry (numero) | Knife Dry → Knife Dry 2: 1.58 (forma) |
| EXPRESS_OIL Glaze/· (forma) | Filbert → Liner: 1.34 (dinamica) · Liner → Round: 1.20 (dinamica) · Round → Wash: 0.87 (dinamica) |
| EXPRESS_OIL Grunge/grunge (numero) | Grunge → Grunge 2: 1.49 (textura) · Grunge 2 → Grunge 3: 1.14 (textura) · Grunge 3 → Grunge 4: 1.01 (textura) |
| EXPRESS_OIL Grunge/scatter (numero) | Scatter → Scatter 2: 2.04 (textura) |
| EXPRESS_OIL Grunge/shred (numero) | Shred → Shred 2: 1.04 (textura) · Shred 2 → Shred 3: 0.45 (forma) |
| EXPRESS_OIL Round/hairy (numero) | Hairy → Hairy 2: 1.76 (textura) |
| INK_PEN Brush/brush (numero) | Brush → Brush 2: 1.24 (textura) · Brush 2 → Brush 3: 1.94 (mistura) |
| INK_PEN Pen/fountain (forma) | Fountain Round → Fountain Square: 2.04 (mistura) |
| INK_PEN Pen/wood (numero) | Wood → Wood 2: 1.82 (mistura) |
| MARKER Marker/· (forma) | Bullet → Chisel: 1.70 (mistura) · Chisel → Liner: 0.94 (dinamica) |
| OIL_AND_ACRYLIC Bristle/bristle (numero+forma) | Bristle Flat → Bristle Flat 2: 0.96 (forma) · Bristle Flat 2 → Bristle Round: 1.11 (forma) · Bristle Round → Bristle Round 2: 1.00 (forma) · Bristle Round 2 → Bristle Wide: 1.12 (forma) |
| OIL_AND_ACRYLIC Dabber/dabber (numero) | Dabber → Dabber 2: 0.82 (dinamica) · Dabber 2 → Dabber 3: 0.93 (textura) · Dabber 3 → Dabber 4: 1.27 (forma) |
| OIL_AND_ACRYLIC Flat/· (forma) | Bright → Filbert: 1.80 (textura) · Filbert → Flat: 1.41 (forma) |
| OIL_AND_ACRYLIC Flat/dry (numero+forma) | Filbert Dry → Flat Dry: 1.15 (textura) · Flat Dry → Flat Dry 2: 1.38 (forma) |
| OIL_AND_ACRYLIC Flat/oily (numero) | Flat Oily → Flat Oily 2: 1.81 (forma) |
| OIL_AND_ACRYLIC Grunge/grunge (numero) | Grunge → Grunge 2: 1.21 (forma) · Grunge 2 → Grunge 3: 1.33 (forma) · Grunge 3 → Grunge 4: 1.26 (textura) |
| OIL_AND_ACRYLIC Grunge/scatter (numero) | Scatter → Scatter 2: 2.54 (textura) · Scatter 2 → Scatter 3: 2.29 (textura) |
| OIL_AND_ACRYLIC Knife/knife dry (numero) | Knife Dry → Knife Dry 2: 1.19 (forma) |
| OIL_AND_ACRYLIC Knife/knife smooth (numero) | Knife Smooth → Knife Smooth 2: 1.34 (forma) · Knife Smooth 2 → Knife Smooth 3: 0.64 (dinamica) |
| OIL_AND_ACRYLIC Metallic/· (numero+forma) | Flat → Flat 2: 0.28 (textura) · Flat 2 → Flat 3: 0.83 (dinamica) · Flat 3 → Round: 0.79 (dinamica) · Round → Round 2: 0.43 (dinamica) · Round 2 → Round 3: 0.62 (dinamica) |
| OIL_AND_ACRYLIC Round/hard (numero) | Round Hard → Round Hard 2: 1.31 (dinamica) |
| OIL_AND_ACRYLIC Round/soft (numero) | Round soft → Round soft 2: 0.77 (dinamica) |
| PASTEL Pastel Dry/· (numero) | Oval → Oval 2: 1.99 (textura) |
| PASTEL Pastel Dry/hard (forma) | Hard → Square Hard: 2.22 (dinamica) |
| PASTEL Pastel Dry/soft (numero+forma) | Flat Soft → Square Soft 2: 1.32 (forma) |
| PASTEL Pastel Oil/oil pastel (numero+forma) | Oil Pastel → Oil Pastel 2: 2.16 (dinamica) · Oil Pastel 2 → Oil Pastel 3: 1.26 (textura) · Oil Pastel 3 → Oil Pastel Flat: 1.70 (textura) |
| PASTEL Pastel Oil/wax soft (numero) | Wax Soft → Wax Soft  2: 2.34 (textura) |
| PASTEL Scrape/wax scrape (numero) | Wax Scrape → Wax Scrape 2: 1.35 (textura) |
| PENCIL Charcoal/charcoal (numero+forma) | Charcoal → Charcoal 2: 1.25 (dinamica) · Charcoal 2 → Charcoal 3: 1.61 (dinamica) · Charcoal 3 → Charcoal Flat: 1.55 (dinamica) · Charcoal Flat → Charcoal Flat 2: 1.64 (dinamica) · Charcoal Flat 2 → Charcoal Flat 3: 1.02 (textura) · Charcoal Flat 3 → Charcoal Liner: 1.99 (forma) · Charcoal Liner → Charcoal Wide: 2.91 (textura) |
| PENCIL Colored Pencil/color (dureza) | 2H Color → H Color: 0.28 (forma) · H Color → HB Color: 0.27 (forma) · HB Color → 2B Color: 0.41 (textura) · 2B Color → 4B Color: 0.43 (forma) · 4B Color → 8B Color: 0.46 (forma) |
| PENCIL Digital/soft (numero) | Round Soft → Round Soft 2: 0.37 (dinamica) |
| PENCIL Dry Media/powder (numero) | Powder → Powder 2: 2.20 (textura) |
| PENCIL Dry Media/textured (numero) | Textured → Textured 2: 1.13 (textura) · Textured 2 → Textured 3: 1.73 (textura) |
| PENCIL Graphite Pencil/· (dureza) | 2H → H: 0.17 (forma) · H → HB: 0.13 (forma) · HB → 2B: 0.41 (textura) · 2B → 4B: 0.43 (forma) · 4B → 8B: 0.46 (forma) |
| PENCIL Graphite Pencil/craft (numero+forma) | Craft → Craft 2: 0.65 (dinamica) · Craft 2 → Craft Flat: 1.00 (forma) · Craft Flat → Craft Flat 2: 0.83 (textura) · Craft Flat 2 → Craft Flat 3: 0.78 (dinamica) |
| WATERCOLOR Bristle/bristle (numero+forma) | Bristle Flat → Bristle Round: 1.81 (mistura) · Bristle Round → Bristle Round 2: 0.83 (dinamica) · Bristle Round 2 → Bristle Wide: 1.31 (dinamica) |
| WATERCOLOR Gouache/gouache (numero+forma) | Gouache Filbert → Gouache Filbert 2: 1.50 (dinamica) · Gouache Filbert 2 → Gouache Flat: 1.28 (mistura) · Gouache Flat → Gouache Flat 2: 1.13 (dinamica) · Gouache Flat 2 → Gouache Liner: 1.16 (dinamica) · Gouache Liner → Gouache Liner 2: 0.98 (mistura) · Gouache Liner 2 → Gouache Round: 1.05 (dinamica) · Gouache Round → Gouache Round 2: 1.51 (textura) · Gouache Round 2 → Gouache Wash: 1.72 (textura) |
| WATERCOLOR Granulation/granulation (numero) | Granulation → Granulation 2: 1.68 (textura) |
| WATERCOLOR Spatter/spatter (numero) | Spatter → Spatter  2: 2.13 (textura) · Spatter  2 → Spatter 3: 2.83 (textura) · Spatter 3 → Spatter 4: 2.33 (forma) · Spatter 4 → Spatter 5: 1.78 (forma) · Spatter 5 → Spatter 6: 2.37 (textura) |
| WATERCOLOR Spatter/splat (numero) | Splat → Splat 2: 1.40 (forma) |
| WATERCOLOR Watercolor/· (numero+forma) | Fan → Flat: 1.64 (dinamica) · Flat → Flat 2: 0.95 (dinamica) · Flat 2 → Wash Flat: 2.20 (dinamica) · Wash Flat → Liner: 2.53 (mistura) · Liner → Liner 2: 1.03 (dinamica) · Liner 2 → Mop: 1.96 (forma) · Mop → Round: 1.52 (textura) · Round → Round 2: 1.14 (textura) · Round 2 → Round 3: 1.08 (dinamica) · Round 3 → Round 4: 1.05 (mistura) · Round 4 → Wash Round: 1.47 (textura) |
| WATERCOLOR Watercolor/dry (forma) | Filbert Dry → Flat Dry: 1.65 (dinamica) |

Passos de série: 0.44 · 0.95 · 1.26 · 1.70 · 2.16 (n = 112).

## A lei (a proposta está no 04 §4)

limiar = p10 do vizinho mais perto no Rebelle, fora da série e dos duplicados = **0.591** (o recto repintado, só os traços do recto: mediana 0.317)
