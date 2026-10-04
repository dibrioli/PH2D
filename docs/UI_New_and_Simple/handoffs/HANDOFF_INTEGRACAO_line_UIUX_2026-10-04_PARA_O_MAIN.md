# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-04 — PARA O `main` (o ponto de entrada único)

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), e só por ordem do Enio. A linha
> FECHOU e PARA: nada integrado, nada enviado. **Este ficheiro é a porta; o mecanismo de cada onda
> está no handoff dela (§1), linkado.** Não substitui nenhum.
>
> **Ordem do dono (04/10)**, depois de fumar o Mask: *«não armou a máscara imediatamente. vamos retirar
> esse modo mask. Depois escreva handoff para integração com o main»*. O Mask saiu em `9bd27b989`
> (reverte `1d87241aa`, `f796202ff`, `94156f3e4`); o código = `a8385d0aa` + docs.

- **Branch** `line/UIUX` (worktree `Worktrees/line-UIUX`) · **HEAD** `9bd27b989` (+ o commit deste doc)
  · **base (merge-base = `main`)** `1ad60a1ce`.
- **O `main` NÃO andou** (0 commits em `1ad60a1ce..main`) ⇒ a integração é **fast-forward** se o `main`
  continuar assim; se andar, rebase/merge sobre o novo `main` e refaça o §4 e o §5.
- `git cherry main HEAD` = **80 `+`** antes do commit deste doc. `git diff --shortstat 1ad60a1ce HEAD` =
  **342 files, +15 530 / −2 477**.

## §0 — Para o `CLAUDE.md` §5.1 (a regra: ≤ 700 bytes por módulo; gate `architecture_claude_md_cabe_no_orcamento`)

Recolhido do §0 de cada onda. Medido hoje em `CLAUDE.md`: UI/UX **358** B · Vector + Esqueleto **551** B ·
3D Modeling **579** B (cada uma com o `\n`).

1. **UI/UX** — trocar o link «Último:» para **este ficheiro**; a frase NÃO muda.
2. **Vector + Esqueleto** — acrescentar (≈ +70 B ⇒ ≈ 620 B): «Criar = *Add ▸ Vector Object* (o contentor; as
   formas são filhas); Edit = `Tab`.» (É a proposta do O_OBJECTO_VETORIAL; **substitui** a do O_VETOR,
   «criar = *Add ▸ Vector*», que o dono recusou.)
3. **3D Modeling** — o pill MODEL saiu: trocar «Smoke pill MODEL · `PH2D_FIELD_SMOKE=<n>`» por «Smoke
   *Add ▸ Model* (nasce em Edit) · `PH2D_FIELD_SMOKE=<n>`» (≈ +15 B ⇒ ≈ 595 B). Proposta do O_MODEL.
4. Nenhuma outra onda propõe mexer no §5.1 (as demais: «só o link»). Confira com `wc -c` e corra o gate.

## §1 — A linha, onda a onda (por ordem; todas citam-se e NÃO se substituem)

Pasta: `docs/UI_New_and_Simple/handoffs/`. Nome = `HANDOFF_INTEGRACAO_line_UIUX_<data>_<NOME>.md`.

