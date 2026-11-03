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
mostra. Por isso ela tem DUAS partes:

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
`Box` · `Act` · `Add` · `Remove` · `Set` · `Swatch`, com ícone, texto e dica); o `populate` regista por
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
| **Fill** (⠿, +) | `Color` amostra + hex · `Opacity (%)` · olho · − | até 3 linhas; a amostra abre o seletor de cor do app (a cor volta pelo `widget_color` a cada quadro); «−» tira a linha, sem linhas o cartão fica ausente com «+»; com a lista cheia o «+» some |
| **Stroke** (⠿, +) | a mesma linha · `Position` + `Weight (px)` · `Cap` + `Join` (3 ícones cada) | idem |
| **Effects** (+) | `Drop shadow ▾` · olho · − · `Offset (px)` X·Y + `Color` · `Blur (px)` + `Spread (px)` | «+»/«−» acrescentam e tiram linhas; os dois desfoques (`Layer blur`, `Background blur`) só mostram o `Blur` |
| **Text** | `Font` · `Weight` + `Size (px)` · `Line height (px)` + `Letter spacing (%)` · `Alignment` | listas e segmentos escolhem |
| **Export** | `Scale` · `Suffix` · `Format` · botão primário `Export` | o Export acende «Exported»; mudar escala ou formato volta a «Not exported yet» |
| **Actions** (↺) | Apply primário, Cancel, emblema | **Apply** grava os valores de partida; **Cancel** e o ↺ voltam a eles; o rótulo e o emblema dizem se há mudanças por aplicar (`src/valores.rs`) |
| **Animation marks** | os quatro estados da marca D17 | Livre · Animado · Chave · Mudado |
| **Ordering** | nasce DOBRADO com `summary` («Z 3») | o padrão do resumo à direita |
| **Physics** | secção AUSENTE (`add: Some(id)`) | «+» acrescenta e mostra o corpo; «−» no cabeçalho tira |

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
(tema do cartão), ↺.

Fotos: `g1_galeria_topo` · `g1b_galeria_ferramentas` (rolagem 1460) · `g2_galeria_meio` · `g2b_galeria_meio` · `g3_galeria_fim` (em `maquete/`; g2..g3 são de antes das ferramentas), com
`PH2D_GALLERY_SMOKE_SCROLL=<px>` a segurar a rolagem (o evento sintético não chega à sessão virtual).
Os corpos de sempre pintam com a tinta do cartão publicada (`card_ink::com`): a janela flutuante
(`PH2D_CARDS=0`) não publica nada e não muda. Dentro do corpo os de sempre seguem a lei do
vocabulário: `row_gap()`/`field_h()`/`encaixe::fila_h` respondem `FIELD_ROW_GAP`/`FIELD_H`, rótulo em
cima `LABEL_TOP`/`LABEL_PX`/`LABEL_BOTTOM` na tinta `CardLabel`, amostras com nome de ferramenta
(`chrome.sample.*`). O editor rico só mostra moldura e cursor com o foco do `store`.
