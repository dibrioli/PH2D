# Vizinhos mais perto — a matriz do EXAME (gerado por `exame_matriz.py`)

31 pincéis · 457 traços de identidade · distância = média dos 5 grupos (forma, tinta, textura, dinamica, mistura) da RMS do desvio robusto (saturado a 6).

**Calibração (os vereditos do dono, 19/10; cada um contra o vizinho entre os que ficaram):** Filbert ~ Soft Pastel = 0.741 («nada novo»); Flat Oily ~ Soft Pastel = 1.338 («bizarro»: diferente); Knife Smooth ~ Soft Pastel = 1.328 («bizarro»: diferente) ⇒ **limiar = 0.992** (média geométrica do maior igual e do menor diferente) · o par da receita Filbert × Acrylic Round = 1.903

⚠️ Na fábrica, 22 pincéis têm o vizinho abaixo do limiar (candidatos a «nada novo» à mesma largura e cor): Dip Pen ~ Ink Bullet 0.263; Ink Bullet ~ Dip Pen 0.263; Colored 4B ~ Oil Pastel 0.293; Oil Pastel ~ Colored 4B 0.293; Marker Liner ~ Ink Bullet 0.343; Wax Scrape ~ Oil Pastel 0.374; Graphite 2H ~ Graphite HB 0.386; Graphite HB ~ Graphite 2H 0.386; Graphite 8B ~ Colored 4B 0.435; Marker Bullet ~ Ink Bullet 0.448; Soft Pastel ~ Wax Scrape 0.452; Graphite 2B ~ Colored 4B 0.608; Square Hard ~ Oil Pastel 0.624; Express Oil ~ Colored 4B 0.700; Wet Round ~ Wet Wash 0.709; Wet Wash ~ Wet Round 0.709; Charcoal ~ Graphite HB 0.749; Wet Liner ~ Soft Pastel 0.756; Ink Brush ~ Express Oil 0.812; Gouache Flat ~ Wet Flat 0.937; Wet Flat ~ Gouache Flat 0.937; Dabber ~ Graphite 8B 0.972

