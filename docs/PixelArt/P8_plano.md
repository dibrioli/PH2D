# P8 — Gradiente, texto, ajustes e FX: o plano (02/11)

> Linha `line/PixelArt`, base `3df5894489` (P7 fechada, smoke da cena 18 aprovado 02/11, por integrar).
> Plano do módulo: [02_plano_modulo.md §P8](02_plano_modulo.md). Molde: [P7_plano.md](P7_plano.md).
> Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> (autocontido, sem o Painter). Mapa dos menus do Aseprite: [01_estudo §1–§2](01_estudo_aseprite.md),
> foto [13](aseprite/13_sub_shift_adjust_fx_colormode_canvas.png).

## 0. O que entra (o menu Edit e as ferramentas do Aseprite 1.3.18, foto 13)

| Grupo | Peças | Onde no Aseprite |
|---|---|---|
| **G — Gradient** | linear / radial, fg→bg, matriz de dither | sub-ferramenta do Paint Bucket (G) |
| **B — Blur · Jumble** | duas ferramentas de pincel | grupo 11 da barra |
| **T — Text** | ferramenta T + *Edit › Insert Text*; fontes do sistema + fontes de pixel (BDF/HEX/FNT/FON do MVP W27); fill/stroke, B/I, antialias | grupo 12 |
| **S — Fill (F) · Stroke (S)** | da selecção, cor de frente | Edit |
| **A — Ajustes** | Invert · Brightness/Contrast · Hue/Saturation (Ctrl+U) · Color Curve (Ctrl+M) | Edit, Edit › Adjustments |
| **X — FX** | Outline (Shift+O) · Convolution Matrix (F9) · Despeckle (Median Filter) | Edit › FX |

Já existe e **não se reconstrói**: `Replace Color` (`PixelRequest::ReplaceColor`, `ph2d-editor-core/src/pixel_panel/mod.rs:460`
— só confere-se o menu Shift+R), o «Gradient» da PALETA (`paleta_remap.rs:82`, outra coisa), `selection::invert` (selecção,
não cor), `dither.rs` (Bayer 2/4/8 presa ao canvas, P7).

## 1. Estado da arte

| App | O que faz | Porquê / o que foi abandonado |
|---|---|---|
| **Aseprite 1.3.18** (referência do dono) | Gradient arrastado de A a B, fg→bg, *Solid* (mistura RGB) ou matriz (Bayer 2/4/8 — a mesma lista da dinâmica). Os filtros abrem um diálogo com **Preview**, **Target** (canais R/G/B/A ou Index; *all frames/layers*; a selecção se houver) e OK/Cancel; um passo do desfazer. Outline: dentro/fora, cor, matriz (círculo/quadrado/horizontal/vertical), fundo. Text (1.3.10+): ferramenta própria, o texto fica editável até confirmar e depois vira pixels. | O bug 1.3.0–1.3.1 da matriz do Gradient (estado partilhado com a dinâmica) ⇒ estado próprio por consumidor, MESMA função (P7_plano §2.7). O Trial **não salva nem exporta** (01_estudo §0): oráculo de COMPORTAMENTO, lido pelo ecrã. |
| **MVP W5/W7/W27** (`docs/Pixel Art/`, nosso) | Gradiente = caminhada pela RAMPA com dither entre degraus (spec §13.3); `outline` sobre índices, externo/interno, 4/8-conn (§14.2, `effects.test.js`); leitores BDF/HEX/FNT/FON + oráculo FreeType por ctypes (`tests/font_oracle.py`); fontes Cozette (MIT), Tom Thumb, Unscii-8/16. | O golden 32×32 do gradiente nunca foi gravado (`app/tests/golden/` vazio) ⇒ a fixtura nasce desta onda. Blur/Jumble/ajustes/convolution/despeckle: o MVP não tem lei. |
| **Pixelorama 1.2.3** (MIT, instalado) | Gradient (linear/radial, dither), Outline, Drop Shadow, HSV, Brightness/Contrast, Invert, Desaturate, Posterize, texto. | Porta permissiva; serve de 2.º oráculo para o que o Trial não deixa medir bem (ex.: Outline em bordas). |
| **LibreSprite** (GPL-2, `/usr/bin/libresprite`) | O tronco antigo do Aseprite: Convolution Matrix e Despeckle com o mesmo diálogo. | Só CORRIDO, nunca lido (`oraculo_libresprite.py`). Se o Trial ler mal um kernel, confere-se aqui. |
| **FreeType** (corrido por ctypes) | Rasterização mono (`FT_LOAD_TARGET_MONO`) e AA. | O Aseprite rasteriza com FreeType ⇒ a máscara mono do FreeType é o oráculo bit a bit do texto sem antialias. |

## 2. O desenho — a porta ÚNICA de cada pergunta

