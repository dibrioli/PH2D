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
| fundo é uma **rampa neutra** | `base` `#292929` (o preset *Default*; ⚠️ o valor de fábrica do *setting* é `#242424` e o preset sobrescreve-o), o resto por `lerp` para preto/branco | 16 degraus `#000…#fff` de `0x11` | `zinc 950…200` | `gray(27/45/55/60/70)` |
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

## §7 — ✅ AS DECISÕES (2026-09-04) e a WAVE 1, construída no mesmo dia

> Enio: *«1 — aceito · 2 — [o cinza] do Godot · 3 — o azul do Godot · 4 — decida»*.

**A 4.ª caiu em QUATRO presets, um por slot do menu, e os quatro vêm da tabela `color_preset` do
Godot** — o critério foi *nenhuma cor nova*: `Dark` (o *Default* do 4.6) · `Gray` · `Light` ·
`Black (OLED)`. ⚠️ O cinza é **`#292929`** (`Color(0.161, …)`, o preset), não o `#242424` que o §3
citou: esse é o valor de fábrica do *setting*, e o preset sobrescreve-o.

### 7.1 — O que existe agora

| peça | onde | o que faz |
|---|---|---|
| **a derivação** | [`ph2d-tokens/src/derive.rs`](../../../crates/ph2d-tokens/src/derive.rs) | as regras do `theme_modern.cpp` (MIT), portadas: `mono` · `dark_color_1/3` · `contrast_color_1/2` · `highlight` · `font @ 0,75/0,55/0,35` · as quatro cores de estado (e as versões escuras para tema claro). `Inputs::of(theme)` → `roles()` → `colour(theme, token)` cobre **todo** `ColorToken` |
| **a família moderna** | [`theme.rs`](../../../crates/ph2d-tokens/src/theme.rs) | `Theme::{Dark, Gray, Light, Oled}`, `CLASSIC`/`MODERN`/`ALL`, `is_modern`, `family`, `from_id`, `default_for(look)`; `next` cicla **dentro** da família |
| **a fábrica** | `ColorToken::factory` | um tema moderno **não tem tabela**: a fábrica dele é a derivação — a camada de override, o DTCG e o gate de contraste não sabem a diferença |
| **a tabela de estados** | [`visuals.rs`](../../../crates/ph2d-tokens/src/visuals.rs) | `Widgets` (5 estados × `bg_fill`/`weak_bg_fill`/`bg_stroke`/`fg_stroke`/`corner_radius`, a forma do egui) + `Chrome` (raio e moldura de painel · placa de secção · campo de texto, com o anel de foco a 2 px do Godot). A clássica **descreve** o clássico; a moderna sai dos papéis |
| **os quatro pintores de cromo** | `panel_chrome` · `section_header` · `button` · `text_input` | lêem a tabela: **moldura zero** nos modernos (só o OLED a traça, como no Godot), raio `4`, moldura do campo **só no foco** |
| **o menu de tema** | [`theme_menu.rs`](../../../crates/ph2d-editor-core/src/screens/hero/theme_menu.rs) + `menu_rows` | **uma família por aparência**; o redesenho abre no `Dark` |

### 7.2 — O que a construção ensinou

- ⚠️ **Achatar o alfa é decisão do porte** (`derive.rs`, topo): o gate de contraste mede a cor do
  token e não a compõe, e há pintores que constroem o `Color` do Vello dos três canais — os slots
  opacos continuam opacos, compostos sobre a base na derivação.
- ⚠️ **As cores de DADO emprestam-se, não se derivam**: `node-cat-*`, `port-*`, `curve-*`,
  `graph-backdrop-*` vêm da tabela do `forge` (escuros) / `sunstone` (claros) — são as únicas com
  matiz por direito. Os eixos são os do Godot.
- ⚠️ **Só cinco `match` exaustivos sobre `Theme` existiam no produto**, e três eram cópias do
  `id()`/`display_name()` — morreram. Uma família nova custou **uma** tabela (`theme_menu.rs`).
- ⛔ **O gate `every_menu_row_reaches_a_handler` lê o FONTE**: a tabela `id ⇄ tema` teve de viver
  num ficheiro próprio e não em `menu_rows.rs` (excluído do censo), senão as oito linhas de tema
  acusavam «sem despacho» com o despacho a funcionar.
- ⚠️ **Os 16 apelidos da timeline dissolveram-se por construção** (gate
  `the_timeline_slots_are_aliases_by_construction`) — a decisão **B** deixa de existir na família
  moderna.

- ⛔⛔ **E o smoke apanhou o tema de arranque com DUAS portas** (Enio: *«o app abriu como era
  antes. Mas ao mudar o theme, as cores mudaram»*): o `HeroScreen::new` já abria no `dark`, e a
  shell — `init.rs`, dois passos depois — escrevia por cima com `resolve_theme(PH2D_THEME)`, cujo
  `None => Theme::Forge` ninguém tinha mudado. *Duas portas para o mesmo default, e a que corre
  por último é a que ninguém lembra.* Cura: uma lei ([`Theme::default_for`]) e as duas portas a
  lê-la, com `PH2D_THEME` a aceitar os oito ids. ⚠️ E o byte do tema no ficheiro de projeto
  (`theme_from_u8`) ganhou os quatro degraus novos — o gate passou a percorrer `Theme::ALL` em vez
  de uma lista de quatro.

### 7.4 — ✅ WAVE 2 (2026-09-05): a PORTA DA MOLDURA, e o censo que a torna obrigatória

*«smoke ok. siga»* (Enio). O que a wave 1 deixou nomeado — *os outros pintores traçam molduras
onde a tabela diz zero* — fechou por uma porta, não por 38 `if`:

| peça | onde |
|---|---|
| **a porta** | [`visuals::frame(theme, feel) -> Frame`](../../../crates/ph2d-tokens/src/visuals.rs) e `visuals::radius(theme, classic)`: no clássico `Frame::Classic` (*«traça o teu»*, byte-idêntico); num tema moderno o traço da tabela — **nenhum** em repouso/hover/activo (só o OLED), o anel de foco a 2 px, e o **erro sempre** |
| **o vocabulário** | `Feel { Rest, Hovered, Active, Focused, Disabled, Error }` — o mínimo comum aos cinco enums de estado dos pintores |
| **a chamada** | [`paint::stroke_frame(scene, rect, radius, theme, feel, w, colour)`](../../../crates/ph2d-editor-core/src/paint.rs) + `paint::frame_radius(theme, classic)` — a cor clássica chega já misturada no eixo do hover, então o pintor não perde a animação que tinha |
| **24 pintores convertidos** | segmentado · trilho (4 chips) · caixa de verificação · campo de número · campo de texto · área de texto · dropdown (chip + popover) · abas · etiqueta (⭐ deixa de ser pílula: raio 4) · botão de ícone (⭐ `Xl` 16 → 4) · cartão (⭐ `Lg` 12 → 4, sem contorno) · amostra de cor · menus de contexto (5 corpos) · popover · tooltip · modal · paleta de comandos (4) · caixa única (hover sem contorno, edição com anel) |

