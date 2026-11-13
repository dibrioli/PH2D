# N.º 7 rodada 4 — a chave `PH2D_ASSENTA_PALPITE=1` a olho (13/11)

Material para o dono decidir se o palpite da `assenta` do Wet (Newton com 0 voltas em vez de 4; sim a 6 traços
0,91 → 0,97 sim-s/s; erro máx 20 níveis, médio 0,9 — [nota](../N7_ITEM1_sim6.md)) pode virar o de fábrica.
Commit medido: `2c3a2185b0` (árvore da linha com edições alheias por comitar em `ph2d-tool-painter`; o motor
`ph2d-wet-paint` limpo).

## Como foi medido

Os dois alarmes de pincel por nome, o MESMO binário de teste, duas corridas (a chave é lida em execução):

```
bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --test it -- \
  a_largura_de_fabrica_e_a_aprovada a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje --nocapture
```

1. **exato**: `env -u PH2D_ASSENTA_PALPITE` (as 4 voltas de fábrica).
2. **palpite**: `PH2D_ASSENTA_PALPITE=1`, ecoada de dentro do processo (`chave no processo: PH2D_ASSENTA_PALPITE=1`).

Nunca `PH2D_PINCEIS_APROVA`. Os vermelhos de base comparam com a aprovação antiga (dívida anterior a esta rodada);
o que interessa é a DIFERENÇA entre as duas corridas.

## Vermelhos

| | exato | palpite |
|---|---|---|
| miniatura | 75 | 75 (a mesma lista) |
| largura | 2: Oil Wash, Smear Wet | 5: Oil Wash, **Smear Soft**, **Smear Spikey**, Smear Wet, **Water Brush** |
| **total** | **77** | **80** |

**A mais com a chave: Smear Soft, Smear Spikey, Water Brush** (largura). Valores com a chave (aprovada, tolerância
do alarme): Smear Soft / Smear Spikey `[3.53, 14.03, 51.33]` × `[3.49, 13.98, 51.58]`; Water Brush
`[4.58, 13.74, 52.64]` × `[4.58, 13.68, 51.86]`; Smear Wet `[5.34, 12.26, 50.19]` (exato `[4.95, 12.28, 50.18]`)
× `[5.03, 12.36, 49.62]`; Oil Wash sem aprovada.

**Iguais ao bit com e sem a chave:** 60 dos 75 vermelhos de miniatura, entre eles Wet Round e Watercolor Basic — o
palpite não lhes toca a miniatura.

### Miniaturas que o palpite pinta diferente (15; píxeis diferentes da aprovada)

| pincel | exato | palpite |
|---|---|---|
| `WetPainter/Blend/Dots/Blender Dots.ph2d-brush.png` | 2845 | 3171 |
| `WetPainter/Blend/Hard/Blender Hard.ph2d-brush.png` | 2590 | 3081 |
| `WetPainter/Blend/Soft/Blender Soft.ph2d-brush.png` | 2647 | 3072 |
| `WetPainter/Ink/Brush/Ink Brush.ph2d-brush.png` | 6573 | 6568 |
| `WetPainter/Ink/Nib/Broad Nib.ph2d-brush.png` | 204 | 369 |
| `WetPainter/Ink/Pen/Dip Pen.ph2d-brush.png` | 1023 | 1061 |
| `WetPainter/Ink/Pen/Ink Bullet.ph2d-brush.png` | 946 | 988 |
| `WetPainter/Marker/Marker/Marker Bullet.ph2d-brush.png` | 593 | 811 |
| `WetPainter/Marker/Marker/Marker Chisel.ph2d-brush.png` | 648 | 768 |
| `WetPainter/Smear/Flat/Smear Flat.ph2d-brush.png` | 3314 | 3376 |
| `WetPainter/Smear/Soft/Smear Soft.ph2d-brush.png` | 3076 | 3139 |
| `WetPainter/Smear/Spikey/Smear Spikey.ph2d-brush.png` | 3436 | 3512 |
| `WetPainter/Smear/Wet/Smear Wet.ph2d-brush.png` | 9523 | 11318 |
| `WetPainter/Water/Clean/Water Brush.ph2d-brush.png` | 10568 | 10714 |
| `WetPainter/Watercolor/Fluid/Wet Dry Brush.ph2d-brush.png` | 4641 | 4833 |

### As 75 miniaturas vermelhas (iguais nas duas corridas)

