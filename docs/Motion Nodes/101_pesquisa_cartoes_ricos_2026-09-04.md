# 101 — Pesquisa: CARTÕES RICOS — o upgrade do nó para superar Blender, Cavalry e Houdini

> **Ordem do Enio, 2026-09-04 (com a foto do *Distribute Points on Faces* do Blender):** *«Os nós
> nus como os nossos são chatos, mal decorados, com poucas possibilidades de conexões mais
> customizadas e complexas. Mas veja o exemplo do blender, nós ricos, cheios de inputs e outputs,
> plenos, não dependentes de painel lateral. Faça uma pesquisa de como fazer um upgrade de nossa
> implementação a ponto de superarmos blender, calvary e houdini.»*

⚠️ **Método.** Pesquisa + plano, **sem código** (o protocolo do `/pd-feature`: o plano vem antes).
Cada afirmação sobre uma referência tem a fonte ao lado; cada afirmação sobre a casa foi lida no
código hoje (`ph2d-panel-motion-graph` · `ph2d-panel-motion-params` · `ph2d-node-registry` ·
`ph2d-nodegraph`); cada número de tamanho é **derivado das constantes do `geom.rs`** e dos três
alvos de tablet que a casa já mede. Irmão do [doc 100](100_estudo_dos_outputs_2026-09-04.md)
(*onde vive o que um nó calculou*) — este é sobre **o que o CARTÃO mostra e deixa tocar**.

---

## §1 — O que a foto do Blender tem, peça a peça — e o que disso a casa já tem

| na foto | o que é | PH2D hoje |
|---|---|---|
| cabeçalho verde com chevron | cor por **categoria** + dobrar o nó | ✅ cor por categoria (`cat_token`) · ⛔ sem dobra |
| `Points` ■ · `Normal` ◆ · `Rotation` ◆ | saídas empilhadas com rótulo; a **forma** do socket diz a ESTRUTURA (valor único / campo) | ✅ a mesma lei do rótulo · ✅ ○ escalar / ◇ vector · ✅ cor por espécie (doc 99) |
| `Random ▾` | um **param de modo** como dropdown **dentro** do cartão | ⛔ vive no painel lateral (`ParamWidget::Enum`) |
| `Mesh` ■ · `Selection` ◆ | entradas com rótulo | ✅ |
| `Density 10.000` com socket ◆· | um **param como socket**: desligado mostra o slider inline; ligado, o slider some | ⛔ o param só tem socket no painel (a *row dirigida*, doc 58/88) |
| `Seed 0` | idem, inteiro | ⛔ idem |
| nada de painel lateral | o cartão basta | ⛔ o cartão mostra título + sockets + readout + moldura |

O que a casa tem e a foto **não** tem: o **selo de papel** (fonte/decisão/junção/terminal — 7
silhuetas, [doc 99 §4](99_estudo_do_mini_cavalry_2026-09-02.md)), o **readout** vivo (o número
que o nó produziu neste quadro), a **moldura** de pré-visualização (doc 86), a **recusa que nomeia
a cura**, e — por baixo — o cook no device. *O que falta não é decoração: é o cartão ser o
segundo HOST dos controlos que o painel já tem.*

---

## §2 — O estado da arte: duas famílias, e o que cada uma ABANDONOU

