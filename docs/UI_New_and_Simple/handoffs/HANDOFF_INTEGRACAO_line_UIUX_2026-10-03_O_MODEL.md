# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-03 — a F3 do Model (Model ▸ Object · Edit)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`…_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md),
> [`…_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md),
> [`…_2026-10-03_OS_MODOS.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md),
> [`…_2026-10-03_O_SCULPT.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md) e
> [`…_2026-10-03_O_FLIP.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md): os commits até
> `84e07e956` continuam descritos lá. Este cobre os DEPOIS e refaz o gate sobre o diff ACUMULADO.
>
> **Ordem do dono (03/10):** a F3 do Model nesta linha, sem integrar. Medido antes de codar, o desenho
> mostrado numa frase por peça; duas escolhas dele: **o pill MODEL sai** e **a peça nova nasce em
> Edit**.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **52 `+`** com este, nenhum integrado. Rebase: no-op.

## §0 — Para o `CLAUDE.md` §5.1

- **UI/UX:** trocar o link do «Último:» por este ficheiro. A frase não muda.
- **3D Modeling:** a frase diz «Smoke pill MODEL» — o pill saiu. Trocar por «Smoke *Add ▸ Model*
  (nasce em Edit) · `PH2D_FIELD_SMOKE=<n>`».

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `568a0f3bc` | `model_mode::Family`; `scene::root_in_hand` + `scene::plant`; N Models no Add; `ModeFamily::{parts, owner_of}`; o cadeado de PARTES; o layout Modeling pede o Edit; o pill MODEL sai; `PH2D_OBJECT_MODE_SMOKE=5`; gates |
| `c8a280443` | o `mut` a mais que o `-D warnings` apanhou; o spec/06 regista a entrega |
| `1bae0c60b` | o sobrevivente da mutação (M14): em Edit sem nada seleccionado o seletor continua a oferecer o Edit |

**O desenho** (spec/06 §4 F3 ▸ Model, com os desvios):
- **`ph2d_app_field3d::model_mode::Family`** — `(Model3D, Edit)`. Edit = o painel `model3d` aberto
  (que ARMA o módulo, `set_armed_by_panel`) com a peça EM MÃOS (`model_mode::target`, thread-local:
  ⛔ fora do mundo, que é o documento do undo). Leis puras `holds(mode, is_target, panel_open)`,
  `releases(ours_now, following)`, `wanted(born, in_hand, panel_open, closing, following, asked)`.
  - **Entrar** = alvo + pedido de abrir o painel (`take_panel_request`, tirado pela shell); outra peça
    pede o enquadramento (`ask_frame_the_part`). **Sair** = alvo `None` + fechar o painel.
  - **`follow`**: o painel SEGUE o modo — um Edit que acabou sem `leave` (o X do painel, outra
    ferramenta que tomou o canvas, a peça apagada: `holds` falso ⇒ o quadro não chama `leave`)
    fecha-o.
  - **`wants`**: a peça nascida pelo Add (`model_mode::born`) pede o Edit; e a PORTA ANTIGA — o
    painel aberto sem o modo (load com peça `ask_open_panel_if_part`, `PH2D_FIELD_SMOKE`, a sonda do
    undo) — pede-o UMA vez por abertura sobre a peça em mãos. Nenhuma cena antiga foi editada.
- **`scene::root_in_hand`** (`scene.rs:220`) — o alvo do modo, senão a 1.ª raiz. Substitui os quatro
  `q.iter(world).next()`: cozer (`sync_scene_and_birth`), o gizmo/pick (`scene_gizmo.rs`), os
  materiais (`materials::sync`). O documento já era recozido do MUNDO a cada quadro — N peças não
  pediram um documento por raiz.
- **`scene::plant`** (`scene.rs:292`) — o nascimento, partilhado pela semente e pelo Add: nome único
  entre raízes (`ph2d_field_ecs::unique_root_name`, agora `pub`), material da semente, ⛔ **uma luz
  por CENA** (a 2.ª peça não acende uma 2.ª lâmpada).
