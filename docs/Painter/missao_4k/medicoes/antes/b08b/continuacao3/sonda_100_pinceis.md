# B08.3 — os 100 pincéis de fábrica à velocidade do dono (sonda_custo_no_app)

Árvore limpa cd02ac77ef (`target/b08b-fonte`), `--profile smoke`, tela 1024² branca, o tamanho do preset, 9 eventos × 40 px por quadro (360 px/quadro, ~4 000 px/s em 60 Hz), 40 quadros por bloco, 2 rodadas intercaladas + 10 quadros de aquecimento; ms = o MÍNIMO das medianas das rodadas. ⚠️ load 13,4 → 18,0 (outros agentes a compilar; as 2 primeiras min. com o build do B08.4 por cima): leia como TECTO. Cru: [sonda_custo_no_app_100.txt](sonda_custo_no_app_100.txt). ✗ = acima de 16,7 ms/quadro.

| # | pincel | pasta | diâm px | depósito | composite | tique | TOTAL ms/quadro (mín) | p95 | quadros > 16,7 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | Filbert | WetPainter | 64 | 151.60 | 55.31 | 0.01 | **209.16 ✗** | 1095.85 | 80/80 |
| 2 | Knife Oily | WetPainter | 51 | 99.19 | 63.92 | 0.00 | **164.29 ✗** | 284.84 | 80/80 |
| 3 | Filbert Dry | WetPainter | 34 | 105.38 | 44.26 | 0.01 | **149.86 ✗** | 699.66 | 80/80 |
| 4 | Flat Bristle | WetPainter | 46 | 74.49 | 67.87 | 0.01 | **142.50 ✗** | 388.97 | 80/80 |
| 5 | Tilted Shade | WetPainter | 40 | 85.24 | 56.31 | 0.01 | **141.03 ✗** | 409.59 | 80/80 |
| 6 | Knife Dry | WetPainter | 51 | 87.17 | 48.64 | 0.01 | **136.88 ✗** | 590.79 | 80/80 |
| 7 | Scratch | WetPainter | 248 | 125.89 | 4.01 | 0.01 | **130.91 ✗** | 246.30 | 80/80 |
| 8 | Metallic Flat | WetPainter | 40 | 81.55 | 43.94 | 0.00 | **127.71 ✗** | 136.47 | 80/80 |
| 9 | Thick Impasto | WetPainter | 64 | 88.87 | 31.52 | 0.01 | **121.21 ✗** | 408.35 | 80/80 |
| 10 | Express Oil | WetPainter | 64 | 93.61 | 4.17 | 0.01 | **99.48 ✗** | 193.00 | 80/80 |
| 11 | Dabber | WetPainter | 51 | 37.90 | 57.93 | 0.01 | **96.24 ✗** | 1119.10 | 80/80 |
| 12 | Knife Smooth 3 | WetPainter | 51 | 51.86 | 39.81 | 0.00 | **92.33 ✗** | 166.39 | 80/80 |
| 13 | Knife Stain | WetPainter | 51 | 39.90 | 43.39 | 0.00 | **85.01 ✗** | 162.20 | 80/80 |
| 14 | Oil Paint | Digital | 80 | 84.29 | 0.00 | 0.00 | **84.29 ✗** | 90.31 | 80/80 |
| 15 | Gouache | Digital | 80 | 82.50 | 0.00 | 0.00 | **82.51 ✗** | 123.91 | 80/80 |
| 16 | Knife Textured | WetPainter | 51 | 36.85 | 42.79 | 0.00 | **80.45 ✗** | 91.03 | 80/80 |
| 17 | Dry Gouache | Digital | 80 | 79.77 | 0.00 | 0.00 | **79.77 ✗** | 91.74 | 80/80 |
| 18 | Knife Grainy | WetPainter | 51 | 36.26 | 39.92 | 0.00 | **76.56 ✗** | 402.13 | 80/80 |
| 19 | Wax Scrape | WetPainter | 141 | 68.67 | 0.88 | 0.00 | **69.82 ✗** | 80.30 | 80/80 |
| 20 | Dabber 4 | WetPainter | 51 | 8.06 | 52.28 | 0.01 | **61.24 ✗** | 420.89 | 80/80 |
| 21 | Water Brush | WetPainter | 20 | 49.43 | 0.60 | 0.00 | **50.53 ✗** | 90.66 | 80/80 |
| 22 | Pastel Side | WetPainter | 20 | 48.71 | 0.35 | 0.00 | **49.37 ✗** | 55.74 | 80/80 |
| 23 | Ink Brush | WetPainter | 20 | 44.23 | 0.51 | 0.00 | **46.40 ✗** | 75.66 | 80/80 |
| 24 | Round Mixing | Digital | 50 | 42.89 | 0.00 | 0.00 | **42.89 ✗** | 47.22 | 80/80 |
| 25 | Charcoal | WetPainter | 37 | 38.16 | 0.46 | 0.00 | **38.95 ✗** | 47.15 | 80/80 |
| 26 | Square Hard | WetPainter | 35 | 38.00 | 0.30 | 0.00 | **38.38 ✗** | 42.84 | 80/80 |
| 27 | Soft Pastel | WetPainter | 29 | 35.60 | 0.34 | 0.00 | **36.18 ✗** | 38.82 | 80/80 |
| 28 | Fan Brush | WetPainter | 48 | 34.21 | 0.33 | 0.00 | **34.77 ✗** | 138.37 | 80/80 |
| 29 | Paint And Apply | Digital | 40 | 34.46 | 0.00 | 0.00 | **34.46 ✗** | 36.45 | 80/80 |
| 30 | Oil Pastel | WetPainter | 77 | 33.43 | 0.50 | 0.00 | **34.12 ✗** | 38.55 | 80/80 |
| 31 | Thick Oil Paint | Digital | 60 | 33.73 | 0.00 | 0.00 | **33.73 ✗** | 34.95 | 80/80 |
| 32 | Broad Nib | WetPainter | 28 | 29.96 | 0.32 | 0.00 | **30.66 ✗** | 87.12 | 80/80 |
| 33 | Marker Chisel | WetPainter | 30 | 28.13 | 0.31 | 0.00 | **28.72 ✗** | 86.25 | 80/80 |
| 34 | Compressed Charcoal | WetPainter | 30 | 25.51 | 0.41 | 0.00 | **26.07 ✗** | 39.69 | 80/80 |
| 35 | Wet Wash Mix | Digital | 30 | 22.22 | 0.00 | 0.00 | **22.23 ✗** | 70.80 | 80/80 |
| 36 | Wet Dry Brush | WetPainter | 20 | 21.37 | 0.31 | 0.00 | **22.08 ✗** | 28.88 | 80/80 |
| 37 | Wet Drip | WetPainter | 20 | 20.76 | 0.27 | 0.00 | **21.23 ✗** | 62.19 | 80/80 |
| 38 | Brush Pen | Digital | 70 | 20.34 | 0.00 | 0.00 | **20.37 ✗** | 27.07 | 51/80 |
| 39 | Wax Crayon | WetPainter | 20 | 19.93 | 0.24 | 0.00 | **20.36 ✗** | 21.81 | 80/80 |
| 40 | Wet Round | WetPainter | 20 | 19.69 | 0.26 | 0.00 | **20.12 ✗** | 23.08 | 80/80 |
| 41 | Graphite HB | WetPainter | 6 | 13.80 | 0.23 | 0.00 | **19.81 ✗** | 56.93 | 80/80 |
| 42 | Marker Bullet | WetPainter | 20 | 19.15 | 0.34 | 0.00 | **19.79 ✗** | 57.04 | 80/80 |
| 43 | Graphite 2B | WetPainter | 6 | 13.64 | 0.23 | 0.00 | **19.61 ✗** | 58.14 | 79/80 |
| 44 | Gouache Blender | Digital | 80 | 19.31 | 0.00 | 0.00 | **19.31 ✗** | 21.68 | 76/80 |
| 45 | Wet Liner | WetPainter | 29 | 16.50 | 1.95 | 0.00 | **18.77 ✗** | 22.91 | 80/80 |
| 46 | Graphite 2H | WetPainter | 6 | 12.99 | 0.20 | 0.00 | **18.56 ✗** | 22.26 | 77/80 |
| 47 | Finger Tip | Digital | 30 | 18.33 | 0.00 | 0.00 | **18.33 ✗** | 72.29 | 80/80 |
| 48 | Wet Wash | WetPainter | 20 | 16.02 | 1.34 | 0.00 | **18.07 ✗** | 19.71 | 69/80 |
| 49 | Ink Bullet | WetPainter | 20 | 17.21 | 0.36 | 0.00 | **17.85 ✗** | 66.91 | 68/80 |
| 50 | Gouache Opaque | WetPainter | 20 | 16.10 | 0.32 | 0.00 | 16.63 | 19.11 | 43/80 |
| 51 | Airbrush | WetPainter | 20 | 15.87 | 0.34 | 0.00 | 16.57 | 39.79 | 47/80 |
| 52 | Dip Pen | WetPainter | 20 | 14.98 | 0.35 | 0.00 | 15.53 | 77.00 | 50/80 |
| 53 | Pastel | Digital | 40 | 14.84 | 0.00 | 0.00 | 14.84 | 16.35 | 3/80 |
| 54 | Wet Flat | WetPainter | 20 | 13.88 | 0.32 | 0.00 | 14.45 | 17.20 | 7/80 |
| 55 | Colored 4B | WetPainter | 6 | 12.93 | 0.25 | 0.00 | 14.24 | 17.10 | 7/80 |
| 56 | Gouache Flat | WetPainter | 20 | 13.18 | 0.25 | 0.00 | 13.73 | 16.09 | 3/80 |
| 57 | Smear Wet | WetPainter | 20 | 12.29 | 0.74 | 0.00 | 13.67 | 23.02 | 16/80 |
| 58 | Chalk | Digital | 42 | 13.33 | 0.00 | 0.00 | 13.33 | 14.78 | 1/80 |
| 59 | Round Watercolor | Digital | 80 | 12.40 | 0.00 | 0.00 | 12.40 | 40.89 | 36/80 |
| 60 | Flat Block-in | Digital | 35 | 11.79 | 0.00 | 0.00 | 11.79 | 13.65 | 0/80 |
| 61 | Graphite 8B | WetPainter | 6 | 10.05 | 0.31 | 0.00 | 11.59 | 13.97 | 1/80 |
| 62 | Splatter | Digital | 84 | 9.47 | 0.00 | 0.00 | 9.47 | 11.83 | 0/80 |
| 63 | Crayon | Digital | 30 | 9.25 | 0.00 | 0.00 | 9.25 | 9.92 | 0/80 |
| 64 | Smear Shattered | WetPainter | 20 | 7.72 | 0.72 | 0.00 | 9.23 | 24.90 | 11/80 |
| 65 | Smear Flat | WetPainter | 20 | 7.46 | 0.74 | 0.00 | 9.12 | 17.64 | 10/80 |
| 66 | Hard Round | Digital | 84 | 8.92 | 0.00 | 0.00 | 8.93 | 14.47 | 2/80 |
| 67 | Smear Soft | WetPainter | 20 | 7.46 | 0.73 | 0.00 | 8.81 | 13.15 | 0/80 |
| 68 | Smear Spikey | WetPainter | 20 | 7.03 | 0.68 | 0.00 | 8.37 | 10.36 | 0/80 |
| 69 | Smear Basic | Digital | 20 | 8.22 | 0.00 | 0.00 | 8.23 | 187.97 | 26/80 |
| 70 | Blender Hard | WetPainter | 20 | 7.03 | 0.37 | 0.00 | 7.80 | 9.48 | 0/80 |
| 71 | Watercolor Basic | Watercolor | 20 | 3.61 | 0.00 | 3.97 | 7.58 | 39.47 | 9/80 |
| 72 | Blender Soft | WetPainter | 20 | 7.04 | 0.34 | 0.00 | 7.53 | 9.52 | 0/80 |
| 73 | Hard Eraser | Digital | 20 | 6.76 | 0.00 | 0.00 | 6.77 | 14.57 | 2/80 |
| 74 | Thin Gouache | Digital | 20 | 6.45 | 0.00 | 0.00 | 6.45 | 8.06 | 0/80 |
| 75 | Watercolor Splash | Digital | 300 | 5.87 | 0.00 | 0.00 | 5.87 | 35.39 | 8/80 |
| 76 | Blender Dots | WetPainter | 20 | 5.07 | 0.36 | 0.00 | 5.68 | 7.63 | 0/80 |
| 77 | Pointillism | Digital | 80 | 5.55 | 0.00 | 0.00 | 5.55 | 10.45 | 0/80 |
| 78 | Soft Airbrush | Digital | 150 | 4.54 | 0.00 | 0.00 | 4.54 | 5.12 | 0/80 |
| 79 | Blender Textured | WetPainter | 20 | 3.49 | 0.56 | 0.00 | 4.40 | 30.56 | 7/80 |
| 80 | Relief Eraser | WetPainter | 20 | 4.36 | 0.00 | 0.00 | 4.37 | 12.32 | 2/80 |
| 81 | Charcoal Stick | Digital | 20 | 3.97 | 0.00 | 0.00 | 3.97 | 4.52 | 0/80 |
| 82 | Burn | Digital | 20 | 3.44 | 0.00 | 0.00 | 3.44 | 58.91 | 20/80 |
| 83 | Soft Round | Digital | 163 | 3.34 | 0.00 | 0.00 | 3.34 | 67.04 | 35/80 |
| 84 | Texture Eraser | Digital | 20 | 3.22 | 0.00 | 0.00 | 3.23 | 7.00 | 0/80 |
| 85 | Dodge | Digital | 20 | 3.14 | 0.00 | 0.00 | 3.14 | 14.43 | 1/80 |
| 86 | Halftone | Digital | 40 | 3.11 | 0.00 | 0.00 | 3.11 | 4.98 | 0/80 |
| 87 | Marker Liner | Digital | 20 | 3.06 | 0.00 | 0.00 | 3.06 | 3.49 | 0/80 |
| 88 | Blur Basic | Digital | 20 | 2.81 | 0.00 | 0.00 | 2.82 | 9.64 | 1/80 |
| 89 | Sketch Pencil | Digital | 10 | 2.41 | 0.00 | 0.00 | 2.41 | 3.16 | 0/80 |
| 90 | Stars | Digital | 108 | 2.31 | 0.00 | 0.00 | 2.32 | 3.60 | 0/80 |
| 91 | Color Glaze | Digital | 20 | 1.75 | 0.00 | 0.00 | 1.75 | 47.26 | 7/80 |
| 92 | Kneaded Eraser | Digital | 20 | 1.69 | 0.00 | 0.00 | 1.70 | 9.17 | 0/80 |
| 93 | G-Pen | Digital | 10 | 1.50 | 0.00 | 0.00 | 1.50 | 2.92 | 0/80 |
| 94 | Turnip Pen | Digital | 8 | 1.14 | 0.00 | 0.00 | 1.15 | 6.06 | 1/80 |
| 95 | Glaze Round | Digital | 14 | 1.08 | 0.00 | 0.00 | 1.09 | 1.75 | 0/80 |
| 96 | Ballpoint | Digital | 5 | 1.07 | 0.00 | 0.00 | 1.07 | 3.06 | 0/80 |
| 97 | Bokeh | Digital | 20 | 0.66 | 0.00 | 0.00 | 0.67 | 1.31 | 0/80 |
| 98 | Mapping Pen | Digital | 3 | 0.34 | 0.00 | 0.00 | 0.35 | 0.66 | 0/80 |
| 99 | Pixel 1px | Digital | 1 | 0.22 | 0.00 | 0.00 | 0.23 | 0.28 | 0/80 |
| 100 | Clone Basic | Digital | 20 | 0.04 | 0.00 | 0.00 | 0.04 | 0.08 | 0/80 |

49/100 acima de 16,7 ms/quadro