| # | Onda | O que entrega | Smoke do dono |
|---|---|---|---|
| 1 | [`2026-10-02_A_ESCALA`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md) | escala da interface (`UiScale` 80–200 %, *Edit ▸ Preferences ▸ Interface scale*), HiDPI (`scale_factor × UiScale`, `UiScaleMap::no_ecra`), coluna `F9` larga, texto no píxel do ecrã (`ao_pixel`) | APROVADO (03/10) |
| 2 | [`2026-10-03_O_MENU_ADD`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md) | `+` da Hierarchy e `Shift+A` abrem o menu Add de objectos (`ph2d_editor_core::object_add`; cada família declara e cria os seus; spec/06 F0+F1) | APROVADO («Smoke OK») |
| 3 | [`2026-10-03_OS_MODOS`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md) | `ObjectMode` por objecto activo: seletor *Mode*, Tab, `Ctrl+Space`=zen, cadeado, botão direito → Add; Image ▸ Paint; `ModeFamily` (spec/06 F2 + F3 Imagem) | APROVADO |
| 4 | [`2026-10-03_O_SCULPT`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md) | Sculpt ▸ Object · Sculpt · Paint; `ModeFamily` vira trait; pill SCULPT e *Window ▸ Sculpt 3D* saem | APROVADO |
| 5 | [`2026-10-03_O_FLIP`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md) | Flip ▸ Object · Draw · Edit; `ph2d_flip::FlipTarget` (o traço cai no desenho SELECCIONADO); pill FLIP sai | APROVADO |
| 6 | [`2026-10-03_O_MODEL`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MODEL.md) | Model ▸ Object · Edit; N peças por cena (`scene::root_in_hand`/`plant`); modo de PARTES (`parts`/`owner_of`); pill MODEL sai | APROVADO |
| 7 | [`2026-10-04_O_VETOR`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_VETOR.md) | Vector ▸ Object · Edit por forma (`DrawMode::EDIT_TOOLS`), Pen/Pencil/Text no Add; pill VECTOR sai; multi-objecto (`joins`/`enter_with`) | **RECUSADO** (F3). Desfeito em parte pela onda 8 (§2 dela): `EDIT_TOOLS`/`object_mode()`, o filtro da fileira do painel, as 7 entradas do Add. Ficam: pill fora, `joins`/`enter_with`, `in_edit`, o censo de famílias |
| 8 | [`2026-10-04_O_OBJECTO_VETORIAL`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_OBJECTO_VETORIAL.md) | o vetor é um OBJECTO-contentor (`VecObject`; formas filhas; regra das soltas `adopt_loose`; uma só entrada no Add; Edit sobre o objecto; gizmo/laço em Edit). **PROJECT_SCHEMA 178→179** | APROVADO (04/10), com a decisão «duplicar a linha de uma forma deixa a cópia AO LADO da original» (`place_beside`) e (a)/(b) do §8 mantidos |
| 9 | [`2026-10-04_O_CTRL_TAB_E_O_MASK`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_CTRL_TAB_E_O_MASK.md) | **Ctrl+Tab** abre/fecha a lista dos modos (`mode_drive::mode_key`, porta única com o Tab; âncora do `⋯`). O Model em 2D ficou MEDIDO e NÃO construído (§2) | Ctrl+Tab: **não aprovado à parte** (o dono só reportou o Mask) ⇒ §6. **Image ▸ Mask: RETIRADO pelo dono** |

### O que do handoff 9 deixou de valer (Mask retirado, `9bd27b989`)

Vale só o **Ctrl+Tab** (o §1(1), o §1(3) e o passo 1–2 e 8 do smoke). **Não vale:** o §1(2) e o §1(4)
inteiros; as partes Mask de §2 (`ObjectMode::Mask`, `ids::OBJECT_MODE_MASK`, `ModeFamily::refusal`,
`LayerStack::owner_of_mask`, i18n `object_mode.mask`/`object_mode.mask_needs_a_layer`), de §3 (a lista de
ficheiros e os contadores «`ALL` 5→6», «i18n +2») e de §4–§5; os **gates do Mask**; o smoke **passos 3–7**.
⛔ **NÃO EXISTEM mais** (verificado por `git grep`): o censo `a_painter_mode_is_asked_by_one_door` e
`ObjectMode::uses_the_painter`, `ModeFamily::refusal`, `LayerStack::owner_of_mask`, `ObjectMode::Mask`, a
cena `PH2D_OBJECT_MODE_SMOKE=7`. `ObjectMode::ALL` = **5** (`[_; 5]`). Registo e porquê: spec/06 F3 ▸
Image ▸ Mask — **não reconstruir sem ler** (o alvo punha-se no quadro SEGUINTE e a máscara nascia branca,
sem efeito visível; a causa exacta do «não armou» nunca foi investigada).

## §2 — O que fica ABERTO depois da linha

