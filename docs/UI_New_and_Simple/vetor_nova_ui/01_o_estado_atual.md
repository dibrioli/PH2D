# 01 — O estado atual: o Vector inteiro dentro de um painel

> Medido no código e fotografado em **2026-10-07** (binário `smoke` do mesmo dia, tela virtual
> `1930 × 1040`, `docs/Components/ferramentas/fotografa_cena.sh`). Tudo o que segue tem endereço.
> ⚠️ Os docs antigos do módulo (`docs/Vector Module/12_ux_chrome.md`, de 2026-05) descrevem uma UI
> **planeada** — atalhos `P/B/R/V/A/N/K`, Zen no `Tab`, QuickMenu — e **quase nada disso existe**
> (§4). Este ficheiro só afirma o que o código faz.

## §1 — As fotos

| foto | o que mostra |
|---|---|
| [`fotos/antes_01_vetor_em_edit_painel_direito.png`](fotos/antes_01_vetor_em_edit_painel_direito.png) | `PH2D_OBJECT_MODE_SMOKE=6`: uma forma em **Edit Mode**, ferramenta Select, o painel **Vector** aberto à direita |
| [`fotos/antes_02_cena_vetorial_layout_draw.png`](fotos/antes_02_cena_vetorial_layout_draw.png) | `PH2D_BUILD_SMOKE=48`: a cena da booleana viva, painel Vector fechado — o chrome que sobra |
| [`fotos/precedente_01_quadro_barra_curta_e_barra_de_estilo.png`](fotos/precedente_01_quadro_barra_curta_e_barra_de_estilo.png) | `PH2D_BOARD_SMOKE=2`: o **Quadro** (MiroClone) — barra curta de ícones à esquerda da área e barra de estilo flutuando sobre a seleção. ⭐ **É o precedente interno** do que este estudo propõe |

## §2 — A janela, região por região (foto `antes_01`)

| região | o que tem hoje com o Vector activo | serve ao vector? |
|---|---|---|
| **Barra do topo** (menu) | `File · Edit · View · Window · Run` · abas de documento (`Scene`, `+`) · abas de **layout** à direita (`Draw · Vector · Flip · Animate · Nodes`) | só o menu genérico |
| **Fila de ferramentas** (por cima da área, [`tool_bar.rs`](../../../crates/ph2d-editor-core/src/screens/hero/tool_bar.rs)) | `MOVE · ROT · SCALE · PIVOT` · `SPACE Global` · `VIEW Selected` · `UNDO · REDO` · chip de modo `Edit Mode ▾` | ⛔ **não**: os chips são do gizmo de OBJECTO e continuam lá em Edit, com a caneta na mão |
| **Coluna esquerda** | Hierarquia (lista de `Path N`) | sim (camadas) |
| **Área de desenho** | régua em cima e à esquerda, grelha, gizmo de seleção | sim |
| **Coluna direita** | abas `Inspector | Vector`; o **Vector** é TODO o módulo | ⚠️ é aqui que tudo mora |
| **Pílula de baixo** ([`bottom_hud.rs`](../../../crates/ph2d-editor-core/src/screens/hero/bottom_hud.rs)) | `EDIT · fps · ms · stall · ev/stamp · draws · sprites · ent · bodies · MB · zoom · cena` | ⛔ estatística de motor; nada sobre o desenho (nem dica da ferramenta, nem coordenadas, nem encaixe) |

## §3 — O painel Vector: 41 secções numa coluna

Fonte da ordem: [`paint_body_plan.rs`](../../../crates/ph2d-panel-vector/src/paint_body_plan.rs) (`fn corpo`, 41
`p.seccao(...)`); a lei de visibilidade: [`section_scope.rs`](../../../crates/ph2d-panel-vector/src/section_scope.rs).

