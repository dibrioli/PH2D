# HANDOFF de integração — `line/GameEngineFuncional`, 08/11 (limpeza §0.11 · editor de animações · Add Object · Add Component · várias folhas · ferramentas na folha visível)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, HEAD `afe704e75c`, **77 commits** sobre `652b506b58` = merge-base = ponta do `main` (conferido 08/11: o `main` não andou; `--ff-only` serve). Nada integrado, nenhum push. **Suíte completa, `nextest-impacted` e `ship.sh` NÃO correram na linha** (regra do dono 23/10): são do integrador. `git diff --shortstat main..HEAD`: 650 files, +26 433 −20 311 (o «menos» é quase todo a limpeza, rodada 0).

⚠️ **O [handoff de 07/11](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-07_ANIMATED_SPRITE.md) (Sprite + Animated Sprite, HEAD `59ad2121e3`) JÁ ESTÁ no `main`, com outros SHAs** (rodada «integ 07/11»: `b7b1b4255f`, `64e95663bb`…). `git merge-base --is-ancestor 59ad2121e3 main` dá NÃO, mas o conteúdo entrou: o `registry_tests.rs` do `main` já traz `118→119→120` do `SpriteAnimStart`/`SpriteCutNotice` e `git log main --grep="GEF 24"` acha os commits. Os 77 daqui começam em `35667a0e70` e nenhum está no `main`.

## 1. Rodadas (mecanismo nos docs linkados)

| # | Rodada | Commits | Smoke do dono |
|---|---|---|---|
| 0 | Aba Scene · Run · tela cheia · pausa · limpeza §0.11 — **nunca teve handoff** | 14: `35667a0e70`, `572d4daf02..dbc607fb7d` | OK 07/11 |
| 1 | Editor de animações onda 1 + auditoria do Animated Sprite | 11: `7f56e2ae30..a5f9e38237` | OK 07/11 (`a5f9e38237`) |
| 2 | Add Object versão B | 4: `68bcc35b68`, `7d175f3d40`, `1052637aad`, `6727c42c71` | OK 07/11 |
| 3 | Add Component (GEF 25) + câmera | 15: `67e7150fba..d54dc3c940` | OK 08/11 |
| 4 | Várias folhas no Animated Sprite (GEF 26) | 23: `41d67017b8..7a21a30615` | OK 08/11 |
| 5 | Ferramentas de imagem na folha visível + fecho | 10: `ba2b76ca0d..afe704e75c` | «Funcionou» 08/11 |

As curas `3a1a681244`, `bf7a6be580`, `eeae7c0587` e as do fecho (`8420913318..afe704e75c`) têm teste e a foto `animado_folhas_trim`; **não consta** que o dono as tenha visto: trate como não smokadas.

**Rodada 0** (reconstruída dos commits). `572d4daf02` aba Scene que acompanha o objeto (`screens/hero/layout_switch.rs`) e menu Run = cinco botões no centro da barra (`transport_bar.rs`, ícones `docs/design/icons/play-*.svg`; [memória](../../../project-memory/project_scene_tab_follows_object_transport_in_bar_2026_11_07.md)). `f3a62586f9` tela cheia com jogar/pausar/parar, fechar pára o jogo; nasce o **CLAUDE §0.11** (sem compatibilidade com projeto antigo, [memória](../../../project-memory/feedback_no_old_projects_no_compatibility_until_the_app_works.md)). `dbc607fb7d` em tela cheia o clique chega aos botões do jogo (provado com cliques injetados em `on_mouse_input`). `75564ef918` pausa padrão-ouro (mundo e som congelam, editor escondido não reage, Pause alterna); `f61a7ebf94` a memória do som pausado mora em `ph2d-audio`. **Limpeza §0.11** (`cf3ee1b6b9`, `09aa461e7a`, `ababb82c8c`, `455f436fe2`, `e2fd40d43e`, `36ce5b06e4`, `2ed3c30056`; ~−17 000 linhas): sai o invólucro versionado do `Transform`, `SpriteVersioned` e as fixtures v1/v3, `board-model/legacy.rs`, `project_migrate.rs`, leitores de versão antiga de vec/skeleton/constraints/timeline/áudio/sfx/macros; `SpriteAnimStart`/`SpriteCutNotice` viram **campos** (`36ce5b06e4`, **schema 217→218**).
⚠️ **É a única rodada SUBTRATIVA**: saíram itens públicos (`SpriteVersioned`, `DesviraFn`/`caixas_v209`, `item_id_v194/v196`, `canal_chave_v23`, `camera_2d_v128`, `save_v1`, `signal_actions_v1`, `vec_filter_v195`, a dep `postcard` de `ph2d-render`). Linha que os **use** não compila; fixture/migração antiga nova cai no §0.11 (apagar, não ressuscitar).