1. **«Que cor tem a célula do gradiente?»** → `ph2d_pixel_core::gradient::at(shape: Linear|Radial, a, b, cell, ends: (fg, bg), fill: Solid|Matrix(Dither)) -> Rgba` (ou índice no Indexed). `s` = projecção (linear) ou distância (radial), clamp [0,1] — a fórmula da spec §13.3 do MVP; com matriz chama **`Dither::reaches`** da P7 (a MESMA função) com o estado `GradientTool.dither`, separado de `Dynamics.gradient`. *Solid*: mistura medida em R-G (RGB? linear? arredondamento?).
2. **«Onde age um filtro?»** → `ph2d_pixel_core::filter::Target { channels, scope: Cel|AllLayers|AllFrames, mask: Option<&Selection> }` e UMA função `apply(sprite, target, &dyn Filter) -> Op`. Todos os ajustes e FX são `Filter` (`fn row(&self, src, dst)`), um passo do desfazer, e o **preview NÃO toca no documento** (lei da casa, `ph2d-preview-drive`): o preview é uma cópia do cel desenhada por cima.
3. **«Que cor sai de cada ajuste?»** → uma LUT por canal (Invert, Brightness/Contrast, Color Curve) ou uma função por píxel (Hue/Saturation), em `filter/adjust.rs`, sRGB 8-bit (o Aseprite trabalha em 8-bit; R-A decide). No **Indexed** o ajuste muda o ÍNDICE para a cor mais próxima da paleta ou muda a PALETA? — medido em R-A, nunca escolhido. Fórmulas publicadas primeiro (contraste em torno de 128, HSL); constante que não bate com R-A = PARE e meça.
4. **«Convolução»** → `filter/convolution.rs`: kernel N×M, divisor, bias, borda. Os kernels predefinidos do Aseprite vêm de um ficheiro de dados dele (não se copia): **mede-se a resposta ao IMPULSO** de cada um no Trial (R-C) — a matriz sai da imagem — e a lista entra como dados nossos com o nome medido.
5. **«Despeckle»** → `filter/median.rs` (mediana W×H por canal, borda medida em R-D).
6. **«Outline»** → `filter/outline.rs`: dentro/fora × matriz (4 formas) × cor × fundo; a lei do MVP §14.2 (lê o original, escreve uma cópia) é o caso «fora, círculo 3×3».
7. **«Blur / Jumble»** → ferramentas de pincel sobre o MESMO funil de carimbo da P7 (`Brush::stamp`, simetria, pixel-perfect, dinâmica). Blur = média do vizinho 3×3 por carimbo (R-B: peso, alfa, opacidade); Jumble = troca com um vizinho ALEATÓRIO — a aleatoriedade é uma semente por traço (`hash3`, determinístico, HR-5), a lei medida é estatística (multiconjunto das cores preservado? raio?).
8. **«Que píxeis acende um glifo?»** → crate NOVA `ph2d-pixel-text` (drop-crate; o `ph2d-pixel-core` fica sem deps): fontes do sistema por `ph2d-system-fonts` + contornos por `skrifa` (já no `Cargo.lock`, 0.44.0, via `ph2d-vector-font`), rasterizador próprio por cobertura com o limiar do FreeType mono (gate contra o oráculo FreeType); fontes de pixel = porte dos leitores BDF/HEX/FNT/FON do MVP W27 com o corpus dele (Cozette MIT, Tom Thumb, Unscii-8/16) **sem uma linha mudada** + licenças ao lado. Fill/stroke do texto = máscara do glifo → o MESMO `Stroke` da selecção (item 9).
9. **«Fill / Stroke da selecção»** → `ph2d_pixel_core::selection::{fill, stroke}` com a cor de frente; o contorno é a borda DENTRO ou FORA da máscara e com que largura (R-S).
10. **Estado do texto** antes de confirmar: vive no app (`ph2d-app-pixel`), não no documento; Enter carimba UM passo do desfazer. **Não toca no `PROJECT_SCHEMA`** — salvo se o dono quiser texto editável depois de confirmado (pergunta §9).

## 3. Contratos congelados (§6) e schema — não encosta

- `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`, `NodeOp`, Vector: os três gates não nomeiam crates do Pixel (P7_plano §3; re-conferir no fecho). As ferramentas novas são variantes do `PixelTool` (`ph2d-editor-core/src/pixel_panel/tools.rs:9`), fora do contrato.
- `PROJECT_SCHEMA`: intocado (texto confirmado = píxeis; estado dos diálogos = sessão). Muda só se o dono pedir texto re-editável.
- Crate nova `ph2d-pixel-text`: `Cargo.lock` +1 nome; `skrifa` na MESMA versão do `ph2d-vector-font` (STACK_VERSOES: duas cópias de `read-fonts` não se falam).
- Catraca `the_shell_only_shrinks` do `ph2d-app-pixel`: **12 562 de 13 424** (sobram 862). A lei mora no core e na crate de texto; o app só liga. Se passar, a cura é MOVER (como a P7b fez para `shells/pixel-scenes`), nunca subir.

