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
| régua esquerda tapada (1366 × 1024) | **87,8 %** | **0,0 %** |
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

### 3.4 As colunas

`DockSides::resolve(mirrored, visible)` é a porta única, e o `mirrored` **troca as colunas** (a
metade que se escreve ao contrário sem o compilador reclamar — tem gate próprio).
`RIGHT_DOCK_PANELS` lista as **5** chaves que partilham o dock da direita (o Inspector e os quatro
aliases: `bgremoval`, `padding`, `painter_sidebar`, `painter_layers`). O nome do campo **é** a
chave, e há **censo** a conferir a lista contra os `let X = inspector;` do construtor,
**descascando comentários** (documentar a cura não pode reprovar o portão).

⚠️ **Uma coluna fechada não é reservada** — a área cresce para dentro dela, senão a régua da
esquerda ficaria a flutuar no meio do desenho.

### 3.5 Gates (5 novos) e provas de mutação (5 aplicadas, 5 mortas)

`crates/ph2d-editor-core/tests/the_rulers_never_share_a_pixel_with_docked_chrome.rs`:

| gate | o que afirma |
|---|---|
| `the_rulers_never_share_a_pixel_with_docked_chrome` | 2 orientações × 4 estados de coluna × 2 faixas × N rects ⇒ intersecção **0** |
| `the_control_the_old_anchor_was_covered_and_the_measure_sees_it` | ⭐ **controlo**: com a âncora antiga, > 80 % / > 20 % tapada |
| `the_ruler_no_longer_steals_the_click_from_the_top_bar_and_the_rail` | ⭐ **controlo**: `hit` respondia `Some` sobre a barra e o trilho; agora `None` |
| `the_dock_column_census_matches_the_layout_aliases` | a lista descreve os aliases do construtor |
| `the_dock_sides_name_a_column_and_the_mirror_swaps_them` | a lei do espelho (que a lei geométrica **não** alcança) |

Mutações: `draw_area` colapsado no `canvas` (lei ✗) · espelho que não troca colunas (lei do
espelho ✗) · lista sem `painter_layers` (censo ✗) · as **duas** âncoras do `paint.rs` revertidas
(gate de fonte do shell ✗ nas duas). Árvore restaurada e verde em cada passo.

O gate de fonte do shell (`shells/desktop/tests/the_node_ops_are_wired.rs`) passa a **proibir o
regresso pelo nome**: `!contains("canvas: layout.canvas,")`.

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

⏳ **Falta o portão batched da workspace** (`scripts/nextest-impacted.sh` + clippy `--all-targets`
+ auditoria) e o `rm -rf target/*/incremental` — DIRETRIZ §1.5.9. A linha **não** está fechada.
