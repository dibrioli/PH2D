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

### 7.7 — ✅ 2026-09-05, os dois reports do smoke da wave 4: a porta do ASSETS e a ESCADA de superfícies

**(a) *«não há meio de abrir assets»*** — a porta do navegador de Assets era **só** o chip
`TOPBAR_RIGHT_ASSETS` do grupo direito da barra legada (Layers · Assets · Script), que o redesenho
não pinta; o censo de alcance da wave 1 (`the_bar_relocated_every_row_of_the_menus_it_replaced`)
só percorria os **menus** que a barra substituiu, e um chip que despachava sozinho não estava em
lista nenhuma — a família do *Export SVG*, um nível ao lado. Cura: a linha **Assets** no menu
*Window* (o mesmo id; o handler continua no `ph2d-panel-asset-browser`), a lista
`LEGACY_PILL_BUTTONS` e a segunda metade do gate (red-first: acusou *«Assets (botão directo da barra
legada)»* antes de a linha existir). ⚠️ **A nota que o escondia tinha envelhecido ao contrário**: o
`NO_DOOR_PENDING` dizia *«sem consumidor»* sobre um id cujo consumidor nasceu com o painel — a
metade de obsolescência do gate acusou-a no dia em que a linha entrou. ⛔ `Layers` e `Script` ficam
fora **com o motivo**: nenhum `apply_event` do app os trata (eram chips mudos também no clássico), e
uma linha de menu para um id sem handler é uma linha morta.

**(b) *«o fundo dos cards tem tão pouco contraste com o fundo dos painéis»*** — medido: **4/255**
no `Dark` (`Bg1` derivava para `dark_3`, `PanelBg` para `dark_1`). ⚠️ **A wave 1 portou as REGRAS
do Godot sem a PILHA de superfícies dele**: no `theme_modern.cpp` (191-209) o `PanelContainer` é a
**`base`** e as superfícies acima dela saem da `_get_base_color(ofs, sat)` — `surface_high` −1.3
(a faixa de um bus de áudio, o «cartão»), `button_normal` −2.0, `button_hover` −2.9,
`button_pressed` −3.2; abaixo, `surface_lower` +1.1 (o `LineEdit`) e `surface_lowest` +1.7 (o fundo
do `GraphEdit`, das abas). Nós tínhamos o painel em `dark_1` e o botão na `base` — a pilha
invertida, com os cartões entalados a 4/255 do painel. A escada reassentou-se, e os números são os
medidos pela sonda:

| tema | canvas (`Bg0`) | painel | Bg1 (cartão) | Bg2 | Bg3 | BgElev | texto-2 sobre o cartão | acento sobre o cartão |
|---|---|---|---|---|---|---|---|---|
| Dark | `#1b1b1b` | `#292929` | `#393939` (+16) | `#424242` | `#4d4d4d` | `#505050` | `#b4b4b4` — 5,57:1 | `#569eff` — 4,25:1 (intacto) |
| Gray | `#282828` | `#3d3d3d` | `#555555` (+24) | `#626262` | `#727272` | `#787878` | `#c9c9c9` — 4,50:1 | `#70bafa` — 3,58:1 (intacto) |
| Light | `#f5f5f5` | `#e6e6e6` | `#d4d4d4` (−18) | `#cacaca` | `#bebebe` | `#b9b9b9` | `#505050` — 5,44:1 | `#2973e6` — 3,02:1 (⚠️ escurecido) |
| OLED | `#000000` | `#000000` | `#000000` | `#000000` | `#1b1b1b` | `#262626` | `#a6a6a6` | `#73bfff` |

⚠️ **O CANVAS não é a escada.** A 1.ª versão desta correcção levou o `Bg0` (o fundo do canvas e do
grafo) para o `surface_lowest` do Godot (`#141414`), e o dono devolveu-o no smoke seguinte
(*«mudou a cor do canvas — volte ao que era antes»*): o `Bg0` fica no `dark_1` (`#1b1b1b`) que ele
já tinha aprovado. A escada começa no painel. E o **piso do texto secundário é `0.65`**, não o
`0.55` do Godot (*«as fontes dos cards podem ser um pouco mais claras»*): títulos e rótulos dos
cartões são todos `Text2`; o `Text1` fica em `0.75` para a hierarquia continuar a ler-se.

Gates: `a_card_stands_off_its_panel_and_the_surface_ladder_climbs` (red-first: *«4 de 255»*; a
escada inteira monótona, o degrau do cartão ≥ 12, os outros ≥ 3; o OLED fica de fora — com
contraste 0 e base preta a família colapsa, e são as *Extra Borders* que separam, como no Godot) e
o WCAG da casa (`the_factory_table_meets_wcag_in_all_modes`), que **reprovou a primeira versão**.

**O que a construção ensinou:**

- ⭐⭐ **Dois números que o Godot escreve e a lei da casa DERIVA.** O texto secundário do Godot é
  `mono@0.55` e o acento vem por inteiro do preset — e nenhum cumpre a WCAG sobre o cartão em todos
  os presets: no *Gray* a base `0.24` eleva o cartão mais (a família é multiplicativa) e o `0.55`
  dá 3,5:1; o azul do *Light* faz 2,5:1 sobre o cartão e **2,99:1 sobre o próprio painel do Godot**.
  A resposta do §0 é medir: a alfa do texto-2 sobe do piso (`0.65`, pedido do dono) só até 4,5:1
  sobre o `Bg1` (Dark `0.65` · Light `0.65` · Gray `0.72`), e o acento escurece 2 % de cada vez
  só até 3:1 — só o *Light* se move (`#2e80ff → #2973e6`); o `#569eff` do Dark tem gate a mantê-lo.
- ⚠️ **O OLED colapsa a família multiplicativa numa cor só** (`0 × k = 0`): o Godot separa os
  estados por bordas e pela cor da fonte; aqui o hover e o pressionado caem num degrau **aditivo**
  em `mono` quando o multiplicativo não os move (`the_three_interactive_states_are_distinguishable`
  reprovou a primeira versão: *«Oled repouso = hover»*).
- ⚠️ **Um apelido que REPETE a fórmula deixa de ser apelido no dia em que a fórmula muda**: os
  `timeline-row-alt`/`ruler-bg` diziam `base` onde o `bg-2` dizia `base` — e o gate dos 16 aliases
  apanhou-os quando o `bg-2` passou a `button_normal`. Hoje delegam.
- ⏳ **Para o smoke**: as linhas alternadas da timeline (`RowAlt = Bg2`) ficam a +25 do painel no
  Dark — a lei dos aliases é do Enio, e a força do zebrado é dele julgar.

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
