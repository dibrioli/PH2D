# 08 — A Widget Gallery em cartões (linha `line/NovaUI`, 03/11)

> Missão do dono (03/11): **reconstruir o painel Widget Gallery na UI de cartões**, o mais perto
> possível do estado da arte, sozinho até ao fim, smoke só no fim. Pedido do dono a meio (03/11):
> *«para testes faça o widget Gallery abrir no painel lateral como um inspector»*.
> A lei da UI é o [05](05_ESPECIFICACAO_UI_COMPLETA.md); a regra de referência é o 05 §0 (nós →
> Figma → Rive → Inkscape/Godot/Unity).

## 0 — O que já está feito nesta linha (commits)

- `79fa5f49b1` docs/CartoesUI trazidos da linha de teste.
- `049989b94a` porte da fundação (famílias 2 e 4 do 06 §3): `ph2d-card-stack`, `ph2d-card-rope`,
  tokens `Card*`/`Kanban*` + `derive::card_tone`, i18n `cartoes.rs`, o núcleo `ph2d-editor-core`
  (`widget/card_ink.rs`, `widget/card_field.rs`, `panel/cards*.rs`, `card_*_ops`, despacho,
  `property_row` escolha/marca).
- `5ee70929c1` + `a243949e11` cortes por responsabilidade (tecto 700) e os 28 SVG `card-*`.
- `7439627882` a fundação volta a ser um DAG (tabela injectada `property_row::cartao::CardRows`,
  `CardFieldSpec` em `widget::card_field`, `CardInk::of_card(Option<Theme>, Theme)`, a lista de
  estilos publica as linhas no store, `publish_object_mode` sai — só o Inspector em cartões o lia).

Régua do `main` (merge-base `3a3896a7ab`), medida antes do porte: **todos os portões verdes**
(LOC, DAG, showcase, a11y, cores/px literais, wiring, z-walk, roda, shell `188 892 / 189 041`).

## 1 — O desenho

**A galeria é uma LISTA DE CARTÕES no painel lateral, como o Inspector em cartões** (fotos
aprovadas: `maquete/c4_maquete_x_app.png`, `b7_duas_copias.png`). Ela pinta com
`ph2d_editor_core::panel::cards::paint_card_list_with(ctx, GAL_PANEL, title, &cards, footer)` no
`ctx.slot` do dock (lado direito, `DEFAULT_SLOT = RightTop`), NÃO numa janela flutuante própria.
`PH2D_CARDS=0` volta à galeria de hoje (a janela flutuante) — o mesmo interruptor do Inspector/Bones.

A galeria é a **fonte de verdade da UI** (DIRETRIZ §5.2): todo painel novo copia o padrão que ela
mostra. Por isso ela tem QUATRO partes, por esta ordem: as ferramentas fictícias (§1.1), os casos reais
(§1.3), as amostras do Rive (§1.4) e os widgets de sempre (§1.2).

### 1.1 O inspector de ferramentas fictícias (o padrão novo — primeiro na lista)

Ordem do dono (03/11): *«a arrumação dos widgets num estilo Figma também é importante. Na imagem temos
o inspector para uma Shape de vector na linha de testes da UI. Veja os ícones para operações como flip
e outras. Decore o painel todo com tools fictícias usando figma como referência assim como o inspector
da linha de testes»*. A galeria deixa de ser catálogo seco («Fields», «Choices»): é o Inspector de um
objeto fictício («Player A»), com o desenho aprovado (`maquete/c4_maquete_x_app.png`, metade direita;
referência 05 §0: nós → Figma → Rive). Fotos: `maquete/g1_galeria_topo.png` e
`maquete/g1b_galeria_ferramentas.png`.

**Tudo vive (zero no-op silencioso), por TABELA:** `src/controlos.rs` + `controlos_forma.rs` +
`controlos_tinta.rs` dão uma linha a cada botão/ícone/segmento/opção (`Kind`: `Toggle` · `Eye` · `Pick` ·
`Box` · `Act` · `Add` · `AddOf` · `Expand` · `Remove` · `Set` · `Swatch` · `Styles` · `Palette`, com ícone, texto e dica); o `populate` regista por
ela, o `prepare` dá a dica e declara as listas, a pintura tira dela ícone e aceso, o `valores.rs` age por
ela. Os campos são a tabela `ids::FIELDS`; as escolhas moram em `Choices` (`Flag`/`Pick`), os números no
store. O gate `nenhum_controlo_da_tabela_e_um_no_op` clica a tabela inteira. As linhas que não cabem na
coluna estreita partem-se à Figma (`caixa_para`, `CAMPO_MIN`): nada se corta (`nenhum_rotulo`, declarado 0).