- **O modo de PARTES (fundação):** no Model editar é seleccionar as formas de DENTRO da peça — linhas
  próprias na Hierarquia. O cadeado exigia a selecção EXACTA; o Edit cairia no 1.º clique.
  - `ModeFamily::parts(entity) -> Option<Vec<u64>>` (`None` = o modo edita o objecto inteiro, a lei
    de antes) e `ModeFamily::owner_of(bits)` (uma parte responde pelo dono).
  - `ModeState::{publish_parts, parts}`; `still_holds(sel, extras: &[u64], held)` aceita qualquer
    parte (várias, nenhuma); `decide(locked, parts, target, additive)` aceita parte e acrescentar
    entre partes.
  - `mode_drive::publish_active`: num modo o activo é a entidade TRANCADA (o seletor não some com
    uma forma seleccionada); em Object uma parte responde pelo dono (`Tab` sobre uma forma = Edit da
    peça). `Step::Leave` de um modo de partes devolve a selecção à peça inteira.
  - As partes do Model = as formas dela + as luzes da cena (a luz é pickável no traçado).
- **Layout Modeling** = `CanvasOwner::Mode(Edit)` e `open` sem `model3d`. ⛔ Antes, a aba abria o
  painel, que armava o módulo e, sem peça, **plantava a demo** (um layout a criar objecto, §6.5).

## §2 — Foundational tocado (aditivo) e contratos

- `ph2d-editor-core`: `object_mode` (campo `parts`, `publish_parts`, `parts`, assinaturas de
  `still_holds` e `decide`), `mode_drive` (`parts`/`owner_of` com omissão, `publish_active`),
  `task_layout` (`CanvasOwner::Model3d` APAGADO), `layout_switch`, e as remoções do pill (§3).
  `hero.rs` **699** · `interaction/state/mod.rs` **696** · `left_rail.rs` **693** · `hero/paint.rs`
  **700** — intocados.
- `ph2d-field-ecs`: `unique_root_name` passa a `pub` (+ re-export).
- **Dependência nova:** `ph2d-app-field3d → ph2d-component-desc` (vocabulário, zero deps — como
  Painter, Sculpt e Flip). `Cargo.lock` pelo cargo.
- `chrome/mod.rs`: bloco gerado conferido por `cargo run -p ph2d-chrome-sync` (**43 → 42**
  handlers, zero diff sobre a remoção à mão). Na árvore combinada, regenere — não resolva à mão.
- Contratos congelados (§6): **nenhum encostado** (`Tool=12`/`PanelEvent=4` intactos; o Model não é
  `Tool`).

## §3 — Superfície de colisão (para o integrador)

**Falha ALTO noutra linha (não compila):**
- `ModeState::still_holds(sel, extras: &[u64], held)` (era `usize`) e
  `object_mode::decide(locked, parts, target, additive)` (+ `parts`).
- `CanvasOwner::Model3d` **APAGADO** — todo `match` sobre `CanvasOwner` noutra linha parte.
- **APAGADOS:** `ids::TOPBAR_MODEL3D`, `chrome::model3d_toggle`, `chrome::MODEL3D_PANEL_KEY`;
  `MODULE_TRUTHS` **20 → 19**.
- `ph2d_app_field3d::object_add::{why_not}` **APAGADO**; `add(sim) -> u64` (era `Result<u64, &str>`).
- Uma linha que toque o 3D Modeling e escreva um `q.iter().next()` sobre `FieldObject` volta a
  editar SEMPRE a 1.ª peça — a porta é `scene::root_in_hand`.

**Funde LIMPO e REPROVA depois:**
- i18n **APAGADAS:** `chrome.topbar.pill.model`, `chrome.menu.model_3d`, `chrome.topbar.name.n3d_model`,
  `chrome.topbar.tip.n3d_model_implicit_field`, `object_add.model.one_per_scene`.
- A fila de pills e o menu *Window* perderam a linha MODEL.
- Gate **APAGADO** (ficheiro inteiro, os dois testes eram do pill):
  `shells/desktop/tests/it/the_model_pill_opens_the_3d_module.rs` (+ `main.rs`).
- Gates **mudados:** `switching_layout_rearranges_the_screen` (o Modeling já NÃO abre o `model3d`),
  `the_saved_layout_takes_the_canvas_…`, `a_layout_never_commands_a_panel_a_bridge_owns`,
  `no_two_layouts_hand_the_canvas_to_the_same_owner` (o braço `Model3d` saiu),
  `every_object_mode_has_a_composed_family` (+ `&mut model`; a 2.ª pergunta conta PARES (tipo, modo)
  e reprova um par declarado por duas famílias — antes, o Edit do Flip escondia o do Model).