## 4. A UI — as 4 condições, por peça

| Peça | Existe? | Pintada + registada | Clique → barramento | Leva a algum lugar |
|---|---|---|---|---|
| Gradient no flyout do Paint Bucket + opções (Linear/Radial, Solid/matriz) | NÃO | flyout das selecções (P6b) | `seam.rs` | arrasto → `gradient::at` |
| Blur / Jumble na barra | NÃO | botão de grupo com flyout | `ids.rs` + `seam.rs` | carimbo → `Filter` local |
| Text: ferramenta T, caixa no canvas, fonte/tamanho/B/I/AA/fill/stroke | NÃO (o seletor de fontes do Vector existe) | reusa o seletor de fontes da casa | `seam.rs` | Enter → `ph2d-pixel-text` → um passo |
| Edit › Fill, Stroke, Invert, Adjustments ›, FX ›, Insert Text | NÃO (`EDIT_ACTIONS` tem 18) | menus da P6 (`ids_selecao.rs`) | `PixelRequest` novos | `filter::apply` |
| Diálogo do filtro (Preview, Target, OK/Cancel + os controlos de cada filtro) | NÃO | conforme a pergunta §9.2 | `seam.rs` | preview = cópia; OK = `Op` |
| Curva (Color Curve) | NÃO há editor de curva no `ph2d-editor-core` fora do Painter | widget NOVO `curve_editor` **com o gate dele** (DIRETIVA §2) | idem | LUT |
| Atalhos F, S, Shift+O, Ctrl+U, Ctrl+M, F9, T, Shift+G | livres no Pixel (conferir `input_teclas.rs`) | — | `input_teclas.rs` | os acima |

Prova de cada linha: um teste `ph2d-ui-testkit` que DIRIGE o clique/tecla real e afirma os píxeis no canvas (DoD §5).
Strings: UI só em inglês (memória `feedback_app_ui_english_only`), i18n em `ph2d-i18n/src/pixel_efeitos.rs`.

## 5. Medição — UMA rodada intercalada no Trial (§0.10), antes de escrever lei

Um só script, `docs/PixelArt/ferramentas/trial_efeitos.py`, no método das rodadas anteriores
(`~/.local/opt/aseprite-trial/Aseprite-trial.AppImage`, `SDL_VIDEODRIVER=x11`, HOME isolado, Xvfb próprio, `xdotool`,
`import -window root`, leitura do centro de cada célula). Sprites NOSSOS, desenhados no próprio Trial pelo script
(ele não abre ficheiros que não salvou? — conferir no 1.º minuto; senão `File › Open` de PNG nosso). Fixturas com
cabeçalho `source`/`method`/`command`, em `crates/ph2d-pixel-core/tests/fixtures/oracle_trial_*.json`.

- **R-G** Gradient: 32×8 e 32×32, linear H/V/diagonal e radial, *Solid* + Bayer 2/4/8, fg/bg conhecidos, RGB **e** Indexed; arrasto curto/longo/fora do canvas → cor de cada célula (a mistura do *Solid*, o arredondamento, a âncora da matriz).
- **R-A** Ajustes sobre uma rampa de 256 tons por canal (4 sprites 256×1: R, G, B, cinza) + 1 sprite de matizes: Invert; B/C em (−100, −50, 0, 50, 100)²; Hue/Sat (±180, ±100, Lightness, modo HSL/HSV se houver); Curve com 3 curvas → **a LUT inteira** por canal. Mais um sprite Indexed: o ajuste mexe nos índices ou na paleta.
- **R-C** Convolution: impulso (1 píxel branco no preto) e borda, cada kernel da lista → a matriz, divisor e bias; o tratamento da borda do cel.
- **R-D** Despeckle: ruído sal-e-pimenta semeado, W×H 3×3 e 5×5 → resultado por célula.
- **R-O** Outline: forma assimétrica de 6 células × dentro/fora × 4 matrizes × fundo transparente/cor.
- **R-S** Fill/Stroke da selecção rectangular e elíptica: que células o Stroke pinta.
- **R-B** Blur: carimbo único sobre um xadrez preto/branco (pesos); Jumble: 3 traços iguais → as cores se conservam? que raio?
- **R-T** Text: «Ag1» em 3 fontes do sistema (DejaVu Sans, uma serifada, uma mono) a 8/12/16 px, AA ligado/desligado, + uma BDF → máscara por célula. Os mesmos casos no oráculo FreeType (ctypes) ⇒ se baterem, o FreeType vira o gate (corre sem ecrã).
- Alvos de cada filtro numa só passagem: com selecção, sem selecção, *all frames*.