⚠️ **E o `paint.rs` bateu o tecto de 700 LOC (731)** — cura por corte, nunca por folga: o que a
shell **publica por quadro** (escala de raio · estilo das linhas · aparência · texto) mudou-se para
[`published.rs`](../../../crates/ph2d-editor-core/src/published.rs), com os caminhos `paint::…`
mantidos por re-export (43 chamadores).

**Os gates:**
- [`every_frame_goes_through_the_theme_door`](../../../crates/ph2d-editor-core/tests/every_frame_goes_through_the_theme_door.rs)
  — censo pelo FONTE: todo ficheiro de `widget/` + `screens/hero/` que chame `stroke_rounded_rect`
  conhece a porta, ou está em `NOT_YET` (**22 ficheiros**, só encolhe) ou em `EXEMPT` (4, por
  mecanismo: a porta, o pintor só-clássico, a pele de documento, os contornos de canvas que **são**
  a mensagem). Com a metade de obsolescência. ⚠️ A 1.ª corrida acusou **9** ficheiros que o meu
  `grep -c` de véspera não tinha visto — *um censo escrito à mão conta o que o autor lembrou*.
- [`the_modern_family_paints_fewer_frames`](../../../crates/ph2d-editor-core/tests/the_modern_family_paints_fewer_frames.rs)
  — carrega no PIXEL: a galeria inteira pintada no `forge` e no `dark`, e o `dark` emite
  **estritamente menos caminhos** (as molduras que não estão lá); controlo: dois temas da mesma
  família emitem geometria **igual**.

### 7.5 — ✅ WAVE 3 (2026-09-05): a DÍVIDA da wave 2 está a ZERO

**O que existe agora:** `NOT_YET` **vazio** — os 22 ficheiros passaram pela porta: **20
convertidos** (33 sítios de `stroke_rounded_rect`) e **2 isentos por mecanismo** (as alças do
`rect2_editor`, que são gizmo sobre conteúdo, e o contorno de secção do showcase, que é a cor de
marcador que o utilizador escolheu — conteúdo, não cromo). Os pintores convertidos: o fantasma de
arrasto de asset · a janela do Input Map · o avatar · o picker de cor do Blender (cartão ×2,
harmonia ×2, hex, conta-gotas, faixa de matiz, amostra) · o picker de cor clássico (cartão + amostra)
· combobox · menu de contexto · lista chave-valor · menu radial (disco + etiquetas) · rádio
segmentado · anel de foco do slider · chip numérico · barra de estado (⭐ deixa de ser pílula) ·
linha seleccionada da árvore · os modais de fill/onion (cartão + amostra de fantasma) · os chips da
barra do topo (rail-chip + chip largo).

**O que a construção ensinou:**

- ⭐⭐ **Um indicador desenhado POR CIMA do widget, fora da tabela de estados dele, morre quando a
  tabela muda.** O modo *Image Tools* ligado era um anel de acento traçado pelo `paint_top_bar`
  sobre o chip — reconstruindo o `chip_rect` à mão, sem o chip saber que estava em modo. Num tema
  moderno (moldura em repouso = `0`) o anel sumiria e **o modo ficaria invisível**. A cura não foi
  «passar o anel pela porta» (a porta devolveria *nada* para `Feel::Active`, que é o desenho), foi
  **o chip saber que está activo**: `paint_topbar_rail_chip(.., active)` e `is_active = active ||
  Pressed` — a mesma matriz do rail para a ferramenta em mãos (`AccentSoft` + contorno de acento no
  clássico, o realce do tema no moderno). ⚠️ É a **única mudança visível no clássico** desta wave: o
  chip do modo ligado passa de *BgElev + anel* a *AccentSoft + contorno* — o look que o rail já dava
  à ferramenta activa, e que este pintor declarava copiar. Gate:
  `the_image_tools_chip_shows_the_mode_in_every_family`.
- ⭐ **O `chip_feel` (estado do botão → `Feel`) mudou-se para o `button_surface`**, ao lado do
  `chip_axis_t`: o rail e os chips do topo declaram copiar a mesma matriz, e uma cópia privada da
  redução no `tool_rail/paint.rs` divergiria no dia em que um estado novo entrasse num só lado.
- ⚠️ **Nem tudo o que é `stroke_rounded_rect` é moldura**, e a porta do RAIO não é para toda
  forma: o **círculo** do avatar e o **disco** do menu radial são a FORMA (a porta faria deles
  quadrados de raio 4) — só o traço passa; e o **halo** do cursor da faixa de matiz (um `Bg0` de
  1 px à volta do cursor claro) é contraste sobre a cor, não cromo — fica, com o motivo no código.
- ⚠️ **A amostra de cor do utilizador fica PLANA no moderno** (harmonia, pré-visualização, cor de
  fantasma): os presets do `ColorPicker` do Godot não têm borda. ⏳ A `color_swatch.rs` (wave 2)
  ainda pinta o anel de repouso como **preenchimento** em `Border` — não é um `stroke`, o censo não
  o vê, e é o próximo a olhar se as amostras destoarem.
- ⚠️ **O censo do FONTE vê que o ficheiro *conhece* a porta, não que a *atravessa*** — um pintor que
  chamasse `stroke_frame` com `Feel::Error` em repouso passaria nele e traçaria na mesma. Daí o
  gate de pixel desta wave: `the_wave_three_painters_lose_exactly_their_frame` (avatar · barra de
  estado vazia · menu de contexto vazio, cada um `2 → 1` caminhos e OLED `= clássico`).

### 7.6 — ✅ WAVE 4 (2026-09-05): a porta chega aos PAINÉIS — onde o artista vive

**A medição que a abriu:** com a dívida do `editor-core` a zero, **59 ficheiros em
`crates/ph2d-panel-*/src` traçavam moldura à mão (79 sítios) e nenhum conhecia a porta** — o censo
da wave 2 só via `widget/` e `screens/hero/`. *Um censo que varre um directório afirma sobre o
directório.*

**O que existe agora:**

