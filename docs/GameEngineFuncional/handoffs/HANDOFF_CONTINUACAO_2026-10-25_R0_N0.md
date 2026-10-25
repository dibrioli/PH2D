# Handoff de continuação — `line/GameEngineFuncional`, R0 + N0 (25/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base `c3662cd8e4` (merge-base com o main). Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-25.md`](HANDOFF_CONTINUACAO_2026-10-25.md) (W1/W2). Plano:
[`04_R0_N0_plano.md`](../04_R0_N0_plano.md) · [`03`](../03_plano_jogo_de_plataforma.md).

## Commits da onda (`git log --oneline main..HEAD`)

| sha | O quê |
|---|---|
| `9d515913ad` | R0a — o Play corre sobre uma cópia; o Stop devolve o documento ao bit |
| `620a723832` | N0 — ciclo de vida do script (ready, physics, process, exit, emissor do sinal, `ph2d.connect`) |
| `bc3bf2ea58` | R0b/R0c — menu Run (janela / ecrã cheio / no editor), janela do jogo, cena J1 |
| `76281fba3d` | moedas — `RunOnCopy`: sobre a cópia o jogo pode tirar objectos do artista |
| `832e0c0e93` | R0b — o palco do jogo (só o mundo; câmara do editor volta; gravar a meio grava o documento) |
| `dcdce09680` | N0 — `process(dt)` recebe o tempo de parede do quadro |
| `bb3a62cda4` | a shell encolhe (ponte do Timer e gates do menu de presets descem) |
| `2d8cdb59b9` | fecho da onda — tectos da shell, censos, avisos no palco |

## R0a — Play sobre cópia

- Lei em `crates/ph2d-app-game` (`Sessao<S>`, `Passo::{Nada,Fotografa,Repoe}`). Sessão e janela moram em
  `ProjectUndo` (`shells/desktop/src/undo.rs`: `jogo` / `janela_do_jogo`); código em
  `shells/desktop/src/undo_jogo.rs` (+`undo_jogo_tests.rs`) e `janela_do_jogo.rs`.
- Subida (`TimelineState::runtime`): `capture_project` ANTES dos motores do passo fixo.
  `take_runtime` subiu do dreno da timeline para o início de `fase_frame_simulation`; gate
  `o_play_de_cima_chega_a_timeline_antes_do_relogio_da_vista`.
- Descida: `apply_project_with` com a selecção. Undo suspenso a correr (motivo «o jogo esta a correr»);
  Ctrl+Z recusado com aviso. Ctrl+S a meio grava a fotografia (`App::documento_do_jogo`).
- Gate: `undo::jogo::tests::o_stop_devolve_o_documento_ao_bit`. Mutação: a saída que não repõe → vermelho.

## RunOnCopy (moedas)

`crates/ph2d-ecs/src/lifetime.rs` (`removable_in_run`). Com fotografia, `ph2d.despawn`, `Destroy` dos sinais
e as mortes da física removem objectos do artista (o Stop devolve-os). Sem cópia (transporte da timeline) a
recusa antiga fica. `ph2d_app_game::marca_a_copia` (em `sessao.rs`) é chamado em `fase_frame_open` depois de
`jogo_quadro`. Testes: `smoke::tests::a_moeda_some_quando_o_jogador_lhe_chega` e
`sem_copia_a_moeda_do_artista_fica_e_a_consola_diz_porque`.

## R0b/R0c — onde jogar

- Menu Run: Play·F5, Pause·F5, Stop·Shift+F5, Play in a Window / Play Fullscreen / Play in the Editor.
  `GameStage` em `crates/ph2d-editor-core/src/game_stage.rs`; ids `MENUBAR_RUN_WINDOW/FULLSCREEN/EDITOR`
  (`ids/menubar.rs`); marca por `ModuleTruth::GameStage`; `OWN_ROWS` 12→15, `MODULE_TRUTHS` 19→22,
  `OUTSIDE_WINDOW_TOGGLES` 6→9; seam test `chrome/menu_bar_run_tests.rs`. Escolher = jogar; padrão = Window;
  F5 fora da corrida entra nela.
- Palco: `ViewState::palco_do_jogo` — `frame_layout` dá a janela inteira ao canvas; `paint_hero_screen` não
  pinta chrome nem auxiliares (sem alvos no `hit_index`); a shell sai cedo nas fases de auxiliares (física,
  canvas, selecção) e não pinta avisos (`App::no_palco`); câmara do jogo ligada; `VistaAntes` guarda a
  `Camera2d` do editor e repõe.
- Janela: 2.ª janela winit (1280×720 lógicos, «PH2D · Game»), surface do MESMO `GpuContext`;
  janela/host/surface trocam com os do editor 1× ao abrir e 1× ao fechar; o quadro desenha-se uma vez, lá; o
  editor fica com a última imagem; fechar a janela do jogo = Stop; eventos da outra janela só sabem fechar
  (`App::da_outra_janela`).

## N0 — ciclo de vida (`crates/ph2d-script/src/scene_life.rs`)

- Ordem do passo fixo: exits pendentes → todos os `init` → `ready` → por objecto `physics` depois `update` →
  esperas → exits dos despawn.
- `process(dt)` 1×/quadro com o tempo de parede, depois da pausa no golpe
  (`FrameClocks.wall_dt` → `Relogio::com_quadro`).
