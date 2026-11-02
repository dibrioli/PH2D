# HANDOFF de continuação — GEF 21, metade 2: o som como asset do projeto (2026-11-02)

Linha `line/GameEngineFuncional` · worktree `Worktrees/line-GameEngineFuncional` · base main `3df5894489` ·
HEAD antes deste handoff `e8f38eae8e`. **A linha continua; NÃO está integrada.** Anterior:
[GEF 21 metade 1](HANDOFF_CONTINUACAO_2026-11-02_21_MOEDAS.md). Plano/estado:
[`21_moedas_plano.md`](../21_moedas_plano.md) (tabela actualizada em `1da2f0054d`).
`git log 3df5894489..HEAD` mostra GEF 20 + GEF 21 (ambas metades).

## Commits desta metade

| Commit | O quê |
|---|---|
| `b13f56c6ab` | feat E2: som asset do projeto + rebobinar + achados de arranjo (lista de testes no corpo do commit) |
| `1da2f0054d` | docs: estado do plano |
| `3fdb3fb3a0` | test: duas mutações sobreviventes (`is_sensor=false`, «não guardar a falha») passam a sangrar |
| `e8f38eae8e` | fix: a queda num campo solta o teclado (`HeroScreen::solta_o_teclado`) |

## Feito — NÃO reconstruir

| Peça | Onde / lei |
|---|---|
| Biblioteca | `ph2d_ecs::ProjectSounds` (`crates/ph2d-ecs/src/audio_2d_library.rs`): nome → bytes, id blake3, blob `PH2DSONS` v1, nome único `stem N.ext`, mesmos bytes = mesmo nome |
| Componente | `AudioSource2D.sound` = NOME de um som da biblioteca (forma igual, sem degrau de componente) |
| Toque | `SceneAudio` toca bytes da biblioteca, cache por conteúdo (`crates/ph2d-app-audio/src/scene.rs`) |
| Import/persistência | `ph2d_app_components::sound_project`: importar com descodificação prévia; save/open importando caminhos antigos (≤213); `import_batch` (Import… / queda de ficheiros); `import_for_scene` = porta `ph2d_app_game::smoke::Portas::importa_som` |
| Cartão Audio Source | saiu da shell → `crates/ph2d-app-components/src/audio_inspector_card.rs` (`build_audio_info`/`apply_audio_edit`); `inspector_audio.rs` MOVIDO. Campo Sound aceita nome, caminho (importa) ou queda |
| Schema | `ProjectFile.sounds` no fim; `PROJECT_SCHEMA` 213→214 (`PROJECT_SCHEMA_SEM_SONS = 213`); `falta_no_fim` refeito como contagem de degraus; braços 189..192 com seis zeros; a tripla NÃO vê |
| Assets | `AssetKind::Sound` · `AssetRef::Sound{asset}` · `DragPayload::Sound{asset}`; ícone Volume2; «Select users» lista as fontes que nomeiam o som; chip «Sound» (fileira de chips parte-se acima de 6); `ASSET_KIND_FILTERS` 6→7 |
| Queda | `asset_drop`: 4.º campo receptor `DropTarget::InspectorSound` via `field_target` (`ph2d_panel_inspector::audio_sound_slot_pick`) → `DropAction::SetSound`, aplicado em `object_drag::write_code`; som no texto de script → WriteCode (o nome); testes cortados para `asset_drop_tests.rs` |
| Import router | `Importables.sounds`, filtro «Sounds» |
| Cena das moedas | importa `docs/GameEngineFuncional/arte/sons/moeda.ogg`; a moeda nomeia `moeda.ogg` (`NOME_DO_SOM`) |
| Placar «Coins: N» | caminho do artista (Text + Label fonte Counter `coins`, prefixo «Coins: »); gate `correr_pelo_chao…` afirma «Coins: 2» por `ph2d_ecs::hud::texto`. A cena não o finge |

### Rebobinar (era ABERTO 3 da metade 1)

Causa de «[rebobinar] 1 estado reposto» por quadro parado: a CÂMERA de jogo (com o jogo parado a vista recebe
tempo e re-nasce a cada quadro), não o contador — medido com sonda. `rewind_runtime_state` só conta o que muda
(`repoe`); a câmera sai sempre (cola a vista ao alvo arrastado) e só conta se saiu da mira do nascimento
(`mira_de_nascer`). Foto: 0 linhas (eram 17). Gate `um_mundo_ja_no_inicio_nao_conta_reposicao_nenhuma`.

### Achados de arranjo no fecho (`ph2d-panel-registry-init`)

Chips cortados → fileira parte-se · Target da câmera (GEF 20) fora da coluna → caixa à largura das irmãs,
Eyedropper DENTRO da ponta direita (registado depois, ganha o clique) · `CARGA_DE_COMANDOS` inspector 100→101
(capacidade nova: o Eyedropper); `o_conta_gotas_do_alvo_arma_e_desarma` reescrito para a lei nova.

## Smoke do dono, 1.ª ronda (02/11)