| Cartão | O que mostra | O que faz |
|---|---|---|
| *(sem título)* | `[tipo] [nome «Player A»] [olho] [cadeado]` | olho e cadeado alternam e acendem |
| **Align** | ícones nus: esquerda · centro H · direita · topo · centro V · base · distribuir H · V | põem a Position na borda ou no centro de uma moldura fictícia (400×300); distribuir iguala os vãos entre dois vizinhos fictícios |
| **Boolean** | segmentos Union · Subtract · Intersect · Exclude + Flatten e Convert nus | o segmento acende; Flatten larga o modo; Convert acende |
| **Transform** (↺) | `Position (px)` X·Y · `Rotation (°)` + girar 90°, espelhar H e V · `Scale` X·Y + corrente · `Skew (°)` X·Y | girar soma 90 (360 → 0); espelhar nega o Scale e acende pelo SINAL do número; a corrente faz um eixo levar o outro na proporção de antes; ↺ repõe só o transform |
| **Layout** | `Direction` (linha, coluna, quebra) · `Size (px)` W·H · `Resizing` Fixed/Hug/Fill ×2 · `Align` grelha 3×3 · `Gap (px)` · `Padding (px)` H·V | segmentos e listas escolhem. A grelha compõe-se com a `row` pública: três `Widget::Icons` de 3 numa coluna estreitada (`c.x`/`c.w`), as folgas na outra |
| **Corners** | `Radius (px)` + cada canto · `Clip content` | «cada canto» acende e mostra os quatro campos; a caixa marca (`corpo::check`) |
| **Constraints** | `Horizontal` · `Vertical` lado a lado | as listas escolhem |
| **Appearance** (olho) | `Opacity (%)` + `Blend` | o olho é o MESMO id do da identidade (como no Inspector aprovado) |
| **Fill** (⠿, +) | `Color` amostra + hex · `Opacity (%)` · olho · − | até 3 linhas; a amostra abre o seletor de cor do app (a cor volta pelo `widget_color` a cada quadro); «−» tira a linha, sem linhas o cartão fica ausente com «+»; com a lista cheia o «+» some. O ⠿ abre a lista REAL `panel::cards::estilos` (a galeria faz o papel da cena do Vetor em `estilos_ficticios.rs`: publica enquanto a lista dela está aberta, atende os pedidos): escolher pinta a amostra, ela mostra o nome e o «soltar» |
| **Stroke** (⠿, +) | a mesma linha · `Position` + `Weight (px)` · `Cap` + `Join` (3 ícones cada) | idem |
| **Effects** (+) | a linha `[glifo Drop shadow] 👁 −`; **as propriedades abrem À ESQUERDA da coluna** (decisão F43 do dono, `project_vector_effects_filters_figma_layout_2026_11_01.md`): `[tipo ▾] ✕` · `Offset (px)` X·Y + `Color` · `Blur (px)` + `Spread (px)` | o «+» abre a lista dos tipos e a opção acrescenta a linha já com esse tipo; «−» tira a dela; clicar a linha abre e fecha o painel do lado (`lado.rs`, a geometria do `side_panel` do Vetor, que é crate inalcançável daqui); os dois desfoques só mostram o `Blur` |
| **Text** | `Font` · `Weight` + `Size (px)` · `Line height (px)` + `Letter spacing (%)` · `Alignment` | listas e segmentos escolhem |
| **Export** | `Scale` · `Suffix` · `Format` · botão primário `Export` | o Export acende «Exported»; mudar escala ou formato volta a «Not exported yet» |
| **Actions** (↺) | Apply primário, Cancel, emblema | **Apply** grava os valores de partida; **Cancel** e o ↺ voltam a eles; o rótulo e o emblema dizem se há mudanças por aplicar (`src/valores.rs`) |
| **Animation marks** | os quatro estados da marca D17 | Livre · Animado · Chave · Mudado |
| **Ordering** | nasce DOBRADO com `summary` («Z 3») | o padrão do resumo à direita |
| **Physics** | secção AUSENTE (`add: Some(id)`) | «+» acrescenta e mostra o corpo; «−» no cabeçalho tira |

