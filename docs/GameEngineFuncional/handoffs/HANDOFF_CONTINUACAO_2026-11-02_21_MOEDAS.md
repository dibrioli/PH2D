# HANDOFF de continuação — GEF 21, metade 1: as moedas montadas com as peças (2026-11-02)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · base main `3df5894489`
(a linha estava a par do main na reabertura; sem rebase) · HEAD antes deste handoff `97ce78ad18`.
**A linha continua; NÃO está integrada.** Anterior:
[GEF 20](HANDOFF_CONTINUACAO_2026-11-02_20_CAMERA_INTUITIVA.md). Plano com a tabela de estado:
[`21_moedas_plano.md`](../21_moedas_plano.md). Smoke do dono da metade 1: **OK em 02/11**.

Decisão do dono (02/11), ficheiro `project-memory/project_game_smoke_composes_pieces_never_ready_made_2026_11_02.md`
(+ uma linha no `project-memory/MEMORY.md`) — ⚠️ **vivem no `project-memory/` do checkout principal, por comitar
lá**; o integrador os recolhe. Lei: a moeda **monta-se à mão com peças genéricas, nunca é um objecto pronto**;
«o objetivo é revisar e melhorar a engine testando em um game real»; sons dos packs Kenney (CC0).

## Commits (os desta metade)

| Commit | O quê |
|---|---|
| `f08629069a` | E3: numa corrida (`RunOnCopy`) a voz de disparo único de uma fonte que saiu da cena ACABA de soar (`SceneAudio.tails`, `forget_absent(eng, vivos, deixa_acabar)`); o laço e o fora-de-corrida (Ctrl+Z) continuam a calar. Ficheiros `crates/ph2d-app-audio/src/{scene.rs,audio_2d.rs,audio_2d_tests.rs}`; gate `in_a_run_a_one_shot_outlives_its_object_and_a_loop_does_not` |
| `6494f86cd2` | J5: moedas do nível 2 (cenas 2/3/4) sem `moeda.luau` + HUD com contador |
| `97ce78ad18` | docs: estado do plano |

`git log 3df5894489..HEAD` mostra 11 commits: os 8 da GEF 20 (ainda não integrados — ver o handoff anterior) e
estes 3.

## Feito — NÃO reconstruir

- **Cada moeda** = Animated Sprite + Physics Body Static com Collider Ball (raio 0,5; escala 0,5) `is_sensor`
  (Trigger) + `SignalOnHit` `"coin"` + `AudioSource2D` (som = caminho `docs/GameEngineFuncional/arte/sons/moeda.ogg`
  = Kenney Digital Audio `powerUp9`, `License_Kenney_Digital_Audio.txt` ao lado) + `SignalActions`
  [`PlaySound self`, `AddToCounter "HUD"` arg 1, `Destroy self`], todas **From: Myself**. O nível 1 mantém o
  `moeda.luau` (é o exemplo de script).
- **HUD** = Add ▸ HUD (`UiCanvas`) com nome `"HUD"` + Counter `"coins"` início 0.
- Código: `crates/ph2d-app-game/src/smoke_moedas.rs` (`veste_moeda`, `placar`).
- `smoke::Portas`: o campo `camera` virou `componentes: &dyn Fn(&mut SimWorld, &str) -> Result<u64, String>`,
  chaveado pelo nome canónico do componente, servido por `ph2d_app_components::object_add::add_by_component`
  (nova). A shell (`janela_do_jogo.rs`) mudou sem alterar o nº de linhas; `shells/game-scenes/tests/it/comp.rs`
  espelha. `shells/game-scenes` ganhou dev-dep `ph2d-tags` (`Cargo.lock`).
