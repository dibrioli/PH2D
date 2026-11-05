# As 12 fontes do Text do Pixel Art (`ph2d-pixel-text`)

Dono, 05/11: a lista do Text resume-se às fontes mais usadas em pixel art, todas dentro do app
(nenhuma do sistema). Embutidas por `include_bytes!` (`src/lib.rs`, `BUILTIN`). **Nenhum arquivo foi
mudado**, o que importa também para a Reserved Font Name da Press Start 2P (OFL).

Não existe ranking público de uso. A lista é a das famílias que se repetem nas coletâneas e no
Google Fonts, filtrada por licença verificável no próprio arquivo.

| Arquivo | Fonte | Autoria e licença | De onde (05/11, salvo indicado) |
|---|---|---|---|
| `cozette.bdf` | Cozette 6×13 | Samhain / Ines e contribuidores. **MIT** (`LICENSE-cozette.txt`) | github.com/the-moonwitch/Cozette, v.1.30.0 (02/11) |
| `DepartureMono-Regular.otf` | Departure Mono | Helena Zhang e Tobias Fried. **MIT** (`LICENSE-DepartureMono.txt`) | github.com/rektdeckard/departure-mono |
| `Micro5-Regular.ttf` | Micro 5 | The Micro 5 Project Authors. **OFL 1.1** (`LICENSE-Micro5.txt`) | github.com/google/fonts `ofl/micro5` |
| `PixelOperator.ttf` · `PixelOperator8.ttf` | Pixel Operator · 8 | Jayvee Enaguas. **CC0 1.0** (`LICENSE-PixelOperator.txt`) | github.com/bauripalash/ttf-pixeloperator-fork (o repositório original, notabug.org, devolve 404) |
| `PixelifySans[wght].ttf` | Pixelify Sans | The Pixelify Sans Project Authors. **OFL 1.1** (`LICENSE-PixelifySans.txt`) | github.com/google/fonts `ofl/pixelifysans` |
| `PressStart2P-Regular.ttf` | Press Start 2P | The Press Start 2P Project Authors. **OFL 1.1**, RFN «Press Start 2P» (`LICENSE-PressStart2P.txt`) | github.com/google/fonts `ofl/pressstart2p` |
| `Silkscreen-Regular.ttf` | Silkscreen | The Silkscreen Project Authors. **OFL 1.1** (`LICENSE-Silkscreen.txt`) | github.com/google/fonts `ofl/silkscreen` |
| `tom-thumb.bdf` | Tom Thumb 4×6 | Brian J. Swetland (1999), modificada por Robey Pointer (2010). **CC0 ou CC-BY 3.0**; as modificações, **MIT** | robey.lag.net/2010/01/23/tiny-monospace-font.html (02/11) |
| `unscii-8.hex` · `unscii-16.hex` | Unscii 8 · 16 | viznut. **Domínio público / CC0** (o `unscii-16-full`, derivado do Unifont e GPL, não está aqui) | viznut.fi/unscii (02/11) |
| `VT323-Regular.ttf` | VT323 | The VT323 Project Authors. **OFL 1.1** (`LICENSE-VT323.txt`) | github.com/google/fonts `ofl/vt323` |

## O em nativo de cada contorno

Medido no ASCII inteiro (`fontTools`): o erro é a maior distância de uma coordenada à grelha de
`unitsPerEm / em`, em píxeis.

| Fonte | em | erro máximo | curvas |
|---|---|---|---|
| Departure Mono | 11 | 0,000 | 0 |
| Micro 5 | 11 | 0,000 | 0 |
| Pixel Operator | 16 | 0,000 | 0 |
| Pixel Operator 8 | 8 | 0,000 | 0 |
| Press Start 2P | 8 | 0,000 | 0 |
| Silkscreen | 8 | 0,040 | 0 |
| VT323 | 16 | 0,496 | 3 404 |
| Pixelify Sans | 10 | 0,500 | 0 |

VT323 e Pixelify Sans não caem na grelha por desenho. Por isso o rasterizador faz o em nativo uma
vez e amplia por um inteiro: todo tamanho é o mesmo desenho. O em delas foi escolhido a olho sobre
`Rag3 sfy`: a Pixelify a 10 sai limpa (a 11 o «a» vira «o», a 12–16 os traços engrossam desiguais);
a VT323 a 16 sai melhor SEM hinting (com ele perde linhas do «a» e do «3»).

## ⛔ Recusas medidas

| Candidata | Por que ficou de fora |
|---|---|
| Jersey 10 | A grelha dá em 18,667: nenhum tamanho inteiro cai nela |
| DotGothic16 | Grelha de 19,6 e 2 MB |
| Tiny5 | Variável com 5 eixos, não medida |
| m5x7, m3x6, Monogram | Muito populares, mas sem licença verificável num arquivo baixável por script |
| Fontes do sistema | Decisão do dono, 05/11. A lista antiga ia às centenas |
