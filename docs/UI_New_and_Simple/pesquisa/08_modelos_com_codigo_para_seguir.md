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
| **G3 — uma LISTA encosta sobre um FIO** | `Tree.v_separation = pow(EDSCALE_RND(base_margin · 0.175), 3)` ⚠️ **corrigido na wave 17**: o `EDSCALE_RND` arredonda `0,7` para `1` ANTES do cubo — esta linha dizia `0,343 ⇒ 0` por truncar. E há um **segundo ramo**: com `enable_touch_optimizations` o Godot dá `separation_margin · 0,9` = **3** | **1** |
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

### 7.15 — ✅ WAVE 12 (2026-09-06): os 23 riscos azuis acabaram — o app inteiro usa cartões

**Ordem do dono:** *«Excelente! Continue!»*, sobre a fila que a wave 9 deixou nomeada.

**Convertidos:** Inspector (18) · Painter Layers (5) · Vector (1) · a **galeria de widgets** (1) —
que é a fonte de verdade do cromo (DIRETRIZ §5.2), logo um risco ali seria a galeria a ensinar o
que o app já não faz. Com o editor de áudio da wave 9, **zero riscos azuis no produto**.

#### ⭐⭐ A porta mudou de forma três vezes, e cada mudança foi cobrada por uma medição

1. **O livro deixou de viajar na assinatura.** Enfiá-lo por todos os pintores de secção custava
   **~20 assinaturas só no Inspector**, para responder vinte vezes a uma pergunta que o quadro
   responde uma vez. ⇒ um `thread_local`, que é **o padrão desta casa para estado de pintura
   por-quadro** e tem a razão escrita no [`published.rs`](../../../crates/ph2d-editor-core/src/published.rs):
   *a shell publica, a folha lê*.
2. **Ganhou um par `begin`/`end`** além do fecho, porque **nem todo corpo cabe numa closure**: o do
   Inspector é um troço linear de ~230 linhas entre o `open_body` e o `close_body`, com dezenas de
   locais vivos. É a mesma forma do `push_clip`/`pop_layer` que esta casa já usa.
3. **E o `begin` deixou de devolver um guarda.** Ele era `#[must_use]` e o chamador carregava-o até
   ao fecho — o que custou **UMA linha** no destructuring do `paint_inspector`, que estava a `263`
   de uma catraca cuja regra é *«as folgas encolhem; elas nunca crescem»*. ⇒ a cena estacionada
   mudou-se para a **pilha, ao lado do livro**, e o guarda desapareceu. ⚠️ Antes disso, o custo já
   tinha sido pago com um corte: o par mudou-se para dentro do `open_body`/`close_body`, *porque
   abrir e fechar um corpo nunca foi orquestração de secção* — é a mesma frase que aquela folga já
   tinha escrito, duas vezes, sobre outras extracções.

#### ⚠️ E o livro é uma PILHA, não um slot — provado por uma mutação que SOBREVIVEU primeiro

Um corpo pode abrir dentro de outro. Com um slot único, o de dentro apagaria o de fora ao sair — e
o de fora perderia os cartões **e o conteúdo**, porque a cena estacionada mora ao lado do livro.

⚠️⚠️ **A primeira redacção do gate não via isso.** Ela media a cena no FIM: com o livro perdido, o
`close_section` do corpo externo cai no braço clássico, **desenha um risco** (a contagem final
continua alta) e devolve um `y` que também avança. *Duas coisas diferentes que dão o mesmo número
no fim.* ⇒ a régua passou a medir o **instante**: um cartão não pinta nada quando fecha — ele é
recolhido, e só chega à cena no `end`; um risco pinta-se ali e já. Se a contagem de caminhos sobe
**naquela linha**, o livro perdeu-se.

#### ⭐ O censo que impede o 24.º risco

[`the_boundary_of_a_section_is_a_card`](../../../crates/ph2d-editor-core/tests/the_boundary_of_a_section_is_a_card.rs),
com **duas** metades — e a segunda é a que o torna honesto:

- nenhum painel nomeia `paint_section_separator`;
- ⚠️ **e o braço CLÁSSICO tem de continuar a nomeá-lo**, senão `PH2D_UI_NEW=0` fica sem fronteira
  nenhuma — um defeito de produto que nenhum outro gate desta linha vê. *Um censo de ausência
  precisa de uma testemunha de presença.*

⚠️⚠️ **E essa testemunha nasceu VÁCUA:** ela fazia `contains` sobre o ficheiro inteiro, e o doc do
próprio módulo **cita a chamada antiga em prosa** — apagar o braço clássico do código deixava-a
verde. A mutação apanhou-o. *Um censo de presença que lê comentários testemunha a documentação,
não o produto.* (É a terceira vacuidade desta espécie em três waves: a assinatura dentro do corpo,
o nome de ficheiro numa isenção, e agora a prosa de um doc.)

### 7.16 — ✅ WAVE 13 (2026-09-06): o cartão do Painter não sumiu — foi ENGOLIDO

**Report do dono**, com as duas telas lado a lado: *«em Audio Editor: Effects temos o card. Já o
card de Painter: Jitter não se vê mais.»*

⭐⭐⭐ **A palavra dele — «não se vê MAIS» — era o diagnóstico.** O Painter já pintava um cartão
próprio para o *Jitter* (e mais onze, via `card.rs::card_frame`) em **`Bg1`**, e a wave 12 pôs um
cartão de **SECÇÃO** por trás dele, também `Bg1`. *Dois `Bg1` encostados são um só.*

⇒ a cura é a lei que ele próprio descreveu na wave 9 e que existia com **zero chamadores**:
*«uma subsecção está com o seu título dentro do card da secção mas o seu conteúdo fica dentro de
outro card/container de COR DIFERENTE»*. O `card_frame` pede o tom à porta
(`CardDepth::Subsection.token()` = `Bg2`) em vez de escolher um. **12 cartões internos** mudam com
uma linha.

#### ⚠️⚠️ E o caminho até à causa tem três achados maiores que o defeito

**1 — `Theme::default()` é `Forge`, que é CLÁSSICO ⇒ nenhum gate de painel deste repo media a
família MODERNA.** A primeira sonda que escrevi devolveu **exactamente a mesma contagem** com e sem
os cartões (1 821 nos dois) — porque no clássico não há cartão nenhum para contar. *Uma sonda no
tema errado mede outro programa.* ⇒ o arnês ganhou [`MockPanelHost::in_theme`], e o redesenho
passou a ser observável de um teste de painel pela primeira vez. **No moderno a diferença é real:
1 591 contra 1 573.**

**2 — «há cartão» e «há UM cartão que engole tudo» dão o mesmo número.** A cena crescia, o
`close_section` era chamado, todos os gates estavam verdes. O que faltava era **contar**: o
`end_section_cards` passa a devolver quantos pintou, e a medição deu **9 cartões** no corpo do
pincel — com **296 px** e **609 px** de altura. *Um cartão que preenche o corpo inteiro não tem
borda visível.*

**3 — o que nenhuma régua desta linha media era o CONTRASTE entre profundidades vizinhas.** O gate
novo (`two_nested_depths_never_paint_the_same_tone`) fecha isso, e nasceu errado **duas** vezes:
- exigia que a escada **SUBISSE**, e reprovou o `Light` sobre uma escada correcta — ali as
  superfícies **escurecem** ao aninhar. *Uma lei escrita na polaridade de um tema é uma lei sobre
  aquele tema;* o que é invariante é a **monotonia**.
- e a barra que eu ia herdar (os `12/255` aprovados em §7.7) é de **outro par** — aquele é *cartão
  contra painel*, este é *subsecção contra secção*, e a escada dá-lhe **10**. ⇒ o gate defende o
  que se prova sem o dono (as três superfícies são TRÊS) e **a pergunta dos 10 px vai ao smoke**.

⛔ **E o OLED lê `0, 0, 0` nas três superfícies** — no preto puro nenhum cartão do app é visível.
A exceção é **herdada** do gate irmão `a_card_stands_off_its_panel`, que já a media e nomeava a
cura do Godot (*Draw Extra Borders*); **não é regressão desta wave**.

### 7.17 — ✅ WAVE 14 (2026-09-06): o agrupamento chega a 101 painéis por UMA porta

Depois do *smoke OK*, a fila dizia *«levar o agrupamento de botões aos outros painéis»* — e o censo
mudou o plano: converter as fileiras à mão dava **12** sítios, mas este app já tem o widget que
**significa** «escolha entre irmãos» — o `SegmentedAdaptive`, com **101 consumidores**. ⇒ a lei
entra na porta, não nos sítios.

**O que mudou, num sítio cada:**

| | antes | agora |
|---|---|---|
| `segmented_gap()` | `Spacing::Xs` = 4 px | **`SEGMENT_HAIRLINE`** = 1 px |
| cada segmento | quatro cantos | **`GroupCell`** — só as bordas de fora |
| um grupo que QUEBRA de linha | fileiras separadas por 4 px | **um bloco**: as fileiras encostam e só os quatro cantos do BLOCO arredondam |

⭐ **Um grupo que quebra continua a ser UM corpo.** Sem isso, uma escolha entre irmãos passaria a
ler-se como dois controlos só por ter mudado de linha — que é o contrário do que a lei diz.

#### ⚠️ E o defeito que quase entrou estava PREVISTO no doc do próprio ficheiro

O `segmented_row_counts` já avisava: *«um contentor medido por uma regra e preenchido por outra é
como a secção seguinte pinta por cima destes botões e lhes mata o alvo»* — com o gate que o provou
citado ao lado. Pois: o **pintor** passou a empilhar as fileiras com o traço de 1 px e o
**medidor de altura** continuava a somar `Spacing::Xs` por fileira. **4 px de dívida por quebra de
linha** — invisível num grupo de uma fileira, cumulativo nos outros.

