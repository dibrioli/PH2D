# Modelos de UI com CÓDIGO COMPLETO para seguir (2026-09-04)

> Enio: *«o que eu pedi do início foi um redesenho completo da UI de modo que se tornasse muito mais
> parecida com Blender/Godot. […] cada widget, até as cores, absolutamente tudo, para tornar a UI
> muito mais minimalista, plana, concisa, coerente e simples. Mas o que foi feito deixou o app com
> praticamente a mesma cara. […] precisamos de um modelo a seguir — com código completo disponível
> que possamos consultar, modificar e depois integrar ao app.»*
>
> Este documento é a pesquisa. Cada candidato traz **licença verificada** (`gh api … .license`),
> **o que exactamente tem de código**, e **como encaixa** na nossa stack (`tokens.json` →
> `ph2d-tokens` → pintores em Vello). A recomendação está no §4; as decisões que são dele no §5.

## §0 — ⛔ Por que o app ficou com a mesma cara, medido

O redesenho de 02–03/09 trocou **pintores** (a caixa única, a marca à direita, a coluna de
animação) e deixou a **PELE** intacta. A cara de um app não está nos 44 widgets — está em
**meia dúzia de números** que todos eles lêem, e nenhum desses números mudou:

| o que dá a cara | hoje ([`tokens.json`](../../design/tokens.json)) | Godot 4.6 / Graphite |
|---|---|---|
| raio dos painéis | **`panel-radius: 16`** | **4** / 0 |
| fundo | cinza **tingido** (`oklch(… 0.004 320)` — matiz magenta em todo bg) | cinza **neutro** |
| acento | **quatro** temas, cada um com acento saturado próprio (magenta · ciano · laranja · azul) | **um** acento, azul `#569eff` |
| molduras | `stroke_rounded_rect` em **271** sítios; cartões e caixas de texto sempre com borda | `border_size: 0` por omissão |
| sombras | 3 tokens (`sm`/`md`/`lg`) + `inset-hi` | nenhuma |
| paleta | **83** slots por tema, escritos à mão | **≤ 5 entradas**, o resto **derivado** |

⚠️ E a superfície que os lê é grande e consistente: **1 629** sítios `ColorToken::`, **294**
`Radius::`, 42 ficheiros de widget. ⇒ *trocar a pele é trocar a tabela, não os 42 pintores* — é o
mesmo achado que a [`spec/02 §1`](../spec/02_o_que_falta_para_comecar.md) já tinha feito para a
migração dos ids: **a UI nova nasce de uma tabela**.

## §1 — Os critérios (os dele, transcritos)

minimalista · plana · concisa · coerente · simples · **parecida com Blender/Godot** ·
**código completo** que se possa consultar, modificar e integrar · licença que permita **portar**.

## §2 — Os candidatos, medidos

