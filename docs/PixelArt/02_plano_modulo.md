# 02 — Plano do módulo Pixel Art da PH2D

> Data: 2026-10-25. Leitor: a LLM que abre a linha. Insumos: [01_estudo_aseprite.md](01_estudo_aseprite.md)
> (o QUE, pela referência da indústria), o MVP web em [`../Pixel Art/`](../Pixel%20Art/) (as LEIS
> numéricas e 612 testes com goldens — [02_spec_engine_e_porte.md](../Pixel%20Art/02_spec_engine_e_porte.md)),
> e a anatomia de módulo da casa (Luz2D, Quadro, CodeEditor).

## §0 — Decisões do dono (25/10)

1. **Módulo NOVO e AUTOCONTIDO.** Não é parte do Painter nem depende dele (o Painter está em
   desenvolvimento). ⇒ ⛔ a §2/§11 do `02_spec_engine_e_porte.md` do MVP, que mapeava o porte para
   planos sidecar no `PainterTool`, está **revogada**. Um diálogo futuro Painter ↔ Pixel será por
   ponte explícita (importar/exportar imagem), nunca por tipo partilhado.
2. **O Aseprite é a espinha das etapas.** Cada onda abaixo cita a área do Aseprite que ela cobre e
   termina quando o artista faz ali **o que faria no Aseprite**, pelos mesmos atalhos por omissão.
3. **O que nos diferencia vem depois da paridade-base** (§P13): rampas, luz na rampa, dither,
   cycling, linter (do MVP) e os pedidos que o Aseprite não atende (áudio, tween, auto-outline).

## §1 — Arquitetura (ADR a abrir: o próximo número livre, hoje `0192`)

| Crate | Papel | Dependências |
|---|---|---|
| `ph2d-pixel-core` | **O documento e as leis**: `Sprite` (modo de cor RGBA/Gray/Indexed, paleta, árvore de camadas, frames, cels esparsos com bbox/opacidade/z-index/link, tags, slices, tilesets, user data), **ops com inverso** (`apply(doc, op) -> inverse`), kernels inteiros (Bresenham canonicalizado, pixel-perfect, elipse por bbox, tabela de stamps, flood fill com tolerância/conectividade, gradiente com dither, RotSprite, resize nearest), compose. | **nenhuma** (como `ph2d-macro`); sem relógio, sem `rand`, sem SO |
| `ph2d-pixel-io` | `.ase` **ler e escrever** (spec pública), folha + JSON compatível Aseprite (`json-hash`/`json-array`), GIF/APNG/PNG/sequência via `ph2d-imageio-*`, paletas `.gpl .pal .hex .act` | core, `ph2d-imageio-*`, `ph2d-aseprite` (ver §4) |
| `ph2d-pixel-render` | GPU: cel → textura (`R8` índice ou `RGBA8`), LUT 256×1, compose com os 19 blend modes num shader, **zoom inteiro nearest**, grade/pixel grid, xadrez, onion skin, Tiled Mode 3×3 | core, `ph2d-render`/wgpu |
| `ph2d-panel-pixel-*` | color bar/paleta, **timeline camadas×frames**, barra de contexto, barra de ferramentas, preview | editor-core, tokens, i18n |
| `ph2d-app-pixel` | a família (`const FAMILY`): objeto **Pixel Sprite** no Add, modos Object/Edit (`Tab`), ferramentas, entrada (rato + `ph2d-pen`), undo, persistência, roteador `PH2D_PIXEL_SMOKE` | todas acima; **nunca** `ph2d-app-painter` |

Leis de arquitetura:
- **Ferramentas são internas à família** (um trait próprio em `ph2d-app-pixel`), ⛔ não entram no
  contrato congelado `Tool=12` (CLAUDE §6) — ele é das ferramentas de bitmap da casa.
- **Persistência:** um campo-blob `Vec<u8>` com versão própria no `ProjectFile` (como `boards`):
  **um** degrau de `PROJECT_SCHEMA`, contado por `scripts/schema-recount.py`, nunca escolhido.
  Dentro do blob, a versão é nossa (migração = função pura).
- **Undo:** o documento tem ops com inverso (lei do MVP: um gesto = um op, cap em bytes). ⚠️ Como
  ele entra na fila única do editor (diff do `ProjectState`) ou fica em fila própria como o
  Motion/Timeline é decisão do ADR, **com medição**: um cel 256×256 RGBA = 256 KiB por passo; o
  diff de componente inteiro por quadro não serve para traço contínuo.