As marcas de animação: todo rótulo de propriedade animável leva a marca, Mass e Z order incluídos. **O Export não leva**: Scale, Suffix e Format são configuração de saída, não propriedade do objeto, e um ponto ali diria «isto anima».

### 1.2 Os widgets de sempre (cada um num cartão, com nome de ferramenta)

As 11 secções de hoje (`widget/showcase/*`) passam a ser CORPOS de cartão: cada pintor de secção
parte-se em cabeçalho (o de hoje, só para `PH2D_CARDS=0`) + corpo (`*_body`, público), e o cartão chama o
corpo com `x`, `w`, `y` do `CardCtx` e o tema da tinta do cartão (`ctx.ink.theme`). Ids de secção: os de
hoje (`ids::INSP_SECTION_*`), para a ordem/tema em `sections.txt` sobreviverem. O título é de ferramenta
(`cards.rs::TOOL_TITLES`); **quem procura um widget acha-o por esta tabela**:

| Widget (secção de `widget/showcase`) | Título na galeria |
|---|---|
| Inputs | **Layer info** |
| Slider | **Motion** |
| Switches | **Preferences** |
| Lists | **Browser** |
| Vector | **Pivot** |
| Status | **Render** |
| Color (swatch) | **Tint** |
| Actions | **History** |
| Identity | **Collaborators** |
| Card | **Comments** |
| Inspector v2 (W6) | **Advanced** |

E o vocabulário do cartão (`CardCtx`): campo com glifo → Transform; par com marca → Position;
`Widget::Choice` de ícones → Boolean, Cap, Join; `Widget::Icons` → grelha do Layout; `Widget::Bare` → Align,
girar/espelhar; `Widget::Dropdown` → Blend, Constraints; `Widget::Swatch` → Fill; `Widget::Text` → nome,
Suffix; `Widget::Badge` → identidade; `Widget::Button` → Export, Apply/Cancel; `corpo::check` → Clip content;
`HeadAction` → ↺, olho, ⠿, «+»; cartão ausente → Physics; dobrado com resumo → Ordering.

### 1.3 Os casos reais (depois das ferramentas)

Ordem do dono (03/11): *«Coloque a UI rica com representantes de casos reais»*. Cada cartão é uma família
deste app com o vocabulário do código (`component.field.*`, `card.bones.*`, `panel.inspector.*`,
`canal.*`: a chave real, para o rótulo seguir o painel real) e as unidades no rótulo; quem constrói a família
copia este cartão. Foto: `maquete/g4_galeria_casos_reais.png`, `g4b_…`, `g4c_…`. Código: `cards_reais_mundo.rs`, `cards_reais_anim.rs`,
`cards_reais_media.rs`, tabela `controlos_reais.rs`, ids `ids_reais.rs`, textos `ph2d-i18n/src/galeria_reais.rs`.