⚠️ **Nenhum gate desta crate o via**, porque todos conferiam UMA das duas respostas. O novo compara
as **duas** (`the_measured_height_is_the_painted_height`), e ambas passam a sair da mesma porta
(`grid_height`). *Um aviso escrito no doc não é um gate — e este ficheiro tinha o aviso há meses.*

⛔ **O `paint_segmented_button` mantém a assinatura antiga** (delega, pintando `GroupCell::Only`):
há **15** chamadores diretos que montam as próprias fileiras — Vector, Motion Params, Grid Snap,
Flip. Convertê-los é a wave seguinte; quebrar-lhes a assinatura agora seria pagar 15 edições antes
de saber se o dono aprova o resultado nos 101.

### 7.18 — ✅ WAVE 15 (2026-09-06): a palavra que não cabe ELIDE, e as grelhas à mão acabaram

**Dois reports do dono na mesma mensagem.**

#### (a) *«Quando a palavra é grande e estreitamos o painel, em vez dos três pontos (…) como no Blender, a palavra passa para baixo e some»*

Duas fotos do mesmo painel a estreitar: `Surface Smooth` ficava `Surface`. **Mecanismo:** o
`paint_text` recebia `max_width` como **orçamento de QUEBRA** (o doc dele dizia-o) e o parley
obedecia — o rótulo virava duas linhas, a altura dobrava, e a segunda caía fora da linha de 22 px.
**Cortada, não elidida.**

⚠️ **A porta da elisão já existia** (`text_elide`, 55 consumidores) e os outros **~269** sítios não
passavam por ela. *Uma porta que a maioria não chama ainda não é a lei.*

A cura é um **argumento**, não um comportamento: `Lines::{ElideToOne, Wrap}`. O `paint_text`
declara uma linha; o `paint_text_block` declara quebra — e é ele que **devolve a altura**, que é o
que distingue os dois casos.

⚠️⚠️ **E eu parti o caso oposto DUAS vezes na mesma jornada:** primeiro pondo a elisão no caminho
**partilhado** (o `paint_text_block` delega lá), depois deixando o próprio `paint_text_block` a
pedir `ElideToOne`. Nos dois casos caíram **três** gates de outras linhas
(`a_hint_that_wraps_pushes_what_comes_after_it_down` e dois irmãos): uma dica que deixa de quebrar
deixa de **empurrar**, e a fileira seguinte é escrita por cima dela. *Uma cura que só sabe o caso
que a motivou apaga o caso oposto.*

⚠️ **E o gate nasceu VÁCUO:** ele afirmava sobre `text_elide::fit(...)` — e a mutação que apagava a
elisão de **dentro do pintor** sobreviveu, porque a porta continuava a elidir quando o teste lhe
perguntava. *Um gate que interroga a porta testemunha a porta, não o pintor.* A régua passou a ser
a **contagem de glifos** que a cena recebeu.

#### (b) *«Em vários lugares não funciona. Veja: Vector: tools»*

Os **15** chamadores que a wave 14 deixou nomeados — eles montam as próprias fileiras e por isso
não viam a lei que entrou no widget. Convertidos: a `button_grid` do **Vector** (a foto dele), as
**quatro** formas do **Grid Snap** (grelha de 3 colunas · coluna de largura inteira · par ·
fileira) e as **quatro** do **Motion Params**.

⭐ **Uma COLUNA também é um bloco:** a pilha vertical do Grid Snap é `n` fileiras de uma peça, e só
o topo da primeira e o fundo da última arredondam. A porta já o exprimia sem código novo.

⚠️ **E cada conversão trocou uma conta de altura à mão pela porta** (`grid_height`) — a conta
antiga somava o vão de 4 px que já não existe, e *um contentor medido por uma regra e preenchido
por outra escreve a fileira seguinte por cima desta*.

⛔ **O invólucro «peça sozinha» do Grid Snap foi APAGADO** — depois de as quatro formas declararem
a posição, ele ficou com zero chamadores. *Um invólucro sem chamador é lixo que a próxima pessoa lê
como se fosse a porta.*

### 7.19 — ✅ WAVE 16 (2026-09-06): o rótulo ganha RESPIRO — e ele só se paga no caso apertado

**Report do dono**, com duas fotos: *«algumas palavras ou mesmo os 3 pontos ficam muito próximos da
borda do botão. Deveria ter algum espaço.»* — `Surface Smo…` acabava colado à moldura, e o `Connect`
/ `Chamfer` / `Bucket` do Vector encostavam nas duas.

**Causa:** a elisão da wave 15 mede contra a largura **inteira** da caixa, então ela corta
exactamente onde a moldura começa. O texto passou a caber — e a caber *até ao fim*.

⭐⭐ **A cura não custa nada no caso comum, e a razão é geométrica:** a centragem é simétrica, logo
deflacionar os **dois** lados não move um rótulo que cabe. O recuo muda **só** o orçamento com que
o rótulo é elidido. *Um respiro que só se paga no caso apertado é o único que não se nota no
outro.*

**O número é do modelo:** `base_margin · 2` = **8 px**, a margem de conteúdo horizontal de um botão
no Godot 4.6 «Modern» (`theme_modern.cpp:289`) — e é o `Spacing::Md`, que a escada desta casa já
nomeia *«default inline padding»*. ⛔ **Não** é o `Button::padding()` (12 px): aquele é o recuo de
um botão que **dimensiona a si próprio**, e aqui a largura vem de fora.

Alcance: `paint_text_centered`, **79 sítios**.

#### ⚠️⚠️ E o gate nasceu vácuo pela TERCEIRA vez nesta jornada

A 1.ª redacção calculava `fit(...)` com o orçamento já recuado e comparava com ele próprio — ela
passaria com o recuo **removido do pintor**. As outras duas foram a elisão (que interrogava a porta)
e o censo do tom (que lia um comentário). *A diferença é sempre a mesma e sempre a mesma correcção:
**chamar o que o produto chama** e medir o que ele emitiu.* Aqui: `paint_text_centered` numa cena,
e contar glifos.

⛔ E a metade de baixo mede que um rótulo **que cabe** sai igual em duas larguras diferentes — sem
ela, um recuo que encolhesse os rótulos curtos passaria.

### 7.20 — ✅ WAVE 17 (2026-09-06): uma LISTA não é um formulário, e a linha mais alta do app desceu

**Vem do §8 do handoff, itens 2 e 7** — os dois eram a mesma frase: *a hierarquia é a linha mais
alta do aplicativo (32 px contra 22), e duas superfícies de LISTA não seguem a lei do Godot.*

#### O censo: cinco listas, QUATRO respostas

| superfície | altura | vão escrito |
|---|---|---|
| hierarquia | `HIER_ROW_H` = **32** (token só dela) | `Spacing::Xxs` = 2 |
| variações do áudio | `VAR_ROW_H` = 22 **à mão** | `Spacing::Xs` = 4 |
| inspector da grade | `ROW_H` = 22 **à mão** | `ROW_GAP` = 2 (const local) |
| âncoras do Inspector | `ROW_H` = 22 **à mão** | **0** (implícito) |
| animações do Inspector | `ROW_H` = 22 **à mão** | **0** (implícito) |

⚠️⚠️ **A lição está nas duas últimas: a resposta certa já estava escrita DUAS vezes e não era
alcançável.** Elas avançam `cur_y += ROW_H`, sem termo nenhum — e como isso é a **ausência** de uma
soma, nenhuma varredura por operador a vê, nenhum doc a afirma e ninguém a podia copiar. *Uma lei
escrita em dois sítios ainda não é uma lei; só uma PORTA é.* (É a terceira vez que esta linha paga
esta frase: o `row_pitch_px` da wave 8 e o `stroke_uniform` do Vector foram as outras duas.)

⚠️ **E as quatro alturas «22 à mão» eram uma COINCIDÊNCIA, não uma derivação:** elas batiam com o
`chrome.row-h` de hoje e não seguiriam o próximo pedido de «mais compacto». As quatro passam a
derivar, e as quatro **saem** da lista de isenções do `the_row_height_is_one_number` — que as
acusou sozinho, pela metade de obsolescência dele.

#### ⛔ O número do modelo NÃO era o que eu tinha registado

```cpp
// theme_modern.cpp:650
int tree_v_sep = enable_touch_optimizations
    ? (separation_margin * 0.9)                       // 4 * 0.9 = 3.6 -> int 3
    : Math::pow(EDSCALE_RND(base_margin * 0.175), 3); // round(0.7) = 1 -> 1^3 = 1
```

A wave 8 registou *«`= 0`»* truncando `0,7³ = 0,343`. O `EDSCALE_RND` **arredonda primeiro**, e o
cubo é de `1`. ⇒ **as linhas de uma lista encostam sobre um FIO de 1 px** — o mesmo
`SEGMENT_HAIRLINE` de que uma peça de grupo é feita, que é a lei da wave 10 virada na vertical.
⛔ **Não é zero:** dois itens seleccionados em seguida têm de continuar a ler-se como dois.
*Uma derivação copiada sem se avaliar a expressão inteira é um número escolhido com cara de lei.*

⏳ **E ela tem um SEGUNDO ramo, que fica nomeado e por construir:** com `enable_touch_optimizations`
o modelo dá `separation_margin · 0,9` = **3 px**. O alvo desta casa é tablet, e o próprio Godot dá
à lista mais ar quando o dedo é o ponteiro — ⛔ ligar isso exige o interruptor que não existe, e é
decisão do dono.

#### A linha da hierarquia: medida antes de descer

O conteúdo é o galo (`Lg` = 12) e quatro ícones de `Xl` = **16 px**. Em 22 sobram **3 px** acima e
abaixo do mais alto — o mesmo ar que a linha de formulário dá ao controlo dela. ⇒ o token
`chrome.hier-row-h` **morreu**: um número que existe para ser diferente de outro é a segunda
resposta a *«que altura tem uma linha?»*. O passo da hierarquia vai de **34 px para 23** (−32 %).

#### ⭐⭐ O defeito que esta wave CRIARIA se ficasse pela metade

