# A área tapada — a foto 1 virada em número (2026-08-30)

> Era um dos dois pré-requisitos nomeados em
> [`01_o_estado_medido.md §6`](01_o_estado_medido.md). ⛔ Sem ele, *"os painéis tapam"* é gosto
> contra gosto; com ele, há um número antes e um número depois.
>
> ⚠️ **Tudo abaixo é derivado dos tokens e do código, não medido em pixels de captura de ecrã.**
> Cada constante tem o ficheiro ao lado.

## §1 — O alvo, e uma coincidência que não é coincidência

```
hero-viewport-w = 1366     hero-viewport-h = 1024        (docs/design/tokens.json)
```

⭐⭐ **1366 × 1024 pontos é exactamente o iPad Pro 12,9".** O viewport de referência do nosso
design system **é** o iPad. Toda a conta abaixo é, portanto, no alvo declarado do Enio — não num
desktop hipotético.

## §2 — As constantes (todas com dono)

| constante | valor | fonte |
|---|---:|---|
| `EDGE_PAD` | 14 | `tokens.json chrome.edge-pad` |
| `TOPBAR_H` | 64 | `chrome.topbar-h` |
| `TOPBAR_GAP` | 16 | `chrome.topbar-gap` |
| `HUD_H` / `HUD_BOTTOM_PAD` | 34 / 18 | `chrome.hud-h` / `hud-bottom-pad` |
| `HIERARCHY_W` | 308 | `chrome.hierarchy-w` |
| `INSPECTOR_W` | 304 | `chrome.inspector-w` |
| `INSPECTOR_MAX_H` | 880 | `screens/layout.rs:39` |
| `RULER_PX` | **20** | `ruler.rs:50` |
| `rail_w` (derivado) | **57** | `tool_rail.rs:85` = `CHIP_X_OFFSET(17) + chip Small(36) + Xs(4)` |

## §3 — ⭐⭐⭐ O mecanismo: a régua é pintada primeiro e o chrome por cima

Duas linhas de código, e a foto 1 sai delas.

**(a) O canvas é a janela inteira** — `screens/layout.rs:328`:

```rust
let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h);
```

⚠️ **E o doc-comment do campo, seis linhas antes (`layout.rs:208`), diz o contrário:**

> *"Visible canvas region (**between rail/inspector on the left and hierarchy on the right,
> between TopBar and HUD vertically**)."*

⛔ **O comentário descreve um layout ANCORADO; o código implementa full-bleed.** E o comentário
logo abaixo do `let canvas` admite-o: *"full width — **panels float over it**"*. A contradição
está no ficheiro, entre si.

**(b) As réguas são pintadas em primeiro lugar** — `screens/hero/paint.rs`:

| linha | o quê |
|---:|---|
| **265** | `ruler::paint_rulers(...)` |
| 420 | `paint_top_bar(...)` |
| 542 | `paint_left_rail(...)` |

⇒ e as réguas ancoram no canvas, que é a janela: `top_band = (canvas.x, canvas.y, canvas.w, 20)`
e `left_band = (canvas.x, canvas.y, 20, canvas.h)` (`ruler.rs:95` e `:101`).

⭐ **Logo: as réguas nascem por baixo de todo o chrome, por construção.** Não é um painel mal
posicionado — é a ordem de pintura mais a origem partilhada.

## §4 — Os números, no iPad Pro (1366 × 1024)

Faixa de chrome: `y ∈ [94, 964]`, altura 870.
Rects: barra superior `(14,14,1338,64)` · rail `(0,94,57,870)` · Hierarchy `(71,94,308,870)` ·
Inspector `(1048,94,304,870)` · HUD `(14,972,1338,34)`.

### 4.1 — As réguas

| régua | área | tapada por | px² | **%** |
|---|---:|---|---:|---:|
| **de cima** (1366 × 20) | 27 320 | barra superior | 8 028 | **29,4 %** |
| **da esquerda** (20 × 1024) | 20 480 | rail 17 400 + barra 384 + HUD 204 | 17 988 | **⛔ 87,8 %** |

⭐⭐⭐ **A régua da esquerda está 87,8 % tapada — e o culpado principal não é um painel
flutuante: é o rail de ferramentas**, que começa em `x = 0`, tem 57 px de largura e cobre os
20 px da régua ao longo de toda a faixa de chrome.

⚠️ **Isto corrige a leitura intuitiva da foto 1.** A queixa parece ser dos painéis; a aritmética
diz que o painel `Hierarchy` (começa em `x = 71`) **não toca a régua**. Quem a tapa é o rail —
que não flutua, é chrome fixo, e estaria lá em qualquer desenho ancorado. *Ancorar os painéis
não cura esta régua.*

### 4.2 — O canvas

| | px | % de 1366 |
|---|---:|---:|
| esquerda: rail 57 + margem 14 + Hierarchy 308 | 379 | 27,7 % |
| direita: Inspector 304 + margem 14 | 318 | 23,3 % |
| **largura total comida** | **697** | **⛔ 51,0 %** |

E em área: **713 154 px² de 1 398 784 = 51,0 % do canvas coberto por chrome.**

⭐⭐⭐ **Metade do iPad é chrome.** Com os dois painéis laterais abertos, o artista desenha em
49 % do ecrã — e as larguras são **fixas em px**, logo a fracção só piora em ecrã mais estreito.

## §5 — O que estes números decidem

1. **A régua tem de sair da origem do canvas.** Enquanto `left_band.x == canvas.x == 0` e o rail
   também estiver em `x = 0`, nenhuma reorganização de painéis a salva. ⇒ ou a régua vive
   **dentro da área de conteúdo** (Blender: a régua é uma *region* do editor, não do ecrã), ou o
   rail deixa de começar em zero. **É a mesma peça em falta do §5 do diagnóstico: regiões.**
2. **51 % é a barra de aceitação da spec nova.** Qualquer proposta de layout mede-se contra este
   número, no mesmo viewport. ⚠️ E o Enio escolheu **painéis ancorados** — ancorar *não reduz a
   área ocupada por si só* (um dock ocupa o mesmo que um flutuante); o que reduz é **colapsar,
   sobrepor por abas, ou ter menos coisa lá dentro**, que é a foto 3.
3. **A escala do Spectrum agrava isto, e tem de entrar na conta.** Se o alvo toque aumenta os
   controlos 1,25×, os 697 px passariam a ~871 px = **63,8 % da largura** se as larguras dos docks
   escalassem junto. ⛔ **Os docks não podem escalar com o alvo** — é uma restrição que a spec tem
   de declarar, não descobrir depois.

## §6 — ⏳ O que continua por medir

- **Estes números são o layout ANCORADO da `screens/layout.rs`.** Os painéis *realmente*
  flutuantes das fotos (o `3D Model`, o `Hierarchy` arrastado) têm posição de utilizador, que não
  é derivável do código — só de sessão real. ⇒ o 51 % é o **piso**, não o pior caso.
- **A régua de cima a 29,4 %**: a sobreposição geométrica é de 6 px. ⚠️ Não foi verificado se os
  *rótulos* da barra superior são desenhados acima do rect dela (`y < 14`), o que aumentaria o
  dano real. A foto 1 sugere que sim — os números da régua e os rótulos `SAVE`/`OPEN`/`IMG`
  aparecem entrelaçados —, mas isso é leitura de imagem, não medição.
