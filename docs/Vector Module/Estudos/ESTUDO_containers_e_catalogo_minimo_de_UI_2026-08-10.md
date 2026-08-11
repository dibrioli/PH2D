# Estudo — contêineres organizadores e o catálogo MÍNIMO de UI

> **Pedido do Enio (2026-08-10):** *"senti falta de estruturas organizadoras como VBoxContainer,
> HBoxContainer e Grid. Faça um estudo para descobrir se são essenciais ou dispensáveis. Faça um
> estudo em Godot e todos os seus objetos de UI e descubra os essenciais/MustHave […] Descarte os
> não essenciais, vamos manter as coisas simples. Tome como base mais o Figma que a Godot. Descubra
> o que nos falta para criar UI/UX extraordinárias."*
>
> Documento de **MEDIÇÃO e TRIAGEM**, não de plano. Todo número foi contado nesta worktree em
> 2026-08-10, com o comando ao lado, **máquina calma** (`load 0,44` — a linha `poca:` do Painter
> ensinou que log de máquina saturada não fala sobre o código). O censo da Godot vem da
> documentação **oficial**, não de memória.

---

## §0 — A resposta em cinco linhas

1. **VBox e HBox já existem** — chamam-se `LayoutDir::Row` / `Column` / `RowWrap`. Não falta
   capacidade nenhuma; o que a Godot resolve com **três classes**, o Figma (e nós) resolve com
   **uma propriedade**. ⇒ **dispensáveis como classes, presentes como comportamento.**
2. **Grid não é essencial** — mas ⚠️ **o argumento que o mantinha fora MORREU**: o ADR-0153 recusa-o
   dizendo que ele *"triplica o custo de build"*, e re-medido hoje ele custa **+11 ms (1,07×)**, não
   3,15×. O que o mantém fora agora é **ganho**, não custo, e essa é uma frase diferente (§2.2).
3. ⚠️ **O que falta de verdade NÃO é um contêiner: é o SIZING.** O Figma tem
   **Fixed · Hug · Fill + min/max**; nós temos **Fixed e Fill**. **Hug não existe** — e medido, ele
   sai **de graça** do motor que já shipa (§2.3).
4. **Da Godot, 40+ classes de UI colapsam em 9 MustHave**, e nós já temos 7 (§3, §4).
5. **Para UI/UX extraordinária faltam TRÊS coisas, e nenhuma delas é um widget novo** (§5).

---

## §1 — O que foi medido (e como reproduzir)

| medição | resultado | comando |
|---|---|---|
| catálogo de widgets vestíveis | **20** | `WidgetKind::ALL` (`ph2d-editor-core/src/widget/skin/kind.rs:76`) |
| cobertura da UI real do app | **~100%** de 438 construções em 23 painéis | `LEVANTAMENTO_…_2026-08-08.md` §6.3 |
| direções de fluxo | **3** (`Row` · `Column` · `RowWrap`) | `ph2d-ecs/src/vec_layout.rs:41` |
| controles do painel de layout | **25 ids** (dir · gap · pad · align · justify · grow · shrink) | `ids/chrome/vector_layout.rs` |
| **hug (`size: auto`) funciona sem measure function?** | ✅ **42,000 × 10,000 exactos** | `cargo test -p ph2d-vec-layout --release hug_probe -- --ignored --nocapture` |
| hug num eixo só | ✅ **100,000 × 12,000** | idem |
| `min` clampa o hug | ✅ **30,000** | idem |
| custo de build do `grid` do taffy | **173-180 ms → 191-192 ms** = **+11 ms, 1,07×** | A/B costas-com-costas, `cargo clean` entre corridas |
| grades de coluna fixa nos painéis reais | **2 em 23** (`GRID_COLS=3` no catálogo do vector · `TOOL_COLS=3` no audio) | `grep -rn "COLS" crates/ph2d-panel-*/src/*.rs` |
| custo do passe de layout | **120 ns/nó**, 1000 nós = **0,7%** de um quadro | ADR-0153 §Decisão 3 |

⚠️ **A sonda do hug é NOVA e fica no repo** (`crates/ph2d-vec-layout/src/hug_probe.rs`,
`#[ignore]`), pela lição que esta linha já pagou: *um número citado que a sonda não imprime mais
deixou de ser reproduzível*.

---

## §2 — Contêineres: o veredito, com o mecanismo

### 2.1 VBox / HBox — a pergunta está respondida, e a resposta é o Figma

