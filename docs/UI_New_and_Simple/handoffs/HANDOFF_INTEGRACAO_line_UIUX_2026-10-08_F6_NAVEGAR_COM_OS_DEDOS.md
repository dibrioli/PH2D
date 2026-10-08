# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-08 (3.ª volta) — F6: NAVEGAR COM OS DEDOS

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Tarefa: [`vetor_nova_ui/05_prompt_F6_toque_completo.md`](../vetor_nova_ui/05_prompt_F6_toque_completo.md);
> plano: [`03_plano.md`](../vetor_nova_ui/03_plano.md) §4 F6. F0–F3 e F4–F5: os handoffs de
> [07/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md) e
> [08/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F4_F5_ALCANCE_E_TOQUE.md) (citados, não reescritos).

- **Branch** `line/UIUX` (worktree `Worktrees/line-UIUX`) · **base (merge-base = `main`)** `2a925859d`
  · F0–F5 (36 commits até `b572da64c`) continuam por integrar e vão JUNTOS com esta jornada.
- Commits desta jornada: `2c412194a..HEAD` (o F6 inteiro; o `main` não andou — sem rebase).

## §0 — Para o `CLAUDE.md` §5.1 (≤ 700 bytes) — JÁ APLICADO nesta linha

UI/UX: «Último:» → este ficheiro; smoke `…|9|10` → `…|9|10|11` (e o `2`, que o `paint_mode.rs` lê e
o roteador omitia). A frase: «toque — dois dedos navegam».

## §1 — O que a linha entregou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **F6.1 navegar com os dedos** | **2 dedos que ANDAM = arrastar a vista; pinça = zoom à volta do ponto entre os dedos** (R4 #11). O reconhecedor pede `NavBegin`/`Pan`/`Zoom`/`NavEnd`; a shell leva-os às portas do RATO: pousar/arrastar/levantar o **botão do MEIO** (`on_mouse_input` + `on_cursor_moved`, com o quadro, o grafo e a timeline a reclamar o pan como com o rato) e a **RODA** (`on_mouse_wheel` com `PixelDelta`, pela inversa da lei da roda) — **nunca uma 2.ª câmera** (gate). O zoom à volta dos dedos é a roda (que zooma ao centro) **mais um pan pela porta do pan** (`Camera2d::anchor_pan_px`, exacto por telescopagem mesmo com o zoom em voo) | `interaction/touch.rs` (`Nav`, `navigate`, `feed_synthetic`, `note_scale`, `tick` → `Vec`), `ph2d-render/camera.rs` (`wheel_factor`, `wheel_px_for`, `anchor_pan_px`), shell `touch_input.rs` (`pinch`), `canvas_zoom::goal` |
| **F6.1 a fronteira** | o toque de 2 dedos (desfazer) e o pan partilham o início; a fronteira é a **folga do UIKit** (10 pt): um toque que andou não desfaz, um pan que não andou não move. A navegação arranca SEM salto (o 1.º movimento além da folga semeia a referência) e um dedo a mais/a menos semeia outra vez | gate `two_travelling_fingers_pan_and_pinch_and_never_undo` (os dois lados + o controlo), `a_finger_joining_or_leaving_mid_navigation_does_not_jump` |
| ⭐ **uma vez por QUADRO** | a navegação publica-se no `tick`, nunca por evento: cada dedo chega num evento próprio e publicar a cada um compõe zooms à volta de dois pontos (arrasto espúrio `δ²/4s` por quadro — a 1.ª foto lia 205 % onde a lei dá 202 %) | gate «dois dedos que andam juntos num quadro são um pan PURO» |
| **F6.3 a ponte encolhe o que pode** | `toggle_focus` desce para `ZenMode::toggle_announced` (a porta única do `Ctrl+Space`, *Ver ▸ Modo foco* e 4 dedos); a lei da roda (`0,9^(dy/16)`) sai do despachante para a câmera; a conta do zoom à volta de um ponto é da câmera. O que fica na shell é só porta: fase do winit, px↔pt, `chrome_hit`, os `on_*` | `zen.rs`, `camera.rs`, `touch_input.rs` |
| **cena 11** | a 9 (layout Vetor, rectângulo + elipse, Caneta) e depois dois dedos SINTÉTICOS pousam no meio da área, andam `(240, 60)` px em 1 s e afastam-se `40 → 80` px no segundo seguinte: a vista arrasta e aproxima 2× à volta deles (101 % → 201 %), o widget Shift·Alt·Ctrl acorda. Os contactos entram pelo reconhecedor (`feed_synthetic`, em px, com a escala anotada pela shell) e saem pelo `tick` — a MESMA porta de um dedo real | `ph2d-app-vec/src/vector_mode_nav.rs` (estágios 40–41) |
| ⛔ **ACHADO e curado: o pan sob o split da Motion andava `h·t²`** | a varredura `fbf6a0f30` (09/09, ~90 sítios → janela da cena) trocou também a chamada a `pan_scene_camera`, que já aplica o split por dentro. Um mês sem ninguém ver — apanhado ao fazer a pinça passar por essa porta. Cura: a janela INTEIRA (`gfx.surface.size()`) | gate `the_dispatcher_hands_the_pan_door_the_whole_window` |