| peça | o que é |
|---|---|
| **o censo alargado** | [`every_frame_goes_through_the_theme_door`](../../../crates/ph2d-editor-core/tests/every_frame_goes_through_the_theme_door.rs) varre também `crates/ph2d-panel-*/src` e `shells/desktop/src` (chaves com o prefixo da crate). Red-first: acusou **exactamente os 59** antes de uma conversão |
| **72 sítios pela porta, 7 isentos por mecanismo** | cartões (Inspector · 7 do Painter) · canvases de curva/falloff · barras de gradiente e amostras de paleta · chips com estado (blend · brush · shape · paper · dropdown do Painter · chrome do grafo) · botões (stack lane · container list · preview toggle · apply) · campos de renomear (asset · clip · marker ⇒ `Focused`) · linhas seleccionadas com tinta (Hierarquia · Flip ⇒ `Active`) · popovers (font/icon dropdown · menu do grafo · probe · lista de tracks) · nós e backdrops do grafo · previews de imagem. Isentos: o marquee do grafo · o crachá `pre` no fio · o halo de socket · os indicadores de «largar aqui» (Flip · Hierarquia · Painter) · o contorno de secção em cor de marcador (Inspector) · a diagonal da matriz de física · as **strips** da timeline (duas adjacentes com a mesma tinta só se separam pelo contorno) |
| ⭐ **`Feel::Selected`** | a peça que faltava no vocabulário: **SELECCIONADO entre iguais, onde a tinta não chega** (um nó no grafo, a amostra escolhida entre várias, uma linha activa sem preenchimento). Lido da fonte do Godot: o `GraphNode` Modern tem `border_width 0` em repouso e o **seleccionado leva 2 px em `mono`** (`gn_panel_selected_style`, `editor_theme_manager.cpp:1444`) — não no acento. É a terceira moldura que um tema moderno traça (foco · erro · selecção). ⚠️ **Não é o `Active`**: um controlo activo COM tinta própria diz-se pela tinta |
| `dropdown_feel` | a redução `DropdownState → Feel` sai do pintor do `Dropdown` para uma porta `pub`, porque o Painter desenha um chip de dropdown à mão (a mesma razão do `chip_border_color`) |
| **gate de pixel num painel** | `the_histogram_surface_loses_exactly_its_frame_in_a_modern_theme` (color-equalization): `2 → 1` caminhos, OLED = clássico |

**O que a construção ensinou:**

- ⭐⭐ **Três anéis eram o ÚNICO sinal de um estado** — a linha activa da máscara, a linha activa das
  camadas do Painter (contorno sem tinta) e o chip *leveling* (a MESMA tinta ligado e desligado).
  Passá-los por `Active` apagava o sinal no moderno: a lição do Image Tools (§7.5), três vezes no
  mesmo dia. A resposta certa não era uma excepção na porta, era **nomear o estado** (`Selected`),
  que o Godot já distingue.
- ⚠️ **A porta faz o ficheiro CRESCER** (9 linhas onde havia 1-7): quatro painéis passaram o teto de
  600 LOC. Cura por corte, nunca por folga — `paint_adjust.rs` perdeu os dois editores bespoke
  (`paint_adjust/{curve,gradient}.rs`) e **a folga de 823 que carregava desde a W4 do Painter
  morreu**; `motion-graph/paint.rs` cedeu os sockets (`paint_socket.rs`); `flip/paint_layers.rs` o
  chip+popover de blend (`paint_layers/blend.rs`); `asset-browser/paint.rs` o cartão
  (`paint/card.rs`). ⚠️ Um irmão novo cujo nome começa por `paint` entra no censo HR-12 — o
  `paint_socket.rs` precisou do opt-out com motivo.
- ⚠️ **Nem todo traço é moldura, e a lista de isenções cresceu com o motivo de cada um** — o censo
  só recusa o que *não conhece* a porta; quem traça mensagem (halo · marquee · «largar aqui») fica
  com o traço directo **e um comentário que diz porquê**, ou na lista `EXEMPT` se é o único traço do
  ficheiro.
- ⚠️ **O raio de um nó do grafo NÃO passa pela porta do raio**: ele escala com o zoom, é geometria
  do documento e não uma quina de cromo. Só a moldura passa.
- ⏳ **Dois sítios para o smoke julgar**: o contorno das células «rótulo | interruptor» do
  transporte da timeline (pedido do Enio em 2026-07-08 — fica intacto no clássico, plano no
  moderno) e a fronteira coluna/grade do navegador de assets (uma linha no clássico, a diferença de
  tom no moderno).
- ⏳ **Fora desta wave**: os traços por OUTROS primitivos (`stroke_rect` · `stroke_line` — 6 sítios
  no `editor-core`, dividers na maioria) e os anéis pintados como PREENCHIMENTO em `Border` (10
  sítios: o anel de repouso do `color_swatch`, divisores) — o censo não os vê, e são a próxima
  medição.

### 7.7 — ✅ 2026-09-05, os reports do smoke da wave 4: a porta do ASSETS e o CONTRASTE do cartão

**(a) *«não há meio de abrir assets»*** — a porta do navegador de Assets era **só** o chip
`TOPBAR_RIGHT_ASSETS` do grupo direito da barra legada, que o redesenho não pinta; o censo de
alcance (`the_bar_relocated_every_row_of_the_menus_it_replaced`) só percorria os **menus** que a
barra substituiu, e um chip que despachava sozinho não estava em lista nenhuma — a família do
*Export SVG*, um nível ao lado. Cura: a linha **Assets** no menu *Window* (o mesmo id; o handler
continua no `ph2d-panel-asset-browser`), a lista `LEGACY_PILL_BUTTONS` e a segunda metade do gate
(red-first: acusou *«Assets (botão directo da barra legada)»* antes de a linha existir). ⚠️ **A nota
que o escondia tinha envelhecido ao contrário**: o `NO_DOOR_PENDING` dizia *«sem consumidor»* sobre
um id cujo consumidor nasceu com o painel. ⛔ `Layers` e `Script` ficam fora **com o motivo**:
nenhum `apply_event` do app os trata (chips mudos também no clássico), e uma linha de menu para um
id sem handler é uma linha morta.

**(b) *«o fundo dos cards tem tão pouco contraste com o fundo dos painéis»*** — medido: **4/255**
no Dark (cartão `Bg1` = `dark_3` = `#1f1f1f`, painel `PanelBg` = `dark_1` = `#1b1b1b`).

⛔⛔ **A PRIMEIRA cura foi construída, shipada e REVERTIDA pelo dono no smoke seguinte, e a lição é
a mais cara desta linha: o `Bg1` responde a DUAS perguntas.** Ele é o fundo dos cartões **e o
fundo do CANVAS** (`hero::canvas_backdrop` — o `canvas.rs` di-lo em letra: *«é o `Bg1`, e não o
token `canvas`»*, e o `clear` do shell lê a mesma porta). Eu reassentei a escada inteira na pilha
do Godot (painel na `base`, `Bg1` em `surface_high`, os botões em `button_*`) e o cartão passou a
destacar-se — **clareando o canvas de `#1f1f1f` para `#393939`**: *«mudou a cor do canvas»*.
⚠️ **E a minha correcção seguinte errou o alvo por não medir**: devolvi o `Bg0` (a moldura do
canvas), não o `Bg1` — *«não corrigiu nada do que pedi»*. Só ao ler o `canvas.rs` até ao fim é que
o token certo apareceu.

⇒ **Quem se move é o PAINEL, que é cromo e não tem outra pergunta agarrada** (`Roles::panel =
base.lerp(black, c·1.8)`), e é também o que o Blender faz: painéis mais escuros que a área de
trabalho. ⛔ Subir o cartão para o `Bg2` estava fora por medição: um botão em repouso PINTA `Bg2`
(`flat_button_surface`), e os botões dentro do cartão desapareceriam.