- `Digital/Airbrush/Soft/Soft Airbrush.ph2d-brush.png`
- `Digital/Basic/Glaze/Glaze Round.ph2d-brush.png`
- `Digital/Basic/Hard/Hard Round.ph2d-brush.png`
- `Digital/Ink/Brush/Brush Pen.ph2d-brush.png`
- `Digital/Ink/Mapping/Mapping Pen.ph2d-brush.png`
- `Digital/Marker/Marker/Marker Liner.ph2d-brush.png`
- `Digital/Oil/Thick paint/Dry Gouache.ph2d-brush.png`
- `Digital/Oil/Thick paint/Gouache Blender.ph2d-brush.png`
- `Digital/Oil/Thick paint/Gouache.ph2d-brush.png`
- `Digital/Oil/Thick paint/Oil Paint.ph2d-brush.png`
- `Digital/Oil/Thick paint/Paint And Apply.ph2d-brush.png`
- `Digital/Oil/Thick paint/Round Mixing.ph2d-brush.png`
- `Digital/Oil/Thick paint/Thick Oil Paint.ph2d-brush.png`
- `Digital/Oil/Thick paint/Thin Gouache.ph2d-brush.png`
- `Digital/Pastel/Chalk/Chalk.ph2d-brush.png`
- `Digital/Pastel/Pastel/Pastel.ph2d-brush.png`
- `Digital/Stamp/Stamp/Bokeh.ph2d-brush.png`
- `Digital/Stamp/Stamp/Stars.ph2d-brush.png`
- `Digital/Texture/Pointillism/Pointillism.ph2d-brush.png`
- `Digital/Texture/Texture/Halftone.ph2d-brush.png`
- `Digital/Watercolor/Splash/Watercolor Splash.ph2d-brush.png`
- `Digital/Watercolor/Wash/Wet Wash Mix.ph2d-brush.png`
- `Watercolor/Watercolor/Basic/Watercolor Basic.ph2d-brush.png`
- `WetPainter/Acrylic/Fan/Fan Brush.ph2d-brush.png`
- `WetPainter/Blend/Dots/Blender Dots.ph2d-brush.png`
- `WetPainter/Blend/Hard/Blender Hard.ph2d-brush.png`
- `WetPainter/Blend/Soft/Blender Soft.ph2d-brush.png`
- `WetPainter/Blend/Textured/Blender Textured.ph2d-brush.png`
- `WetPainter/Ink/Brush/Ink Brush.ph2d-brush.png`
- `WetPainter/Ink/Nib/Broad Nib.ph2d-brush.png`
- `WetPainter/Ink/Pen/Dip Pen.ph2d-brush.png`
- `WetPainter/Ink/Pen/Ink Bullet.ph2d-brush.png`
- `WetPainter/Marker/Marker/Marker Bullet.ph2d-brush.png`
- `WetPainter/Marker/Marker/Marker Chisel.ph2d-brush.png`
- `WetPainter/Oil/Dabber/Dabber 4.ph2d-brush.png`
- `WetPainter/Oil/Dabber/Dabber.ph2d-brush.png`
- `WetPainter/Oil/Flat/Filbert Dry.ph2d-brush.png`
- `WetPainter/Oil/Flat/Filbert.ph2d-brush.png`
- `WetPainter/Oil/Flat/Flat Bristle.ph2d-brush.png`
- `WetPainter/Oil/Flat/Oil Wash.ph2d-brush.png`
- `WetPainter/Oil/Flat/Thick Impasto.ph2d-brush.png`
- `WetPainter/Oil/Flat/Tilted Shade.ph2d-brush.png`
- `WetPainter/Oil/Knife/Knife Dry.ph2d-brush.png`
- `WetPainter/Oil/Knife/Knife Grainy.ph2d-brush.png`
- `WetPainter/Oil/Knife/Knife Oily.ph2d-brush.png`
- `WetPainter/Oil/Knife/Knife Smooth 3.ph2d-brush.png`
- `WetPainter/Oil/Knife/Knife Stain.ph2d-brush.png`
- `WetPainter/Oil/Knife/Knife Textured.ph2d-brush.png`
- `WetPainter/Oil/Metallic/Metallic Flat.ph2d-brush.png`
- `WetPainter/Pastel/Dry/Pastel Side.ph2d-brush.png`
- `WetPainter/Pastel/Dry/Soft Pastel.ph2d-brush.png`
- `WetPainter/Pastel/Dry/Square Hard.ph2d-brush.png`
- `WetPainter/Pastel/Oil/Oil Pastel.ph2d-brush.png`
- `WetPainter/Pastel/Oil/Wax Crayon.ph2d-brush.png`
- `WetPainter/Pastel/Scrape/Wax Scrape.ph2d-brush.png`
- `WetPainter/Pencil/Colored/Colored 4B.ph2d-brush.png`
- `WetPainter/Pencil/Graphite/Graphite 2B.ph2d-brush.png`
- `WetPainter/Pencil/Graphite/Graphite 2H.ph2d-brush.png`
- `WetPainter/Pencil/Graphite/Graphite 8B.ph2d-brush.png`
- `WetPainter/Pencil/Graphite/Graphite HB.ph2d-brush.png`
- `WetPainter/Pencil/Scratch/Scratch.ph2d-brush.png`
- `WetPainter/Smear/Flat/Smear Flat.ph2d-brush.png`
- `WetPainter/Smear/Shattered/Smear Shattered.ph2d-brush.png`
- `WetPainter/Smear/Soft/Smear Soft.ph2d-brush.png`
- `WetPainter/Smear/Spikey/Smear Spikey.ph2d-brush.png`
- `WetPainter/Smear/Wet/Smear Wet.ph2d-brush.png`
- `WetPainter/Water/Clean/Water Brush.ph2d-brush.png`
- `WetPainter/Watercolor/Fluid/Wet Drip.ph2d-brush.png`
- `WetPainter/Watercolor/Fluid/Wet Dry Brush.ph2d-brush.png`
- `WetPainter/Watercolor/Fluid/Wet Flat.ph2d-brush.png`
- `WetPainter/Watercolor/Fluid/Wet Liner.ph2d-brush.png`
- `WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png`
- `WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png`
- `WetPainter/Watercolor/Gouache/Gouache Flat.ph2d-brush.png`
- `WetPainter/Watercolor/Gouache/Gouache Opaque.ph2d-brush.png`