A Godot tem `VBoxContainer` e `HBoxContainer` como **classes distintas** porque a árvore de nós dela
não conhece *"propriedade de contêiner"*: cada disposição é um tipo, e trocar de vertical para
horizontal é **trocar o nó** (e reparentar os filhos). O Figma unificou-os num só objeto com uma
propriedade de direção — e é esse o modelo que já shipa aqui:

| Godot | nosso | onde |
|---|---|---|
| `VBoxContainer` | `LayoutDir::Column` | `vec_layout.rs:46` |
| `HBoxContainer` | `LayoutDir::Row` | `vec_layout.rs:44` |
| `HFlowContainer` | `LayoutDir::RowWrap` | `vec_layout.rs:48` |

⇒ **Nada a construir.** Se a falta foi sentida na *tela*, ela é de **nome/descobribilidade**, não de
motor: os chips existem (`VECTOR_LAYOUT_DIR_ROW` / `_COL` / `_WRAP` / `_OFF`) e o painel os oferece.
Vale confirmar no smoke se os rótulos dizem o que o artista procura.

⚠️ **`VFlowContainer` (coluna que quebra) não existe e é dispensável:** uma coluna que transborda
para uma segunda coluna é layout de jornal, não de UI — e nem o Figma o oferece como direção.

### 2.2 Grid — o argumento morto e o argumento vivo

O ADR-0153 recusa o grid com **dois** argumentos, e eles têm forças muito diferentes hoje:

| argumento do ADR | estado em 2026-08-10 |
|---|---|
| *"triplica o custo de build (0,20 → 0,63 s)"* | ✅ **A RAZÃO SOBREVIVE e é a pergunta ERRADA** — ver a correção abaixo. |
| *"nada a jusante honraria um `dir = grid` — seria controlo morto"* | ✅ **VIVO, mas por pouco.** Há **duas** grades de coluna fixa nos 23 painéis, ambas feitas à mão. |

> ⚠️ **CORREÇÃO (a wave do item 5, no mesmo dia) — este quadro trazia `+11 ms` e o número era
> IMPOSSÍVEL.** Re-medido com a máquina calma e decomposto:
>
> | o quê | ms |
> |---|---|
> | sem `grid` (taffy + a nossa crate, cold) | 282-295 |
> | a crate `grid` sozinha | 304 |
> | taffy + a nossa, com `grid` **já quente** | 720-748 |
> | tudo cold, com `grid` | 760-1151 |
>
> A metade **MAIOR** (~440 ms) é o **módulo de grid do próprio `taffy`**, logo um A/B que limpe só a
> nossa crate e deixe o `taffy` quente **mede a nossa crate duas vezes e nada mais** — é de onde o
> `+11 ms` saiu. O número do ADR (0,20 → 0,63 s) estava **essencialmente certo**; foi a refutação
> dele que estava errada.
>
> ⇒ **Mas a RAZÃO é a pergunta errada, e é aí que o argumento de facto morre:** 3× sobre 0,28 s são
> **~0,47 s absolutos, pagos uma vez por build limpo**, contra uma corrida de CI de ~30 min
> (**0,03%**) — e o inner loop deste repo (`cargo check -p`) nunca os paga. *O ADR escreveu uma
> razão e tratou-a como orçamento.*

**O que uma grade dá que o flexbox não dá** (e é só isto):

1. **colunas alinhadas entre linhas** quando os conteúdos têm tamanhos diferentes;
2. **refluxo automático** ao inserir/remover um item, sem o artista re-agrupar nada.

**O que o cobre hoje sem grid:** uma coluna de linhas — `Column` de `Row`s, cada `Row` com filhos de
`grow = 1`. É como se fazia UI antes do CSS Grid, e é exactamente o que o `paint_catalog.rs` faz à
mão com `slot % GRID_COLS`.

⇒ **Veredito ORIGINAL: DISPENSÁVEL, não descartado** — ele não entrava naquela rodada porque o
sizing (§2.3) o precede: sem `hug`, uma grade de células que não se ajustam ao conteúdo resolve
metade do problema. ⚠️ **E quando entrar, entra com a nota do §0 do `CLAUDE.md` ao lado:** o Figma
só ganhou grid no Config 2025 e a própria comunidade dele reporta a feature **incompleta** (min/max
não suportado em contentores de grid; `hug` e `fr` chegaram depois) — *tomar o Figma como base* aqui
significa herdar uma feature que o próprio Figma ainda está a assentar.