Os quatro companheiros de uma linha (galo, olho, grupo, cadeado) inflavam o rectângulo do ícone com
folga própria — `16 + 2·4 = 24`, `12 + 12 = 24`. Isso cabia numa linha de 32 e **transborda** numa
de 22, e no `HitIndex` **quem regista depois ganha** ⇒ a fatia de baixo da linha `N−1` passaria a
comandar o olho da linha `N`. ⚠️ **Nenhum gate de registo o veria** — o companheiro está vivo,
registado e alcançável; só está grande demais. ⇒ `row_tall`: a folga fica na horizontal e a vertical
é a da linha, que é o que o `Tree` do Godot faz (a célula de um botão **é** a linha). Isso tirou 16
LOC ao `paint_hierarchy_row` e **a catraca de LOC desceu de 248 para 232**, sozinha.

#### O que os gates apanharam

- ⭐ **O censo achou um sítio que eu não tinha visto, e era de OUTRA pergunta:** o
  `paint_variation.rs` fechava a **mesma** lista com `Sm` (6) no braço vazio e `Xs` (4) no cheio —
  duas respostas a *«quanto ar fica DEPOIS desta lista?»* na mesma função. A régua passou a separar
  o **avanço** (`+=`) da **saída** (`return`), e as duas saídas passaram a dizer o mesmo, para que a
  wave que decidir o fim-de-grupo (item 4 do handoff) mexa num sítio. *Uma régua larga demais ainda
  acusa verdades — só não é sobre elas que ela fala.*
- ⛔ **Os dois PISOS do gate medido são load-bearing:** a fixtura do painel tem **uma** linha
  («Scene Root»), então sem entregar uma cena viva toda lei sobre o que acontece *entre* duas
  linhas seria verdadeira por vacuidade. O arnês ganhou `MockPanelHost::set_hierarchy_rows` —
  método NOMEADO, nunca um `store_mut()`, no molde do `set_panel_scroll`.

| prova de mutação | resultado |
|---|---|
| o vão da hierarquia volta a `Xxs` | ✅ morreu (passo lê 24, a lei dá 23) |
| a linha volta a 32 px | ✅ morreu |
| o alvo do olho volta a inflar | ✅ morreu (sai da linha) |
| a lista de áudio escolhe o próprio vão | ✅ morreu (censo) |
| a lista de âncoras volta ao literal `22.0` | ✅ morreu (censo de altura) |

**Portão:** `13 060` testes / `0` falhados (a única vermelha da corrida foi a
`ph2d-timeline::nesting_clock::the_cost_of_depth_is_linear_not_explosive`, membro confirmado da
família de flakes de recurso do CLAUDE.md §5.0 — **3 de 3 verde sozinha**, com `load 50` impresso ao
lado, e zero linhas do diff naquela crate).

### 7.21 — ✅ WAVE 18 (2026-09-06): as LISTRAS — o tom que separa duas linhas encostadas

**Report do dono**, com a foto do *Outliner* do Blender: *«Outline do Blender tem algo interessante
que ainda não temos: linhas pares e ímpares têm tonalidade discretamente diferente».*

⭐ **É o companheiro exacto da wave 17.** Quando as linhas de uma lista encostam (vão de 1 px), o
que devolve *onde acaba uma e começa a outra* deixa de ser o vão — e passa a ser o TOM. Sem a
listra, a wave 17 teria trocado um problema por outro.

#### ⛔ O número não podia vir do Blender

O código dele é **GPL** e esta linha **não o lê** (a lei da triagem, `pesquisa/02`); o manual dele
(CC-BY-SA) documenta o painel de temas **sem enumerar os valores**. ⇒ o número vem do **egui**
(Apache-2.0 OR MIT, já vendorizado), que tem exactamente este slot com exactamente este propósito:

```rust
// crates/egui/src/style.rs:1512 e :1576
faint_bg_color: Color32::from_additive_luminance(5), // visible, but barely so
```

⭐ **E é o MESMO nos dois modos dele** (claro e escuro) — o que se porta é a **magnitude**, `5` em
255, nunca uma cor.

#### A direcção é DERIVADA, e é isso que a faz servir os oito temas

Somar sempre seria a lei do egui à letra, e parte-se num tema claro: um fundo a `250` sobe para
`255` e encosta no tecto. ⇒ a régua é **afastar-se do extremo mais próximo**, medida pela
`Color::relative_luminance`, que é a régua de claro-escuro que esta casa já usa nos testes de
contraste. Num tema escuro a linha ímpar **clareia**; num claro **escurece**.

⭐⭐ **E isso resolve de graça os dois temas em que uma lei fixa falharia:** o **Light** do Godot,
em que a elevação escurece por decisão dele, e o **OLED**, em que o contraste é `0` e não há para
onde escurecer — ali só há uma direcção possível, e a régua escolhe-a sozinha. *É o item 6 do §8 do
handoff a ficar meio resolvido sem ninguém lhe tocar: no OLED as listras vêem-se.*

⛔ **Não é o degrau da ESCADA de cartões** (12 e 10 em 255): esse diz *«isto está dentro daquilo»*.
Um degrau desse tamanho entre duas linhas irmãs leria como **aninhamento** — que é precisamente a
confusão que a palavra do dono (*discretamente*) exclui. Há gate: a listra anda **no máximo metade**
do degrau de cartão.

#### ⚠️ A listra é da LISTA, não da LINHA

Uma linha não sabe onde está: o que ela sabe dizer sobre si é o **estado** (apontada, seleccionada,
silenciada). A alternância é uma propriedade da **sequência**, e só quem itera a conhece. ⇒ a porta
(`widget::list_rows::paint_row_stripe`) é chamada pelo laço, e o estado é pintado por cima depois —
que é também a única ordem em que *seleccionado* se lê igual nas linhas pares e ímpares.

#### ⛔⛔ O índice é o VISUAL, e é aí que estava o defeito difícil

Uma hierarquia **salta** linhas: um ramo recolhido, um filtro de busca activo. Contar pelo índice do
**dado** poria duas linhas do mesmo tom encostadas exactamente quando o artista fecha um ramo — um
defeito **intermitente**, que se reporta como *«às vezes as listras somem»* e não se reproduz.

⭐ **A fixtura do gate é construída para separar as duas leis:** oito linhas com nomes alternados
`keep`/`drop`; filtrar por `keep` deixa visíveis os índices de dado `0,2,4,6` (**todos pares**) e
por `drop` deixa `1,3,5,7` (**todos ímpares**). Pela lei visual os dois casos pintam **duas**
listras — a mesma geometria; pelo índice do dado, um pinta **zero** e o outro **quatro**. ⇒ a
igualdade É a afirmação.

⚠️ **E ela é cega a uma coisa, o que exigiu um segundo gate:** se ninguém pintasse listra nenhuma,
«par» e «ímpar» emitiriam a mesma geometria e a igualdade passaria. *Uma igualdade prova qual é a
lei, nunca que a lei corre.* ⇒ o irmão mede o **custo de acrescentar uma linha**: com linhas
idênticas os saltos têm de **alternar** entre dois valores, e o maior é o que traz a listra.

#### Onde ela vai, e as duas que ficam de fora COM MOTIVO

| lista | listra | porquê |
|---|---|---|
| hierarquia | ✅ | é a que o dono fotografou |
| âncoras do Inspector | ✅ | |
| animações do Inspector | ✅ | |
| variações do áudio | ⛔ | **toda** linha já enche o próprio fundo (`Bg3`, ou `Accent` na seleccionada): uma listra por baixo de um fundo opaco não se vê |
| inspector da grade | ⛔ | é um **bloco de leitura** rótulo/valor de 5 linhas fixas, não uma lista de itens escolhíveis |

#### O que os gates apanharam — e um deles é uma lei minha

- ⭐⭐ **`nothing_inside_a_section_wears_the_section_tone` (wave 13) apanhou-me a escrever
  `ColorToken::Bg1` à mão** nas duas listas do Inspector. A cura é a lei: **o tom vem da PORTA**
  (`CardDepth::Section.token()`). ⛔ E é a `Section`, não a `Subsection`: a listra não é uma
  superfície nova dentro do cartão — é o próprio fundo do cartão movido 5/255.
- ⚠️ **O `#[allow]` mudou de dono.** A extracção que pagou o tecto de LOC inseriu a função nova
  **entre** o `#[allow(clippy::too_many_arguments)]` e o `fn paint_hierarchy_body` a que ele
  pertencia: o atributo passou a ser meu e o dono ficou sem ele. Só o *duplicado* é que o clippy
  viu — *a metade silenciosa é sempre a outra*.
- **Catraca de LOC paga por corte, nunca por tolerância:** `paint_hierarchy_body` `352 → 296` (as
  **linhas de parentesco**, 80 linhas, saíram para `paint_parentage_lines`). É o terceiro corte
  daquele ficheiro pela mesma lei.
- Mais três gates de arquitectura que um módulo de widget novo acorda: o bloco `mod` é **codegen**
  (o doc-comment não pode viver lá dentro), a galeria exige secção ou isenção escrita, e o HR-12
  exige a11y ou isenção. As duas isenções estão escritas com o mecanismo: *uma listra não regista
  alvo nenhum, e um leitor de ecrã não lê tons.*

| prova de mutação | resultado |
|---|---|
| a listra passa a seguir o índice do DADO | ✅ morreu |
| a hierarquia deixa de pintar listra | ✅ morreu |
| TODA linha ganha listra | ✅ morreu |
| a direcção fica fixa (só soma) | ✅ morreu no tema claro |

**Portão:** `13 066` testes / `0` falhados; clippy `--all-targets -D warnings` limpo.

### 7.22 — ✅ WAVE 19 (2026-09-07): a cauda de um BLOCO, e o token cujo nome mentia

**Fecha os itens 3 e 4 do §8 do handoff** — e eles eram a mesma doença vista de dois lados.