- **Model desenhado no canvas 2D, em Object** — **LINHA NOVA, por escolha do dono.** Medido: o trace é CPU
  (`ph2d_field_render::trace`), 46–57 ms/quadro a 560², viewport exclusivo em ecrã cheio; o `Transform`
  da raiz não é lido pelo cook (`ph2d-field-ecs/src/cook.rs:38–55`). Falta a ponte textura-GPU, o afim 2D
  do `Transform` da raiz e o z-order entre camadas 2D; ≈ 800–1 200 linhas em 3–5 crates.
- **F4 — layouts** (`spec/06`): o campo «modo ao abrir»; a barra MOVE/ROT/SCALE continua visível nos modos
  de criação (Sculpt/Flip/Model/Vector).
- **Image ▸ Mask**: recusado (acima); registado em spec/06.
- Do Vector: o **Soldar** em Edit consome as formas e cai para Object (nomeado, não curado); caneta DENTRO
  do Edit a acrescentar à forma não existe; o press do Width/Trim pela shell sem gate de condução.
- Do Model: a câmera do módulo é UMA (trocar de peça enquadra a nova).
- Do Sculpt: em Sculpt a barra da esquerda ainda mostra MOVE/ROT/SCALE (pré-existente, F4); o `Edit` da
  malha (D6) não existe.
- Um gate do despacho de teclado (Tab/Ctrl+Space/Ctrl+Tab) pede o `App`; hoje só a foto o vê.

## §3 — Foundational tocado e contratos congelados

**Contratos congelados (§6 do CLAUDE.md): NENHUM encostado** — lido em cada onda (`Tool=12`/`PanelEvent=4`/
`NodeOp=2`… intactos; `DrawMode` vive em `ph2d-tool-vector`, fora do gate `architecture_vector_contract_surface`,
que lê `ph2d-vector-doc`, intocado). **ADR novo: nenhum** (sem ficheiro novo em `docs/architecture/decisions/`).

**Foundational aditivo:** `ph2d-tokens` (`UiScale`) · `ph2d-editor-core` (`object_add`, `object_mode`,
`screens/hero/mode_drive`, `ui_scale`, chrome `object_mode_menu` z=46, `tool_activation`, `task_layout`) ·
`ph2d-i18n` (tabelas `object_add`, `object_mode`) · `ph2d-ecs` (`vec_object.rs`, `VecObject`) ·
`ph2d-component-desc` (⚠️ `ObjectKind::Sculpt3D::marker()` passou de `BakedForm` a `Sculpt3dPieceRef`;
`ObjectKind::Vector` lê `VecObject`) · `ph2d-flip` (`FlipTarget`; `FlipState::active_layer` APAGADO) ·
`ph2d-field-ecs` (`unique_root_name` `pub`) · `ph2d-vec-{scene,entities,render}`.

**Dependência nova** (`→ ph2d-component-desc`, vocabulário, zero deps): `ph2d-editor-core`, `ph2d-app-painter`,
`ph2d-app-sculpt3d`, `ph2d-app-flip`, `ph2d-app-field3d`, `ph2d-app-vec` ⇒ `Cargo.lock` +6 linhas.
**Regenere, não resolva à mão:** `Cargo.lock` (cargo) e o bloco gerado de `chrome/mod.rs`
(`cargo run -p ph2d-chrome-sync`: 44→41 handlers ao longo da linha).

**CONTAR, nunca escolher (recontar na árvore combinada):**
- **`PROJECT_SCHEMA` 178 → 179** — só a onda 8 (`VecObject` registado). Ficheiros: `shells/desktop/src/project_schema.rs`
  (linha da constante) **e** a tripla `(179, 13, 22)` em `project_schema_tests.rs`; **a tripla não vê o degrau**.
  Se outra linha também subir, o degrau reconta-se: `python3 scripts/schema-recount.py`.