| pincel | 1.º vizinho (d) | por grupo (forma · tinta · textura · dinâmica · mistura) | 2.º | 3.º | veredito |
|---|---|---|---|---|---|
| Acrylic Round | Wet Liner (1.358) | 0.90 · 0.48 · 3.13 · 1.50 · 0.78 | Square Hard (1.414) | Oil Pastel (1.509) | claramente longe |
| Airbrush | Wet Dry Brush (1.344) | 1.25 · 0.54 · 2.15 · 2.17 · 0.60 | Graphite 2B (1.367) | Scratch (1.388) | claramente longe |
| Charcoal | Graphite HB (0.749) | 0.68 · 1.01 · 0.64 · 0.54 · 0.88 | Graphite 2B (0.765) | Graphite 2H (0.881) | PERTO — «nada novo»? |
| Colored 4B | Oil Pastel (0.293) | 0.49 · 0.07 · 0.45 · 0.33 · 0.12 | Graphite 8B (0.435) | Wax Scrape (0.511) | PERTO — «nada novo»? |
| Dabber | Graphite 8B (0.972) | 1.50 · 0.54 · 0.95 · 1.06 · 0.81 | Soft Pastel (0.978) | Wax Scrape (0.983) | PERTO — «nada novo»? |
| Dip Pen | Ink Bullet (0.263) | 0.27 · 0.03 · 0.28 · 0.16 · 0.57 | Marker Liner (0.726) | Marker Bullet (0.804) | PERTO — «nada novo»? |
| Express Oil | Colored 4B (0.700) | 0.68 · 0.19 · 1.18 · 0.98 · 0.47 | Graphite 8B (0.717) | Graphite 2H (0.730) | PERTO — «nada novo»? |
| Filbert (saiu) | Soft Pastel (0.741) | 0.53 · 0.31 · 0.93 · 1.70 · 0.24 | Wax Scrape (1.007) | Graphite 8B (1.040) | PERTO — «nada novo»? |
| Flat Oily (saiu) | Soft Pastel (1.338) | 2.54 · 0.46 · 1.37 · 2.12 · 0.21 | Graphite 8B (1.536) | Wax Scrape (1.551) | claramente longe |
| Gouache Flat | Wet Flat (0.937) | 0.73 · 1.07 · 0.40 · 1.22 · 1.27 | Wet Liner (1.373) | Graphite HB (1.573) | PERTO — «nada novo»? |
| Graphite 2B | Colored 4B (0.608) | 0.52 · 0.85 · 0.65 · 0.42 · 0.60 | Oil Pastel (0.681) | Square Hard (0.703) | PERTO — «nada novo»? |
| Graphite 2H | Graphite HB (0.386) | 0.36 · 0.27 · 0.53 · 0.54 · 0.23 | Colored 4B (0.511) | Graphite 8B (0.553) | PERTO — «nada novo»? |
| Graphite 8B | Colored 4B (0.435) | 0.51 · 0.33 · 0.66 · 0.46 · 0.20 | Oil Pastel (0.436) | Wax Scrape (0.461) | PERTO — «nada novo»? |
| Graphite HB | Graphite 2H (0.386) | 0.36 · 0.27 · 0.53 · 0.54 · 0.23 | Colored 4B (0.526) | Graphite 8B (0.554) | PERTO — «nada novo»? |
| Ink Brush | Express Oil (0.812) | 0.51 · 0.51 · 0.89 · 0.97 · 1.18 | Graphite HB (0.818) | Wet Liner (0.868) | PERTO — «nada novo»? |
| Ink Bullet | Dip Pen (0.263) | 0.27 · 0.03 · 0.28 · 0.16 · 0.57 | Marker Liner (0.343) | Marker Bullet (0.448) | PERTO — «nada novo»? |
| Knife Smooth (saiu) | Soft Pastel (1.328) | 2.53 · 0.45 · 1.63 · 1.82 · 0.20 | Oil Bristle (1.515) | Wax Scrape (1.520) | claramente longe |
| Marker Bullet | Ink Bullet (0.448) | 0.49 · 0.03 · 0.44 · 0.64 · 0.64 | Marker Liner (0.569) | Dip Pen (0.804) | PERTO — «nada novo»? |
| Marker Liner | Ink Bullet (0.343) | 0.35 · 0.03 · 0.34 · 0.40 · 0.60 | Marker Bullet (0.569) | Dip Pen (0.726) | PERTO — «nada novo»? |
| Oil Bristle | Soft Pastel (1.233) | 2.64 · 0.43 · 1.08 · 1.45 · 0.56 | Dabber (1.266) | Graphite 8B (1.381) | claramente longe |
| Oil Pastel | Colored 4B (0.293) | 0.49 · 0.07 · 0.45 · 0.33 · 0.12 | Wax Scrape (0.374) | Graphite 8B (0.436) | PERTO — «nada novo»? |
| Scratch | Charcoal (1.165) | 1.00 · 0.79 · 1.89 · 0.82 · 1.31 | Graphite 2B (1.240) | Wet Round (1.280) | claramente longe |
| Soft Pastel | Wax Scrape (0.452) | 0.36 · 0.16 · 1.36 · 0.20 · 0.18 | Graphite 8B (0.472) | Oil Pastel (0.626) | PERTO — «nada novo»? |
| Square Hard | Oil Pastel (0.624) | 0.65 · 0.38 · 1.43 · 0.43 · 0.23 | Wax Scrape (0.664) | Colored 4B (0.676) | PERTO — «nada novo»? |
| Wax Scrape | Oil Pastel (0.374) | 0.56 · 0.31 · 0.35 · 0.33 · 0.32 | Soft Pastel (0.452) | Graphite 8B (0.461) | PERTO — «nada novo»? |
| Wet Drip | Wet Wash (2.600) | 3.48 · 0.41 · 2.61 · 4.14 · 2.36 | Wet Round (2.651) | Scratch (2.667) | claramente longe |
| Wet Dry Brush | Airbrush (1.344) | 1.25 · 0.54 · 2.15 · 2.17 · 0.60 | Graphite 2B (1.410) | Charcoal (1.428) | claramente longe |
| Wet Flat | Gouache Flat (0.937) | 0.73 · 1.07 · 0.40 · 1.22 · 1.27 | Wet Round (1.557) | Charcoal (1.587) | PERTO — «nada novo»? |
| Wet Liner | Soft Pastel (0.756) | 0.36 · 0.54 · 0.59 · 0.74 · 1.54 | Express Oil (0.859) | Ink Brush (0.868) | PERTO — «nada novo»? |
| Wet Round | Wet Wash (0.709) | 1.24 · 0.23 · 0.66 · 1.03 · 0.39 | Scratch (1.280) | Charcoal (1.325) | PERTO — «nada novo»? |
| Wet Wash | Wet Round (0.709) | 1.24 · 0.23 · 0.66 · 1.03 · 0.39 | Scratch (1.295) | Charcoal (1.531) | PERTO — «nada novo»? |