✅ **CONSTRUÍDO (2026-08-10, a wave do item 5), e a nota acima foi honrada:** o sizing pousou
primeiro (waves 1+2), então o `Hug` já existia — e **medido**, ele SOBREVIVE a colunas `1fr` (a
moldura mede 38×22 nos três casos, idêntico ao `Fixed`), o que refuta o medo natural de o `fr`
colapsar sob `MaxContent` e a forma desaparecer. E o *track sizing* incompleto que o Figma ainda
está a assentar **não foi copiado**: as colunas são iguais, que é o que o único consumidor MEDIDO
deste repo faz à mão (`paint_catalog.rs`). Ver o §7.

### 2.3 ⭐ O achado: o que falta não é um contêiner, é o SIZING

O Figma dá **três** modos de tamanho por eixo, mais dois limites:

| Figma | nós | estado |
|---|---|---|
| **Fixed** | o tamanho da forma | ✅ (é o default, e o `VecLayoutItem` até cita *"É o default do Figma (Fixed)"*) |
| **Fill container** | `VecLayoutItem::grow` | ✅ (exposto como número de *flex-grow*, não com o nome) |
| **Hug contents** | — | ❌ **não existe** |
| **Min / Max** | — | ❌ **não existe** |

⚠️ **E a ausência do hug tem uma causa registada que a medição derruba.** O cabeçalho da
`ph2d-vec-layout` diz:

> *"Não há measure function. Uma folha entra com o tamanho que ela JÁ tem […] O que falta ao texto é
> REFLUIR a uma largura — outra wave (W2a)."*

A frase está **certa** e a conclusão implícita está **errada**: uma *measure function* é precisa para
uma **FOLHA** cujo tamanho depende do espaço oferecido (texto que reflui). Um **CONTENTOR** que se
ajusta ao conteúdo é *intrinsic sizing* puro, e o flexbox resolve-o sem perguntar nada a ninguém.

**Medido** (`hug_probe`, três casos):

| caso | esperado | medido |
|---|---|---|
| moldura `auto` · 3 filhos de 10 · gap 4 · pad 2 | 42 × 10 | **42,000 × 10,000** |
| largura **fixa** 100 + altura `auto` sobre filho de 6, pad 3+3 | 100 × 12 | **100,000 × 12,000** |
| `auto` sobre filho de 10, com `min = 30` | 30 | **30,000** |

⇒ **Hug, hug-num-eixo-só e min/max saem do motor que já shipa, sem uma linha de measure function.**
O que falta é **autoria** (o par `Auto | Fixed` por eixo) e **a fatia** (hoje a raiz é sempre
resolvida em `nodes[0].size`, a bbox do retângulo desenhado).

⚠️ **O vocabulário já existe neste repo, noutro objecto:** o texto tem
`wrap_width: Option<f64>` com o par **Width: Auto | Fixed** no painel
(`ids/chrome/vector_text.rs:42`). *O texto sabe abraçar o próprio conteúdo; a moldura não.*

### 2.4 A segunda falta: **Absolute position**

Medido em `shells/desktop/src/layout_live.rs:340` — a fatia percorre `Children` e **todo** filho
vira nó do fluxo. **Não há opt-out.** Isso torna **inexprimível** o *Absolute position* do Figma: um
badge sobre a quina de um card, um selo *"novo"* sobre um botão, um overlay dentro de uma moldura
com auto layout.

⚠️ O motor já sabe fazê-lo (`Position::Absolute` é nativo do taffy) e a fatia já sabe pular
sub-árvores. O canal barato é um **componente marcador** (`VecLayoutAbsolute`) — que cunha
`stable_type_id` próprio e **não bumpa `PROJECT_SCHEMA`**, ao contrário de apender um `bool` ao
`VecLayoutItem`.

### 2.5 E os outros contêineres da Godot dissolvem em propriedades

| Godot | dissolve em | veredito |
|---|---|---|
| `MarginContainer` | `VecLayout::pad` | ✅ já temos |
| `CenterContainer` | `align`/`justify = Center` | ✅ já temos |
| `PanelContainer` | `VecFrame` + preenchimento + `pad` | ✅ já temos |
| `FoldableContainer` | `SectionHeader` + colapso (2026-08-09) | ✅ já temos |
| `AspectRatioContainer` | — | ⛔ dispensável (um caso; o artista fixa o tamanho) |
| `SplitContainer` | os divisores do chrome do app | ⛔ fora do vetor: é *chrome do editor*, não conteúdo autorado |
| `SubViewportContainer` · `GraphElement` | — | ⛔ fora de escopo |
| **`ScrollContainer`** | — | ⚠️ **falta, e é MustHave** (§5.3) |
| `TabContainer` | `Tabs` (o widget) + trocar o conteúdo | 🔶 metade: a faixa existe, *"cada aba mostra um painel"* não |

---

