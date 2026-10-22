# A biblioteca de fábrica inteira — matriz, custo calmo e arrumação (J5, agente BIBLIOTECA, 2026-10-22)

> 45 presets em `assets/brushes/{Digital,WetPainter}/<meio>/<grupo>/`. Matriz: `exame.sh --matriz` (biblioteca
> pintada de novo, chave `7eba4984b4beaabf`, HEAD `953eb21e32`) → [`matriz/vizinhos.md`](../matriz/vizinhos.md),
> [`matriz.csv`](../matriz/matriz.csv). Custo: `exame.sh --custo "<nome>" 5,40,200`, um de cada vez pela
> `vez_pesada.sh`, perfil smoke, 60 Hz; o script esperava `load ≤ 4` antes de cada um e repetia se passasse (nenhum
> passou). **Load de todas as 135 medições: 1,0–3,4**.
> Estado do exame: [01_CATALOGO](../01_CATALOGO.md) §3b–§3c + os `exames/*.md` da J5.

## §1 — Todos os presets

✅ aprovado (18) · ◐ curado-não-aprovado (27). «Vizinho» = o 1.º NOSSO fora da própria família (as ferramentas
pela escala delas, `escala_ferramenta.json`). p95 = ms/quadro a pintar a 200 px; RSS = pico entre 5/40/200 px.