| tema | painel | cartão = canvas (`Bg1`) | degrau | botão (`Bg2`) | texto-2 |
|---|---|---|---|---|---|
| Dark | `#131313` | `#1f1f1f` | **+12** | `#292929` | `#bfbfbf` (era `#a3a3a3`) |
| Gray | `#1c1c1c` | `#2f2f2f` | **+19** | `#3d3d3d` | `#c5c5c5` |
| Light | `#fefefe` | `#f1f1f1` | **−13** | `#e6e6e6` | `#454545` |
| OLED | `#000000` | `#000000` | — (excepção) | `#000000` | `#b3b3b3` |

**(c) *«as fonts dos cards podem ser um pouco mais claras»*** — os títulos e os rótulos dos cartões
do Painter são todos `Text2`. O piso da alfa sobe de `0.55` (o valor do Godot) para **`0.70`**, e o
`Text1` acompanha (`0.75 → 0.80`) para a hierarquia entre rótulo e valor continuar a ler-se.

**Os gates que ficam:** `the_canvas_ground_is_the_one_the_owner_approved` (o `Bg1` preso ao
`dark_3` e o `Bg0` ao `dark_1` — as fórmulas que ele viu; com a **recusa medida da escada** escrita
no doc-comment) e `a_card_stands_off_its_panel` (≥ 12/255, OLED de fora).

**O que a construção ensinou:**

- ⭐⭐⭐ **Um token que responde a duas perguntas move as duas.** O `Bg1` é o canvas *e* o cartão;
  nenhum gate ligava os dois, e o `canvas.rs` já avisava — *o aviso estava escrito no ficheiro que
  eu não li antes de mexer no token*. A régua que faltava é agora um gate.
- ⚠️ **Uma correcção que não mede o alvo não corrige nada**: reverti o `Bg0` porque o `paint_canvas_bg`
  o pinta primeiro, sem verificar QUEM pinta o pixel que o dono vê (em modo vivo aquele fill é
  saltado e o `clear` do shell usa o `Bg1`).
- ⛔ **A pilha de superfícies do Godot não se importa sem os papéis dela**: ela assume um token por
  papel (poço · painel · cartão · botão), e aqui um token serve dois papéis.

### 7.8 — ✅ WAVE 5 (2026-09-05): a moldura que NÃO é um traço, e três itens fechados por medição

**O buraco que a wave 4 nomeou, medido:** dos três candidatos, dois estavam limpos e um era real.

| candidato | medido |
|---|---|
| **sombras** | ⛔ **não existem**: nenhum consumidor de token de sombra no código (o `tokens.json` ainda os carrega; o `§0` deste doc contava-os como parte da «mesma cara») |
| outros primitivos de traço (`stroke_rect` · `stroke_line` · `stroke_polyline`) | ⛔ **não são molduras**: são a curva de um editor, os fios do grafo, a grade, o cursor de uma faixa — conteúdo |
| **anéis por PREENCHIMENTO** | ⭐ **um real**: a amostra de cor pinta um rect na cor da borda e põe a cor do artista por cima, recuada — e continuou emoldurada depois de a pele plana ter apagado as vizinhas |

**A porta:** [`paint::fill_ring(theme, feel, classic_w, classic_colour) -> Option<(recuo, cor)>`](../../../crates/ph2d-editor-core/src/paint.rs), a irmã do `stroke_frame` para a moldura que é um preenchimento. ⚠️ **O recuo É a largura do anel** e volta da porta: escolhê-lo no pintor daria meia moldura no foco.

**O gate:** [`a_ring_painted_as_a_fill_is_still_a_frame`](../../../crates/ph2d-editor-core/tests/a_ring_painted_as_a_fill_is_still_a_frame.rs) — ⚠️ **a régua não pode ser «um `fill` numa cor de borda é suspeito»**: a maioria é legítima e não é moldura (divisores, a trilha de um slider, a bandeja de um grupo), e o fonte não distingue — a espessura vive numa variável. ⇒ a régua é a **DECLARAÇÃO**: os 8 sítios estão numa tabela dizendo o que são (`Ring` · `Divider` · `Track`), e um `Ring` **tem de chamar a porta**. Com a metade de obsolescência. Mais o gate de pixel `the_colour_swatch_loses_the_ring_that_was_a_fill` (clássico 2 caminhos, moderno 1, OLED = clássico).

**Três itens do §7.3 fechados por MEDIÇÃO, sem escrever código:**

- ✅ **A fonte já é a que o Godot recomenda** — `InterVariable.ttf` (SIL OFL) vem embutida em `ph2d-text`, com o eixo `opsz`. O item dizia *«não medido»*.
- ✅ **O `Spacing` já é o do Godot**: o `base_spacing 4` dele é o nosso `Xs`, e a escala (`2 · 4 · 6 · 8 · 12 · 16`) é múltipla de 4.
- ⏳ **O ritmo da linha continua a ser decisão do dono, e agora com número**: a nossa linha mede **28 px** com fonte **13**; o Godot Modern dá **26** ao botão (fonte 14 + 6+6) e **22** ao campo de texto. Baixar é o único knob de densidade que sobra, e mexe na geometria de todos os painéis — não se faz sem ele.

### 7.9 — ✅ WAVE 6 (2026-09-06): a LINHA fica compacta, e as quatro cópias que ela revelou

**Ordem do dono:** *«linhas dos painéis mais compactas (28 px → 24 px) — sim»*. `chrome.row-h`
**28 → 24** no [`tokens.json`](../../design/tokens.json) — um número, e os 647 sítios que o lêem
encolhem juntos (⚠️ 18 deles são contexto `const`, o que fecha a porta a uma função com tema: a
altura de linha é geometria de LAYOUT, calculada antes de haver tema, ao contrário do raio, que é
escalado na pintura).

**O que se ganha, medido:** um painel de 600 px passa de **21,4 para 25 linhas** (+17 %). E o
`motion.spline_wrap` **saiu da lista de painéis que estouram o dock** — foi de `755` para `679`
sobre um corpo de `754`, sem ninguém lhe tocar; o `motion.bezier_warp` desceu de `969` para `873`
e continua nomeado (24 params são a superfície da referência).