## §3 — O censo da Godot (fonte oficial) e a triagem

> Fonte: `docs.godotengine.org/en/stable/classes/` — as listas **"Inherited By"** de `Control`,
> `Container`, `BaseButton`, `Button` e `Range`, lidas hoje.

**Contêineres (14 diretos):** `AspectRatioContainer` · `BoxContainer` (→ `H`/`VBoxContainer`) ·
`CenterContainer` · `EditorProperty` · `FlowContainer` (→ `H`/`VFlowContainer`) · `FoldableContainer`
· `GraphElement` · `GridContainer` · `MarginContainer` · `PanelContainer` · `ScrollContainer` ·
`SplitContainer` (→ `H`/`V`) · `SubViewportContainer` · `TabContainer`.

**Controles (20 diretos):** `BaseButton` (→ `Button` → `CheckBox`, `CheckButton`,
`ColorPickerButton`, `MenuButton`, `OptionButton`; `LinkButton`; `TextureButton`) · `ColorRect` ·
`Container` · `GraphEdit` · `ItemList` · `Label` · `LineEdit` · `MenuBar` · `NinePatchRect` ·
`Panel` · `Range` (→ `ProgressBar`, `ScrollBar`, `Slider`, `SpinBox`, `TextureProgressBar`,
`EditorSpinSlider`) · `ReferenceRect` · `RichTextLabel` · `Separator` · `TabBar` · `TextEdit`
(→ `CodeEdit`) · `TextureRect` · `Tree` · `VideoStreamPlayer` · `VirtualJoystick`.

⚠️ **Nota que muda a leitura:** `Popup`, `PopupMenu`, `AcceptDialog` e `FileDialog` **não são
`Control`** — herdam de `Window`. Na Godot um menu e um diálogo são **janelas do SO**; aqui são
superfícies desenhadas (o `defers_a_popover` do `Dropdown` já é exactamente essa distinção). Não há
nada a portar.

### A triagem, controle a controle

| Godot | nosso | veredito |
|---|---|---|
| `Button` | `Button` | ✅ temos |
| `CheckBox` | `Checkbox` | ✅ temos |
| `CheckButton` | `Toggle` | ✅ temos |
| `OptionButton` | `Dropdown` | ✅ temos |
| `TextureButton` | `IconButton` | ✅ temos |
| `TabBar` | `Tabs` | ✅ temos |
| (rádio) | `RadioGroup` · `SegmentedAdaptive` | ✅ temos (a Godot faz com `ButtonGroup`) |
| `Label` | texto vetorial vivo (com `wrap_width`) | ✅ temos |
| `LineEdit` | `TextInput` | ✅ temos |
| `SpinBox` | `NumberInput` | ✅ temos |
| `HSlider` | `Slider` | ✅ temos |
| `ProgressBar` · `TextureProgressBar` | `ProgressBar` · `LevelMeter` | ✅ temos |
| `Separator` | `Divider` | ✅ temos |
| `Panel` | `Card` / `VecFrame` | ✅ temos |
| `ColorRect` | **é uma forma** | ✅ trivial |
| `TextureRect` | uma forma com imagem | ✅ temos |
| `ColorPickerButton` | `ColorSwatch` (abre o picker do chrome) | ✅ temos |
| `ItemList` | `ListItem` (sem virtualização) | 🔶 parcial |
| `TextEdit` | `TextArea` (nativo, não vestível) | 🔶 parcial |
| `ScrollBar` | scrollbar do chrome, não autorável | ⚠️ **falta** (§5.3) |
| `Tree` | `tree_view` nativo, não vestível | ⛔ descartar: um painel autorado descreve lista FIXA |
| `MenuBar` · `MenuButton` | — | ⛔ descartar: é chrome de aplicação |
| `NinePatchRect` | — | ⛔ **descartar por construção** (§6) |
| `RichTextLabel` | — | ⛔ descartar (BBCode é um mini-parser; o texto vetorial já tem eixos, tracking, alinhamento) |
| `CodeEdit` · `GraphEdit` · `VideoStreamPlayer` · `ReferenceRect` · `VirtualJoystick` · `EditorProperty` · `EditorSpinSlider` | — | ⛔ descartar (ferramenta de editor, mídia ou plataforma) |
| `LinkButton` | `Button` | ⛔ descartar (é um `Button` com sublinhado) |

---

## §4 — A lista MustHave (o que obrigatoriamente precisamos ter)

**Contêineres — 4, e são propriedades, não classes:**

