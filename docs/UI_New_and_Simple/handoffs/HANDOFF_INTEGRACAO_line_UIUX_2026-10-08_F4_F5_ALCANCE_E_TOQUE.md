# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-08 — F4 (o alcance) e F5 (o toque) da NOVA UI DO VECTOR

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Tarefa: [`vetor_nova_ui/03_plano.md`](../vetor_nova_ui/03_plano.md) §4 (F4, F5); prompt:
> [`04_prompt_F4_F5.md`](../vetor_nova_ui/04_prompt_F4_F5.md). F0–F3: o
> [handoff de 07/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md) (citado, não reescrito).

- **Branch** `line/UIUX` (worktree `Worktrees/line-UIUX`) · **base (merge-base = `main`)** `2a925859d`
  · F0–F3 (11 commits, `f6217180a..1746cb0d5`) continuam por integrar e vão JUNTOS com esta jornada.
- Commits desta jornada: `cc01e2272..HEAD` — inclui a retirada da roda (`61e5ce36c`) depois do smoke.

## §0 — Para o `CLAUDE.md` §5.1 (≤ 700 bytes)

UI/UX: «Último:» → este ficheiro; `PH2D_OBJECT_MODE_SMOKE=1|4|6|7|8|9` → `…|9|10`.

## §1 — O que a linha entregou