⭐⭐⭐ **E a mudança de um número revelou QUATRO cópias dele**, todas `28.0` escrito à mão — o valor
que o token tinha: `TOGGLE_H` e `HEX_ROW_H` (o picker de cor), o `SWATCH_W` do rig de impasto
(**cujo doc já dizia *«sized to the row height»***) e o `ROW_H` do editor de áudio (*«transport
button row height»*). ⚠️ **Nenhum teste ficou vermelho:** o app teria linhas de duas alturas e o
defeito só se vê a olho. As quatro passam a ler o token, e a espécie morre num censo:
[`the_row_height_is_one_number`](../../../crates/ph2d-editor-core/tests/the_row_height_is_one_number.rs)
— toda constante cujo NOME diz «altura de linha» ou deriva do token, ou está declarada com a
régua própria (as **9** legítimas: listas densas de 22 px, a linha de socket do grafo que escala
com o zoom, o alvo de toque de 44 px de uma barra de progresso, o menu flutuante da vista 3D).
⚠️ **A régua separa PALAVRAS, não subcadeias** — a primeira versão, escrita a `grep`, acusou
`DUR_ARROW_HALF_W` e `NARROW_HALF` três vezes: `ROW_H` vive dentro de `ARROW_H`.

⚠️ **E o portão da shell apanhou uma regressão da WAVE 4** que as suítes de painel não viam: o
`the_asset_card_asks_the_law_instead_of_painting_the_swatch` lê o FONTE de
`ph2d-panel-asset-browser/src/paint.rs`, e o cartão mudou-se para `src/paint/card.rs` quando o
ficheiro cruzou o tecto de LOC. O gate ficou vermelho sobre código correcto. ⇒ a lente dele passa
a ser a **crate**: *um censo que aponta a um ficheiro mede o sítio, não a lei — e quem corta um
ficheiro em dois não devia ter de saber que gate de outra pessoa aponta para ele.*

⏳ **Fica para o dono:** a linha da **hierarquia** continua em **32 px** (`chrome.hier-row-h`), a
mais alta do app — com as de formulário a 24, ela destoa. Baixá-la aperta o ícone, o nome, o olho
e o cadeado na mesma linha; é medição de uma wave, não um número a mudar.

### 7.10 — ✅ WAVE 7 (2026-09-06): a FOLGA — o vão entre controlos e o ar à volta do risco

**Ordem do dono** (com foto do editor de áudio): *«distância entre botões e espaço entre
divisores ainda excessivo»*. As duas metades são grandezas diferentes e as duas foram medidas
contra o Godot Modern, que é o modelo desta linha.

**(a) O RISCO entre secções: 17 px → 9.** O `paint_section_separator` punha `Spacing::Md` de cada
lado de uma linha de 1 px ⇒ **8 + 1 + 8 = 17 px** de altura reservada. O Godot dá **8** ao
separador *inteiro* (`separation = base_margin · 2`, com o `StyleBoxLine` a levar margens
**negativas** de `−base_margin`, que é como ele desconta a própria espessura). Com o `Xs` ficam
**9** — o número do modelo mais o pixel da linha, que nós não descontamos porque o nosso risco é
pintado dentro da faixa. ⚠️ Um número, **30 chamadas em 15 ficheiros**.

**(b) O VÃO entre controlos: `Sm` → `Xs`, e ele já era `Xs` na maioria do app.** O `separation_margin`
do Godot Modern é **`base_spacing = 4`**, e o censo da árvore devolveu a divisão: **64 sítios já
usavam `Xs` (4)** e **82 usavam `Sm` (6)** — *duas respostas à mesma pergunta, e a que o artista vê
depende do painel em que está*. Os 82 (65 `let gap` + 17 `let row_gap`, **54 ficheiros**) passam a
`Xs` por renomeação com `assert` de contagem. ⛔ Isto **não** é um token novo: a escada de espaço
não mudou: mudou quem a lê.

⚠️ **O que a wave revelou, e é a lição da wave 6 outra vez:** os dois retratos de altura de dock
mexeram-se **sem ninguém tocar num painel**. O `motion.bezier_warp` foi de `873` para **`825`** (e
continua a ser o único nomeado), e a fixtura do gate da **rolagem** deixou de servir: ela nomeava
`source.shape` à mão, e o painel encolheu para `734` num corpo de `754` ⇒ *deixou de estourar, logo
deixou de testar rolagem alguma, e o gate teria ficado **verde por vacuidade***. ⇒ a fixtura passa a
ser **DERIVADA** do próprio censo (`height_census().first()`, o painel mais alto que existir hoje):

```rust
let (tallest, _) = height_census().first().copied().expect("o registry não é vazio");
```

⭐ *Uma fixtura escrita à mão para «o painel que estoura» é um retrato de uma árvore; toda wave de
espaçamento a desactualiza, e a forma de falhar é a pior — o gate fica verde.*

⏳ **Fica para o dono, com o número ao lado:** a linha da **hierarquia** segue em **32 px**
(`chrome.hier-row-h`) contra as 24 de formulário — herdada da wave 6 e agora mais visível, porque
tudo em volta dela apertou.

### 7.11 — ✅ WAVE 8 (2026-09-06): as LEIS do Godot e do Blender, e a que nos faltava

**Ordem do dono**, com três fotos lado a lado (Blender · nós · Godot): *«veja a diferença entre
nós e o Blender e a Godot. Blender e Godot com aspecto muito mais compacto e profissional.
Espaçamento muito regrado e universal.»* E a seguir: *«vá até ao código da Godot e Blender para
encontrar as leis necessárias para a nossa UI.»*

⚠️ **A triagem desta linha ([§1 do doc 02](02_referencias_e_licenca.md)) decide de onde cada
metade vem:** o editor do Godot é **MIT** — lemos **e portamos**; o código do Blender é **GPL** e
esta linha não o lê, por decisão já escrita. O Blender entra pelo **HIG** dele (CC-BY-SA), que a
própria triagem chama de fonte melhor, *«porque diz a intenção, que o código não diz»*.

#### As leis do Godot 4.6 «Modern» (medidas em `editor/themes/`, MIT)

Com o `base_spacing = 4` de fábrica (`editor_theme_manager.h:67`):

| lei | derivação | valor |
|---|---|---|
| **G1 — nenhum espaço é escolhido** | tudo é `base_margin · k`, `k ∈ {0.75, 1, 1.5, 1.75, 2, 2.5, 3, 4}` | — |
| **G2 — o vão entre irmãos tem NOME** | `separation_margin`, lido por `BoxContainer`, `HBox`, `VBox`, `GridContainer`, `FlowContainer`, `FoldableContainer` | **4** |
| **G3 — uma LISTA não tem vão: as linhas encostam** | `Tree.v_separation = pow(base_margin · 0.175, 3)` = `0,343` | **0** |
| **G4 — uma GRELHA é mais apertada que uma pilha** | `GridContainer.v_separation = widget_margin.y − 2` | **3** |
| **G5 — o separador de secção é `base · 2`** | `Separator.separation`, com o `StyleBoxLine` a levar margens **negativas** de `−base_margin` | **8** |
| **G6 — o vão vertical é forçado a PAR** | *«if the vsep is odd it will be lopsided»* — `forced_even_separation` | par |
| **G7 — o botão é `(base·2, base·1.5)` de conteúdo** | `button_style.content_margin` | (8, 6) |

⭐⭐⭐ **A lei que responde ao dono é a G1+G2, e ela é sobre o MECANISMO, não sobre o número:**
*o espaço não é escolhido onde se pinta — ele tem um nome, e o nome é o que impede a segunda
resposta.*