- **Achados lidos no código antes de construir (não são defeitos):** E1 — um Collider Trigger SEM corpo é
  descartado em silêncio pela ponte física (`parts.rs` `owner_body → continue`); o artista não o alcança porque a
  paleta só oferece Physics Body (que traz a Collision Shape). `Destroy` de objecto do documento já é permitido na
  cópia do Play (`removable_in_run`). `SignalTagFilter` existe (só sinaliza se o outro pertence a uma tag) e
  **não é usado ainda** — o J6 (inimigos) vai precisar dele para que inimigos não apanhem moedas.
- **Medido no app** (`fotografa_cena.sh`, `PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_DRIVE=window PH2D_SIGNAL_LOG=1`): o
  condutor apanhou 3 moedas, contador 1→2→3; `[audio-2d] 4 fonte(s) · 1 voz(es) viva(s)` depois de uma moeda sair
  (a cauda funciona); `[jogo] Stop … igual ao do Play: true`; moedas de volta; contador reposto pela invariante
  «rebobinar».

## Superfície de colisão (merge-base `3df5894489`)

Este handoff cobre só a metade 1; a superfície total da linha soma a da GEF 20. Tocado nesta metade:
`crates/ph2d-app-audio/src/{scene,audio_2d,audio_2d_tests}.rs`, `crates/ph2d-app-components/src/object_add.rs` (+ teste),
`crates/ph2d-app-game/src/{smoke,smoke_moedas}.rs`, `shells/desktop/src/janela_do_jogo.rs`,
`shells/game-scenes/{Cargo.toml,tests/it/{comp,jogo_moedas}.rs}`, `Cargo.lock` (só dev-dep `ph2d-tags`),
`docs/GameEngineFuncional/{21_moedas_plano.md,arte/sons/*}`.

- **Schemas sem delta** (`PROJECT_SCHEMA 213`). Sem ADR. Nenhum contrato §6 tocado.
- Catraca `the_shell_only_shrinks_and_every_app_crate_too`: `ph2d-app-game` tecto 2 728 chegou a 2 740; **curado** por
  corte de comentários/`println` de `smoke_moedas.rs` e `smoke.rs` (verde). Shell sem crescimento. Tectos de LOC:
  `smoke.rs` ~545, `smoke_moedas.rs` 85.

## O que um merge pode partir

- **`smoke::Portas`**: quem mais use o campo `camera` (ou construa `Portas` na shell / em `game-scenes`) colide —
  agora é `componentes`.
- `ph2d-app-audio` `scene.rs`: `forget_absent` mudou de assinatura (3.º argumento `deixa_acabar`).
- `o_nivel_2_nasce_so_pelo_menu` foi actualizado (moedas são Trigger sem script): outra linha que mexa nas moedas
  do nível 2 reconfere-o.
- Arquivo de som por caminho cru relativo ao cwd: continua assim até o E2 (abaixo).

## Prova de fecho (verde, POR NOME)

- fmt nas crates tocadas; `clippy -D warnings` em `ph2d-app-audio`, `ph2d-app-components`, `ph2d-app-game`,
  `ph2d-host-game-scenes`, `ph2d-host-desktop`.
- Testes: audio 2 nomeados (incl. o gate do E3) · game-scenes `jogo_` 51 (incl.
  `correr_pelo_chao_apanha_as_duas_moedas_do_chao_uma_vez_cada` e `a_moeda_nao_tem_script_e_o_hud_tem_o_contador`
  em `jogo_moedas.rs`) · app-game `smoke` 6 · app-components `object_add` 7 · editor-core `the_shell_only_shrinks` 2.
- Binário do smoke: 2.º `cargo build -p ph2d-host-desktop --profile smoke` → `Finished \`smoke\` profile [optimized]
  target(s) in 0.22s` (zero Compiling).
- Mutação feita: `SignalFrom::Anyone` em `smoke_moedas` → `jogo_moedas` **RED** (as cinco moedas saem).
  **O lote completo NÃO correu** (ver ABERTO 4). A suíte completa NÃO correu (só na integração).