- **`the_shell_only_shrinks`** (todo `.rs` de `shells/desktop`): HEAD do Flip **196 914** → HEAD
  **196 841**; esta onda **−73** (o gate do pill −72, src −1); acumulado desde a base **+232**; tecto
  **196 990**.

**Enums, ids e chaves:** nenhum id novo (o `OBJECT_MODE_EDIT` já existia); env
`PH2D_OBJECT_MODE_SMOKE=5`.

**Muda comportamento:**
- O pill **MODEL** e *Window ▸ Model 3D* saíram. Modelar = seleccionar a peça + *Mode ▸ Edit Mode*
  (ou Tab), ou *Add ▸ 3D ▸ Model Solid* (nasce em Edit), ou a aba **Model** de cima com uma peça
  activa.
- **N peças por cena**: o Add planta ao lado («Model 2», …); o traçado, o gizmo, o *Add shape…* e os
  materiais são os da peça em Edit; a outra fica intocada.
- Em Edit a selecção anda pelas formas e pelas luzes; clicar outra peça é recusado com o aviso.
- A aba Model sem peça activa fica em Object e **não** planta a demo.

## §4 — Fecho: gate batched, mutação, auditoria

**Gate batched** (verificador, 1× desde `1ad60a1ce`):

| portão | resultado |
|---|---|
| `nextest-impacted` (base = merge-base) | **19 879 / 19 879** |
| `ph2d-panel-registry-init --all-features` | **136 / 136** (33 ignorados) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | 1 `unused-mut` no gate novo → `c8a280443` → verde |
| `clippy --workspace --all-targets -D warnings` | idem → verde |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `cargo fmt --all --check` | verde |

**Mutação** (agente `mutacao`; controlo VERDE com população > 0 — 5 · 29 · 2 testes —, âncora 1×):
**16 / 16 sangraram** depois da cura do M14. M1 `root_in_hand` sem o alvo · M2 `holds` sem o painel ·
M3 `releases` sem a borda · M4 `wanted` sem o `asked` · M5 as partes sem as luzes · M6 `owner_of`
mudo · M7 `enter` sem abrir o painel · M8 uma luz por peça · M9 nome repetido · M10 `still_holds`
exacto também nas partes · M11 o cadeado sem as partes · M12 o activo sem o dono · M13 o Tab sem
devolver a peça · **M14** o activo sem a entidade trancada (sobreviveu: o gate só olhava o seletor com
uma forma seleccionada, e a forma responde pelo dono pelo outro caminho; cura `1bae0c60b`: com
NADA seleccionado o seletor tem de oferecer o Edit — confirmado VERMELHO na linha 481 sob a mutação) ·
M15 `&mut model` fora da lista da shell · M16 o Model a declarar o par do Flip. Lado INDEPENDENTE:
a família FALSA de partes no editor-core (`mode_drive_tests.rs`), a REAL na crate dela
(`model_mode_tests.rs`, `object_add_tests.rs`).

**Auditoria (DIRETIVA §3), três lentes** (LOC lidas ~1 600):
- **Correção — a outra peça intocada.** TRAÇO: *Add shape…* → `smoke::ask_shape` →
  `sync_scene_and_birth` (`scene.rs:373`) → `root_in_hand` (`:382` → `:220`) → `model_mode::target`
  ← `enter` (`model_mode.rs:166`) / `follow` (`:197`) → `where_to_add(world, root, …)`. VERMELHA:
  `two_pieces_edit_in_one_and_the_other_is_untouched` (M1, M7). NÃO-CHECADO: o clique no traçado com
  câmara (pede `App`) — o pick sai do mesmo `root_in_hand` (`scene_gizmo.rs`).
- **Fiação — do seletor ao módulo armado.** TRAÇO: linha do seletor → `EditorAction::ObjectMode` →
  `fase_hero_frame.rs:54` → `fase_object_mode.rs:38/44` → `mode_drive::drive` → `Family::enter` →
  `take_panel_request` em `fase_field3d_requests.rs:121` (mesmo quadro) → `panel_visibility` →
  `set_armed_by_panel` (`fase_world_panel_bridges.rs:123`). VERMELHAS: M15/M16 (o censo da shell),
  M7. Foto `PH2D_OBJECT_MODE_SMOKE=5` (abaixo).
