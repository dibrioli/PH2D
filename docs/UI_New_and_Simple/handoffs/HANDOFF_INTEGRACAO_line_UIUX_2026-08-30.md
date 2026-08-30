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
2. **`C` — a barra de menus.** Medido: o `screens/hero/topbar/` tem **zero** ocorrências de
   `open_context_menu`/`ContextMenuKind::` — **a barra de cima não abre menus hoje**. E **não
   existe cabeçalho de área** em lado nenhum (`area_header`/`AreaHeader`/`editor_header` → zero).
3. **`F` — Layouts + cabeçalho por área**, que é o que dá destino aos 11 comandos de câmera do
   painel 3D Model.
4. **`G` — esvaziar os painéis** (66 de 74 entradas do painel medido têm outro dono). É aqui que
   a área se ganha, não em `E`.
5. **`I` — cortar os temas 4 → 2**: **desbloqueado** (§5.1), à espera do veredito do Enio.

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