#### As leis do HIG do Blender (CC-BY-SA, `human_interface_guidelines/layouts.md`)

⚠️ **Sem um único número** — é intenção, e por isso complementa o Godot em vez de repetir:

- **B1 — Property Split**: rótulo à esquerda, controlo à direita, na MESMA linha, alinhados por
  todo o painel.
- **B2 — Order of importance**: o mais usado em cima; o resto abaixo ou em sub-painel.
- **B3 — Enums**: *dropdown* acima de 2–3 itens; abaixo disso, **expandido a toda a largura no
  topo** quando a propriedade define o painel (é o `None | Vertices | Faces` da foto dele — um
  controlo **segmentado**, com vão ZERO entre as partes).
- **B4 — Sub-painéis acima de «um rótulo por cima de um bloco de botões»**: *«o título de um
  sub-painel ocupa pouco mais que um rótulo, organiza mais, e permite recolher»*.
- **B5 — ⛔ Não usar disposição espacial para comunicar sentido.**

#### ⛔ O que nós tínhamos: SETE respostas para UMA pergunta

Censo de 2026-09-06 sobre *«quanto avança de uma linha para a seguinte?»*:

| onde | valor | alcance |
|---|---|---|
| `ROW_H_PX + Spacing::Xs` | 4 px | 21 sítios |
| `ROW_H_PX + Spacing::Sm` | **6 px** | 20 sítios — o **Inspector** e o **Painter Layers** inteiros |
| `ROW_H_PX + Spacing::Xxs` | 2 px | 3 sítios |
| um local `gap` / `row_gap` | 4 px | 52 sítios |
| `grid_snap::layout::row_gap()` | **6 px** | escondida atrás de uma função |
| `showcase::row_gap()` | **6 px** | 18 chamadas — a maquinaria de que o Inspector é feito |
| `asset_browser::paint::gap()` | **6 px** | 13 chamadas |

⚠️⚠️ **As três últimas são a lição:** uma cópia atrás de uma **função** não aparece na varredura
que procura o operador. A primeira leitura contou **quatro** respostas porque procurou
`ROW_H + <espaço>`; as outras três só apareceram ao perguntar *«que função desta árvore devolve
um degrau da escada e chama-se vão?»*. *Um censo que procura a FORMA de uma expressão é cego a
quem lhe deu um nome.*

⚠️ **E a escada NÃO era o defeito, apesar de ser o suspeito óbvio:** a nossa
(`2·4·6·8·12·16·24·32·48`) é `base·k` com `base = 4` em **todos** os degraus — o mesmo vocabulário
do Godot. *O defeito nunca foi que degraus existem; era que a escolha se fazia no sítio da
pintura.*

#### A cura: a porta, e o portão que a torna lei

[`ph2d_tokens::row_gap_px()`](../../../crates/ph2d-tokens/src/spacing.rs) (o vão — o primitivo do
modelo, G2 = **4 px**) e `row_pitch_px()` (a conveniência, `altura + vão`). **99 sítios** e as
**7** cópias passam por ela. ⚠️ A porta nasceu com a forma errada — só sabia responder
`altura + vão`, e há sítios que empilham uma caixa cuja altura é medida em tempo de pintura;
*uma porta que só serve metade dos chamadores deixa a outra metade a escrever o número.*

O portão é [`the_gap_between_two_rows_is_one_answer`](../../../crates/ph2d-editor-core/tests/the_gap_between_two_rows_is_one_answer.rs),
com **duas** metades porque as cópias tinham duas formas: nenhum sítio escreve o passo à mão, e
nenhuma função chamada «vão de linha» escolhe um degrau. **2 de 2 mutações mortas.**

#### ⚠️ O defeito que EU introduzi, e que a suíte apanhou

O renomeio tratou `ROW_H` como um nome só. A barra de progresso declara o **seu próprio**
`ROW_H = 44 px` (um alvo de toque, já declarado no censo irmão), com vão `Md` — e passou a
avançar 28. `progress::tests::column_rows_never_overlap` foi vermelho com a mensagem exacta
(*«row at y=44 overlaps the row above it»*). ⇒ revertido, e a auditoria a seguir conferiu **um a
um** que todo `gap` local substituído valia mesmo 4 px no `HEAD`. *Uma renomeação que casa por
NOME tem de perguntar o VALOR de cada casamento.*

#### ⏳ O que a wave ACHOU e NÃO fez (com o número, para o dono decidir)

- ⛔ **`chrome.section-gap` (14 px) não tem um único consumidor da pergunta que nomeia.** Os
  **quatro** usos reais tratam-no como **tamanho de ícone** (a seta do menu de contexto, o
  chevron da barra do topo, o interruptor da hierarquia, o piso da altura de um chip). É a família
  *«um controlo que mente»*: o nome responde a uma pergunta e os consumidores fazem outra. ⛔ Não
  lhe toquei — mudá-lo encolhe quatro ícones. A cura é uma wave própria (dar aos ícones o token
  deles e devolver o nome à secção).
- ⏳ **O fim de um GRUPO ainda tem duas respostas** (`Md` = 8 em 4 sítios, `Lg` = 12 em 1) — a G5
  do Godot diz **8**. São 5 sítios; ficam nomeados no portão.
- ⏳ **As duas superfícies de LISTA não seguem a G3** (as linhas deviam encostar): a hierarquia
  avança `HIER_ROW_H + 2` e a lista de variações do áudio `22 + 4`. É a mesma medição que a
  pergunta aberta da altura de 32 px da hierarquia.
- ⏳ **B1/B3/B4 do Blender são composição, não espaçamento** — e é aí que está a outra metade da
  distância para as fotos dele: a grelha de 22 botões iguais do editor de áudio é exactamente o
  *«rótulo por cima de um bloco de botões»* que a B4 manda trocar por sub-painéis, e não temos
  controlo **segmentado** (vão zero) para o que é uma escolha entre irmãos.

### 7.12 — ✅ WAVE 9 (2026-09-06): o CARTÃO substitui o RISCO — o modelo de painel do Blender

**Ordem do dono**, com três telas e a palavra `CARD` escrita à mão sobre a do Blender:
*«Gostei do modo Blender onde uma secção está dentro de um card. Uma subsecção está com o seu
título dentro do card da secção mas o seu conteúdo fica dentro de outro card/container de cor
diferente. Estude o Blender e traga isso para nós. Vamos eliminar os nossos divisores azuis.»*
Mais quatro queixas na mesma mensagem: *«espaços grandes, irregulares»* · *«botões com quinas de
raios altos»* · *«espaçamento entre divisores irregular»* · *«o nome de um efeito de áudio parece
um botão»*.

#### O modelo, do manual do Blender (CC-BY-SA)

> *«The smallest organizational unit in the user interface is a panel. The panel header shows the
> title of the panel. It is always visible. Some panels also include subpanels.»*
> — `interface/window_system/tabs_panels.rst`

