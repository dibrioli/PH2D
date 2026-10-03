# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-03 — os modos de edição por objecto (spec/06 F2 + a F3 da Imagem)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md)
> e [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md):
> os commits da escala e do menu Add continuam descritos lá. Este cobre os 4 commits DEPOIS do
> `f491f37b1` e refaz o gate sobre o diff ACUMULADO desde a base.
>
> **Ordem do dono (03/10):** a F2 do spec/06 junta com a F3 da Imagem, nesta linha, sem integrar;
> o desenho foi mostrado antes de codar e aprovado (*«faça como vc sugere»*), incluindo o **Tab**.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **33 `+`** com este commit, nenhum integrado. Rebase: no-op
  (o `main` não andou).

## §0 — Para o `CLAUDE.md` §5.1 (UI/UX)

Trocar o link do «Último:» por este ficheiro. A frase do módulo não muda.

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `004a8c683` | o modo do objecto activo, o seletor *Mode*, o cadeado, Tab/Ctrl+Space, o botão direito; Image ▸ Paint abre o Painter |
| `c50f43b3c` | os gates de fora aprendem o `CanvasOwner::Mode`; `PH2D_OBJECT_MODE_SMOKE=1\|2` |
| `c5cee174a` | os 5 vermelhos do gate batched: o quadro desce para a fundação (`mode_drive`), a shell volta para baixo do tecto |
| `1fe49861b` | o censo de NodeId segue o código movido; a ordem dos mods do vetor (fmt, da F1); o spec/06 |

**O desenho** (spec/06 §4 F2, já escrito lá com os desvios):
- `ph2d_editor_core::object_mode` — `ObjectMode {Object, Paint}`, `ModeState` (a ENTIDADE, nunca o
  tipo; `last` por entidade para o Tab), `ModeRequest {Enter, Toggle, Open}`, as leis puras
  `resolve`/`still_holds`/`decide`, `refusal`, `active_of(selection, extras)` (= o ÚLTIMO
  acrescentado, o activo do Blender) e `TOOLS_OPENED_BY_A_MODE = [("painter", Paint)]`.
- `ph2d_editor_core::screens::hero::mode_drive` — `ModeFamily { modes, holds, enter, leave }`,
  `drive(families, kind_of, name_of, …)` (publica os modos do activo · a rede `still_holds` ·
  o pedido · o seletor), `refused` (o cadeado numa porta), `right_click_on_canvas`.
- `ph2d_app_painter::paint_mode::FAMILY` (Image ▸ Paint: põe/larga o Painter;
  `holds_an_image` distingue a tela da peça 3D) e `smoke_step`.
- `ph2d_app_components::object_mode::drive` — o tipo e o nome pelo marcador (`kind_of`).
- Shell: `render_loop/fase_object_mode.rs` = só `MODE_FAMILIES` + a chamada.
- O seletor é o 1.º pulldown da área: `interaction::AreaMenus` (o seletor à frente, os do módulo
  atrás — cada escritor reescreve a sua parte, a ordem de escrita no quadro não importa).

## §2 — Foundational tocado (aditivo) e contratos

- `ph2d-editor-core`: módulos NOVOS `object_mode` e `screens/hero/mode_drive` (+ `pub mod` em
  `lib.rs`/`hero.rs`); chrome NOVO `object_mode_menu.rs` (z=46, lista regenerada pelo
  `ph2d-chrome-sync`); `tool_activation::press_the_active_pill` (MOVIDO da shell).
  `hero.rs` **699/700** · `interaction/state/mod.rs` **696/700** (o campo mudou de TIPO, sem linha) ·
  `left_rail.rs` **693/700** · `action_bus.rs` **650**.
- **Dependências novas:** `ph2d-editor-core → ph2d-component-desc` (vocabulário, zero deps) e
  `ph2d-app-painter → ph2d-component-desc`. `Cargo.lock` regenerado pelo cargo.
- Contratos congelados (§6): **nenhum encostado** (o `Tool=12` intocado: o downcast do Painter usa o
  `as_any_mut` que já existia).

## §3 — Superfície de colisão (para o integrador)

**Falha ALTO noutra linha (não compila):**
- `ph2d_app_painter::painter_lock` **APAGADO** (`decide`/`Decision`/`REFUSAL`/`locked_entity`
  → `ph2d_editor_core::object_mode`; `collapse_to_last` → o `mode_drive` colapsa ao entrar).