## Ficheiros

- Lado a lado (esquerda exato, direita palpite; em cima a miniatura, em baixo os traços de largura 4 · 14 · 52 px
  com o anel do cursor, quando o pincel está vermelho na largura):
  - [Blender_Dots_exato_vs_palpite.png](Blender_Dots_exato_vs_palpite.png)
  - [Blender_Hard_exato_vs_palpite.png](Blender_Hard_exato_vs_palpite.png)
  - [Blender_Soft_exato_vs_palpite.png](Blender_Soft_exato_vs_palpite.png)
  - [Broad_Nib_exato_vs_palpite.png](Broad_Nib_exato_vs_palpite.png)
  - [Dip_Pen_exato_vs_palpite.png](Dip_Pen_exato_vs_palpite.png)
  - [Ink_Brush_exato_vs_palpite.png](Ink_Brush_exato_vs_palpite.png)
  - [Ink_Bullet_exato_vs_palpite.png](Ink_Bullet_exato_vs_palpite.png)
  - [Marker_Bullet_exato_vs_palpite.png](Marker_Bullet_exato_vs_palpite.png)
  - [Marker_Chisel_exato_vs_palpite.png](Marker_Chisel_exato_vs_palpite.png)
  - [Oil_Wash_exato_vs_palpite.png](Oil_Wash_exato_vs_palpite.png)
  - [Smear_Flat_exato_vs_palpite.png](Smear_Flat_exato_vs_palpite.png)
  - [Smear_Soft_exato_vs_palpite.png](Smear_Soft_exato_vs_palpite.png)
  - [Smear_Spikey_exato_vs_palpite.png](Smear_Spikey_exato_vs_palpite.png)
  - [Smear_Wet_exato_vs_palpite.png](Smear_Wet_exato_vs_palpite.png)
  - [Water_Brush_exato_vs_palpite.png](Water_Brush_exato_vs_palpite.png)
  - [Wet_Dry_Brush_exato_vs_palpite.png](Wet_Dry_Brush_exato_vs_palpite.png)
- `exato/` e `palpite/`: as imagens soltas do relatório `target/relatorio-pinceis/` de cada corrida, mesmo nome
  (`<Pincel>_1-miniatura-1.png` = a miniatura que o motor pinta hoje; `<Pincel>_2-largura-{0,1,2}.png` = o traço
  a 4 · 14 · 52 px). Só as que diferem entre as corridas. Smear Soft, Smear Spikey e Water Brush não têm largura
  em `exato/`: passaram o alarme sem a chave, e o relatório só desenha quem falha.