E do HIG, a razão de o sub-painel ganhar ao risco:

> *«When a label would help give context to multiple buttons, it often makes sense to organize
> them in a subpanel. The use of subpanels is generally preferred over a single label button in a
> row above a block of buttons.»* — `layouts.md`

⭐⭐⭐ **A fronteira de uma secção é a BORDA DE UM CORPO, não uma linha entre dois vizinhos.** Um
risco diz *«acabou»* e não diz *«do quê»*: com ele o espaço acima e o espaço abaixo não pertencem
a ninguém — e é por isso que a folga em volta se lia irregular **por mais que a apertássemos**
(waves 7 e 8 apertaram-na duas vezes e a queixa voltou). *Estávamos a afinar o número errado.*

#### ⭐⭐ O mecanismo, e porque ele não re-dispõe uma única linha

O nosso desenho é imediato: um pintor anda de cima para baixo e devolve o `y`. Um cartão tem de
ser pintado **por baixo** de um conteúdo cuja altura só se conhece **depois** — o que parece pedir
duas passagens, e duas passagens registariam o hit-index **duas vezes** (um defeito, não um custo).

A saída é **estacionar a cena**: o corpo pinta-se numa cena vazia, os cartões vão para a cena real,
e o corpo volta por cima com `Scene::append`. É lícito porque o `VectorScene` é um *newtype* de UM
campo sobre a cena do Vello (a troca é **sem perdas**) e porque um `append` herda a pilha de
recortes aberta — que é o que mantém a rolagem do painel a funcionar.

⭐ **E o cartão é um RECUO PARA FORA do bloco já pintado**, nunca uma caixa que empurra o conteúdo
para dentro: *nenhuma linha muda de sítio, logo nenhum gesto muda de alvo.* Foi isto que tornou a
conversão possível sem tocar na disposição de uma única secção — o sítio de chamada passa de
`y = separator(y, x, w, scene, theme)` para `y = cards.close(scene, x, w, y)`, a mesma forma.
*O risco já marcava o fim de uma secção; ele só não sabia dizer o princípio.*

**A escada de fundos**, que é o que diz «subsecção»: painel `panel-bg` (`#131313`) · cartão de
secção `bg-1` (`#1f1f1f`) · cartão de **subsecção** `bg-2` (`#292929`). Degraus de 12 e 10 em 255
— a medida que o dono aprovou em §7.7. O **título** de uma subsecção fica no cartão do pai e só o
**conteúdo** desce para o cartão claro: é literalmente o que ele descreveu.

⛔ **O tema CLÁSSICO não muda:** `PH2D_UI_NEW=0` continua a desenhar o risco, e a escolha vive
**dentro da porta** — nenhum painel ganhou um `if` de tema.

Porta: [`widget::section_cards`](../../../crates/ph2d-editor-core/src/widget/section_cards/mod.rs).
Gates: 4, e o que paga o mecanismo mede a **CENA** (*«o corpo estacionado volta inteiro»*) — ⚠️ se
o `append` deixasse cair o corpo, todo painel convertido ficaria **em branco** e nenhum gate de
geometria o veria, porque os `Rect` continuariam certos. Mutação: **morta**.

#### O que mais entrou

- ⭐ **A linha de lista deixou de parecer um botão.** Lei do Godot Modern para o `selected` de uma
  `Tree` (`theme_modern.cpp:709`): é o *flat pressed* com **`content_margin_all(0)`** — ele
  **SANGRA** de ponta a ponta do corpo, sem recuo e sem moldura. O nosso realce tinha raio de chip
  e a largura exacta do `Bypass` logo abaixo; hoje transborda a folga do cartão.
- ⭐ **As quinas.** A porta do raio já existia (`visuals::radius` → 4 px, o `corner_radius` do
  Godot Modern) e **25 pinturas passavam ao lado dela**, a 6 px. Passam agora. ⛔ Uma fica de
  fora, declarada: um *post-it* tem cor de marcador fixa e o pintor dele não recebe tema.

#### ⚠️ A mesma lição, pela segunda vez em duas waves: o censo linha-a-linha mente

O primeiro censo do raio contou **26** desvios. Refeito com uma varredura que atravessa linhas,
são **75** — a chamada que o dono apontou (o realce da corrente de efeitos) estava escrita em cinco
linhas e era **invisível** à primeira. *Um censo que lê o fonte tem de saber a forma do que lê* —
na wave 8 a forma escondida era uma **função**, aqui é uma **quebra de linha**.

⏳ **E os outros 50 não foram convertidos de propósito:** metade deles é **canvas**, não cromo — a
régua da timeline, as células do Flip, o gizmo 3D, as tiras de clip. O raio de um clip de timeline
é desenho do documento, não do painel, e achatá-lo com o resto seria o erro simétrico. A partição
cromo/canvas é uma wave própria, no molde do censo da moldura (§7.4–§7.8).

#### ⏳ O que fica, e é o resto da ordem dele

Está convertido **um** painel — o **Editor de Áudio**, que é o da foto. Os outros **23 riscos
azuis** (Inspector: 11 · Painter Layers: 9 · Vector: 2 · Grid Snap: 1) esperam a mesma conversão,
agora que a máquina está paga e provada. ⚠️ **A ordem é deliberada:** o mecanismo era o risco desta
wave, e prová-lo no painel que ele fotografou antes de tocar em oito crates é a ordem honesta.

### 7.13 — ✅ WAVE 10 (2026-09-06): o GRUPO — e a quina passa a ser o que separa

**Ordem do dono**, cinco pontos: *«espaços demais entre botões»* · *«raios das quinas ainda com
valores altos»* · *«a própria altura dos botões pode ser menos sem reduzir o tamanho da font»* ·
*«o nome do filtro continua a parecer um botão»* · e ⭐ *«uma coisa muito legal que o Blender tem:
se 2 ou mais botões estão lado a lado, só as bordas externas dos botões das extremidades recebem
arredondamento»*.

⭐⭐⭐ **O quinto ponto é a resposta ao primeiro.** Numa fileira do Blender as peças **encostam** —
o que separa duas peças de um mesmo controlo é a **QUINA**, não o espaço. Um vão entre elas diria
que são coisas diferentes, e elas não são: são uma escolha entre irmãos, que o HIG manda expandir
a toda a largura (`layouts.md`, «Mode toggling buttons»). *Estávamos a pôr folga onde o modelo põe
geometria.*

Porta: [`widget::GroupPos`](../../../crates/ph2d-editor-core/src/widget/button_surface.rs) +
`segment_rects`, com o primitivo `paint::fill_rounded_rect_radii` (o raio deixou de ser um número e
passou a ser quatro). A grelha EDIT inteira do editor de áudio passou por ela, e com isso
**morreram as larguras à mão** (`third`, `half`, `TOOL_COLS`) — a fileira agora é derivada.