Pela ordem em que abre: **Tool** · Shape · Pencil · Symmetry · Frame · Clip · Layout · Anchors ·
Component · Widget · States · Shape Params · Connector · Text · Font · Paragraph · Text on Path ·
Axes · **Stroke** · Texture Pattern (traço) · Brush · **Fill** · Fill Type · Texture Pattern
(preenchimento) · Snap · Transform · Appearance · Vertex · Boolean · Expand · Blend · Morph ·
Morph States · Envelope · Pattern on Path · Contour · Effects · Filters · Align · Arrange · Path.

**O que a foto prova:**

1. **As 16 ferramentas são botões de TEXTO** numa grelha de 5 linhas dentro da secção *Tool*
   (`Select Node Pen / Pencil Shape Text / Connect Build Fillet / Chamfer Width Cut / Trim Bucket
   Frame`). Não há ícone, não há agrupamento por família, e a troca de ferramenta exige ir ao
   painel. Os 16 são o `DrawMode` ([`params_mode.rs`](../../../crates/ph2d-tool-vector/src/params_mode.rs)).
2. **O primeiro controlo de ESTILO (`Stroke`) aparece a `y ≈ 935` de `1040`**, abaixo de Clip,
   Layout, Prefab, Widget Skin e States (com 4 linhas `Rec` e 11 curvas de easing). O `Fill` está
   abaixo da dobra. Num editor vetorial, cor de preenchimento e de traço são **os** controlos mais
   usados (todo concorrente os põe à vista — ver [`02_estudo`](02_estudo_os_apps_amados.md)).
3. **A fila de ferramentas no topo mostra o gizmo de objecto em Edit Mode.** Em Edit, `MOVE/ROT/
   SCALE` não falam da caneta nem dos nós; o espaço mais caro da tela fala de outro modo.
4. **A pílula de baixo é de motor**, não de desenho.

### ⭐ A classificação que já existe — e que é o mapa das barras

O `section_scope.rs` já divide as secções em **três assuntos** (report do Enio de 2026-08-31,
*«deixar no painel apenas o que é útil para a ferramenta em uso»*):

| assunto | exemplos | onde os apps amados o põem |
|---|---|---|
| **o GESTO** (o que a ferramenta na mão vai produzir) | Shape, Pencil, Symmetry, opções de Pen/Node/Cut | **barra contextual no topo** (Affinity *Context Toolbar*, Graphite *Tool Options*) |
| **a SELEÇÃO** (o que já está na tela) | Stroke, Fill, Appearance, Transform, Text, Boolean, Align, Arrange, Path… | **painel de propriedades à direita** + atalhos rápidos (Affinity *Studio*, Figma *Properties*) — o mais usado também perto da seleção |
| **o DOCUMENTO** | Snap, guias, tokens | **barra de baixo / topo** (Affinity *Snapping Manager* na toolbar; Penpot/Inkscape barra de encaixe) |

⇒ **A distribuição não precisa de ser inventada:** a casa já tem a pergunta certa por secção, só
que a resposta é sempre «a mesma coluna».

## §4 — Teclado: o que existe de verdade

Medido em [`keyboard_cadeia.rs`](../../../shells/desktop/src/input_dispatch/keyboard_cadeia.rs):

| tecla | faz | onde |
|---|---|---|
| `U` `I` `D` `X` | booleana Union/Intersect/Subtract/Exclude das duas últimas formas | `:343-346` |
| `T` | entra/sai do modo Text | `:365` |
| `Delete`/`Backspace` | apaga nó ou forma | `:356` |
| `]` `[` · `Ctrl+A` | anda pelos nós · todos os nós (só em Node) | `:96-122` |
| `Ctrl+C/X/V/D/G` | copiar/cortar/colar/duplicar/agrupar | `:180-250` |
| `Tab` · `Ctrl+Tab` | modo (Object ↔ Edit) · lista de modos | spec/06 |

