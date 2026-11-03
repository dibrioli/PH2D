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
| E2 som como asset do projecto | feito `b13f56c6ab`: `ph2d_ecs::ProjectSounds` (nome → bytes, blob `PH2DSONS`), `AudioSource2D.sound` = o NOME; `ProjectFile.sounds` (`PROJECT_SCHEMA` 213→214, o load importa os caminhos antigos); Import…/queda de ficheiros → Assets (chip «Sound»); 4.º campo receptor `InspectorSound` (gate `o_campo_sound_recebe_um_som_e_so_um_som`); a cena importa o `moeda.ogg` (`o_som_da_moeda_e_um_som_do_projeto`) |
| texto do placar | **caminho do artista** (Text dentro do HUD + *Label* fonte *Counter* `coins`, prefixo «Coins: »): o *Label* existe para qualquer objecto (`ph2d-component-desc` `hud.rs`, `O::ANY`) e soma o contador pelo nome; gate `correr_pelo_chao…` afirma `«Coins: 2»` pela porta `ph2d_ecs::hud::texto`. A cena NÃO o finge (decisão do dono 02/11: montar com as peças); o dono monta-o no smoke. ⚠️ Cliques não fotografáveis (a tela virtual não recebe eventos sintéticos) |
| `[rebobinar] 1 estado reposto` a cada quadro parado | feito `b13f56c6ab`: medido com sonda — era a **câmera de jogo** (com o jogo parado a vista recebe tempo e re-nasce a cada quadro), não o contador. A porta só conta o que MUDOU (`repoe`); a câmera sai sempre (é o que cola a vista ao alvo arrastado) e só conta se saiu da mira do nascimento. Foto: `0` linhas (eram `17`). Gate `um_mundo_ja_no_inicio_nao_conta_reposicao_nenhuma` |
| achados no fecho (gates de arranjo) | a fileira de chips dos Assets cortava os nomes com o 7.º chip → parte-se acima de 6; o Target da câmera (GEF 20) saía da coluna da secção → caixa à largura das irmãs, Eyedropper dentro da ponta direita (Inspector `100 → 101` comandos) |
| Add ▸ Text + reseed do Sound | feito `b3ac769b15` (crate `ph2d-app-text`; o texto nasce escrito em Draw ▸ Text, filho do HUD seleccionado) e `2f2e2af52c` (o som largado no Sound aparece sem re-seleccionar; confirmado no app 02/11); auditoria `8b9cba1cda` |
| placar «Coins: N» montado pelo dono | **ADIADO por decisão dele** (02/11: «serão necessários muitos ajustes em HUD, Label, Text») — próxima onda, depois da integração |
| ainda aberto | arrumar um som numa gaveta de catálogo (pede a família no `SavedAssignment`, outro degrau) · tirar um som do projeto (pede lápide no `ProjectState`, como a imagem) — os dois recusam com voz |
