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

### 1.1 O vocabulário do cartão (o padrão novo — primeiro na lista)

Cada cartão mostra uma família do 05 §4, com valores vivos (o store guarda-os; arrastar, escrever,
clicar MUDAM o que se vê). Títulos e rótulos em inglês, pela tabela de strings:

| Cartão | O que mostra (vocabulário do `CardCtx`) |
|---|---|
| **Fields** | `Position (px)` par X·Y com marca de animação (`Cell::key`); `Rotation (°)` campo só com glifo de ícone; `Size (px)` W·H; `Name` campo de texto (`Widget::Text`) |
| **Choices** | segmentado de texto (`Widget::Choice`: Low · Mid · High); segmentos de ícone (`Widget::Icons`: Cap Butt/Round/Square); ícones nus (`Widget::Bare`); caixa de marcar à Figma (`corpo::check`, «Snap to grid»); lista suspensa (`Widget::Dropdown`: Normal · Multiply · Screen) |
| **Color** | a linha do Fill à Figma: `Widget::Swatch` com hex + `Opacity (%)` + olho/menos (`Bare`). A amostra abre o seletor de cor do app (`set_picker_target`, a cor volta pelo `widget_color` a cada quadro); o «−» tira o Fill e o cartão fica vazio com o «+» (`add`) que o devolve |
| **Actions** | `Widget::Button` primário e secundário; `Widget::Badge`; ação no cabeçalho (`HeadAction` ↺). **Apply** grava os valores de partida do vocabulário; **Cancel** e o ↺ voltam a eles; o rótulo sobre o Apply («Saved» · «Not applied») e o emblema (✓ · tracejado) dizem se há mudanças por aplicar (`src/valores.rs`) |
| **Animation marks** | os quatro estados da marca D17 lado a lado (Livre · Animado · Chave · Mudado), cada um com o rótulo dele |
| **Ordering** | cartão que nasce DOBRADO com `summary` («Z 3») — o padrão do resumo à direita |
| **Physics** | secção AUSENTE (`add: Some(id)`): título esmaecido e «+» que a acrescenta (e passa a mostrar um corpo) |

### 1.2 Os widgets de sempre (cada um num cartão, pintado com o tema do cartão)

As 11 secções de hoje (`widget/showcase/*`: Inputs, Slider, Switches, Lists, Vector, Status, Color
(swatch), Actions, Identity, Card, Inspector v2) passam a ser CORPOS de cartão: cada pintor de
secção parte-se em cabeçalho (o de hoje, só para `PH2D_CARDS=0`) + corpo (`*_body`, público), e o
cartão chama o corpo com `x`, `w`, `y` do `CardCtx` e o tema da tinta do cartão (`ctx.ink.theme`).
Ids de secção: os de hoje (`ids::INSP_SECTION_*`), para a ordem/tema em `sections.txt` sobreviverem.

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

`PH2D_GALLERY_SMOKE=1` abre a galeria no painel lateral, já visível. O dono: arrastar o ícone de um
campo (o número muda), clicar Low/Mid/High, marcar «Snap to grid», abrir a lista suspensa, dobrar
um cartão, arrastar um cartão para outra posição, botão direito no título (tema do cartão), ↺.