## §2 — Medido e decidido (técnico, delegado)

- **Sem rotação da vista**: a `Camera2d` tem `center`, `height_world`, `cull_mask` — nenhum ângulo. O
  gesto de rodar não existe (o prompt mandava medir, não inventar).
- **Sem «quick-pinch encaixa no ecrã»**: não há *enquadrar tudo* / fit / home para a câmera da cena
  (procurado: `fit`, `frame_all`, `enquadr`, `zoom_to_fit`, `reset_view`, `home_view`, `center_view`,
  `NamedKey::Home`). É pergunta ao dono (§8), não invenção.
- **A roda zooma ao CENTRO, não ao cursor** (idioma Blender, como o pan do meio) — ficou assim; a
  pinça é que zooma aos dedos, e a diferença é um pan. Zoom em voo (`canvas_zoom`, mola `Surface`):
  a compensação lê o DESTINO (`goal`) antes e depois do entalhe e aplica o pan com o `h` VIVO — a
  conta `(p−c)·(h'−h)/h_vivo` cancela o `h_vivo` na porta do pan, logo é exacta por telescopagem,
  esteja a mola onde estiver (gate `the_anchor_pan_keeps_the_world_under_the_finger_still`).
- **O quadro (MiroClone) toma a pinça como toma a roda** (`board_view::wheel`, que já zooma ao
  cursor); quando ele toma, o destino da cena não muda e a compensação não corre.
- **A foto confirma a lei** (cena 11, 2.ª foto): zoom 201 % (= 2× 101 %); as formas onde a conta
  `M + 2,02·(p − M)` as põe, a ±15 px — a incerteza é a do centro da área (`last_content`), não
  da lei. ⚠️ A 1.ª foto (205 %, âncora a fugir ~20 px) é a que levou à lei «uma vez por quadro».
- **A folga de 10 pt é a fronteira dos DOIS gestos** — não se acrescentou nenhum número novo.

## §3 — ⛔ QuickShape: continua PARADO (CLAUDE.md §0.9)

Pergunta feita ao dono no início da janela (instalar o Xournal++ — GPL-2, corre-se como caixa preta,
só a saída é gravada), sem resposta até ao fecho. Nada escrito. Comportamento-alvo em R4 L165–169.

## §4 — Foundational tocado e contratos