#### A escada tem TRÊS degraus, e só os das pontas tinham nome

| pergunta | porta | valor | derivação (Godot Modern, MIT) |
|---|---|---|---|
| de uma LINHA para a seguinte | `row_gap_px` / `list_row_gap_px` | 4 / 1 | `separation_margin` · `Tree.v_separation` |
| de um BLOCO para o seguinte | **`block_gap_px`** (nova) | **6** | `base_margin · 1,5` — um degrau que o tema usa **13** vezes |
| de um CARTÃO DE SECÇÃO para o seguinte | **`section_gap_px`** (nova) | 8 | `Separator separation = base_margin · 2` (`:885`) |

⛔ **Censo: 78 sítios respondiam à cauda de um bloco, com QUATRO respostas** — `Sm` (6) em 40 ·
`Xs` (4) em 29 · `Md` (8) em 8 · `Lg` (12) em 1.

⭐⭐ **A resposta maioritária era a certa, e não por ser maioria:** **6** é o único valor que a
escada admite. Um bloco tem de separar-se **mais** que duas linhas dele (4) e **menos** que duas
secções (8) — a `8` ele lê-se como uma secção, a `4` como mais uma linha. *A hierarquia que o olho
vê tem de bater com a que existe.*

⚠️ **E o `section_gap_px` não é número novo: ele já era shipado sem nome**, como `pad * 2.0` dentro
do `SectionCards::close_at` (wave 9). ⭐ Duas derivações independentes no mesmo número — a nossa
(`card_pad·2`) e a do Godot (`base·2`) —, e nenhuma escolhida. *Enquanto uma grandeza não tem nome,
ela não é COMPARÁVEL com a vizinha* — e foi exactamente isso que deixou 8 sítios responderem à
cauda de um bloco com o número da secção, sem que nada pudesse dizer que respondiam à pergunta
errada.

#### A conversão foi UNIFORME, e é isso que a torna segura

⛔ **Nada de julgar 78 sítios um a um.** O degrau que cada sítio escreveu **é** a evidência da
intenção dele: `Xs` disse *«sou uma linha»*, `Sm`/`Md`/`Lg` disseram *«sou um bloco»*. ⇒ **só os 9
que discordavam da escada mudam de valor** (8 e 12 → **6**, a direcção que o dono pediu cinco
vezes); os outros 69 mudam de **dono**. *Onde eu não tinha certeza, o valor não se mexe.*

#### ⛔⛔ E o `chrome.section-gap` MENTIA — os quatro consumidores dele pedem um ÍCONE

| onde | o que ele pede |
|---|---|
| `context_menu_overlay.rs` | o **glifo** de um item de menu |
| `topbar/cluster_painter.rs` | o **galo** de um cluster do topo |
| `section_header/mod.rs` | o **piso da altura** de um chip de cabeçalho |
| `ph2d-panel-hierarchy/row.rs` | a **amostra de cor** de uma linha |

⚠️ **Um token cujo nome descreve outra pergunta é pior que um número à solta:** quem quisesse
apertar o vão entre secções mexeria neste e **encolheria quatro ícones**, em painéis diferentes,
sem nada a ficar vermelho. E a pergunta que o nome reclamava ficou sem dono — que é como a cauda de
um bloco chegou a quatro respostas. ⇒ ele passa a chamar-se **`chrome.inline-icon`**, e o vão de
secção tem hoje o nome dele. *Renomear não move um pixel — move quem responde.*

#### ⭐ O censo apanhou TRÊS sítios que a minha conversão não viu

- **`card.rs:83`** — a cauda estava numa **tupla de retorno** e acabava em **vírgula**, não em fim
  de linha. *Um censo que procura a forma de uma expressão é cego à pontuação que a rodeia* — é a
  sexta vez nesta jornada que a lição volta, e desta vez foi o gate a apanhá-la, não o smoke.
- **duas caudas com DOIS degraus** (`y + Spacing::Md.px() + …`) onde o primeiro `Md` era a **altura
  da pista** de um slider, não um vão. A cura não é isentar: é **dar-lhe nome** (`track_h`), porque
  uma altura escrita em linha que reaparece na cauda **lê-se como um segundo vão** — para o censo e
  para quem lê.

| prova de mutação | resultado |
|---|---|
| uma cauda volta a escolher o degrau (`+ Spacing::Md`) | ✅ morreu |
| o cartão volta a computar o vão (`pad * 2.0`) | ✅ morreu |
| o bloco passa a valer o mesmo que a secção | ✅ morreu |
| o bloco passa a valer o mesmo que a linha | ✅ morreu |

**Portão:** `13 069` testes / `0` falhados; clippy `--all-targets -D warnings` limpo na **workspace
inteira**.

### 7.23 — ✅ WAVE 20 (2026-09-07): o GRUPO chega aos botões soltos, e o ritmo interno é UM número

**Report do dono**, com três fotos: *«vários grupos de botões em vários painéis (flip, Audio
Editor, etc) ainda estão sem seus grupos de botões ajuntados no novo formato. Tudo o que puder for
ajuntado, ajunte. Apenas quando o grupo for nitidamente de função diferente é que deve permanecer
grupos afastados. Já na seção Markers os botões podem ser agrupados pois se referem ao mesmo
assunto.»* — mais: *«entre grupos de botões temos um espaçamento, entre sliders outro espaçamento.
Para ambos vamos colocar o padrão de espaçamento de 3 px».*

#### Parte B — ⭐ O `3 px` dele é EXACTAMENTE o número do modelo

`GridContainer.v_separation = round(widget_margin.y − 2)` = `(4 + 1) − 2` = **3**
(`theme_modern.cpp:983`, com `widget_margin.y = increased_margin + 1` em `:286`). O corpo de um
painel desta casa **é** uma grelha de controlos, e era a constante da grelha que faltava. *Ele
chegou ao número pelo olho; eu fui buscar a derivação.*

⛔⛔ **Isto FUNDE duas portas que a wave 19 separou — um dia antes — e o motivo não é o veredito do
dono: é a PREMISSA da 19 ter dissolvido.** Aquela wave defendeu um `block_gap` (6) maior que o
`row_gap` (4) assim: *«a fronteira de um bloco tem de se ler mais que a fronteira entre duas linhas
DELE»*. Isso só vale enquanto as linhas de um bloco distam o vão de linha — e **a parte A desta
wave junta os botões em grupos, onde as peças distam um fio de 1 px**. Com o interior a `1`, uma
fronteira a `3` lê-se com folga, e o degrau do meio deixa de se pagar. ⇒ §0.0: *quem muda o
substrato tem de reconferir a nota que dependia dele.*

A escada fica com **três** degraus, cada um uma constante do Godot Modern:

| pergunta | porta | valor | derivação |
|---|---|---|---|
| duas linhas de uma LISTA | `list_row_gap_px` | 1 | `Tree.v_separation` |
| dois CONTROLOS de uma secção | **`control_gap_px`** | **3** | `GridContainer.v_separation` |
| dois CARTÕES de secção | `section_gap_px` | 8 | `Separator separation` |

⚠️ **O nome é `control_gap` e não `row_gap`, de propósito:** o antigo descrevia uma população mais
estreita do que a que o chamava — e é exactamente assim que um `block_gap` nasce ao lado dele.
*Um nome que só cobre metade dos leitores convida o segundo número.* **95 sítios** renomeados.

#### Parte A — o que estava ajuntado e o que não estava

| painel | o que era | o que passou a ser |
|---|---|---|
| **Flip · Mode** | três controlos (`3` + `3` + `2`) com vão entre eles | **um corpo** de 8 peças, `3·3·2` |
| **Flip · Sculpt Brush** | dois controlos de 4 | **um corpo**, `4·4` |
| **Flip · Self Overlap / Airbrush** | dois chips soltos | **um par**, `1·1` |
| **Áudio · Markers** *(o dono nomeou-a)* | `Add | Delete` junto **+** `Split` fora | **um corpo**, `2·1` |
| **Áudio · presets** | `Apply Save Load` com vão, largura dividida à mão | **um corpo** de 3 |
| **Áudio · commit** | `Bypass` **+** `Apply | Cancel` | **um corpo**, o toggle encosta na fileira |
| **Áudio · Variations** | quatro controlos | **um corpo**, `2·2·1·2` |
| **Mixer · M / S** | dois interruptores com vão | **um par** |
| **4 ferramentas de imagem + upscale** | `Cancel | Apply` com vão | **um par**, nas cinco |
| **Tokens** | duas ordens com vão | **um par** |

⚠️ **E o Flip reimplementava a disposição inteira:** largura dividida por `N`, `Spacing::Sm` de vão
e o botão de quatro quinas — enquanto a porta existia desde a wave 10. *Uma cópia da disposição não
se lê como cópia: lê-se como um painel que ainda não foi convertido.* O painel tinha ainda o
**próprio vão de linha atrás de um CAMPO** (`row_gap: Spacing::Xs.px()`), a 4.ª ocorrência da
lição da wave 8.

#### ⭐⭐⭐ A causa dos dois dialectos era um WIDGET que não conhecia a lei

O chip segmentado sabia a lei do grupo desde a wave 10; o **`Button` não**. Por isso o editor de
áudio juntava `Apply | Cancel` e **cinco** ferramentas de imagem desenhavam `Cancel | Apply`
separados — o mesmo par, dois idiomas, escolhidos pelo widget e não pelo desenho. ⇒ `Button::cell`
(neutro `Only`, **byte-idêntico** para os ~100 botões soltos) e o mesmo para o interruptor do mixer.
*Uma lei que só metade dos widgets conhece produz dois dialectos no mesmo aplicativo.*

#### O que os gates apanharam

- ⭐ **O censo achou QUATRO sítios que eu não tinha visto** (mixer, color-equalization, tokens,
  upscale) — a conversão foi feita pela lista que eu li, o censo varreu a árvore.