- **Render:** não há na casa caminho pronto «buffer de pixels com zoom inteiro»
  (`ImageFilterMode::PixelArt` só escolhe `Nearest`). É peça nova, em `ph2d-pixel-render`.
- **Caneta:** `crates/ph2d-pen` (ADR-0189) já existe; a dinâmica (P7) usa-a.

## §2 — Leis herdadas do MVP (não se renegociam)

Do [`02_spec_engine_e_porte.md`](../Pixel%20Art/02_spec_engine_e_porte.md) §3–§15, validadas em 28
ondas e smokes: kernels **inteiros** (não existe meio pixel) · RNG `splitmix32` semeado · arredondar
= `floor(x+0.5)` · iteração em ordem definida (`BTreeMap`) e save byte-determinístico · linha
**AB == BA** · pixel-perfect de Ricky Han · elipse por bbox simétrica por construção · **tabela de
stamps autorada** (1..10 com golden individual) · flood fill scanline sem recursão · RotSprite com o
original fixo durante o preview · reordenar a paleta remapeia e o RGBA final fica byte-igual.

⭐ **Os vetores de teste do MVP viram a suíte de paridade** do `ph2d-pixel-core`. Hoje vivem
**dentro** dos `app/tests/*.test.js` (45 ficheiros; `app/tests/golden/` está vazio — medido 25/10):
o 1.º passo de cada onda é extraí-los para JSON lido pelos dois lados. É o oráculo mais barato.

## §3 — Oráculos (CLAUDE §0.9)

| Oráculo | Licença | Serve para | Limite |
|---|---|---|---|
| **Aseprite Trial 1.3.18.6** | EULA | **observar** comportamento, menus, diálogos, atalhos (Xvfb + fotos) | ⛔ não salva, não exporta, sem `--script` (medido) |
| **Aseprite licenciado** (US$ 19,99) | EULA | `-b` CLI: folha+JSON, GIF, conversão de modo com dither, `--scale`; **`.ase` escritos pelo próprio Aseprite como fixturas** | ⚠️ **decisão do dono** (compra) |
| Vetores de teste do MVP | nosso | kernels, paleta, dither, transform, tiles | só o que o MVP tem |
| LibreSprite 1.3 (instalado) | GPL | correr por CLI, comparar saída do comportamento herdado de 2016 | ⛔ só correr, nunca ler |
| Pixelorama 1.2.3 (instalado) | MIT | tilemaps iso/hex, keyframes de efeito, export | — |
| Pillow / ffmpeg / FreeType | livres | GIF/APNG/PNG/fontes (já usados no MVP) | — |

⚠️ **Sem a licença, a escrita de `.ase` (P9) só se verifica pela spec e pela leitura de volta no
Trial** (abrir o nosso ficheiro no Trial e fotografar) — mais fraco que comparar byte a byte com
um `.ase` escrito pelo Aseprite.

## §4 — O que se reusa da casa (sem Painter)

- `crates/ph2d-aseprite` — leitor clean-room da spec; hoje devolve **quadros RGBA compostos + tags**
  (é para o Sprite Inspector). O módulo precisa da **estrutura** (camadas, cels, paleta indexada,
  user data, slices, tilemaps): estender a crate-folha com uma API estruturada, sem quebrar a atual.
- `ph2d-imageio-*` (PNG, GIF, APNG, WebP, QOI…), `ph2d-color`, `ph2d-pen`, `ph2d-i18n`, tokens.
- `ph2d-sprite-sheet`/`ph2d-sprite-screen` e o **Sprite Inspector** são o CONSUMIDOR natural do que
  o módulo produz (P14), não peças do editor.

## §5 — Ondas

Cada onda: **referência Aseprite** (secção do estudo) · **entrega** · **gates** · **smoke**
(`PH2D_PIXEL_SMOKE=n`, cena fotografada antes de mandar ao dono). As P0–P6 formam o marco
**«dá para fazer no PH2D o sprite animado que se faz no Aseprite»**.