Kill-criterion: se o Trial não deixar abrir/criar o sprite de entrada em 2 tentativas, a peça mede-se no LibreSprite
(mesmo diálogo, mesmo tronco) e fica escrito qual oráculo respondeu. Leitura de ecrã que não reproduz em 3 corridas
iguais = recusa MEDIDA escrita, nunca lei inventada.

## 6. Gates, red-first, e as fixturas

- `ph2d-pixel-core/tests/it/gradient.rs` contra `oracle_trial_gradient.json` (R-G) + o golden 32×32 que o MVP prometeu (gravado agora); a mesma `Dither::reaches` com dois estados (o bug 1.3.0 como teste: mudar a matriz da dinâmica não muda a do Gradient).
- `filter_adjust.rs` (LUTs de R-A, byte a byte), `filter_convolution.rs` (R-C), `filter_median.rs` (R-D), `filter_outline.rs` (R-O + o teste do MVP `effects.test.js`), `selection_fill_stroke.rs` (R-S).
- `filter_target.rs`: fora da selecção = intocado; canal desligado = intocado; preview não muda o documento; um passo do desfazer; *all frames* = um passo.
- `ph2d-pixel-text`: leitores BDF/HEX/FNT/FON contra os vetores do MVP; máscara mono = FreeType bit a bit (R-T).
- Blur/Jumble: pesos de R-B; Jumble determinístico com a mesma semente e conserva o multiconjunto (se R-B o confirmar).
- Widget `curve_editor`: gate próprio (pontos não cruzam em X, pega presa ao quadro), como o `range_slider` da P7.
- Seam (`shells/pixel-scenes`): cada menu/tecla/ferramenta da §4 dirigida e o píxel afirmado.
- Mutações no fecho: projecção do gradiente (radial↔linear), estado de dither partilhado, LUT do contraste (pivô), máscara do `Target` ignorada, borda da convolução, outline dentro↔fora, limiar do glifo, preview a escrever no documento.

## 7. Smoke — cena 19 (`PH2D_PIXEL_SMOKE=19`; `max_level` 18 → 19 em `crates/ph2d-app-pixel/src/smoke.rs:32`)

Sprite 64×48 com um boneco simples, fundo branco, uma selecção feita. Passos: Gradient radial com Bayer 4 no fundo ·
Outline fora no boneco · Hue/Saturation com Preview, Cancel e OK · Ctrl+Z · Text «PH2D» numa fonte de pixel e numa do
sistema · Fill/Stroke da selecção · Blur num canto. Os números da mensagem ao dono saem da sonda headless
(`tests/it/cena19.rs`) corrida ANTES; a cena é fotografada (fábrica + layout do dono, `layout.txt` sha256 antes/depois).

## 8. Ordem de trabalho (um bloco)

1. `trial_efeitos.py` + a rodada R-G/R-A/R-C/R-D/R-O/R-S/R-B/R-T (e o oráculo FreeType) — UMA corrida, fixturas gravadas.
2. Núcleo: `gradient.rs`, `filter/{mod,adjust,convolution,median,outline}.rs`, `selection::{fill,stroke}`, Blur/Jumble; crate `ph2d-pixel-text` (leitores + rasterizador). Gates vermelhos → verdes.
3. App: `PixelTool` +Gradient/Blur/Jumble/Text, `PixelRequest` dos menus, preview como cópia, estado do texto, atalhos.
4. UI: flyouts, menus Edit/Adjustments/FX, diálogo dos filtros, widget `curve_editor` (+ gate), opções do Text; seam tests.
5. Cena 19, sonda, fotos, mutações, handoff.

## 9. Perguntas ao dono (antes de codar)

1. **Gradient**: só entre a cor de frente e a de fundo (como o Aseprite), ou também «pela paleta» (caminha pelos tons vizinhos da paleta, como o MVP fazia)? Recomendo: fundo/frente agora, «pela paleta» como opção na mesma ferramenta.
2. **Janelas dos filtros** (Brightness, Hue/Sat, Curve, Outline…): janelinha flutuante com Preview / OK / Cancel como no Aseprite, ou uma secção no painel da direita no padrão do Inspector? Recomendo a janelinha (o filtro é um gesto de uma vez, e o canvas fica à vista).
3. **Texto**: vira píxeis ao confirmar (Aseprite), ou fica um texto que se reabre e edita depois? Recomendo píxeis ao confirmar (editável até lá).
4. **Fontes de pixel de fábrica**: trazer as do MVP (Cozette, Tom Thumb, Unscii) junto com as fontes do computador? Recomendo sim.