- ⛔ **Duas isenções nasceram OBSOLETAS**: escrevi-as a partir do que o código era **antes** das
  minhas próprias conversões. A metade de obsolescência acusou-as na primeira corrida.
- ⚠️ **O gate de geometria nasceu com a fixtura errada:** um `Button` de tipo `Default` num tema
  moderno é **fantasma** — não pinta fundo nem moldura —, e a régua leu `0` segmentos. *Uma régua
  de geometria precisa de uma fixtura que emita geometria*; o piso disse-o em voz alta.
- ⚠️⚠️ **O `#[allow]` mudou de dono outra vez** (2.ª vez em três waves): a função nova entrou entre
  o atributo e a função a que ele pertencia, em **dois** ficheiros. Só o *duplicado* é que o clippy
  vê.

| prova de mutação | resultado |
|---|---|
| o `Button` esquece a célula do grupo | ✅ morreu |
| uma ferramenta volta a dispor a fileira à mão | ✅ morreu |
| o controlo passa a valer o mesmo que a secção | ✅ morreu |

**Portão:** `13 072` testes / `0` falhados (a única vermelha foi a flake conhecida
`ph2d-timeline::nesting_clock`, **3 de 3 verde sozinha** com `load 36` impresso ao lado); clippy
`--all-targets -D warnings` limpo na workspace inteira.

### 7.24 — ✅ WAVE 20b (2026-09-07): os três reports do smoke da 20

**Report do dono**, foto do Audio Editor com duas setas vermelhas e duas verdes.

#### 🔴 1 — «um espaçamento exagerado» (entre os dois blocos do EDIT)

`y += grid_height(5, ROW_H) + Spacing::Md.px();` — um `8` escrito à mão. ⛔ **O censo da wave 19
não o via:** ele procura a **CAUDA** de um pintor (uma expressão final, sem `;`) e esta fronteira é
uma **instrução a meio da função**. *Um censo que conhece uma forma da mesma pergunta é cego às
outras* — é a sexta vez nesta jornada, e a **primeira em que foi o olho do dono a apanhá-la** em vez
de um gate. ⇒ o censo ganhou a metade que lê `grid_height(…) + rung` (2 sítios; o irmão vivia no
`motion-params`).

#### 🔴 2 — «a ausência de espaçamento» (Bypass colado ao Apply | Cancel)

⛔⛔ **A wave 20 juntou-os por engano, e a régua que eu usei era larga demais.** Eu escrevi *«os três
respondem à mesma pergunta»* — mas quase tudo dentro de uma secção responde à mesma pergunta. A
régua que decide é a **dele**, e ela pergunta *o que a peça É*: o `Bypass` é um **estado** que se
liga e desliga; `Apply` e `Cancel` são **ordens** que terminam a audição. *«Apenas quando o grupo
for nitidamente de função diferente é que deve permanecer grupos afastados»* — um interruptor ao
lado de dois comandos é exactamente esse caso.

#### 🟢 3 — «as setas no mesmo grupo dos botões Apply, Save e Load»

⚠️ **A fileira de cima tem larguras DESIGUAIS** — duas setas estreitas e fixas, o nome a ocupar o
resto —, e é por isso que ela ficava de fora: o `block_cells` reparte cada fileira em partes
**iguais**. ⇒ nasce o `block_cells_of` (larguras dadas) e o `stepper_middle_w`.

⭐ **O nome ganha SUPERFÍCIE**, e não é decoração: sem ela o corpo tem um buraco no meio da primeira
fileira e o que se lê é *duas setas soltas* em vez de *um selector*. É o mostrador de um stepper — a
peça do meio de `[−][ valor ][+]`. ⛔ E ela **não é clicável**: o nome não abre lista nenhuma, e uma
affordance que o painel não pode honrar é pior que nenhuma.

Vale para os **três** selectores do painel (preset, efeito, estratégia de variação) e o do
**Delivery** — o do efeito tem uma peça a mais, o `↺`, que passou a ser **peça do corpo** e não um
ícone solto por cima dele.

⚠️ **E o `ARROW_W = 26.0` estava declarado TRÊS vezes**, uma por ficheiro de selector — a mesma
espécie do `Bg1` com doze cópias e do vão de linha atrás de um campo.

#### ⛔⛔ E a bancada de MUTAÇÃO tinha um defeito que escondia a verdade

Restaurar com `mv f.bak f` devolve o mtime do **backup**, anterior ao do ficheiro mutado que o cargo
acabou de compilar — e o cargo mede frescura por **mtime**. ⇒ ele guarda o artefacto **da mutação**,
e a corrida seguinte mede o programa errado. Apanhei-o porque um gate acusou `dois CONTROLOS (8)`
sobre uma fonte que diz `3`, com um aviso de *«`widget_margin_y_px` never used»* sobre uma função
visivelmente chamada. *Quando o compilador contradiz o ficheiro aberto à frente, suspeite do
artefacto.* ⚠️ **Ele escondia-se atrás do `cargo fmt`** (que actualiza o mtime ao reescrever), e é
por isso que sobreviveu a várias waves. Cura: `touch` no fim de toda restauração.

| prova de mutação | resultado |
|---|---|
| o vão exagerado do EDIT volta | ✅ morreu |
| o controlo passa a valer 10 | ✅ morreu |

**Portão:** `13 073` testes / `0` falhados; clippy `--all-targets -D warnings` limpo. ⚠️ O teto de
LOC do `paint.rs` do editor de áudio (767 de 600) foi pago por **corte** — as portas de grupo
saíram para `paint_groups.rs`, e a linha do corte é a pergunta: o pai pinta **uma peça**, o irmão
diz **como N peças formam um corpo**.

### 7.25 — ✅ WAVE 20c (2026-09-07): *«pode juntar»* — os dois vãos que sobravam no EFFECTS

**Report do dono**, foto com duas setas verdes sobre os dois vãos que restavam na secção.

#### 🟢 1 — o CABEÇALHO da secção vira um corpo de três fileiras

`◀ preset ▶` · `Apply | Save | Load` · `◀ efeito ↺ ▶`. As três respondem à mesma pergunta — *o que
estou a editar?*: um preset é o ponto de partida da cadeia, as três ordens agem sobre ele, e o
selector diz que efeito está aberto nos parâmetros logo abaixo.

⚠️ **As fileiras têm larguras desiguais ENTRE SI** (`3` peças de larguras dadas · `3` iguais · `4`
peças), e é o `block_cells_of` da wave 20b que as exprime.

#### 🟢 2 — o `Bypass` encosta na CADEIA, não no `Apply | Cancel`

⭐ **É a mesma pergunta da 20b resolvida do lado certo.** Ali eu tinha-o colado ao `Apply | Cancel`
e o dono reprovou; separá-lo deixou-o a flutuar. A resposta é **onde ele age**: o `Bypass` silencia
a **cadeia inteira** — ele é o rodapé daquela lista, não o vizinho de cima de duas ordens. ⇒ o vão
acima dele fecha, o de baixo fica.

#### ⛔⛔ E uma mutação SOBREVIVEU: a porta nova não tinha régua

Triplicar o fio entre as peças do `block_cells_of` não acordou gate nenhum. A porta nasceu na 20b e
o que a usava só era medido por **censos de FONTE**, que não olham para um pixel. *É a 4.ª vez nesta
jornada que escrevo a peça certa e não a gateio.* ⇒ três réguas próprias (as peças encostam sobre um
fio · as fileiras encostam · a última peça acaba onde o bloco acaba · só as quinas de fora
arredondam · o meio do stepper desconta os DOIS fios), e as três mutações passam a morrer.

| prova de mutação | resultado |
|---|---|
| as peças do bloco de larguras dadas deixam de encostar | ✅ morreu *(sobreviveu antes das réguas)* |
| o meio do stepper esquece os dois fios | ✅ morreu |
| toda peça do bloco arredonda os quatro cantos | ✅ morreu |

**Portão:** `13 076` testes / `0` falhados; clippy `--all-targets -D warnings` limpo.

### 7.26 — ✅ WAVE 21 (2026-09-07): a linha ESCOLHIDA de uma lista tem UMA voz

**Report do dono**, foto da cadeia de efeitos: *«veja que o nome Low-Pass está com o fundo na cor
dos botões, sem layout adequado, grudado nos botões abaixo. Os botões abaixo deveriam ser do mesmo
grupo juntos. Estude o layout e corrija.»*

#### ⛔⛔ Cinco listas, QUATRO dialectos para a mesma frase

| lista | o que pintava | o que isso é |
|---|---|---|
| hierarquia | `AccentSoft` + raio + **moldura de acento** | uma caixa |
| **cadeia de efeitos** | **`Bg3`** a sangrar, sem quinas | **o repouso de um BOTÃO** |
| âncoras / animações | **`Bg2`** com raio | o tom do HOVER, numa caixa |
| variações do áudio | **`Accent` cheio**, texto invertido | um botão aceso |

⚠️ **Nenhum estava errado sozinho** — cada um lia-se bem no painel dele. O que estava errado era
haver quatro: *duas superfícies que dizem a mesma coisa de maneiras diferentes ensinam ao artista
que elas são coisas diferentes.*

**A lei, com as três derivações:** o tom é `AccentSoft` — o *pressed* do modelo
(`style_tree_selected = flat_button_pressed`, `theme_modern.cpp:709`), **nunca o repouso**; ela
**SANGRA** (`content_margin_all(0)` do mesmo sítio); e ⛔ **não tem quinas nem moldura**, que é
veredito do dono medido três vezes — *o que diz «botão» é a QUINA*.

#### E a cadeia tinha MAIS TRÊS respostas só dela

Altura **18 px** (contra as 22 de toda linha do app), vão **zero** (escrito como a ausência de um
termo) e o realce acima. ⛔⛔ **E ela não estava no censo de listas** — que é uma lista **declarada
à mão**: *uma superfície que ninguém declarou escapa a um censo por declaração.* Hoje ela é a
sexta, com altura, vão, listra de paridade e realce pela porta.