**Rodada 1** — [continuação](HANDOFF_CONTINUACAO_2026-11-07_EDITOR_ANIMACOES_ONDA1.md) · [estudo](../estudo_editor_de_animacoes_2026-11-07.md). Tela nova (lista · folha · cartão com tira), cor por animação (`AnimationTag.color`, **schema 218→219**), faixas na folha, arrastar a letra dos campos; Once de UMA célula acaba e grita On Finish; oráculo Godot 4.7.2 (`crates/ph2d-ecs/tests/fixtures/anim_godot/`, `the_anim_ends_match_godot` 5 verdes + 2 `#[ignore]`; divergências recusadas no README da pasta). Mutações: 10/10.

**Rodada 2** — [continuação](HANDOFF_CONTINUACAO_2026-11-07_ADD_OBJECT.md) · [plano 24](../24_add_object_plano.md). Gesto **opt-in** `PaletteModel.select: Option<PaletteSelect>` (1 clique destaca, duplo/Enter escolhe), faixa da dica, aspeto compacto (`command_palette/{look,grupo,selecao}.rs`), 4 grupos, palavras de busca, ensino da busca vazia, Recent (3, `~/.ph2d/sections.txt`, fora do `.ph2dproj`). Add Component, Ctrl+K e a biblioteca do Motion ficam `select: None`.

**Rodada 3** — [continuação](HANDOFF_CONTINUACAO_2026-11-08_ADD_COMPONENT.md) · [plano 25](../25_add_component_plano.md). 11 assuntos, 63 dicas (`ph2d-game-dicas/src/componente.rs`), «+N» em plaquinha (`PaletteItem.badge`), recibo com Undo e × (`recibo.rs`), duplo clique real e foco da busca só após clique/tecla; a Game Camera sai do Add Component (só Add Object ▸ Camera); ícone da câmera no canvas na cor `ObjCamera` com a sombra da lei GEF02 (`ph2d-camera-frame/src/tinta.rs`). LOC no limite: `card_field.rs` 499/500, `fase_object_add_tests.rs` 600/600.

**Rodada 4** — [plano 26](../26_animated_sprite_varias_folhas_plano.md) · [V1](HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V1_FEITA.md) → [V5](HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_V5_FEITA.md) → [AUDITADA](HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_AUDITADA.md). `FrameRef{Cell{sheet,cell},Empty}`, `SpriteGrid.sheet`, `AnimationTag.sheet`, componente `SpriteSheets` (só ids de imagem; `SHEETS_MAX=255`), textura = cache derivado (**schema 219→220**, `b9ab501c58`). Cartão Image: ▾ na miniatura, **Load sheet** (não cria animação), **Delete sheet** (quadros vazios em alerta; Remove empty frames · Delete empty animations). Editor: escolhe a folha, tira pinta cada quadro pela sua folha, marca de inserção e voo. Em todo o app: em uso, assets, miniatura, gravar/abrir, macro (também noutro projeto), duplicar, Stop; Painter abre e grava a folha visível. Cena «Two Sheets» = `PH2D_GAME_SMOKE=5`.

**Rodada 5** — [doc 27](../27_ferramentas_na_folha_visivel.md). Funil `texture_edit::read_shown_source` → `commit_edited_texture`, lei em `ph2d-anim-sheets::shown_edit`; Trim, Padding, Make Square, Upscale, BG removal, Color equalization, Equalize sizes, Painter, Use as brush/paper, Send to board agem na folha à vista; numa folha extra o deslocamento soma-se ao **Pivot da folha** (o `Transform` é do objeto). **Rasterize recusa** num objeto com folhas extra (`eeae7c0587`). Portão da fiação: `shells/desktop/tests/it/image_tools_act_on_the_shown_sheet.rs` (verbo novo em `hero_intents/image_edit/` fora da lista reprova).

## 2. Foundational tocado (`git diff --stat main..HEAD -- crates/ph2d-ecs crates/ph2d-render crates/ph2d-editor-core`; `src`: +3 830 −2 783)