| # | capacidade | estado |
|---|---|---|
| 1 | **fluxo em linha / coluna** (`Row` · `Column`) | ✅ **temos** |
| 2 | **quebra** (`RowWrap`) | ✅ **temos** |
| 3 | **recuo + vão + alinhamento + distribuição** | ✅ **temos** |
| 4 | **recorte do transbordo** (`VecFrame::clip`) | ✅ **temos** |

**Sizing — 5, e é aqui que o buraco está:**

| # | capacidade | estado |
|---|---|---|
| 5 | **Fixed** | ✅ **temos** |
| 6 | **Fill** (`grow`/`shrink`) | ✅ **temos** |
| 7 | **Hug contents** | ❌ **falta** — sai de graça do motor (§2.3) |
| 8 | **Min / Max** | ❌ **falta** — idem |
| 9 | **Absolute position** (sair do fluxo) | ❌ **falta** (§2.4) |

**Widgets — o catálogo dos 20 já cobre ~100% da UI real deste app.** Nada obrigatório falta.

⇒ **A lista de obrigatórios que NÃO temos tem exactamente três itens: 7, 8 e 9.**

---

## §5 — O que falta para UI/UX *extraordinária*

### 5.1 O sizing completo (itens 7-8-9) — **a maior alavanca, e a mais barata**

Sem `hug`, **toda moldura tem tamanho digitado à mão**: o botão não cresce com o rótulo, o card não
cresce com o conteúdo, e trocar uma palavra obriga o artista a redimensionar. É a diferença entre
*"eu componho e o layout obedece"* e *"eu componho e depois arrumo"* — e o segundo é o que faz uma
ferramenta parecer velha.

Preço: o motor já responde (medido); falta o par **Auto | Fixed** por eixo no painel (o precedente
literal é o do texto), dois números de limite, e um marcador para o absoluto.

### 5.2 O **estado** ligado ao dado (o laço que fecha a UX)

Já existe muito mais do que parece: `ph2d-ui-state::StateSets` viaja no arquivo (`PROJECT_SCHEMA`
v56), há painel de estados, Smart Animate e a mola; e o **R0 do runtime** integrou a saída de sinais.
O que falta é o **consumidor**: um botão autorado emite `SignalOrigin::Control`, e nada ainda
**liga** *nome do sinal → ação*. É a tabela de ligação que o handoff do R0 nomeia — conteúdo
autorado, não motor.

### 5.3 O **scroll** numa moldura autorada

Toda UI real tem lista que transborda. O painel autorado rola (é o chrome que rola); uma **moldura**
não. O `clip` já existe — falta o deslocamento e a roda. Sem ele, uma UI de jogo com inventário,
uma lista de saves ou um log são inexprimíveis.

---

## §6 — O descartado, com o motivo (cercas de Chesterton)

| descartado | motivo — e ele é estrutural, não preguiça |
|---|---|
| **`NinePatchRect` / 9-slice** | ⚠️ **Descartável POR CONSTRUÇÃO:** o 9-slice existe porque a Godot desenha **raster**, e esticar uma textura deforma o canto arredondado. Nós desenhamos **vetor** — a forma re-cozinha no tamanho novo e o raio continua o raio. ⚠️ *Reabre no dia em que uma UI vestir uma TEXTURA*, e aí a resposta é 9-slice na forma, não um widget. |
| `Tree` · `ItemList` virtualizados | um painel autorado descreve uma lista **FIXA**; uma árvore é função dos dados. É a mesma fronteira que o levantamento de 08/08 já traçou para o Inspector. |
| `MenuBar` · `MenuButton` · `Popup*` · diálogos | são **chrome de aplicação** (e na Godot nem são `Control`: são `Window`). O app já os tem nativos. |
| `RichTextLabel` / BBCode | um mini-parser de marcação por um ganho que o texto vetorial (eixos variáveis, tracking, alinhamento, refluxo) já cobre. |
| `SplitContainer` | divisor é chrome do editor, não conteúdo autorado. |
| `VFlowContainer` | coluna que transborda para outra coluna é layout de jornal; nem o Figma o oferece. |
| `AspectRatioContainer` | um caso; o artista fixa o tamanho. |
| `CodeEdit` · `GraphEdit` · `VideoStreamPlayer` · `VirtualJoystick` · `ReferenceRect` · `Editor*` | ferramenta de editor, mídia ou plataforma — nada disto é vocabulário de UI. |
| **Cassowary / solver de constraints** | já recusado no plano-mãe §5, e continua certo: as âncoras (W3) cobrem o HUD, e align/distribute cobre o relacional. |

---

## §7 — A ordem recomendada, com o preço

