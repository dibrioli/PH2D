# O que a subida do stack abriu para a UI (2026-08-30)

> Pergunta 1 do Enio: *"Descubra o que a atualização Vello/WGPU/Rust nos trouxe de novo que pode
> nos ajudar a construir UI/UX melhor."*
>
> ⚠️ **Os saltos reais** (de `docs/Atualizar Stack/01_inventario.md`): `vello` **0.8 → 0.10**
> (duas versões) · `wgpu` **28 → 29** (uma) · `parley` **0.6 → 0.11** (**cinco**) ·
> `skrifa` 0.40 → 0.44 · `accesskit` → 0.24.1.
>
> ⭐ **O salto grande não é o Vello: é o `parley`.** O motor de texto andou cinco versões, e é o
> subsistema onde toda a UI vive.

---

## §1 — `parley` 0.6 → 0.11: o achado maior

### 1.1 — ⭐⭐ Existe um EDITOR DE TEXTO no stack que já pagámos, e ninguém o ligou

O `parley` traz **`PlainEditor`** — layout + cursor + seleção + edição, com
`parley::editing` e `parley::cursor` como módulos próprios (reorganizados na 0.7).

Medido, o que nós de facto importamos de `parley` em `crates/ph2d-text/`:

```
3 × parley::PositionedLayoutItem::GlyphRun
2 × parley::Layout
1 × parley::LayoutContext
1 × parley::FontContext
1 × parley::setting::Tag
1 × parley::AlignmentOptions::default
```

⛔ **Zero uso de `PlainEditor`, `parley::editing`, `parley::cursor` ou `LayoutAccessibility`.**
Nós usamos o `parley` como *shaper* e desenhamos o resto à mão.

⭐ **Isto importa porque o Enio nomeou o alvo:** *"até editor de texto de codificação"*. O
editor de código é a única das ferramentas dele que **não** precisa de motor novo — precisa de
ligar o que a subida já trouxe.

### 1.2 — Hit-testing exato de texto (0.7)

`Cluster::from_point_exact` — dado um ponto, qual cluster. É a primitiva de **selecionar texto
com a caneta**, que é precisamente o gesto que um app iPad/Wacom tem de acertar. Não usamos.

### 1.3 — ⭐ `x-height` e `cap-height` em `RunMetrics` (0.8)

O centramento vertical óptico de um rótulo faz-se pela **cap-height**, não pela caixa da linha —
é por isso que um rótulo centrado pela caixa parece sempre um pouco alto. A métrica passou a
existir na 0.8. Não a usamos: o nosso centramento é
`y + (ROW_H - LABEL_FONT_SIZE) * 0.5` (p.ex. `crates/ph2d-editor-core/src/grid_snap/inspect.rs:297`),
que é a aproximação pela caixa.

### 1.4 — ⭐⭐ Caixas fora-de-fluxo e posicionamento por linha (0.9)

`InlineBoxKind { InFlow, OutOfFlow, CustomOutOfFlow }` + `inline_min_coord`/`inline_max_coord`
por linha ⇒ **texto que flui à volta de regiões excluídas.**

É o substrato de: rótulo que contorna um ícone, texto de ajuda que contorna um widget, e
qualquer painel onde o texto tenha de coexistir com um controlo na mesma linha. Hoje resolvemos
isso posicionando à mão.

### 1.5 — Controlo de quebra de linha grau-CSS (0.7, 0.8, 0.10, 0.11)

`TextWrapMode` (0.7) · `text-indent` e `break_line_with_next` (0.8) · **`complex-scripts`** para
CJK/Tailandês/Khmer/Lao/Myanmar (0.10) · `set_line_break_override` +
`From<WordBreak/OverflowWrap/TextWrapMode>` para `StyleProperty` (0.11).

⚠️ O `complex-scripts` é **feature opt-in**. Se a UI vai ser traduzida (HR-15, i18n), a quebra
de linha em CJK depende de a ligarmos — e ligá-la depois de a UI estar desenhada muda larguras.

### 1.6 — ⭐ Todas as propriedades de texto do AccessKit (0.8)

`LayoutAccessibility` + AccessKit 0.24. Temos HR-12 (a11y) e um `WidgetStore::register` que já
emite `Node` — mas **o texto** ainda não expõe as suas propriedades. É o caminho barato para
subir a11y sem escrever árvore de acessibilidade à mão.

### 1.7 — ⚠️ Duas mudanças que já nos morderam e não devem ser re-descobertas