#### 🔴 «grudado nos botões abaixo» / «os botões abaixo deveriam ser do mesmo grupo»

⚠️⚠️ **A wave 20b separou o `Bypass` do `Apply | Cancel` e a 20c pôs o `Bypass` na lista — as duas
leituras estavam erradas, e o que as gerou foi eu procurar o sítio dele pela FUNÇÃO** (*«é um
estado, não uma ordem»* · *«ele age sobre a cadeia»*) **em vez de pela SUPERFÍCIE**. Os três
partilham a mesma faixa de acção no fim da secção, e é isso que o olho lê; a lista acima é outra
superfície, e o que os separa é o vão que ela agora deixa. ⇒ `Bypass` / `Apply | Cancel` = um corpo
`[1, 2]`, e a lista fecha com o vão de um controlo.

#### O que os gates apanharam — e as duas vezes que a régua era a errada

- **A hierarquia declara-se em `paint.rs` e pinta a linha em `row.rs`** — *uma superfície pode ser
  dois ficheiros*, e um censo que lê um ficheiro por superfície acusa o inocente.
- **O inspector da grade não tem linha escolhida nenhuma**, logo procurar a porta lá é procurar a
  resposta a uma pergunta que ele não faz. ⇒ a lei do realce ganhou a **sua própria** lista de
  sítios, com a metade que declara *«esta não tem realce, e é porquê»*.
- ⚠️ **`list_rows.rs` virou pasta**, e a entrada de isenção do HR-12 passou a descrever um ficheiro
  que já não existe: *um ficheiro que vira pasta renomeia-se aos olhos de todo censo por caminho.*

| prova de mutação | resultado |
|---|---|
| a linha escolhida volta ao tom de um botão parado | ✅ morreu |
| a linha escolhida ganha quinas | ✅ morreu |
| uma lista volta a pintar o realce sozinha | ✅ morreu |

**Portão:** `13 080` testes / `0` falhados; clippy `--all-targets -D warnings` limpo.

### 7.27 — ✅ WAVE 22 (2026-09-07): a QUINA de um controlo vem da porta do tema

**Fecha o item 5 do §8 do handoff** — *«50 raios ainda passam ao lado da porta»*.

#### ⛔⛔ O caso que nomeia a wave

`Button::radius()` devolvia `Radius::Md` (**4**) sem perguntar nada, enquanto um chip segmentado
ao lado pintava **3** (o `corner_radius` do Godot Modern, wave 8). *Dois widgets encostados na mesma
fileira, dois raios.* E a causa não é um esquecimento pontual: a wave 2/3 levou a porta da
**moldura** a 44 pintores, e o **raio** só a acompanhou onde o mesmo sítio já a chamava — ficando
de fora dos **primitivos**. ⇒ **um primitivo que escolhe sozinho não é uma excepção: é a resposta
que ~100 sítios herdam sem saber.**

**65 sítios convertidos** em 48 ficheiros. No tema clássico o desenho é **byte-idêntico** (a porta
devolve o que recebe); num tema moderno cada um passa a pintar o `3`.

#### ⭐ A partição tem TRÊS classes, e não duas

O handoff dizia *«cromo contra canvas»*. Medida, a partição tem uma terceira:

| classe | fica | mecanismo |
|---|---|---|
| a quina de um **CONTROLO** | ⇒ pela porta | é a lei |
| uma **FORMA** deliberada (`Radius::Full`) | ⛔ fora | **a porta ACHATA** — `visuals::radius` devolve `MODERN_CORNER_RADIUS_PX` para qualquer `classic > 0`, logo uma pílula sairia rectângulo e um avatar redondo sairia quadrado |
| o **CANVAS** | ⛔ fora | o raio é geometria do documento (a régua da timeline, a célula da tira de filme, o marquee, o gizmo 3D) |

⚠️ **A segunda classe tem gate próprio** (`the_door_flattens_every_positive_radius_including_a_pill`):
sem ele, «as formas ficam de fora» lê-se como zelo em vez de mecanismo — e no dia em que a porta
deixasse de achatar, a lista de isenções passaria a descrever uma razão que já não existe.

#### E uma quarta coisa que se lê igual: o raio usado como TAMANHO

`showcase/status.rs` usa `Radius::Xl2.px()` (20) como o **tamanho de um spinner**. É a mesma espécie
do `chrome.section-gap` que a wave 19 renomeou — *um token cujo nome descreve outra pergunta* —, e
fica **nomeado na isenção** com a cura (um token de tamanho), para não se perder.

#### O que a construção corrigiu em mim

- ⚠️ **A minha conversão passou por cima de duas decisões ESCRITAS** — o post-it do `showcase/notes`
  e o contorno de marcador do Inspector têm, os dois, um comentário a dizer *porque* ficam fora da
  porta, e o script converteu-os na mesma. *Um script que edita por forma não lê o motivo que está
  duas linhas acima.* Revertidos, e agora são isenções declaradas.
- ⚠️ **A 1.ª régua contava LINHAS e acusou dois inocentes**: duas chamadas já estavam dentro da
  porta, numa chamada de três linhas. ⇒ a régua conta **parênteses para trás**. *É a sexta vez que
  a forma multi-linha morde esta linha.*

| prova de mutação | resultado |
|---|---|
| o `Button` volta a escolher a própria quina | ✅ morreu |
| um painel volta a escolher a quina | ✅ morreu |
| a porta deixa de achatar | ✅ morreu |

**Portão:** `13 083` testes / `0` falhados; clippy `--all-targets -D warnings` limpo.

### 7.28 — ✅ WAVE 23 (2026-09-07): o RECUO de um filho é UM número

**Não fecha item de handoff — nasce de uma AUDITORIA do placar.** O «⏳ o que sobra do estudo
§5.3» do [README](../README.md) tinha cinco itens medidos em 04/09, e **quatro já tinham fechado**
sem ninguém reabrir a lista: os cantos dos painéis (a porta do tema dá 3), a moldura dos cartões
(`stroke_frame` devolve `Stroke::NONE` no moderno), a moldura permanente das caixas de texto
(o `Chrome::field_border` só é visível no clássico) e as pílulas de etiqueta e amostra (as duas
passam pela porta do raio desde a wave 22). *O placar de uma linha envelhece à velocidade das
próprias waves dela* — e o que sobrava de verdade não estava na lista.

#### ⛔⛔ O que o censo achou: CINCO superfícies, QUATRO respostas

A pergunta é uma: *quanto se desloca para a direita a linha de um filho?*

| superfície | escrevia | passo |
|---|---|---|
| Hierarquia | `Spacing::Xl` | **16** |
| `variant_editor` | `INDENT_PX = 16.0` à mão | **16** |
| Painter Layers | `LAYER_INDENT_STEP = 14.0` à mão | **14** |
| Catálogo do Asset Browser | `Spacing::Md` | **8** |
| `tree_view` (a GALERIA) | `Spacing::Lg` | **12** ✅ |

⭐⭐ **A galeria de widgets já tinha a resposta do modelo, e nenhuma superfície do produto a
copiou.** O `tree_view` é a peça de referência do cromo e só é pintado na bancada — *uma
referência que ninguém chama não ensina nada; ela só regista que a resposta certa já era
conhecida.* É o simétrico exacto da wave 12, onde a galeria pintava um risco azul que o produto
já não fazia: ali a referência estava atrasada, aqui está adiantada, e nos dois casos ninguém
compara.

#### A lei, e o recurso do piso dela

Godot Modern (MIT, `theme_modern.cpp:653`):

```cpp
item_margin = EDSCALE_RND(MAX(3 * increased_margin, 12))
```

Com o `increased_margin` desta casa (`Spacing::Xs` = 4): `MAX(12, 12)` = **12 px**. Porta:
[`ph2d_tokens::list_indent_px`](../../../crates/ph2d-tokens/src/spacing.rs).

⚠️ **O piso não é um número de segurança: ele tem RECURSO, e o recurso é a coluna da seta.** Dois
níveis consecutivos põem as suas setas a *um passo* de distância, logo um passo mais estreito que
a seta faz a do filho entrar por cima da do pai. É por isso que ele é escrito como
`.max(tree_chevron_col_px())` e não contra o `12` cru do Godot — *um limite legítimo diz de que
recurso ele é* (§0.0). ⭐ E a coluna da seta desta casa é `Spacing::Lg` = 12: as duas derivações
independentes caem no mesmo número.

⚠️⚠️ **Hoje os dois lados do `max` valem 12, logo o piso NÃO é observável no produto** — apagá-lo
devolve o mesmo valor e a mutação sobrevive. Por isso a derivação vive numa função com os dois
termos abertos (`indent_from`), que o teste chama com uma escala apertada. *Uma cerca que só se lê
no valor de hoje é uma cerca que a próxima mutação atravessa sem acordar ninguém.*

#### ⭐ A segunda metade: os dois números que a hierarquia sincronizava à MÃO

A linha desenha a seta (`row.rs`) e o desenhador do parentesco desenha o fio que sai de baixo dela
(`paint.rs`). As **duas medidas que os dois têm de partilhar** — a coluna da seta e o recuo interno
da linha — estavam escritas nos dois ficheiros, cada uma com metade de um comentário a mandar
sincronizar (*«MUST match `paint.rs::row_inner_pad`»* / *«sync with row.rs chev_w»*).

⚠️ **Um par sincronizado por comentário não é uma lei: são duas leis que hoje concordam** — e a que
derivasse tirava o fio de baixo da seta, que é um report que este painel já pagou (Enio,
2026-05-26: *«a linha que mostra parentesco deveria sair exactamente abaixo da setinha»*).

⏳ **E o modelo discorda do VALOR do recuo interno, o que fica NOMEADO e não corrigido:**
`Tree.inner_item_margin_left = base_margin` = **4 px** contra os **2** desta casa. Mexer nele
desloca toda linha da hierarquia — é medição de uma wave própria.