⛔ **Não existe tecla para trocar de ferramenta** — nem `V` (Select), nem `A` (Node), nem `P`
(Pen), nem `M`/`R` (formas). Medido: `VECTOR_MODE_PEN` só é lido por cliques de painel
(`tool_panel_event.rs:69`, `event_clicks.rs`, `populate_modes.rs`). Todos os apps do estudo têm
estas teclas de uma letra, e são quase as mesmas em todos (Affinity, Illustrator, Figma).

## §5 — O que JÁ temos e encaixa (o trabalho é realojar, não construir)

| peça | onde | o que dá a esta obra |
|---|---|---|
| **Barra curta de ícones + barra de estilo flutuante** do Quadro | [`board_bar.rs`](../../../crates/ph2d-editor-core/src/screens/hero/board_bar.rs) (+ `board_bar_pen/notes/look`) | o padrão Figma/Miro já implementado **na nossa casa**, com ids derivados de UMA tabela e ícone = contorno da forma |
| **Trilho com «cara» por modo e flyout** | [`left_rail.rs`](../../../crates/ph2d-editor-core/src/screens/hero/left_rail.rs) — a cara Painter troca o bloco de transform pelas ferramentas de pintura, e *Shapes* abre um flyout | o modelo exacto de «toolbar esquerda que muda com o modo» |
| **Fila de ferramentas** como região da área | [`tool_bar.rs`](../../../crates/ph2d-editor-core/src/screens/hero/tool_bar.rs) | o sítio da **barra contextual** — a faixa já é paga (D2) |
| **Chip de área** (`Edit Mode ▾`, pulldowns) | `mode_drive.rs`, spec/06 | modo + menus do módulo, custo de altura zero |
| **Menu radial** | [`radial.rs`](../../../crates/ph2d-editor-core/src/screens/hero/radial.rs) — projeção das ferramentas, 8 sectores | o *Quick Menu* (Procreate/Affinity iPad) |
| **Paleta global** `Ctrl+K` | `global_palette.rs` | a porta para tudo o que não cabe |
| **A lei por secção** | `section_scope.rs` | a tabela de destino (§3) |
| **Pílula de baixo** | `bottom_hud.rs` | a casa da **dica da ferramenta** e do estado de encaixe |

## §6 — As travas que esta obra herda (não re-litigar sem facto novo)

- **D1–D7** ([`00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md)): painéis ancorados com
  flutuação DECLARADA · encaixes FIXOS · a régua e o trilho são regiões da ÁREA · **Vetor é um
  layout próprio** · o modo é do TIPO do objecto · e *«alguns objetos têm modo de edição próprio
  como vector, cujas tools são completamente específicas e onde toda a tela vai mudar»* (Enio,
  30/08 — **é a autorização deste estudo, escrita há cinco semanas**).
- ⛔ **A faixa nova de 28 px foi RECUSADA** (Enio, 31/08: *«o app tem tablets e iPad como alvo. Não
  podemos ir perdendo espaço.»*). Toda barra deste plano tem de caber **onde já se paga altura**
  (a fila de ferramentas, a pílula de baixo) ou **ser sobreposta e recolhível** (a barra curta do
  Quadro). Orçamento medido: [`medicoes/06`](../medicoes/06_o_orcamento_de_ecra_em_tablet.md) —
  e o orçamento de chips de área é **3**.
- **A shell só encolhe** (`the_shell_only_shrinks`): o código destas barras vive em
  `crates/ph2d-app-vec` / `ph2d-panel-vector`, nunca em `shells/desktop`.
- **HR-15**: zero hex, zero `f32` de UI, zero texto cru — tokens e `ph2d-i18n`.
- **Contrato `Tool=12`** congelado (ADR-0040): as barras são **vistas** sobre o `DrawMode` e os
  ids que já existem — *um comando com dois ids tem dois sítios a apodrecer*.
- ⭐ **«Antes de dar casa nova a um comando, procure o controlo que já a tem»** (D2, 01/09): os
  chips `MOVE/ROT/SCALE` estavam mortos e o remédio foi LIGÁ-LOS, não construir outros.
