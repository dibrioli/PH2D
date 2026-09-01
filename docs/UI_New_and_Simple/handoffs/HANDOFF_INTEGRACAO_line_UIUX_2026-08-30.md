# HANDOFF de integração — `line/UIUX`, 2026-08-30

> Leitor: a próxima LLM (ou o agente integrador). Denso de propósito.
> ⛔ **A linha NÃO integra e NÃO pusha** (`CLAUDE.md` §0.7). Isto é o que ela entrega.

Branch `line/UIUX`, worktree `Worktrees/line-UIUX/`, base `main` em `066b4f92e`.

---

## §1 — O que a linha é

Aberta a pedido do Enio para **redesenhar a UI/UX** do app. A etapa 1 (levantamento, medição,
triagem de licença das referências, nove decisões dele) vive em
[`docs/UI_New_and_Simple/`](../README.md) e **não** se repete aqui. Este handoff cobre o que
virou **código**.

---

## §2 — Entregas de código, por ordem

| # | commit | o quê |
|---|---|---|
| 1 | `bc8d51a42` | **Unidade de ÂNGULO** — `Settings → Angle unit` (graus/radianos), persistida. `PROJECT_SCHEMA` **103 → 104** |
| 2 | `de7f88c3b` | o submenu **MARCA** a opção activa (a 8.ª ponta que faltava) |
| 3 | `4a3d8c932` | os **rótulos e o PASSO** do Inspector seguem a unidade (a 9.ª ponta, e eram dois defeitos) |
| 4 | `b4035fbd2` | ⭐ **As RÉGUAS viram REGIÕES da área de desenho** |
| 5 | `9f6db8566` | duas notas minhas **refutadas**, o roteiro do smoke corrigido, gate novo dos apelidos de cor |
| 6 | `7378d8b1f` | ⛔⛔ **a auditoria achou uma REGRESSÃO minha** — os quatro defeitos, curados (§3-bis) |
| 7 | `24e25728c` | a auditoria corrigiu um **número** meu (87,8 → **86,8 %**) — 13 sítios |
| 8 | `14df4796c` | o portão de fecho com o número **honesto**, e duas flakes novas nomeadas |
| 9 | `41ff3fb48` | ⭐ **as RÉGUAS valem em TODOS os modos** (Enio) — e isso dissolveu o teorema do #7 (§3-ter) |
| 10 | `a260a30fd` | a wave das réguas-em-todo-modo documentada, e a próxima **medida antes de começada** |
| 11 | `b7264acfb` | ⭐⭐ **AS COLUNAS LATERAIS SÃO ANCORADAS** — um bloco governava **dezasseis** painéis |
| 12 | `cc3dc6b44` | ⭐ **o chrome legado sai de cena e as bandas ficam FLUSH** — 94 px em cima e 60 em baixo de espaço morto |
| 13 | `66f460f7c` | ⭐ **os números da régua VERTICAL rodam**, e a régua **encosta** na coluna |
| 14 | `28f9d95d0` | ⭐ **A BORDA INTEIRA redimensiona a coluna** — seta bidireccional, e os pontinhos saem |
| 15 | `d15640c62` | ⭐⭐ **A BARRA DE MENUS** — *File · Edit · View · Window*, e ela **realoja** verbos (§10) |
| 16 | `810c1abd4` | ⭐⭐ **A FILA DE FERRAMENTAS** — os chips do trilho deitados por cima da área (§11) |
| 17 | `eb2013fc5` | ⛔⛔⛔ **A AUDITORIA achou SETE defeitos, e o dominante era meu** (§12) |
| 18 | `165d6a096` | ⛔⛔ **UMA TABELA para a verdade de cada módulo** — e o `if` com um lado morto (§13) |
| 19 | `9bda8e3de` | ⭐ **A FUGA DO GIZMO ficou inerte** — a `D1` cumprida sem apagar a lei (§14) |
| 20 | `ce7814813` | ⭐⭐ **OS SEIS ENCAIXES e a declaração de cada painel** — a `D1` como tipo (§15) |

---

## §3 — A wave das réguas (commit 4), mecanismo

### 3.1 O que mudou

`HeroLayout` ganha **`draw_area`**: o que sobra da janela depois de o chrome **docado** tirar a
sua faixa — abaixo da barra de topo, acima do HUD, à direita do trilho, entre as colunas laterais
**abertas**. As duas réguas do canvas passam a ser ancoradas nela.

```
antes:  ruler host = layout.canvas  (= a viewport INTEIRA)
depois: ruler host = layout.draw_area
```

| | antes | depois |
|---|---:|---:|
| régua esquerda tapada (1366 × 1024) | **86,8 %** | **0,0 %** |
| régua de cima tapada | **29,4 %** | **0,0 %** |

### 3.2 ⭐ Por que custou uma linha e não uma reescrita — três factos MEDIDOS

1. `ruler::top_band` / `left_band` / `hit` **já recebiam um rect por argumento**
   (`crates/ph2d-editor-core/src/ruler.rs`).
2. `ruler::in_band` **já filtrava** os traços que caem fora da faixa — não foi preciso clipping
   novo.
3. ⭐⭐ **A projecção não depende desse rect.** `grid::world_bounds` deriva de
   `view.window_w`/`window_h`; `view.canvas` só decide **onde a faixa é pintada** e o
   `push_clip` da grelha.

⇒ a régua mudou-se **sem levar a projecção consigo**: um traço marcado em 100 continua a cair no
mesmo pixel de ecrã. *Uma régua que se mudasse e apontasse para outro sítio seria pior do que uma
tapada* — é a lei do §5.0 do `CLAUDE.md` sobre cenas que ensinam o contrário.

### 3.3 ⛔⛔ O segundo defeito, de INPUT, que nenhuma sonda deste repo via

A régua **não está no `HitIndex`**. O gesto da guia é geométrico (`ruler::hit`) e é despachado em
`shells/desktop/src/input_dispatch.rs:3517`, com `return` quando acerta — o hit-test de chrome só
vem em `:4989`. Enquanto o hospedeiro foi a viewport inteira:

- press nos **6 px de cima de qualquer botão da barra** (banda `y ∈ [0,20]`, barra em `y = 14`),
- press nos **3 px da esquerda de qualquer chip do trilho** (banda `x ∈ [0,20]`, chip em `x = 17`),

**criava uma guia em vez de carregar no botão** (só em modo Vector — `rulers_live()` exige
`panel_visible("vector")`).

⚠️ **Toda a família de gates de costura deste repo pergunta ao `HitIndex`.** Uma superfície que
faz hit-test **fora** do índice é invisível a todos eles: 15 gates verdes conviviam com isto.
*Escrever a próxima sonda de costura a partir do `HitIndex` volta a não ver a próxima.*

⚠️ **A cerca que sobrevive:** o `guide_smoke.rs` já nomeava o mecanismo — *«uma régua permanente
comeria o pen-down do Painter nos 20 px de cima»* — e foi por isso que a régua é escopada ao
Vector. Isso **continua verdade**: a mudança tirou as faixas de cima do **chrome**, não de cima do
**desenho**.

### 3.4 As colunas — ⛔ **SUPERSEDIDO pelo §3-bis, não leia esta versão**

A 1.ª redacção descrevia um `DockSides { left, right }` alimentado por uma lista de cinco chaves
e um censo que varria o `layout.rs`. **Os três estavam errados por construção** e foram
substituídos no commit `7378d8b1f` — o mecanismo, o porquê e a lei que daí saiu estão no
**§3-bis**. *Um handoff que descreve o desenho revogado ao lado do vigente é a forma de alguém
reconstruir o revogado.*

### 3.5 Gates e provas de mutação

**9 gates · 8 mutações, 8 mortas.** A tabela e o placar estão no **§3-bis**, que é onde os gates
de facto ficaram. O gate de fonte do shell
(`shells/desktop/tests/the_node_ops_are_wired.rs`) proíbe o regresso **pelo nome**:
`!contains("canvas: layout.canvas,")`.

---

## §3-bis — ⛔⛔ A AUDITORIA (4 lentes) achou **quatro** defeitos, e o dominante era meu

⭐ **Leia esta secção antes de acreditar no §3.** A wave anunciava «0 %» e no modo real deixava
**31,2 %** — *pior* que os 29,4 % que dizia curar.

### D1 (dominante, regressão) — `"vector"` fora da lista das colunas

Ao pegar na ferramenta Vector, `shells/desktop/src/render_loop/vector_bridge.rs` põe
`panel_visible("inspector") = false`, e o **painel Vector** passa a desenhar no rect do dock
direito (`ph2d-panel-vector/src/paint.rs`, `ctx.layout.inspector`). A minha `RIGHT_DOCK_PANELS`
de cinco chaves não o continha ⇒ a área crescia **para dentro do painel**.
⚠️ E `rulers_live()` **exige** `panel_visible("vector")` ⇒ era o **único** modo em que a régua
existe.

⭐⭐ **A cura não é uma lista maior: são DEZASSETE as crates de painel que desenham naquele
rect** (`ctx.layout.inspector` / `.padding`) — é um slot de **takeover**, não um painel; doze
delas não têm alias no `layout.rs` e lêem o campo directamente, de outra crate.

⭐⭐⭐ **A cura é um TEOREMA:** o único consumidor da `draw_area` é a régua, e *régua viva ⇒
painel Vector visível ⇒ coluna da direita ocupada*. Reservá-la **sempre** custa **zero** ao único
consumidor e é imune a qualquer inquilino futuro. ⇒ o campo `right` desapareceu; o que sobra é
`hierarchy_open`, porque **só a coluna da Hierarchy pode ficar vazia** e ela tem **um** inquilino
medido. Há gate no fonte a defender as duas metades do teorema.

### ⛔ Por que os meus três gates eram cegos — é uma LEI, não um descuido

> **Uma exclusão e o gate que a verifica não podem ler a MESMA lista.**

- **a LEI** partilhava a premissa com o produto (`if docks.right { push(right_panel) }`): com a
  lista errada, o painel saía da **exclusão** e da **acusação** ao mesmo tempo, e o gate devolvia
  `0.0` *por não olhar*. Hoje o oráculo põe `l.inspector` **sempre**, seja quem for que lá esteja.
- **o CENSO** varria `let X = inspector;` no `layout.rs` — tinha a **forma** de uma conferência de
  dois lados e media um subconjunto que não era a pergunta. Hoje varre as **crates de painel**.
- **o ESPELHO** iterava a própria lista: auto-referencial.

### D2 — o desenho tinha guarda, o hit-test não

`paint_rulers` saía com `w <= RULER_PX`; o `hit` só perguntava `contains`. Numa área com
`0 < w <= RULER_PX` a régua **respondia sem aparecer** — o inverso do invariante que
`offers.rs` declara (*visível ⇔ vivo*). ⚠️ **E a wave tornou-o alcançável:** antes exigia uma
janela de 20 px (impossível), depois uma de **~735 px de largura**. Cura: `ruler::live_bands`,
a porta única. *Os dois lados já liam o mesmo rect e continuavam a ler predicados diferentes.*

### D3 (regressão minha, menor) — a guia inagarrável

O predicado do **agarrar** foi com a faixa para a `draw_area` por arrasto, e uma guia largada
sobre um painel passou a ser inagarrável — **sem comando de limpar guias em lado nenhum**.
⇒ nascer e morrer alcançam a **faixa**; agarrar alcança a **janela**, como sempre alcançou.

### D4 — o dock do fundo

O `timeline` nasce **exactamente** em `area_x0` e ocupa 240 px no fundo da banda: partilhava
**4 800 px²** com a régua da esquerda. O próprio `layout.rs` chama-lhe *«General timeline dock»*,
logo «depois do chrome DOCADO» era impreciso. Cura: `reserve_bottom_strip`, chamada **depois** do
`dock_timeline_into_motion` (que move o rect — a ordem é load-bearing e tem gate).

### Placar

**9 gates** (eram 5 — três substituídos por errados-por-construção, quatro novos) ·
**8 mutações, 8 mortas**. As três novas: a coluna do takeover não reservada (**a regressão** ⇒ lei
vermelha), o `hit` a deixar de perguntar a porta, a fiação da faixa de fundo apagada.

### ⚠️ O que a auditoria nomeou e NÃO foi curado

`clamp_panel_rect` (`hero/paint.rs`) move e alarga os painéis **depois** de a `draw_area` ser
fixada — um painel **arrastado à mão** ainda tapa a régua. É a `allowed_slots` / `can_float` da
D1 do Enio (a fase seguinte), não esta.

### ⛔ E a auditoria corrigiu um NÚMERO meu, que eu tinha propagado por nove documentos

A medição original dava a régua esquerda a **87,8 %** tapada, somando *«rail 17 400 + barra 384 +
HUD 204»*. O termo do HUD é **falso**: o `bottom_hud` é **centrado** (`x ∈ [443, 923]`) e a faixa
da régua é `x ∈ [0, 20]` — **não se tocam**. O número é `17 784 / 20 480` = **86,8 %**.

⭐ A conclusão não muda (o trilho sozinho já faz 85 %), mas o dígito tinha viajado para nove
documentos, dois doc-comments, um gate e o roteiro de um smoke antes de alguém o reconferir.
**Um termo a mais numa soma sobrevive a toda leitura que confia no total.**

---

## §3-ter — As réguas saem do escopo Vector, e a 3.ª tentativa das colunas

**Enio, 2026-08-30:** *«as réguas devem funcionar em todos os modos e layouts e não apenas para
vector».* `rulers_live()` passa a ter **uma** condição: o interruptor do artista.

⚠️ **A cerca que caiu era legítima, e o substrato dela dissolveu-se no mesmo dia.** Ela dizia que
uma régua permanente comeria o pen-down do Painter nos 20 px de cima. O defeito real ali era a
faixa ser **invisível** (nascia debaixo do trilho e da barra); com ela dentro da `draw_area` está
à vista, e carregar numa régua visível para criar uma guia é o que todo DCC faz.
**Preço nomeado:** os 20 px de cima e da esquerda da área deixam de ser pintáveis em qualquer
ferramenta — o preço do Photoshop, e o interruptor desliga-o.

### ⛔⛔ E isto matou o TEOREMA do §3-bis, com horas de vida

A cura da regressão dizia *«régua viva ⇒ Vector visível ⇒ coluna da direita ocupada»*. A primeira
implicação evaporou-se. *Quem move o número que tornava algo inalcançável reconfere a nota*
(`CLAUDE.md` §0.0) — e quem o moveu fui eu.

⭐⭐⭐ **A 3.ª tentativa é a que a auditoria já recomendava: pergunta-se ao que ACONTECEU.** Todo
painel publica o próprio rect por quadro (`set_panel_rect`) e limpa-o quando some — 20 crates.
`DockSides::from_published` cruza os rects publicados com o rect da coluna. **Não há lista**, logo
não há lista a apodrecer. Preço: **um quadro** de atraso. `COLUMN_TAKEN_FRAC = 0.5` impede que um
flutuante que só roça a coluna a reserve. `HeroLayout::side_columns()` devolve-as **ordenadas por
`x`** — é o que torna o `mirrored` inofensivo, e eu escrevi a versão errada primeiro.

⛔ Tecto de LOC estourado por mim (711/700), curado por **corte**: `screens/dock_sides.rs`.

---

## §3-quater — ⏳ A PRÓXIMA WAVE: ancorar as colunas (medida, não começada)

**Enio, com foto (2026-08-30):** *«Funciona, mas legal não ficou. Acho que só fica legal depois de
fixar os painéis nas laterais, jogar os botões laterais para outro lugar (a princípio podemos
apenas escondê-los pois devem passar para área de cima do canvas como na godot)».*

### O que a medição diz

⭐⭐ **UM bloco governa 16 painéis.** `screens/hero/paint.rs` aplica o *offset* de arrasto e o
*resize-delta* a `layout.inspector` / `layout.hierarchy` e depois **espelha o rect arrastado** para
os quatro aliases. Os 16 painéis que lêem `ctx.layout.inspector`/`.padding`/`.painter_layers`
recebem já o rect movido — *arrastar o Inspector arrasta os dezasseis, sem que nenhum saiba*.
⇒ curar ali cura os 16 **sem tocar numa linha das crates deles**.

| frente | sítios |
|---|---|
| desligar a flutuação das colunas | **1** bloco (`paint.rs`) + **3** braços (`interaction/dispatch/blender.rs`) |
| tirar as alças do `HitIndex` | 3 registos no Inspector + 3 na Hierarchy + 4 `store.register` no `pre_populate.rs` — ⚠️ **têm de sair EM PAR**, senão o `architecture_panel_wiring_parity` acende (ou pior: ficam mortas sob o dedo) |
| flutuantes VERDADEIROS, a decidir 1 a 1 | **5** crates (`authored`, `grid-snap`, `wet-tuning`, `timeline`, `widget-gallery`) + o `audio_overlay` do shell — é a *«flutuação DECLARADA»* da D1 |
| quinas | **1** linha (`panel_chrome.rs`, `Radius::Sm`) — ⛔ e **nenhum gate a defende**: a mudança mais visível do trabalho é a única sem rede |
| *flush* da barra/HUD | ⚠️ `EDGE_PAD` serve **quatro** propósitos hoje (borda da janela · x dos painéis · timeline · `draw_area`) — zerá-lo global seria quatro decisões numa. O corte honesto é separar *pad de borda* de *gap entre colunas* |
| gates a ficar vermelhos | **9 certos**, 7 prováveis/de compilação |
| sombra | **não existe** — o efeito de vidro é só o alfa do `PanelBg` |

⛔ **Duas armadilhas nomeadas:**
1. **`PANEL_RADIUS` (16 px, token `chrome.panel-radius`) NÃO tem consumidor de pintura** — o raio
   real é `Radius::Sm` (6). Quem cortar quinas «pelo token» vai ao número errado.
2. **`popover_region()` está ancorado no `left_rail`** — mover ou esconder o trilho move a banda
   em que sete popovers do painel Vector nascem, e há gate a medi-la.

### E os botões do trilho

⭐ **Esconder o trilho NÃO os torna inalcançáveis** — o **menu radial** (`P` segurado) projecta a
secção do meio de `rail_entries`, e a **paleta global** projecta a lista inteira. Mas o destino
que o Enio nomeou é a **barra por cima do canvas**, e o porte tem um achado que o barateia:

⭐⭐ **`paint_topbar_rail_chip` já É o mesmo chip na horizontal** — declarado no código como cópia
verbatim do chip do trilho, a ler as **mesmas** constantes (`LABEL_VISUAL_EXTENT_PX`,
`LABEL_TO_CHIP_GAP_PX`), com o rótulo **por cima** em vez de rodado, e com o hit registado no rect
em repouso. Falta-lhe: visibilidade (`pub(super)`), os braços `Compound`/`Swatch`/`Divider`, o
`active`, e a orientação do flyout.

⛔ **O que o porte tem de curar de caminho:** o pintor e o registo de hit do trilho são **dois
laços separados sobre a mesma lista, com a aritmética escrita duas vezes e SEM GATE** — um eixo
horizontal no pintor e vertical no hit compilaria e passaria a suíte inteira. A porta que falta é
`entry_rects(rail, rect, size) -> Vec<(NodeId, Rect)>`, consumida pelos dois.

⚠️ E o menu radial **depende de haver exactamente três `Divider` e de a secção 2 ser as
ferramentas** — mudar a ordem muda o radial em silêncio.

---

---

## §4 — ⛔ FRONTEIRA declarada, com o preço medido

**A CENA não se mudou, e continua full-bleed por baixo das réguas.**

O sub-rectângulo da cena é ancorado em `(0, 0)` **por construção em toda a cadeia**:
`CenterSplit::scene_viewport` devolve `[0, 0, w, h·t]` e **todo** consumidor lê só as DIMS —
`field_gizmo::scene_window_wh` → `scene_camera_window` → `pan_scene_camera`, mais
`uniform_for_subrect` / `set_viewport` no render.

⇒ **dar ORIGEM à cena é uma mudança na porta única do mundo↔tela**, não um rect. É a obra da
docagem (a `E`/`A2` do [`spec/02 §3`](../spec/02_o_que_falta_para_comecar.md)), e é **onde o
orçamento dela de facto está** — o §3 daquele documento subestimava-o, e está corrigido.

⚠️ Enquanto isso não acontecer, **um painel arrastado à mão ainda pode tapar a régua**. O que
esta wave cura é o chrome **docado**; a cerca que proíbe o resto é a `allowed_slots` /
`can_float` da D1.

---

## §5 — ⚠️ Premissas que a implementação REFUTOU (leia antes de acreditar nos docs)

1. **`PanelLayout::Sidebar` não tem leitor de produção.** O `medicoes/03` afirmava que o
   `blueprint` é o único tema que o liga e que cortá-lo apagaria *«o único modo ancorado do
   app»* — daí saía a **trava dura nº 1** da ordem de arranque. Medido: só a declaração, o
   produtor, o re-export e o `#[test]` do próprio ficheiro; `screens/layout.rs` nem importa o
   tipo. ⇒ **a trava não existe.**
   ⚠️ A forma do erro: *li o PRODUTOR de um valor e concluí a existência de um CONSUMIDOR.* É o
   **id órfão** do §5.0, não o **knob morto** — curas opostas.
2. **O degrau `A` «não depende de nada»** — depende, em metade (o §4 acima).
3. **A cura das réguas não precisava de mexer no módulo `ruler.rs`.** A nota previa uma
   separação entre «geometria da faixa» e «projecção»; ela **já existia**.

---

## §6 — ⛔ Recusas MEDIDAS — não as reconstrua

| recusa | medição |
|---|---|
| **Fundir os 16 apelidos de cor do Timeline** (83 → 67 slots/tema) | Colisão: **zero** (nenhuma das 6 linhas vivas toca `color.rs`, `ph2d-panel-timeline/` ou `tokens.json`). Zero-pixel: **medido**, 64 pares, 0 divergências. ⛔ **Falta o VEREDITO, não a medição**: equivalência ≠ desejabilidade. O Spectrum (Apache-2.0, a nossa referência) mantém tokens de componente a apelidar os globais **de propósito**; fundir troca 16 nomes por 58 sítios directos. Decisão de design system, do Enio. O facto ficou **executável** em `the_sixteen_timeline_slots_are_pure_aliases` |
| **Encolher `layout.canvas`** para a área ancorada | É a viewport inteira por decisão (`hero/tests.rs::layout_canvas_spans_full_viewport_default` pina-o), e os painéis flutuam sobre ele. Mexer nele **é** a docagem — ver §4. O doc-comment do campo descrevia um layout ancorado e **está agora nomeado como contradição**, em vez de corrigido em silêncio |