| Onde | O que | Natureza |
|---|---|---|
| `ph2d-ecs` | `FrameRef`, `SpriteGrid.sheet`, `AnimationTag.{sheet,color}`, `SpriteSheets` (registado), `ShownSheet` (UI, não registado), `linked_art_chain` (movida da shell), `sheet_cells_of`/`cells_per_sheet`, `anim_steps(world,e,name)` | **aditivo** (campos/componente; postcard é posicional ⇒ degraus do §3). Exceção: `SpriteAnimStart`/`SpriteCutNotice` deixam de ser componentes (rodada 0) |
| `ph2d-render` | `sprite/sheets.rs` (`SheetTextures`, `SheetView`, `sheet_view`, `resolve_sheet_textures`, `upload_sheet_image`), `image_of.rs`, `open_preview_cell` | aditivo; **subtrativo**: `sprite_versioned.rs`, fixtures v3, dep `postcard` |
| `ph2d-editor-core` | `AnimFrameRef{sheet,cell}`, `InspectorAnimRow.{frames,sheet}`, `InspectorAnimInfo.{frame,sheets}`, `AnimFieldEdit::Frames`, `AnimCardInfo.{extra_sheets,any_sheet}`, `PaletteSelect` + `command_palette/*`, `PaletteItem.badge`, `WidgetStore::ADD_RECENT_MAX = 3`, `object_add*.rs`, `toast.rs`, `screens/hero/{transport_bar,layout_switch}.rs` | aditivo; `select: None` é o padrão: as outras paletas ficam byte a byte |
| outras | `ph2d-panel-inspector` (`cards_sprite_folhas.rs`, `recibo.rs`), `ph2d-i18n`, `ph2d-game-dicas`, `ph2d-app-components`, `ph2d-audio`/`ph2d-transport`/`ph2d-app-audio` (pausa), `ph2d-camera-frame`, `shells/desktop` (`main.rs` +6 −5; 1 024 linhas, 1 023 no `main`, pré-existente) | consumidores / composição |

## 3. Contratos, schema, crates novas

**`PROJECT_SCHEMA` 217 (`main`) → 220: +3**, todos desta linha, escada e tripla em `project_schema.rs`/`_tests.rs` (`(220, 13, 29)`): **218** campos (rodada 0) · **219** cor da animação (1) · **220** várias folhas (4). VEC 29, FLIP 13, timeline 25 inalterados. **Reconte pelo `main` do dia** (`python3 scripts/schema-recount.py <valor>`). §0.11: ninguém abre `.ph2dproj` anterior a 218.

**Registo** (`ph2d-ecs`, `registry_tests.rs:262`): `main` 120, linha 119 ⇒ **DELTA −1** (−2 do 07/11, +1 `SpriteSheets`). Espelhos de `ph2d-render`/`ph2d-script` derivam de `ph2d-ecs`; a sonda do script está **cega** para eles: confira à mão.

**§6 intocado** (corrido 08/11, vazio): `git diff --stat main..HEAD -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-nodegraph crates/ph2d-imageio/src/color.rs crates/ph2d-vector-doc crates/ph2d-vector-traits` e `-- scripts .config .github .claude`. Sem ADR novo (próximo livre 0199).

**Crates NOVAS** (`git diff --name-status main..HEAD | grep Cargo.toml`): **`ph2d-anim-sheets`** (lei pura: `ops`, `apply`, `recut`, `shown_image`, `shown_edit`, `names`, `editor_ui`, `split`, `panel_frame`, `sheet_cache`; 39 testes) e **`ph2d-anim-editor`** (documento do editor: `doc*`, `layout`, `paint*`, `bands`, `marks`, `strip_bar`). Dependem delas `ph2d-app-sprite-anim` e `shells/desktop`; `ph2d-render` perde `postcard`. **`Cargo.lock` +50 −1**: conflito = `cargo metadata`, nunca à mão. Catraca `ph2d-app-sprite-anim` = 5 209 (era 7 545); `ph2d-app-components`/`ph2d-app-painter` perto do tecto (cura = mover). **`cargo machete` não foi corrido pela linha** (`ship.sh`); nenhum `cfg(target_os` novo (`git diff -U0 main..HEAD -- '*.rs' | grep -c '^+.*cfg(target_os'` = 0).

## 4. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional/scripts/collision-surface.sh`, 08/11)