| ordem | wave | ganho | custo | estado |
|---|---|---|---|---|
| ~~1~~ | **Hug + Min/Max** | o auto layout deixa de ser meio-feito | o motor já respondia | ✅ **CONSTRUÍDA** (2026-08-10) |
| ~~2~~ | **Absolute position** | badge/overlay dentro de auto layout | componente marcador ⇒ zero bump | ✅ **CONSTRUÍDA** (2026-08-10) |
| ~~3~~ | **Scroll na moldura** | lista longa deixa de ser inexprimível | `clip` já existia; o motor já transbordava | ✅ **CONSTRUÍDA** (2026-08-10) |
| ~~4~~ | a tabela **sinal → ação** | o botão autorado *faz* alguma coisa | conteúdo autorado; o R0 já mede que o canal custa ~0 | ✅ **CONSTRUÍDA** (2026-08-10) |
| ~~5~~ | **Grid** | colunas alinhadas + refluxo | **~0,47 s** de build, uma vez (o `+11 ms` era falso — §2.2) + UI + fatia | ✅ **CONSTRUÍDA** (2026-08-10) |

### ✅ O que a wave 1+2 entregou (2026-08-10)

**O vocabulário de sizing do Figma está completo:** `Fixed` · `Hug` · `Fill` (o `grow`, que já
existia) · `Min` · `Max` · `Absolute position`.

| onde | o quê |
|---|---|
| motor | `Len::{Fixed,Hug}` · `min`/`max` por eixo · `LayoutError::HugWithoutFlow` |
| documento | `VecLayoutSize` (do NÓ) · `VecLayoutAbsolute` (marcador) — **zero bump de `PROJECT_SCHEMA`** |
| fatia | `size_of` (porta única, irmã do `frame_style`) · o fora-do-fluxo sai da FATIA · a RAIZ entra no laço de colocação |
| UI | Width/Height `Fixed \| Hug` · os quatro limites · o toggle Absolute, que **esconde Grow/Shrink** |
| smoke | **`PH2D_BUILD_SMOKE=66`** |

⚠️ **A previsão da §2.3 foi confirmada pela construção:** o abraço **não precisou de measure
function nenhuma**, e o `clip` veio de graça (o `frame_clip` já lê a `LiveGeometry`, então uma
moldura que encolhe leva o recorte junto).

⚠️ **E ela dissolveu uma pergunta do §5.3:** *scroll* continua em aberto, mas `Hug` + `Max` já
cobrem metade do caso que o motivava — uma lista que cresce com o conteúdo **até** um teto. O que
falta é o que passa DO teto, e é aí que a rolagem começa.

### ✅ O que a wave 3 entregou (2026-08-10)

**Uma moldura que recorta e cujo conteúdo não cabe ROLA com a roda.** A sonda decidiu o desenho
antes de qualquer código (`ph2d-vec-layout::overflow_probe`, pela porta do PRODUTO): os filhos
**transbordam** em vez de encolher (`y = 0/40/80/120/160` numa moldura de 100), o par `Hug + Max`
transborda igual, e o **controle** (moldura de 400) reporta −200 — o excedente é derivável.

| onde | o quê |
|---|---|
| motor | nada — ele já transbordava, e é esse o achado |
| fatia | `layout_live::scroll` — o excedente MEDIDO por passe · o deslocamento herdado pelos ancestrais · o alvo da roda PUBLICADO por quem coloca |
| gesto | `layout_scroll_gesture` — a roda, antes do zoom e depois do painel |
| smoke | **`PH2D_BUILD_SMOKE=67`** (a LISTA e o CONTROLE, na mesma caixa) |

⚠️ **O deslocamento é VISTA e não documento**, e a razão é a que fez o modo de preview existir: o
undo deste editor é por DIFF do mundo, então um scroll no ECS faria **cada tique de roda virar um
passo de undo**. Ele não viaja no arquivo — reabrir um projeto mostra o topo da lista.

⚠️ **E ele não rouba o zoom:** só uma moldura que **recorta** e que **transborda** é alvo, e as duas
condições juntas só valem numa lista que o artista fez transbordar de propósito. É o precedente do
Gap Closure, com a mesma frase: *a roda crua é load-bearing*.

⚠️ **A previsão da §5.3 sobreviveu ao contacto:** *"o `clip` já existe — falta o deslocamento e a
roda"* estava certa, e o que ela não previa é que **o motor também já estivesse pronto**. O trabalho
foi todo de costura.

---

### ✅ O que a wave 4 entregou (2026-08-10)