## Smokes (comando inteiro)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Hierarchy → **Coin 1**: o Inspector mostra Physics Body (Trigger), Signal On Hit = `coin`, Audio Source 2D e
   Signal Actions (3 linhas).
3. **Run ▸ Play in a Window**; setas + espaço até as moedas.
4. Cada moeda tocada faz um «plim» ascendente e some — só essa.
5. **Stop**: as 5 moedas voltam.
   Errado se: sem som · um toque remove várias · as moedas não voltam.

## ABERTO — metade 2 da GEF 21 (janela NOVA)

1. **E2 — som como ASSET do projecto, por referência** (como as imagens; decisão do dono 26/10). Hoje
   `AudioSource2D.sound` é caminho cru lido por `std::fs::read` relativo ao cwd (`crates/ph2d-app-audio/src/scene.rs`,
   `sample`); o campo Sound do Inspector é texto + Browse (`shells/desktop/src/render_loop/fase_inspector_commits_audio.rs`,
   `inspector_audio.rs:86` `Path::exists`; modelo `crates/ph2d-editor-core/src/screens/hero/inspector_model_audio.rs`
   `AudioFieldEdit::Sound`). Mapa das portas: partição de import `shells/desktop/src/import_router.rs:77`; `AssetDb`
   `crates/ph2d-asset/src/{db,asset}.rs` (`enum Asset` `non_exhaustive`, sem variante de som); persistência
   `ProjectFile.assets`/`SavedAsset` em `shells/desktop/src/project.rs` + `project_assets.rs` (**degrau de
   `PROJECT_SCHEMA` 213→214, contado contra o main por `python3 scripts/schema-recount.py`**); painel Assets
   `AssetKind`/`AssetRef` em `crates/ph2d-asset-index/src/lib.rs` + `crates/ph2d-panel-asset-browser`; `DragPayload`
   em `crates/ph2d-editor-core/src/interaction/drag_payload.rs`; campo de queda
   `crates/ph2d-app-components/src/asset_drop.rs` (`field_target`/`verdict`/`field_marks`/`resolve`) — o slot Sound
   é o **4.º campo receptor e entra por essa porta** (regra GEF 20b). Precedente de biblioteca por NOME:
   `ph2d_script::ProjectScripts` (`ProjectFile.scripts`, v190) — modelo a considerar.
   ⚠️ A catraca da shell tem 2 linhas de folga e `ph2d-app-game` está no tecto: **MOVER código para fora antes de
   acrescentar**.
2. **Texto do placar pela cena.** Um Text vetorial só ganha entidade pelo sync `VecState` da shell (3 quadros;
   `shells/desktop/src/hud_smoke.rs`). Caminho do artista: ferramenta Text dentro do HUD + componente «Label»
   (fonte Counter `coins`, prefixo «Coins: ») — existe, mas os cliques **não foram verificados nem fotografados**.
   Ou verificar e ensinar no smoke, ou um recurso ECS de pedido drenado pela fase vec (estilo ADR-0075) para a cena
   pedir um texto.
3. `[rebobinar] 1 estado(s) vivo(s) reposto(s)` imprime a cada quadro parado sob `PH2D_SIGNAL_LOG`
   (`shells/desktop/src/render_loop/fase_fabrica_e_morte.rs` ~335): conta um Counter já no início como «reposto».
4. **Lote de mutação** (fim da linha): E3 com `deixa_acabar` ignorado · laço mantido em `tails` · tirar `is_sensor` ·
   `AddToCounter` para outro nome.
5. Abertas herdadas do [GEF 20](HANDOFF_CONTINUACAO_2026-11-02_20_CAMERA_INTUITIVA.md): conta-gotas só apanha
   sprites; gate de placa (verde 02/11); mutações em lote / perf do `LayerCompositor` do 19/4.
6. Próximo do plano de jogo: J6 (inimigos; precisa do `SignalTagFilter`). Integrar/shipar só por ordem do dono.