| Cartão | Lei (decisão do dono) | O que vive |
|---|---|---|
| **Image** | «Use» logo abaixo de Image, nunca checkbox (26/10) | Still → `Frame`; Repeated → `Width`/`Height (px)`; Animated → `Columns`, `Rows`, `Frames/s` |
| **Physics Body** | o nome real do cartão | `Body` Dynamic/Kinematic/Static: Dynamic → `Mass (kg)`, `Gravity Scale`, `Linear/Angular Damping`, `Lock Rotation`; Kinematic → só o Lock; Static → a frase |
| **Collider** | ícones de forma | Box → `Width`/`Height`; Circle → `Radius`; Capsule → `Radius` + `Half height`; `Offset`, `Friction`, `Bounce`, `Trigger` (os nomes reais, em vez de Bounciness / Is sensor) |
| **Health** | | `Max`, `Start` (o real, em vez de Current), a barra, `Regen (/s)`, `Regen Delay (s)`, `Overheal`: sem ele o Start não passa do Max |
| **Platform Player** | só movimento (25/10) | `Speed (m/s)`, `Accel`, `Air`, `Jump Height (m)`, `Air Jumps`, `Coyote Time (s)`, `Jump Buffer (s)` |
| **Game Camera** | alvo por arrastar, escrever ou conta-gotas, NUNCA lista (02/11); os defeitos de 05 §8: título, células do cartão, marca na linha do rótulo | `Target` escreve-se e diz «Follows X» ou «No object named…»; o conta-gotas arma e o 2.º clique faz de clique no canvas; `Game Format`, `Frame`, `Dead Zone (px)`, `Damping` (o real, em vez de Follow speed), `Lookahead (px)`, `Active`, `Look Through` (estado da vista: sem marca). **Modelo para o cartão do Inspector** |
| **Animation** | UM cartão com lista de regras (25/10) | `Clip` + ▶, `Frames/s`, `Who Picks` Me/Script/Rules; Rules → `When` ▾ `Play` ▾ −, «+» até 3 |
| **Brush** | Paint type Water·Oil (01/11), Volume independente da diluição (31/10), Paper press é controlo (28/10), um só Dilution (E10) | `Preset`, `Size (px)`, `Paint type`, `Dilution`, `Volume (µm)`, `Paper press`; Water → `Re-wet`; Oil → `Body` |
| **Bones** | os nomes de `card.bones.*` | `Length (px)`, `Bend`, `Angle Limit` → `Min`/`Max (°)`, `Smart Bone`, `IK` → `Mix`, `Chain length` |
| **Constraint** | pilha; alvo sem lista | `[Look At ▾] 👁 −`, `Target`, `Influence (%)`, `Space`; sem linha, ausente com «+» |
| **Light 2D** | | `Type` Point/Ambient, `Color`, `Intensity`; Point → `Size (m)`, `Range (m)`, `Casts shadows` |
| **Audio** | o rack segue o F43 | medidor (`paint_level_meter`, o Gain move-o, o mudo zera-o), `Gain (dB)`, a lista de efeitos com «+» de tipos e `Mix (%)` + `Damp` à esquerda |
| **Pixel Art** | paleta indexada em tokens | `Foreground`/`Background` (a caixa contornada é o alvo), a troca, a grelha 8×2, `Grid size (px)`, `Pixel grid` |
| **Script** | o script mora no projeto (ADR-0188) | `Script` ▾, `Edit script`, e as propriedades que cada script expõe mudam com ele |

### 1.4 As amostras do Rive (depois dos casos reais)

Ordem do dono (03/11): *«Se encontrar layouts e widgets interessantes em Rive, traga algumas amostras para
nós também»*. A regra 05 §0 põe o Rive depois de nós e do Figma; é oráculo que se observa (CLAUDE.md §0.9): só
o conceito, conferido na documentação pública. Nenhum asset nem código do Rive; o desenho é nosso. Foto:
`maquete/g5_galeria_rive.png`. Código: `cards_rive.rs`, `controlos_rive.rs`, `ids_rive.rs`, `galeria_rive.rs`.

| Cartão | O que o Rive confirma (página) | O que vive aqui |
|---|---|---|
| **Interpolation** | Linear, Cubic (duas pegas), Hold: <https://rive.app/docs/editor/animate-mode/interpolation-easing> | três ícones; o quadro pinta a curva; `X1 Y1 X2 Y2` e as duas pegas arrastáveis escrevem nos mesmos números; Linear e Hold tiram os campos |
| **State machine inputs** | Number, Boolean, Trigger (`fire()`): <https://rive.app/docs/runtimes/web/rive-parameters>. ⚠️ As próprias docs dizem que as entradas estão a ser substituídas por data binding (<https://rive.app/docs/runtimes/android/migrating-from-legacy>): o padrão vale pelo desenho, não como rumo de produto | número com campo, caixa, `Fire` que pisca e conta; «+» com lista de tipos, «−» |
| **Trim path** | Start, End (% do comprimento), Offset, Sequential ou Synced: <https://rive.app/docs/editor/manipulating-shapes/trim-path> | `Start (%)`, `End (%)`, a barra do trecho, `Offset (°)`, `Mode`. A página não diz a unidade do Offset; o «Synced» é a palavra dela |
| **Solo** | um filho pintado de cada vez, com rádios: <https://rive.app/docs/editor/manipulating-shapes/solos> | linhas com olho, clicar uma faz dela a única |
| **Gradient** | Solid, Linear, Radial, barra com paragens, clique cria: <https://rive.app/docs/editor/fundamentals/fill-and-stroke> | ícones de tipo, barra pintada, pegas clicáveis (escolhem; a posição muda-se no `Position (%)`), «+» até 3, «−» só com mais de duas. A página não mostra campo de posição: ele é nosso |