| peça | o que é | onde mora |
|---|---|---|
| ⛔ **RETIRADO: o menu de roda** | a roda inteira (o `P` fora do vetor, de 2026-08-23, e o `J` + o leque por famílias do vetor desta jornada) **saiu do app por ordem do dono** depois do smoke (08/10: *«não gostei desse menu… ele mistura ferramentas de forma inadequadas… melhor eliminar ele»*), e com ela a **poeira de UI** (as faíscas), cujo ÚNICO gatilho era escolher na roda — sem ela seria código morto. O som de clique fica (outros gestos o armam). Registo: `00_DECISOES_DO_ENIO.md` D10.2 | apagados: `widget/{radial_menu,radial_fan}.rs`, `screens/hero/radial*.rs`, `interaction/state/radial_ops.rs`, `motion_burst*.rs`; shell `radial_input*.rs`, `ui_burst_paint.rs`, `render_loop/fase_ui_burst_paint.rs` |
| **F4.2 paleta que ensina** | o `Ctrl+K` projecta as barras do módulo em grupos com o NOME DO LUGAR (*Left bar* = ① + flyout · *Top bar* = ② + um sub por pulldown · *Next to the selection* = ④); o mesmo id, a mesma porta (`apply_event(Click)`); amostras abrem o seletor (`open_picker_on`), campos numéricos ficam FORA; escolher faz **PISCAR onde o comando mora** — o botão, o pulldown que o contém, ou o `⋯` que guarda um dos dois | `screens/hero/global_palette_bars.rs`, `flash.rs`, `WidgetStore::{flash, flashing, flash_pill, note_clock}` |
| **F5.3 modificadores** | +1 dedo = Shift, +2 = Shift+Alt enquanto o primeiro arrasta; widget **Shift · Alt · Ctrl** no canto de baixo do lado da mão livre (segurar = segurar, tocar = travar; travado SEMPRE à vista; o rato também trava); modificadores efectivos = teclado OU toque | `interaction/touch.rs`, `screens/hero/touch_mods.rs`, `chrome/touch_mods.rs`, shell `touch_input.rs` |
| **F5.4 gestos** | toque de 2/3/4 dedos = desfazer/refazer/foco; tocar e segurar = menu de contexto (clique direito no ponto). Portas visíveis: *Edit ▸ Undo/Redo* (já existiam), **_View ▸ Focus mode · Ctrl+Space_** (nova), o widget, o próprio menu | idem + `EditorAction::FocusMode`, `MENUBAR_VIEW_FOCUS` |
| **F5.6 tablet** | ③ **recolhido = faixa de ícones FLUTUANTE** abaixo da janela de referência (iPad 11" e mini), dentro da área do lado da coluna (0 px de layout, como a ①; a alça de 6 px fica); cada ícone é a ABA do painel — o clique reabre a coluna com ele à frente (`slot_tabs::apply_event` → `dock_reopen::reopen_for`), com a dica do nome; **Left-handed** (era *Mirror UI*, mesmo interruptor) leva também a ① e o widget para a direita e grava-se no `prefs.txt`; **_View ▸ Tool options at the bottom_** (por layout) põe a ② por baixo da área; o `⋯` e os pulldowns sobem pelo encaixe ao ecrã que JÁ existia (`clamp_menu_to_viewport` — a mutação mostrou que a 1.ª redacção, que os ancorava à mão, era redundante, e saiu), e o flyout de grupo do Painter (posto à mão) sobe pela lei nova | `dock_reopen::{strip_icons, paint_strip_icons, reopen_for}`, `HeroLayout::{compact, move_tool_bar_to_bottom}`, `tool_bars::placed(.., left_handed)`, `prefs.rs`, `layout_persist.rs` |
| **cena 10** | a 9 (layout Vetor, rectângulo + elipse, Caneta) e depois a paleta a escolher *Snap ▸ ao caminho* — o pulldown pisca até 8 s depois | `ph2d-app-vec/src/vector_mode_reach.rs` |

## §2 — Medido e decidido (técnico, delegado)

- (A roda por famílias — 6 sectores + a porta, leque com raio derivado da diagonal do rótulo — foi
  construída, medida e provada por mutação, e RETIRADA pelo dono no smoke: ver §1. ⛔ Não reconstruir
  sem ler o D10.2.)
- **O piscar: 3 períodos do token `Slow` (400 ms)**, cada um acende de golpe e esmorece (3 acesas, acaba
  apagado) — o tecto de três da WCAG 2.3.1, a 1,25 Hz; com movimento reduzido, anel fixo pelo mesmo
  tempo. Relógio: `dispatch_tick` (todo quadro) → `note_clock`.
- **Números do toque = os do UIKit** (o iPad é o alvo): `minimumPressDuration 0,5 s`,
  `allowableMovement 10 pt`. O 1.º dedo só vira `Down` ao andar além da folga (o `Down` sai na ORIGEM)
  ou ao levantar (um toque) — um 2.º dedo antes disso faz do contacto um GESTO e nenhum traço nasce.
- **«Compacto» = mais estreito que a janela de REFERÊNCIA dos tokens (`HERO_VIEWPORT_W` = 1366, o iPad
  12,9")**: o iPad 11 (1194) e o mini (1133). O app não sabe que é um iPad; sabe a largura.
- ⛔ **A faixa NÃO reserva área e NÃO recolhe sozinha — medido e refeito no gate.** A 1.ª redacção
  reservava 32 px por coluna fechada e recolhia o ③ no primeiro quadro: o piso do ecrã com as colunas
  fechadas caiu de `88,7 %` para **`86,0 %`** no iPad 11 (`collapsing_both_columns_still_gives_the_tablet_back`,
  a recusa de 31/08 com número) e a coluna pintada no iPad 11 sumiu (`a_coluna_pintada_encolhe_com_a_janela`,
  lei de 09/20). ⇒ a faixa flutua (o princípio do D10: «sem pagar faixa nova»), e «③ recolhida em
  faixa» lê-se *quando o artista a recolhe, é uma faixa*.
- **Canhoto = UM interruptor** (o *Mirror UI* que já trocava as colunas), com o nome que o canhoto
  procura (o *Right-hand interface* do Procreate confunde — R4).

### ⛔ O que a FOTO apanhou e nenhum gate via (os dois têm gate agora, provado vermelho contra a lei antiga)

1. **Uma linha-interruptor troca de id ao ser executada** (`VECTOR_SNAP_PATH_ON ↔ _OFF`): no quadro
   seguinte o id escolhido não estava em lado nenhum e nada piscava. Cura: o pulldown que contém o
   comando lê-se NA ESCOLHA (`flash_pill`). O gate passou a republicar as barras depois da escolha e a
   aplicar os interruptores do Snap como a ponte do vetor; com a lei antiga: *«Snap to shapes foi
   escolhido e nada pintado pisca onde ele mora»*.
2. **A porta da moldura (`stroke_frame`) no tema moderno ignora cor e intensidade** — o estado piscava
   e a tela não. O anel é um realce passageiro, não a moldura do botão: traço directo com
   `FRAME-RAW-OK`. Gate `the_ring_paints_in_every_theme`; com a porta antiga: *«Dark: o anel do piscar
   não desenhou nada»*.
3. (da cena, não do produto) o 1.º pulldown da área é o seletor de MODO — escolher «a 1.ª linha» tirava
   o vetor da mão. A cena escolhe o Snap pelo nome.

## §3 — ⛔ QuickShape: PARADO (CLAUDE.md §0.9)

É uma LEI (reconhecer linha/arco/elipse/polígono num traço) ⇒ só com oráculo CORRIDO. Triagem por
artefacto instalado (`pacman -Qi`, 08/10): Xournal++ e Rnote (têm reconhecedor) **não estão
instalados**; OpenToonz (BSD-3) só tem a ferramenta *Geometric* por arrasto, não reconhece; Krita,
Inkscape, Pencil2D, MyPaint (GPL) sem reconhecedor encontrado; Godot/libmypaint sem a função. **O que
o dono decide:** instalar o Xournal++ (GPL-2, corre-se como caixa preta — o `_ComoInvestigarApps` o
permite, só a saída é gravada) ou outro. Comportamento-alvo já registado em R4 L165–169 (Procreate) e
L205 (Concepts).

## §4 — Foundational tocado e contratos

- **`ph2d-editor-core`** — ⛔ **APAGADOS** (ALTO para toda linha que os use): `widget::{RadialItem,
  RadialOpen, paint_radial_menu, radial_*}`, `screens::hero::radial` (`RADIAL_MORE`, `fit`),
  `WidgetStore::{open_radial, close_radial, radial_point, radial}`, `motion_burst` (`BurstField`) — e
  na shell `App::ui_burst` e `radial_input`. **Novos**: `ToolBarsSlot` +4 campos (`clock_ns`, `flash`, `touch`, `row_bottom`) e as portas em
  `tool_bars_ops.rs`; `interaction::touch` (novo); `ids`: `TOUCH_MOD_{SHIFT,ALT,CTRL}`,
  `MENUBAR_VIEW_FOCUS`, `MENUBAR_VIEW_TOOL_ROW_BOTTOM`; `ModuleTruth::ToolRowBottom`;
  `EditorAction::FocusMode` (o enum é `#[non_exhaustive]`); o `HeroScreen::new` passou a
  `ViewState::default()` (idêntico, −7 linhas no tecto); `HeroLayout::{compact, move_tool_bar_to_bottom}`;
  `dock_columns::reopens`; `dock_reopen::{strip_icons, paint_strip_icons, reopen_for}` e
  **`slot_tabs::apply_event`**: o clique numa aba de um painel de coluna recolhida reabre-a (ALTO para
  quem mexa nesse braço); `tool_bars::placed` ganhou o 5.º
  parâmetro `left_handed` (ALTO para chamadores noutra linha); chrome `touch_mods` (z=47, `dispatch_all`
  regenerado pelo `ph2d-chrome-sync`: 42 handlers); `left_rail` regista os 3 botões do widget.
- **`ph2d-i18n`**: `chrome_reach.rs` (novo, na cadeia); `chrome.menu.mirror_ui` = «— Left-handed»;
  **apagadas** `chrome.radial.more` e `shell.input_handlers.no_tools_to_show_here`.
- **Contratos congelados (§6): nenhum.** `Tool=12`/`PanelEvent=4` intocados (o toque são cliques e
  modificadores pelas portas que já existiam).

## §5 — Superfície de colisão e contadores (DELTA sobre `2a925859d`, somando F0–F3)

- `MODULE_TRUTHS` 18 → **19** · `OWN_ROWS` 9 → **11** (F0–F3 tinham levado 17→18 e 8→9).
- `BarFamily` inalterado · ícones: 0 novos nesta jornada · o `App` perde 1 campo (`ui_burst`).
- Cenas `PH2D_OBJECT_MODE_SMOKE`: **+10**. `PROJECT_SCHEMA`: inalterado (`layout.txt`/`prefs.txt` são por
  máquina).
- **`layout.txt` +1 campo por layout** (`tool_row_bottom`) · **`prefs.txt` +1** (`left_handed`) —
  ficheiros antigos lêem-se iguais (chave ausente = destro, fila em cima).
- **Em silêncio:** outra linha que pinte por cima do canto de baixo da área (o widget) ou da borda da
  área do lado do ③ recolhido num tablet estreito (a faixa); um menu novo aberto por um chip da fila ② que
  não pergunte `tool_row_bottom` abre para baixo do ecrã com a fila embaixo.
- Shell: `touch_input.rs` (novo; o teclado num `thread_local` — o `App` só larga campos),
  `input_handlers` (o braço do `P` SAIU; o `Ctrl+Space` pela `toggle_focus`), `keyboard_tail` (a
  soltura do `P` saiu), `despacho_mover`, `render_loop/{mod, fase_chrome_clock}`, `main.rs` (arm `Touch`, tick, `mod`; um
  comentário condensado para o tecto), `despacho_metodos_janela_e_vetor` (modificadores efectivos),
  `fase_bus_chrome` (`FocusMode`), `prefs`, `forwarding_persist`, `init`, `layout_persist` (+ testes, e os
  gates `the_mode_keys_are_wired`, o do menu, `the_highlight_has_one_source` (o som do pie menu saiu da
  lista) e `a_tecla_do_gatilho…` (o controlo do canvas passa do `P` ao `M`); `the_ui_burst_is_wired`
  apagado com a feature). **Delta da shell (linhas, `numstat`): esta jornada `src +224 −30` (net **+194**, com ~25 de teste em
  `src`) · `tests +8 −2`; somado a F0–F3 sobre a base: `src` net **+240**, `tests` net +97.** Dentro da
  folga medida a 07/10 (3 367); o grosso é o `touch_input.rs` (a ponte do toque) — candidato a descer
  para a crate se outra família a usar.

## §6 — Gate final

Três corridas do `verificador` sobre o diff acumulado (a 1.ª e a 2.ª reprovaram — 14 e depois 2 + um
lint, todos meus e curados acima; a 3.ª é o atestado), `load` 30–110 com outras linhas a compilar:
- fmt ✓ · warnings-deny sobre o workspace inteiro com todos os alvos ✓ · o lint como o `ship.sh`
  (`--features ph2d-spike/bevy_ecs -- -D warnings`) ✓ · `machete` ✓ · standalone-optional ✓ ·
  workflow-packages ✓ · censos da árvore 12/12 (114/114) · `doc-index --check` ✓ · tectos de LOC e
  catracas da shell 12/12 (`the_shell_only_shrinks`, `file_loc_caps`, `fn_loc_caps`,
  `the_app_only_sheds_fields`).
- `nextest-impacted` (`BASE=2a925859d`, `NO_FAIL_FAST=1`): **16 794 / 16 795** — o vermelho é
  `ph2d-physics-ecs::measure_player_budget::the_cost_of_a_player_is_linear_in_their_number` (relógio,
  sem diff nesta linha), **3/3 verde sozinho** com `load 72–84` ⇒ flake de carga (FLAKES_DE_CARGA.md).
- **4.ª corrida, depois da retirada da roda (D10.2):** tudo ✓ menos duas catracas que a remoção deixou
  OBSOLETAS (o gate exige que desçam): `TETO_CAMPOS` 174 → **173** (`ui_burst` saiu) e
  `FN_OVERAGE_OK` `main.rs::new` 213 → **212** — baixadas e verdes; `nextest-impacted` 16 767 / 16 769,
  os 2 vermelhos eram essas duas.
- ⚠️ O que só a árvore combinada pega: quem use a roda ou a poeira noutra linha, ou chame
  `tool_bars::placed` (ALTO); `MODULE_TRUTHS`/`OWN_ROWS`/`OUTSIDE_WINDOW_TOGGLES` somam com qualquer linha
  que acrescente linhas ao menu *View*.

### Mutação (agente `mutacao`, verde visto antes de cada uma, árvore limpa no fim)

**24 + 1 mutações; 21 sangraram à primeira** (M1–M5 eram da roda, que saiu). Os três sobreviventes e o que se fez:
- **M14** (tirar `!moved` do tocar-e-segurar): a cláusula era MORTA (um dedo único que anda além da
  folga já é traço — o `committed` cobre) ⇒ saiu.
- **M16** (o 2.º dedo nunca vira gesto): sobreviveu no teste dos dedos extra e sangrou no dos toques —
  sangra no total.
- **M23** (o menu da fila embaixo ancorado no chip): o gate media só a 1.ª linha; passou a medir o menu
  INTEIRO e a mutação continuou viva — porque o encaixe ao ecrã já fazia o trabalho. A âncora à mão
  era redundante e saiu; o flyout do Painter (sem encaixe) ganhou gate próprio, que sangra
  (`y = 1042` fora do ecrã) com a lei desligada.
- Fora do agente, à mão: o gate do piscar com republicação (a lei antiga reprova) e o da tinta do anel
  (a porta da moldura reprova).

## §7 — Smoke do dono

Ver o relatório ao dono (passos numerados). Foto: [`f4_02`](../vetor_nova_ui/fotos/f4_02_cena10_a_paleta_faz_piscar_o_snap.png) (o *Snap ✓* com o anel).

## §8 — O que fica ABERTO

- **QuickShape** — §3, decisão do dono (instalar um oráculo).
- **Os gestos de dedo pedem um ecrã de toque** para o smoke: provados por gate (o reconhecedor e a
  ponte), não vistos por um dedo real. Pan/pinça de 2 dedos (R4 #11) não estava no escopo.

## §9 — Binário de smoke (2.ª corrida)

Depois do `rm -rf target/*/incremental` (47 GB) — `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida:

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.27s
```

## §10 — `agent-loop-profile.sh`

```
  ✗ paralelismo de ferramenta              1.15/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                239   alvo: <= 800
  ✗ teste : verificação                     409 : 243   alvo: <= 1,0  razao 1.7x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  32%   alvo: >= 80%
  ✗ contexto relido por passo (media)         486 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```
⚠️ As edições foram quase todas por `python3` com `assert` de contagem (vários sítios por passo); a
régua conta-as contra o alvo do `Edit`.