#### ⛔⛔ O preço escondido: apertar uma entrada invalidou a constante calibrada à volta dela

O catálogo do Asset Browser é o único que **alarga** (8 → 12), e a coluna dele tinha
`NOMINAL_W = 140.0` escrito à mão com a composição só no comentário: *«recuo + até três níveis de
indentação + um rótulo de ~12 caracteres + a contagem»*. Com a largura fixa, o passo maior comeria
**12 px do nome mais fundo** — exactamente o defeito que aquele número existia para evitar.

⇒ a largura passa a ser a **soma** (`recuo + 3·passo + texto + margem`), e o que sobrevive à
mudança é o **orçamento de TEXTO** (`CONTENT_W = 98`, contado do que o `140` shipava), nunca o
total. *O número que shipou é a evidência do que cabia.* ⚠️ E as duas metades do comentário estavam
**desactualizadas**: ele dizia `Spacing::Md` de recuo onde o código escreve `Spacing::Sm`.

#### O portão

`crates/ph2d-editor-core/tests/the_indent_of_a_child_is_one_number.rs` — 5 testes:

1. toda superfície que recua por nível chama a porta;
2. **a metade de obsolescência**: as 5 declaradas ainda recuam;
3. nenhuma constante de recuo nasce fora da porta;
4. o passo nunca fica mais estreito que a coluna da seta;
5. os dois ficheiros da hierarquia lêem a geometria partilhada de **uma** declaração cada — ⚠️ este
   **nasceu de uma mutação que SOBREVIVEU**, e é a **5.ª vez** que esta linha escreve a porta certa
   e não a gateia.

Mais 3 testes de unidade em `spacing.rs` (o valor do modelo · o piso com escala apertada · o piso
contra a coluna da seta) e 2 em `paint_catalog.rs` (o orçamento de texto sobrevive ao recuo · a
largura É a soma).

⚠️ **A régua distingue `*` de `*`.** Um passo por nível lê-se como uma multiplicação por uma
profundidade, e a varredura ingénua (*a linha fala de `depth` e tem um `*`*) acusa dois inocentes,
os dois porque o `*` deles é uma **desreferência** (`*rect`, `(*v as f32)`). A régua exige o `*`
binário, que o `rustfmt` escreve sempre com espaço dos dois lados e que um deref nunca tem. *Um
censo que parseia o fonte tem de saber todas as formas do que lê* — 7.ª ocorrência.

**Provas de mutação: 7 escritas, 7 mortas** (Painter Layers re-escolhe o passo · a porta perde o
piso · nasce uma constante própria · a coluna da seta volta a ser cópia · o recuo interno volta a
ser cópia · uma declaração partilhada desaparece [controlo de vacuidade] · a largura da coluna
volta a ser escolhida).

### 7.29 — ✅ WAVE 24 (2026-09-07): o CARET pergunta ao pintor onde o texto começa

**Nasce de um censo abortado.** A wave ia ser *«o vão entre um ícone e o seu rótulo»* — cinco
respostas medidas (`12` num toast, `8` em seis sítios, `6` em dois, `4` na hierarquia). ⛔ **E o `4`
da hierarquia é VEREDITO ESCRITO do dono**, com data e citação no código: *«Icon → name gap
tightened Md (8) → Xs (4) 2026-05-24 per user: "nome mais próximos dos ícones"»*. Uniformizar em
`6` — o `Tree.icon_h_separation` do modelo — sobreporia uma decisão dele com uma derivação minha.
⇒ *o censo fica medido e a escolha é dele*; a wave seguiu para a metade que é mecanismo puro.

#### ⛔⛔ O defeito: o mapeador de clique→caret COPIAVA os números do pintor

`text_ops::byte_offset_from_click_xy` tem um braço por classe de campo, e cada um precisa de saber
onde o pintor pôs a primeira letra:

| braço | o pintor desenha em | o caret procurava em |
|---|---|---|
| `TextInput` de uma linha | `rect.x + Spacing::Lg.px()` | `rect.x + 12.0` |
| `NumberInput` | `rect.x + Spacing::Lg.px()` | `rect.x + 12.0` |
| `Combobox` | `+ Lg + ícone + Md` | `+ 12.0 + ícone + 8.0` |
| campo `Hex` | `rect.x + Spacing::Md.px() + 36` | `rect.x + 8.0 + 36.0` |

⭐⭐⭐ **Os números da direita são os valores de FÁBRICA dos da esquerda**, e é isso que torna o
defeito invisível: enquanto ninguém autora a escala numérica, as duas contas dão o mesmo. Desde que
ela virou autorável, o pintor lê o valor **vivo** e a cópia não — mexer no `spacing.lg` faz o
utilizador clicar numa letra e escrever noutra.

#### ⚠️⚠️ A família já tinha sido diagnosticada e curada pela METADE

O braço do `TextArea` ganhou a porta dele (`text_area_metrics`) com **este mecanismo escrito ao
lado, em prosa**, e um gate a prová-lo com a escala autorada (*medido: com `md = 20`, clicar no meio
de qualquer linha punha o caret na linha seguinte*). Os outros **três braços do mesmo `match`**
ficaram a copiar. *Curar um braço de uma família deixa os outros com o defeito **e** com a
aparência de resolvidos — e o diagnóstico correcto, escrito ao lado do primeiro, não os alcança.*

#### As portas

`field_pad_x()` (o recuo horizontal de um campo, partilhado pelo campo de texto e pelo numérico) ·
`text_input::text_origin_x` · `combobox::text_origin_x` · `combobox::inline_icon_size` ·
`hex_field::text_origin_x`. Os pintores passam a ler as mesmas.

⭐ **E a fórmula do tamanho do ícone do combobox estava em TRÊS cópias** — o pintor, o
`clear_button_rect` e o mapeador de caret (esta última com os limites em literais). *Três cópias de
uma conta são três leis que hoje concordam.*

#### O portão

`caret_doors.rs`, 6 testes. Os dois comportamentais correm **duas vezes** — escala de fábrica como
CONTROLE (o mundo em que a cópia acerta por coincidência; sozinho ele é verde sobre o produto
quebrado) e escala **autorada** como discriminador. A forma é herdada do braço já curado.

⭐ **O 6.º teste encodifica o defeito directamente:** *o mapeador de caret não escreve NÚMEROS de
geometria*. É a única régua que o apanharia **antes** de alguém autorar a escala. Ele isenta duas
coisas, as duas nomeadas: o `0.0` de um `max` e a `APPROX_ADVANCE_RATIO`.

⚠️ **E a 1.ª corrida do censo da fórmula acusou o PRÓPRIO GATE** como segundo dono — a agulha
aparece literalmente dentro dele. *Uma varredura de fonte que não se exclui mede-se a si própria.*

**Provas de mutação: 7 escritas, 6 mortas.** A 7.ª **sobreviveu e não é buraco**: repor a conta em
linha no pintor do hex devolve a mesma expressão (os dois termos são o mesmo token e a mesma
constante), e o pintor é o dono dela — é a leitura *«a linha era redundante»*, não *«falta um
gate»*. ⚠️ Duas das seis só morreram **depois** de eu escrever o gate que faltava, e as duas eram
cópias que *hoje concordam* — a mesma espécie do piso do recuo na wave 23.

#### ⏳ Fica medido e por decidir (é do dono)

O vão entre um ícone e o seu rótulo tem **cinco respostas**: `12` (o toast, um literal cru), `8`
(`list_item`, `tree_view`, combobox, topbar, showcase), `6` (menu de contexto, cabeçalho de secção
— e é o do modelo), `4` (hierarquia, **por veredito dele**) e dois locais. O modelo parte a
pergunta por classe (`Tree.icon_h_separation` = 6 · `Button.h_separation` = 4 ·
`CheckBox.h_separation` = 8 — e a nossa caixa de verificação **já está** nos 8). *Uniformizar exige
saber se o «mais próximos» de 2026-05-24 vale para todo o app ou só para a hierarquia.*

#### ⏳ E um ponto cego do portão de números mágicos, medido aqui

O gate `no_magic_numeric` varre `widget/`, `screens/` e os painéis — **nunca a raiz de
`ph2d-editor-core/src`**, que é onde vivem os primitivos que todo pintor chama. Medido com a régua
do próprio gate: **759 sítios**, dos quais ~570 são fixturas de `dispatch/tests/`. O resto real é
liderado por `paint.rs` (14 — o pintor do **toast**, inteiramente fora do sistema de tokens),
`grid_snap/state.rs` (12), `floating_panel.rs` (11), `ruler.rs` (10) e `zones.rs` (8). *Um gate que
varre um directório afirma sobre o directório.*

### 7.30 — ✅ WAVE 25 (2026-09-07): o vão ícone→rótulo é **4**, por decisão do dono

**Fecha o item que a wave 24 devolveu.** O censo estava feito e a escolha era dele: perguntado se
o veredito de 2026-05-24 na Hierarquia (*«nome mais próximos dos ícones»*, `Md`→`Xs`) valia só
para aquele painel ou para o app inteiro, respondeu **«para o app todo»**.

⛔⛔ **O número diverge do modelo, e a divergência é deliberada, datada e gateada.** O Godot Modern
parte a pergunta por classe — `Tree.icon_h_separation` = `base_margin·1,5` = **6** ·
`Button.h_separation` = **4** · `CheckBox.h_separation` = **8** — e esta casa passa a ter **4** para
todos. *Um veredito de produto medido no ecrã ganha de um número portado*, e há um teste com o nome
inteiro a impedir que alguém «corrija» a casa de volta para o `6`.

As cinco respostas que morreram: `12` à mão no balão de aviso · `8` na lista, na árvore, no
combobox e na barra do topo · `6` no menu de contexto e no cabeçalho de secção · `4` na Hierarquia,
nas camadas do Painter e na pilha do Vector · e dois locais.

#### ⭐ O que NÃO é esta pergunta — três leis vizinhas, cada uma com dono