- **`Glyph::y` passou a Y-down** (0.8). O `STACK_VERSOES.md` regista que o sinal do `y_offset`
  inverteu e que **foi a cura de um defeito nosso**.
- Larguras de avanço acima de `wght 400` **encolheram até −0,50 px**. Todo layout calibrado à
  mão contra a largura antiga está 0,5 px fora.

---

## §2 — `vello` 0.8 → 0.10

### 2.1 — ⭐ Atlas de imagem persistente (0.9)

*"Image atlas residency is now preserved across renders, avoiding repeated atlas rebuilds and
uploads."*

⇒ **ícones e miniaturas param de ser re-carregados por quadro.** Para uma UI com 53 widgets
primitivos e centenas de ícones, é a diferença entre pagar o atlas todo quadro e pagar uma vez.

⚠️ **Com um preço, e nós já o pagámos:** quem recozinha pixels tem de chamar
`mark_texture_dirty` / `mark_override_image_dirty`, senão a imagem **congela**. Conferido: temos
**um** consumidor (`shells/desktop/src/fx_live.rs:393`), com o wrapper em
`crates/ph2d-render/src/vello_pass.rs:130` e **dois gates** a defendê-lo
(`shells/desktop/tests/every_recook_tells_vello_the_pixels_changed.rs`, incluindo um que impede
satisfazer o censo com um `mark_texture_dirty` que não faz nada).

⚠️ **Mas o censo é do `fx_live` apenas.** Qualquer superfície de UI nova que recozinhe pixels
(miniatura de camada, pré-visualização de pincel, thumbnail de asset) herda a armadilha e
**não** está coberta por aquele gate.

### 2.2 — ⭐ Emboldening sintético (0.9)

`GlyphRun::font_embolden` + `DrawGlyphs::font_embolden`. **Não usamos** (grep: zero ocorrências).

⚠️ Relevante porque o nosso modo `Crisp` engorda o texto pedindo **peso maior à fonte variável**
(`docs/UI_Plans/2026-05-24-crisp-text-rendering.md`, "Boost de `FontWeight` por faixa de
tamanho"). O `font_embolden` é uma **segunda alavanca**, de natureza diferente (engorda o
contorno em vez de trocar o cut). Não é «melhor» — é outra, e a existência dela merece medição
antes de a próxima spec de tipografia escolher.

⛔ **Não conclua que substitui o nosso weight boost.** Um cut «Text» do Inter é desenhado; um
embolden é dilatado. São coisas diferentes e a comparação ainda não foi feita.

### 2.3 — `brush_transform` no `GlyphRun` (0.9)

Transformar o brush independentemente do glifo ⇒ **texto com gradiente/textura** sem hack. Não
usamos.

### 2.4 — `ImageQuality::High` passou a bicúbico Mitchell de verdade (0.9)

Na 0.8 era bilinear disfarçado. Nós **usamos** o eixo (`crates/ph2d-editor-core/src/project.rs:129`,
`PixelArt → Low` / `Smooth → High`), então **a aparência do modo Smooth mudou** com a subida.
⚠️ O `STACK_VERSOES.md` avisa que *"a paridade pré-visualização ↔ sprite assada mudou de
significado"*.

### 2.5 — ⭐ Correcção do desfoque por meio-pixel (0.9)

*"Blurry image rendering due to incorrect half-pixel offset."*

⇒ **Os nossos ícones podiam estar desfocados por um bug do Vello, agora corrigido.**

⚠️ **Isto NÃO é o nosso `SnapX`.** Conferido: `SnapX {None, Half, Full}`
(`crates/ph2d-tokens/src/typography.rs:179`) é **snap de glifo**, aplicado em
`crates/ph2d-editor-core/src/paint.rs:119`. O bug do Vello era de **imagem**. São subsistemas
diferentes e o `SnapX` continua a fazer sentido.
⏳ **Mas fica a pergunta por medir:** se alguém compensou desfoque de ÍCONE em algum sítio, essa
compensação é agora **dupla**. Não foi varrido.

### 2.6 — Interpolação de gradiente em alfa não-pré-multiplicado (0.10)

Corrige o escurecimento clássico de gradiente para transparente. Relevante para qualquer
scrim/fade/vinheta da UI.

### 2.7 — Binning acima de 256 bins (0.10)

*"Rendering scenes whose binning requires more than 256 bins."* — ⚠️ **uma UI cheia É uma cena
grande.** Com 25 painéis, 2 073 ids e o canvas por baixo, este limite era alcançável. Era um
defeito de renderização em cena densa e desapareceu.