| Relato | Estado |
|---|---|
| (a) «o som não entrou em Sound ao arrastar» | Hipótese medida no código: o dono apagou o texto (campo focado) e `write_text` (`crates/ph2d-panel-inspector/src/sync_sections.rs:165`) salta o campo focado. Cura `e8f38eae8e`: `HeroScreen::solta_o_teclado` nos três campos de queda (Sound, Target, objecto do Script); gate `soltar_o_teclado_tira_o_foco_do_campo`. ⚠️ **NÃO confirmado no app** (tela virtual não recebe cliques). Se falhar, perguntar se o contorno ficou verde |
| (b) «Não há ferramenta Text; a Hierarchy não deixa criar objetos Vector» | Add ▸ Vector só arma o pincel (`ph2d-app-vec/src/object_add.rs`, `vector_mode.rs:54`, `fase_object_add.rs:~104`); o objecto nasce no 1.º traço. Text (`DrawMode::Text`, `crates/ph2d-tool-vector/src/params_mode.rs:35`, atalho T) só existe dentro de um Vector. **DECISÃO DO DONO:** entrada **Add ▸ Text** própria, que cria o objecto de texto já em modo Text. Memória `project-memory/project_add_text_entry_owner_decision_2026_11_02.md` — ⚠️ no checkout principal, por comitar lá; o integrador a recolhe (o `MEMORY.md` está em 18,6 KB, acima do alvo de 17 KB) |

## Superfície de colisão (merge-base `3df5894489`; soma a da GEF 20 e da metade 1)

`ph2d-ecs` (`audio_2d*`, `lib.rs`, `rewind_runtime.rs`, `tests/it/rewind_runtime.rs`) · `ph2d-app-audio` (`scene`,
`audio_2d`, testes) · `ph2d-app-components` (`Cargo.toml` +`ph2d-audio-decode`, `sound_project*`, `audio_inspector*`,
`asset_drop*`, `asset_card_verbs`, `asset_catalog_verbs`, `asset_index_build`, `audio_2d_smoke`, `object_drag`,
`lib.rs`) · `ph2d-app-game` (`smoke.rs` `Portas::importa_som`, `smoke_moedas.rs`) · `ph2d-asset-index` ·
`ph2d-editor-core` (`drag_payload`, `hero.rs` `solta_o_teclado`) · `ph2d-i18n` (5 ficheiros) ·
`ph2d-panel-asset-browser` (ids/paint/state/event/card) · `ph2d-panel-inspector` (`camera.rs`, `sections/audio.rs`,
`state_components`, `lib`, teste da câmera) · `ph2d-panel-registry-init` (catraca 100→101) · shell (`project*.rs`,
`import_router*`, `asset_drag_wire`, `janela_do_jogo`, `project_catalogs`,
`render_loop/{mod,snapshots_inspector,fase_inspector_commits*}.rs`) · `shells/game-scenes` (`comp.rs`,
`jogo_moedas.rs`) · `Cargo.lock`.

### Contadores (delta/tectos)

| Item | Antes → depois / tecto |
|---|---|
| `PROJECT_SCHEMA` | 213 → 214 (reconta-se contra o main: `python3 scripts/schema-recount.py`) |
| `ASSET_KIND_FILTERS` | 6 → 7 |
| `CARGA_DE_COMANDOS` inspector | 100 → 101 |
| Catraca shell | 188 948 / 189 041 |
| `ph2d-app-game` | 2 728 / 2 728 — **NO TECTO: mover antes de crescer** |
| `ph2d-app-components` | 80 599 / 80 869 |

## O que um merge pode partir

- `smoke::Portas` ganhou `importa_som` (quem a constrói na shell / em `game-scenes` colide).
- `SceneAudio::play` ganhou o 5.º arg `sons`.
- `DragPayload` / `AssetRef` / `AssetKind` / `DropTarget` / `DropAction` ganharam variantes: `match` exaustivo noutras linhas parte.
- `ASSET_KIND_FILTERS` 6→7 (quem indexa/afirma o 6 reconfere).
- `PROJECT_SCHEMA` 214: reconta-se contra o main.
- `rewind_runtime_state` conta só mudanças: quem afirmava contagens exactas reconfere.

## Prova de fecho

- fmt + `clippy -D warnings` nas crates tocadas (ecs, app-audio, app-components, app-game, asset-index, editor-core,
  i18n, panel-asset-browser, panel-inspector, panel-registry-init, host-desktop, host-game-scenes).
- Testes por nome verdes (lista no commit `b13f56c6ab`); crates inteiras com motivo «registo de painel»:
  panel-inspector 547 · panel-asset-browser 58 · panel-registry-init 158 · i18n 24 · asset-index 28.
- Mutação, lote de 14: 12 sangraram de início; `is_sensor=false` e «não guardar a falha» sobreviviam → gates
  reforçados (`3fdb3fb3a0`) e verificados vermelhos.
- Binário smoke compilado 2× (2.º `Finished smoke in 0.65s`). **Suíte completa NÃO correu** (só na integração).

## Smoke (comando inteiro)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=4 ./target/smoke/ph2d-host-desktop`
2. Assets: o som `moeda.ogg` aparece com ícone de alto-falante; filtro «Sound». Arraste-o para o campo **Sound** do
   Audio Source de uma moeda (apague o texto antes, de propósito: é o caso (a)).
3. **Run ▸ Play in a Window**: cada moeda tocada faz o «plim» e some.
   Errado se: o campo Sound não aceita a queda · sem som · salvar e reabrir perde o som.

## ABERTO, por ordem

1. Confirmar no smoke a queda no Sound (`e8f38eae8e`).
2. **Add ▸ Text** (decisão do dono): família Vector; cria o `VecShape::Text` pela porta da ferramenta, entra em
   Draw ▸ Text, filho do HUD se ele estiver seleccionado; depois o dono monta o placar e o smoke o fotografa.
3. Arrumar um som numa gaveta (pede a família no `SavedAssignment` do `ProjectState`) e tirar um som do projeto
   (pede lápide no `ProjectState`, como a imagem) — hoje recusam com voz.
4. Herdadas da GEF 20: conta-gotas só apanha sprites; perf do `LayerCompositor` (19/4).
5. J6 inimigos (precisa do `SignalTagFilter`). Integrar/shipar só por ordem do dono.