### 2.1 A família do cartão auto-suficiente
- **Blender (Geometry/Shader Nodes)** — *«The node parts include the title, sockets, properties»*;
  o param É um socket com o seu widget inline; desligado mostra o valor, ligado esconde-o
  ([manual](https://docs.blender.org/manual/en/latest/interface/controls/nodes/parts.html)).
  Para conter o crescimento: **`Ctrl-H`** esconde sockets não usados e **`H`** dobra o nó
  (mesma fonte); **painéis dobráveis dentro do nó** (4.x: *«Node panels can be used to group
  sockets in a collapsible section … added to node groups and to built-in nodes»*, com
  *«boolean sockets as checkboxes in their headers»* no 4.5 —
  [release notes](https://developer.blender.org/docs/release_notes/4.5/geometry_nodes/),
  [Blender Studio](https://studio.blender.org/blog/new-geometry-nodes-features-in-blender-42/)).
  E o **5.0 re-significou a FORMA do socket**: *«the socket shape indicates the higher level
  structure of the data, whether it is a single value, field, list, or volume grid»*
  ([devblog](https://code.blender.org/2025/08/new-socket-shapes/)).
- **Unreal Blueprints** — pinos com valor por omissão inline; **`Promote to Variable`** num pino
  ([docs](https://dev.epicgames.com/documentation/en-us/unreal-engine/blueprint-variables-in-unreal-engine));
  **nós compactos** (`ShouldDrawCompact` — o `+` desenha-se como um `+`,
  [unrealist](https://unrealist.org/custom-blueprint-nodes/)).
- **Unity Shader Graph** — *«Collapse … will hide all unconnected Ports»*; pré-visualização no
  nó, dobrável ([docs](https://docs.unity3d.com/Packages/com.unity.shadergraph@14.0/manual/Node.html)).
- **Grasshopper** — o **ZUI**: *«zooming in until the Insert ⊕ and Remove ⊖ controls are
  visible»* — controlos que só existem a partir de um zoom
  ([Rhino](https://developer.rhino3d.com/guides/scripting/scripting-component/)).
- **Mini Cavalry** — cartão de **220 px**, `renderUI` **escrito à mão em cada um dos 134 nós**
  (`select`, `range` com o valor ao lado, cor, texto), *slots promovidos* (um param vira socket
  por menu — *«Abrir slot»* —, um par X/Y vira **um** socket `Position`), keyframe por param no
  menu, pega de redimensionar, moldura de pré-visualização com *throttle*.

### 2.2 A família do tile + painel
- **Houdini** — o tile mostra só o **badge** e *«the most important parameter to understand
  what the node is doing — for the File node this is the filename»*; tudo o resto vive no
  *Parameter pane*; *«parameter nodes for promoted parameters … start out collapsed to keep the
  network cleaner»* ([SideFX](https://www.sidefx.com/docs/houdini/network/organize.html),
  [badges](https://houdinihelp.ru/network/badges.html)).
- **Nuke** — tile + *postage stamp* + *Properties*; e o stamp tem **preço**: *«set the postage
  stamp on nodes to be static, or disable the thumbnails in order to improve performance»*
  ([Foundry](https://support.foundry.com/hc/en-us/articles/207682435)).
- **TouchDesigner** — *«The viewer of a node can be (1) the interior of a node (the Node
  Viewer) … »* + o *Parameter Dialog* com páginas ([Derivative](https://docs.derivative.ca/Parameter_Dialog)).
- **Cavalry** — o *Attribute Editor* é a superfície principal; mas o **Dependency Graph** mostra
  *«a Layer … as a block containing a list of attributes, with the input and output connection
  for each attribute represented by a 'port'»*, cor por tipo
  ([Cavalry](https://cavalry.studio/docs/user-interface/menus/window-menu/dependency-graph/)).

### 2.3 O que foi tentado e ABANDONADO ou corrigido — e o que cada caso nos diz
| quem | o que | lição |
|---|---|---|
| Blender 2.8 → hoje | as **pré-visualizações no nó** saíram do core e voltaram como add-ons (*Node Preview*, *Preview-it*, *AnyNode* — [extensions](https://extensions.blender.org/add-ons/node-preview/)) | uma imagem por nó por quadro é um orçamento; a nossa moldura já é **só** para nós posicionais |
| Blender 3.0 | os nós de atributo **por NOME** (*Attribute Math* etc.) foram substituídos por **fields** com socket ◆ — *«shifted from addressing attributes by name to allowing attribute data to be passed around with node links»* ([devblog](https://code.blender.org/2021/08/attributes-and-fields/)) | ⚠️ o nosso `value.attribute` é o padrão 2.93; o socket-que-carrega-o-campo é o 3.0 — é o pino no cartão, não o texto no painel |
| Blender 5.0 | a **forma** do socket mudou de significado ([devblog](https://code.blender.org/2025/08/new-socket-shapes/)) | forma = estrutura, cor = tipo; a casa já separa (○/◇ + 3 espécies) e pode ganhar a 3.ª forma: *param dirigível* |
| Blender | `Ctrl-H` esconde o não-usado | o cartão cheio **é o problema que o próprio Blender esconde**; o nosso default deve nascer escondido |
| Nuke | stamps **estáticos** por desempenho | a moldura tem de ter LOD/throttle — o Mini Cavalry já faz (1/5 dos quadros para os não-seleccionados) |
| Houdini | params **fora** do tile, de propósito | com centenas de params o cartão não os aguenta; a casa tem `max 24` — cabe, **com painéis e promoção** |
| Mini Cavalry | 134 `renderUI` à mão | é a **segunda fonte** em escala: cada cartão é código próprio; o nosso painel é **derivado de uma tabela** (`ParamUiHint`) — a vantagem a manter |

### 2.4 Síntese — feature a feature
| | Blender | Houdini | Cavalry | Mini Cavalry | **PH2D hoje** |
|---|---|---|---|---|---|
| params no cartão | ✅ todos | ⛔ | ⛔ (só no Dep. Graph) | ✅ à mão | ⛔ |
| param como socket | ✅ | ⛔ (promote p/ HDA) | ✅ (todo atributo) | ✅ por promoção | ✅ **por fio** (`d`), sem socket visível |
| esconder/dobrar | `Ctrl-H` / `H` / painéis | — | — | pega de redimensionar | ⛔ |
| forma do socket = estrutura | ✅ (5.0) | — | — | ✅ 7 formas | ✅ ○/◇ |
| cor = tipo/espécie | ✅ | — | ✅ | ✅ 7 | ✅ 3 espécies |
| cor de cabeçalho = categoria | ✅ | ✅ (tile) | — | ✅ | ✅ |
| silhueta = papel | ⛔ | ✅ (formas de nó) | ⛔ | ✅ 7 | ✅ 7 |
| pré-visualização no nó | add-on | ⛔ | ⛔ | ✅ | ✅ moldura (posicionais) |
| readout vivo | inspecção no hover | ⛔ (Info) | ⛔ | ✅ probes | ✅ |
| nó compacto | `H` | — | — | — | ⛔ |
| ZUI / LOD por zoom | esconde texto | — | — | — | ⛔ (há `zoom`) |
| recusa que nomeia a cura | ⛔ | ⛔ | ⛔ | ⛔ | ✅ |
| cook no device | ⛔ (CPU) | ⛔ | ⛔ | ⛔ | ✅ |

---

## §3 — A casa hoje, medida

**O cartão** (`geom.rs` · `paint.rs::draw_card`): `CARD_W = 190` · `HEADER_H = 26` · `ROW_H = 22`;
altura = cabeçalho + `max(entradas, saídas)` linhas + readout (se houver). Pinta: cabeçalho por
categoria, **selo de papel**, título elidido, glifos de socket (○/◇ × 3 espécies), **rótulos de
porta** (só com ≥ 2 saídas), readout, alternador da moldura, véu de inércia, risco de *bypass*.
Gestos (`GraphHitKind`): `Node` · `SocketIn/Out` · `Wire` · `Waypoint` · `Backdrop` ·
`PreviewToggle` · `InertBadge`. **Há `zoom`** (`ViewState::zoom`, `screen = origem + pan +
grafo × zoom`). Crate: **12 402 LOC**, tectos de 700 por ficheiro.

**O painel de params** (`ph2d-panel-motion-params`, **8 169 LOC**, dos quais **2 977** são o
renderer de rows): **derivado da tabela** `ParamUiHint` (`label · min · max · step · widget ·
unit`) + `ParamGroup` (secções dobráveis) + `ParamGate*` (esconder por modo). **13 espécies de
row**: `Scalar · Color · Toggle · Enum · Angle · Seed · Text · Curve · Gradient · Palette ·
Channels · Source · File`. Edições saem por **uma porta** (`push_param_intent`). ⚠️ O `populate`
regista um **pool estático**: `7 + 2 × MAX_ENUM_OPTIONS(48)` = **103 widgets por slot × 33
slots = 3 399** — para UM nó (o seleccionado).

**O param dirigível** (doc 58): `Graph::drive_param(node, "amplitude", (src, port))` é uma
**aresta que pousa num NOME** (registo `d`), o cook vê-a como dependência, os 134 nós leem-na pelo
mesmo funil (`EvalCtx::param = fio > override > default`) — *«a porta nunca foi o requisito»*.
Hoje o fio pousa na **row do painel** (a *row dirigida*, `PH2D_DRIVEN_ROW_SMOKE=1`, com o elo e
o nome do cartão que a dirige). ⇒ **o modelo do socket-por-param já existe no documento; o que
não existe é o socket no cartão.**

**O catálogo** (130 manifestos com params): **média 5,2 · mediana 4 · máx 24** params; 70 nós com
0–4, 39 com 5–9, 15 com 10–14, 6 com ≥ 15 (`bezier_warp` 24 · `emitter` 21 · `noise` 19 ·
`spline_wrap` 18 · `boids` 16 · `value.noise` 15). Entradas: média 1,6 · máx 5.

---

## §4 — A régua que decide o desenho: o TABLET

O alvo do produto é tablet/iPad ([memória](../../project-memory/feedback_a_permanent_band_must_return_more_screen_than_it_eats.md)):
**1366×1024 · 1194×834 · 1133×744**, e as duas colunas laterais custam **612 px absolutos**
(*«a mesma decisão custa 20 % mais no aparelho pequeno»*). Um cartão à Blender — **todo param
uma linha** — com as nossas constantes (`26 + linhas × 22`, sem o pad):

| nó | linhas (params + max(in,out)) | altura | % da altura do iPad **mini** | % do 12,9" |
|---|---:|---:|---:|---:|
| mediano (4 params, 1–2 in, 1 out) | 6 | **158 px** | 21 % | 15 % |
| `motion.oscillator` (13 + 2) | 15 | **356 px** | 48 % | 35 % |
| `motion.emitter` (21 + 0 → 1) | 22 | **510 px** | 69 % | 50 % |
| `motion.bezier_warp` (24 + 2) | 26 | **598 px** | 80 % | 58 % |
| hoje — oscilador (2 in / 1 out) | 2 | **70 px** | 9 % | 7 % |
| **magro** — oscilador com 2 params dirigidos/promovidos | 4 | **114 px** | 15 % | 11 % |

⇒ **«todos os params no cartão» é inviável no alvo** (o oscilador sozinho comeria metade do
mini) — e é exactamente o que o Blender esconde com `Ctrl-H`, os painéis e o `H`. ⭐ **Mas o
cartão magro devolve MAIS do que custa:** se o cartão carrega os controlos que o artista de facto
mexe, a coluna de params pode ficar **fechada** — e fechar as duas colunas devolve **89–92 %** da
área (a maior alavanca medida no chrome). *A tese do Enio — «não dependente de painel lateral» —
é a certa para o tablet, desde que o cartão nasça magro e cresça por promoção.*

---

## §5 — O desenho: uma porta por pergunta

**D1 — Que params aparecem no cartão?** Três razões, **derivadas**, nunca uma lista à mão por nó:
(a) o param está **DIRIGIDO** (existe `d` — o fio tem de pousar em algo visível; é a 3.ª condição
do doc 58 que hoje só o painel cumpre); (b) o artista **promoveu-o** (Mini Cavalry *«Abrir
slot»* / Unreal *Promote*), e a promoção é **estado de DOCUMENTO** — registo novo `q <id>
<param>` (⚠️ `p` já é o registo de override — corrigido no doc 102), append-only, como `d` e `pos`; (c) o cartão está **EXPANDIDO** (o inverso do `H` do
Blender): então todas as rows, em **secções** que são os `ParamGroup` que já existem (o painel
dentro do nó do Blender 4.x, de graça). Fora disso o cartão é o de hoje. ⛔ Um param
`Text`/`Curve`/`Gradient`/`File` não promove (Mini Cavalry recusa o mesmo: *«não é número/cor»*).

**D2 — De onde vêm os widgets?** **Do mesmo renderer do painel.** O `rows_paint*` + `snapshot`
(2 977 LOC) sai para uma crate-folha (`ph2d-param-rows`) com dois hosts: o painel (byte-idêntico)
e o cartão. *Um renderer, dois hosts* — a mesma lei que o `sim_extract_sheet::unfolded_quad` e o
`stroke_uniform` pagaram: a segunda cópia diverge. ⚠️ O **pool estático** (3 399 widgets por nó)
**não escala a N cartões**: só o cartão **QUENTE** (hover/seleccionado) tem widgets vivos
registados; os outros pintam as rows como **glifos e texto** (é o que o Blender faz — o widget só
responde sob o rato). Tocar numa row de um cartão frio torna-o quente **nesse quadro** e o toque
seguinte é o widget — o mesmo *two-phase* da `settle` do undo.

**D3 — O socket do param.** Cada row no cartão ganha um socket à esquerda, **espécie número**
(`PortValue`), **forma nova = «dirigível»** (a 3.ª forma, o ◆· do Blender): vazio quando ninguém
o dirige, cheio quando há `d`. Largar um fio nele = **`drive_param`, a porta que já existe** — o
cartão é a segunda porta para o mesmo verbo, e a row do painel continua a ser a primeira. A recusa
é a de hoje (*só um output de valor conduz um param*, `source_can_drive`, com o toast que nomeia o
conversor).

**D4 — A altura do cartão é uma FUNÇÃO**, em `geom.rs` ao lado de `card_h`, de
`(dirigidos ∪ promovidos, expandido)` — o pintor e o hit-test leem a mesma (é a lei do doc 86:
*«a card drawn taller than it is clickable has a dead strip»*). **Gate de tablet por cartão:** o
cartão **magro** de qualquer nó do registry ≤ 25 % da altura do mini (o oscilador magro dá 15 %).

**D5 — LOD por zoom** (Grasshopper/Blender): abaixo de `z₁` só título e sockets de fluxo; entre
`z₁` e `z₂` rows como texto; acima de `z₂` widgets e as pegas de promoção (o ⊕ do Grasshopper).
⛔ Os limiares **medem-se** (W0): a legibilidade é `fonte × zoom ≥ n px`, e `n` sai da fonte
que o `TextSystem` usa, não de gosto. ⚠️ Abaixo de `z₁` as rows **não se registam** — pintar
sem registar é um controlo morto, e registar sem pintar é um alvo invisível; o gate mede os dois.

**D6 — Compacto** (Unreal): um nó de valor com uma saída e ≤ 1 param (`value.math`, `value.gain`)
desenha-se como **pílula com o glifo** e o readout; é o `H` do Blender feito default para quem
não tem o que mostrar.

**D7 — Largura**: `CARD_W = 190` fica; a pega de redimensionar (Mini Cavalry) é registo `w <id>
<px>` no documento, **opcional**, e a régua é a elisão do título (que já existe).

**D8 — Decoração que ENSINA, não que enfeita**: (i) o **param de modo** (o `Random ▾` da foto) é
o primeiro `Enum` do nó, **sempre** no cartão como subtítulo/dropdown — é o *«most important
parameter»* do Houdini, derivado (a lista dos `ParamGate::when` diz qual param governa os
outros); (ii) ícone de categoria no cabeçalho — os tokens de categoria existem, os ícones do
design system ([`docs/design`](../design/)) têm de ser medidos por categoria; (iii) o balão do
socket lista as **colunas** (doc 100 §7.4 — *Blender socket inspection*).

**D9 — Edição.** O widget no cartão emite o **mesmo `MotionParamIntent`** que o painel — uma fila,
um consumidor, o undo por diff que já existe. ⛔ Nenhum caminho novo de escrita.

---

## §6 — Contratos congelados: prova de que NADA disto os toca

`NodeManifest` (§6, `=8`) é `&'static` e **não recebe um campo**: tudo o que o cartão precisa já
mora fora dele — `NodeRegistry::register_ui` / `register_param_ui` / `register_primary_input`
(side-metadata) e o **documento** (`Graph`: `layout`, `labels`, `param_sources`, e os registos
novos `q`/`w`, append-only, `v3` só quando existem — o precedente exacto do `d`). `Tool=12` /
`PanelEvent=4` não entram: o cartão fala pela fila de intents que o painel já usa. Prova a
correr no fecho: `architecture_contract_surface` + `architecture_tool_contract_surface` verdes
sem ADR, e `grep -c "pub struct NodeManifest" -A12` inalterado.

## §7 — As quatro condições de UI (cada uma é independente)
1. **o componente EXISTE** — as 13 rows existem; o socket-de-param **não** (novo glifo em
   `paint_socket_glyph`, nova variante `GraphHitKind::ParamSocket { node, param }`);
2. **é pintado E registado** — `HitIndex` + `WidgetStore` só no cartão quente (D2); o gate
   `every_widget_file_wires_a11y` exige a linha do ficheiro novo;
3. **o clique chega ao barramento** — `MotionParamIntent` (já provado no painel) e `drive_param`
   (já provado no doc 58) — o cartão **reutiliza**, não cria;
4. **a sequência leva a algum lugar** — o cook lê o param (`EvalCtx::param`), o fio dirige-o: já
   verde. O risco desta wave está TODO na condição 2 — logo os gates são de costura real.

## §8 — Gates, red-first, e a fixtura que contém o fenómeno
Fixtura: o grafo do `PH2D_DRIVEN_ROW_SMOKE=1` (grid → scale → oscillator → move → output, com
`amplitude` dirigida por um `value.lfo` e `frequency` por `lfo → gain → map_range`) — já tem
dois params dirigidos e um por promover.
1. `a_driven_param_paints_a_socket_on_the_card_and_the_hit_rect_agrees` — geom = paint.
2. `dropping_a_value_wire_on_a_param_socket_drives_it` — costura REAL (`seam_*`), não `Click`
   sintético (o chip do Vector morreu sob o dedo com o clique sintético verde).
3. `a_promoted_param_survives_save_and_load` — ida-e-volta do registo `q`; `v7` só se existir.
4. `the_card_height_is_a_function_of_promotion_and_expansion` — a função de D4, e o tecto de
   25 % do mini sobre **todo** o registry (mutação: apagar a dobra deixa o `bezier_warp` a 80 %).
5. `below_the_lod_zoom_no_row_is_painted_and_none_is_registered` — as duas metades.
6. `the_card_builds_no_param_row_of_its_own` — censo estrutural: zero construções de `ParamRow`
   fora da crate-folha (a segunda fonte proibida por grep).
7. `a_cold_card_registers_no_widget_and_the_hot_one_registers_its_rows` — o pool não multiplica.
8. `the_mode_param_is_on_every_card_that_has_one` — D8(i), derivado dos gates do registry.

## §9 — Waves, por ordem, com o preço e o smoke
| wave | o que | porta | preço/pré-requisito | smoke |
|---|---|---|---|---|
| **W0** | MEDIR: pintar 20 cartões com rows (harness `ph2d-ui-testkit`), o `n px` de legibilidade por zoom, o custo do `HitIndex` por widget | — | 1 sonda | tabela no doc |
| **W1** | extrair o renderer de rows para `ph2d-param-rows` | um renderer, dois hosts | painel **byte-idêntico** (golden) | `PH2D_DRIVEN_ROW_SMOKE=1` igual |
| **W2** | rows dos params **DIRIGIDOS** no cartão + socket-de-param + largar fio = `drive_param` | D1(a) · D3 | W1; gates 1, 2, 6, 7 | `PH2D_CARD_SMOKE=1`: a mesma cena, o fio pousa no cartão |
| **W3** | **promoção** (menu no painel e no cartão) — registo `q` | D1(b) | W2; gate 3 | promover `phase_stagger`, gravar, abrir |
| **W4** | **expandir/dobrar** com secções (`ParamGroup`) + **compacto** | D1(c) · D6 | gate 4 | `bezier_warp` expandido |
| **W5** | **LOD por zoom** + pega de largura (`w`) | D5 · D7 | W0; gate 5 | zoom out até sumirem as rows |
| **W6** | **decoração**: param de modo no cabeçalho, ícone de categoria, forma «dirigível» | D8 | gate 8 | `motion.noise` mostra `Perlin ▾` |
| **W7** | balão do socket com as colunas | doc 100 §7.4 | `stream_at` existe | passar o rato no `Out` |

⚠️ **Cada wave imprime o custo em % nos três alvos** (memória do tablet) — e a W2 é a primeira
que tem de dizer *«com a coluna de params fechada, o artista faz X sem a abrir»*.

## §10 — Onde SUPERAMOS, e o que fica para o Enio

**Ninguém dos três tem as quatro coisas juntas:** silhueta por papel + cor por categoria + cor por
espécie + forma por estrutura — e nenhum tem o readout, a moldura, a recusa que nomeia a cura e
o cook no device. Com W2–W6 o cartão passa a ter o que o Blender tem (widgets, socket-por-param,
painéis, dobra) **sem** o defeito que o Blender esconde com `Ctrl-H` (o cartão nasce magro), com
a promoção do Mini Cavalry/Unreal, o ZUI do Grasshopper, o *most important parameter* do Houdini
— e o **substrato deles não tem**: o param dirigido por NOME (`d`) já permite promover **sem
mexer no manifesto**, o que nem o Blender (grupos) nem o Houdini (HDA) fazem de graça.

**Decisões do Enio (produto):** (1) o default magro (D1) contra «todos os params, como o
Blender» — os números da §4 são a régua; (2) largura fixa ou pega (D7); (3) o par X/Y como UM
socket `Position` (Mini Cavalry) — na casa os params são `f32` e um `Vec2` dirigido pede dois `d`
ou um nó divisor; (4) se a W6 traz ícones (custa uma tabela de 13 categorias no design system).

⛔ **Recusas medidas deste doc:** todos os params no cartão por omissão (§4: 48–80 % do mini) ·
`renderUI` por nó (§2.3: segunda fonte em escala) · widgets vivos em todo cartão (§3: 3 399 por
nó) · pré-visualização por nó sem LOD (Nuke/Blender pagaram).
