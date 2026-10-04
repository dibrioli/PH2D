# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-04 — a F3 do Vector (Vector ▸ Object · Edit)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`…_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md),
> [`…_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md),
> [`…_2026-10-03_OS_MODOS.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md),
> [`…_2026-10-03_O_SCULPT.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md),
> [`…_2026-10-03_O_FLIP.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md) e
> [`…_2026-10-03_O_MODEL.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MODEL.md): os commits até
> `e99d5d72e` continuam descritos lá. Este cobre os DEPOIS e refaz o gate sobre o diff ACUMULADO.
>
> **Ordem do dono (03/10):** a F3 do Vector nesta linha, sem integrar. Medido antes de codar; três escolhas dele: **o pill VECTOR sai**; **a forma nova do Add nasce
> em Object**; **as ferramentas de criar** — primeiro escolheu 4 chips no trilho (Caneta · Lápis · Formas
> · Texto), o gate batched mostrou que não cabem no iPad (§5) e, re-perguntado com o número, escolheu o
> **menu Add**.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste ficheiro. `git cherry main HEAD` = **59 `+`** (60 com ele), nenhum integrado. Rebase: no-op.

## §0 — Para o `CLAUDE.md` §5.1

- **UI/UX:** trocar o link do «Último:» por este ficheiro. A frase não muda.
- **Vector + Esqueleto:** a frase não diz pill (diz smokes). Propõe-se acrescentar «Edit = `Tab` sobre
  a forma; criar = *Add ▸ Vector*», mantendo a entrada ≤ 700 bytes (confira o gate
  `architecture_claude_md_cabe_no_orcamento`).

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `030b0a5ed` | modos (F3 Vector): Object · Edit; o pill VECTOR sai; (4 chips no trilho — desfeitos em `575304e41`) |
| `8e2aa233c` | em Edit a forma fora do modo não desenha nós (`VecViewState::in_edit`) — achado pela FOTO |
| `feaf821e1` | spec/06 regista a entrega |
| `575304e41` | as ferramentas de criar vão para o menu Add, não para a fila (iPad) + curas do 1.º gate |
| `1924c2ac3` | fmt |

**O desenho:**
- **`ph2d_tool_vector::DrawMode::EDIT_TOOLS`** = Node · Fillet · Chamfer · Width · Trim, e
  `DrawMode::object_mode()` (ficheiro irmão `params_mode_object.rs`; o `params_mode.rs` estava no tecto
  700). **Edit** = as que mexem na forma NO LUGAR (o Trim escreve no mesmo id, `vec_trim.rs:108`);
  **Object** = as que criam (cada forma é uma linha da Hierarquia) e as que trabalham sobre várias.
- **`ph2d_app_vec::vector_mode::Family`** — `(Vector, Edit)`. Edit = ferramenta `vector` na mão com uma
  EDIT_TOOL + o alvo `VecState::edit: EditTarget { paths, following, last_tool, armed }` (fora da
  `VecScene`, que é o undo). Leis puras `holds(mode, is_target, tool)`, `adopt(tool, following)`,
  `releases(ours_now, following)`.
  - `enter` põe o Node (ou a última do Edit); `leave`/`follow` largam a ferramenta DO EDIT quando o Edit
    acaba (a de criar fica). `wants` = a porta antiga: 9 cenas `PH2D_BUILD_SMOKE` pedem Node/Trim/Fillet e
    o Edit entra sobre `pen.selected_paths().last()` (sem forma, espera). `parts` = as formas do Edit.
- **`VecViewState::editing` + `in_edit()`** (`ph2d-vec-scene/src/structure.rs`): a porta ÚNICA do clique
  (`is_pickable`: nó, quina, largura, aparar, caixa, laço — medido) E do desenho das âncoras
  (`ph2d-vec-render/src/overlays.rs`). Escrito em `render_loop/fase_vector_view_and_drives.rs`, levado ao
  gesto por `view_state_for_pick`.
- **Fundação — o MULTI-OBJECTO:** `ModeFamily::joins(mode)` e `enter_with(mode, entity, joined, tools)`
  (omissão: `false` / chama `enter`). No `mode_drive::drive`, um modo que junta leva ao entrar os
  seleccionados do MESMO tipo (`same_kind_selected`) sem colapsar a selecção (`select_together`) e
  devolve-os todos ao sair. Porquê: editar nós de várias formas (plano 25 §6, `PH2D_BUILD_SMOKE=70`/`71`/`81`)
  já era do módulo; o Blender faz o mesmo.
- **Menu Add:** `ph2d_app_vec::object_add::{PEN, PENCIL, TEXT}` (`TOOLS`) ao lado de `SHAPES`; `ENTRIES` = 7.
  `arm(entry, vec)` põe `edit.armed`; `render_loop/fase_object_add.rs` pede `ActivateTool("vector")`
  (`Born::Tool`, nada nasce); `Family::follow` aplica o modo quando a ferramenta chega à mão (o
  `ActivateTool` só se aplica depois do dreno). As formas do Add nascem em Object (escolha do dono).
- **Painel do vetor** (`ph2d-panel-vector/src/paint_modes.rs`): só mostra as ferramentas do modo em curso.
- **Saem** o pill VECTOR e *Window ▸ Vector*. A aba Vector de cima CONTINUA `CanvasOwner::Tool("vector")`
  (Object com a ferramenta na mão): com `Mode(Edit)` seria idêntica à aba Model (gate
  `no_two_layouts_hand_the_canvas_to_the_same_owner`).
- **`PH2D_OBJECT_MODE_SMOKE=6`** (`vector_mode::smoke_step`): rectângulo + elipse pelo Add, Edit na
  elipse, seletor aberto.

## §2 — Foundational tocado (aditivo) e contratos

- `ph2d-editor-core`: `mode_drive` (`joins`/`enter_with`/`same_kind_selected`/`select_together`) e as
  remoções do pill (`ids/chrome/topbar.rs`, `chrome/vector_toggle.rs` **APAGADO**, `chrome/mod.rs` com
  bloco gerado conferido por `cargo run -p ph2d-chrome-sync`, **42 → 41** handlers — na árvore combinada
  regenere —, `menu_bar.rs` `MODULE_TRUTHS` **19 → 18**, `menu_tables.rs`, `fixture.rs`, `topbar/mod.rs`);
  a prosa que citava o `TOPBAR_VECTOR` passou ao `TOPBAR_MOTION`.
- Aditivos: `ph2d-vec-scene`, `ph2d-vec-entities`, `ph2d-vec-render`, `ph2d-tool-vector`.
- `hero.rs` · `interaction/state/mod.rs` · `left_rail.rs` · `hero/paint.rs` (693–700 de 700): intocados.
- **Dependência nova:** `ph2d-app-vec → ph2d-component-desc` (vocabulário, zero deps, como Painter/Sculpt/
  Flip/Model). `Cargo.lock` pelo cargo.
- Contratos congelados (§6): **nenhum encostado** (`Tool=12`/`PanelEvent=4` intactos; o `DrawMode` vive
  em `ph2d-tool-vector`, fora do gate `architecture_vector_contract_surface`, que lê `ph2d-vector-doc` —
  conferido).

## §3 — Superfície de colisão (para o integrador)

**Falha ALTO noutra linha (não compila):**
- **APAGADOS:** `ids::TOPBAR_VECTOR`, `chrome::vector_toggle`; `MODULE_TRUTHS` **19 → 18**.
- `VecViewState` ganhou o campo `pub editing`: todo literal `VecViewState { … }` SEM
  `..Default::default()` noutra linha parte. `VecState` ganhou `pub edit`.
- `ph2d_app_vec::object_add::ENTRIES` **4 → 7**: quem iterar `ENTRIES` esperando que cada uma crie forma
  deve usar `SHAPES`.

**Funde LIMPO e REPROVA depois:**
- i18n **APAGADAS** `chrome.topbar.pill.vector`, `chrome.menu.vector`; **NOVAS**
  `object_add.vector.{pen,pencil,text}`. A fila de pills e o menu *Window* perderam a linha Vector.
- Catraca da altura do painel `vector` **1 262 → 1 239**
  (`ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs`): quem mexa no painel re-mede.
- `shells/desktop/tests/it/the_app_never_reshapes_a_still_screen.rs` passou a ABRIR o menu Add (sem ele:
  1 014 textos < tecto 1 024): quem tire texto do app pode voltar a derrubá-la.
- Gates **mudados:** `the_menu_bar_relocates_the_verbs_it_shows.rs` (toggles só Motion);
  `ph2d-panel-vector/tests/it/seam.rs` (pill pintado no modo dele + gate novo
  `the_tool_row_shows_only_the_tools_of_the_current_mode`); `every_object_mode_has_a_composed_family.rs`
  (FAMILIAS 4 → 5 com `&mut vector`; tabela D6 de PARES; gate novo `the_vector_edit_reaches_the_frame_view`).
- **`the_shell_only_shrinks`** (todo `.rs` de `shells/desktop`): HEAD do Model **196 841** → HEAD
  **196 909**; esta onda **+68**; acumulado desde a base **+300**; tecto **196 990** — folga **81**.

**Enums, ids e chaves:** nenhum id novo (`OBJECT_MODE_EDIT` já existia); env
`PH2D_OBJECT_MODE_SMOKE=6`.

**Muda comportamento:** pill e *Window ▸ Vector* saem; `Tab` sobre forma = Edit (só ela agarra e mostra
nós; várias = Edit de todas); o painel só mostra as ferramentas do modo; *Add ▸ Vector Pen/Pencil/Text*
põe a ferramenta na mão; as 9 cenas antigas entram sozinhas no Edit da forma seleccionada.

## §4 — Fecho: gate batched, mutação, auditoria

**Gate batched** (verificador, diff acumulado desde `1ad60a1ce`), 2 corridas. A 1.ª teve **8 falhas
reais** (clippy `single_element_loop`; os 4 chips na FILA no iPad 11 — `the_tool_bar_is_a_region_of_the_area`
e `the_area_hands_its_commands_to_the_bar_and_the_app_menu`; a fixtura do ecrã parado; a catraca do
painel): curas em `575304e41`. 2.ª (HEAD `575304e41`) + fmt `1924c2ac3`:

| portão | resultado |
|---|---|
| `nextest-impacted` | **21 086 / 21 087** — 1 flake de carga (abaixo) |
| `ph2d-panel-registry-init --all-features` | **136 / 136** |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `clippy --workspace --all-targets -D warnings` | verde |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `cargo fmt --all --check` | verde (após `1924c2ac3`) |

O flake: `ph2d-tool-painter … mask_tests::the_mask_stroke_cost_does_not_follow_the_canvas` («0.48 ms @1024²
vs 1.28 ms @2048²»), passa sozinho 3/3 a loadavg 13,9–17,5; **já é membro** de
[`FLAKES_DE_CARGA.md`](../../DevOps/FLAKES_DE_CARGA.md).

**Mutação** (agente `mutacao`; controlo VERDE com população > 0, âncora 1×): **20 / 20 sangraram**
(`in_edit`, `is_pickable`, overlays, cópia no pick; `holds`/`adopt`/`releases`; `enter_with`/`follow`/
`wants`/`drive`/devolver os juntos; Trim em Object, painel, `arm`; lista da shell, linha da vista, par do
Flip). Lado INDEPENDENTE: família FALSA que junta em `mode_drive_tests.rs`, a REAL em
`vector_mode_tests.rs`, o `PenTool` em `ph2d-vec-edit/src/multi_path_tests.rs` (com CONTROLO).

**Auditoria (DIRETIVA §3), três lentes** (LOC lidas ~2 000):
- **Correção — a outra forma intocada.** TRAÇO: `Tab` → `mode_drive::drive` `Step::Enter` →
  `vector_mode::Family::enter_with` → `VecState::edit.paths` → `fase_vector_view_and_drives.rs` →
  `pen.set_view` / `view_state_for_pick` → `VecViewState::is_pickable`/`in_edit` → `PenTool`
  (`on_press_node`, `hit_test`, `corner_tool`, `selection.rs`); `draw_overlays`. VERMELHAS: M1–M4/M19.
  NÃO-CHECADO: o press do Width e do Trim passa pela shell (`despacho_clique_vetor_premido.rs` →
  `path_at`), coberto pela porta mas sem gate que o conduza (§7c).
- **Fiação — *Add ▸ Vector Pen* até à mão.** pick → `fase_object_add.rs` → `object_add::arm` →
  `ActivateTool` → `fase_bus_tool_panel.rs` → `Family::follow`. VERMELHAS:
  `a_create_tool_entry_arms_the_tool_and_is_not_a_shape`, `a_real_click_on_every_entry_reaches_the_drain`.
- **Cadeado com multi-objecto:** `joins` → `parts` → `object_mode::decide`/`still_holds` (lei intacta).
  VERMELHAS: M8, M13, M14.

**Fotos** (`fotografa_cena.sh`), conferidas: `PH2D_OBJECT_MODE_SMOKE=6` — fila de cima sem VECTOR nem chips,
seletor com Edit marcado, painel só com Node · Fillet · Chamfer · Width · Trim (+ Marquee), Path 0/1 na
Hierarchy, nós laranja SÓ na elipse (a 1.ª foto mostrou as âncoras do rectângulo → `8e2aa233c`).
`PH2D_OBJECT_ADD_SMOKE=1` — o Add com 2D·9, incluindo Vector Pen, Pencil e Text.

## §5 — Premissas derrubadas

- *«14 variantes de `DrawMode`»* — são **17**. *«O `parts` serve os nós»* — os nós NÃO são entidades
  (`(VecPathId, usize)`); o `parts` serviu ao MULTI-OBJECTO (o Edit já editava várias formas).
- *«4 botões na barra esquerda»* — é a FILA de cima: +~160 px (~490 → ~650), não cabe no iPad 11 (582)
  nem no mini (521); a pergunta ao dono omitiu o número e foi refeita.
- *«A aba Vector pede o Edit»* — seria idêntica à do Model. A foto apanhou o que nenhum gate via
  (âncoras de formas não-agarráveis).

## §6 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Botão direito no canvas vazio → **Add Object** → **Vector Rectangle**. Nasce «Path 0», seleccionado,
   e o botão do topo diz **Object Mode**. Errado: diz Edit Mode.
3. **Tab** → **Edit Mode**: aparecem os nós da forma (quadradinhos laranja) e o painel **Vector** à
   direita mostra só **Node · Fillet · Chamfer · Width · Trim**. Arrastar um nó muda a forma. Errado: o
   painel mostra Pen/Shape, ou não há nós.
4. **Tab** → Object. **Add Object** → **Vector Ellipse**, arrastá-la para o lado. Com a elipse
   seleccionada, **Tab**: só a elipse mostra nós; clicar num canto do rectângulo não agarra nada; clicar
   «Path 0» na Hierarchy é recusado com o aviso *«Leave Edit Mode (Tab)…»*. Errado: o rectângulo mostra
   nós ou mexe.
5. **Tab** → Object; seleccionar as duas (Ctrl+clique na Hierarchy) e **Tab**: as duas mostram nós e uma
   caixa arrastada apanha os nós das duas. **Tab** → as duas continuam seleccionadas.
6. Botão direito → **Add Object** → **Vector Pen**: a caneta fica na mão (o painel Vector mostra **Pen**
   marcado e as outras de criar). Clicar pontos no canvas desenha um traço novo — nova linha na
   Hierarchy. O mesmo com **Vector Pencil** (arrastar) e **Vector Text** (clicar e escrever). Errado:
   nada fica na mão, ou nasce um objecto vazio.
7. A barra de cima já não tem **VECTOR** e o menu **Window** já não tem *Vector*; a fila de ferramentas
   de cima é a de antes (MOVE · ROT · SCALE · PIVOT · SPACE · VIEW · UNDO · REDO).

## §7 — O que fica para a próxima janela

- (a) O **Soldar** em Edit CONSOME as formas e cria uma nova (`weld.rs`, decisão do dono 31/08) ⇒ o Edit
  cai para Object com a rede seleccionada — nomeado, não curado.
- (b) Caneta DENTRO do Edit a acrescentar à forma (como o Figma) não existe — cada traço é um objecto.
- (c) O press do Width/Trim pela shell sem gate de condução.
- (d) `Ctrl+Tab`; Image ▸ Mask; F4 (MOVE/ROT/SCALE visível nos modos de criação); Model em Object não desenha.
- (e) A **F3 está COMPLETA** (Image, Sculpt, Flip, Model, Vector); a próxima é a F4 (layouts) ou o que o
  dono mandar.

## §8 — Perfil do loop (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                174   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                538 : 254   alvo: <= 1,0  razao 2.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  41%   alvo: >= 80%  (744 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         385 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §9 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Blocking waiting for file lock on package cache
    Finished `smoke` profile [optimized] target(s) in 0.28s
```