**Um sinal move a cena.** A saída do R0 tinha três produtores e **um** consumidor — um toast —,
então um nome aparecia e nada acontecia. A ligação que faltava é `HostStates.on_signal`:
*quando `name` chegar, este hospedeiro vai para `role`*.

| onde | o quê |
|---|---|
| modelo | `ph2d_ui_state::SignalBinding` + `StateSets::targets(name)` — a busca é por NOME e ignora quem gritou (ADR-0143) |
| documento | `HostStates.on_signal` (**`PROJECT_SCHEMA` 70→71**, provisório) |
| UI | a lista *On Signal* no fim da seção States: campo de texto + os quatro papéis + lixeira + `+ Signal` |
| produtor | na preview, um clique completo sobre um hospedeiro publica o `Name` dele |
| consumidor | o frame lê a saída **sempre** e **age só na preview** |
| smoke | **`PH2D_BUILD_SMOKE=68`** (um botão, um menu, e um CONTROLE que não escuta) |

⚠️ **Três decisões foram DERIVADAS, não escolhidas, e valem mais que a feature:**

1. **A tabela mora dentro do `HostStates`.** O `retain_hosts` já corre por frame ⇒ uma forma
   apagada leva as ligações dela **sem uma linha a mais**. Uma tabela global precisaria da
   própria varredura de hospedeiros mortos.
2. **A única ação é *ir para um papel*, e o limite é o que a preview consegue DESFAZER** — ela
   captura a pose de todo id mencionado em qualquer estado autorado. Uma ação que mudasse
   qualquer outra coisa não seria restaurada, e o documento mudaria por o artista ter olhado.
   ⇒ *uma ação nova traz consigo a metade que a desfaz, ou não entra.*
3. **O cursor anda sempre; só a AÇÃO é gateada na preview.** Fora dela não há restauração e o
   undo regista. ⚠️ E isto **não** contradiz o botão *Show*, que escreve o mundo: a diferença é
   *quem pediu* — uma pose que o artista pede com um clique custa um passo de undo e ele sabe
   porquê; uma pose que **chega sozinha** não pode cobrar nada.

⚠️ **E a metade de produtor não custou campo nenhum:** o nome do sinal é o **`Name` da entidade**
— o mesmo que a Hierarquia mostra e que o botão do painel autorado já publica. Foi a mesma razão
que fez o `VecWidget` não guardar `key`.

⚠️ **Nenhum variante novo no `PanelEvent`** (contrato CONGELADO): o nome digitado viaja no
`SelectOption`, que já é o canal string-valued deste app — o Painter carrega nele
`"layer:channel:index:x:y"`. O doc dele dizia *"RadioGroup option selected"*, falso há muito, e
**é o comentário que mandaria a próxima wave gastar um ADR num variante que este já dá**;
corrigido, com a contagem intacta e o gate a prová-lo.

⚠️ **DUAS mutações sobreviveram na 1ª rodada e as duas eram FIXTURE minha:** soltar no **vazio**
deixa `chain.first()` em `None` dos dois lados (a guarda do alvo fica inobservável — o caso que a
exercita é soltar sobre **outro** alvo), e sem o rato **passar por cima antes do aperto** o `hot`
está vazio no `Down`, então a guarda do alvo devolve `None` sozinha e a do *soltar* fica coberta
por ela. São defesas em camada, e uma fixture sem o hover mede apenas a primeira.

**Aberto, com o preço ao lado:** o papel é um catálogo de **quatro** (`Default`/`Hover`/`Pressed`/
`Disabled`), então um menu *aberto* autora-se no slot `Pressed` — o nome fica torto. ⚠️ E a cerca
que o `role.rs` escreve (*"papéis existem para não precisar de tabela de gatilhos"*) **teve a
premissa movida por esta wave**: ela continua de pé para o RATO, cujo papel é derivado, e deixou
de valer para os SINAIS, que agora têm tabela. Estados de nome livre são decisão de produto, e o
preço é `DOC`/`PROJECT_SCHEMA` mais um seletor que hoje é uma segmentada de quatro.

---

⚠️ **A ordem é por RAZÃO ganho/custo, não por tamanho** — e o (1) é o único item onde os dois lados
já estão medidos.

---

## §8 — Correções que este estudo produz (e que valem mais que a lista)

1. ⚠️ **O ADR-0153 tem um argumento que não sobrevive à medição de hoje.** Ele recusa o grid porque
   *"triplica o custo de build"*; re-medido costas-com-costas, **+11 ms (1,07×)**. A recusa continua
   defensável — pelo **outro** argumento —, mas quem a ler hoje decide com um número que não é mais
   verdade. *Quem move o número que tornava algo inalcançável tem de reconferir a nota* (§0 do
   `CLAUDE.md`).