| meio | grupo | nome | exame | vizinho, d | p95 200 px | RSS MiB | load |
|---|---|---|---|---|---|---|---|
| Digital/Basic | Flat | Flat Block-in | ✅ | Pixel 1px 0,664 | 1,10 | 26 | 1,8–2,1 |
| Digital/Basic | Glaze | Glaze Round | ✅ | Wet Wash 1,163 | 1,04 | 31 | 1,5–1,6 |
| Digital/Basic | Hard | Hard Round | ✅ | Wet Liner 0,725 | 0,93 | 27 | 1,1–1,2 |
| Digital/Basic | Pixel | Pixel 1px | ✅ | Flat Block-in 0,664 | 0,44 | 27 | 2,1–2,6 |
| Digital/Basic | Soft | Soft Round | ✅ | Ballpoint 0,825 | 0,49 | 27 | 1,2–1,3 |
| Digital/Effects | Effects | Burn | ✅ | Glaze Round 1,460 | 0,99 | 31 | 1,4 |
| Digital/Effects | Effects | Color Glaze | ✅ | — (NaN) | 0,98 | 31 | 1,1–1,2 |
| Digital/Effects | Effects | Dodge | ✅ | — (NaN) | 1,03 | 31 | 1,1 |
| Digital/Eraser | Hard | Hard Eraser | ✅ | Kneaded Eraser 1,377 | 1,27 | 27 | 1,1–1,2 |
| Digital/Eraser | Soft | Kneaded Eraser | ✅ | Hard Eraser 1,377 | 0,40 | 28 | 1,1–1,2 |
| Digital/Ink | Ballpoint | Ballpoint | ✅ | Soft Round 0,825 | 1,49 | 32 | 2,2–2,5 |
| Digital/Ink | Mapping | Mapping Pen | ✅ | Pixel 1px 0,798 | 0,45 | 28 | 1,1 |
| Digital/Marker | Marker | Marker Liner | ◐ | Hard Round 1,685 | 1,35 | 35 | 1,8–2,0 |
| Digital/Stamp | Stamp | Bokeh | ✅ | Airbrush 2,333 | 0,26 | 28 | 1,5–1,6 |
| Digital/Stamp | Stamp | Stars | ✅ | Bokeh 2,776 | 0,21 | 28 | 1,1 |
| Digital/Texture | Texture | Halftone | ✅ | Soft Round 2,438 | 0,61 | 27 | 1,2 |
| Wet/Airbrush | Spray | Airbrush | ◐ | Square Hard 0,928 | 4,00 | 95 | 2,7–3,4 |
| Wet/Blend | Soft | Blender Soft | ✅ | Kneaded Eraser 2,883 | 5,53 | 97 | 1,7–2,0 |
| Wet/ExpressOil | Round | Express Oil | ◐ | Oil Pastel 0,952 | 4,31 | 97 | 1,2–1,3 |
| Wet/Ink | Brush | Ink Brush | ◐ | Dip Pen 0,876 | 12,54 | 97 | 1,1–1,4 |
| Wet/Ink | Pen | Dip Pen | ◐ | Ink Bullet 0,843 | 3,37 | 85 | 1,1 |
| Wet/Ink | Pen | Ink Bullet | ◐ | Dip Pen 0,843 | 4,24 | 86 | 1,2–1,3 |
| Wet/Marker | Marker | Marker Bullet | ◐ | Dip Pen 1,141 | 4,60 | 87 | 1,1–2,3 |
| Wet/Oil | Bristle | Oil Bristle | ◐ | Flat Block-in 1,054 | 8,88 | 101 | 1,6–2,8 |
| Wet/Oil | Dabber | Dabber | ◐ | Soft Pastel 0,818 | 6,29 | 105 | 1,1 |
| Wet/Oil | Flat | Filbert | ✅ | Dabber 0,891 | 9,74 | 102 | 1,2–2,5 |
| Wet/Pastel | Dry | Pastel Side | ✅ | Soft Pastel 1,093 | 1,99 | 91 | 1,8–3,0 |
| Wet/Pastel | Dry | Soft Pastel | ◐ | Wet Liner 0,773 | 7,03 | 103 | 1,3–1,6 |
| Wet/Pastel | Dry | Square Hard | ◐ | Wax Scrape 0,864 | 6,50 | 93 | 1,1–1,2 |
| Wet/Pastel | Oil | Oil Pastel | ◐ | Graphite 2B 0,777 | 5,26 | 105 | 2,1–2,6 |
| Wet/Pastel | Scrape | Wax Scrape | ◐ | Square Hard 0,864 | 6,66 | 104 | 1,0–1,1 |
| Wet/Pencil | Charcoal | Charcoal | ◐ | Graphite 2H 0,865 | 3,10 | 87 | 1,3 |
| Wet/Pencil | Colored | Colored 4B | ◐ | Oil Pastel 0,800 | 3,62 | 86 | 1,2 |
| Wet/Pencil | Graphite | Graphite 2B | ◐ | Oil Pastel 0,777 | 3,53 | 87 | 1,1 |
| Wet/Pencil | Graphite | Graphite 2H | ◐ | Charcoal 0,865 | 3,96 | 87 | 1,1 |
| Wet/Pencil | Graphite | Graphite 8B | ◐ | Soft Pastel 0,804 | 3,57 | 88 | 1,1–1,3 |
| Wet/Pencil | Graphite | Graphite HB | ◐ | Ballpoint 0,974 | 4,01 | 87 | 1,2 |
| Wet/Pencil | Scratch | Scratch | ◐ | Airbrush 1,002 | 6,95 | 104 | 1,7–2,0 |
| Wet/Watercolor | Fluid | Wet Drip | ◐ | Scratch 2,480 | 4,72 | 98 | 1,1–1,2 |
| Wet/Watercolor | Fluid | Wet Dry Brush | ◐ | Airbrush 1,218 | 5,71 | 103 | 1,1–1,2 |
| Wet/Watercolor | Fluid | Wet Flat | ◐ | Gouache Flat 0,763 | 4,23 | 94 | 1,1–2,5 |
| Wet/Watercolor | Fluid | Wet Liner | ◐ | Hard Round 0,725 | 4,42 | 99 | 1,8–2,3 |
| Wet/Watercolor | Fluid | Wet Round | ◐ | Glaze Round 1,326 | 5,71 | 101 | 1,7–1,8 |
| Wet/Watercolor | Fluid | Wet Wash | ◐ | Glaze Round 1,163 | 6,20 | 100 | 1,5–1,7 |
| Wet/Watercolor | Gouache | Gouache Flat | ◐ | Wet Flat 0,763 | 3,74 | 94 | 1,2–1,4 |

