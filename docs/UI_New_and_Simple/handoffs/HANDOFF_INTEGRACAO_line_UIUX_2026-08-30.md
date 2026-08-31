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
| 18 | *(a seguir)* | ⛔⛔ **UMA TABELA para a verdade de cada módulo** — e o `if` com um lado morto (§13) |

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