## 2 — Regras que mordem (medidas na linha de teste)

- **Nada pinta fora da tinta do cartão**: cor por `CardInk::color`, geometria por
  `ph2d_card_stack::metrics`, texto por `ph2d_i18n` (HR-15). Os defeitos do Camera (05 §8) são
  exactamente o contrário disto.
- **Registar é obrigatório** (populate incondicional, por TABELA que o paint também percorre):
  campo = `InteractiveState::NumberInput`, segmento/botão/ícone = `Button`, dropdown = `Dropdown`.
  Cada campo é marcado antes de pintar: `store.mark_card_field(id, CardFieldSpec{step, scrub, default})`.
- **Todo ícone sem texto tem dica** (`store.set_tooltip`), senão o censo
  `every_icon_in_a_card_has_a_tooltip` reprova.
- **A altura é a do quadro anterior** (a mola leva a diferença).
- O `showcase/` não pode chamar `panel::` (DAG da fundação): a composição em cartões vive na crate
  `ph2d-panel-widget-gallery`; o `widget/showcase` só expõe os corpos.
- O portão `architecture_widget_showcase_coverage` passa a varrer também
  `crates/ph2d-panel-widget-gallery/src/` (a galeria É o showcase) — é lá que `CardField`/`CardInk`
  aparecem.
- Tectos: ficheiro de painel ≤ 600, fn ≤ 200; widget ≤ 500; `shells/desktop` não cresce.

## 3 — Smoke (no fim)

`PH2D_GALLERY_SMOKE=1` abre a galeria no painel lateral, já visível. O dono: clicar os ícones do Align
(a Position muda), girar 90° e espelhar no Transform (o número muda, o espelho acende), fechar a corrente e
arrastar o X do Scale (o Y segue), escolher um Union/Subtract, abrir uma lista suspensa, «+» e «−» em Effects,
Export (acende «Exported»), dobrar um cartão, arrastar um cartão para outra posição, botão direito no título
(tema do cartão), ↺. Depois: o ⠿ do Fill (a lista de estilos), a linha do Effects (propriedades à esquerda), o
«Use» da Image, escrever na Target da Camera, «+»/«−» das regras, Water/Oil, Fire, os filhos do Solo, a pega da
curva.

Fotos em `maquete/`: `g1_galeria_topo` · `g1b_galeria_ferramentas` (rolagem 1460, com `PH2D_GALLERY_SMOKE_SIDE=1`: a linha do efeito aberta) · `g4_galeria_casos_reais` (2700) · `g4b_…` (4250) · `g4c_…` (5100) · `g5_galeria_rive` (5950) · `g2_galeria_meio` (6950) · `g2b_galeria_meio` (7700) · `g3_galeria_fim` (9400), com
`PH2D_GALLERY_SMOKE_SCROLL=<px>` a segurar a rolagem (o evento sintético não chega à sessão virtual).
Os corpos de sempre pintam com a tinta do cartão publicada (`card_ink::com`): a janela flutuante
(`PH2D_CARDS=0`) não publica nada e não muda. Dentro do corpo os de sempre seguem a lei do
vocabulário: `row_gap()`/`field_h()`/`encaixe::fila_h` respondem `FIELD_ROW_GAP`/`FIELD_H`, rótulo em
cima `LABEL_TOP`/`LABEL_PX`/`LABEL_BOTTOM` na tinta `CardLabel`, amostras com nome de ferramenta
(`chrome.sample.*`). O editor rico só mostra moldura e cursor com o foco do `store`.