- **Registos de componentes:** `ph2d-ecs` `registry_tests` 108→109; espelhos `ph2d-render`/`ph2d-script` 109→110 (**+1** cada).
- **`MODULE_TRUTHS`** 22→18 (pills SCULPT, FLIP, MODEL, VECTOR saem; `ShellOwned` apagado).
- **Cenas `PH2D_OBJECT_MODE_SMOKE`: 1..6** (1 Painter · 2 Paint · 3 Sculpt · 4 Flip · 5 Model · 6 Vector); a **7 NÃO
  existe** (era o Mask). Outra linha que queira cena nova usa a 7 — confira o gate `no_two_object_smoke_modes_claim_the_same_number`.
- **Catraca `the_shell_only_shrinks`** (tecto **196 990**, soma entre linhas; base 196 609): conta final em §5
  (colada pela janela principal). Tectos de 700 linhas: `hero.rs` 699, `hero/paint.rs` 700, `left_rail.rs` 693,
  `interaction/state/mod.rs` 696, `ph2d-ecs/src/scene/registry.rs` 700 — **duas linhas que acrescentem aqui somam contra o mesmo tecto**.
- Catracas: altura do painel `vector` 1 262 (= a base); Flip `CORTES_/LETRAS_PERDIDAS_NO_DEGRAU_ESTREITO` `flip` 2→4.

## §4 — Superfície de colisão

**Corra, em CADA worktree, pelo caminho ABSOLUTO** (CLAUDE.md §1; a coluna `base:` é o merge-base — leia o `main`
no ficheiro): `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/<wt>/scripts/collision-surface.sh`. Worktrees vivas
(`git worktree list` de hoje): `line-3DModeling` · `line-PainterWatercolor` · `line-Vector` · `line-components` ·
`line-editor-core` (`integ/refatoracao-final`) · `line-motion-value` · `line-sculpt3d` · `integ-arquivo-docs` ·
`line-contexto` (= `main`) · `line-UIUX` (esta).

**Pontos quentes desta linha (de cada onda):**
- **`ph2d-editor-core` `object_mode` / `screens/hero/mode_drive` / trait `ModeFamily`** (`modes, holds, enter, leave,
  follow, wants(&mut ToolRegistry), parts, owner_of, joins, enter_with, parts_take_the_object_gizmo`): quem mexer
  aqui colide; `line-editor-core` é o suspeito (partir `hero`). **Recorde: uma variante nova de `ObjectMode`, ou um
  `match` sobre ele, vindo de outra linha precisa de TODOS os braços** (hoje 5: Object · Paint · Sculpt · Draw · Edit;
  falha ALTO ao compilar). `CanvasOwner` ganhou `Mode(ObjectMode)` e
  **perdeu `Model3d`**: todo `match` exaustivo parte.
- **`shells/desktop` `render_loop`**: `fase_object_add.rs` (`FAMILIES`), `fase_object_mode.rs` (`MODE_FAMILIES`),
  `fase_hero_paint` (escritor único de `.escala_do_ecra`), `fase_vector_*` (`tree_settle`, `view_and_drives`,
  `layout_recook`), `fase_image_tools_mode_and_pills`, `fase_tool_mirrors`; `input_handlers.rs` (Tab/Ctrl+Tab/Ctrl+Space).
  A shell só encolhe (§3).
- **APAGADOS (outra linha que os use não compila):** `ids::TOPBAR_{SCULPT3D,FLIP,MODEL3D,VECTOR}`, `chrome::{sculpt3d,flip,
  model3d,vector}_toggle`, `EditorAction::ToggleSculpt3d`, `Sculpt3dRequests::{toggle_request,take_toggle}`,
  `ph2d_app_sculpt3d::mode::{apply_toggle,sync_pill}`, `painter_lock`, `FlipState::active_layer`, `FlipMode::Select`,
  `CanvasOwner::Model3d`, `object_add::why_not` (Model), `strip_resolve::target`; as i18n `chrome.topbar.pill.{sculpt,
  flip,model,vector}`, `chrome.menu.{sculpt_3d,flip,model_3d,vector}`; gates apagados `the_sculpt_pill_…`,
  `the_model_pill_opens_the_3d_module`. Um censo de «pills/linhas do menu Window» vindo de outra linha reprova.