---

## §7 — Aberto, por ordem de dependência

1. **`E` — os seis encaixes + `allowed_slots`/`can_float`** (D1/D4). Inclui **remover a fuga do
   gizmo de navegação** no mesmo trabalho (senão passa a fugir de uma moldura que já não o
   alcança) e **dar origem à cena** (§4).
2. ✅ **`C` — a barra de menus FECHOU** (entrega 15, §10). ⏳ O que ela **não** trouxe: o
   **cabeçalho por área** continua a não existir (`area_header`/`AreaHeader`/`editor_header` →
   zero), e é ele que dá destino aos comandos que são do editor e não do app.
3. **`F` — Layouts + cabeçalho por área**, que é o que dá destino aos 11 comandos de câmera do
   painel 3D Model.
4. **`G` — esvaziar os painéis** (66 de 74 entradas do painel medido têm outro dono). É aqui que
   a área se ganha, não em `E`.
5. **`I` — cortar os temas 4 → 2**: **desbloqueado** (§5.1), à espera do veredito do Enio.

### ⏳ O que estas duas waves deixaram aberto, com o preço ao lado

| item | estado |
|---|---|
| a fila **transborda** numa janela estreita | há gate com a folga impressa (68 px no alvo de referência, modo Painter). ⛔ A cura quando faltar: **quebrar em duas linhas** (a faixa cresce) ou um menu de transbordo — ⚠️ encolher o chip mente sobre o preset de tamanho |
| o **cabeçalho por área** (D2, metade 2) | não existe. É o dono declarado do selector de **modo** e das opções do editor |
| dois gates ainda fazem `include_str!` do `paint.rs` | são de **presença**: falham alto (§11.5) |
| o hit do trilho **vertical** sem gate comportamental | ele já não consegue derivar (pergunta à porta), e só pinta sob `F9` (§11.6) |
| o `cluster_painter::paint_topbar_rail_chip` continua a ser uma **cópia verbatim** da matriz de tinta do chip | o doc dele di-lo (*«Matriz copiada verbatim — DO NOT diverge»*). Com o `RailAxis` ele passa a ser dispensável; ⛔ não foi tocado porque é chrome legado a caminho de sair |

### ⏳ O que a obra dos encaixes (§15) deixou nomeado

| item | estado |
|---|---|
| **`audio_editor` é uma 2.ª coluna da direita** | ⛔ na catraca `REACHES_PENDING` com o mecanismo. A cura é a **regra 1** do modelo — `n > 1` num encaixe são **ABAS** —, que é a wave seguinte natural e serve `mixer + editor` de uma vez |
| **os rects dos seis encaixes** | o vocabulário existe; `HeroLayout` ainda **não** os resolve (hoje há uma coluna por lado, não duas metades). A metade de cima/baixo só ganha sentido quando um encaixe hospedar mais de um painel |
| **a cena ainda é full-bleed** | dar ORIGEM à cena é a parte cara da docagem (§11), e continua por fazer: o desenho corre por baixo das colunas, e é o chrome que se muda |
| **o piscar de 1 quadro** | a área nasce com a largura toda e encolhe no quadro seguinte (§15.4). Medido, nomeado, não curado |

### As três decisões que continuam a ser do Enio
Como partir o `DrawMode` nos dois eixos · adoptar o campo `Mode` do Workspace · o que acontece
aos 9 toggles de módulo (2⁹ combinações contra um Layout *um-de-N*).
[`spec/02 §5`](../spec/02_o_que_falta_para_comecar.md).

---