- **Coerência do cadeado com partes.** As três portas de selecção (`fase_hierarchy_select_lock.rs:41`,
  o clique e o laço) perguntam a `mode_drive::refused` (`:204`), que lê `ModeState::parts` publicadas
  pelo quadro (`:137`, e logo ao entrar `:159`); o pick do traçado escreve a selecção pela ponte e é
  a rede `still_holds` (`:139`) que o julga. VERMELHAS: `a_mode_of_parts_holds_and_admits_its_parts_only`,
  `a_mode_of_parts_lets_the_selection_move_inside_the_piece` (M10–M14). Paint/Sculpt/Draw não
  declaram partes: a lei exacta deles ficou (os 29 gates do modo, verdes).

**Foto** (`fotografa_cena.sh`, `PH2D_OBJECT_MODE_SMOKE=5`): «Model» + «Sphere» + uma «Light» na
Hierarchy, o seletor aberto *Object Mode · Edit Mode* (Edit marcado), o painel Model 3D, a esfera
traçada com o gizmo 3D, e a barra de cima sem o MODEL. Conferida.

## §5 — Premissas do briefing que a medição derrubou

- *«Uma 2.ª raiz nunca é cozida — o item caro; um documento por raiz?»* — não foi caro: o documento
  já é recozido do MUNDO a cada quadro, e «que peça?» eram **quatro** `q.iter().next()`. Fora do
  Edit a peça não se desenha (como a escultura), então traçar N campos não foi preciso.
- *«Reuse o `ModeFamily`»* — reusou, mas com um desvio na FUNDAÇÃO: no Model a selecção é das
  PARTES, e o cadeado exacto derrubaria o Edit no 1.º clique. Não estava no briefing.
- *«O que seria Object é mover a peça inteira no canvas 2.5D (D9)»* — hoje o `Transform` 2D da raiz
  NÃO entra no cozimento (`cook.rs` compõe só a pose do pai): o gizmo do objecto não move o traçado.
  Fica nomeado (§7), não é desta onda.
- A aba Modeling sem peça **plantava a demo** — um layout que criava objecto, violando o §6.5 do
  spec. Curado com o `CanvasOwner::Mode(Edit)`.

## §6 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Botão direito no canvas vazio → **Add Object** → **Model Solid**. Nasce «Model» (com «Sphere»
   dentro) e «Light» na Hierarchy, o botão do topo diz **Edit Mode**, a esfera aparece e à direita
   o painel **Model 3D**. Errado: diz Object Mode, ou a esfera não aparece.
3. No painel, **+ Add shape… (A)** → uma forma → ela entra na mesma peça.
4. Na Hierarchy, clicar a «Sphere» e depois a forma nova (com Ctrl para as duas) → continua em
   **Edit Mode**. Errado: volta a Object Mode.
5. **+** da Hierarchy → **Model Solid** outra vez → nasce «Model 2», já em Edit, só com a esfera
   dele no ecrã. Errado: aparece «já tem um Model», ou as formas do 1.º.
6. Em Edit, clicar «Model» (o 1.º) na Hierarchy → não troca, com o aviso *«Leave Edit Mode (Tab)…»*.
7. **Tab** → Object Mode: o painel Model 3D fecha e a peça some do canvas. **Tab** outra vez → volta.
8. Seleccionar «Model» e **Tab** → Edit no 1.º, com as formas dele (e o «Model 2» intocado).
9. A barra de cima já não tem **MODEL**, e o menu **Window** já não tem *Model 3D*.
10. Tab até Object, seleccionar uma imagem (Ctrl+N) e clicar a aba **Model** lá de cima → arruma os
    painéis e fica em Object — não aparece peça nenhuma.

## §7 — O que fica para a próxima janela

- **F3 Vector** (a partição do `DrawMode`; o `ObjectMode::Edit` já existe e o `parts` pode servir os
  nós — a medir).
- Model em Object: a peça não se desenha no canvas 2D, e o gizmo do objecto move um `Transform` que
  o traçado não lê (D9: o 3D como camada entre camadas).
- A câmera do módulo é UMA: trocar de peça enquadra a nova (não guarda a vista de cada peça).
- `Ctrl+Tab`; Image ▸ Mask; F4 (a barra MOVE/ROT/SCALE continua visível nos modos de criação).

## §8 — Perfil do loop (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (10% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                196   alvo: <= 800
  ✗ cargo test : cargo check                520 : 221   alvo: <= 1,0  razao 2.4x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%  (783 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         364 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## §9 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
(preenchido no commit seguinte)
```