- `CanvasOwner` ganhou `Mode(ObjectMode)`: todo `match` exaustivo noutra linha parte (aqui foram 4).
- `EditorAction::ObjectMode(ModeRequest)` — o enum é `#[non_exhaustive]`, mas um `match` interno
  exaustivo da fundação partiria.
- `GizmoStateGroup` ganhou o campo `mode` (`Default`): um literal de struct noutra linha parte.
- `WidgetStore::area_menus` mudou de `Vec<AreaMenu>` para `AreaMenus` (privado ao módulo).

**Funde LIMPO e REPROVA depois:**
- `the_shell_only_shrinks`: a shell desta linha está SOB o tecto agora; outra linha que cresça a
  shell soma (o tecto soma entre linhas).
- `node_id_collisions::FORMAS_NAO_LITERAIS`: a entrada `fase_image_tools_mode_and_pills` SAIU e
  entrou `tool_activation.rs :: press_the_active_pill`.
- Gates que exigiam o pill do Painter na barra IMG foram reescritos (`every_image_tool_*`):
  uma linha que acrescente um gate de «todo pill de `image_tools`» tem de saltar o
  `TOOLS_OPENED_BY_A_MODE`.
- Quem pede `ActivateTool { tool_id: "painter" }` (hoje só a aba do Painter ao lado da escultura e
  os smokes do Painter) já não colapsa a selecção.

**Enums, ids e chaves (append-only):**
- Ids NOVOS: `OBJECT_MODE_OBJECT = hash_node_id("object_mode.row.object")`,
  `OBJECT_MODE_PAINT = hash_node_id("object_mode.row.paint")` (`ids/chrome/rail.rs`).
- i18n NOVAS: `object_mode.*` (8, tabela `ph2d-i18n/src/object_mode.rs`). ⛔ **APAGADAS:**
  `app.painter.painter_lock.leave_the_painter_to_select_another_sprite` e
  `shell.fase_image_tool_activation.painter_kept_the_last`. Mudou o texto de
  `shell.init_subsystems.press_1_brush_2_move_3` (`Tab=Mode, Ctrl+Space=Zen`).
- Env: `PH2D_OBJECT_MODE_SMOKE=1|2`.

**Muda comportamento:**
- **Tab** pede o modo (Object ↔ o último); o **zen** passou a **Ctrl+Space** (consumido sempre,
  para não cair no play da linha do tempo). O Tab do modo Node do vetor (andar pelos nós) corre
  antes e continua dele.
- O botão **Painter** saiu da barra IMG; pintar uma imagem é *Mode ▸ Paint Mode* (ou Tab). Os
  outros utilitários de imagem ficam no IMG.
- A aba **Draw** já não pega o Painter: pede `Open(Paint)` — entra se o activo for imagem, senão
  Object com a ferramenta de omissão.
- O **cadeado** é do modo: em Object nada é recusado; em Paint, as 3 portas recusam com
  *«Leave Paint Mode (Tab) to select another object»*.
- **Botão direito** no canvas livre, em Object, depois de todos os reivindicantes → menu Add.
- Selecção trocada por outra porta (criar, duplicar, apagar, desfazer) em Paint → volta a Object
  e o Painter é largado.

**Tectos e a shell:** esta onda **+105 −97** na `shells/desktop` (líquido **+8**); acumulado desde a
base **+684 −322**. A catraca `the_shell_only_shrinks` está **verde**: a lógica do modo vive na
fundação e nas famílias, e saíram da shell o acender dos pills (→ `tool_activation`) e o colapso do
Painter pelo barramento.

## §4 — Fecho: gate batched, mutação, auditoria

**Gate batched (1× sobre o diff acumulado desde `1ad60a1ce`, verificador; load 9–32 por outra
sessão):**
- 1.ª corrida: `nextest-impacted` 19 668, **5 vermelhos**, todos desta onda e determinísticos
  (o DAG da fundação, a catraca da shell, o censo `set_*_mode`, os 2 gates do pill do Painter) →
  curados em `c5cee174a`.
- 2.ª corrida: **19 674**, 1 vermelho (o censo de NodeId pedia o nome do código movido) e o
  `fmt --check` a acusar a ordem dos mods do vetor (da F1) → curados em `1fe49861b`, re-corridos
  sozinhos: **verdes**; `cargo fmt --all -- --check` limpo.
- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` · `clippy --all-targets -D
  warnings` · `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh`:
  **verdes** nas duas corridas.

**Gates novos (30):** `object_mode` 13 · `mode_drive` 9 (com uma família FALSA — o lado
independente) · `paint_mode` 4 (o Painter real: as duas imagens, Tab, o vazio, a tela da peça 3D) ·
`the_mode_selector_answers_a_real_click` 2 (com todos os painéis) ·
`the_painter_is_reached_by_the_paint_mode_of_an_image` 1 · o braço `Mode` em 3 gates de fora + 1
de `task_layout_tests`.

**Mutação (agente `mutacao`, controlo VERDE e âncora 1× antes de cada uma): 11 de 11 sangraram.**

| M | mutação | gate que sangrou |
|---|---|---|
| M1 | `decide` sem a recusa do aditivo | `the_lock_refuses_…` · `the_lock_door_refuses_and_says_why` |
| M2 | `publish` sem o `Object` à frente | `the_selector_faces_…` · `the_selector_follows_the_active_type` |
| M3 | `still_holds` sem `extras == 0` | `the_mode_holds_only_its_own_entity` |
| M4 | o `drive` sem a rede `still_holds` | `another_door_changing_the_selection_returns_to_object` |
| M5 | o `drive` sem colapsar ao entrar | `entering_takes_only_the_active` · `painting_one_image_leaves_the_other_untouched` |
| M6 | `right_click_on_canvas` sem olhar o modo | `the_right_click_adds_only_in_object_mode` |
| M7 | a linha do seletor sem empurrar o pedido | `the_chip_opens_the_selector_and_a_row_asks_for_the_mode` |
| M8 | `holds_an_image` sem `!on_screen_canvas` | `the_painter_on_the_sculpt_screen_is_not_the_image_mode` |
| M9 | a barra IMG sem o filtro do modo | `every_image_tool_has_a_target_in_the_painted_frame` |
| M10 | `rail_shows_painter_tools` sem o termo Paint | `the_painter_is_reached_by_the_paint_mode_of_an_image` |
| M11 | `AreaMenus::set_leading` sem tirar o velho | `the_mode_menu_leads_whatever_the_write_order` |

**Auditoria (DIRETIVA §3):**

```
LENTE:  wiring (o seletor até ao Painter sobre a entidade)
CLAIM:  escolher Paint Mode numa imagem põe o Painter em mãos sobre ELA, e Object larga-o
TRAÇO:  mode_drive::drive (publish → menu → store.publish_mode_menu) → tool_bar.rs bar_rail
        (area_menus, slot 0) → clique no chip: tool_bar_overflow::apply (AreaCommands{0}) → clique
        na linha: object_mode_menu::apply → bus ObjectMode(Enter(Paint)) → fase_bus_tool_panel
        (pd.object_mode_request) → fase_hero_frame → fase_object_mode → components::object_mode::drive
        → mode_drive::drive (resolve → replace_selection(activo) → FAMILY.enter → set_active painter)
ASSERÇÃO-VERMELHA: the_chip_opens_the_selector_and_a_row_asks_for_the_mode (rota, com painéis) +
        painting_one_image_leaves_the_other_untouched (o Painter real) + tab_goes_there_and_back
NÃO-CHECADO-PELA-COMPILAÇÃO: a tecla Tab e o botão direito chegam pelo `App` (placa) — as duas
        portas são provadas pelo pedido que produzem (mode_drive) e pela FOTO; o despacho de teclado
        em si não tem gate
LOC LIDAS: ~1 400
```

```
LENTE:  correção (o cadeado)
CLAIM:  em Paint, nenhuma porta troca o objecto em edição; o que muda por outra porta devolve a Object
TRAÇO:  despacho_clique_pick · despacho_clique_largar · fase_hierarchy_select_lock → mode_drive::refused
        → object_mode::decide; as ~60 portas programáticas → mode_drive::drive passo 2 (still_holds)
ASSERÇÃO-VERMELHA: the_lock_door_refuses_and_says_why · the_mode_holds_only_its_own_entity ·
        another_door_changing_the_selection_returns_to_object (M1, M3, M4 sangraram)
