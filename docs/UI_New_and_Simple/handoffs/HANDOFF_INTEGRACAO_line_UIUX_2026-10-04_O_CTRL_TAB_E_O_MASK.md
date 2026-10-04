# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-04 — o Ctrl+Tab e o Mask (os três abertos dos modos)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui** os oito
> anteriores desta linha:
> [`…_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md),
> [`…_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md),
> [`…_2026-10-03_OS_MODOS.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md),
> [`…_2026-10-03_O_SCULPT.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md),
> [`…_2026-10-03_O_FLIP.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md),
> [`…_2026-10-03_O_MODEL.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MODEL.md),
> [`…_2026-10-04_O_VETOR.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_VETOR.md) e
> [`…_2026-10-04_O_OBJECTO_VETORIAL.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_OBJECTO_VETORIAL.md).
> Este cobre 4 commits e refaz o gate sobre o diff ACUMULADO desde a base.
>
> **Ordem do dono (04/10): os três abertos, o mais barato primeiro.** Briefing:
> [`HANDOFF_CONTINUACAO_line_UIUX_2026-10-04_OS_TRES_ABERTOS.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-04_OS_TRES_ABERTOS.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **77 `+`** antes do commit deste doc (78 com ele), nenhum
  integrado. Rebase: no-op.

## §0 — Para o `CLAUDE.md` §5.1

- **UI/UX:** trocar o link do «Último:» por este ficheiro. A frase não muda.

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `a8385d0aa` | Ctrl+Tab abre a lista dos modos (spec/06 §3.2) |
| `1d87241aa` | Image ▸ Mask pinta a máscara da camada |
| `f796202ff` | gate reforçado: a troca Paint↔Mask não larga o Painter (a mutação M1 sobrevivia) |
| `94156f3e4` | a cura achada pela FOTO: o Mask morria no quadro seguinte |

### (1) Ctrl+Tab

- Tab e Ctrl+Tab vão a UMA porta, `ph2d_editor_core::screens::hero::mode_drive::mode_key(hero, list)`.
  Sem Ctrl = `ModeRequest::Toggle` como antes; com Ctrl = `apply_event(Click(ids::area_menu_button(0)))`
  — o clique do próprio selector do cabeçalho, logo o 2.º toque fecha (o toggle de
  `tool_bar_overflow`).
- Sem selector (nenhum objecto activo) não faz nada: `WidgetStore::has_mode_selector` e
  `AreaMenus::has_leading` impedem de abrir o pulldown do MÓDULO, que ocupa então o slot 0.
- **Medido antes:** Ctrl+Tab estava livre (os dois braços do Tab — `input_handlers.rs` e o passo de
  nós em `input_dispatch/keyboard_cadeia.rs` — são ambos `if !ctrl`).
- **Curado de passagem:** um chip que TRANSBORDOU para `⋯` não tem rect; abri-lo sem rato (Ctrl+Tab,
  a paleta global) ancorava o menu em `(0, viewport.y)`. Agora ancora sob o chip `⋯`
  (`tool_bar_overflow::apply`, `.or_else(rect_for(TOOL_BAR_OVERFLOW))`). Shell −1 linha.

### (2) Image ▸ Mask

- **Medido antes** (dois exploradores + verificação): o Painter tem DUAS máscaras — a da CAMADA
  (`LayerKind::Mask`, presa ao dono por `parent.mask`, gravada no projecto, preto esconde; a linha
  «Mask» do painel de camadas, `ph2d-panel-painter-layers/src/paint_mask_row.rs`, com olho/Invert/
  Apply) e o pincel de PROTECÇÃO (`ph2d-tool-painter/src/tool/paint/mask.rs`, congela pixels,
  transitório, não gravado). **Escolha do dono (perguntado a 04/10 com a tabela): a máscara da CAMADA.**
- ⛔ **Premissa refutada:** o briefing dizia que `ObjectMode::Mask` já estava no vocabulário — não
  estava (5 modos); foi acrescentado.
- **Desenho:** `ObjectMode::Mask` (rótulo «Mask Mode», linha `OBJECT_MODE_MASK`);
  `ph2d_app_painter::paint_mode::Family` declara (Image, Paint) e (Image, Mask) — o MESMO Painter
  com outro alvo. Mask = a máscara da camada activa (`add_mask_to_active` se falta, branco = tudo
  visível; um passo de undo); Paint = a camada dona (`LayerStack::owner_of_mask`, porta única nova
  que substitui duas cópias à mão em `mutate.rs::apply_mask` e `watercolor_backdrop.rs`).
- **Desvios do desenho (o porquê, medido):**
  - (a) o `leave` da família já NÃO larga o Painter: largar e retomar no mesmo quadro desfaz-lhe a
    tela (0×0) ou deixa um bake diferido. O `follow` larga-o quando não resta nem Paint nem Mask, e
    só quando o Painter está numa IMAGEM (o Paint do Sculpt também é `Paint` e é da família dele).
  - (b) o alvo põe-se no `follow` (`aim`), não no `enter`: ao entrar o Painter ainda não tem a imagem
    (a shell dá-lha mais tarde no quadro) — medido na app real com log temporário: quadro 1 tamanho
    (0,0), do quadro 2 alvo = a máscara, mantido 391 quadros. Estado entre quadros: `thread_local`
    `FOLLOWING` (modo + alvo + `wrong`), como `model_mode::target`.
  - (c) `wants`: escolher a linha «Mask» do painel de camadas em Paint leva o modo a Mask; uma camada
    de cor leva-o de volta a Paint.
  - (d) porta nova no trait: `ModeFamily::refusal() -> Option<String>` (omissão `None`), mostrada
    pelo `drive` como toast de aviso depois do `follow`. Mask sem máscara possível (medido: só em
    `HARD_CAP_LAYERS = 999` — uma camada de ajuste nunca fica alvo de pintura, o Painter repõe a de
    cor) cai em Paint com «No mask could be added to this layer — back to Paint Mode».
  - (e) `match` exaustivos estendidos: `ph2d-tool-flip` `params.rs::tools_of` e `ph2d-app-sculpt3d`
    `sculpt_mode.rs` (enter).
- Cena `PH2D_OBJECT_MODE_SMOKE=7` (entra em Mask na 1.ª imagem; 1..6 estavam usados).

### (3) Model desenhado no canvas 2D em Object — MEDIDO e NÃO construído

Escolha do dono: LINHA NOVA, depois. Números: o trace é CPU (`ph2d_field_render::trace`, 46–57 ms/
quadro a 560², viewport exclusivo em ecrã cheio); o `Transform` da raiz não é lido pelo cook
(`ph2d-field-ecs/src/cook.rs:38–55`); falta a ponte textura-na-GPU, o afim 2D do `Transform` da raiz
e o z-order entre camadas 2D; estimativa ~800–1 200 linhas em 3–5 crates. Registado em spec/06
F3 ▸ Model.

### (4) A cura que a FOTO achou (`94156f3e4`)

O Mask entrava e caía em Object no quadro seguinte: a fase da shell
`render_loop/fase_image_tools_mode_and_pills.rs` largava qualquer ferramenta de imagem salvo
`mode == ObjectMode::Paint`, e `HeroScreen::rail_shows_painter_tools` (`offers.rs`) também perguntava
`== Paint`. Verde em todos os gates; apanhou-o a foto da cena 7. Cura: `ObjectMode::uses_the_painter()`
(Paint · Mask), e os dois leitores passam por ela. Gate de censo NOVO
`crates/ph2d-editor-core/tests/it/a_painter_mode_is_asked_by_one_door.rs`
(`nobody_compares_the_mode_with_paint_by_hand`): nenhuma comparação à mão `==/!= …ObjectMode::Paint`
em `shells/desktop/src` nem `ph2d-editor-core/src` (exclui `object_mode.rs`, `*_tests.rs`, `tests/`);
controlo do filtro com as duas formas que existiam; controlo da varredura > 500 ficheiros; mutação
(repor a linha antiga da shell) ⇒ VERMELHO.
**Candidata a memória:** «a lei de modo escrita para o 1.º irmão (`== Paint`) parte o 2.º (Mask)».

## §2 — Foundational tocado e contratos

- `ph2d-editor-core`: `ObjectMode` (+`Mask`; `ALL` passa a `[_; 6]`; `uses_the_painter`),
  `ids::OBJECT_MODE_MASK`, `ModeFamily::refusal` (método com omissão), `mode_drive::mode_key`,
  `AreaMenus::has_leading`, `WidgetStore::has_mode_selector`, âncora de recurso em
  `tool_bar_overflow`, `offers.rs`. `ph2d-i18n` `object_mode.rs` (+2 chaves: `object_mode.mask`,
  `object_mode.mask_needs_a_layer`). `ph2d-tool-painter` `LayerStack::owner_of_mask`.
- Contratos congelados (§6): **nenhum encostado.**

## §3 — Superfície de colisão

Ficheiros tocados:
- `crates/ph2d-editor-core/src/{object_mode.rs, ids/chrome/rail.rs, interaction/area_menu.rs,
  interaction/state/dock_width_ops.rs, screens/hero/mode_drive.rs,
  screens/hero/chrome/tool_bar_overflow.rs, screens/hero/offers.rs}`
- `crates/ph2d-editor-core/tests/it/{main.rs, a_painter_mode_is_asked_by_one_door.rs}`
- `crates/ph2d-i18n/src/object_mode.rs`
- `crates/ph2d-app-painter/src/{paint_mode.rs, paint_mode_tests.rs}`
- `crates/ph2d-tool-painter/src/{layers/stack.rs, tool/layers/mutate.rs, tool/paint/watercolor_backdrop.rs}`
- `crates/ph2d-tool-flip/src/params.rs`, `crates/ph2d-app-sculpt3d/src/sculpt_mode.rs`
- `crates/ph2d-panel-registry-init/tests/it/the_mode_selector_answers_a_real_click.rs`
- `shells/desktop/src/{input_handlers.rs, render_loop/fase_image_tools_mode_and_pills.rs}`
- `docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`

**Contadores (DELTA):** `ObjectMode::ALL` 5 → 6 (`[_; 6]`); i18n +2 chaves; shell −3 linhas (ver §4).
Sem degrau de `PROJECT_SCHEMA` nem de registos nesta onda.

**O que um merge pode partir:** outra linha que acrescente uma variante de `ObjectMode` ou um
`match` sobre ele precisa do braço novo (`Mask`) — falha ALTO ao compilar; e qualquer leitura do
comprimento de `ObjectMode::ALL`. Uma comparação à mão `== ObjectMode::Paint` que chegue de outra
linha põe o censo `nobody_compares_the_mode_with_paint_by_hand` VERMELHO.

## §4 — Fecho (DIRETRIZ §1.5.9)

**Gate batched** (`verificador`, diff acumulado desde `1ad60a1ce`) em `f796202ff`:
- nextest-impacted **21 112 / 21 113**; o 1 vermelho foi
  `ph2d-tool-painter tool::paint::mask::mask_tests::the_mask_stroke_cost_does_not_follow_the_canvas`
  (razão de relógio: 0,46 ms @1024² vs 1,19 ms @2048²), listado em
  [`FLAKES_DE_CARGA.md`](../../DevOps/FLAKES_DE_CARGA.md) (linha 53); `load` 36→68; passa sozinho
  3/3 a `load` 47.
- `ph2d-panel-registry-init --all-features` 140/140; `CARGO_BUILD_WARNINGS=deny check --workspace
  --all-targets` verde; clippy `--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`
  verde; fmt verde; machete, `check-standalone-optional.sh`, `check-workflow-packages.sh` verdes.
- **Depois da cura da foto**, re-gate em `94156f3e4` (BASE=`f796202ff`): nextest-impacted
  **16 851 / 16 851** (`load` 62→91), check com avisos negados, clippy `-D warnings`, fmt — verdes.

**Catraca da shell `the_shell_only_shrinks`: 196 984 / 196 990 (folga 6).** ⛔ Premissa refutada: o
briefing dizia «196 982, folga 8» em `24770a080`; a contagem real aí era **196 987** (folga 3). Nesta
onda: −1 (`input_handlers`) −2 (`fase_image_tools`) = 196 984.

**Mutação** (agente `mutacao`, controlo verde com população > 0, âncoras 1×):
- Ctrl+Tab **5/5 sangram:** M1 `mode_key` ignora `list`; M2 sem `has_mode_selector`; M3 sem a âncora
  `⋯`; M4 o braço da shell volta a `if !cmd_chord`; M5 `has_leading` sempre true.
- Mask **9/9 sangram** no código final. 1.ª passagem 8/9 — M1 (o `leave` da família larga o Painter
  outra vez) SOBREVIVEU: o gate media o alvo, e largar+retomar devolvia o mesmo alvo; o dano é a
  tela (0×0) ou um bake diferido ⇒ gate reforçado em `f796202ff` (`canvas_size() == (4,4)` e
  `!take_deferred_bake()` depois de Mask→Paint); M1 re-mutado ⇒ VERMELHO.
- Gate de censo de `94156f3e4`: mutação ⇒ VERMELHO.

**Gates (nomes):**
- `ph2d-panel-registry-init`: `ctrl_tab_opens_the_mode_list_and_a_second_closes_it`,
  `ctrl_tab_without_a_selector_opens_nothing`, `the_shell_hands_tab_and_ctrl_tab_to_the_mode_key`
  (lê `input_handlers.rs`), `an_overflowed_selector_opens_under_the_overflow_chip`.
- `ph2d-app-painter`: `mask_paints_the_layer_mask_and_paint_the_layer_without_dropping_the_painter`,
  `choosing_the_mask_row_moves_the_mode`, `mask_waits_for_the_image_before_aiming`,
  `the_image_family_never_drops_the_sculpt_painter`,
  `a_mask_that_cannot_be_added_falls_back_to_paint_and_says_why`.
- `ph2d-editor-core` (censo): `nobody_compares_the_mode_with_paint_by_hand`.

**Auditoria, três lentes:**
- *Ligação do Ctrl+Tab:* winit `KeyCode::Tab` (`input_handlers.rs`, braço sem guarda) →
  `mode_key(hero, cmd_chord)` → `HeroScreen::apply_event(Click(area_menu_button(0)))` →
  `chrome::tool_bar_overflow::apply` → `open_context_menu(AreaCommands{slot:0})`. Com foco de texto os
  acordes de comando passam o portão da entrada de texto (como o Ctrl+K).
- *Ligação do Mask:* linha `OBJECT_MODE_MASK` → bus `ObjectMode(Enter(Mask))` → `fase_object_mode` →
  `ph2d_app_components::object_mode::drive` → `mode_drive::drive` → `Family::enter` (Painter na mão) →
  `follow`/`aim` (alvo = a máscara) → `fase_image_tools_mode_and_pills` já não o larga
  (`uses_the_painter`).
- *NÃO verificado por compilação:* o toque da tecla em si (teclas sintéticas não chegam ao ecrã
  virtual) — provado pelo gate de fonte + o gate do clique, e deixado ao smoke do dono; pintar na
  máscara com o pincel real (o caminho de pintura da linha «Mask» é pré-existente).

**Fotos** (`fotografa_cena.sh`, conferidas): `PH2D_OBJECT_MODE_SMOKE=1` — selector aberto com
Object Mode · Paint Mode · Mask Mode; `=2` — Paint Mode com o Painter (controlo); `=7` — 1.ª foto:
«Object Mode», sem Painter (o defeito do §1(4)); depois de `94156f3e4`: chip «Mask Mode», Painter na mão.

## §5 — Smoke do dono (lista técnica, para o registo)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=1 ./target/smoke/ph2d-host-desktop`
   — cria-se e selecciona-se uma imagem branca, e a lista «Object Mode» abre sob o botão do topo.
2. Ctrl+Tab: a lista fecha; Ctrl+Tab outra vez: abre. Errado: nada acontece, ou alterna o zen/outra coisa.
3. Clicar «Paint Mode». No painel do Painter (à direita), separador «Brush», escolher uma cor forte
   (p. ex. vermelho) em «Color» e pintar um traço na imagem. (A imagem é BRANCA sobre papel branco:
   esconder com preto não mostraria diferença, por isso pinta-se antes um traço de cor.)
4. Ctrl+Tab → clicar «Mask Mode»: o botão do topo diz «Mask Mode». No separador «Layers», sob
   «Layer 1» há uma linha «Mask» realçada. Errado: o botão volta a «Object Mode», ou não há linha Mask.
5. De volta a «Brush», pôr «Color» a preto e pintar por cima do traço vermelho: onde se pinta, o
   vermelho desaparece. Pôr branco e pintar: volta. Errado: aparece um traço preto na imagem.
6. Em «Layers», clicar «Layer 1»: o botão passa a «Paint Mode» (volta a pintar cor). Clicar a linha
   «Mask»: volta a «Mask Mode».
7. Tab: «Object Mode», o Painter fecha. Tab outra vez: volta a «Mask Mode» (o último modo). Errado:
   vai a Paint.
8. Clicar no canvas vazio (nada seleccionado) e Ctrl+Tab: nada abre.

## §6 — O que fica para a próxima janela

- (a) O Model desenhado no 2D em Object — linha nova por escolha do dono (números em §1(3)).
- (b) As layouts F4 (spec/06).
- (c) O separador «Draw» do topo pede Paint; em Mask passa a Paint (por desenho: o separador = Paint).
- (d) Uma recusa de `ModeFamily::refusal` só existe hoje na família do Painter.

## §7 — Perfil do laço do agente

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                227   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                719 : 246   alvo: <= 1,0  razao 2.9x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (1732 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         486 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §8 — Binário de smoke

Último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke` (2.ª corrida).

```
(PREENCHER: a 2.ª corrida)
```