| # | modelo | o que é | licença | o que traz de CÓDIGO | encaixe | veredito |
|---|---|---|---|---|---|---|
| **1** | **Godot 4.6 — tema «Modern»** | o editor do Godot, tal como ele é hoje: o [godot-minimal-theme](https://github.com/passivestar/godot-minimal-theme) (3,8 k★) **portado nativo** e tornado omissão em 4.6 | **MIT** | [`theme_modern.cpp`](../referencias/godot-editor-src/editor/themes/theme_modern.cpp) **2 960 LOC** — tema para **108** tipos de controlo, derivado de **8 knobs** | é *exactamente* o look pedido, e o código já está vendorizado | ⭐⭐⭐ **o modelo** |
| 2 | **Pixelorama** | ferramenta de **arte** (sprites, animação) feita **em** Godot | **MIT** | `assets/theme.tres` + [`Themes.gd`](https://github.com/Orama-Interactive/Pixelorama/blob/master/src/Autoload/Themes.gd): **9 temas de UM ficheiro**, cada um = `(base_color, accent_color, contrast = 0,3)` | prova que a receita do nº 1 chega para uma app de arte | ⭐⭐ a prova |
| 3 | Material Maker | editor de nós em Godot | MIT | `material_maker/theme/` | mesma família | ⭐ |
| 4 | **Graphite** | editor 2D vetorial + raster em **Rust**, por nós, *«o Blender do 2D»* — o irmão de produto mais próximo do PH2D | **Apache-2.0** | chrome em Svelte/CSS: [`Editor.svelte`](https://github.com/GraphiteEditor/Graphite/blob/master/frontend/src/components/Editor.svelte) com a paleta inteira; **27 widgets** (6 botões · 15 inputs · 5 rótulos) + 4 de layout + 6 menus flutuantes | web ⇒ **valores e comportamento**, não código reutilizável | ⭐⭐ a referência VISUAL no nosso domínio |
| 5 | **iced** | toolkit Rust | **MIT** | [`palette.rs`](https://docs.iced.rs/src/iced_core/theme/palette.rs.html): `Palette {background, text, primary, success, warning, danger}` → `Extended` (fundo × 7 desvios `0,03…0,20`; cada papel `base/weak/strong` com texto `readable`), **em OKLch** | ⭐ o nosso `tokens.json` já é OKLch: o algoritmo porta-se para o `ph2d-tokens` como está | ⭐⭐ o algoritmo de derivação |
| 6 | **egui** | toolkit Rust | **Apache-2.0 OR MIT** | [`style.rs`](https://docs.rs/egui/latest/src/egui/style.rs.html): `Widgets {noninteractive, inactive, hovered, active, open}` × `WidgetVisuals {bg_fill, weak_bg_fill, bg_stroke, corner_radius, fg_stroke, expansion}` | **5 estados × 6 campos = 30 números** dizem tudo o que um controlo interactivo pode ser | ⭐⭐ a espec de ESTADOS do widget |
| 7 | Masonry / Xilem | o toolkit da **Linebender** (a família do Vello e do Parley que usamos) | Apache-2.0 | [`theme.rs`](https://github.com/linebender/xilem/blob/main/masonry/src/theme.rs): 14 cores (rampa *zinc* 900→500, acento `#3b7ee4`) + 12 tamanhos (linha 18/24, borda 1) | é o nosso renderer; o look é genérico | ⭐ |
| 8 | Dear ImGui | a UI de ferramenta canónica (C++) | MIT | `ImGuiStyle`: ~55 cores + `*Rounding`/`*Padding`/`*BorderSize` | vocabulário de *o que um estilo completo precisa de ter* | ⭐ lista de verificação |
| 9 | Adobe Spectrum 2 | design data | Apache-2.0 | tokens JSON (já vendorizado) | o `scale-set` desktop/mobile — o knob do **tablet** | ⭐ só para o tablet |
| 10 | **Blender** | o look que ele quer | ⛔ **GPL** (código) · CC-BY-SA (manual + HIG) | **nenhum** — ver §2.1 | regras e valores **observados**, nunca fonte | ⭐ regras, não código |
| 11 | Zed / gpui | editor Rust muito plano | `gpui` Apache-2.0, mas `ui` e `assets/themes` **GPL** | — | ⛔ os temas são GPL | ⛔ |
| 12 | Floem | toolkit Rust com *design system* | MIT | não vendorizado | look genérico | — |
| 13 | **Mini Cavalry** (o MVP dele) | `/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2` | **dele** | ver §2.2 | é o gosto do dono, já escrito em CSS | ⭐⭐ a régua de gosto |

### 2.1 — Blender: a única porta legítima é a que já usamos

O código é GPL e **não se lê nesta linha** ([`02 §1`](02_referencias_e_licenca.md)). O que se
pode fazer, e chega:

- **Os valores** — o manual (CC-BY-SA) diz que em *Preferences › Themes* cada cor se lê e edita
  «em RGB ou hexadecimal» ([`themes.rst`](../referencias/blender-manual/manual/editors/preferences/themes.rst)).
  Ler um valor no app a correr é **comportamento observado**, não fonte. É o Enio quem tem o
  Blender aberto; o que ele quiser copiar de lá copia-se por esse caminho.
- **As regras** — o HIG (CC-BY-SA) já está vendorizado. A página de cor abre com a frase que
  explica a nossa foto: *«Avoid accent colors when there's no need to grab the user's attention.
  […] The UI should remain calm with subtle contrasts. Currently Blender uses accent colors too
  freely.»* ([`color.md`](../referencias/blender-developer-docs/docs/features/interface/human_interface_guidelines/color.md)).

⇒ **Blender dá as regras e o alvo visual; o Godot dá o código.** Não há contradição: o tema
*Modern* do Godot é, por construção, uma UI «calma com contrastes subtis».

### 2.2 — O MVP do próprio Enio já está nesta família

Medido em `mini-cavalry-v2.html` (2 619 linhas) + `src/editor/`:

| | |
|---|---|
| paleta do chrome | a rampa **`zinc`** do Tailwind, `950 → 200` (`#09090b … #e4e4e7`) — **cinza neutro** |
| acento | `indigo-500/600` para o que está activo; `blue/emerald/amber/red-400` como **dados** (estado, sockets) |
| raio | **`4 px` em 23 sítios**, depois 3 e 2 — o 12 aparece **uma** vez |
| molduras / sombras | 34 bordas de 1 px · 26 `box-shadow` |
| grafo de nós | [`visual-tokens.js`](file:///home/enio/Documentos/Recursos/Nodes/MiniCavalryV2/src/editor/visual-tokens.js): **9 cores de categoria** OKLCH *dark-safe* + **7 silhuetas** + cardinalidade de pino + espessura de fio — e abre com *«Doc PH2D §6»* |

⇒ o gosto dele, escrito por ele, é **rampa neutra + um acento + raio 4** — o mesmo do Godot
Modern, do Graphite e do egui. O `tokens.json` de hoje (fundo tingido, quatro acentos, raio 16,
três sombras) é o que **destoa**, e é por isso que «o slider ficou bom e o app ficou igual».

## §3 — ⭐⭐⭐ Onde os quatro modelos planos CONVERGEM (e é isto que se porta)

| lei | Godot Modern | Graphite | Mini Cavalry | egui |
|---|---|---|---|---|
| fundo é uma **rampa neutra** | `base` `#242424`, o resto por `lerp` para preto/branco | 16 degraus `#000…#fff` de `0x11` | `zinc 950…200` | `gray(27/45/55/60/70)` |
| **um** acento | `#569eff` | `#00a8ff` (overlay) | `indigo` | `rgb(90,170,255)` |
| raio | **4** (0..6) | 0 | 4 | 2–3 |
| borda | `border_size: 0` | — | 1 px | 1 px `gray(60)` |
| texto por **alfa** sobre mono | `0,75 · 0,55 · 0,35` (normal · secundário · desactivado) | rampa | — | `gray(140/180/210/240)` |
| cor de estado é **fixa**, não deriva do acento | info/success/warning/error | error `#d6536e` · warning `#d5aa43` | `red/amber/emerald-400` | `warn/error` |
| **cores de dado** são o único sítio com matiz | — | 9 tipos, cada um com o par `dim` | 9 categorias + silhueta | — |

⭐ **As três coisas que o `tokens.json` de hoje faz ao contrário são exactamente as três primeiras
linhas** — fundo tingido, quatro acentos, raio 16. As outras já batem certo (temos `text-1/2/3`,
`danger/warn/info`, `node-cat-*` com 7 categorias).

## §4 — Recomendação

**Modelo = Godot 4.6 «Modern» (código, MIT) + HIG do Blender (regras) + Graphite / Mini Cavalry
(a régua visual no nosso domínio).** E a integração é *tokens primeiro*, na ordem que a
[`spec/02 §1`](../spec/02_o_que_falta_para_comecar.md) já mandava:

1. **A derivação entra no `ph2d-tokens`.** Portar as ~60 linhas de
   [`_get_base_color` + `populate_shared_styles`](../referencias/godot-editor-src/editor/themes/theme_modern.cpp)
   (Godot, MIT) — `dark_color_1 = base.lerp(preto, contrast·1,15)`, `contrast_color_1/2 =
   base.lerp(mono, contrast·1,15 / ·1,725)`, `highlight = accent@0,275`, `font = mono@0,75/0,55/0,35`
   — e, para os papéis semânticos, a `Extended` do iced (MIT, já em OKLch). Um tema passa a ser
   **5 entradas** (`base`, `accent`, `contrast`, `radius`, `spacing`), e os 83 slots passam a ser
   **derivados**. ⭐ A decisão **B** (os 16 apelidos da timeline) dissolve-se por construção.
2. **Um tema novo, `modern`, ao lado dos quatro** — nasce das entradas do Godot (`#242424` ·
   `#569eff` · `0,3` · `4` · `4`), com `PH2D_UI_THEME` a escolhê-lo e o interruptor que já existe
   (`PH2D_UI_NEW=0`) a devolver o clássico. ⛔ Não se apagam os quatro; eles viram *presets* de
   entradas, como os do Godot (*Default · Gray · Light · Solarized · Black (OLED)*).
3. **A tabela de ESTADOS do widget** (a forma do egui: 5 estados × 6 campos) vira **a única
   porta** que os pintores lêem para fundo/borda/raio/traço — hoje cada pintor decide isso sozinho
   (271 `stroke_rounded_rect`). É a peça que torna «plana» e «coerente» a mesma coisa.
4. **Os quatro elementos de chrome que dão a cara** — moldura de painel, cabeçalho de secção,
   botão, caixa de texto — repintados da tabela. Os 42 widgets seguem porque já lêem tokens.
5. **Medir**: o `Widget Lab` pinta antes/depois; os gates do tablet
   (`the_chrome_never_eats_more_of_a_tablet_than_this`) continuam a cobrar os três alvos.

⛔ **O que NÃO fazer:** ler fonte do Blender (GPL) · copiar temas do Zed (GPL) · redesenhar widget a
widget com a pele antiga — é o que se fez em 02–03/09, e o resultado foi «a mesma cara».

## §5 — ⏳ As decisões que são do Enio

1. **Aceita o Godot 4.6 «Modern» como modelo?** (código MIT, já vendorizado, look pedido.)
2. **Qual é o cinza base:** o do Godot (`#242424`), o do Graphite (`#222`) ou o seu `zinc-900`
   (`#18181b`)? — é uma entrada, muda o app inteiro.
3. **Um acento só?** O azul do Godot (`#569eff`) ou o magenta do `forge`. O HIG do Blender manda
   usá-lo *pouco*.
4. **Quantos temas ficam** (a decisão **I**): com a derivação, um tema custa cinco números — a
   pergunta deixa de ser «4 → 2» e passa a ser *«que presets oferecemos»*.

## §6 — O que ficou vendorizado (gitignorado; `bash fetch-referencias.sh` reconstrói)

| pasta | licença | o que se lê lá |
|---|---|---|
| `godot-editor-src/editor/themes/` *(já existia)* | MIT | `theme_modern.cpp` (2 960) · `theme_classic.cpp` (2 602) · `editor_theme_manager.cpp` (760) · `editor_color_map.cpp` (239) |
| `godot-minimal-theme/` | MIT | o `.tres` original e o README com os valores recomendados |
| `pixelorama/` | MIT | `assets/theme.tres` · `Themes.gd` · `ThemeUtils.gd` |
| `material-maker/material_maker/theme/` | MIT | os temas do editor de nós |
| `graphite/frontend/src/` | Apache-2.0 | `components/Editor.svelte` (paleta) · `components/widgets/**` (27 widgets) |
| `iced/core/src/theme/` | MIT | `palette.rs` |
| `egui/crates/egui/src/style.rs` | Apache-2.0 OR MIT | `Visuals` · `Widgets` · `WidgetVisuals` · `Spacing` |
| `xilem/masonry/src/theme.rs` | Apache-2.0 | as constantes |
| `imgui/` | MIT | `imgui.h` (`ImGuiStyle`) · `imgui_draw.cpp` (`StyleColorsDark`) |

## Fontes

- Godot: [`editor_theme_manager.cpp`](https://github.com/godotengine/godot/blob/master/editor/themes/editor_theme_manager.cpp) · [`editor/settings/editor_settings.cpp`](https://github.com/godotengine/godot/blob/master/editor/settings/editor_settings.cpp) (as omissões: `style = Modern`, `base_color (0.14,0.14,0.14)`, `accent (0.34,0.62,1.0)`, `contrast 0.3`, `icon_saturation 2.0`, `border_size 0`, `corner_radius 4`, `base_spacing 4`) · [godot-minimal-theme](https://github.com/passivestar/godot-minimal-theme) · [Godot Minimal Theme 2.0 — GameFromScratch](https://gamefromscratch.com/godot-minimal-theme-2-0-from-passivestar/)
- Pixelorama: [repositório](https://github.com/Orama-Interactive/Pixelorama) · [Themes.gd](https://github.com/Orama-Interactive/Pixelorama/blob/master/src/Autoload/Themes.gd)
- Material Maker: [repositório](https://github.com/RodZill4/material-maker) · [LICENSE.md](https://github.com/RodZill4/material-maker/blob/master/LICENSE.md)
- Graphite: [repositório](https://github.com/GraphiteEditor/Graphite) · [graphite.art](https://graphite.art/)
- iced: [`palette.rs`](https://docs.iced.rs/src/iced_core/theme/palette.rs.html) · [`Extended`](https://docs.iced.rs/iced/theme/palette/struct.Extended.html)
- egui: [`style.rs`](https://docs.rs/egui/latest/src/egui/style.rs.html) · [`Visuals`](https://openrr.github.io/openrr/egui/style/struct.Visuals.html)
- Masonry: [`theme.rs`](https://github.com/linebender/xilem/blob/main/masonry/src/theme.rs)
- Dear ImGui: [Colors and Styles](https://ocornut-imgui.mintlify.app/styling/colors-and-styles) · [dear-imgui-styles](https://github.com/GraphicsProgramming/dear-imgui-styles)
- Zed: [licença do gpui](https://github.com/zed-industries/zed/blob/main/crates/gpui/Cargo.toml) · [discussão sobre a licença da `ui`](https://github.com/zed-industries/zed/discussions/13694) · [`assets/themes/one/one.json`](https://github.com/zed-industries/zed/blob/main/assets/themes/one/one.json)
- Blender: [Themes — manual](https://docs.blender.org/manual/en/latest/editors/preferences/themes.html)
- Spectrum: [spectrum-design-data](https://github.com/adobe/spectrum-design-data)
- Floem: [repositório](https://github.com/lapce/floem)