### P0 — Fundação (estudo §1 File/New, §2 Pencil/Eraser/Eyedropper/Zoom/Hand)
ADR do módulo · as 5 crates vazias com gates (`[lints] workspace`, núcleo sem deps) · `Sprite` v1
(RGBA **e** Indexed desde o 1.º dia — o MVP era só índice e recusava `.ase` RGBA) · objeto **Pixel
Sprite** no Add com diálogo New (largura, altura, modo de cor, fundo) · canvas: zoom inteiro, pan
(Space/H), xadrez, pixel grid · Pencil 1px, Eraser, Eyedropper (Alt) · cor de frente/fundo, `X` ·
undo/redo · gravar e reabrir no `.ph2dproj`. **Gates:** roundtrip byte-igual do blob; um traço =
um passo de undo; Bresenham AB==BA (golden do MVP). **Smoke 1:** criar 32×32, desenhar, Ctrl+Z,
salvar, reabrir.

### P1 — Desenho (estudo §2)
Tamanho e forma do pincel (redondo/quadrado/linha + ângulo), **pixel-perfect**, Line, Curve,
Rectangle/Filled/**rounded**, Ellipse/Filled, Contour, Polygon, Spray, Paint Bucket (tolerância,
contíguo, 4/8, camada ativa/visíveis), todos os modificadores Shift/Ctrl/Alt/Space, **Simetria**
H/V/diagonal com eixo a meio pixel, **Tiled Mode** (x, y, ambos). **Gates:** goldens de stamps
1..10, elipse par/ímpar espelhada, fill sem recursão em 4096². **Smoke 2.**

### P2 — Cor, paleta e inks (estudo §4, §2 inks)
Modo Indexed completo (índice transparente), **Edit Palette (F4)**, Palette Size, ordenar, selecionar
por uso, seletores Tint/Shade/Tone, espectro, rodas RGB/RYB e **roda de normal map**, presets,
Load/Save (`.gpl .pal .hex .act`, `.ase`), New Palette from Sprite, **Replace Color**, **Color Mode**
RGB↔Gray↔Indexed com dither (none/Bayer 2/4/8/matriz). **Inks:** Simple, Alpha Compositing, Copy
Alpha+Color, Lock Alpha, **Shading** (rampa = cores selecionadas). **Gates:** reordenar paleta ⇒
RGBA byte-igual; conversão com dither contra o oráculo (licença) ou o golden do MVP. **Smoke 3.**

### P3 — Camadas (estudo §3)
Background, normal, **grupo** (colapsar, blend e opacidade de grupo), **referência** (e *from file*),
visível/lock/**solo**, opacidade, **19 blend modes** no shader, renomear, duplicar, Merge Down,
Flatten, Flatten Visible, Layer via Copy/Cut, Paste as New Layer/Reference. **Gates:** cada blend
contra uma tabela-fixtura (Pillow/oráculo), grupo pass-through vs isolado. **Smoke 4.**

### P4 — Frames, cels e timeline (estudo §1 Frame, §3)
Painel **timeline camadas × frames** (Tab mostra/esconde), New Frame / Empty / Duplicate Cels /
**Linked Cels**, Delete, **duração por frame**, Frame e Cel Properties (opacidade, z-index),
camadas *continuous*, navegação (Home/End/←/→/Alt+G), **onion skin** (anteriores/seguintes,
vermelho/azul), Playback (velocidades, Play Once, Rewind on Stop), Preview flutuante, Constant
Frame Rate, Reverse Frames. **Gates:** cel linkado editado muda todos; playback determinístico
(relógio do host passado ao core). **Smoke 5.**

### P5 — Tags e playback avançado (estudo §3 Tags)
Tags com nome/cor/intervalo, direção **Forward/Reverse/Ping-pong/Ping-pong Reverse**, **repeat**,
subtags e grupos de tags, Play Subtags & Repetitions, Jump to First/Last in Tag, Set Loop Section.
**Gates:** a sequência de frames tocada por tag = tabela-fixtura (do oráculo com licença:
`--play-subtags` + `--data`). **Smoke 6.**

### P6 — Seleção e transformação (estudo §1 Edit/Select, §2 grupo 1)
Marquee retangular/elíptico, Lasso, Polygonal Lasso, **Magic Wand**; modos replace/add/sub/
intersect; All/Deselect/**Reselect**/Inverse; **Color Range**; Modify Border/Expand/Contract;
Load/Save MSK · Move (Auto Select Layer) · **Transform (Ctrl+T)** com alças, pivô, skew, campos
numéricos, **RotSprite** e Fast · Flip H/V, Rotate 90/180, **Shift** (com wrap) · Copy Merged ·
Paste Special · Crop/Trim · **Canvas Size** e **Sprite Size** (interpolações) · Rotate Canvas.
**Gates:** RotSprite@90 == rot90 byte a byte; flip 2× = identidade; pintar fora da seleção = no-op.
**Smoke 7.** ⇒ **Marco P0–P6.**