- **`ph2d-editor-core`** — `interaction::touch`: `TouchOut` ganha `NavBegin([f32;2])`, `Pan([f32;2])`,
  `Zoom { factor, about }`, `NavEnd` (ALTO para quem faça `match` exaustivo — o enum não é
  `#[non_exhaustive]`; hoje só a shell); **`tick` passa a devolver `Vec<TouchOut>`** (era `Option`);
  novos `feed_synthetic`, `note_scale`. `ZenMode::toggle_announced` (novo). `WidgetStore::clock_ns`
  (novo getter).
- **`ph2d-render`** — `Camera2d::{WHEEL_NOTCH_PX, WHEEL_ZOOM_PER_NOTCH, wheel_factor, wheel_px_for,
  anchor_pan_px}` (novos, puros).
- **`ph2d-app-vec`** — `vector_mode_nav` (novo), `vector_mode` lê `"11"`.
- **`ph2d-i18n`**: nada (zero texto novo; os toasts do foco já existiam).
- **Contratos congelados (§6): nenhum.** `Tool=12`/`PanelEvent=4` intocados.

## §5 — Superfície de colisão e contadores (DELTA sobre `2a925859d`, somando F0–F5)

- `MODULE_TRUTHS` 19 · `OWN_ROWS` 11 · `BarFamily` · ícones · `layout.txt`/`prefs.txt`: **inalterados
  nesta jornada** (os degraus de F4–F5 continuam a valer).
- Cenas `PH2D_OBJECT_MODE_SMOKE`: **+11**. `PROJECT_SCHEMA`: inalterado.
- **Em silêncio:** outra linha que chame `TouchGestures::tick` (era `Option`, é `Vec`) ou faça `match`
  em `TouchOut`; outra linha que use `touch_input::toggle_focus` (apagado — é `ZenMode::toggle_announced`);
  quem leia `0.9_f32.powf(dy / 16.0)` do despachante por texto (a lei mudou de casa).
- Shell: `touch_input.rs` (`pinch`, os 4 braços novos), `input_dispatch.rs` (a lei da roda pela
  câmera; a janela INTEIRA na porta do pan), `input_handlers.rs` + `fase_bus_chrome.rs`
  (`toggle_announced`), `canvas_zoom.rs` (`goal`); gates `the_mode_keys_are_wired` (a porta mudou de
  casa), `the_camera_pan_has_exactly_one_door` (+1), **novo** `the_fingers_navigate_by_the_mouse_doors`.
  **Delta da shell (`numstat`): esta jornada `src +85 −28` (net **+57**) · `tests` +~70; somado a F0–F5
  sobre a base: `src` net **−22**.** Shell a 185 603 linhas contra o tecto 189 041 — a ponte do toque é
  porta (`on_*` da `App`) e o que era lei desceu (foco, roda, âncora).

## §6 — Gate final

(a preencher no fecho)

### Mutação (agente `mutacao`, verde visto antes de cada uma, árvore limpa no fim)

(a preencher)

## §7 — Smoke do dono

Ver o relatório ao dono (passos numerados). Foto:
[`f6_01`](../vetor_nova_ui/fotos/f6_01_cena11_dois_dedos_arrastam_e_aproximam.png) — a cena 9 antes:
[`depois_01`](../vetor_nova_ui/fotos/depois_01_cena9_caneta_na_mao_com_duas_formas.png).

## §8 — O que fica ABERTO

- **QuickShape** — §3, decisão do dono (instalar um oráculo).
- **«Quick-pinch encaixa no ecrã»** — não há *enquadrar tudo* no app; se o dono o quiser, é uma
  porta visível (menu *Ver*) + a pinça rápida sobre ela.
- **Um dedo real** — a lei está provada por gate e a cena 11 por foto, mas nenhum dedo de verdade
  tocou um ecrã: o smoke num iPad/ecrã de toque continua por fazer (ver §8 de 08/10).
- **Customizar a barra por arrasto** — fora (2.ª obra, só se pedido).

## §9 — Binário de smoke (2.ª corrida)

(a preencher)

## §10 — `agent-loop-profile.sh`

(a preencher)