| item | antes | agora | fonte |
|---|---|---|---|
| vão entre botões de um grupo | `Spacing::Xs` = 4 px | **1 px** (o traço da costura) | Blender, a foto dele |
| quinas de dentro de um grupo | 4 px | **0** | a lei que ele apontou |
| raio de cromo | 4 px | **3 px** | `editor_theme_manager.cpp:277` (o outro estilo do Godot) |
| altura de linha | 24 px | **22 px** | `density.compact`, que este repo já declarava |
| realce de linha de lista | raio 6, largura de botão | **sangra, raio 0** | `theme_modern.cpp:709` |

⚠️ **A altura é DERIVADA, não escolhida:** a fonte de um botão é `13 px` e a caixa do glifo `15`,
logo `22` deixa `3,5` px de folga de cada lado — que é o `base_margin · 0.75` do Godot. *O pedido
era «menos altura sem mexer na fonte», e o piso da fonte é o que responde.*

⚠️ **E a quina zero é o que finalmente separa a linha de lista do botão:** com a lei do grupo, um
botão desta casa arredonda **pelo menos um** canto; uma linha de lista não arredonda **nenhum**.
*É a mesma régua do Blender lida do outro lado — lá o que agrupa é a quina que fica, aqui o que
separa é a quina que não existe.* O sangramento sozinho (wave 9) não bastara.

#### ⚠️⚠️ Três reduções de fixtura na mesma corrida — e uma vacuidade que só a mutação viu

A descida de `24 → 22` px moveu três retratos escritos à mão, e o portão apanhou os três:

1. O retrato do dock (`motion.bezier_warp` `825 → 777`) — **o terceiro em três waves**.
2. Duas fixturas de **rolagem** da timeline deixaram de conter o fenómeno (`13` linhas já cabiam
   em `300` px). ⭐ **O gate era honesto e disse-o em voz alta** — *«a fixture nao contem o
   fenomeno»* — em vez de passar por vácuo. As duas passam a **procurar** o limiar em vez de o
   nomear.
3. ⭐⭐⭐ E o censo que verifica que os pintores deste painel **perguntam ao store** ficou verde
   sobre código partido: ao partir `button` em *delegador* + *implementação*, ensinei-o a aceitar
   a delegação — e a fatia a que ele chama «corpo» **começa na própria assinatura**, então
   `button_in_group` media-se a delegar **para si mesmo**. Apagar a pergunta ao store dos **dois**
   pintores deixava-o verde. *Um censo que procura uma CHAMADA encontra a DEFINIÇÃO, e o único
   instrumento que o diz é a mutação.*

⚠️ **E o `paint.rs` cruzou o tecto de 700 LOC** ao ganhar o raio por canto ⇒ cortado por
responsabilidade: a família do rectângulo mudou-se para `paint_rounded.rs` (608 + 118), **com a
prova dela** — *uma prova que fica na casa antiga mede um nome, não uma lei*.

### 7.14 — ✅ WAVE 11 (2026-09-06): a lei do grupo tinha DUAS dimensões, e eu aplicara uma

**Ordem do dono**, depois de ver a fileira agrupada: *«na horizontal ficou bom. Na vertical ainda
tem muito espaço ainda.»*

⭐⭐⭐ **Ele tem razão, e a leitura correcta é que a lei nunca foi horizontal.** No cartão
*Transform* do Blender que ele próprio fotografou, o `Location X / Y / Z` é uma **coluna** de
linhas que encostam, com arredondamento só no topo da primeira e no fundo da última. *Eu tinha
portado metade da lei — a metade que a foto dele mostrava de lado.*

**A generalização:** [`GroupCell`](../../../crates/ph2d-editor-core/src/widget/button_surface/group.rs)
`{ col, row }`, e **um canto só arredonda se estiver na borda das DUAS**. Um bloco de 3×2 botões
passa a ter **quatro** cantos, não doze — e o gate conta-os.

⚠️ **E a forma real é RAGGED, não rectangular:** a barra de ferramentas deste app é `3 · 3 · 2`, e
o próprio Blender empilha `Location X/Y/Z` (3 linhas de 1) com um `Mode` de uma peça só. Uma
grelha uniforme partiria o bloco em três — que é exactamente a folga que o dono estava a ver. ⇒
`block_cells(origin, &[3, 3, 2], row_h)`.

**O que virou um corpo só no editor de áudio:**

| bloco | fileiras | o que era |
|---|---|---|
| Transporte | `1 · 2 · 2 · 1` | Play · Stop\|Loop · Load\|Export · Batch LUFS |
| Barra de ferramentas | `3 · 3 · 2` | tools · clipboard · estrutura |
| Operações do clipe | `2 · 2 · 2 · 2 · 2` | Undo…Gain, mais Invert\|Force Mono |
| Operações de selecção | `2 · 2` | Trim\|Silence · Fade In\|Out |

⚠️ **E os avanços de linha DENTRO de um bloco morreram** — quem posiciona é o bloco, e a altura
total sai de `grid_height`. *Um `y += pitch` sobrevivente seria a folga a voltar por uma porta que
já não é a única.*

#### ⚠️ Três coisas que o portão apanhou, e todas são a mesma espécie

1. **`segment_rects` devolvia meia resposta** (`GroupPos`, só a coluna) — cada chamador teria de
   construir a célula à mão, e metade esquecer-se-ia da segunda dimensão. Hoje devolve a célula:
   *uma fileira solta é um bloco de UMA linha, e dizê-lo na porta é o que impede a próxima
   metade.*
2. **O `button_surface.rs` cruzou o tecto de 500 LOC** ⇒ cortado por responsabilidade em *cor*
   (`mod.rs`, 312) e *forma de grupo* (`group.rs`, 358).
3. ⭐⭐ **E o corte partiu DOIS censos que isentavam a porta pelo NOME DO FICHEIRO** — ela era
   `widget/button_surface.rs` e passou a `widget/button_surface/mod.rs`. *Um censo que aponta a um
   ficheiro mede o sítio, não a lei — e quem corta um ficheiro em dois não devia ter de saber que
   censo de outra pessoa aponta para ele.* **É a segunda vez que esta linha paga isto** (a
   primeira foi o cartão de asset, na wave 6): a isenção passa a ser pelo **módulo**.

### 7.3 — ⏳ O que a wave 1 NÃO fez (nomeado)

- ~~os outros ~38 pintores continuam a escolher fundo/borda sozinhos~~ ✅ **§7.4 + §7.5** — 24
  convertidos na wave 2, os **22** restantes na wave 3; `NOT_YET` está **vazio** e o gate impede
  um pintor novo de nascer sem a porta;
- o `panel-radius: 16` do `tokens.json` fica para o clássico; a docagem já usa `0`;
- `Spacing` não foi tocado (o `base_spacing 4` do Godot coincide com o `Xs`);
- a fonte (o Godot recomenda *Inter*; a casa tem `FONT_SANS`) — não medido.

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