### P7 — Pincéis e dinâmica (estudo §2)
**New Brush (Ctrl+B)** da seleção, alinhamento do padrão (fonte/destino/pincel), flip/rotate do
pincel, biblioteca de pincéis, **dinâmica** pressão/velocidade → tamanho/ângulo/gradiente com
limiares, **stabilizer** — sobre `ph2d-pen`. **Smoke 8** (com a Wacom).

### P8 — Gradiente, texto, ajustes e FX (estudo §1 Edit)
Gradient linear/radial com dither (MVP W5) · Blur e Jumble · **Text** (fontes do sistema + BDF/PSF
do MVP W27, fill/stroke) · Fill (F) / Stroke (S) da seleção · Brightness/Contrast, Hue/Saturation,
Color Curve, Invert · **Outline (Shift+O)**, Convolution Matrix, Despeckle. **Smoke 9.**

### P9 — Export / Import (estudo §5)
Export As (escala, direção, pixel ratio, frames/camada) · **Export Sprite Sheet** com as abas
Layout/Sprite/Borders/Output e **todas** as opções do CLI · JSON compatível (`json-hash`/`json-array`,
`meta.frameTags/layers/slices`) · Import Sprite Sheet · GIF, APNG, PNG/sequência, WebP · **`.ase`
ler e escrever** · Repeat Last Export · (depois) modo `--batch` do binário PH2D. **Gates:** o nosso
JSON lido pelos importadores (Godot/Unity/Phaser — fixturas do MVP W13) e pelo nosso Sprite
Inspector; `.ase` escrito por nós aberto no Aseprite. **Smoke 10.**

### P10 — Tilemaps (estudo §3)
Camada tilemap, tilesets (tile 0 vazio, flips X/Y/D), Draw Pixels **Manual/Auto/Stack**, **Draw
Tiles**, Space+Tab, Export Tileset · além do Aseprite: **autotile 47** (MVP W8, que resolve um mapa
real — o MVP deixou isto PARCIAL) e grelhas **iso/hex** (como o Pixelorama). **Smoke 11.**

### P11 — Slices (estudo §3)
Slice tool, **9-slice** e **pivot** arrastáveis (pedido da comunidade), keys por frame, export em
`meta.slices`/`--split-slices`. **Smoke 12.**

### P12 — Ergonomia (estudo §6)
Atalhos por omissão **iguais aos do Aseprite** (e editáveis), Run Command, Undo History, autosave
e recuperação, user data/propriedades, preferências do módulo (grade, onion, xadrez). **Smoke 13.**

### P13 — O que só a PH2D tem
Do MVP: **rampas OKLCH**, pincel de sombreamento na rampa, **luz na rampa**, pincel de dither,
**color cycling**, **linter**, perspectiva 1/2/3 pontos e isometria guiada, quantização que preserva
rampas, vista CRT. Dos pedidos que o Aseprite não atende: **áudio** na timeline (módulo Áudio),
**auto-outline como efeito de camada**, **tween** de posição/rotação de cel (sobre o Motion/Timeline
da casa), smudge. Cada item com smoke próprio.

### P14 — O sprite vive na cena
Pixel Sprite como objeto do mundo: **tags viram clips** de Animação (Objectos e Componentes),
slices viram colliders/pivots (Física), **normal map** da roda de normais alimenta a Luz 2D,
o Sprite Inspector abre o que o módulo grava. (Diálogo com o Painter fica para quando ele fechar.)

## §6 — Ordem e paralelismo

P0 é série (todos dependem do documento). Depois: **P1 ∥ P2 ∥ P3** (ferramentas, paleta, camadas
tocam crates/ficheiros distintos) → P4 → **P5 ∥ P6** → o resto em paralelo conforme as linhas.
Cada onda fecha com handoff; smoke é do dono.

## §7 — Perguntas ao dono

1. **Comprar a licença do Aseprite (US$ 19,99)?** Recomendo **sim**: é o único jeito de ter o
   Aseprite como oráculo que se corre (folha, GIF, conversão de cor, `.ase` de referência). Sem ela,
   o Trial só serve para olhar.
2. O nome que aparece no menu Add: «Pixel Sprite»? (proposta)