- **`ph2d-vec-*` / `ph2d-app-vec` / `ph2d-ecs` (`line-Vector`, `line-components`)**: `VecViewState::editing` é
  `Option<Vec<VecPathId>>` (todo literal parte); `VecState.edit`; `Family::new(vec, sim)`; `object_add::add(entry, sim,
  vec, at)`; `ObjectKind::Vector` = `VecObject` (**forma lê `Empty`**); `group_entities`/`ungroup_entities`/`selection_root`
  respeitam a fronteira do objecto; a regra das soltas muda a **contagem de raízes/`RootOrder`** de qualquer cena
  com formas (gates de outra linha que contem raízes mudam de número).
- **`ph2d-flip` / `ph2d-app-flip` / `ph2d-tool-flip`**: `FlipTarget` em ~14 leitores; `object_add::add(…, &mut FlipState)`.
- **`ph2d-app-sculpt3d`** (`line-sculpt3d`): `Sculpt3dScene` ganhou `preso` e `pede_o_modo` (literal só em `birth.rs`);
  o `D` já não entra no barro; fora de Sculpt/Paint o barro sai.
- **`ph2d-app-field3d` / `ph2d-field-ecs` (`line-3DModeling`)**: `scene::root_in_hand` substitui os 4 `q.iter().next()`
  (um `iter().next()` novo sobre `FieldObject` edita SEMPRE a 1.ª peça); `scene::plant` (luz por CENA).
- **`ph2d-app-painter` (`line-PainterWatercolor`)**: `paint_mode::Family`; o botão Painter saiu da barra IMG.
- **i18n**: tabelas NOVAS `object_add.rs`, `object_mode.rs` + linha na cadeia do `tr_ingles`; chaves
  `chrome.menu.scale_*`, `chrome.topbar.pill.settings`=`PREFS`; **fusão de tabelas = append-only, nunca regenerar de um lado**.
- **Memória:** a linha acrescenta 3 ficheiros em `project-memory/` e toca `MEMORY.md` e o tópico
  `reference_topic_code_pattern_gotchas.md`; o `main` já tem `MEMORY.md` modificado na árvore — funda e recheque o tecto de bytes.

## §5 — Gate final (HEAD 9bd27b989, BASE 1ad60a1ce)

Agente `verificador`, uma corrida, diff acumulado desde `1ad60a1ce`:

| portão | resultado | `load` (1 min) |
|---|---|---|
| `nextest-impacted` (BASE `1ad60a1ce`) | **21 108 / 21 108** (o flake da família, `the_mask_stroke_cost_does_not_follow_the_canvas`, passou) | 17,7 → 17,2 |
| `ph2d-panel-registry-init --all-features` | **140 / 140** | 17,2 → 18,1 |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde | 18,1 → 49,0 |
| `clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings` | verde | 45,3 → 66,5 |
| `cargo fmt --all --check` · `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` (32 nomes / 397 membros) · `doc-index.sh --check` (20 índices) | verdes | — |
| catraca `the_shell_only_shrinks` | **196 986 / 196 990** (folga 4) | — |

## §6 — Smoke do dono AINDA por fazer, no `main` integrado

1. `cd ~/Documentos/Projetos/PH2D && PH2D_OBJECT_MODE_SMOKE=1 ./target/smoke/ph2d-host-desktop` — cria-se uma
   imagem, selecciona-se, e a lista «Object Mode» abre sob o botão do topo.
2. **Ctrl+Tab** fecha a lista; **Ctrl+Tab** outra vez abre. Errado: nada acontece, ou alterna o zen.
3. Com nada seleccionado, **Ctrl+Tab** não abre nada.
4. Numa imagem, o seletor mostra **só** *Object Mode* e *Paint Mode* (sem *Mask Mode*). **Tab** alterna os dois.
5. Com um chip transbordado para `⋯`, **Ctrl+Tab** ancora a lista sob o `⋯` (não em `(0, topo)`).

## §7 — Binário de smoke (2.ª corrida)

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.32s
```