- `exit` 1×/vida: despawn, morte/troca de script, Stop (`ScriptHost::scene_stop`, escritas deitadas fora).
- `on_signal(self, nome, de)`; `ph2d.connect(obj, nome, fn)` — só esse emissor, morre no `exit`.
- `ph2d.d.luau` regenerado (`cargo run -p ph2d-bindgen -- --write`).
- 9 testes `scene::api_tests::life` + `script_bridge` `tres_tiques_num_quadro_sao_um_process`; 16 mutações
  vermelhas.

## Cena de smoke `PH2D_GAME_SMOKE=1` (`crates/ph2d-app-game/src/smoke.rs`)

Chão com vão, 2 plataformas, paredes, meta verde, PlatformPlayer (setas/A-D, ↑/Z salta), câmara a seguir,
5 moedas com `moeda.luau`. Nasce PARADA. Condutor `PH2D_GAME_SMOKE_DRIVE=window|fullscreen|editor` (joga ao
quadro 60, seta direita 120–240, pára ao 420). Log: «[jogo] Play: documento <hash>» e «[jogo] Stop: … igual
ao do Play: true». `fotografa_cena.sh` ganhou `FOTO_JANELA_A_CADA=1`.

## Medido

- Hash do documento `c768d248a6970738` antes do Play e depois do Stop: igual nas 3 variantes, 4 rodadas.
- Ecrã antes × depois do Stop: 0,45 % de pixéis diferentes (só os avisos). Hierarquia 14 → 12 objectos a
  jogar (moedas apanhadas) → 14 depois do Stop.
- Shell (catraca `the_shell_only_shrinks`): o main estava no tecto (189 041); a linha chegou a +466. A ponte
  do Timer desceu para `ph2d-app-components` (`timer_tick.rs`, 9 testes) e os testes do menu de presets para
  `ph2d-panel-timeline` (10) → 188 838 antes do fecho (203 abaixo); `main.rs` 1061 → 1034
  (`FILE_OVERAGE_OK` baixado). **Recontar no fecho da linha.**

## O que um merge pode partir

Contadores de `editor-core` (`OWN_ROWS`, `MODULE_TRUTHS`, `OUTSIDE_WINDOW_TOGGLES`) somam entre linhas:
recontar, nunca escolher. A catraca da shell e `FILE_OVERAGE_OK` de `main.rs` também. `ph2d.d.luau` é gerado:
em conflito, regenerar. `take_runtime` mudou de sítio (ordem do quadro).

## Verde e não corrido

Verde: clippy `-D warnings` de 10 crates; testes por nome de todas as crates tocadas. **Suíte não corrida**
(dono 23/10: só antes da integração).

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Run ▸ Play in a Window (ou F5).
3. A janela «PH2D · Game» abre só com o jogo; setas andam, ↑ salta, moedas somem.
4. Shift+F5 ou fechar a janela pára; o editor volta com as cinco moedas e o aviso «Game stopped · the scene
   is back as it was before Play». Run ▸ Play Fullscreen e Play in the Editor são iguais.
5. Errado se: moeda não volta, a vista do editor muda, a janela mostra painéis.

## ABERTO / próximo

1. **R1 — vista dupla ao vivo** (editor e jogo a desenhar ao mesmo tempo): separar a codificação do mundo de
   `fase_hero_frame`; «uma vista do mundo» = câmara + tamanho + alvos.
2. **N1 do plano 03:** objectos do J2/J3 no menu Add (Sprite, Animated Sprite, Static Body, Character,
   Collision Shape) com Inspector e tooltip ilustrado.
3. **View ▸ Focus mode** parece morto na UI nova (não esconde nada): medir e decidir/registar.
4. N0: sem `disconnect`; o `exit` do Stop lê o mundo sem portas; `attach_script` mantém a recusa em
   objectos do artista.
5. Fecho da linha (`/pd-linha-fechar`) NÃO corrido: W1 tinha seam test do Hero real e mutações por fazer.

## Smoke do dono (25/10) e curas

| Report | Mecanismo | Cura | Medido |
|---|---|---|---|
| *Play in the Editor* não enquadra a câmara no canvas visível («áreas ocultas por trás dos painéis») | a câmara do jogo media a vista pela proporção da JANELA e escrevia-a na câmara do editor, que cobre a janela inteira | `ph2d_app_game::encaixe` (`aspecto` do canvas visível `hero.last_canvas`; `na_janela`: escala pela altura do canvas, centro deslocado) em `fase_game_camera.rs`; o palco *Editor* também liga a câmara do jogo | foto: o herói segue dentro do canvas entre os painéis; testes `encaixe::tests` (2) |
| *Play in a Window*: o editor atrás «espelha» a janela | o palco ligava-se no quadro do Play, ANTES de a janela abrir: esse quadro (sem painéis, câmara do jogo) era o último do editor | na janela, `palco_do_jogo` só liga em `abre_a_janela_do_jogo`; falhar a abertura joga no editor (sem tentar a cada quadro, `Sessao::foto_mut`); o título do editor não se escreve na janela do jogo | foto da 1.ª janela do app (`FOTO_ESCOLHA=head`): o editor com os painéis atrás do jogo |
| *Play Fullscreen* | — | — | OK do dono |