NÃO-CHECADO-PELA-COMPILAÇÃO: entre a porta programática e o quadro seguinte passa UM quadro com a
        selecção nova e o Painter ainda em mãos; o `drive` corre antes das pontes do Painter, logo
        o Painter não chega a ligar-se à entidade nova (ordem em fase_hero_frame.rs)
LOC LIDAS: ~600
```

## §5 — Premissas do briefing que a medição derrubou

- *«O cadeado precisa de UMA porta para 69 chamadas a `replace_selection(`»* — ele JÁ existia
  para o Painter, com uma lei pura e 3 portas de gesto; as outras chamadas são programáticas. A
  cura foi trocar a pergunta das 3 portas (Painter em mãos → modo) e pôr a rede `still_holds`.
- *«Abrir o Painter = o toggle de módulo»* — eram **três** caminhos: IMG + pill, a aba Draw (só com o
  IMG ligado), e a aba do Painter ao lado do Sculpt (a pintura da peça 3D, que fica para a F3 dela).
- O plano punha o `ModoActivo` no `HeroScreen`; vive no `GizmoStateGroup` (ao lado da selecção),
  sem tocar no `hero.rs` além do `pub mod mode_drive`.
- O 1.º desenho (o quadro na shell) estourou a catraca em **+386** e fechou um ciclo na fundação
  (`action_bus → object_mode → screens → action_bus`): o quadro desceu para `screens::hero`, e
  `object_mode` ficou sem conhecer o Hero.

## §6 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. **Ctrl+N** → **Create** → nasce uma imagem, seleccionada. No topo do canvas, depois de
   UNDO/REDO, aparece o botão **Object Mode**. Errado: o botão não aparece.
3. Clicar **Object Mode** → abre a lista com **Object Mode** e **Paint Mode** → clicar **Paint Mode**
   → a barra troca para as ferramentas de pintura (BRUSH, ERASE…) e abre o painel do Painter;
   pintar na imagem. Errado: nada muda, ou não pinta.
4. Em Paint, clicar noutra imagem (ou noutra linha da Hierarchy) → não troca, e aparece
   *«Leave Paint Mode (Tab) to select another object»*.
5. **Tab** → volta a **Object Mode** (a barra volta ao mover/rodar); **Tab** outra vez → Paint de novo.
6. **+** → **Empty** (seleccionado) → **Tab** → aviso *«… has only Object Mode»*; a lista do botão
   só tem **Object Mode**.
7. **Ctrl+Espaço** → modo zen (painéis escondidos); de novo → voltam.
8. Botão direito no canvas vazio, em Object → abre a janela **Add Object**.
9. Com a imagem seleccionada, clicar a aba de cima **Draw** → entra em Paint. Com o Empty
   seleccionado, **Draw** fica em Object.
10. O botão **IMG** já não tem o Painter; os outros (remover fundo, margem…) continuam lá.

Fotos (tela virtual, `fotografa_cena.sh`): `PH2D_OBJECT_MODE_SMOKE=1` (o seletor aberto sobre uma
imagem nova) e `=2` (em Paint, com a coluna de pintura e o painel do Painter) — conferidas.

## §7 — O que fica para a próxima janela

- **F3 Sculpt** (o próximo pela medição): *Sculpt ▸ Sculpt · Paint* — a `ModeFamily` da escultura;
  a pintura da peça 3D deixa a aba do Painter + IMG e vira o Paint do Sculpt (aí `Paint` é
  declarado por DOIS tipos, e o `mode_drive` já procura a família por `(tipo, modo)`).
- Depois Flip (escreve sempre no 1.º desenho, `autokey.rs:54`) → Model (um por cena) → Vector (a
  partição do `DrawMode`).
- ⏳ `Ctrl+Tab` (a lista de modos); Image ▸ **Mask** (D6) quando o módulo existir; a F4 (o campo
  *«modo ao abrir»* para as outras abas).
- Um gate do despacho de teclado (Tab/Ctrl+Space) pede o `App`; hoje só a foto o vê.

## §8 — Perfil do loop (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                182   alvo: <= 800
  ✗ cargo test : cargo check                488 : 215   alvo: <= 1,0  razao 2.3x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  37%   alvo: >= 80%
  ✗ contexto relido por passo (media)         485 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil
```

## §9 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
(colado no fim)
```