```
merge-base 652b506b58 · 77 commit(s) · 650 arquivo(s)
PROJECT_SCHEMA 220 (base: 217) · tripla (220, 13, 29) (base: (217, 13, 29)) · VEC 29=29 · FLIP 13=13 · DOC 25=25
REGISTRO ph2d-ecs 119 (base: 120) · espelhos render/script: SONDA CEGA
CONTRATO node.rs / tool.rs: intocado · ADR: nenhum
Cargo.lock: +"ph2d-anim-editor" +"ph2d-anim-sheets" · marcadores de conflito: nenhum
LOC: ✗ 1024/600 shells/desktop/src/main.rs (1 023 no main)
```
Rode-a pelo **caminho ABSOLUTO** em cada worktree; `base:` é o merge-base — leia o `main` no ficheiro. Arquivos quentes (o que outra linha também mexe): `shells/desktop/src/main.rs` · `project_schema.rs` + `_tests.rs` · `crates/ph2d-ecs/src/scene/registry{,_tests}.rs` · `crates/ph2d-editor-core/src/widget/command_palette{.rs,/*}` · `…/screens/hero/{transport_bar,menu_bar,paint}.rs` · `…/tests/it/architecture_the_shell_only_shrinks.rs` · `…/tests/it/node_id_collisions.rs` · `crates/ph2d-panel-inspector/src/cards_sprite*.rs` · `crates/ph2d-i18n/src/{component_add,object_add}.rs` · `CLAUDE.md` §5 (Game Engine) · `project-memory/MEMORY.md` · `Cargo.lock` · `.config/testes-lentos.tsv` (regenerar). Maiores adições: `shells/game-scenes/tests/it/editor_animacoes/*` (~1 400), `cards_sprite_folhas.rs` (426), `command_palette/*` (~1 200).

## 5. Medido e provado (fecho, em `afe704e75c`)

- **fmt** aplicado (`317ef7deeb`). **clippy `-D warnings`, 46 pacotes tocados: 0 achados após as curas** — `5d0ebb02be` (`paint_toggle` recebe a faixa como `[y, h]`) e `afe704e75c` (var sem uso; 2× `too_many_arguments` pelo par `(entity, sheet)`; `slice::from_ref`; `safe_clamp` em `transport_bar`; **ciclo do DAG `interaction↔object_add` quebrado** por `WidgetStore::ADD_RECENT_MAX`; aresta `widget→interaction` de volta a 41/41; `fase_snapshots_publish` 206→ partida em `animation_editor`).
- **Gates:** `ph2d-editor-core` filtro `arch` **137/137**; desktop `file_loc_caps`/`fn_loc_caps`/`architecture_*` verdes; `the_shell_only_shrinks` verde; imageio contract 6/6; i18n 18/18. Testes da linha verdes: `ph2d-host-desktop` conjunto nomeado 36/36, `ph2d-anim-sheets` 39, `editor_animacoes` 63.
- **Mutações do fecho:** bloco `ba2b76ca0d` **15/16** sangraram (5b = gate textual de `read_shown_source`, coberto ponta a ponta pelo relato do smoke; 5a descoberta por desenho, GPU). Devidas do fecho **27/29**; os 2 buracos fechados por testes novos: `8420913318` (Recent lê e mostra só 3, `take(3)` nas duas pontas) e `81657b9972` (folha extra com recurso que não descodifica = `Unreadable`). **As 6 leis do Add Object devidas desde a rodada 2, agora provadas:** destaque inicial = Sprite · duplo clique escolhe · gesto opt-in · altura reservada da faixa da dica · filtro por palavras de busca · corte de 3 do Recent.
- **Ponta a ponta** (foto `animado_folhas_trim`, relato): `extra=[([0.48,0.48]…)]` → `[([0.44833335, 0.45], [-0.0026190376, -0.03000021])]` após o Trim; `run.png` 288×96 → 269×90, 6×2 (o Pivot da folha andou, o objeto não).
- **Binário:** `cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida: «Finished \`smoke\` profile [optimized] target(s) in 0.37s», zero `Compiling`.

## 6. Riscos de integração

- **Primário sujo:** `git -C /home/enio/Documentos/Projetos/PH2D status --short` ANTES. Tinha `M project-memory/MEMORY.md` e `??` de outras sessões, **incluindo `project_edit_mode_turns_sprite_still_2026_11_08.md` e `project_image_tools_act_on_visible_sheet_2026_11_08.md`, que NÃO estão nesta branch** (por isso sem link). Nunca descartar: funda o `MEMORY.md` por reconta, sem `checkout`.
- **`CLAUDE.md` §5.1**, entrada Game Engine: esta linha troca o «Último» e a frase; tecto 700 bytes/entrada e do §5 inteiro.
- **Rodada 0 subtrativa:** compile o workspace antes de testar; `grep -rn "SpriteVersioned\|DesviraFn\|transform_versioned\|project_migrate" --include=*.rs` nas outras linhas.
- `PH2D_GAME_SMOKE=5` mudou de conteúdo («Two Sheets» tem a folha `run`): fotos antigas dela não valem.

## 7. SMOKE do dono

Antes: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build -p ph2d-host-desktop --profile smoke` (depois da fusão, o mesmo no `cd /home/enio/Documentos/Projetos/PH2D`).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=5 ./target/smoke/ph2d-host-desktop`. Na Hierarchy clique **Two Sheets**; no cartão **Image** a miniatura tem **▾**: o dropdown lista as duas folhas (nome, `C × R`, ✓). Errado: sem ▾ ou sem a folha `run`.
2. Escolha **run** no ▾ e use **Trim** nas ferramentas de imagem: a folha `run` encolhe, a arte não se mexe na tela, a folha 0 fica intacta; **Ctrl+Z** devolve num passo. Errado: folha 0 mudou ou o boneco saltou.
3. No cartão **Animation** clique o ícone de filme (**Edit Animations**): o título escolhe a folha; clicar um quadro da tira põe a marca de inserção à direita; **Load sheet** não cria animação; **Delete sheet** deixa quadros vazios em alerta com os dois botões de limpeza.
4. `PH2D_OBJECT_ADD_SMOKE=1` (Add Object: clique destaca, duplo clique escolhe, **Recent** guarda 3) e `PH2D_COMPONENT_ADD_SMOKE=1` (11 assuntos, «+N» em plaquinha, recibo com Undo e ×); a Game Camera só nasce por Add Object ▸ Camera.
5. Run são cinco botões no centro da barra; jogar em tela cheia, pausar (mundo e som congelam), parar; fechar a janela pára o jogo.

Fotos: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE_FOTO=animado_folhas|animado_folhas_editor|animado_folhas_apagada|animado_folhas_trim bash docs/Components/ferramentas/fotografa_cena.sh PH2D_GAME_SMOKE=5 1930 1040 <out.png>` (uma por vez; `animado_folhas_trim` imprime o relato do Pivot).

