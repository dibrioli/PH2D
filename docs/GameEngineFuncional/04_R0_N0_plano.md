# R0 + N0 — a janela de jogo e o ciclo de vida do objecto (plano, 25/10)

> Onda seguinte ao [plano 03](03_plano_jogo_de_plataforma.md); smoke = **J1**: a cena abre numa janela de
> jogo e fecha de volta ao editor intacto.

## O que JÁ existe (medido antes de desenhar, §5.0)

| Peça | Onde | O que faz hoje |
|---|---|---|
| A «cena a correr» | `ph2d-timeline/src/runtime.rs` (`TimelineState::runtime`), drenada em `render_loop/fase_timeline_drain.rs:36` | o Play de cima (`TransportCmd::Play` → `RunScene`) liga-a; o Reset (`StopScene`) desliga-a; o transporte da timeline também a desliga |
| O relógio | `ph2d-core/src/playhead.rs` + `ph2d-transport` | scripts (`motores.rs:142`), física (`dispatch.rs:114`), fábricas, partículas só andam com ele a tocar |
| F5 | `ph2d-app-components/src/posse_do_jogo.rs` + `shells/desktop/src/input_dispatch/posse_do_jogo.rs` | F5 alterna o relógio (pausa/continua); o «Para» manda `TransportCmd::Reset`; o teclado vai ao jogo com a cena à vista |
| Repor ao parar | `script_bridge::rewind`, `rewind_runtime_state`, `physics rewind_to(0)` | o Reset repõe o ESTADO VIVO (scripts, temporizadores, corpos pela nascença) — **não** o documento |
| O undo | `shells/desktop/src/undo*.rs` (`ProjectState::capture`/`restore`) | ⛔ **uma corrida inteira colapsa num passo de undo** (`post_frame_undo_steps`): o que o jogo criou, apagou ou renomeou FICA no documento depois do Stop |
| Precedente | `prefab_cancel_pending` → `prefab_cancel` (`fase_open_recipe.rs:65`, `prefab_exit.rs:40`) | fotografa o projecto ao abrir a receita e o Cancel repõe-no — o mesmo padrão que o Play precisa |
| Ecrã cheio | `ph2d-editor-core/src/zen.rs` + `zones.rs:100` | com o zen, só a zona central tem área (a janela inteira) |
| Câmara de jogo | `fase_game_camera.rs:77-92` | com `game_camera_preview` a câmara do jogo escreve na câmara do editor |
| Janelas | `init.rs:90-108`, `main.rs:968` | UMA janela; o `window_event` ignora o `WindowId`; o mundo desenha-se em `game_rt`/`world_rt` do tamanho da janela, o vetor e o chrome no mesmo passe Vello |
| Ciclo de vida Luau | `ph2d-script/src/api_table.rs:47-68`, `scene.rs` | só `init`, `update(dt)` (passo fixo), `on_signal(name)`; sem `exit`, sem emissor, `on_signal` vai a todos |
| Catraca da shell | `architecture_the_shell_only_shrinks` | ⛔ o main está **no tecto** (189 041) e a W1 pôs +32: a onda tem de MOVER mais do que acrescenta |

## O desenho

**R0a — o jogo corre sobre uma CÓPIA; o Stop devolve o bit** (o modelo do *Play Mode* do Unity: o que se
muda a jogar não fica). Na **subida** de `runtime` (o quadro em que o Play entra, antes dos motores) o
projecto é fotografado pela MESMA porta do undo e do save (`capture_project`), e um passo pendente é
empurrado antes (nada do artista se perde). Durante a corrida o undo **não regista** (motivo nomeado
«o jogo está a correr»). Na **descida** a fotografia volta pela porta do Ctrl+Z (`apply_project_with`, com
a selecção de antes) e vira o baseline: zero passos, documento ao bit. Pausa (F5) não repõe; só o Stop.
A lei (subida/descida, quem guarda a fotografia) vive numa crate nova, `ph2d-app-game` — a família do
jogo, que vai crescer até ao export (J10).

**R0b — ecrã cheio no canvas.** Jogar «em ecrã cheio» liga o zen e a câmara do jogo enquanto corre, e
esconde os auxiliares do editor (gizmo, selecção, grelha); o Stop repõe a vista como estava.

**R0c — a janela de jogo.** Jogar «numa janela» abre uma 2.ª janela winit com a sua surface; o mundo é
desenhado lá com a câmara do jogo e sem chrome, pelos mesmos passes do canvas (parametrizados por alvo,
tamanho e câmara); fechar essa janela = Stop. O teclado da janela de jogo é do jogo.

**N0 — o ciclo de vida** (nomes simples, à Godot): `init(self)` (nasce) · `ready(self)` (depois de TODOS
os `init` do tique — os outros já existem) · `physics(self, dt)` (passo fixo; `update` fica como está) ·
`process(self, dt)` (cada quadro desenhado) · `exit(self)` (o objecto sai: despawn, morte, troca de
script ou Stop). **Sinais por objecto:** o sinal leva quem o emitiu; `on_signal(self, nome, de)`; e
`ph2d.connect(obj, "nome", fn)` liga uma função a um sinal DE um objecto.

## Gates

- **O bit:** fotografar → jogar (mover, criar, apagar, renomear) → Stop → o hash do `ProjectState` é o de
  antes; e a fila de undo tem o mesmo tamanho. Mutação: tirar a reposição, tirar a supressão.
- **A ordem:** a fotografia é tirada ANTES dos motores do 1.º quadro (gate de ordem no texto do quadro).
- **N0:** cada callback corre no momento certo, em teste de cena (`scene_api_tests_*`), e o gate
  `a_api_e_a_tabela` conta os hooks novos no `.d.luau`.
- **Smoke (`PH2D_GAME_SMOKE=1`):** cena de plataforma (chão, plataformas, um herói com script); o app
  imprime o hash antes do Play e depois do Stop.

## ⛔ Recusas

| Recusado | Porquê |
|---|---|
| Correr o jogo num 2.º `SimWorld` | todos os sistemas (física, scripts, desenho, partículas) leem O mundo; duplicá-los é reescrever o quadro — a fotografia pela porta do undo dá o mesmo contrato (o documento não muda) |
| Repor só o estado vivo (o Reset de hoje) | não desfaz o que o script criou, apagou ou renomeou — é o defeito medido |
| Fotografar no fim do quadro (como o `prefab_cancel`) | o 1.º quadro do jogo já correu `init`/`update` e a fotografia sairia suja |

## Medido no fecho

- Hash do documento `c768d248a6970738` antes do Play e depois do Stop: igual nas 3 variantes
  (`PH2D_GAME_SMOKE_DRIVE=window|fullscreen|editor`), 4 rodadas.
- Ecrã antes × depois do Stop: 0,45 % de pixéis diferentes (só os avisos). Hierarquia 14 → 12 → 14.
- Shell: main no tecto (189 041), linha a +466; depois de descer a ponte do Timer e os gates do menu de
  presets: 188 838; `main.rs` 1061 → 1034.

### ⛔ Recusas MEDIDAS (novas)

| Recusado | Porquê |
|---|---|
| Zen para o palco do jogo | só fala com o layout antigo; View ▸ Focus mode na UI nova não esconde nada (controlo provavelmente morto, por verificar) |
| 2.º `SimWorld` | ver a tabela acima |
| Repor só o estado vivo · fotografar no fim do quadro | ver a tabela acima |
| Desenhar o mundo 2× por quadro já | a codificação do mundo está fundida com o despacho da UI em `fase_hero_frame` — é o R1 |