## §2 — Pares perto (limiar 0,591; passo de série 0,406)

**Nenhum par fora de família está abaixo de 0,591** (eram 6 em 28 na J2). Abaixo do limiar só ficam passos DENTRO da
série graduada do lápis, todos acima do passo visível:

| par | d | recomendação |
|---|---|---|
| Graphite 2B ~ Graphite HB | 0,483 | série graduada, fica (passo ≥ 0,406) |
| Colored 4B ~ Graphite 8B | 0,518 | série graduada, fica |
| Colored 4B ~ Graphite 2B | 0,562 | série graduada, fica |
| Graphite 2H ~ Graphite HB | 0,571 | série graduada, fica (era 0,43 na J2: abriu) |
| Flat Block-in ~ Pixel 1px | 0,664 | o par fora de família mais perto; acima do limiar — fica, os dois aprovados |

Os outros 2 passos não adjacentes da série (2B~2H 0,615 · 4B~HB 0,623) já estão acima do limiar. Família Marker:
Bullet ~ Liner 1,72 · família Wet: o menor passo Round ~ Wash 0,99. Fidelidade: a origem no Rebelle é o 1.º dos 228
em 13 de 26 (mediana da posição 1; era 2).

⚠️ **Defeito da bancada (não consertado: é ferramenta, não dado):** em `vizinhos.md` quatro linhas dizem «PERTO —
Airbrush (nan)»: **Dodge, Color Glaze, Hard Eraser, Kneaded Eraser**. É falso: `exame_matriz.py` ordena o NaN à
frente no top-3. As borrachas têm distância válida entre si (1,377) e ao Blender Soft (2,9–3,4); **Dodge e Color
Glaze são NaN contra os 44**: sobre papel branco sem base não deixam traço e não são `ferramenta` (o exame
deles, em `efeitos.md`, foi sobre base).
Cura (bancada): saltar NaN no top-3; efeitos com a cena «sobre».

## §3 — Custo e memória

**Nenhum preset passa o tecto** (p95 > 16,7 ms a 200 px ou RSS > 150 MiB). Os mais caros a 200 px: **Ink Brush
12,54 ms** (11/475 quadros > 16,7 ms — o único que se aproxima; a vigiar), Filbert 9,74 (7/475), Oil Bristle 8,88
(8/475), Soft Pastel 7,03, Scratch 6,95. RSS: Digital 26–35 MiB, Wet 85–105 MiB (o pico, Dabber e Oil Pastel 105).
A 5 e 40 px todos ≤ 2,6 ms de p95.

## §4 — Organização

- **Rótulos:** as 27 pastas (meio e grupo) têm braço em `rotulo_da_pasta`; o portão
  `cada_pasta_de_fabrica_tem_rotulo` (app-painter, `--features panel-painter-layers`) corre **verde** por nome.
  Nada em falta.
- **Miniaturas:** abertas 8 ao acaso (Graphite 8B, Stars, Wet Wash, Hard Eraser, Blender Soft, Dodge, Filbert,
  Pixel 1px): todas legíveis O portão `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` guarda-as.
- **Ordem no navegador** (`Biblioteca::carregar` ordena o caminho relativo; `publicar` põe o meio de agora à
  frente, estável): ordem ALFABÉTICA do caminho. Desarrumado, sem conserto por dado:
  1. **Graphite sai 2B · 2H · 8B · HB**, não no eixo 2H · HB · 2B · 8B; e o **Colored 4B** (membro da série pela
     origem) fica noutro grupo. Curar = renomear (o nome é chave da bancada e das receitas) ou ordem de
     série no `publicar` (código).
  2. **A família Marker está partida em dois meios**: Marker Liner em `Digital/Marker/Marker`, Marker Bullet em
     `WetPainter/Marker/Marker` — mudar de meio é mudar o motor, não arrumação.
  3. Grupos que repetem o meio: `Effects/Effects`, `Marker/Marker`, `Stamp/Stamp`, `Texture/Texture`. Cosmético.
  - **Arrumado:** nada — nenhum rótulo falta e a ordem não é valor de dados.