- **seta → ícone**: a Hierarquia dá-lhe `Xxs` = 2, por outro veredito do mesmo dia;
- **ícone → ícone** numa fileira: é a lei do GRUPO (wave 20), e ali as peças **encostam**;
- **ícone → chip**: dois widgets, não um widget e a legenda dele.

As três estão na lista de isenções **com o mecanismo escrito**, porque a régua textual não as
distingue — e uma isenção sem motivo é a porta pela qual a sexta resposta volta.

#### ⚠️⚠️ A régua teve de aprender a ler INSTRUÇÕES, e o adversário foi o `cargo fmt`

A 1.ª redacção lia **linhas**, e o formatador do próprio repo derrotou-a no mesmo dia: ele partiu
`host.x + field_pad_x() + inline_icon_size(host) + icon_label_gap_px()` em quatro linhas, e
**nenhuma** tinha ao mesmo tempo o nome do ícone e o vão. *Um censo que parseia o fonte tem de
saber todas as formas do que lê* — 8.ª ocorrência nesta linha, e a primeira em que o adversário é
uma ferramenta do repo, não um autor.

⇒ a régua normaliza o fonte em **instruções**: comentários fora, continuações juntas, e o corte é
`;` e chaveta **sempre**, mais a vírgula **só a profundidade zero de parênteses**. ⚠️ Cada metade
desse corte foi paga por um falso positivo: sem a chaveta, os braços de um `match` colam-se e o
planeador da barra do Flip aparecia com o `ICON_W` de um braço e o `Spacing::Xs` de outro; com a
vírgula a cortar em qualquer profundidade, `paint(x + ícone + vão, …)` partia-se ao meio.

⚠️ **E a régua acusou-se a si própria** (2.ª vez em duas waves): o gate contém, por construção, a
composição que afirma — os testes internos passam a ficar fora da varredura.

#### ⭐ E o censo é de MUNDO ABERTO

A 1.ª redacção só olhava a lista declarada — e uma superfície **nova**, que é precisamente como
uma sexta resposta nasce, passava sem ser vista. *Um censo que só mede a própria lista não impede
nada.* Hoje ele varre a árvore e exige a porta ou uma isenção nomeada. As três metades de
obsolescência (declaradas · isentas · o valor) impedem que ele envelheça para verde.

#### ⭐ Um mirror a menos, de borla

O `list_item` calculava a folga do valor **refazendo a aritmética do pintor**, com a fórmula do
tamanho do ícone copiada e um comentário a apontar *«espelha o pintor, linha 106»*. *Um comentário
que aponta um número de linha é um ponteiro que envelhece na primeira edição.* Hoje as duas leem
`list_item::icon_size`. É a mesma espécie que a wave 24 curou no combobox.

**Provas de mutação: 3 escritas, 3 mortas** (a árvore re-escolhe o vão · alguém repõe o `6` do
modelo · nasce uma superfície com número cru — esta última é a que a régua de LINHA não via).

### 7.31 — ✅ WAVE 26 (2026-09-07): a COLUNA DO TOPO nunca passou por porta nenhuma

**Nasce de um ponto cego que a wave 24 mediu e nomeou.** As waves 2–5 levaram a porta da moldura a
44 pintores e puseram a catraca a **zero** — e os dois inquilinos da coluna do topo, o **balão de
aviso** e a **barra de trabalho**, continuavam a traçar um `stroke_rounded_rect` cru a 1 px. Num
tema moderno eles desenhavam o contorno que a pele plana apagou em toda a casa.

#### ⛔⛔ Porque o censo não os viu — são DOIS buracos, não um

**(a) Uma isenção escrita para uma FUNÇÃO protegia o FICHEIRO.** O `paint.rs` estava na lista com o
motivo *«é a PORTA: `stroke_frame` chama `stroke_rounded_rect` por definição»*. A frase é verdade
para o corpo daquela função — e o pintor do balão vive **290 linhas abaixo, no mesmo ficheiro**.
*Uma isenção nomeia uma coisa e cobre tudo o que partilhe o ficheiro com ela.* ⇒ o censo passa a
apagar o corpo das funções que **são** a porta antes de perguntar, e a isenção de ficheiro sai.

**(b) O censo enumerava directórios À MÃO** — `widget/`, `screens/hero/` e **um ficheiro escrito à
mão** (`paint.rs`). O `progress.rs`, que pinta o outro inquilino da mesma coluna, **nunca foi
olhado**. ⇒ ele varre a raiz inteira do `editor-core`, e o alargamento acusou exactamente **três**
ficheiros: o `progress.rs` (curado), o `paint_rounded.rs` (a casa do primitivo — isento, e ao
contrário do `paint.rs` esta isenção descreve o ficheiro inteiro) e o `gizmo/paint.rs` (as alças de
um gizmo sobre o canvas — mesma família do marquee).

#### ⭐ O gate mede PIXEL, e tinha de medir

⚠️ **O censo textual não podia fechar isto**, e a razão é que as duas curas deixam o ficheiro a
*conhecer* a porta — logo ele fica verde mesmo com um traço cru ao lado. Pior: a cláusula
`|| body.contains("visuals::")` abençoa o ficheiro inteiro por uma menção. ⇒ o gate desta wave
pinta a coluna nos **dois** temas e conta caminhos: o clássico tem de emitir **mais** que o
moderno. A régua é a DIFERENÇA, nunca um absoluto — um número absoluto envelheceria à primeira
mudança de conteúdo do balão.

⭐ E um terceiro teste exige que os **dois inquilinos percam o mesmo**: curar só o balão deixaria a
mesma coluna com duas peles, que é pior que os dois errados por igual.

#### ⭐ E os literais do balão eram TODOS tokens, ao valor exacto

`13.0` = `TypeToken::Base` · `1.5` = `StrokeToken::Default` · `1.0` = `StrokeToken::Thin` ·
`16.0` = `Spacing::Xl` · `4.0` = `Spacing::Xs`. *Um pintor fora do sistema não estava a divergir
dele — estava a repetir a tabela de cor, à mão.* ⚠️ O recuo da faixa de acento passa a ser a
**largura da moldura do tema**: num tema moderno não há moldura, e a faixa encosta à borda em vez
de deixar um fio do fundo à mostra. ⏳ Só o lado do ícone (24) fica sem token — nomeado em
`TOAST_ICON_PX`, porque o `inline-icon` (14) é o glifo de uma linha e o `icon-btn-size` (36) é um
botão, e este está no meio.

#### ⛔ E o `JobQueue` tinha uma armadilha de omissão

`#[derive(Default)]` sobre um `cap: usize` dava **zero** — e o `push` de uma fila cheia devolve
`false` **em silêncio**, por desenho. Logo `JobQueue::default()` era uma fila que descartava toda
barra sem erro nenhum. ⚠️ **O produto usa `new()`, então nunca mordeu o utilizador: a primeira
vítima foi o gate desta wave**, que pintou uma coluna vazia e acusou o pintor. *Um `derive` que
produz um estado que o construtor nunca produz é uma segunda definição do tipo, escrita por
omissão.* Curado (`Default = new`) e gateado pelo comportamento.

**Provas de mutação: 3 escritas, 3 mortas** (o balão volta ao traço cru · a barra volta ao traço
cru · a armadilha do `Default` volta). ⚠️ **A primeira delas SOBREVIVEU à primeira tentativa** —
com o censo textual, porque a cláusula `visuals::` mantinha o ficheiro abençoado. *Foi essa
sobrevivência que obrigou o gate a mudar de régua, de texto para pixel.*

#### ⚠️ E o portão de fecho desta wave é o retrato da FAMÍLIA DE FLAKES DE CARGA

Três corridas da **mesma árvore**, com a máquina entre `load 16` e `41`, devolveram **quatro
reprovadas diferentes**:

| corrida | reprovada | crate | o que ela mede |
|---|---|---|---|
| 1 | `a_wet_move_costs_what_the_footprint_costs…` | `ph2d-tool-painter` | razão de dois relógios |
| 2 | `no_expression_allocates_no_link_frame` | `ph2d-timeline` | contador de alocações |
| 3 | `the_mask_stroke_cost_does_not_follow_the_canvas` | `ph2d-tool-painter` | razão de dois relógios |
| 3 | `the_cost_of_a_player_is_linear_in_their_number` | `ph2d-physics-ecs` | razão de dois relógios |

As quatro deram **3 de 3 verde sozinhas** com o `/proc/loadavg` impresso ao lado, e o diff desta
wave tem **zero linhas** nas três crates. ⭐ *A assinatura decisiva não é nenhuma delas
individualmente: é o CONJUNTO de reprovadas MUDAR entre corridas do mesmo binário* — um defeito de
lógica reprova sempre o mesmo caso.

⏳ **Três das quatro já estão nomeadas na família do `CLAUDE.md §5`; a quarta não:** o
`ph2d-physics-ecs::the_cost_of_a_player_is_linear_in_their_number` é membro novo confirmado, e fica
registado aqui em vez de num ficheiro partilhado a meio da linha — a promoção dele para a lista do
roteador é da integração. *A lista nunca estará completa, e é o próprio §5 que o diz.*

#### ⏳ Fica medido e nomeado: o censo da moldura é por FICHEIRO, e a pergunta é por CHAMADA

Medido em 2026-09-07 com uma régua por-chamada (o corpo da porta apagado, cada `stroke_rounded_rect`
classificado pelos **próprios argumentos**): **29 chamadas cruas** em ~20 ficheiros. A maioria vive
em ficheiros que já estão isentos com motivo escrito (o marquee, o «largue aqui», as alças, a pele
de um documento). Ficam ~**14 ficheiros sem veredito** — entre eles o anel de foco de um botão, o
contorno de erro de um campo, a amostra de cor de um menu e quatro do grafo de motion. *Cada um
precisa da mesma pergunta que esta wave respondeu para dois: é moldura de repouso, ou é a mensagem?*

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
