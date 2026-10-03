# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-03 — a F3 do Flip (Flip ▸ Object · Draw · Edit)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`…_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md),
> [`…_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md),
> [`…_2026-10-03_OS_MODOS.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md) e
> [`…_2026-10-03_O_SCULPT.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md): os commits até
> `f6eaf5351` continuam descritos lá. Este cobre os DEPOIS e refaz o gate sobre o diff ACUMULADO.
>
> **Ordem do dono (03/10):** a F3 do Flip nesta linha, sem integrar. Medido antes de codar, o desenho
> mostrado numa frase por peça; duas escolhas dele: **o pill FLIP sai** e **o desenho novo nasce em
> Draw**.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **46 `+`** com este, nenhum integrado. Rebase: no-op.

## §0 — Para o `CLAUDE.md` §5.1 (UI/UX)

Trocar o link do «Último:» por este ficheiro. A frase do módulo não muda. (Flip: a frase dele também
não; o smoke `PH2D_FLIP_HARDNESS_SMOKE` continua a entrar sozinho — ver §1.)

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `048508edf` | `ObjectMode::{Draw, Edit}`; `ph2d_flip::FlipTarget`; `ph2d_app_flip::flip_mode::Family`; `ModeFamily::wants(tools)`; `FlipMode` sem `Select`; o painel por modo; o pill FLIP sai; `PH2D_OBJECT_MODE_SMOKE=4`; censos do painel |
| `784856137` | o censo `every_object_mode_has_a_composed_family` (a lista de famílias da shell) |

**O desenho** (spec/06 §4 F3 ▸ Flip, escrito lá com os desvios):
- **`ph2d_flip::FlipTarget { object, layer }`** (`crates/ph2d-flip/src/target.rs`) — o ALVO da
  autoria: o desenho em edição + a camada activa nele (`resolve`, `layer_in`, `drawing`). Substitui
  `FlipState::active_layer`. As **14** leituras de «o 1.º objecto» (`autokey::target_drawing` — a
  porta dos 5 gestos —, `bridge` ×2, `layers` ×2, `pass` ×2, `select::visible_key`, `strip::fps`,
  `strip_drag`, `strip_resolve::target` (apagado), `transform::active_pose`, `draw_app`, os dois
  `flip_gesturing`) passam por ele; **10** cópias da regra «a activa, senão a de topo» apagadas.
  `target.object == None` (Object) ⇒ nenhum gesto cai. ⛔ Não mora no `FlipDoc`: ele é `PartialEq`
  para o diff do undo, e entrar num modo viraria passo de histórico.
- **`ph2d_app_flip::flip_mode::Family<'a>`** (`&mut FlipState`, `&FlipDoc`) — `(Flip, Draw)`,
  `(Flip, Edit)`. Leis puras `holds(mode, is_target, tool)`, `adopt(tool, following)`,
  `releases(now, following)`.
  - **Entrar** = ferramenta Flip na mão + `target.object` = o desenho da entidade (outro desenho ⇒
    larga a camada e as chaves marcadas da tira) + uma ferramenta do grupo (a última desse modo,
    `FlipState::last_tool`, senão a 1.ª). **Sair** = alvo `None`, ferramenta de omissão.
  - **`follow`** escreve o alvo em todo quadro; a ferramenta só sai quando um modo do Flip ACABOU
    (`FlipState::following`).
  - **`wants(tools)`**: o desenho nascido (`FlipState::born`, marcado pelo `object_add`) pede o
    Draw; e a **porta antiga** — a ferramenta Flip na mão sem o modo dela (as ~20 cenas
    `PH2D_FLIP_*_SMOKE` que fazem `tools.set_active("flip")`) — pede o modo do grupo dela sobre o
    alvo vivo, senão o 1.º desenho. Nenhuma cena antiga foi editada.
- **`FlipMode`**: `Select` saiu (é o Object); `DRAW_TOOLS` (Draw · Erase · Fill · Colorize · Trace),
  `EDIT_TOOLS` (Edit · Reshape), `object_mode()`, `tools_of()`. Default = `Draw`. O painel
  (`paint_sections::mode_row` + `tool_button`, pública) só pinta as do modo; no Edit lêem-se
  «Select · Sculpt»; a fileira chama-se «Tool».
- O layout **Flip** (`task_layout`) = `CanvasOwner::Mode(Draw)`. Activar a ferramenta num documento
  vazio **já não cria um objecto** (`fase_tool_mirrors`, §6.5 do spec).

## §2 — Foundational tocado (aditivo) e contratos

- `ph2d-editor-core`: `object_mode` (+ `Draw`, `Edit`), `ids/chrome/rail.rs` (+ 2 ids),
  `mode_drive` (`wants` com o registo), `task_layout`, e as remoções do pill (§3). `hero.rs` **699** ·
  `interaction/state/mod.rs` **696** · `left_rail.rs` **693** · `hero/paint.rs` **700** ·
  `action_bus.rs` **647** — intocados.
- `ph2d-flip`: `target.rs` novo (+ re-export). **Dependência nova:** `ph2d-app-flip →
  ph2d-component-desc` (vocabulário, zero deps — como Painter e Sculpt). `Cargo.lock` pelo cargo.
- `chrome/mod.rs`: bloco gerado regenerado por `cargo run -p ph2d-chrome-sync` (**44 → 43** handlers).
- Contratos congelados (§6): **nenhum encostado** (`Tool=12`/`PanelEvent=4` intactos).

## §3 — Superfície de colisão (para o integrador)

**Falha ALTO noutra linha (não compila):**
- `FlipState::active_layer` **deixou de existir** → `FlipState::target` (`FlipTarget`). Toda linha que
  toque o Flip (gestos, tira, painel de camadas) e o leia parte — a cura é `target.layer` / `target`.
- Assinaturas que trocaram `Option<LayerId>` por `FlipTarget`: `autokey::target_drawing`,
  `select::{visible_key, visible_drawing}`, `layers::apply_panel_event` (`&mut FlipTarget`),
  `bridge::publish`, `strip::fps`, `transform::active_pose`, os de `edit_gesture`/`pose_gizmo`/
  `selection_gizmo`/`tween_correct`/`select_points`/`select_segment`/`selection_overlay`/`trace`.
  `strip_resolve::target` **APAGADO**.
- `FlipMode::Select` e `ids::FLIP_MODE_SELECT` **APAGADOS**; `FlipMode::ALL` 8 → 7.
- `ObjectMode` ganhou `Draw` e `Edit`: todo `match` exaustivo noutra linha parte (o Vector, ao
  partir o `DrawMode`, VAI usar o `Edit` — o tipo é `(Vector, Edit)`, a família é outra).
- `ModeFamily::wants(&mut self, &mut ToolRegistry)` (era sem argumento).
- `object_add::add(entry, sim, doc, &mut FlipState)` (era o `FlipEntityMap`).
- **APAGADOS:** `ids::TOPBAR_FLIP`, `chrome::flip_toggle`; `MODULE_TRUTHS` **21 → 20**.

**Funde LIMPO e REPROVA depois:**
- A fila de pills e o menu *Window* perderam a linha FLIP.
- i18n **APAGADAS:** `chrome.menu.flip`, `chrome.topbar.pill.flip`, `panel.flip.tool.edit`,
  `shell.fase_tool_mirrors.{flip,layer_1}`. **MUDADA:** `panel.flip.tool.mode` = «Tool».
- Catracas **SUBIDAS** (população, não regressão — anotado no sítio):
  `CORTES_NO_DEGRAU_ESTREITO` e `LETRAS_PERDIDAS_NO_DEGRAU_ESTREITO`, `flip` **2 → 4** (`Square`,
  `Squares`: as fileiras Tip/Cap do Draw, que a varredura não via com a fábrica no Select).
- Régua mudada: `selectores_de_cor` — painel sem campo sozinho na fileira usa a coluna da LEI
  (`caixa_do_controlo`) em vez de «sem coluna» (o Flip em Draw). M15 prova que não aprova qualquer x.
- `the_saved_layout_takes_the_canvas_…`: o controlo de população perdeu o `flip`.
- Gate NOVO: `shells/desktop/tests/it/every_object_mode_has_a_composed_family.rs` (+ `main.rs`).
- **`the_shell_only_shrinks`** (conta todo `.rs` de `shells/desktop`): base **196 609** → depois da
  Sculpt **196 862** → HEAD **196 914**; esta onda **+52** (src **−19**, o gate novo **+71**);
  acumulado **+305**; tecto **196 990** (folga 76).

**Enums, ids e chaves (append-only):** `OBJECT_MODE_DRAW = hash_node_id("object_mode.row.draw")`,
`OBJECT_MODE_EDIT = hash_node_id("object_mode.row.edit")`; i18n `object_mode.draw` «Draw Mode»,
`object_mode.edit` «Edit Mode»; env `PH2D_OBJECT_MODE_SMOKE=4`.

**Muda comportamento:**
- O pill **FLIP** e *Window ▸ Flip* saíram. Desenhar = seleccionar o desenho + *Mode ▸ Draw Mode*
  (ou Tab), ou *Add ▸ 2D ▸ Flip* (nasce em Draw), ou a aba **Flip** de cima com um desenho activo.
- **Com dois desenhos o traço cai no SELECCIONADO** (antes: sempre o 1.º). O painel de camadas, a
  tira de quadros, a pré-visualização e o peek são os desse desenho.
- A seta preta do painel saiu; o gizmo do desenho é o do modo Object.
- A aba Flip sem desenho activo fica em Object e não cria um desenho.

## §4 — Fecho: gate batched, mutação, auditoria

**Gate batched** (verificador, 1× desde `1ad60a1ce`):

| portão | resultado |
|---|---|
| `nextest-impacted` (base = merge-base) | **19 873 / 19 873** (11 087 ignorados) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde, zero avisos |
| `clippy --workspace --all-targets -D warnings` | verde |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `cargo fmt --all --check` | o gate novo → formatado (`784856137`) |
| `ph2d-host-desktop --test it` · `ph2d-panel-registry-init --all-features` | **937** · **135** verdes |

Antes do gate, a corrida dirigida achou **3** vermelhos no `panel-registry-init`, todos POPULAÇÃO: a
fábrica do painel passou do `Select` (poucas fileiras) ao `Draw` (todas) — os dois cortes Tip/Cap e o
seletor de cor que a régua não sabia medir (§3).

**Mutação** (agente `mutacao`; controlo VERDE com população > 0, âncora 1×): **15 / 15 sangraram.**
M1 `drawing` → 1.º objecto · M2 `layer_in` sem o filtro · M3 `holds` sem o alvo · M4 `adopt` sempre ·
M5 `releases` sem a borda · M6 `enter` sem largar camada/chaves · M7 `enter` sem escrever o alvo ·
M8 sem a última ferramenta · M9 `born` consumido antes da entidade · M10 `object_add` sem `born` ·
M11 o painel pinta as 7 · M12 `DRAW_TOOLS` sem o Fill · M13 `&mut flip` fora da lista da shell ·
M14 a família sem o Edit · M15 a coluna da lei +8 px. Lado INDEPENDENTE: a família REAL com
`FlipDoc` + `FlipTool` verdadeiros (`flip_mode_tests.rs`); o quadro continua com as falsas.

**Auditoria (DIRETIVA §3), três lentes** (LOC lidas ~1 400):
- **Correção — o outro desenho intocado.** TRAÇO: `despacho_clique_flip.rs:22` → `draw_app.rs:103`
  `flip_canvas_down` → `:189` `bake::bake_stroke` → `bake.rs:40` → `autokey.rs:54`
  `target.resolve(flip)`; o alvo vem de `flip_mode.rs:118` (enter) / `:146` (follow). VERMELHA:
  `two_drawings_draw_on_one_and_the_other_is_untouched` (controlo: em Object nada cai; M7).
  NÃO-CHECADO: o gesto inteiro com câmara (pede `App`) — o funil é o `target_drawing`, a única porta.
- **Fiação — do seletor à ferramenta.** TRAÇO: linha do seletor → `EditorAction::ObjectMode` →
  `fase_hero_frame.rs:54` → `fase_object_mode.rs` (a lista) → `mode_drive::drive` →
  `flip_mode::Family::enter`. VERMELHAS: `every_mode_family_is_in_the_frame_list` (M13),
  `the_composed_families_declare_every_creation_mode` (M14), `the_mode_selector_answers_a_real_click`.
- **Coerência ferramenta ↔ modo.** Portas que põem a ferramenta na mão: o modo (enter), as cenas
  antigas (`wants` → `adopt`), a troca de grupo (`adopt`, M4). Portas que a tiram: Tab/Object
  (`leave`), a rede `still_holds` (`follow` → `releases`, M5). Uma porta NOVA que ponha a ferramenta
  sem modo é adoptada no quadro seguinte, por desenho.

## §5 — Premissas do briefing que a medição derrubou

- *«O traço cai SEMPRE no 1.º objecto (`autokey.rs:54`)»* — não era UMA porta: eram **14** leituras
  (painéis, pré-visualização, peek, pose, assentar a origem) e 10 cópias da regra da camada. Curar só
  o `autokey` deixava o painel de camadas a mostrar o desenho errado.
- *«`Draw` e `Edit` são modos dentro da ferramenta»* — o `FlipMode` era 1 modo (Select = Object) + 7
  ferramentas em dois grupos; os `Pairs` não são modo.
- *«As cenas de smoke do Flip terão de aprender o modo»* — não: a ferramenta na mão PEDE o modo.
- O handoff da Sculpt dava a shell em **«líquido −41»** desde a base; a régua da catraca (todo `.rs`)
  mede **+253** nesse ponto. O número de lá estava errado; o tecto continua respeitado.

## §6 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Botão direito no canvas vazio → **Add Object** → **Flip**. Nasce «Flip Drawing» na Hierarchy, o
   botão do topo diz **Draw Mode**, e à direita o painel **Flip** com *Tool: Draw · Erase · Fill ·
   Colorize · Trace*; em baixo, **Frames**. Errado: diz Object Mode, ou aparece «Select» na Tool.
3. Desenhar no canvas → aparece o traço.
4. **Draw Mode** → a lista tem **Object Mode · Draw Mode · Edit Mode** → **Edit Mode** → a Tool passa
   a **Select · Sculpt**; clicar no traço selecciona-o. Errado: a Tool continua com Draw/Erase.
5. **Tab** → Object Mode: o painel Flip some e aparece o quadradinho azul de mover o desenho.
6. **+** da Hierarchy → **Flip** outra vez → um 2.º desenho, já em Draw. Desenhar nele: o 1.º não
   muda, e o painel de camadas e a tira são os do 2.º. Errado: o traço aparece no 1.º.
7. Em Draw, clicar o 1.º desenho na Hierarchy → não troca, com o aviso *«Leave Draw Mode (Tab)…»*.
8. A barra de cima já não tem **FLIP**, e o menu **Window** já não tem *Flip*.
9. Tab até Object, seleccionar uma imagem (Ctrl+N) e clicar a aba **Flip** lá de cima → arruma os
   painéis e fica em Object (não cria desenho).

Foto (tela virtual, `fotografa_cena.sh`): `PH2D_OBJECT_MODE_SMOKE=4` — o desenho novo em Draw com o
seletor aberto nas três faces e o painel só com as ferramentas do Draw. Conferida.

## §7 — O que fica para a próxima janela

- **F3 Model** (um por cena: a ponte coze só a 1.ª raiz) → **Vector** (a partição do `DrawMode`; o
  `ObjectMode::Edit` já existe).
- O Flip no degrau estreito corta «Square»/«Squares» (Tip/Cap) — da família dos ~46 cortes fora do
  Inspector (handoff de 20/09 §7).
- `Ctrl+Tab`; Image ▸ Mask; F4 (a barra MOVE/ROT/SCALE continua visível nos modos de criação).

## §7b — Depois do fecho (03/10)

- **Smoke do dono: APROVADO** («smoke OK. Siga»).
- A F3 do Model segue NESTA linha, noutra janela:
  [`HANDOFF_CONTINUACAO_line_UIUX_2026-10-03_F3_MODEL.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-03_F3_MODEL.md).

## §8 — Perfil do loop (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                195   alvo: <= 800
  ✗ cargo test : cargo check                480 : 236   alvo: <= 1,0  razao 2.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  39%   alvo: >= 80%  (732 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         373 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## §9 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.20s
```