## §8 — Como smokar

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && env PH2D_BUILD_SMOKE=45 cargo run -p ph2d-host-desktop --release
```

A cena imprime o roteiro no terminal (`[guides] …`). ⚠️ **Pegue a ferramenta Vector primeiro** —
as réguas vivem com ela em mãos (`rulers_live()`).

O que tem de acontecer: as duas faixas graduadas aparecem nas bordas da **área de desenho** (a de
cima logo abaixo da barra; a da esquerda logo à direita do painel *Hierarchy*), **inteiras**.
Fechar a *Hierarchy* faz a régua da esquerda acompanhar. E o topo dos botões da barra volta a ser
clicável.

Como saber que deu errado: a régua da esquerda a nascer **em cima do trilho** (é o defeito
antigo), ou os traços a deixarem de coincidir com as linhas da grelha (seria a projecção a ter-se
mudado, que é o que o §3.2 diz que **não** acontece).

**Unidade de ângulo** (entrega 1-3): menu *Settings → Angle unit*, alternar entre *Degrees* e
*Radians*; o Inspector muda **rótulo e passo** dos campos de rotação/skew, e a opção activa fica
**marcada** no submenu.

⚠️ **A partir da entrega 9 as réguas valem em TODO modo** — a nota acima sobre *«pegue a
ferramenta Vector primeiro»* é histórica.

**A barra de menus** (entrega 15): sem `F9`, no topo, quatro títulos. Cada um abre por baixo de si.
*Window* tem os treze módulos; *View* tem Hierarchy / Inspector / Rulers / Theme…; *File* tem New
Image… / Open / Import / Save / Save As…; *Edit* tem Undo / Redo / Preferences….
Deu errado se: um título não abre, um menu abre por cima do título, ou o menu **fica aberto**
depois de escolher (o caso apertado são as linhas do *Window*, que os painéis consomem antes do
chrome).

**A fila de ferramentas** (entrega 16): a faixa horizontal por cima da área de desenho, com os
mesmos chips que a coluna tinha e o rótulo **por cima** de cada um. A régua de cima fica **por
baixo** dela.
Deu errado se: um chip pintado não pega ao clique · a fila entra por cima da Hierarquia ou do
Inspector · a régua de cima nasce por baixo da fila · um chip aparece **cortado** na ponta (a
faixa corta a tinta e o clique juntos, então um chip cortado é um chip **inalcançável** — hoje
ela **quebra de linha** em vez de cortar).

**As curas da auditoria** (entrega 17):
1. *Window → Image Tools* — a fila ganha a segunda secção com as **dez ferramentas de imagem**.
   Escolha o **Painter**: os chips de pintura tomam a fila. (Sem isto o Painter era inalcançável
   sem `F9`.)
2. *File → Scenes…* abre a lista de cenas com busca; o menu **Run** tem Play · Pause · **Rewind**.
3. Abra o menu *Window*: o módulo que está aberto aparece **marcado**. O mesmo em *View* para
   Hierarchy / Inspector / Rulers.
4. Com o Painter em mãos, arraste **em cima da barra de menus e da fila**: não pode aparecer tinta
   nenhuma por baixo delas.
5. Estreite a janela (ou *Theme… → Rail Buttons: Large*): a fila **quebra para uma segunda linha**
   em vez de perder botões.
⚠️ *Mostrar Hierarquia* e *Mostrar Inspector* **saíram** da fila de propósito — eles são layout,
não ferramenta, e vivem no menu *View*.

---

## §9 — Estado do portão de fecho

Corridos e **verdes** no momento da escrita:

- `cargo test -p ph2d-editor-core` — **1 106 + 60** testes, 0 falhas, 6 ignorados
- `cargo test -p ph2d-host-desktop --test the_node_ops_are_wired` — **15/15**
- `cargo test -p ph2d-tokens --test the_sixteen_timeline_slots_are_pure_aliases` — 1/1
- `cargo check -p ph2d-host-desktop` — limpo

### ⛔⛔ E o portão batched deu uma lição sobre si mesmo

A 1.ª corrida do `nextest-impacted.sh` foi invocada com `| tail -25` e devolveu **`exited with
code 0`**. Ela tinha **reprovado**: `Summary 3497/12017 tests run: 3496 passed, 1 failed`, e
**8 520 testes nunca correram** porque o nextest cancela na primeira falha.

- **O pipe destruiu o código de saída** — a armadilha que o repo já tem escrita
  (`project-memory/feedback_pipe_masks_script_exit_code.md`). O script preserva o exit code do
  cargo *de propósito*, e quem o anulou fui eu, no `| tail`.
- **A reprovada é um FLAKE DE CARGA já catalogado**:
  `flip_smooth::resample_measurement::precisao::orcamento::…`, membro nomeado da família no
  `CLAUDE.md` §5.0 (*«a falha MUDA de teste entre corridas»*).
- ⇒ re-corrido com `--no-fail-fast`, que é a metade que faz os 8 520 escondidos correrem. *Um
  vermelho de flake não é só um falso positivo: ele **esconde a suíte**.*

**A corrida honesta: `20 175 testes, 20 171 passaram, 4 falharam, 1 991 saltados` (179 s).**
As quatro são gates de **razão de um recurso** — a família inteira do §5.0 —, e **nenhuma vive
numa crate que este diff toca** (`git diff --name-only main..HEAD` não devolve `flip`,
`ph2d-mesh`, `soft-body` nem `tool-painter`):

| reprovada | sozinha, 3× | nota |
|---|---|---|
| `flip_smooth::…::orcamento::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` | **3/3 verde** | nomeada no §5.0 |
| `ph2d-tool-painter::…::the_mask_stroke_cost_does_not_follow_the_canvas` | **3/3 verde** | nomeada no §5.0 |
| `ph2d-node-motion-soft-body::cap_gates::the_shape_match_is_linear_in_the_mesh` | **3/3 verde** | ⏳ **não** estava na lista — mede a linearidade de dois relógios |
| `ph2d-mesh::measure_normals::measure_normals_parallel_speedup` | ⚠️ **2/3** | ⏳ **não** estava na lista, e ⭐ **reprova mesmo SOZINHA** |

⭐ **A `measure_normals_parallel_speedup` é o achado desta corrida.** Ela mede um *speedup* de
paralelismo — uma razão entre dois relógios cujo numerador depende de quantos núcleos o SO lhe
deu **neste instante** — e reprova **1 em 3** com a máquina calma. Isso põe-na numa espécie
ligeiramente pior que a família do §5.0: não é «reprova sob fan-out», é «reprova às vezes».
⛔ Não é desta linha para curar, mas o §5.0 diz que *a lista nunca estará completa* — e estas
duas são as que ela ganhou hoje.

⏳ Falta o `rm -rf target/*/incremental` e a **UMA linha** no `CLAUDE.md` §5 — DIRETRIZ §1.5.9.
A linha **não** está fechada, e não há ordem do Enio para integrar.

---

## §10 — A BARRA DE MENUS (commit 15), mecanismo

Enio, 2026-08-30: *«pode tirar também os botões do topo para começarmos a trabalhar a barra
superior»*. É a obra **`C`** do [`spec/02 §2`](../spec/02_o_que_falta_para_comecar.md) — a barra
global da **D2**.

### §10.1 — A lei: 25 das 29 linhas levam um id que JÁ EXISTIA

O `Save` é o `CTX_MENU_SAVE` do `chrome::io_menu`. O `Undo` é o `TOOL_UNDO` do trilho. O `Vector` é
o `TOPBAR_VECTOR` que o pill levava, e quem o despacha continua a ser o mesmo handler. ⇒ nasceram
**quatro** ids de linha, e só onde não havia porta nenhuma:

| linha nova | o verbo existia, alcançável por |
|---|---|
| `New Image…` | só a tecla `Cmd/Ctrl+N` |
| `Preferences…` | só o pill da engrenagem (retirado) |
| `Theme…` | só o pill do tema (retirado) |
| `Rulers` | só uma caixa **dentro do painel do vetor** — que deixou de ser o dono no dia em que as réguas passaram a valer em todos os modos |

### §10.2 — O menu *Window* devolve os treze módulos

⚠️ Entre a retirada dos pills e esta barra, o **único** caminho até Vector / Motion / Flip /
Physics / Sculpt 3D / Model 3D / Image Tools / Áudio×2 / Tokens / Authored / Galeria / Grelha era a
tecla `F9`. Uma tecla de bissecção não é uma porta de produto.

### §10.3 — UMA tabela, três consumidores

`menu_bar::MENUS` é a fonte; `menu_bar::menu_rects` é a porta. O pintor, o registo de hit e o
despacho de ponteiro perguntam à mesma função. ⛔ Em `pointer_down_menus.rs` estavam **cinco**
cópias do mesmo bloco de dez linhas (uma por pill) e a barra acrescentava mais quatro — a sexta
cópia é onde uma delas nasce com o `kind` do vizinho por copiar-colar.

### §10.4 — ⛔⛔ O fecho do menu teve de subir para o topo do `apply_event`

O registo de **painéis** é caminhado **antes** do `chrome::dispatch_all`: um
`Click(TOPBAR_AUDIO_MIXER)` é consumido pelo painel do mixer e o chrome **nunca o vê**. Um fecho
escrito num handler de chrome ficaria morto exactamente nas treze linhas do menu *Window* — o
artista escolheria *Audio Mixer*, o painel abriria, e o menu ficaria pousado por cima dele.

### §10.5 — ⛔ Os gates vivem em DUAS crates, e a divisão não é escolha

| linhas | quem despacha | crate que consegue prová-lo |
|---|---|---|
| 9 do *Window* + File/Edit/View | `chrome/*` | `ph2d-editor-core` |
| **Audio Mixer · Audio Editor · Widget Gallery · Grid Settings** | o `event.rs` do próprio painel | `ph2d-panel-registry-init` |

O `test_support::ensure_panel_registry` da `editor-core` é um `{}`. A **1.ª redacção** do gate
estava toda lá e acusou aquelas quatro de serem botões mortos — código correcto. *Um gate escrito
de uma camada deixa a outra por medir.*

### §10.6 — ⚠️ Duas correcções que os próprios gates cobraram

1. *«escolher FECHA o menu»* é **falso** para o `New Image…`, que o **substitui** pelo modal. A
   propriedade honesta é *«este menu deixa de estar aberto»*.
2. O alvo do gate dos painéis era a string `"audio-mixer"`; o painel chama-se `"audio_mixer"`. Um
   nome errado lê-se exactamente como *«a costura está morta»*, que é a acusação mais cara que um
   gate pode fazer. Hoje a pergunta é **derivada**: *que painéis mudaram de visibilidade? exactamente um.*

---

## §11 — A FILA DE FERRAMENTAS (commit 16), mecanismo

Enio, 2026-08-30: *«ainda temos os botões da lateral»* + *«os botões laterais reaparecendo como uma
barra horizontal por cima do desenho»*.

### §11.1 — ⭐⭐ A porta única da geometria, e a dívida que ela pagou

`widget::entry_rects(rail, rect, size, axis)` responde *«onde cai cada entrada?»*. Antes dela a
resposta estava escrita **três** vezes — o pintor (`tool_rail/paint.rs`), o hit do trilho
(`hero/left_rail.rs`) e o hit do flyout — cada um com o seu `let mut y`. O comentário do segundo
dizia *«Hit-rects MUST mirror exactly what `paint_tool_rail` paints»*, que é a confissão do
defeito: **um espelho não é uma lei**, e um pintor horizontal com um hit vertical compilaria e
passaria a suíte inteira.

⚠️ **A ADVANCE é a mesma nos dois eixos** (um chip anda `chip_px`, um divisor anda `1 + 2·gap`); o
que muda é o eixo em que ela corre e onde fica o rótulo — rodado à esquerda na coluna, direito por
cima na fila. Há gate a compará-los entrada a entrada.

### §11.2 — A fila é uma REGIÃO da área, não uma faixa da janela

Ela sai de `HeroLayout::tool_bar`, cortado da **área de desenho** (entre as colunas), e a régua
começa por baixo dela. ⛔ Uma barra de ferramentas à largura do ecrã passaria por cima da
Hierarquia e do Inspector — que é exactamente o modelo `x = 0` do trilho antigo, o que tapava
**86,8 %** da régua da esquerda.

### §11.3 — ⛔⛔ O achado: a fila NÃO CABIA, e o transbordo era MUDO

| modo | a fila pede | a área dá (alvo 1366, duas colunas abertas) |
|---|---:|---:|
| objecto | 358 px | 746 px |
| painter (antes) | **779 px** | 746 px ⇒ **−33** |

⚠️ **A faixa blinda a tinta E o hit** (`push_clip` nos dois), o que é o comportamento certo — e é
por isso que o transbordo é silencioso: o último verbo (*Shapes*) **desaparecia** em vez de se
sobrepor à coluna. Sem número, o primeiro verbo novo a entrar na lista apaga o último e ninguém vê
nada acontecer. Há gate com a folga impressa.

⭐ **A cura veio da D3, não de encolher nada:** *Mostrar Hierarquia* e *Mostrar Inspector* **nunca
foram ferramentas** — eles não mudam o gesto do ponteiro, mudam o **layout**, e a decisão do Enio
já os mandava para o menu *Ver* (*«~19 ferramentas · 2 layout → menu Ver»*). Saíram da lista do
trilho; a fila passa a 678 px, com **68 de folga**. *Um verbo no sítio errado só custa quando o
sítio certo fica cheio.*

### §11.4 — ⛔⛔ E isso partiu o MENU RADIAL, por índice

O radial pedia as ferramentas por `split(Divider).nth(1)` — **a secção do meio**. Com os dois
toggles fora, o índice `1` passou a apontar para *espaço/vista*, e o menu que existe para pôr **as
ferramentas** sob a caneta passou a oferecer *Frame view*. O gate dele apanhou-o.

⭐ **A cura não é corrigir o número: é deixar de haver número.** Nasceu
`left_rail::tool_section(store, painter_active)` — uma porta com **nome** —, e o radial e o
`rail_entries` pedem-lhe a mesma coisa. *«Uma fileira condicional torna todo despacho por ÍNDICE
num bug silencioso»*, um nível acima: aplicado a **secções**.

### §11.5 — ⛔⛔ O corte por LOC desarmou DOIS gates, em direcções OPOSTAS

`paint.rs` passou o tecto (708/700) e o bloco da geometria foi cortado para o irmão
`frame_layout.rs` — *pure code motion*, produto intacto. As duas espécies de gate reagiram de
maneiras opostas:

| o gate afirma | o que o corte lhe fez |
|---|---|
| **presença** (*«alguém chama isto»*) | reprovou **alto**, com uma acusação falsa |
| **ausência** (*«isto NÃO voltou»*) | ficou **verde e vazio** — a prova mudou-se para fora do alcance dele |

⚠️ **A segunda é a perigosa:** `the_side_columns_are_anchored` exigia que
`blender_picker_offset(…)` não estivesse no `paint.rs`, e depois do corte essa ausência passou a
ser **de graça** — o offset podia voltar no ficheiro ao lado com o gate verde.

⇒ nasceu `tests/common/hero_sources.rs`: a pergunta passa a ser sobre o **módulo**
(*alguém em `screens/hero` faz isto?*), varrendo recursivamente. Dois gates convertidos.

⏳ **Ficam DOIS por converter** — `the_chrome_reads_the_ui_clock` e
`the_ruler_prints_the_projects_unit` ainda fazem `include_str!` do `paint.rs`. São gates de
**presença**, logo falham alto: são um incómodo, não um buraco. Converter é trabalho de quem lhes
mexer a seguir.

### §11.6 — ⚠️ O que NÃO está gateado, nomeado

O registo de hit do trilho **vertical** não tem gate comportamental próprio — ele passou a
perguntar à porta, logo **já não consegue derivar**, mas uma re-introdução deliberada do offset só
é apanhada pelo gate de números mágicos (medido: uma mutação com `+5.0` só acorda o
`no_magic_numeric`). E o trilho vertical só pinta sob `F9`.

---

---

## §12 — ⛔⛔⛔ A AUDITORIA das duas barras: **sete defeitos**, e o pior é o mais barato de descrever

A auditoria correu sobre as entregas 15 e 16 com a suíte **verde e sem avisos**. Esse é o ponto:
**nada do que se segue era visível a um único gate do repo.**

### §12.1 — ⛔⛔⛔ O PAINTER e as DEZ ferramentas de imagem ficaram INALCANÇÁVEIS

Elas eram pintadas num sítio **só** — `paint_image_action_row`, dentro do `paint_top_bar` — e a
entrega 12 tirou aquela barra de cena. A auditoria mediu o resto da porta:

| caminho | havia? |
|---|---|
| linha de menu | ❌ o *Window* tinha o **modo**, não as ferramentas |
| paleta de comandos global | ❌ projecta o trilho, os painéis e as rows dos menus-folha |
| paleta de ferramentas do canvas | ❌ gateada em `hero_screen.is_none()` — o caminho de demo |
| atalho de teclado | ❌ nenhum handler levanta `ActivateTool` |

⇒ **o Painter era inalcançável, e com ele TODA a face de pintura da fila nova**:
`rail_shows_painter_tools()` exige `active_tool_id == Some("painter")`, que nunca podia acontecer.
Os 22 `PAINTER_RAIL_*` e os dois flyouts eram código sem forma de aparecer — *incluindo o trabalho
de transbordo que esta linha acabara de pagar.*

⭐ **Cura:** as ferramentas entram na **fila**, com o modo ligado (`Window → Image Tools`), como
uma secção depois de um divisor. Variante nova `ToolRailEntry::Glyph` (o ícone delas é um `BezPath`
de manifesto, não um `IconId`), pintada pelo **pintor canónico** com `IconButtonStyle::Plain` — há
uma cerca a exigi-lo (`canonical_icon_button`), e ela apanhou-me à primeira.
⚠️ Elas **não** entram no `rail_entries`: com a `F9` ligada o `paint_top_bar` regista os mesmos
ids, e dois rectângulos para um id no mesmo quadro é ambiguidade resolvida por ordem de pintura.
⭐ E isto cura de lado o *knob morto de 3.ª espécie* que a auditoria também nomeou: a linha
*Window → Image Tools* consumia o clique e **não tinha efeito observável**; hoje ela faz aparecer
as ferramentas.

### §12.2 — ⛔⛔⛔ As duas barras NÃO ENGOLIAM o clique: o Painter pintava ATRAVÉS delas

`chrome_hit::pointer_over_chrome` é `panel_at().is_some() || hit_index.hit().is_some()`. As barras
não publicam rect de painel e registavam **só** os títulos e os chips. Medido a 1920×1024:

| barra | faixa pintada | alvos registados | **passa** |
|---|---:|---:|---:|
| menus | 1366 × 28 | 179 px de títulos | **86,9 %** |
| fila | 1308 × 54 | 20 736 px² | **70,6 %** |

Inclui a banda do **rótulo por cima de cada chip**: clicar no nome do próprio botão não fazia nada
e ia parar à arte.

⭐ **A cura não é nova** — é o `RAIL_BACKDROP` que o trilho tem desde 2026-07-16, acrescentado
depois de um report do Enio com este sintoma exacto. A barra nova nasceu sem ele.
⛔⛔ **E o gate que devia ter apanhado mede a outra metade:**
`the_chrome_swallows_the_click_it_was_given` afirma que cada consumidor de canvas **PERGUNTA** ao
`pointer_over_chrome` — todos perguntavam. **Ninguém afirmava que o chrome REGISTA um rectângulo
que responda que sim.**

### §12.3 — ⛔⛔ O transbordo apagava chips em silêncio, e o meu gate media UMA célula

O `HitIndex::register` **descarta** um rect totalmente cortado ⇒ um chip a mais não ficava
truncado, ficava **inexistente**. Medido:

| largura | preset | colunas | resultado |
|---|---|---|---|
| 1280 | Large | 308/304 | **Undo e Redo desaparecem** |
| 1366 | Large | 308/304 | **Redo desaparece** |
| 1920 | Small | 720/720 (o `DOCK_W_MAX`, alcançável a arrastar) | **4 desaparecem** |
| 1366 | Small | 720/720 | **os dezasseis** |

⛔ **O gate que escrevi media 1366 px, preset `Small`, colunas por omissão** — uma célula.
*Uma varredura de uma célula não é uma varredura.*

⭐ **Cura: a fila QUEBRA DE LINHA** e a faixa cresce (`horizontal_lines` + `tool_bar_h(size, lines)`).
⚠️ Não há circularidade: a **largura** da área não depende da **altura** da faixa, então o
`frame_layout` resolve uma vez com a faixa a zero, lê a largura, e resolve outra vez. O gate novo
varre **4 larguras × 3 presets × 3 larguras de coluna × 2 modos**.

### §12.4 — ⛔⛔ Verbos sem porta: a lista de cenas e o rebobinar

`TOPBAR_PROJECT` (a `SceneList`, e com ela o campo de busca `CTX_SCENE_SEARCH`) e `TOPBAR_RESET`
(rebobinar) não tinham **nenhum** caminho fora da `F9`. ⭐ Curados: linha **`File → Scenes…`** e um
menu **`Run`** (Play · Pause · Rewind) — o transporte é **um** relógio (`Playhead`), e ganhou casa.

⭐⭐ **E nasceu o CENSO que faltava:** `every_topbar_verb_has_a_door_that_is_not_the_legacy_key` lê
os `TOPBAR_*` **do ficheiro de ids** (o slug sai da própria linha, não de um `to_lowercase()`
adivinhado) e exige, para cada um, uma linha de menu **ou** uma entrada com o motivo medido — com
a metade que recusa uma excepção obsoleta. ⚠️ Ele nomeia também **três mortos PRÉ-EXISTENTES**
(`TOPBAR_RIGHT_LAYERS`/`_ASSETS`/`_SCRIPT`: pintados, registados, com tooltip, e sem consumidor no
repo inteiro) e **um órfão** (`TOPBAR_PLAY_TOGGLE`, nunca pintado).

### §12.5 — ⛔⛔ Dezasseis linhas de alternância sem estado, e a lei estava escrita no ficheiro ao lado

O menu *Window* dizia **exactamente a mesma coisa** com o Vector aberto e fechado. Antes da barra a
indicação existia: o laço de reconciliação da shell força `Pressed` no pill do tool activo e o pill
lia-o. *O pill saiu; a marca não foi com ele para lado nenhum.*

⚠️ O `context_menu_overlay` **documenta esta lei**, paga na unidade de ângulo desta mesma linha:
*«fiar o clique não é fiar o ESTADO»*. A barra repetiu-a **dezasseis vezes de uma vez**.
⭐ Cura: `row_is_marked_by_button_state` (a lista dos módulos é **derivada** da tabela do *Window*)
+ `publish_toggle_state`, porque a régua vive no `HeroScreen` e quem pinta a marca só vê o `Store`.

### §12.5-bis — ⛔⛔⛔ E o censo da marca achou um `if` com um LADO MORTO, pré-existente

Curar a marca obrigou a perguntar *«ela MEXE quando se clica?»* — e a resposta foi **não, em dez
das treze**. A causa: o laço de reconciliação da shell só percorre os clusters `image_tools` e
`vector_tools` do **registry de ferramentas**, e os pills de módulo não estão em cluster nenhum
(`TOPBAR_VECTOR` é `hash_node_id("topbar_vector")`; o manifesto é `hash_node_id("vector")`).
**Ninguém escreve o `ButtonState` deles**, e nunca escreveu.

⚠️⚠️ **E isso é mais do que uma marca em falta:** o `chrome::vector_toggle` **LÊ** esse estado para
decidir a direcção — *activar* ou *cancelar*. Com ele preso em `Normal`, o segundo clique volta a
activar em vez de desligar. *Um estado que ninguém escreve e alguém lê não é uma marca em falta: é
um `if` com um lado morto.*

✅ **CURADO na entrega 18 — ver §13.** A cura é uma tabela (`menu_bar::MODULE_TRUTHS`) que pergunta
a verdade de cada módulo **onde ela vive**, com dois consumidores: a marca do menu e a **direcção**
do toggle.

### §12.6 — ⛔ O gate do relógio da UI ficou a medir código que a `F9` esconde

`the_chrome_reads_the_ui_clock` afirma que o `paint.rs` passa `&hero.motion` ao `paint_left_rail` e
ao `paint_top_bar`. **Continua verdade — dentro do ramo `if hero.view.legacy_chrome`.** A barra de
menus nova não recebia `UiMotion` nenhuma e resolvia a cor pelo `ButtonState` duro: *o defeito para
o qual o gate foi escrito, reintroduzido na superfície que substituiu a que ele guardava.*

⚠️ **É a família do §11.5 um passo pior:** ali um **corte** desarmou dois gates; aqui foi um
**ramo** — sem rename e sem ficheiro movido, nada podia falhar alto. Curado: o título lê o eixo, e
um título com o menu **aberto** fica fora dele (a lei do chip activo).

### §12.7 — ⚠️ Um número meu, errado, num doc-comment

*«Sem eles a fila usa 699 px, com 47 de folga»* — é **678 px, com 68**. O `699` é o que dá se se
tirarem os dois chips e se **esquecer o divisor que ia com eles**. ⭐ O gate IMPRIME o número certo
(`--nocapture`); eu fi-lo de cabeça ao lado de um gate que o media. Corrigido, com a lição escrita
ao lado. (Os outros dois — `779` e `86,8 %` — a auditoria reproduziu e confirmou.)

### §12.8 — O que a auditoria mediu e NÃO achou defeito

Nada no trilho para além do §12.1 · nenhuma catraca de LOC inflacionada (seis entradas
**desceram**, cada uma paga por extracção) · o `TOPBAR_LEAF_MENUS` correctamente **sem** os kinds
novos (senão a paleta duplicaria ids) · e a quebra do menu radial por índice foi real, apanhada
pelo gate dele, e a cura (`tool_section`) é a certa.

### §12.9 — ⏳ O que fica aberto da auditoria

| item | porquê fica |
|---|---|
| `the_chip_axis_has_one_door` tem uma lista de **dois** pintores escrita à mão | um terceiro pintor de chip fica descoberto por construção; é derivável e não foi derivado |
| `the_hero_paint_docks_the_timeline_into_motion` tem um `hero_sources()` **local e não-recursivo** ao lado do partilhado e recursivo | duas respostas a uma pergunta, um commit de idade; falha alto, logo é incómodo e não buraco |
| `TOPBAR_RIGHT_LAYERS`/`_ASSETS`/`_SCRIPT` mortos e `TOPBAR_PLAY_TOGGLE` órfão | **pré-existentes**, agora com endereço no censo do §12.4 |
| `cluster_painter::paint_topbar_rail_chip` continua cópia verbatim da matriz de tinta | com o `RailAxis` passa a ser dispensável; é chrome legado a caminho de sair |

---

## §13 — ⛔⛔ A VERDADE DE CADA MÓDULO passa a ter UMA tabela (entrega 18)

A §12.5-bis nomeou o defeito e deixou-o aberto. Ele fechou.

### §13.1 — O defeito tinha duas caras, e a segunda é a cara

Ninguém escrevia o `ButtonState` dos pills de módulo: o laço de reconciliação da shell só percorre
os clusters `image_tools` e `vector_tools` do **registry de ferramentas**, e um pill de módulo não
está em cluster nenhum (`hash_node_id("topbar_vector")` ≠ `hash_node_id("vector")`).

1. **a marca não aparecia** — o menu *Window* dizia o mesmo com o Vector aberto e fechado;
2. ⚠️ **e o `chrome::vector_toggle` LIA esse estado para escolher a direcção.** Preso em `Normal`,
   `currently_active` era **sempre falso**: o segundo clique voltava a **activar** em vez de
   desligar, e não havia como fechar o módulo pelo menu.

*Um estado que ninguém escreve e alguém lê não é uma marca em falta: é um `if` com um lado morto.*

### §13.2 — A cura: `menu_bar::MODULE_TRUTHS`, e a verdade perguntada ONDE ELA VIVE

| variante | o que responde | quem a usa |
|---|---|---|
| `Panel(nome)` | `is_panel_visible` | physics · model3d · tokens · authored · áudio×2 · galeria · grelha · hierarquia · inspector |
| `Tool(id)` | `image_edit.active_tool_id` | vector · motion · flip |
| `ImageMode` | `image_edit.mode_on` | Image Tools |
| `Rulers` | `view.rulers_visible` | a régua |
| `ShellOwned` | ⚠️ ninguém aqui | sculpt3d — a verdade dele é *«há barro no ecrã»*, e só a shell a vê |

⭐ **Uma tabela, DOIS consumidores:** a marca do menu (`publish_toggle_state`) e a **direcção** do
toggle (`module_is_on`). Escrever a verdade duas vezes é como as duas se separaram.

### §13.3 — ⛔ E o ESPELHO da shell servia UMA ferramenta

`ImageEditState::active_tool_id` internava contra um `match` de **um** literal (`"painter"`) e
filtrava por `mode_on`. ⇒ os três `Tool(_)` liam sempre *«não está activa»*.

⭐ Hoje a internagem vem do **registry** (`manifests()`), sem lista à mão e sem alocar — uma
ferramenta nova entra sozinha —, e o filtro `mode_on` saiu (ele pertence a quem pergunta pelo
Painter, e `rail_shows_painter_tools` já o exige).

⚠️⚠️ **E ele estava a TRÊS LINHAS dentro de um closure do `render_loop`, onde nada o media:** um
`grep -rn active_tool_id shells/desktop/` devolvia **um** ficheiro, o próprio. Foi extraído para
`active_tool_mirror.rs` **para poder ser gateado** — 2 gates, 2 mutações mortas, com controlo a
confirmar que o filtro casa testes.

### §13.4 — ⚠️ O censo teve de saber o que a CRATE dele não consegue conduzir

A 1.ª redacção acusou **nove** linhas de mentir. Sete não mentiam: a `editor-core` é que não as
alcança.

| verdade | porque não se mede lá |
|---|---|
| `Tool(_)` | o clique empurra `ActivateTool` para o **barramento**; quem drena é a shell |
| `ShellOwned` | não há flag a conduzir |
| clique **não consumido** | o handler vive numa **crate de painel** |

⇒ a exclusão é **derivada da tabela e do valor de retorno**, não uma lista escrita à mão, e as
metades que faltam moram onde correm: as quatro de painel em `ph2d-panel-registry-init`, a
**decisão** dos três `Tool(_)` na própria `editor-core` (semeando o espelho, que é a fronteira da
crate), e o espelho na shell.

⭐ *A lista de pendentes desceu de nove para zero no mesmo dia em que nasceu — e as sete que saíram
não foram curadas: foram medidas no sítio certo.*

---

## §14 — ⭐ A FUGA DO GIZMO ficou INERTE, e a lei ficou (entrega 19)

A **D1** manda retirar a fuga do gizmo de navegação no mesmo trabalho que ancora os painéis:
*«ela é o remédio do sintoma; com os painéis fora da vista passaria a fugir de uma moldura que já
não a alcança»*. O `00_DECISOES_DO_ENIO.md` chama-lhe **remédio duplo**.

### §14.1 — A cura não foi apagar a lei: foi dar-lhe a ÁREA CERTA

`field3d_navball::safe_corner` recebia **o viewport inteiro**, que as colunas docadas tocam — por
isso elas empurravam o gizmo. Hoje ela recebe a `HeroLayout::draw_area`, que **começa depois delas**:
uma coluna docada deixa de a alcançar e a fuga fica **inerte por construção**, sem uma linha de lei
mudar.

⛔ **A lei FICA, e não por preguiça:** o que ainda a alcança são as janelas que **declaram flutuar**
(Grid Snap, galeria de widgets), e a lei dela já diz que só conta quem toca a **aresta**. Apagá-la
deixaria o gizmo por baixo de uma dessas. ⇒ *a D1 pede que a fuga deixe de ser necessária para o
chrome docado, não que o app perca a defesa contra o que flutua* — e há gate para os dois lados.

### §14.2 — ⛔⛔ E o meu 1.º gate era do lado errado da costura, medido

O gate que escrevi passava a área **à mão** ao `safe_corner` e afirmava que as colunas não a movem.
Ele ficou verde — e a mutação que devolve **o viewport** ao produto **SOBREVIVEU**.

*Um gate sobre a LEI não é um gate sobre quem a ALIMENTA.* É a mesma família do
`the_chrome_swallows_the_click_it_was_given` (§12.2), que afirmava que todo consumidor **pergunta** e
nunca que alguém **responde** — e a segunda vez que ela morde nesta linha.

⇒ a decisão *«qual área?»* saiu do laço de render para `field3d_navball::area_for`, que é uma função
com dois gates e duas mutações mortas. ⚠️ E o gate da lei ficou **com controlo**: os mesmos
rectângulos, medidos contra a área ANTIGA, **têm** de mover o gizmo — senão o teste passaria com a
lei apagada, com a área a zero, ou com obstáculos que não tocam nada.

---

## §15 — ⭐⭐ OS SEIS ENCAIXES: a D1 deixa de ser prosa e passa a ser um TIPO (entrega 20)

É a obra **`A`/`E`** do [`spec/02`](../spec/02_o_que_falta_para_comecar.md), na fatia que o §7 dela
recomenda: *os seis encaixes + o descritor de painel + o gate que prova que um painel de
propriedades **não consegue** ser posto sobre a viewport*.

### §15.1 — O vocabulário

`screens::slot::{Slot, SlotSet}` — **seis** encaixes (`LeftTop`/`LeftBottom`/`RightTop`/
`RightBottom`/`Bottom`/`Center`), e o número é **derivado**: os 12 do Godot são duas colunas por
lado = **89,6 %** da largura do alvo de 1366 (spec §2). `SlotSet` é um bitset porque tem de viver
numa **constante associada** de trait.

O `Panel` ganha três constantes **com default**, para os 24 painéis não mudarem no mesmo commit:
`ALLOWED_SLOTS` (default `ANY_DOCK`, ⛔ **nunca o centro**), `DEFAULT_SLOT`, `CAN_FLOAT` (default
`false`).

### §15.2 — ⭐ A declaração foi MEDIDA antes de escrita

Uma sonda pintou o quadro com **todos** os painéis abertos e leu os rects publicados:

| resultado | painéis |
|---|---|
| `overlap = 0` com a área de desenho | 14 (inspector, hierarchy, physics, tokens, vector, model3d, mixer, …) |
| `overlap > 0`, e **declaram** flutuar | `grid_snap` · `widget_gallery` · `authored` |
| `overlap > 0` e **não** declaram | ⛔ **`audio_editor`** |

⇒ `CAN_FLOAT = true` foi escrito nos **cinco** que de facto se arrastam (os três acima + o
`wet_tuning` e a `timeline`, que têm rect próprio com clamp na crate deles).

### §15.3 — ⛔ O gate achou UM violador, e ele é uma decisão de modelo

O `audio_editor` encaixa-se **a oeste do Inspector** (`insp.x − 240 − gap`) para poder estar aberto
ao lado do Audio Mixer, que ocupa a coluna. Isso é uma **segunda coluna da direita** — e a spec §2
recusa-a por aritmética. ⇒ a cura é a **regra 1** do modelo (*`n > 1` num encaixe são **abas***), que
é wave própria. Fica na catraca `REACHES_PENDING`, **com o mecanismo**, e a metade de baixo do gate
recusa a entrada no dia em que ela deixar de descrever algo.

### §15.4 — ⚠️ E o gate obrigou a nomear uma propriedade do DESENHO

`DockSides::from_published` lê os rects do quadro **anterior**: no 1.º quadro nenhuma coluna está
reservada e a área de desenho ocupa a **largura toda** (medido: `x=0, w=1366` no primeiro,
`x=308, w=754` do segundo em diante). ⇒ um gate de um quadro só mediria o estado transitório e
acusaria toda a gente — ele pinta **três** e afirma que a área assentou antes de medir.

⚠️ **Isto é também um facto de produto**, agora nomeado: no quadro em que um painel lateral aparece,
a régua e a fila de ferramentas ocupam a largura toda e encolhem no quadro seguinte. Um piscar de
16 ms que ninguém reportou — mas que fica escrito, porque é a próxima coisa que alguém vai ver e não
saber explicar.


---

## §16 — ⭐⭐⭐ AS ABAS: `n > 1` num encaixe partilham a coluna (entrega 21)

Commit `405408c31`. A **regra 1** do modelo de áreas (`spec/01_modelo_de_areas.md` §2), que o §15
tinha deixado nomeada como *«a wave seguinte natural»*.

### §16.1 — O defeito medido, e o que a medição achou por baixo dele

O `audio_editor` encaixava-se **a oeste** do Inspector (`insp.x − 240 − gap`) para poder estar
aberto ao lado do `audio_mixer` — uma **segunda coluna da direita**, que a spec recusa por
aritmética (duas colunas por lado = **89,6 %** de 1366). Ele publicava **168 480 px²** sobre a área
de desenho, e era a única entrada da catraca do gate da D1.

⚠️⚠️ **E ele não era o único a partilhar a coluna — era o único a fazê-lo às escondidas.** Medido
em 2026-08-30 com tudo aberto:

| rect publicado | quantos painéis |
|---|---:|
| `(1062, 28, 304, 996)` — a coluna da direita, ao pixel | **13** |
| `(0, 28, 308, 996)` — a coluna da esquerda | 1 |
| `(322, 784, 726, 240)` — a faixa de baixo | 1 |
| rect próprio (declaram `CAN_FLOAT`) | 3 |

Os treze não colidiam **por convenção** (só um está visível de cada vez, conduzido pela ferramenta
activa) e **nada no repo o afirmava**. ⇒ *as abas não introduzem a partilha: elas tornam visível a
que já existia, e dão-lhe um gesto.*

### §16.2 — ⭐ A selecção NÃO é estado novo

A aba escolhida é **o ocupante mais ao topo da ordem z**, e clicar numa aba é `bump_panel_z` — o
mesmo verbo que clicar dentro do painel já usava. Guardar *«qual aba está escolhida»* ao lado da
ordem z seria a segunda resposta à mesma pergunta, e as duas divergiriam no primeiro clique que uma
delas não visse.

⚠️ **Isso obrigou a ordem z a ser reconciliada com a visibilidade** (`slot_tabs::reconcile_z`), e as
**duas** metades são obrigatórias:

| metade | sem ela |
|---|---|
| **poda** (esquece quem já não está em cena) | fechar e reabrir devolve o painel à posição velha — atrás de uma aba que ninguém tocou |
| **acrescento** (quem apareceu vai ao topo) | um painel acabado de abrir nem entra na ordem, e o `PANEL_Z_ORDER_FALLBACK` põe-no no fundo |

⭐ De borla: **abrir um painel flutuante passa a levantá-lo**, o que antes não acontecia.

### §16.3 — ⛔ O `DEFAULT_SLOT` era decoração, e TRÊS painéis mentiam

Ele nasceu na entrega 20 **com default `RightTop`**. Medido: **20 dos 21** painéis herdavam-no, e
três mentiam — `hierarchy` publica a coluna da **esquerda**, `timeline` e `flip_frames` a faixa de
**baixo**. *Uma declaração que ninguém confronta com a realidade é decoração*, e ela só passou a
custar quando as abas começaram a derivar dela quem divide o quê.

⇒ o default **morreu** (cada painel declara), e o gate `the_slot_a_panel_declares_is_where_it_paints`
confronta a declaração com o rect publicado.

⚠️ **E o `motion_graph` É o centro.** Ele parte a área de desenho em duas regiões **irmãs**
(`CenterSplit`) — literalmente o que a decisão **D5** diz que uma região é. A regra do gate era
*«ninguém declara o CENTER»* e estava **errada por omissão**; hoje é *«quem declara o centro declara
SÓ o centro»* mais *«no máximo um reclamante»*. Ele declarava `RightTop` só porque havia default, e
as abas iriam lê-lo como ocupante da coluna da direita, onde ele nunca esteve.

### §16.4 — A geometria: os seis encaixes passam a ter RECT

- **`HeroLayout::slot_rects(occupied)`** — a metade de uma coluna só existe quando a **irmã** está
  ocupada. Dividir uma coluna com um ocupante só tirar-lhe-ia metade da altura por uma razão que não
  existe. ⛔ Deriva de `side_columns()` (ordenado por `x`): ler `layout.hierarchy` **pelo nome** daria
  os encaixes espelhados em silêncio sob `ui_mirrored`.
- **`HeroLayout::reserve_slot_tabs(counts, h)`** — a faixa sai do encaixe e o que estava debaixo dela
  **desce**. ⛔ A regra é **derivada** (*todo rect docado que TOCA a faixa começa onde ela acaba*) e
  não uma lista de campos: são **cinco** campos que são o mesmo rect da coluna da direita
  (`inspector`, `bgremoval`, `padding`, `painter_sidebar`, `painter_layers`), e uma sexta futura
  desenharia **por baixo** das abas sem nenhum gate a ver (o rect publicado continua dentro da coluna).
  ⚠️ O `flip_strip`, que é baixo e vive no fundo da faixa inferior, **não** é empurrado — e está certo:
  ele nunca esteve debaixo das abas.
- **`Panel::TITLE`, sem default** + gate `the_tab_and_the_menu_call_a_panel_the_same_thing`, conduzido
  pela tabela `MODULE_TRUTHS` que a entrega 18 criou. Um derivado do `ID` daria *"Tokens"*,
  *"Sculpt3d"* e *"Grid Snap"* onde o menu *Window* diz *"Design Tokens"*, *"Sculpt 3D"* e
  *"Grid Settings"*: **três divergências à nascença**, sem uma linha de erro.
- **`tab_node_id` = XOR com um sal.** A aba e o painel são controlos diferentes com rects diferentes,
  e o `HitIndex` mapeia `id → rect`: o mesmo id faria o segundo apagar o primeiro. ⚠️ XOR é uma
  **bijecção**, logo o derivado não pode criar colisão que o espaço de ids de painel já não tivesse —
  *uma segunda função de hash, sim, poderia*.
- **`MIN_TAB_W = 64`** é o piso de **legibilidade**, não de layout: com sete ocupantes numa coluna de
  304 px cada aba teria 43 px. Abaixo do piso as abas **transbordam**, e as que não cabem continuam
  alcançáveis pelo menu *Window* — que é onde um painel sempre se abriu.

### §16.5 — ⛔⛔ Duas leis que a implementação cobrou

**1. `with_registry_opt`, nunca `with_registry_ref`.** O `pre_populate` corre dentro do
`HeroScreen::new`, e nem toda a gente que constrói um hero instalou o registry de painéis — na
própria `ph2d-editor-core` o `test_support::ensure_panel_registry` é um `{}`. A variante `_ref` faz
`panic!`, e **12 testes de chrome desta crate morreram** (paleta de comandos, modais de fill e de
onion) — nenhum deles tem a ver com painéis. *Uma leitura obrigatória de um recurso opcional
transforma um serviço em requisito, e quem paga é quem nunca o pediu.*

**2. ⚠️⚠️ A feature nova ESVAZIOU a população de dois gates meus.** O gate da D1 pintava *«tudo
aberto»* e lia os rects publicados — e com abas **doze dos treze** ocupantes da coluna deixam de
publicar. A varredura passou de **18 medidos para 2**, e o gate **continuaria verde**. Foi o controlo
`measured >= 10` que o disse, escrito na entrega 20 por outra razão.

⇒ os dois gates passam a medir **um painel de cada vez** (abrir só ele, pintar, ler). *Um gate cuja
população uma feature nova esvazia passa a medir nada, sem uma linha de erro* — e o único aviso
possível é um controlo de população escrito antes.

### §16.6 — A catraca DESCEU

`REACHES_PENDING` do gate da D1 fica **vazia**. ⭐ E desceu porque a **metade de obsolescência** a
acusou primeiro (`audio_editor (já não publica rect)`) — a metade que a `CLAUDE.md` §5.0 exige, e
que aqui pagou o custo dela pela primeira vez nesta linha.

### §16.7 — Verificação

| portão | resultado |
|---|---|
| `ph2d-editor-core` | **1344** ✓ · 16 ignorados |
| `ph2d-panel-registry-init` | 11 alvos, **0** falhas |
| `ph2d-host-desktop` | **4763** ✓ · 266 ignorados |
| `cargo check --workspace --all-targets` | limpo |
| clippy `--all-targets` (6 crates do diff) | limpo |
| `fmt` · tecto de LOC · gates de doc | ✓ |

⚠️ **O `hero.rs` bateu em `701/700`** e o corte foi por **responsabilidade**, não por folga:
`hero/pre_dispatch.rs` — *o que corre antes do registry de painéis, e por que essa ordem é
load-bearing* (as linhas do menu *Window* e as abas têm ids que um painel consome).

**Seis mutações MORTAS**, cada uma com controlo no próprio filtro:

| mutação | gate que a matou |
|---|---|
| `hidden_by_tabs` devolve vazio | `the_mixer_and_the_editor_share_one_column_as_two_tabs` |
| a reserva de abas não empurra ninguém | `every_docked_layout_rect_is_pushed_by_a_tab_bar` |
| clicar numa aba não levanta o painel | `clicking_a_tab_changes_which_panel_draws` |
| o painel de tokens volta ao nome derivado | `the_tab_and_the_menu_call_a_panel_the_same_thing` |
| a hierarquia volta a declarar a coluna da direita | `the_slot_a_panel_declares_is_where_it_paints` |
| a ordem z deixa de ser reconciliada | `the_mixer_and_the_editor_share_one_column_as_two_tabs` |

⚠️ **O controlo apanhou um `NÃO COMPILA` transiente** na 1.ª corrida de um dos alvos — o que tornaria
a 4.ª «morte» indistinguível de um erro de compilação. Ela foi **re-verificada isolada**, com dois
controlos verdes antes e a mensagem de falha a nomear exactamente `tokens: o menu diz "Design Tokens"
e a aba diz "Tokens"`.

### §16.8 — ⏳ O que esta entrega deixa nomeado

- **Arrastar um painel de um encaixe para outro** não existe: `ALLOWED_SLOTS` já descreve para onde
  ele PODERIA ir, e `DEFAULT_SLOT` é hoje a resposta final. O gesto é wave própria.
- **A ordem das abas** é a ordem z, logo ela muda quando se troca de aba. Reordenar por arrasto (o
  que o Godot faz) é outra wave.
- **As metades de coluna (`LeftBottom` / `RightBottom`) não têm ocupante nenhum** — a lei está
  escrita e gateada, e nada a exercita no produto até um painel as declarar.
- **Um painel escondido por aba não corre o `paint_fn`**, logo não publica `content_h` nem clampa
  scroll nesse quadro. É o que «não visível» sempre significou; fica nomeado porque a próxima
  pessoa a ler um `content_h` velho vai querer saber porquê.
- **Duas barras de título empilhadas**: a fila de abas fica por cima de um painel que ainda pinta o
  próprio cabeçalho. Não é defeito de mecanismo — é decisão de desenho de quem vê.

---

## §17 — ⛔⛔⛔ «COM O MODEL TUDO VIRA CANVAS»: havia DUAS portas para a mesma pergunta (entrega 22)

Commit `b03f96817`. Report do Enio no smoke da entrega 21: *«quando coloco Model, não consigo mais
clicar nos menus superiores nem nas abas. É como se tudo fosse canvas.»*

### §17.1 — A causa não era o módulo 3D

| porta para *«isto é moldura ou desenho?»* | como responde | quem perguntava |
|---|---|---|
| `chrome_hit::pointer_over_chrome` | o **índice de acerto** — o que o chrome pintou neste quadro | todo o resto do app |
| ~~`forwarding::cursor_over_hero_chrome`~~ | uma **lista de 4 ids de fundo escrita à mão** | só o `field3d` e o `sculpt3d` |

Os dois módulos 3D correm **antes** do despacho de chrome (`input_dispatch.rs`) e reclamam o gesto;
a guarda deles era a lista. Quando a barra de pills saiu (§10) e a barra de menus (§11), a fila de
ferramentas (§12) e as abas (§16) entraram:

| entrada da lista | estado em 30/08 |
|---|---|
| `RAIL_BACKDROP` | **viva** — a fila de ferramentas reusa o id (por isso ela continuava a funcionar) |
| `TOPBAR_LEFT_BACKDROP` · `TOPBAR_RIGHT_BACKDROP` · `TOPBAR_IMAGE_TOOLS_BACKDROP` | **mortas** — a barra legada só é pintada sob `F9` |
| `MENUBAR_BACKDROP` · a fila de **abas** | **descobertas** — nasceram fora da lista |

⇒ o clique na barra de menus e nas abas ia para a cena 3D, que o consumia e voltava.

### §17.2 — ⭐ A cura foi APAGAR a segunda porta

Completá-la funcionaria no dia em que fosse escrita e voltaria a apodrecer na wave seguinte — que é
literalmente o que aconteceu. Os **quatro** pontos de chamada (`field3d`/`sculpt3d` × `down`/`wheel`)
passam a `chrome_hit::pointer_over_chrome`, e `cursor_over_hero_chrome` + `hero_chrome_backdrop_at`
foram **removidos**.

⭐⭐ A porta única é estritamente melhor em três eixos:

1. **Não apodrece** — pergunta ao índice, que é escrito por quem pinta; uma faixa nova fica coberta
   no dia em que é pintada.
2. **Cobre painéis por `panel_at`**, os rects publicados, em vez de uma lista de ids de painel.
3. **Sabe excluir o GIZMO** (desenhado *sobre* a obra, e por isso não é moldura) — subtileza que o
   `chrome_hit` documenta e que uma regra de *«o índice reclamou ⇒ é UI»* estragaria, carvando zonas
   mortas onde o artista mais pinta.

⭐ E curou de borla o **irmão** que um doc-comment do `field3d` nomeava como *«um irmão por curar,
que não é desta linha»*: o `sculpt3d` tinha exactamente a mesma forma. *Uma nota não cura; uma porta
cura.*

### §17.3 — ⛔⛔ O gate que existia para isto falhou EM SILÊNCIO, por duas razões

`every_chrome_backdrop_is_known_to_the_scene` existe **exactamente** para recusar um `*_BACKDROP`
novo que nasça fora da lista. O `MENUBAR_BACKDROP` nasceu fora dela na mesma semana e o gate não se
mexeu:

1. ⚠️ **Ele varria UM SUBDIRETÓRIO** (`crates/ph2d-editor-core/src/ids/chrome/`) e o id novo foi
   escrito em `ids/menubar.rs`, **uma casa acima**. *Um gate que varre um diretório afirma sobre o
   diretório, não sobre o repo.*
2. ⚠️ **O piso `found >= 4` não o salvou:** ele foi satisfeito pelos quatro fundos **legados**, que
   continuam *declarados* mesmo já não sendo pintados por ninguém. *Um piso contado sobre
   DECLARAÇÕES não nota que as declarações deixaram de ter consumidor.*

⇒ hoje a varredura é da **árvore inteira** de ids (com um controlo sobre `files.len()`), o piso
subiu, e a lista **mudou de dono**: ela é a dos obstáculos que o gizmo de navegação contorna (W50),
não a porta da cena.

### §17.4 — Os gates novos, nas DUAS metades da costura

| gate | onde | o que afirma |
|---|---|---|
| `the_app_frame_is_reachable_by_the_hit_index` | `ph2d-panel-registry-init` (VIVO) | a barra de menus, a fila de ferramentas, a fila de abas e as duas colunas — **mais cada título e cada aba, um a um, no próprio centro** — respondem *moldura* |
| `the_scene_asks_the_one_chrome_door` | `shells/desktop` (FONTE) | as **quatro** portas de cena perguntam à porta única · a porta velha não renasce · o `pointer_up` **não** pergunta |

⚠️ **As duas metades são precisas:** a viva afirma que **há o que recusar**, a de fonte que **alguém
pergunta**. Uma sozinha fica verde com a outra partida.

⛔ **E a 1.ª versão do gate de fonte perguntava ao FICHEIRO — uma mutação SOBREVIVEU.** Apagar a
pergunta do `field3d_pointer_down` deixava-o verde, porque o `field3d_wheel`, no mesmo ficheiro,
ainda a fazia. Hoje ele pergunta **por função**. *É a mesma lição que o
`the_sculpt_gesture_is_wired` já tinha pago, no ficheiro ao lado, com a mesma forma* — e ela voltou
porque eu escrevi um gate novo em vez de olhar para o vizinho.

### §17.5 — E uma catraca DESCEU sozinha

O `MENUBAR_BACKDROP` estava em `NO_CONSUMER_PENDING` (`the_painted_control_reaches_a_consumer`) como
*«termina por AUSÊNCIA»* — verdade no dia em que foi escrita. Ao entrar em `CHROME_BACKDROPS` ele
ganhou um consumidor **positivo** (o gizmo contorna o rect dele), e a **metade de obsolescência**
acusou a linha no mesmo dia. *A dívida foi paga por uma wave que não estava a olhar para ela* — e
sem essa metade a catraca guardaria para sempre uma nota que já não descreve nada.

### §17.6 — Verificação

| portão | resultado |
|---|---|
| `ph2d-host-desktop` | **4765** ✓ · 266 ignorados |
| `ph2d-editor-core` | **1344** ✓ · 16 ignorados |
| `ph2d-panel-registry-init` | 12 alvos, **0** falhas |
| `cargo check --workspace --all-targets` · clippy · fmt | limpo |

**Quatro mutações MORTAS**, com controlo: o fundo do menu não registado · as abas não registadas ·
o `pointer_down` do `field3d` sem a pergunta · a porta velha a renascer no `forwarding`.

### §17.7 — ⏳ O que fica nomeado

- **O HUD do fundo flutua DENTRO da área de desenho** e não regista rect no estado medido — se
  algum dia registar, a porta única cobre-o automaticamente. Fica escrito porque era a única
  superfície que a lista antiga também não cobria, e ninguém tinha reparado.
- **`cursor_over_hero_panel` continua a ser uma lista de ids de painel escrita à mão**, com três
  gates a apontar-lhe. Ela já não está no caminho da cena 3D (a porta única usa `panel_at`), mas é
  a mesma espécie — e a wave que a apagar deve ler o §17.1 primeiro.

---

## §18 — ⭐⭐⭐ O ARTISTA MOVE UM PAINEL: a posição deixa de ser constante (entrega 23)

Commit `aa07ef132`. A decisão **D4** entregue como gesto — a metade que o §16.8 tinha nomeado como
*«não existe: o `ALLOWED_SLOTS` já descreve para onde ele poderia ir, falta o gesto»*.

### §18.1 — O gesto, e onde a D1 muda de natureza

Arrastar a aba de um painel para outra coluna move-o. Um **toque** continua a trocar de aba; a
distância percorrida separa os dois.

⭐⭐ **E aqui a D1 deixa de ser uma verificação:**

> *«O erro não é detectado, é **inexprimível**.»*

Um encaixe que o painel não permite **não é oferecido** — não se pinta, não se testa, não existe
para aquele gesto. ⛔ A alternativa (aceitar a largada e depois recusá-la) é a forma que o Enio
nomeou como errada: *o artista faz o gesto, vê a resposta e não sabe porquê.*

E as declarações passam a **dizer alguma coisa**:

| painel | declara | porquê |
|---|---|---|
| coluna de propriedades | `SIDES` | a faixa de baixo tem 240 px; uma lista ali fica com duas linhas |
| **tira** (Flip, timeline) | `BOTTOM` | numa coluna de 304 px ela mostraria dois quadros |
| grafo do Motion | `CENTER` | ele **é** o centro (§16.3) |

### §18.2 — ⛔⛔ O refactor que o gesto OBRIGOU: `PaintCtx::slot`

Mover o painel mudava **quem o contava** e não **onde ele pintava** — porque cada painel lia um
campo do layout **com o nome de outro painel**:

`layout.inspector` · `layout.hierarchy` · `layout.padding` · `layout.bgremoval` ·
`layout.painter_sidebar` · `layout.painter_layers` · `layout.timeline` · `layout.flip_strip`

**Oito nomes para a mesma pergunta** — *onde é que eu fico?* —, cada um com o nome do painel que ali
morava quando a posição era fixa. ⚠️ Enquanto ela era constante, lia-se bem. No dia em que o artista
**move** um painel, um campo chamado `inspector` lido pelo painel de Física deixa de poder estar
certo.

⇒ **21 crates convertidas para `ctx.slot`**, uma linha cada. ⚠️ Um painel que **flutua** também o
lê: para ele é a posição de **nascimento**, e o encaixe que ele declara é a resposta mais
significativa que existe.

### §18.3 — ⛔⛔ Duas coisas que a MEDIÇÃO refutou, as duas escritas por mim com confiança

**1. A supressão do clique.** Eu escrevi *«uma aba arrastada não é uma aba clicada»* com um
comentário a explicar o cenário — e **a mutação que a apagava SOBREVIVEU**. Ao olhar: o cenário já
estava coberto, porque o `apply_click` só dispara com `still_hot`, isto é com a largada **dentro** do
rect da aba onde o dedo desceu; arrastar para outra coluna nunca produziu clique nenhum.

⚠️ O que a supressão de facto fazia era **matar o empurrão**: o limiar são poucos pixels e um dedo
que carrega mexe-se sempre um pouco ⇒ trocar de aba passava a **depender da firmeza da mão**, que é
a pior espécie de defeito de interface. O gate `a_five_pixel_nudge_on_a_tab_still_switches_it`
nasceu **vermelho** com ela. *Código inerte com um comentário confiante é pior que código ausente.*

**2. `SlotSet::ANY_DOCK` no arnês.** Ele contém as **duas metades** de cada coluna, e a lei do
`slot_rects` é *«a metade só existe quando a irmã está ocupada»* ⇒ pedir `ANY_DOCK` **parte a coluna
ao meio**. Medido: o corpo do painel do Motion caiu para **346 px** e **26 nós** passaram a
«transbordar» sem uma linha de produto se mexer. *Um conjunto de ocupação não é uma lista de sítios
possíveis: é quem lá está.*

### §18.4 — O limiar tem consequência, e ela não é o resultado do gesto

A primeira ronda de mutação deixou-o **sobreviver**: largar sobre o próprio encaixe é um no-op, logo
apagá-lo não muda estado nenhum. ⭐ O que ele decide é **o que se vê**: sem ele, pousar o dedo numa
aba acende as colunas todas e apaga-as ao levantar — *um piscar que nenhuma asserção de estado final
apanha*. Gate `a_still_press_on_a_tab_never_lights_the_drop_zones`, com o controlo da metade
positiva (passado o limiar, elas **têm** de acender).

### §18.5 — Verificação

| portão | resultado |
|---|---|
| `ph2d-editor-core` | **1344** ✓ |
| `ph2d-host-desktop` | **4765** ✓ |
| `ph2d-panel-registry-init` | 13 alvos, **0** falhas |
| `cargo check --workspace --all-targets` · clippy `--workspace --all-targets` · fmt | limpo |

⚠️ O `paint.rs` bateu **703/700** e o corte foi por responsabilidade: `hero/panel_walk.rs` —
*quais painéis pintam este quadro, e onde*.

**Oito mutações mortas** com controlo. ⭐ **Duas sobreviveram, e as duas mandaram mudar o PRODUTO,
não o gate** (§18.3.1 e §18.4).

### §18.6 — ⏳ O que fica nomeado

- ⛔ **A arrumação NÃO sobrevive ao fecho do app.** O mapa de excepções vive no `WidgetStore`, que
  não é gravado. A D4 diz que *«um layout é `{encaixe → [painéis], posição das divisórias}`»* e é
  **trivialmente serializável** — o sítio natural é o `~/.ph2d/prefs.txt`, que já é lido e escrito.
  Wave própria, pequena.
- **A ordem das abas dentro de um encaixe** é a ordem z, logo muda ao trocar de aba. Reordenar por
  arrasto (o que o Godot faz) é outra wave.
- **As metades de coluna** (`LeftBottom` / `RightBottom`) continuam sem ocupante possível: nenhum
  painel as declara sozinhas, e `SIDES` inclui-as, então uma largada na metade de baixo resolve
  para ela — mas como a irmã não está ocupada, a lei devolve a coluna inteira e o resultado
  vê-se igual. ⚠️ *Isto é o modelo a funcionar, não um bug* — mas é a próxima coisa que alguém vai
  achar estranha.
- **A largura da coluna não muda com o conteúdo**: mover um painel largo para uma coluna estreita
  dá-lhe a largura da coluna. A D4 promete arrastar a **divisória**, e isso já existe
  (`DOCK_SEAM_PX`).

---

## §19 — ⭐⭐ A ARRUMAÇÃO SOBREVIVE AO FECHO (entrega 24)

Commit `7e64b09d4`. `~/.ph2d/layout.txt` — a frase da **D4** ao pé da letra (*«um layout é
`{encaixe → [painéis], posição das divisórias}`»*), e a metade que o §18.6 nomeava como aberta.

### §19.1 — O ficheiro, e onde ele mora

`slot.<painel>=<encaixe>` — **só as excepções**, quem o artista moveu — mais `dock_w_left` /
`dock_w_right`, a divisória que ele arrastou. Instalado **antes do primeiro quadro**; gravado
quando muda.

⚠️ **Guardar só as excepções** é o que faz um painel que nasce amanhã ir para onde ele próprio
declara, **sem uma linha de migração**.

**Por que um ficheiro próprio e não uma chave no `prefs.txt`:** o `Prefs` é `Copy` (três escalares) e
o espelho que decide *«isto mudou?»* é um `Cell<Option<Prefs>>`. Uma arrumação é uma **colecção** de
tamanho variável — pô-la ali obrigaria o tipo a deixar de ser `Copy`. ⇒ o irmão certo é o
`palette_persist`, que já resolve esta classe. ⚠️ *Não é uma casa nova: é a terceira gaveta da que já
existe* (mesma pasta, texto, std-only, best-effort, espelho por hash FNV).

### §19.2 — ⛔ A validação vive na LEITURA, nunca na escrita

Um encaixe que o painel **já não permite** é saltado ao instalar; um painel que já não existe também.
O ficheiro é do artista, mas o `ALLOWED_SLOTS` é do **produto**: se uma wave estreitar o que um
painel aceita, uma arrumação gravada não pode ressuscitar um sítio onde ele deixou de caber.
*O ficheiro pode ser mais velho que a regra.*

### §19.3 — ⚠️ `dock_width` e `dock_width_choice` não são a mesma leitura

| leitura | devolve |
|---|---|
| `dock_width(side)` | **sempre um número** — o default quando ninguém arrastou |
| `dock_width_choice(side)` | **a escolha**, ou `None` |

⛔ Persistir a primeira escreveria o default **como se fosse uma decisão do artista** — e no dia em
que o default mudasse, toda arrumação gravada continuaria a prender a coluna no número velho sem
ninguém ter pedido nada. Mutação a provar.

### §19.4 — ⛔⛔ Um comentário meu dizia o CONTRÁRIO do que o código fazia

Escrevi *«o primeiro quadro NÃO grava»* com a condição inline `c.get() != Some(h)` e o espelho a
arrancar em `None` — o que é **sempre verdade**. ⇒ o ficheiro era reescrito no arranque de **toda**
sessão, inclusive de uma em que o artista não tocou em nada (e com o disco cheio, um erro por sessão
sobre um facto que ninguém mudou).

⭐ A cura não foi corrigir a condição: foi dar-lhe **nome** (`should_save`), que é o que a torna
gateável. O gate `the_first_observation_of_a_session_never_writes` é hoje o dono da frase.

⚠️ **É a segunda vez na mesma jornada** que um comentário confiante meu descreveu o oposto do código
(a outra: a supressão do clique, §18.3.1). As duas foram apanhadas por coisas diferentes — uma por
mutação, outra por reler ao escrever o gate — e a lição comum é a mesma: *uma decisão dentro de um
hook não é gateável, e por isso não é confrontada.*

### §19.5 — Gates

| gate | onde | o que afirma |
|---|---|---|
| 8 em `src/layout_persist_tests.rs` | shell | ida-e-volta · **cada campo move o hash** · lixo saltado sem envenenar o resto · ficheiro ausente = omissão · ordem normalizada · a primeira observação não grava · a arrumação volta **mas o encaixe proibido não** · o que está instalado é o que se grava |
| 3 em `tests/the_arrangement_is_read_at_boot_and_written_on_change.rs` | shell (FONTE) | alguém **instala** antes do primeiro quadro · alguém **grava** no hook dos outros dois inquilinos · a decisão tem nome |
| `every_slot_survives_a_round_trip_through_its_wire_name` | editor-core | ⚠️ o nome de ficheiro é `snake_case` e **não** o `Debug`, que muda quando alguém renomeia a variante — e nesse dia toda arrumação gravada cairia no default, em silêncio |

⛔ **A segunda linha é a que impede a feature de não existir:** o `layout_persist` podia estar
inteiro, testado e correcto, e o artista continuar a perder a arrumação — bastava ninguém o chamar.

**Cinco mutações mortas** com controlo: a validação do `ALLOWED_SLOTS` na leitura · a ordem deixar de
ser normalizada · as larguras saírem do hash · o `current` gravar a largura efectiva em vez da
escolha · a primeira observação passar a gravar.

### §19.6 — Verificação

`ph2d-editor-core` **1345** ✓ · `ph2d-host-desktop` **4776** ✓ · `ph2d-panel-registry-init` 0 falhas ·
`check --workspace --all-targets` e clippy limpos · fmt.

⚠️ O `chrome_ops.rs` bateu **706/700** e o corte foi por responsabilidade
(`state/dock_width_ops.rs` — *a largura das duas colunas*). ⚠️ **E a minha inserção tinha caído ENTRE
um doc-comment e a função dele, em dois sítios** — os dois reparados no mesmo corte. *Um `///` órfão
não falha nada e passa a descrever o vizinho errado.*

### §19.7 — ⏳ O que fica nomeado

- **A arrumação é UMA.** A D7 pede **oito LAYOUTS** nomeados por tarefa (*Editor 2D · Editor de
  Texto · Runtime · …*), e o ficheiro de hoje guarda um só. O formato aguenta a extensão (uma secção
  por layout), e o que falta é o **selector** — que é a decisão D3 e wave própria.
- **A ordem das abas** continua a ser a ordem z e não é gravada: reabrir o app mostra os ocupantes na
  ordem do registry, com o último raiz à frente.

---

## §20 — ⛔⛔⛔ «VOLTOU AO ZERO»: eram TRÊS defeitos, e o detector estava no caminho errado (entrega 25)

Commit `a93463605`. Report do Enio no smoke da entrega 24: *«não funcionou. Voltou ao zero.
Precisamos da opção de resetar. Coloque nas opções de Theme.»*

### §20.1 — ⭐ A medição desmentiu o sintoma antes de qualquer cura

O ficheiro dele **existia e estava certo**:

```
# PH2D layout
slot.audio_mixer=left_top
```

E uma sonda que reproduz o arranque exacto (registry → `HeroScreen::new` → `install(load())` →
pintar → abrir o Mixer) mostrou-o a desenhar em `x = 0` — a coluna da **esquerda**, onde ele o
deixou. ⇒ *a persistência da posição nunca esteve partida*, e curá-la teria sido trabalho sobre o
sítio errado.

⚠️ Duas coisas na mesma leitura contavam a história inteira: o `slot` **estava** lá e o
`dock_w_left` **não** — e ele tinha arrastado a borda.

### §20.2 — ⛔⛔ Defeito 1: guardar ONDE sem guardar SE é indistinguível de não guardar nada

O painel que ele moveu **nasce fechado** (`DEFAULT_VISIBLE = false`). Ao reabrir o app, a posição
estava restaurada e **não havia nada no ecrã a mostrá-la**.

⇒ **`open.<painel>=1`** entra na arrumação, com a mesma lei dos encaixes: guarda-se a **diferença**
do que o painel declara, logo uma entrada **inverte** o default (abre o que nasce fechado, **fecha o
que nasce aberto**) e um painel novo não precisa de migração.

### §20.3 — ⛔⛔⛔ Defeito 2: o detector estava no caminho de um GESTO

Ele vivia no `forward_to_hero`, ao lado dos outros dois inquilinos da persistência — e **os dois
gestos que arrumam o app não passam por lá**:

| gesto | por que escapava |
|---|---|
| arrastar a **borda** de uma coluna | `dock_seam_move` / `dock_seam_up` fazem `return` no `input_dispatch` |
| largar uma **aba** noutro encaixe | é resolvido **dentro** do `paint`, depois do hook |

⇒ a largura da coluna dele **nunca foi gravada**. A detecção mudou-se para o **quadro** (depois do
`paint_hero_screen`) e para o módulo que é dono do facto (`layout_persist::save_if_changed`).

> *Um detector no caminho de um gesto só vê os gestos que passam por ele; o quadro vê todos, porque
> é onde o estado assenta.*

⚠️ **Os outros dois inquilinos FICAM no hook, e está certo:** a escolha de paleta e a de carácter são
cliques que atravessam o hero. O gate leva o controlo que impede alguém de os arrastar consigo.

### §20.4 — ⭐ Defeito 3 (o pedido): o RESET, onde ele o pediu

*View → Theme… → **Reset Panel Layout***. Repõe **as três** coisas de uma vez — o encaixe de cada
painel, quais estão abertos, e a largura das colunas.

⛔ **Repor duas de três não é repor:** o artista clica, vê o ecrã mudar e conclui que funcionou, e o
terço que ficou morde-o mais tarde **sem ligação com o gesto que o deixou**.

⚠️ **Ele não apaga o ficheiro, e não precisa:** o gravado é uma **projecção** do que o app tem agora,
e o detector do quadro grava a projecção vazia sozinho. *Apagar seria um segundo caminho para o
mesmo facto, e o dia em que os dois discordassem seria silencioso.*

⚠️ E o `MENUBAR_VIEW_RESET_LAYOUT` **não entra no `MODULE_TRUTHS`**: ele é um **verbo**, não um
estado — não tem marca de «ligado», e clicá-lo duas vezes é o mesmo que uma.

### §20.5 — ⚠️ O gate da costura apanhou a MINHA mudança de sítio

Ao mover a detecção, `the_arrangement_is_written_when_it_changes` ficou **vermelho** — ele afirmava
o **endereço** (*está no `forwarding.rs`*). ⭐ Foi reescrito para afirmar o **mecanismo**:
*está no quadro* **e** *não está no hook*, com a tabela dos dois gestos que escapavam colada ao
doc-comment. *Um gate que afirma um endereço tem de ser reescrito quando o endereço muda; um que
afirma o mecanismo sobrevive à mudança e continua a defender a razão.*

### §20.6 — Verificação

`ph2d-editor-core` **1345** ✓ · `ph2d-host-desktop` **4779** ✓ · `ph2d-panel-registry-init` 0 falhas ·
`check` e `clippy --workspace --all-targets` limpos · fmt.

**Cinco mutações mortas** com controlo: o reset esquecer a visibilidade · o reset esquecer a largura ·
a linha do menu deixar de despachar · os painéis abertos saírem da projecção · o `install` deixar de
repor a visibilidade.

### §20.7 — ⏳ O que fica nomeado

- **Quem fica à frente quando dois painéis abrem no mesmo quadro** resolve-se por **ordem do
  registry**, que é arbitrária. No arranque isso decide qual aba o artista vê primeiro. Uma ordem
  autorada (a última à frente, gravada) é wave pequena e ainda não existe.
- **A ordem das abas** continua a ser a ordem z e não viaja no ficheiro.

---

## §21 — ⭐⭐⭐ OS LAYOUTS POR TAREFA: a D7 (entrega 26)

Commit `bfa2707f7`. O eixo que a **D3** separa dos outros dois:

> | eixo | quem decide | onde vive | o que muda |
> |---|---|---|---|
> | **Layout** | o **utilizador** | barra de cima (abas) | que **áreas** existem e que editor está em cada |

### §21.1 — Seis abas, à direita da barra de menus

**Draw · Vector · Flip · Model · Animate · Nodes.** Escolher uma arruma a tela: os painéis daquele
layout abrem, **todos os outros fecham**, e a ferramenta opcional é pegada.

⭐ **Elas ocupam o espaço que já estava vazio** — os cinco menus gastam ~250 px de 1366. É onde o
Blender põe as dele (*workspace tabs*, na topbar). ⛔ **Não é uma segunda faixa:** ela custaria mais
28 px permanentes ao alvo de 1024 pontos, que é precisamente o que a barra de menus evitou.

⚠️ **As abas recusam-se a pintar se não couberem** sem tocar nos títulos dos menus. *Uma aba por
cima de um menu é um clique que troca de tarefa quando o artista queria abrir o ficheiro.*

### §21.2 — ⭐ A costura com o MODO é UM CAMPO, e é a do Blender

O Workspace dele tem um `Mode:` — *«switch to this Mode when activating the workspace»*. Ortogonais,
**com um atalho declarado**; ⛔ não acoplados. Aqui é o `LayoutSpec::tool`: *Vector* arruma **e** pega
na ferramenta de vetor, porque um layout de vetor com o canvas noutro modo é uma arrumação que não
serve para nada.

⚠️ Um layout que **não** declara ferramenta **não mexe** na que está em mãos — trocar para *Animate*
a meio de um traço não pode largar o pincel.

### §21.3 — ⛔ A lista de abertos é ABSOLUTA, não um diff

Um layout que só *acrescentasse* painéis acumularia o que a tarefa anterior deixou: *Nodes* depois de
*Draw* daria o grafo **mais** as camadas do pintor. *Um layout é o estado da tela, não um passo sobre
ele.* Pela mesma razão a troca **limpa as excepções de encaixe** — elas pertencem à arrumação de quem
as fez.

⚠️ **A largura das colunas NÃO é reposta**, de propósito: ela é a **medida da mão** de quem usa o
ecrã, não da tarefa. Ela viaja com o layout no ficheiro, para quem a quiser diferente por tarefa.

### §21.4 — ⛔ Dois dos oito não existem, e o bloqueador é de outra pessoa

| layout | bloqueador |
|---|---|
| **Código** | não há editor de texto neste app |
| **Runtime** | `shells/game` / R1, **adiado pelo Enio** |

⇒ eles **não são abas mudas**. *Uma aba que não faz nada é o controlo morto que este repo mais
paga.* Entram no dia em que o bloqueador cair.

### §21.5 — O ficheiro ganha SECÇÕES

```
active=vector

[vector]
dock_w_left=280
```

⚠️ **Uma arrumação POR layout, e é o que a D7 obriga:** quem alarga a coluna no *Vector* não a quer
alargada no *Animate*. ⭐ E um layout que o artista **nunca mexeu não tem secção** — é isso que o
deixa receber uma mudança futura na tabela de fábrica; quem tem secção fica preso ao que gravou, e é
o que se quer, mas só para quem de facto mexeu.

### §21.6 — ⛔⛔ Duas mutações SOBREVIVERAM, e as duas eram buracos meus

1. **A troca deixar de limpar os encaixes** — nada movia um painel antes de trocar de tarefa.
2. **A composição apagar a arrumação dos OUTROS layouts** — nada exercitava **dois** layouts. ⇒ a
   composição saiu do hook e virou `compose(...)`, uma função com nome.

⚠️ **É a segunda vez nesta jornada** que uma decisão dentro de um hook se prova indefensável (a
primeira foi o `should_save`, §19.4). *Uma decisão dentro de um hook é uma afirmação que ninguém pode
contradizer.*

### §21.7 — ⚠️ E um gate teve de escolher o ORÁCULO certo

`every_panel_a_layout_names_is_a_crate_that_exists` pergunta à **pasta** e não ao registry: esta build
de teste corre com as features de omissão da `panel-registry-init` e a do **app** liga mais três
(`painter_layers`, `flip`, `flip_frames`). Perguntar ao registry acusaria **três ids correctos** de
não existir.

> *Uma ausência por feature e um erro de escrita leem-se iguais num registry; só a árvore os separa.*

⭐ O irmão `every_named_panel_that_this_build_registers_actually_opens` mede o **produto** naquilo que
esta build tem. Os dois são precisos.

### §21.8 — Verificação

`ph2d-editor-core` **1353** ✓ · `ph2d-host-desktop` **4784** ✓ · `ph2d-panel-registry-init` 0 falhas ·
`check` e `clippy --workspace --all-targets` limpos · fmt · **8 mutações mortas** com controlo.

⚠️ Tecto de LOC: o `hero.rs` bateu **701/700** e o corte foi por responsabilidade —
`hero/panel_host.rs` (*o que o hero **empresta***, contra *o que ele **é***). A superfície do
`PanelHostInternal` só cresce com a migração dos painéis (ADR-0029), e crescer ali empurrava o tecto
de quem não tem nada a ver com painéis.

### §21.9 — ⏳ O que fica nomeado

- **Os dois layouts bloqueados** (§21.4).
- **A tabela de fábrica é minha, não dele.** A D7 diz *para que serve* cada layout; que painéis cada
  um abre foi derivado disso mais o que existe hoje. É a primeira coisa a re-smokar.
- **Nenhum atalho de teclado** abre um layout. O Blender tem `Ctrl+PgUp/PgDn`; aqui não há.
- **Quem fica à frente quando dois painéis abrem no mesmo quadro** continua a ser a ordem do
  registry (§20.7) — e agora ela decide qual aba de painel o artista vê ao trocar de layout.

---

## §22 — ⛔⛔⛔ «O GRAFO DE NODES PERSISTE» + as faixas encaixadas (entrega 27)

> Enio, 2026-08-31, com três fotos (uma delas do Godot como referência):
> *«gostei da idéia, muito boa! Temos alguns bugs: se abro Nodes e depois Model o grafo de Nodes
> persiste. Procure outros problemas similares. Outra coisa: a timeline e o canvas dos Nodes devem
> ser bem encaixados entre os painéis laterais como na godot (sem espaços).»*

### §22.1 — A causa NÃO era o grafo: **quase todo painel deste app pertence à FERRAMENTA**

O `layout_switch::apply` fecha tudo o que a lista de abertos não nomeia. Ele corre na **pintura**
(fim do quadro). As pontes das ferramentas correm **antes** da pintura do quadro seguinte, e
reescrevem a visibilidade dos painéis delas a partir de `tools.active()`:

```text
motion_bridge:  panel_visibility.insert("motion_params",  motion_active)     // TODO o quadro
vector_bridge:  panel_visibility.insert("vector",         vector_active)
painter_bridge: panel_visibility.insert("painter_layers", painter_is_active)
flip_bridge:    panel_visibility.insert("flip",           flip_active)
```

⇒ **a lista «absoluta» não é a última palavra.** Enquanto o `LayoutSpec::tool` fosse
`Option<&'static str>` com `None = «não mexe na ferramenta em mãos»`, um layout que não largasse a
ferramenta trazia os painéis dela de volta **um quadro depois** de os fechar.

⚠️ **Valia para DOIS dos seis**, e a segunda mordida estava na foto do próprio Enio antes de ele a
notar: no *Animate* (`tool: None`, vindo do *Vector*) o dock direito mostrava as abas
**`Inspector | Vector`** — a ponte do vetor a reabrir o painel dela num layout que a não pediu.

| layout | `tool` era | consequência |
|---|---|---|
| `Modeling3d` | `None` | o grafo do Motion persistia por cima da peça — **o report** |
| `Animation` | `None` | o painel do Vector (ou o do pintor) sobrevivia à troca |

### §22.2 — ⭐⭐ A cura nº1: **o canvas tem um dono, e todo layout o nomeia**

`LayoutSpec::tool: Option<&str>` → `LayoutSpec::canvas: CanvasOwner`:

```rust
pub enum CanvasOwner { Tool(&'static str), Model3d }
```

- ⚠️ **`Tool("move")` é o que este app tem em vez de «nenhuma».** O registry nunca fica sem
  ferramenta activa (`activate_default` no arranque; o `set_active` exige um id), e **todo** gesto
  de largar deste app — o `CancelActiveTool`, o pill do vetor, o do motion — já era escrito como
  *«volta à de omissão»*. ⇒ o *Animate* pede o `move`, e não fica calado.
  ⛔ *A linha `Some(x) → None` da tabela do `field3d_mode::took_the_canvas` descreve um estado que
  nenhum gesto deste app produz.*
- ⚠️ **`Model3d` não pede ferramenta nenhuma**, e isso é load-bearing — ver §22.3.

### §22.3 — ⭐⭐⭐ A cura nº2: a lei do `field3d_mode` dizia-se **simétrica** e só uma metade soltava uma FERRAMENTA

⛔⛔ **O defeito é do módulo 3D e é ANTERIOR às abas.** Abrir o MODEL pelo menu *Window* com o
Motion em mãos tinha exactamente o mesmo resultado: nada largava a ferramenta, e a `motion_bridge`
reabria o grafo a cada quadro. A aba nova só o tornou fácil de alcançar.

A tabela do módulo ganha a **terceira linha**:

| quem entra | quem cede |
|---|---|
| uma ferramenta é pegada, ou o barro aparece | o painel MODEL **fecha** |
| o MODEL é aberto | o **barro** sai da tela |
| o MODEL é aberto | a **ferramenta em mãos** volta à de omissão *(novo)* |

⚠️⚠️ **E ela não pode ser escrita sem RE-BASELINE, senão a lei morde-se a si própria.** Largar a
ferramenta *é* mudar o dono do canvas: o `note_owner` do quadro seguinte compararia a neutra com a
modal guardada, leria *«outro tomou o canvas»* e **fecharia o painel que a largou**. O produto
entraria em ciclo e a suíte ficaria verde.

⇒ a decisão **e** o registo vivem numa função só, `field3d_mode::model_takes_the_canvas(now,
neutral)` — que devolve `true` e escreve o dono novo no mesmo acto. Com o registo do lado do
chamador, a mutação que o apagasse sobrevivia. **Provado**: apagar o `LAST.with` mata dois gates.

⚠️ **Terceiro achado, de graça:** o `model_just_opened` vivia **dentro** de
`#[cfg(feature = "sculpt3d")]` — uma build sem aquela feature nunca avançava a borda. Hoje é lido
uma vez, fora do `cfg` (ele **consome** a transição: uma segunda chamada no mesmo quadro devolve
`false`).

### §22.4 — ⭐⭐ A cura nº3: **um layout só comanda o que nenhuma ponte possui**

A lista de abertos encolheu, e a fronteira tem gate derivado da árvore
(`shells/desktop/tests/a_layout_never_commands_a_panel_a_bridge_owns.rs`). A classificação é
**mecânica**:

| o que a ponte escreve | é… | porquê |
|---|---|---|
| `insert(<id>, <identificador>)` | **POSSE** | um facto sobre a ferramenta, recalculado a cada quadro |
| `insert(<id>, true/false)` | **empurrão** | uma decisão tomada UMA vez, numa borda |
| `insert(<id>, !x)` | **empurrão** | a tomada de conta do inspector, também de borda |

*Um empurrão pode ser desfeito por quem quer que seja depois; uma posse não.* É por isso que o
`timeline` (que a `motion_bridge` **abre** por cortesia e nunca fecha) continua a ser do layout, e
o `motion_graph` não. Censo medido: **12 posses**, e o gate exige três nomes conhecidos lá dentro.

⚠️ **O `inspector` sai da tabela por um motivo próprio: ele tem DOIS escritores com uma ordem
fixa.** Seis pontes escrevem-no na borda de uma tomada, **depois** de o layout ter pintado. *Um
campo com dois escritores e uma ordem fixa tem um dono só, e não é quem escreve primeiro.*

| layout | `open` | `canvas` |
|---|---|---|
| Draw | `hierarchy` | `Tool("painter")` |
| Vector | `hierarchy` | `Tool("vector")` |
| Flip | `hierarchy` | `Tool("flip")` |
| Model | `hierarchy`, `model3d` | `Model3d` |
| Animate | `hierarchy`, `timeline` | `Tool("move")` |
| Nodes | `hierarchy`, `timeline` | `Tool("motion")` |

### §22.5 — ⛔⛔ E um gate **afirmava o defeito**

`a_layout_that_declares_a_tool_asks_for_it_and_one_that_does_not_leaves_the_hand_alone` ficou
**verde durante o report inteiro**: ele media a *decisão* (*«o Animate não declara ferramenta, logo
não pede nenhuma»*) em vez da *consequência* (*«e por isso a tela dele fica com os painéis da
tarefa anterior»*).

> *Um gate escrito a partir da intenção do código pina o que o código faz, não o que ele deve.*

Substituído por `every_layout_hands_the_canvas_over_and_none_inherits_it`, que varre os seis e
confronta cada um com o `CanvasOwner` dele.

### §22.6 — ⭐⭐ As faixas encaixam entre as colunas (o segundo pedido)

**São DUAS regiões, e o defeito de cada uma era o oposto do da outra:**

| região | o defeito | o que se via |
|---|---|---|
| `timeline` / `flip_strip` | 20 px a **MENOS** de cada lado | um buraco entre o painel e a faixa |
| `motion_graph` + `center_viewport` | a janela **INTEIRA** em vez da área | a banda do grafo por cima do fundo das duas colunas |

- **A faixa do fundo** era `left_col_right + EDGE_PAD`, e acabava `EDGE_PAD` antes da outra coluna.
  ⚠️ **É um resíduo com data:** em 2026-08-30 o `EDGE_PAD` saiu do `area_x0` (as colunas ficaram
  *flush*) e ficou aqui — e o doc do `reserve_bottom_strip`, escrito **nesse mesmo dia**, já
  afirmava que o timeline nascia *«literalmente no `area_x0`»*, o que era falso por 20 px.
  *Duas aritméticas para a mesma borda divergem no dia em que só uma é corrigida.*
- **O split do centro** partia `viewport.x .. viewport.x + viewport.w`. ⇒ a banda do grafo nascia
  **por baixo** das duas colunas e, como o painel dela é pintado depois, comia o terço de baixo da
  Hierarquia e do dock da direita (~`2 × 300 × 430 px²` no alvo de referência). Hoje ela é a coluna
  da **área** — D5, a mesma lei da `draw_area`: *regiões são IRMÃS numa fila.*

⚠️ **Só o x/w mudou; o y/h ficou.** A fracção `t` continua a ser a da banda de chrome na vertical,
que é o que o `CenterSplit::scene_viewport` (o renderizador da cena) lê — mexer nela poria o
`set_viewport` e o painel a discordar, que é o *drift* que aquele doc conta.

⚠️⚠️ **E as DUAS metades do split tiveram de encolher juntas.** O painel do grafo deteja a
orientação por `rect.x > center.x` e mede o arrasto do divisor contra `center + rect`: narrar só a
banda do grafo faria um split **horizontal** ler-se como **vertical**. *Duas metades de uma
partição não podem sair de janelas diferentes.*

### §22.7 — O que ficou aberto, nomeado

- ⏳ **O modelo de posse é o que a D4 vai ter de mudar.** Hoje quase todo painel pertence a uma
  ferramenta, e a D4 diz que *o artista escolhe qual painel vai onde* — enquanto a ponte reescrever
  a visibilidade a cada quadro, o artista não pode fechar o painel do pintor com o pintor em mãos.
  ⛔ Reescrever as 12 pontes é wave própria e mexe em módulos de outras linhas.
- ⏳ **A tomada de conta do inspector é legado dos docks sem abas.** Ela existe porque, antes das
  abas, dois painéis no dock direito sobrepunham-se; hoje eles tabulam. Retirá-la é decisão de
  produto (e devolveria o `inspector` à tabela dos layouts).
- ⏳ Sob split **vertical**, a cena é renderizada em `w·t` da **janela** e a banda do grafo mede
  `t` da **área** — a cena visível fica ligeiramente mais estreita do que a fracção diz. É a mesma
  família do *«a cena é full-bleed por baixo do chrome»* que o `draw_area` já nomeia, e fecha com a
  docagem da cena (A2), não aqui.
- ⏳ Continuam de fora as duas abas bloqueadas (**Código**: falta um editor de texto · **Runtime**:
  o `shells/game`/R1, adiado pelo Enio).

---

## §23 — ⛔⛔ O INSPECTOR FECHADO e a faixa que SE SOLTAVA da banda (entrega 28)

> Enio, 2026-08-31, com duas fotos: *«em animate o inspector está sendo escondido. Por padrão deve
> ficar visível. Em nodes, arrastar a timeline na vertical deve ajustar o canvas dos nós e não
> deixar espaços vazios nem sobrepor os nodes.»*

### §23.1 — ⛔⛔ «Não comandar» e «comandar fechado» são a MESMA linha de código

Na entrega 27 tirei o `inspector` de **todas** as listas de abertos, com o argumento — correcto —
de que ele tem dois escritores e que o layout perde sempre a corrida. ⇒ **e fechei-o em toda
parte**, porque o `layout_switch::apply` é absoluto sobre o registry inteiro:

```rust
for p in reg.panels() {
    hero.panel_visibility.insert(p.manifest.id, spec.open.contains(&p.manifest.id));
}
```

> *«O layout não o comanda» e «o layout comanda-o fechado» leem-se igual num campo ausente, e só a
> segunda é o que o código faz.*

⭐ **A lei certa é derivável dos dois lados**, e substitui o gate que eu tinha escrito
(`no_layout_claims_the_inspector_…`, que defendia o defeito):

| a ferramenta do layout escreve `insert("inspector", !active)`? | o layout… |
|---|---|
| **sim** (motion · vector · flip · upscale · equalize_sizes · color_equalization) | **não** o nomeia — a ponte substitui-o na coluna e desmentiria a tabela |
| **não** (painter · move · o modelador) | **nomeia-o** — senão ele fecha e não há quem o reabra |

⇒ *Draw*, *Model* e *Animate* voltam a mostrá-lo. O censo das seis pontes sai do **nome do
ficheiro** (`<tool>_bridge.rs`), que é a convenção daquele directório.

### §23.2 — ⛔⛔⛔ A faixa não era arrastada: o PAINEL soltava-se dela

A costura do timeline chamava `geom::resized(...)` e escrevia **`TimelinePanelState::rect`** — um
rect LIVRE. A partir do primeiro toque na borda o painel deixava de ler a faixa que o layout lhe
dava:

| sentido do arrasto | o que se via |
|---|---|
| para baixo | **espaço vazio** por cima do painel (a faixa ficou onde estava; o painel foi-se) |
| para cima | o painel **por cima** do grafo |

> *Uma borda de painel docado que devolve um rect livre é um painel que deixa de estar docado
> quando se lhe toca.*

⭐⭐ **A cura é a mesma forma das colunas: a borda escreve uma MEDIDA.**
`WidgetStore::dock_bottom_h` é a irmã VERTICAL de `dock_width` — mesma porta, mesmo clamp, mesma
distinção `…_choice()` para o que se grava —, e `ChromeBands::bottom_dock_h` leva-a ao layout. Quem
partilha a banda (o grafo, por `dock_timeline_into_motion`) segue **por construção**, não por uma
segunda conta que possa discordar.

⛔⛔ **E `MOTION_TIMELINE_H` teve de MORRER.** Ela dizia que o timeline é *«mais baixo dentro do
Motion»* e era uma **segunda altura** ao lado da autorada: com ela, arrastar a costura dentro do
Nodes não mexia nada — quem mandava ali era uma constante. Hoje `dock_timeline_into_motion` lê
`self.timeline.h`. *A faixa tem UMA altura; docá-la dentro do split não pode inventar outra.*

⚠️ **Três coisas foram RETIRADAS com ela**, e a retirada é a decisão:

| o que saiu | porquê |
|---|---|
| 7 dos 8 agarres (3 bordas + 4 cantos) | numa faixa docada os lados são as costuras das **colunas** e o fundo é a borda da **janela** — eram gestos inexprimíveis |
| `TimelinePanelState::rect` | era a segunda ideia de *«onde o timeline está»*, e ganhava à do layout |
| `geom::resized` + os 4 gates dela | *uma função que só sobrevive nos próprios testes é a última prova de que a capacidade que ela servia foi retirada* |

⚠️ O gate `corners_are_registered_after_the_edges_they_overlap` **ordenava agarres que não deviam
existir** — ele foi substituído por `the_only_grip_is_the_top_seam`.

### §23.3 — O que fica

- A altura autorada **viaja no ficheiro** (`dock_h_bottom`, por layout) e entra no hash do
  detector, como as duas larguras.
- ⚠️ A banda nova aparece **um quadro depois** do arrasto — o `ctx.slot` daquele quadro já estava
  calculado. É a mesma latência das larguras de coluna, e invisível a 60 fps.
- ⏳ O tecto de `MOTION_TIMELINE_MAX_FRAC` (45 % da banda) continua a ser um número **escolhido**,
  não medido: ele existe para o editor de nós não virar uma fita. Se o Enio quiser a faixa maior do
  que isso dentro do Nodes, o que se mexe é o divisor da cena, não este tecto.

### §23.4 — ⛔⛔⛔ E o portão desta linha estava a correr sobre a build POBRE

O fecho desta entrega correu `cargo test -p ph2d-panel-registry-init -p ph2d-host-desktop …` **numa
invocação só** — e dois gates que passavam há uma entrega inteira ficaram vermelhos:

| corrida | `flip_frames` registado? | veredito |
|---|---|---|
| `-p ph2d-panel-registry-init` (as entregas 21–27) | **não** — a feature não está nas de omissão dele | ✅ verde |
| `-p ph2d-panel-registry-init -p ph2d-host-desktop` | **sim** — o shell liga-a, e o cargo **unifica** as features | ❌ dois vermelhos |

> *Uma suíte verde crate-a-crate não é a suíte do produto: é a suíte da build mais pobre que aquele
> crate consegue ter.*

⚠️ **E o ✗ lê-se como flake** (passa sozinho, reprova em conjunto). Não é: o que muda é a
**população**, não a carga. O sinal que os separa — um flake de carga muda de teste entre corridas;
este reprova sempre o mesmo caso com o mesmo número.

**O que estava escondido lá dentro (defeito meu, da entrega 23):** a tira do Flip pintava
`(0, 732, 1366, 292)` numa banda de `240` — **147 528 px² de painel por cima da área de desenho**,
que é literalmente a foto 2 da D1.

⭐ **Duas causas, uma por cada metade:**

1. **O painel inflava-se para fora da banda.** Ele somava `TIMELINE_DOCK_H` (para *«empilhar acima
   do timeline»*) e `grow` (a barra que quebra em linhas). ⚠️ **As duas premissas dissolveram por
   baixo dele e a nota nunca foi reconferida:** o timeline e a tira declaram o **mesmo encaixe**, e
   desde as abas de encaixe (entrega 21) dois painéis no mesmo sítio são **abas** — nunca duas
   faixas ao mesmo tempo; e a altura da banda passou a ser **autorada**, então a constante somada
   descrevia um número que o artista move.
2. **A reserva da área de desenho lia um rect que ninguém pintava.** `layout.flip_strip` (132 px)
   era a geometria que a tira **declarava** na entrega 22 e não a que ela **pinta** desde a 23
   (`ctx.slot`). ⇒ a área ficava reservada até 132 px do fundo e a tira ocupava 240.
   *Reservar a geometria que um painel declarava, em vez da que ele pinta, é reservar o sítio
   errado com toda a confiança.*

⇒ `HeroLayout::flip_strip` e `FLIP_STRIP_H` **morreram**, e a reserva passou a ser **uma só, sobre
a banda `Bottom`**, quando qualquer um dos dois painéis está visível.

⚠️ **Passa a ser regra do fecho desta linha:** o portão corre os crates de gate **na mesma
invocação do shell**, nunca `-p <crate>` um a um.

### §23.5 — ⛔⛔ E o TECTO da faixa matava metade do gesto que ele pediu

A docagem no Motion cortava a faixa a `0,45 × a banda` — **`202` px** no alvo de referência —, e a
faixa nasce com **`240`**. ⇒ a costura nascia **já no tecto**: arrastar para BAIXO funcionava,
arrastar para CIMA era inerte, e nada na tela dizia porquê.

> *Um limite que corta o valor de fábrica não é um limite: é metade do controlo desligada de
> origem.*

⭐ O tecto passou a defender o **hospedeiro** e não a ser uma fracção: o grafo nunca fica com menos
do que o **piso de uma faixa docada** (`120` px, o mesmo `DOCK_H_MIN` abaixo do qual um dock deixa
de ser usável). No alvo de referência isso dá `330` de tecto e `120` de piso — **os dois lados
vivos**, com a de fábrica a `240` no meio. Gate:
`the_seam_has_travel_in_both_directions_from_the_factory_height`, provado por mutação (repor a
fracção mata-o).

### §23.6 — O que fica aberto desta entrega

- ⏳ **A tira do Flip e o timeline são ABAS no mesmo encaixe, e ninguém decidiu isso** — foi o que
  a entrega 21 tornou verdade sem que a tira o soubesse (ela ainda tinha código para *empilhar*).
  Hoje funciona (uma aba de cada vez), mas é uma decisão de produto por tomar: no Flip, ver os
  quadros **e** a linha do tempo ao mesmo tempo pode ser o que se quer, e isso pede um segundo
  encaixe inferior — que a D1 recusou por medição (12 encaixes = 89,6 % do alvo).
- ⏳ **A tira do Flip já não CRESCE com a barra que quebra** — ela fica na banda, e uma barra em
  duas linhas come das células. Quem quiser mais arrasta a costura. ⚠️ Não foi medido quantas
  linhas a barra quebra numa janela de 1366; se for mais de duas, o piso da banda pode ficar curto.
- ⏳ O `MOTION_GRAPH_MIN_H` (`120`) é o piso de uma faixa docada **emprestado** ao grafo. É
  defensável (abaixo dele um painel deixa de ser usável) mas não foi medido **no grafo** — o número
  certo é *o cabeçalho + um cartão de nó*, e isso mora noutra crate.
- ⏳ Continuam de fora as duas abas bloqueadas (**Código** · **Runtime**) e os itens do §22.7.

---

## §24 — ⛔⛔⛔ O DIVISOR do canvas de nós: offset de 96 px e um tremor de 6,7 px (entrega 29)

> Enio, 2026-08-31: *«segurar e arrastar o topo do canvas de nós tem um bug, um offset e um
> tremor.»*

### §24.1 — Duas contas para a mesma banda

O painel do grafo escrevia a fracção do divisor **reconstruindo a banda**:

```rust
let t = (g.y - center.y) / (rect.y + rect.h - center.y);   // center_viewport + motion_graph
```

Isso É a banda — **até a timeline docar dentro do split** (W4.T4) e passar a comer o fundo do
`motion_graph`. A partir daí:

| quem | denominador de `t` |
|---|---|
| o painel, ao arrastar | `chrome_h − altura_da_timeline` |
| o layout, ao aplicar (`top_h = band.h · t`) | `chrome_h` |

⇒ **as duas metades do report, e as duas medidas** (alvo de referência 1366 × 1024, com a timeline
docada):

| sintoma | medida |
|---|---|
| **offset** | o dedo larga em `352` e o divisor vai para `448` — **96 px** |
| **tremor** | com o dedo **parado**, o divisor oscila `672 → 665,3 → 670,9 → 666,2 …` — **6,7 px** |

⭐ O tremor não é ruído: a altura da timeline é ela própria **clampada pela altura do grafo**, logo
o denominador depende do resultado. *Uma fracção medida contra uma grandeza que depende dela não
converge — ela vibra.*

### §24.2 — A cura: a banda tem um DONO, e ele publica-a

`HeroLayout::split_band` — a região que o divisor parte, escrita onde ela é decidida. O painel
mede contra ela (`split_fraction(band, rect, pointer)`, pura e exportada) e mais ninguém a
reconstrói.

⚠️ **O gate tem de ser de IDA-E-VOLTA e atravessar as duas crates** — medir só a fórmula
confirmaria a fórmula, e o que estava errado era ela **não ser a inversa de quem a aplica**:
`crates/ph2d-panel-motion-graph/tests/the_divider_lands_where_the_pointer_is.rs`, com a docagem da
timeline **obrigatória** (sem ela o gate mede o caso que nunca falhou) e um irmão que segura o dedo
parado por dez quadros.

⚠️ **E a 1.ª redacção do gate acusou a CERCA**: com alvos em px fixos (`300`), `t = 0,246` cai fora
do `T_MIN = 0,25` e o `clamp_t` movia o divisor 4 px — um offset real, mas do clamp, não da conta.
Os alvos saem agora da própria banda, dentro da faixa legal. *Um gate de ida-e-volta que amostra
fora do domínio mede a cerca e chama-lhe defeito.*

⚠️ **Sexto tecto de LOC desta linha** (`interact.rs`, 622/600): a lei do divisor mudou-se para
`split.rs`. O corte é por responsabilidade — aquele ficheiro **despacha** gestos, e isto é **uma
lei**, a única deste painel que tem de ser a inversa exacta de código noutra crate.

---

## §26 — ⛔⛔⛔ O CABEÇALHO É REVERTIDO, e a restrição de ecrã vira GATE (entrega 31)

> Enio, 2026-08-31: *«Lembre-se que esse app tem tablets e iPad como alvo. Não podemos ir perdendo
> espaço. Desfaça isso. Veja nos planos se há mais motivos de perder espaço e ajuste.»*

### §26.1 — A reversão, e o que ela custava

A entrega 30 nasceu e morreu no mesmo dia. `git revert` inteiro: a faixa, os gates dela, e as duas
linhas de menu **voltam** ao *View* e ao *Look* — porque sem a faixa elas ficariam sem casa, e um
comando sem casa é pior do que um comando na casa errada.

| alvo | com o cabeçalho | sem ele |
|---|---:|---:|
| iPad 12.9 | 49,3 % | **50,8 %** |
| iPad mini | ~36,3 % | **37,6 %** |

⇒ **28 px permanentes por dois interruptores.** *Uma faixa permanente tem de devolver mais do que
consome.*

⚠️ **A D2 continua certa sobre o ÂMBITO** — um comando do editor não pertence a um menu do app. O
que foi recusado é a **faixa própria**. Se a metade 2 voltar, ela cabe **onde já se paga altura**:
a fila de ferramentas, ou um popover à direita dela. Registado na própria D2 e no `spec/02 §8`.

### §26.2 — ⭐⭐⭐ E a restrição deixou de ser uma nota

A `D2` tinha escrito, desde 30/08, *«⚠️ o preço que o Enio aceitou: a barra global come uma faixa
de altura permanente»* — e **uma segunda faixa foi construída na semana seguinte**. Uma nota não
trava nada.

⇒ `crates/ph2d-editor-core/tests/the_chrome_never_eats_more_of_a_tablet_than_this.rs`: a área de
desenho, em % da janela, nos **três** tablets, pelas mesmas funções do `hero::frame_layout` (com as
duas passagens — a altura da fila depende da largura da área). Piso medido, e **tecto** de
obsolescência: uma célula que suba `2` pontos reprova, porque nesse dia a barra deixou de defender
o que mede.

⛔ **Provado por mutação:** repor uma faixa de 28 px derruba-o (`49,3 %` contra o piso `50,8 %`).

### §26.3 — ⛔⛔ A auditoria achou o resto, e o pior estava vivo

**Medido:** [`medicoes/06`](../medicoes/06_o_orcamento_de_ecra_em_tablet.md).

| # | onde se perde | número |
|---|---|---|
| 1 | ⛔ **a fila de ferramentas DOBRA** (`54 → 108 px`) no iPad 11 e no mini **com o pincel em mãos** | `−3,2` e `−3,3` pontos, enquanto se pinta |
| 2 | as duas colunas são `612 px` **absolutos** — não escalam com o ecrã | `44,8 %` da largura no 12,9", **`54,0 %`** no mini |
| 3 | ⭐ **recolher** as duas devolve `89–92 %` — mais do que todo o chrome vale | e custa **dois itens de menu**, um por coluna |

⭐ **O (1) tem cura DECIDIDA, e não tinha antes.** O §7 deste handoff listava duas saídas — *«quebrar
em duas linhas (a faixa cresce) ou um menu de transbordo»* — sem critério. O alvo tablet escolhe:
**a faixa não cresce; o excesso vai para um controlo de transbordo.** ⛔ Encolher o chip continua
fora (mente sobre o preset que o artista escolheu).

⚠️ **Só o (2) não tem cura barata**, e a razão está no `spec/01 §6`: ele declara que a largura do
encaixe **não escala com o alvo de toque** — e não dizia nada sobre ecrãs **menores**, onde o mesmo
número absoluto custa proporcionalmente mais. Baixar o default é decisão de produto (muda o que o
Enio vê no 12,9").

### §26.4 — ⏳ O que fica, por ordem de retorno

1. ⭐⭐ **`G` — esvaziar os painéis** sobe a primeira prioridade: é o **único** degrau do plano que
   DEVOLVE ecrã (66 de 74 entradas do painel medido têm outro dono).
2. ⏳ **Um gesto de RECOLHER as duas colunas** — a melhor razão custo/benefício que a medição achou.
   Decisão de produto: que gesto, que tecla, se elas voltam sozinhas.
3. ⏳ **O transbordo da fila de ferramentas** — decidido, por fazer.
4. ⛔ O cabeçalho por área só volta **sem faixa própria**.

---

## §27 — ⭐⭐⭐ A FILA NÃO CRESCE: o `⋯` (entrega 32)

O item nº 1 da auditoria de espaço (§26.3), e o primeiro passo que o Enio aprovou depois dela.

### §27.1 — O que estava vivo

A fila resolvia o transbordo **crescendo**: `54 → 108 px` no iPad 11 e no mini, no instante em que o
Painter entrava em mãos (**10** entradas em repouso, **18** com ele). `−3,2` e `−3,3` pontos de área
de desenho, permanentes, *justamente quando o ecrã faz falta*.

### §27.2 — ⭐ Uma PORTA decide o que cabe, e os três leem-na

`tool_bar::bar_split(store, painter, image_tools, area_w) -> (fila, transbordo)` — a fila devolvida
**já leva o `⋯` no fim**. O pintor, o registo de hit e o corpo do menu leem **a mesma** função.

⚠️ **O `⋯` reserva o lugar dele ANTES de a conta correr.** Senão a última entrada cabia por um triz,
o `⋯` nascia por cima dela, e o alvo do dedo ficava ambíguo.

⚠️ **E os divisores nas costuras caem** — um no fim da fila que fica não separa nada, e um no
princípio do transbordo também não.

⚠️ **A aritmética é a MESMA** (`widget::entry_advance`, que passou a ser pública por isto): uma
terceira cópia dela poria o `⋯` a discordar de onde os chips de facto caem.

### §27.3 — ⚠️ O transbordo é PUBLICADO, não recalculado

O corpo do menu vive no `context_menu_overlay`, que só tem `&WidgetStore` — sem a largura da faixa
nem o modo do Painter. ⇒ quem **calcula** publica (`tool_bar::publish_overflow`, uma vez por
quadro), e quem desenha lê.

⛔ Uma segunda conta do lado do menu poria um chip **nos dois sítios, ou em nenhum**. ⚠️ E ele é
escrito em **todo** quadro, vazio incluído: *um mapa que o tique apaga não envelhece* — um que só se
escrevesse ao transbordar deixaria chips fantasma atrás do `⋯` depois de a janela crescer.

### §27.4 — ⛔ E NÃO há handler novo

Os ids dentro do menu são **os mesmos** da fila, logo quem despacha continua a ser o `chrome::rail_*`.
O `chrome::tool_bar_overflow` (z=45, entre os toggles de vista e os do trilho) faz só duas coisas:
abre/fecha o `⋯`, e **fecha o menu deixando o clique PASSAR** (`return false`) quando o artista
escolhe um chip. *Um verbo copiado para ali seria a segunda porta que o `CLAUDE.md` §5.0 cataloga.*

### §27.5 — ⛔⛔ Duas coisas que só os gates apanharam

1. **O `⋯` nasceu MORTO sob o dedo.** Pintado e registado no `HitIndex`, mas sem `InteractiveState`
   — o Down não arma o `active` e o Up nunca emite `Click`. O gate de **gesto real** apanhou-o na
   primeira corrida; um `apply_event(Click(id))` sintético teria passado.
2. ⭐⭐ **O gate do orçamento continuou a medir `40,8 %` depois da cura** — porque ele **reproduzia**
   o `tool_bar_lines` em vez de perguntar o que o produto faz. *Um gate que reproduz a fórmula pina
   a fórmula, não o produto.* Corrigido para uma linha, como o `frame_layout`.

### §27.6 — ⭐⭐ E foi o TECTO da catraca que obrigou a registar o ganho

Com o gate a medir o produto, o iPad 11 saltou `40,8 → 44,0` e o mini `37,6 → 40,9` — e a metade de
**obsolescência** reprovou, exigindo que os pisos descessem para os números novos em vez de ficarem
como folga silenciosa. *É exactamente para isto que ela existe.*

| alvo | a pintar: cabem | atrás do `⋯` | área antes | área agora |
|---|---:|---:|---:|---:|
| iPad 12.9 | 18 | 0 | 50,8 % | 50,8 % |
| iPad 11 | 13 | 5 | 40,8 % | **44,0 %** |
| iPad mini | 12 | 7 | 37,6 % | **40,9 %** |

### §27.7 — ⏳ O que fica

- ⏳ **O `⋯` é hoje só o transbordo da fila.** Ele é também a casa que a `D2` metade 2 precisa (os
  comandos do editor, sem faixa nova) — mas isso é wave própria: os ids daqueles comandos são de
  painéis, e o despacho deles não passa pelo `chrome::rail_*`.
- ⏳ O menu abre **uma coluna** de chips. Numa lista de 7 isso é `~7 × 60 px` de altura; acima de
  ~10 pede duas colunas, e o número não foi medido.

### §27.8 — ⭐⭐ E a cura apagou uma dependência circular

A altura da faixa dependia da largura da área ⇒ o `frame_layout` resolvia o layout **duas vezes**
(uma com a faixa a zero, para ler a largura; a definitiva a seguir). Com a altura constante, a
**segunda passagem morreu** — e com ela a `tool_bar_lines`, que só sobrevivia nos próprios testes.

*A cura de um defeito de espaço apagou uma circularidade que ninguém tinha ido lá buscar.*

---

## §28 — ⭐⭐⭐ O PAINEL DEIXA DE SER O DEPÓSITO: o pulldown da ÁREA (entrega 33)

> Enio, 2026-08-31, depois de smokar o `⋯`: *«Smoke OK. Qual a próxima?»*

O degrau **`G`** do plano (`spec/02 §3`), que a restrição de ecrã pôs em primeira prioridade: *«o
painel deixa de ser o depósito por omissão»*. A medição da **D2** é dura — o painel `3D Model` tem
**74 entradas** e só **8** são propriedades do objecto; as outras **66 têm outro dono** — e o
sintoma não é uma opinião: aquele painel precisou de **barra de rolagem** (report do Enio,
2026-08-27), porque não cabia.

### §28.1 — A metade 2 da D2 volta, e sem faixa nova

O *cabeçalho por área* foi construído na entrega 30 e **revertido na 31** (`28 px` permanentes,
`−1,5` ponto de área de desenho). O âmbito ficou de pé; o que faltava era um sítio.

⇒ **a FILA é o cabeçalho da área.** Ela já é uma região da área (não da janela), já subtrai a altura
que subtrai, e desde a entrega 32 já sabe transbordar. O módulo que tem o canvas publica os comandos
dele e a fila ganha **um chip** no fim, depois de um divisor.

| peça | onde |
|---|---|
| o campo `area_entries` + `area_face` | `interaction::state` (`set_area_commands`, lido em todo quadro) |
| o chip na fila | `hero::tool_bar::bar_rail` (`ids::AREA_COMMANDS`) |
| o menu | `ContextMenuKind::AreaCommands` — rows **dinâmicas**, montadas no `context_menu_overlay` |
| o interruptor | `chrome::tool_bar_overflow` (tabela `BAR_MENUS`, agora com **dois** chips) |
| quem publica | `ph2d_panel_model3d::publish_area_bar`, chamado pelo `render_loop` em todo quadro |

⚠️ **Rows dinâmicas são a ÚNICA excepção do `menu_rows`**, e é de natureza: uma tabela estática ali
teria de conhecer os ids de todo módulo do app — o acoplamento que a D2 existe para não ter.

### §28.2 — ⛔⛔ UM chip e não nove, e o número é MEDIDO

A tentação era pôr as nove entradas cruas na fila. **Mutação 6**: com elas a fila precisa de
**2 linhas até no iPad 12,9\"** — o maior dos três alvos — e ainda transborda `2` chips para o `⋯`.

> *Poupar altura gastando largura não poupa nada.*

⇒ um **pulldown**, cuja face é a **leitura** da vista actual (`Front`, `User`, …) — que é uma coisa
que o painel nunca chegou a dar: as seis fileiras dele acendiam o chip certo, mas só com o painel
aberto.

### §28.3 — Os ids são os MESMOS, e é isso que faz o clique chegar

`HeroScreen::apply_event` entrega todo evento a **todo** painel do registry e o braço decide por
**id**, nunca por posição. ⇒ um chip pintado na fila despacha para o mesmo `ModelIntent::SetView` de
sempre, **sem uma segunda porta**. O `event.rs` do painel não mudou uma linha; o `populate` continua
a registar as duas famílias (é o registo que as mantém vivas sob o dedo), e só o `paint` deixou de
as desenhar.

### §28.4 — ⛔⛔⛔ DUAS redacções erradas sobre quem fecha o menu, e a mutação é que o disse

A 1.ª versão desta wave escreveu, com confiança, que *«o painel consome o clique antes do chrome,
logo o menu nunca fecharia»* — e pôs uma cura no `pre_dispatch`. **Mutação 2: apagar essa cura
deixou todos os gates verdes.**

A 2.ª versão concluiu então que o interruptor do `apply` é que fechava. **Mutação 3: apagar o
interruptor também deixou tudo verde.**

⭐ **O que de facto fecha, sob o dedo, é uma regra genérica do store, um nível abaixo:**
`dispatch::pointer_down` fecha todo menu aberto num **Down** primário que não *pertença* a ele
(`click_belongs_to_the_open_menu`). Medido: depois do Down no chip o menu **já está fechado**, e o
Up seguinte não levanta `Click` nenhum. E **não há fonte de `Click` sem um Down antes** — o
`focus.rs` é o caminho do Up do ponteiro, e nenhuma tecla emite `Click`.

⇒ a cura do `pre_dispatch` foi **retirada** (era a terceira cópia de uma lei que já existe), e a do
`apply` **fica** — mas por outro motivo, e agora com gate:

⚠️ **a paleta de comandos global (`global_palette`) chama `apply_event(Click(id))` sem ponteiro
nenhum**, e ela projecta a lista que a fila pinta. Por esse caminho o fecho genérico nunca corre, e
sem o interruptor escolher o chip **re-abriria** um menu já aberto. Gate:
`the_palette_path_toggles_the_menu_because_it_has_no_pointer_down` (mutação 4 mata-o).

> ⭐⭐ *Uma metade que «obviamente» faz alguma coisa pode ser inerte porque a lei já vive num nível
> abaixo — e a única forma de saber é apagá-la.* Nesta wave isso aconteceu **três vezes seguidas**,
> e a terceira revelou o consumidor real (a paleta), que nenhuma das duas primeiras tinha visto.

### §28.5 — Gates (4 novos) e as 6 mutações

`crates/ph2d-panel-registry-init/tests/the_area_hands_its_view_commands_to_the_bar.rs`

| gate | o que prende |
|---|---|
| `the_area_pulldown_opens_serves_a_view_and_closes` | Down/Up REAIS: o chip abre · a vista pede `SetView` · o menu fecha; e **antes de abrir os ids não estão em lado nenhum** (prova que saíram do painel em vez de ganharem um 2.º sítio) |
| `closing_the_module_takes_the_pulldown_off_the_bar` | publicado em todo quadro, vazio incluído |
| `the_area_costs_one_chip_and_the_bar_is_still_one_line` | **1** chip nos três tablets, e a fila numa linha |
| `the_palette_path_toggles_the_menu_because_it_has_no_pointer_down` | o caminho sintético, que é o único em que o interruptor é load-bearing |

| # | mutação | resultado |
|---|---|---|
| 1 | `AREA_COMMANDS` sai do `left_rail::populate` | ✗ *(nasce morto sob o dedo)* |
| 2 | o fecho sai do `pre_dispatch` | **SOBREVIVEU** ⇒ a cura era inerte, e foi retirada |
| 3 | o interruptor sai do `apply` | **SOBREVIVEU** ao gate de gesto ⇒ levou ao gate da paleta |
| 4 | idem, com o gate da paleta | ✗ |
| 5 | desarmar deixa de limpar | ✗ |
| 6 | os nove chips crus voltam à fila | ✗ *(2 linhas até no 12,9")* |

### §28.6 — O que fica ABERTO no degrau `G`

O painel `3D Model` perdeu **9** das 74. Faltam, com o dono já escrito na D2:

- `export.*` (3) → **menu Arquivo** (barra global — é do app, não do editor);
- `add.*` (20) → o pulldown da área (⚠️ hoje é **um** chip que abre a paleta de formas — conferir se
  já não está resolvido antes de o mover);
- `kind`/`act`/`verb`/`op`/`mode` (17) → o pulldown da área, **como pulldowns separados** ou secções
  de um só — ⚠️ isto é a pergunta de desenho que a próxima wave tem de responder ANTES de mover:
  um único pulldown com 40 linhas é um depósito com outro nome.
- ⏳ e os **outros painéis** ainda não foram censados — o número «66 de 74» é do `3D Model` só.

---

## §29 — ⛔⛔⛔ O 3D DESENHAVA NA JANELA INTEIRA: a área e as réguas (entrega 34)

> Enio, 2026-08-31, com duas setas na foto: *«A viewport ainda não se encaixa na área correta para
> ela. Veja que atravessa as réguas. Tente encaixar corretamente, inclusive com as 4 viewports ao
> mesmo tempo.»*

### §29.1 — A causa, em uma linha

`render_loop/mod.rs` chamava `field3d_smoke::draw(Rect::new(viewport.x, viewport.y, …))` — **o
ecrã inteiro**. A porta da divisão (`field3d_layout::rects`) ladrilha o que recebe **exactamente**
(há gate desde a W90), então ela ladrilhava a *janela*: por baixo da barra de menus, da fila de
ferramentas, da coluna da esquerda e das duas réguas. Com a divisão aberta o efeito duplica — as
costuras caem onde não há área e a moldura do activo sai pela borda do ecrã.

⚠️ **A divisão nunca esteve errada.** O que estava errado era o rectângulo a que ela era aplicada —
e é por isso que o corte de responsabilidade dos gates (§29.5) separa *«como a área se divide»* de
*«que área é essa»*.

### §29.2 — A cura: uma porta, e ela responde depois das RÉGUAS

| peça | onde |
|---|---|
| a lei | `ph2d_editor_core::ruler::content(canvas, rulers_on)` |
| o rect publicado | `HeroScreen::last_content`, escrito no `hero/paint.rs` |
| a porta do módulo | `field3d_layout::area(hero, viewport)` |
| os consumidores | o **desenho** (os 4 viewports) e o **gizmo de navegação** |

⭐ **A porta mudou-se do `field3d_navball` para o `field3d_layout`**: o dono de *«que rectângulos o
canvas 3D ocupa»* é este módulo, e o gizmo é só um dos clientes. *Uma porta com o nome de um dos
seus clientes convida à segunda cópia.*

⚠️ **E o pointer sai de graça:** `viewport_at`/`canvas_area`/`divider_cursor` derivam tudo das áreas
que os viewports guardaram, que vêm do mesmo `rects(area, split)`. Um call site, quatro consumidores.

### §29.3 — ⚠️ Por que o 3D recua e a cena 2D não

A `HeroLayout::draw_area` **declara** que a cena 2D é *full-bleed*, por baixo das réguas (dar-lhe uma
origem é a obra da docagem, nomeada com o preço). Isso não é incoerência: o 3D **é** um viewport com
moldura e divisão, e uma régua de coordenadas 2D por cima dele não é *chrome de borda* — é uma faixa
a comer a imagem. ⇒ a lei vive numa **função à parte** (`ruler::content`), e quem a quer, pede-a.

### §29.4 — ⚠️ A condição foi HOISTADA, e é a parte que podia apodrecer

`rulers_on = hero.rulers_live() && hero.grid.view.is_some()` passou a ser **uma** ligação, lida por
quem pinta as réguas **e** por quem publica o recuo. ⛔ Se cada metade perguntasse por si, um quadro
sem `grid.view` publicaria `20 px` de recuo contra uma régua que não foi pintada — a mesma doença de
duas metades a divergir, com o sinal trocado. O gate de fonte que já existia
(`the_ruler_is_painted_with_the_canvas_the_layout_resolved`) foi **alargado** para exigir as duas
metades da mesma ligação.

### §29.5 — Gates (5 novos/alargados) e as 3 mutações

| gate | onde | o que prende |
|---|---|---|
| `the_content_area_never_shares_a_pixel_with_either_ruler` | core | a LEI, em 4 rects, e que ela não encolhe **mais** do que as faixas |
| `an_area_too_small_for_a_ruler_keeps_all_of_itself` | core | a mesma porta do predicado (`live_bands`) |
| `the_module_lives_inside_the_rulers_and_the_four_pieces_stay_there` | shell | a porta lê `last_content`, e os **4** quadrantes ficam dentro em 3 posições da costura |
| `with_the_rulers_off_the_module_takes_the_whole_drawing_area` | shell | sem réguas não há recuo |
| `the_three_d_module_is_drawn_into_the_area_never_into_the_window` | shell (FONTE) | **quem alimenta** — a única que apanha o defeito do report |

| # | mutação | resultado |
|---|---|---|
| A | a porta volta a ler `last_canvas` (as réguas dentro) | ✗ |
| B | o desenho volta a receber `viewport` (o defeito) | ✗ *(só o gate de fonte o vê)* |
| — | o gate de fonte alargado | ✗ com a condição não-hoistada |

⚠️ **A mutação B é a lição repetida:** os gates de `field3d_layout` passam o rect **à mão**, então
todos ficam verdes com o produto a alimentar a janela. *Um gate sobre a lei não é um gate sobre quem
a alimenta* — é a nota que a porta já carregava do gizmo, e esta é a **segunda** vez que ela morde.

### §29.6 — ⚠️ O tecto de LOC cortou o ficheiro de gates, e o corte é por responsabilidade

`field3d_layout_tests.rs` foi a `657/600`. ⇒ `field3d_area_tests.rs` (irmão): *«que área é esta»*
saiu de *«como ela se divide»*. ⛔ Não uma excepção na lista.

---

## §30 — ⭐⭐⭐ O `3D Model` ESVAZIA-SE: dois pulldowns e o menu File (entrega 35)

**Commit:** ver `git log`. **Degrau do plano:** `G` (*esvaziar os painéis*), a continuação da §28.

### §30.1 — O que mudou de sítio, e por que para DOIS sítios

| fileira | nº | destino | critério da **D2** |
|---|---:|---|---|
| verbos do gizmo (`Move`/`Rotate`/`Size`) + referencial (`Global`/`Local`) | 5 | **2.º pulldown de área** (*Gizmo*) | vale só naquele editor |
| níveis de exportação (`Draft`/`Fine`/`Max`) | 3 | **menu global → File** | escrever um arquivo vale em **todo o app** |

⇒ com a §28 (9 entradas), o painel perdeu **17 das 74**.

⛔ **O corte é por ÂMBITO, e não por quem foi o último a mexer no assunto.** A exportação é do
módulo 3D — e mesmo assim não é do editor 3D: ela escreve um ficheiro, que é uma operação do app. A
tabela de destino da D2 (`00_DECISOES_DO_ENIO.md` §D2) já dizia *«barra global → Arquivo»*.

### §30.2 — ⭐⭐ DOIS pulldowns e não um: o critério é a FACE

O `AreaMenu` tem **três** campos, e cada um responde a uma pergunta diferente:

| campo | pergunta | exemplo |
|---|---|---|
| `label` | *o que é este grupo?* | `View` · `Gizmo` |
| `face` | ***qual é o estado AGORA?*** | `Front` · `Rotate` |
| `rows` | *o que ele abre* | as 6 vistas + os 3 gestos de câmera |

⭐ **É a `face` que faz um chip valer o lugar FECHADO.** Um chip cuja face não distingue nada custa
a mesma largura e não informa. ⇒ **dois grupos com a mesma face seriam um grupo só** — e catorze
linhas atrás de uma face única seriam o depósito da foto 3 mudado de sítio, que é literalmente o
defeito que o degrau `G` existe para curar.

⭐ **E o referencial fica com o VERBO, não com a vista.** *«Um referencial sem verbo não quer dizer
nada»* — é o que o `paint.rs` do painel já dizia quando as duas fileiras eram vizinhas. A tabela da
D2 agrupava-o com `view.*`/`camera.*` (o `11`); separá-lo do verbo poria metade de uma lei num sítio
e metade noutro.

### §30.3 — ⭐ O ORÇAMENTO DE CHIPS DA ÁREA É `3`, e foi MEDIDO antes de o desenho ser escolhido

Sonda sobre `tool_bar::bar_split` + `widget::horizontal_lines`, módulo armado, os três alvos:

| alvo | largura da área | 1 chip | 2 | 3 | 4 |
|---|---:|---|---|---|---|
| iPad 12,9" | `754,0` | 1 linha | 1 | 1 | 1 |
| iPad 11 | `582,0` | 1 linha | 1 | 1 | **2** |
| iPad mini | `521,0` | 1 linha | 1 | 1 | **2** |

⭐⭐ **A medição DECIDIU o desenho, e não o contrário.** A pergunta em aberto era se os três verbos
do gizmo podiam ser chips CRUS na fila (um clique, como no Godot). A resposta é **não**: `1 + 3 = 4`
chips de área põem os dois alvos pequenos em duas linhas. ⇒ um 2.º **pulldown**, que cabe.

⚠️ O `MAX_AREA_MENUS = 4` é um tecto de **REGISTO** (quantos ids o `left_rail` cunha às cegas), não
o limite real — quem manda é a largura, e quem a mede é o gate, que corre com o que o módulo de
facto publicou. Ultrapassar não parte nada (o `⋯` absorve); custa a 2.ª linha.

### §30.4 — ⭐⭐ UMA aritmética para os dois destinos

O pintor do menu passou a somar **estáticas + contribuídas** para **todo** `ContextMenuKind`:

```rust
let statics = menu_rows::menu_rows(req.kind);           // a tabela do app
let contrib = match req.kind {                          // o que o módulo publicou
    ContextMenuKind::AreaCommands { slot } => store.area_menu_rows(slot),
    kind => store.menu_contrib(kind),
};
```

⭐ **Um pulldown de área é simplesmente o caso em que as estáticas são `&[]`**, e o *File* é o caso
em que as duas existem. ⛔ A versão anterior tinha um `if` só para o `AreaCommands` — a segunda
porta que apodrece.

⚠️ **`ContextMenuKind::AreaCommands` ganhou `slot: u8`, e a identidade tinha de estar ali:** com os
dois chips a abrir o mesmo `kind`, o pintor teria de guardar *qual deles* nalgum sítio à parte, e
essa segunda verdade divergiria do chip que o dedo carregou. **Mutação A** prova-o.

### §30.5 — ⚠️ Três correcções ao que a tabela da D2 dizia

1. **A célula `add.*` (20) já estava FECHADA e a tabela não sabia.** A paleta de formas (W100)
   reduziu-a a **um** chip que abre o catálogo genérico da casa (busca, categorias, rolagem).
   ⛔ Quem for pegar nela pelo número `20` reconstrói trabalho já pago.
2. **A tabela contradiz o próprio critério no `verb`/`kind`.** Ela manda `mod.*` ficar no painel
   *«porque é propriedade do objecto»* — e o **verbo** (com que operação esta forma dobra sobre o
   resto) e o **carácter** da mistura são propriedades do objecto pela mesma régua. Ficam.
3. **O `11` não é uma unidade.** Ele é `view(6) + camera(3)` num pulldown e `frame(2)` noutro, pela
   razão do §30.2.

### §30.6 — ⛔ `AreaMenu` NÃO vive em `src/widget/`, e dois opt-outs foram o sinal

A 1.ª versão pô-lo no `widget/tool_rail.rs`. Isso disparou **três** portões, em cadeia:

| portão | o que disse |
|---|---|
| `widget_primitives_under_loc_cap` | `tool_rail.rs` a `523/500` |
| `every_widget_is_shown_or_explicitly_opted_out` | `area_menu` não aparece na galeria |
| `every_widget_file_wires_a11y` | `area_menu.rs` não liga semântica nenhuma |

⭐⭐ **Dois opt-outs para o MESMO ficheiro é o sinal de que ele está no sítio errado, não de que os
gates são chatos.** O `AreaMenu` não pinta nada e não tem semântica de acessibilidade — o chip que
o mostra é um `ToolRailEntry::Compound`, que a galeria já cobre. ⇒ ele vive em
`interaction/area_menu.rs`, ao lado do `types_menu` (*que menu a área contribui* é a mesma pergunta
que o `ContextMenuKind` responde do outro lado), e **os três portões ficam verdes sem uma folga**.

### §30.7 — As cinco MUTAÇÕES

| # | o que se apagou | quem morreu |
|---|---|---|
| A | `slot: u8::try_from(slot)` → `slot: 0` (os dois chips abrem o mesmo menu) | `the_gizmo_pulldown_is_a_second_menu_and_serves_the_verb_and_the_frame` |
| B | a contribuição para o `MenuBarFile` → `Vec::new()` | `the_file_menu_serves_an_export_and_the_module_owns_it` |
| C | o merge deixa de somar as estáticas (`statics.iter().copied()` → `empty()`) | idem, na asserção *«as linhas que o menu já tinha desapareceram»* |
| D | o 2.º pulldown passa a usar a face do 1.º | `each_pulldown_wears_its_own_reading` (`["Right","Right"]` contra `["Right","Rotate"]`) |
| E | o ramo `!armed` deixa de escrever vazio | `closing_the_module_takes_the_commands_off_the_bar_and_off_the_file_menu` |

⚠️ O gate de gesto carrega em **pixels** (Down + Up pelo `dispatch_pointer`) — um `Click` sintético
passaria com o chip morto sob o dedo, que é como o `⋯` nasceu.

### §30.8 — O que fica ABERTO

- ⏳ **As operações booleanas e as acções** (duplicar/apagar/isolar) não têm destino decidido. Elas
  são gestos **sobre a selecção**, o que não é nem *propriedade do objecto* nem *comando do editor*
  no vocabulário limpo da D2 — a 3.ª categoria que a tabela dela não tem.
- ⛔ **Nenhum outro painel foi censado.** O `66 de 74` é do `3D Model` e só dele; qualquer número
  para os outros 24 painéis seria inventado.
- ⏳ O 3.º slot de área está livre e **medido** (cabe). O primeiro módulo que o queira herda a
  tabela do §30.3 — e a linha do gate que a imprime.

---

## §31 — ⛔⛔⛔ O GIZMO JÁ TINHA CONTROLOS, E ELES ESTAVAM MORTOS (entrega 36)

> Enio, 2026-09-01, com foto do trilho: *«esses botões de mover, rot e scale já existiam. só não
> estavam ligados a cada modo.»*

**Isto é uma CORRECÇÃO da §30, não uma wave nova.** O 2.º pulldown de área (*Gizmo*) que a §30
construiu foi **apagado**, e no lugar dele os chips que já existiam passaram a conduzir o gizmo.

### §31.1 — A medição que o report obrigou a fazer, e que eu não tinha feito

`chrome/rail_tools.rs` faz dos quatro chips um **rádio exclusivo**… que só escreve o próprio
`ButtonState`. Censo em toda a árvore:

| controlo | escreve | **quem lê** |
|---|---|---|
| `TOOL_TRANSLATE` · `TOOL_ROTATE` · `TOOL_SCALE` | o próprio `ButtonState` | **o pintor do chip, e mais ninguém** |
| ~~`TOOL_PIVOT`~~ | o próprio `ButtonState` | ⛔ **ERRADO — ele tem DOIS leitores.** Ver §32 |
| `TOOL_SPACE` | `store.tool_space_local` | **a face do próprio chip, e mais ninguém** |

⛔⛔ **São a 2.ª espécie de controlo morto do `CLAUDE.md` §5.0** — *o clique chega, a luz acende, e o
valor não alcança consumidor nenhum*. E são-no **nos dois modos**: o gizmo 2D também não os lê.

### §31.2 — ⭐⭐⭐ A LEI, e ela custou uma entrega inteira

> **Um controlo MORTO e um controlo AUSENTE produzem o mesmo report — e as curas são OPOSTAS.**
> Para o ausente constrói-se; para o morto **liga-se**. Construir por cima de um morto deixa o app
> com **dois** sítios para o mesmo verbo, e o que apodrece é o que ninguém relê.

⇒ **antes de dar casa nova a um comando, procure o controlo que já a tem.** O sintoma que eu li
(*«o painel 3D tem uma fileira de verbos que não devia estar num painel de propriedades»*) estava
certo; a conclusão (*«logo, faça-lhe um pulldown»*) saltou a pergunta *«e quem mais já faz esta
pergunta na tela?»*.

⚠️ **Nenhuma sonda deste repo faz essa pergunta**, e a §5.0 já o dizia: o
`architecture_panel_wiring_parity` mede *focalizabilidade*, os `seam_*` provam que o clique **chega
à ferramenta** — nenhum pergunta se a **escrita** dele chega a um **efeito**.

### §31.3 — O que a ligação é, mecanicamente

| chip | intent | ponte |
|---|---|---|
| `MOVE` / `ROT` / `SCALE` | `SetGizmoMode { slot }` | a **CHAVE i18n** do verbo → posição em `ModelSnapshot::modes` |
| `SPACE` | `SetGizmoFrame { slot }` | *o referencial SEGUINTE* — ele é um interruptor e a fileira é uma lista |

⚠️ **Pela CHAVE, nunca pelo índice.** Uma tabela `[TOOL_TRANSLATE → 0, …]` seria uma **segunda
contagem**: acrescentar um verbo ao `Mode::ALL` do shell faria o `SCALE` mandar rodar, e nada
acusaria. `RAIL_VERBS` é **uma** tabela, lida pela luz **e** pelo clique.

⭐ **A LUZ vem da CENA.** Com o módulo armado o painel consome estes ids **antes** do
`chrome::dispatch_all`, logo o rádio do `rail_tools` nunca corre — quem acende é
`area_bar::publish_rail_state`, do retrato. *Fiar o clique não é fiar o ESTADO.*

⚠️ **A bandeira `armed` é obrigatória e é nova** (`state::set_armed`): o `HeroScreen::apply_event`
entrega todo evento a **todo** painel do registry, visível ou não — sem ela um módulo 3D fechado
roubaria o `MOVE` ao editor 2D. ⛔ Deduzi-la do retrato não serve: o retrato sobrevive ao fecho, e
um retrato velho responderia *«sim»* para sempre.

⛔ **O `PIVOT` fica de fora:** o gizmo deste módulo tem três verbos (`Mode::ALL`) e nenhum é *mover
o pivô*. Inventar-lhe um destino seria pior do que a ausência — há gate a afirmá-lo
(`a_rail_chip_with_no_verb_behind_it_does_nothing`).

⛔⛔ **CORRECÇÃO (§32): o `PIVOT` NÃO é um dos mortos, e esta secção disse que era.** Ele tem dois
leitores no shell. Mecanismo do meu erro e a cura: §32.

### §31.4 — ⭐ Ligar e apagar é UMA obra, não duas

As famílias `model3d_mode_button` e `model3d_frame_button` **morreram**, com os dois braços de
`event.rs`, as duas linhas do `CHIP_FAMILIES` e as duas fileiras do `paint.rs`. Ficar com as duas
famílias deixaria ids registados que nada pinta — a **3.ª** coisa que a §5.0 cataloga (*o id
ÓRFÃO*), cuja cura é apagar e cujo sintoma se lê igual ao do morto.

⚠️ **Os gates de costura foram RE-APONTADOS, não apagados:** eles carregam agora no `ROT` e no
`SPACE` do trilho, que é o gesto real. *Um gate que segue o controlo é cobertura; um gate apagado
com o controlo é cobertura perdida.*

### §31.5 — As quatro MUTAÇÕES

| # | o que se apagou | quem morreu |
|---|---|---|
| F | `RAIL_VERBS` faz o `ROT` apontar para o verbo `move` | `clicking_a_verb_reaches_the_gizmo_intent` (slot `0` em vez de `1`) |
| G | a chamada a `publish_rail_state` | `the_rail_verb_lights_up_from_the_scene_not_from_the_click` |
| H | a guarda `state::armed()` no braço do verbo | `clicking_a_verb_reaches_the_gizmo_intent`, na metade *«módulo fechado não rouba o clique»* |
| I | um 2.º pulldown de área com os verbos lá dentro | `the_area_offers_no_second_door_for_the_gizmo` **e** `the_area_costs_one_chip_and_the_bar_is_still_one_line` **e** `each_pulldown_wears_its_own_reading` |

⭐ A mutação **I** é o gate contra repetir **o meu próprio erro** desta jornada.

### §31.6 — O que fica ABERTO (inalterado da §30)

- ⏳ As **operações booleanas** e as **acções** (duplicar/apagar/isolar) não têm destino decidido.
- ⛔ Nenhum outro painel foi censado — o `66 de 74` é do `3D Model` e só dele.
- ⏳ **O `PIVOT`, o `VIEW` e os chips do trilho no editor 2D continuam sem consumidor.** O `VIEW`
  (`TOOL_HOME`) empurra `SetViewFocus` e está vivo; os outros quatro só ficam vivos **enquanto o
  módulo 3D tem o canvas**. *Uma varredura de controlos mortos no trilho é wave própria, e esta
  entrega mediu o primeiro caso dela.*

---

## §32 — ⭐⭐⭐ O INSTRUMENTO QUE FALTAVA, e a correcção que o obrigou (entrega 37)

### §32.1 — ⛔⛔ Primeiro, o meu erro: `head -20` disse «zero leitores» e eu acreditei

A §31 afirmou que os **quatro** verbos de transformação não tinham consumidor. São **três**. O
`TOOL_PIVOT` tem **dois**, os dois no shell:

| ficheiro | o que faz com o valor |
|---|---|
| `shells/desktop/src/input_dispatch.rs` | lê o `ButtonState` para armar o arrasto **`GizmoDragKind::MovePivot`** |
| `shells/desktop/src/render_loop/snapshots.rs` | lê o mesmo para **realçar o ponto do pivô** |

O censo que os devia ter mostrado foi:

```
grep -rn "TOOL_TRANSLATE\|TOOL_ROTATE\|TOOL_SCALE\|TOOL_PIVOT" … | grep -v tests | head -20
```

⛔⛔ **O `head -20` cortou exactamente a linha do shell.** *Um `grep` truncado devolve «zero
consumidores» com a mesma cara de um `grep` completo* — é a família do
`feedback_a_swallowed_panic_silently_shrinks_the_candidate_set`, com outro instrumento a encolher a
população em silêncio.

⚠️ **Regra:** um censo que vai decidir se algo está MORTO corre **sem `head`** e **conta**. Se a
saída for grande, agrupe (`| sed 's|:.*||' | sort | uniq -c`) — nunca trunque.

### §32.2 — ⛔ E o erro tinha custado um DEFEITO, que este commit cura

Com o módulo 3D armado o painel consome os verbos **antes** do `chrome::rail_tools`, logo o rádio
exclusivo dele não corre — e um `PIVOT` aceso de antes **ficava aceso**. Dois chips acesos ao mesmo
tempo, e pior: a ferramenta de pivô do editor 2D continuava **armada** por baixo de um canvas do 3D.

⇒ `area_bar::publish_rail_state` apaga-o. *Quem toma o rádio herda a promessa dele: exactamente um
aceso.* Gate red-first: `the_pivot_chip_goes_dark_when_the_module_takes_the_rail`.

### §32.3 — ⭐⭐⭐ E o instrumento: `the_rail_names_a_consumer_for_every_chip`

> `CLAUDE.md` §5.0: *«⛔ Nenhum instrumento do repo pergunta se o VALOR chega a um consumidor.»*

Agora há um, para a fila. Ele não pergunta *«este id existe?»* nem *«alguém despacha este clique?»*
— os dois já têm gate. Pergunta o **terceiro passo**: *o que este chip escreve é LIDO por alguém que
decide?*

| metade | como |
|---|---|
| a **população** | **derivada** de `left_rail::rail_entries`, nos dois modos |
| o **veredito** | a tabela `RAIL_CONSUMERS`, uma linha por chip: `ReadBy(<ficheiro>)` ou `DeadOnPurpose(<mecanismo>)` |
| ⭐ a **prova** | o ficheiro declarado é **aberto e procurado** pelo nome do chip — a linha morre se o leitor sair |
| o **censo de obsolescência** | nenhuma linha descreve um chip que a fila já não pinta |
| o **controle positivo** | a população não pode vir vazia, e a regra do «morto de propósito» é exercida por um caso |

⭐⭐ **É a terceira metade que o torna um instrumento em vez de um comentário.** Uma tabela que só
afirmasse *«este tem consumidor»* envelheceria no dia em que o consumidor saísse — que é exactamente
como o rádio do trilho chegou a 2026 sem ninguém a lê-lo.

⭐ **O CENSO, medido:** **`20` chips da fila, `20` com consumidor, `0` declarados mortos.** Antes
desta jornada eram **quatro** sem leitor (`MOVE`/`ROT`/`SCALE` e o `SPACE`) — e ninguém sabia, porque
nada perguntava.

⚠️ **O `⋯` e os pulldowns de área ficam FORA da população, e é justo:** eles não saem do
`rail_entries` (o `⋯` vem do `bar_split` quando a linha transborda; os pulldowns vêm do módulo que
tem o canvas). Pô-los na tabela e não na população faria o censo de obsolescência acusá-los para
sempre. Cada um tem o gate dele.

⛔ **E o `DeadOnPurpose` tem barra:** o motivo tem de trazer o **mecanismo** da ausência (≥ 8
palavras, o critério do `WIDGET_OPT_OUT` da galeria). *«Ainda não ligámos» descreve um controlo
morto, não uma decisão.* ⚠️ A variante é mantida viva pelo **controle positivo**, não por um
`#[allow(dead_code)]`: *a cura de um aviso não é silenciá-lo, é dar-lhe o consumidor que falta.*

### §32.4 — As duas MUTAÇÕES

| # | o que se apagou | quem morreu |
|---|---|---|
| K | toda menção a `TOOL_TRANSLATE` no ficheiro declarado como consumidor dele | `every_declared_consumer_really_reads_its_chip` (*«declarado como consumidor e não menciona o chip»*) |
| L | o caminho do consumidor do `PIVOT` aponta para um ficheiro inexistente | idem (*«o ficheiro … não existe»*) |

### §32.5 — O que fica ABERTO

- ⏳ **A §5.0 do `CLAUDE.md` precisa de uma linha na integração:** ela declara que *nenhum*
  instrumento pergunta pelo consumidor, e isso deixou de ser verdade **para a fila**. ⛔ Continua
  verdade para os painéis, a topbar e os menus — a frase não se apaga, ganha um «excepto».
- ⏳ **A mesma sonda para a TOPBAR e para os menus** é a wave seguinte óbvia, e é onde o número de
  controlos é maior (148 itens `CTX_MENU_*`).
- ⏳ Do degrau `G`: as **operações booleanas** e as **acções** do painel 3D continuam sem destino
  decidido, e nenhum outro painel foi censado.

---

## §33 — ⭐⭐ A SONDA DOS MENUS, e uma AUDITORIA que parou uma obra (entrega 38)

Bloco pedido pelo Enio (*«siga implementando em blocos maiores»*). Ele produziu **um instrumento,
duas medições limpas e uma obra revertida** — e a obra revertida é o achado mais caro.

### §33.1 — ⛔⛔⛔ O degrau `B` foi CONSTRUÍDO e REVERTIDO: o plano e o gate discordavam

O [`spec/02 §2`](../spec/02_o_que_falta_para_comecar.md) listava, na tabela *«o que pode começar
HOJE»*: **`B` — fundir os 16 apelidos de cor · mudança de zero pixels · independente de tudo.**

Re-medi a premissa e ela **continua verdadeira**: `0` divergências em `64` comparações (16 pares ×
4 temas). Construí a fusão inteira — `40` sítios re-apontados, `16` variantes apagadas do
`ColorToken`, `57` entradas fora do `tokens.json`. O `cargo check` partiu num gate cujo nome eu não
tinha lido: **`the_sixteen_timeline_slots_are_pure_aliases`**, escrito por esta mesma linha, com o
veredito no doc-comment:

> *«equivalência não é desejabilidade, e é por isso que a fusão não foi feita aqui … A referência
> que licenciámos (Adobe Spectrum) mantém tokens de COMPONENTE a apelidar os globais **de
> propósito**: é o que deixa um componente divergir depois sem tocar nos consumidores. Fundir troca
> 16 nomes por 58 sítios de chamada directos, e transforma «a playhead passa a ter cor própria» de
> uma linha de token em sete de código. Essa é uma decisão de **design system**, do Enio — não uma
> dedução que este gate autorize.»*

**Revertido.** ⭐⭐⭐ **A lei:** *quando uma linha de plano e o GATE que mede o assunto discordam,
manda o gate* — ele foi escrito por quem foi lá medir; a linha do plano é o resumo que envelhece.

⚠️ E o modo de falha tem forma própria: **eu confirmei a PREMISSA e li isso como confirmação da
OBRA.** *Provar que uma mudança é inócua não é provar que ela é desejada.*

⇒ o `B` passa a estar marcado no plano como **recusa medida à espera do veredito do Enio**, ao lado
do `I` (quantos temas).

### §33.2 — E mais duas linhas do plano mandavam construir o que JÁ EXISTE

| degrau | o plano dizia | a medição |
|---|---|---|
| **`A`** — modelo de áreas | *«pode começar hoje»* | ✅ **feito**: `Slot`(6) + `SlotSet` + `Panel::{ALLOWED_SLOTS, DEFAULT_SLOT, CAN_FLOAT, TITLE}`, com os **dois** gates que a spec §3 pedia |
| **`E`** — remover a fuga do gizmo | *«tem de entrar no mesmo trabalho»* | ✅ **inerte por construção** desde o degrau `D`, e a fuga **fica** de propósito para quem declara `CAN_FLOAT` |

⚠️ ⭐ **O `DEFAULT_SLOT` chegou a ter default e foi RETIRADO por medição:** 20 dos 21 painéis
herdavam-no e **três mentiam**. *Uma declaração com default é decoração até alguém a confrontar.*

### §33.3 — ⭐⭐ O instrumento: `every_menu_row_reaches_a_handler`

A irmã da `the_rail_names_a_consumer_for_every_chip`, sobre a outra superfície. **Os dois lados são
derivados** — não há tabela a envelhecer:

| metade | como |
|---|---|
| população | todo `ids::NOME` no fonte do `menu_rows.rs`, menos as **8 TABELAS** da timeline |
| veredito | varredura da árvore por um sítio que **não** seja a declaração, a tabela que pinta, o **registo** ou um teste |
| controle positivo (população) | `≥ 100` linhas, senão o parser partiu e o gate ficaria verde por vácuo |
| controle positivo (varredura) | um id inventado tem de dar `0`; um que existe tem de dar `> 0` |

⭐ **MEDIDO: `110` linhas de menu, `0` sem sítio de despacho.** *Medir e confirmar É o resultado* —
o trilho tinha quatro mortos, os menus têm zero, e sem a sonda as duas frases teriam o mesmo peso.

⚠️ **`110` linhas e não `118` símbolos:** oito dos nomes são TABELAS de menu, e uma tabela não tem
handler — os itens dentro dela é que têm (cada uma com gate anti-item-morto próprio na
`ph2d-panel-timeline`).

### §33.4 — ⛔⛔ A MUTAÇÃO que reprovou a 1.ª versão do gate

| # | o que se apagou | resultado |
|---|---|---|
| M | o **braço inteiro** do *Save As…* no `chrome::io_menu` | ⛔ **SOBREVIVEU** — o gate ficou verde |
| M′ | o mesmo, com o gate a excluir também os sítios de **REGISTO** | ✅ morre, nomeando `CTX_MENU_SAVE_AS` |

⭐⭐⭐ **A causa é uma lei:** o id continuava a aparecer no `pre_populate.rs`, que **regista** o
widget. *Um id registado é um id que EXISTE, não um id que FAZ.* ⇒ a varredura exclui as quatro
famílias (`/ids/`, a tabela que pinta, o registo, os testes), e cada exclusão está na tabela do
doc-comment com a razão ao lado.

### §33.5 — As outras duas medições, ambas limpas

- **Painéis:** os `24` painéis têm todos pelo menos um ficheiro de costura que carrega num clique
  real (`MockPanelHost` / `apply_panel_event`). ⇒ a doença do trilho não se repete lá.
- **Menus:** ver §33.3.

### §33.6 — O que fica ABERTO, e é do Enio

O plano está **esgotado de itens que eu possa decidir sozinho**:

| item | o que falta |
|---|---|
| **`B`** — os 16 apelidos de cor | ⏳ **veredito**: fundir (paleta de 83 → 67 slots por tema) ou manter a indirecção que deixa a timeline divergir sem tocar em 58 sítios |
| **`I`** — quantos temas | ⏳ veredito (4 → 2 perde `workshop`/`blueprint`) |
| **o gesto de RECOLHER** | ⏸️ adiado por ele; é a maior razão custo/benefício medida (`89–92 %` de tela) |
| **`G`** nos outros painéis | ⚠️ o Inspector é o painel de propriedades **por definição** — não é lá que o `G` ganha. O alvo seguinte tem de sair de um censo por-linha, que é uma wave por painel |