### 2.8 — Texturas destruídas ao libertar (0.10) e leitura de memória partilhada no `clip_leaf` (0.9)

Memória de GPU devolvida na hora, e correcção de leitura inválida em lanes inactivos do shader
de **clip**. ⚠️ O clip é o que um dock usa para recortar conteúdo — a correcção é directamente
do nosso lado do problema.

---

## §3 — `wgpu` 28 → 29

### 3.1 — ⭐⭐ Espaço de cor explícito na superfície (HDR)

`SurfaceConfiguration::color_space` + `SurfaceCapabilities::format_capabilities`.

⇒ **é agora possível declarar o espaço de cor da janela.** Para um app de pintura num monitor
de gama larga isto é substancial: hoje entregamos sRGB implícito.

Conferido: **não usamos** (`grep color_space` em `crates/`+`shells/` = zero).

⛔ **Não é trabalho desta etapa** e é maior do que parece (toca gestão de cor de ponta a ponta,
e o `ColorProfile` vive em `ph2d-imageio` sob gate de contrato). Fica **nomeado**, não proposto.

### 3.2 — Apresentação de superfície mudou de forma

`Surface::get_current_texture()` devolve `CurrentSurfaceTexture` (enum) em vez de
`Result<…, SurfaceError>`, com variante `Suboptimal` dedicada. Já absorvido pela subida; conta
aqui porque **é onde se trata o redimensionamento de janela**, e um dock que arrasta redimensiona
muito.

### 3.3 — Coisas que existem e não são nossas

Mesh shaders, TLAS binding arrays, `SHADER_I16`, coordenadas baricêntricas, operações
cooperativas. ⛔ Nada disto toca UI 2D — está aqui só para que a próxima janela não gaste tempo
a reavaliá-las.

### 3.4 — Do 28 (que atravessámos): texturas transitórias e `LoadOp::DontCare`

`TEXTURE_USAGE_TRANSIENT` (Vulkan/Metal) e `LoadOp::DontCare`. São alavancas de memória para
passes offscreen — relevantes se a UI nova usar render targets por dock.

---

## §4 — Rust 1.98 / edition 2024

Nada de específico de UI. O que muda é MSRV e ferramenta. ⚠️ O `vello` 0.10, `parley` 0.11 e
`fontique` 0.11 pedem **1.88**; o `wgpu` 29 pede 1.87 — estamos folgados em 1.98.

---

## §5 — O placar: o que a subida abriu, e o que já consumimos

| capacidade nova | subsistema | consumimos? |
|---|---|:--:|
| `PlainEditor` / `parley::editing` / `parley::cursor` | texto | ⛔ **não** |
| `Cluster::from_point_exact` (hit-test exato) | texto | ⛔ **não** |
| `cap-height` / `x-height` em `RunMetrics` | texto | ⛔ **não** |
| Caixas fora-de-fluxo + posicionamento por linha | texto | ⛔ **não** |
| `complex-scripts` (CJK/Thai/…) | texto | ⛔ **não** (feature opt-in) |
| Propriedades de texto do AccessKit | a11y | ⛔ **não** |
| `TextWrapMode` / `WordBreak` / `OverflowWrap` | texto | ⛔ **não** |
| Atlas de imagem persistente | render | ⭐ **sim**, com 2 gates — mas o censo cobre 1 consumidor |
| `font_embolden` | texto | ⛔ **não** |
| `brush_transform` no GlyphRun | texto | ⛔ **não** |
| `ImageQuality::High` bicúbico | render | ⭐ **sim** (e a aparência mudou) |
| Correcção do meio-pixel de imagem | render | ⭐ **de graça** |
| Gradiente em alfa não-pré-multiplicado | render | ⭐ **de graça** |
| Binning > 256 bins | render | ⭐ **de graça** (e importa numa UI densa) |
| `SurfaceConfiguration::color_space` (HDR) | wgpu | ⛔ **não** |
| Texturas transitórias / `LoadOp::DontCare` | wgpu | ⛔ **não** |

⭐⭐ **A conclusão que interessa ao desenho:** a subida foi paga e **quase tudo o que ela abriu
está no motor de TEXTO, por consumir.** Uma UI de editor é, em peso, texto — rótulos, listas,
árvores, campos numéricos, e o editor de código que o Enio nomeou. A alavanca não está em
desenhar widgets novos; está em ligar o `parley` que já temos.
