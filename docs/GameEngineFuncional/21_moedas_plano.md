# GEF 21 — J5 moedas: montadas à mão com as peças (plano)

> Item J5 de [`03_plano_jogo_de_plataforma.md`](03_plano_jogo_de_plataforma.md). Decisão do dono (02/11):
> **a moeda é montada com as peças genéricas, nunca um objecto pronto** — *«o objetivo é revisar e melhorar
> a engine testando em um game real»* (`project-memory/project_game_smoke_composes_pieces_never_ready_made_2026_11_02.md`).
> Sons: **Kenney CC0** (pacote *Digital Audio*, `License.txt` lido 02/11).

## A composição (o que o artista faz no editor)

| Objecto | Como nasce | Peças |
|---|---|---|
| `Coin 1..5` | Add ▸ Animated Sprite (já) | + **Collider** (círculo, *Trigger* ligado) · **Signal On Hit** `coin` · **Audio Source 2D** (o som da moeda) · **Signal Actions**: em `coin` → *Play Sound* (Self) · *Add to Counter* 1 (Named `Score`) · *Destroy* (Self) |
| `Score` | Add ▸ HUD | rótulo com fonte *Counter* `coins` · **Counter** `coins` (início 0) |

Sai o `moeda.luau` (distância medida à mão) da cena 2/3/4; o nível 1 fica com ele (é o exemplo de script).

## O que a composição revelou (lido no código 02/11, antes de construir)

| # | Costura | Estado lido | Onda |
|---|---|---|---|
| E1 | Collider *Trigger* num objecto **sem corpo** (Animated Sprite) | a ponte larga-o em silêncio (`ph2d-physics-ecs/src/bridge/parts.rs` `owner_body` → `continue`) | **curar**: um trigger sem corpo é o próprio sensor fixo (a *Area2D* do Godot) |
| E2 | `AudioSource2D.sound` | caminho cru, `std::fs::read` relativo ao cwd (`ph2d-app-audio/src/scene.rs:192`); o índice de assets não conhece áudio | **curar**: o som é asset do projecto por referência, como a imagem (dono 26/10), com queda de Assets no campo (porta `asset_drop::field_target`) |
| E3 | *Play Sound* + *Destroy* no mesmo objecto, mesmo quadro | por medir | medir; o som de um disparo tem de sobreviver ao dono |
| E4 | Stop devolve o contador a 0 · o placar aparece em Play (janela e ecrã cheio) | por medir (`CounterRuntime` não grava; o HUD cola-se à câmera do jogo) | medir |
| E5 | uma moeda conta UMA vez | por medir (Destroy só no dreno do fim do quadro) | medir |

*Destroy* de um objecto do documento já é permitido sobre a CÓPIA do Play (`removable_in_run`) — não é defeito.

## Portões

- `shells/game-scenes/tests/it/jogo_moedas.rs`: o nível 4 montado SÓ pelo menu e por componentes (zero Luau
  nas moedas); a corrida do condutor apanha Coin 1 e 2 → contador 2, dois pedidos de som, as duas saem; a
  Coin 3 fica; Stop → contador 0 e as cinco de volta.
- Um teste por cura na crate dona (E1 na ponte da física, E2 no áudio/assets, E3 no áudio).
- Provas de mutação sobre E1, E2, E3 e a contagem.
- Foto `PH2D_GAME_SMOKE_FOTO=moedas`: o placar «Coins: 2» na janela de jogo.

## Estado

| Item | Estado |
|---|---|
| plano | feito |
| E1 trigger sem corpo | **não é defeito do artista**: a paleta só oferece *Physics Body* (traz o Collision Shape); Static + Trigger funciona |
| E3 som que sobrevive ao dono | feito `f08629069a` (gate `in_a_run_a_one_shot_outlives_its_object_and_a_loop_does_not`) |
| composição das moedas + HUD com Counter | feito `6494f86cd2` (`smoke_moedas.rs`, gates `jogo_moedas` 2; mutação *From: Anyone* sangra) |
| E4/E5 no app | medido 02/11 (`PH2D_SIGNAL_LOG=1`, condutor `window`): 3 moedas, contador 1→2→3, vozes vivas depois da saída; Stop devolve o documento ao bit e o «rebobinar» repõe o contador |
| E2 som como asset do projecto | **aberto** — metade 2 da onda (degrau de `PROJECT_SCHEMA`, Assets, queda no campo Sound) |
| texto do placar pela cena | **aberto**: o texto vetorial só nasce pela shell (`VecState`), em 3 quadros; o caminho do artista (Text + *Label*) já existe |
| `[rebobinar] 1 estado reposto` a cada quadro parado | aberto (só com `PH2D_SIGNAL_LOG`): conta um contador que já está no início como reposto |