2. ⚠️ **A frase *"não há measure function"* está certa e a conclusão que dela se tira, não.** Hug de
   **contentor** não precisa de measure function nenhuma — está medido nos três casos da §2.3.
3. ⚠️ **O `LEVANTAMENTO_…_2026-08-08.md` aponta para uma §6.4 que não existe** (linha 249: *"o que
   sobra é o que a §6.4 chama de horizonte"*; o arquivo acaba na §6.3). **Este estudo é esse
   horizonte** — e a referência quebrada é a assinatura de um doc que envelheceu entre o commit e a
   frase, exactamente a classe que a auditoria de 09/08 passou a jornada a curar.

---

## §9 — O que este estudo NÃO diz

- **Não diz que o catálogo de 20 widgets está completo para JOGOS.** Ele foi medido contra a UI
  **deste app** (438 construções, 23 painéis). Um HUD de jogo tem minimapa, barra de vida
  segmentada e diegético — e nenhum deles é um widget: são desenho.
- **Não mede a descobribilidade.** A §2.1 conclui que Row/Column existem; se o Enio sentiu falta
  deles *olhando para a tela*, isso é um achado de UI que só o smoke fecha.
- **Não trata do Inspector** (lista função da seleção) nem da UI dos jogos — as duas fronteiras que
  o levantamento de 08/08 já nomeou.

### ✅ O que a wave 5 entregou (2026-08-10)

**A grade honra os quatro degraus** — o motor (`Dir::Grid { columns }` → `Display::Grid` +
`repeat(N, 1fr)`), o documento (`LayoutDir::Grid` + `VecLayout::columns`), a fatia viva e o painel.
O argumento vivo do ADR-0153 (*"nada a jusante honraria"*) morre por construção.

| onde | o quê |
|---|---|
| motor | `Dir::Grid` · `MAX_GRID_TRACKS` (medido) · `LayoutError::GridTooManyTracks` · o `align` a governar **`align_content` E `align_items`** |
| documento | `LayoutDir::Grid` (variante apendada) · `VecLayout::columns` — **`PROJECT_SCHEMA` 71→72** |
| painel | o 5º chip do rádio · a row **Cols** (só na grade) · o Justify **estreitado a três** |
| gesto | a régua de reordenar passa a ler em **LINHAS** (e isto conserta o `RowWrap` também) |

**Três coisas que a medição decidiu, e não o gosto:**

1. **O teto de faixas é `i16::MAX`, e ele morde pelas LINHAS.** Bissectado contra o motor: 1 coluna
   → 32767 filhos; 2 → 65534; 3 → 98301 — sempre **32767 linhas**, que é a largura do
   `OriginZeroLine(i16)` do `taffy`. Um cap na contagem de COLUNAS teria sido taste **e não teria
   evitado o pânico**, que vem das linhas — e elas ninguém autora, nascem da contagem de filhos.
   Acima do teto o `solve` **recusa**; acomodar ali não dá uma pose errada, dá um `unwrap` a
   estourar dentro do `taffy`.
2. **A contagem é um CAMPO, não o corpo do variante.** Com ela no variante, ir a `Row` e voltar
   destruiria o número; como campo ela sobrevive à ida e à volta, como o vão e o recuo já
   sobrevivem — e o `DIRS` da shell continua a ser **uma** tabela lida nas duas direções.
3. **O `align` numa grade governa DUAS propriedades do CSS.** Sem isso ela herdaria
   `align-content: normal`, que numa grade se comporta como `stretch`: as LINHAS seriam esticadas
   para encher a moldura, e a mesma cena que um `Row` encosta no topo sairia espalhada. O gate vivo
   nasceu **vermelho** exactamente aí.

⚠️ **E a régua de reordenar era ERRADA, não imprecisa** — medido antes de uma linha ser escrita:
numa grade 3×3 as três células da coluna 0 partilham o mesmo `x`, então *"quantos centros estão
antes do cursor"* devolvia o **slot 3** para uma soltura na célula (0,0). O `RowWrap` shipa com o
mesmo defeito desde que nasceu; ele ficou invisível ali porque uma faixa de wrap raramente alinha
duas linhas. **Esta wave conserta os dois.**

**Aberto, com o preço ao lado:** *track sizing* autorado (`fr` × `auto` × comprimento) — é um
vocabulário próprio e hoje seria um controlo que ninguém move; o `justify_content` de uma grade é
**inerte por construção** com colunas `1fr` (não há sobra horizontal a repartir), então distribuir
as COLUNAS só faz sentido junto com o track sizing.