## 8. ABERTO

1. **Dono 08/11, para DEPOIS de integrar (onda nova):** entrar no modo **Edit** de um objeto Image devolve a sprite a **Still**; as operações que mudam tamanho/margem (Trim, Padding, Make Square, Upscale…) **avisam** que podem estragar as animações (memória `project_edit_mode_turns_sprite_still_2026_11_08.md`, só no primário). Torna discutível a hipótese do doc 27 §3.3; `shown_edit` fica como está até lá.
2. **[27 §3](../27_ferramentas_na_folha_visivel.md):** (3) folha 0 de um Animated recebe `Sprite.size` de UMA célula e Trim/Make Square/Padding põem lá o tamanho da imagem inteira — hipótese **não medida** (curar = separar o tamanho do verbo do `pre_size` do undo, nove entradas); (4) **o dono decide:** soltar/escolher imagem no cartão com folha extra à vista (`image_choice.rs`) troca a folha 0 — deve trocar a folha à vista?; (6) as ilhas do BG removal nascem com escala 1 e sem flip. Feitos: 1, 2, 5.
3. **Escolha conhecida:** trocar de folha com o Painter ligado ACHATA as camadas da folha antiga (não há documento do Painter por folha). Rasterize só recusa (por folha seria outro verbo).
4. `Documents::wants_frame` sem leitor (doc 27 §4): contrato mantido; o portão `a_frame_request_has_a_listener` reprova se o laço deixar de ser `Poll`.
5. **Sem prova:** clique real na tela virtual (XTest) · mutação 5a (GPU) · cena 5 sem tutorial em PDF · «Start…» cortado no piso do painel (chrome comum) · Frames/s, Cycle e Rules sem losango de Auto Key (decisão (e) do handoff de 07/11, o dono decide).
6. **Fila** ([23](../23_refundacao_plano.md)): Inspector e Hierarchy · Prefab; o dono confirma. `line/GameEngineFuncional02` (física em cartões) é separada; a ordem é do integrador.

## ⛔ Recusas MEDIDAS

Folha ou tira dentro do cartão (27/10) · empatar o Godot no tique de fim (muda com o passo; README do oráculo) · Rasterize por folha · `ImageEditSnapshot` numa folha extra (restauraria campos do `Sprite` com o tamanho dela) · «Load sheet» que cria animação (dono 08/11).
