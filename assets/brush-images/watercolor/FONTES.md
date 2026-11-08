# Fontes das imagens de pincel (aguarela)

Todas de **«CC0 Watercolor Textures»**, autor **PuzzleAndy**, OpenGameArt, publicado 25/05/2025.
Página (licença lida NA página, campo «License(s): CC0»): https://opengameart.org/content/cc0-watercolor-textures
Baixadas em 08/11/2026.

| Ficheiro | Original (URL) | sha256 do original | O que se fez |
|---|---|---|---|
| `splash_1.png`, `splash_2.png`, `splash_3.png` (256×256, cinza 8 bits, 255 = tinta) | https://opengameart.org/sites/default/files/watercolor_11_0.jpg (2553×1998) | `2b13cc18f3fa98e292b0f76fcbad324457e43cb1d196267983e8de4f57c49e55` | mapa «azul» `max(0, B−R)` + auto-level; recortes quadrados (na escala 1000 px de largura: `560,140` lado 180 · `90,440` lado 240 · `720,230` lado 240); `-level 38%,72%`; multiplicado por janela radial (sem cantos); redimensionado a 256; aparado ao conteúdo (limiar 4 %) e re-enquadrado em quadrado com 8 % de margem preta, 256² |

Preparação: ImageMagick 7 (`magick`), script em `docs/Painter/handoffs/material_2026-11-08_splash_wash/`.

Testado e NÃO usado (08/11): um grão de imagem da aguada a partir de `watercolor_7_0.jpg` na Wet Wash Mix — não aparecia no traço e custava ~4× o tempo; retirado.

## Limpeza (08/11, janela principal)
O dente do papel da foto aparecia como riscas dentro de cada mancha. Cada `splash_<i>.png` passou por:
`magick splash_<i>.png -statistic median 7x7 -blur 0x2.5 -level 12%,40% -blur 0x1.2` e uma orla ligeiramente
mais escura (`m*0.72 + (m − erode(m, Disk:5) desfocado 0x3)`), cinza de 8 bits.
