# HANDOFF de integração — `line/GameEngineFuncional`, 10/11 (os ABERTOS do Pivot + o Align ao criar)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, base `bc4a6ce930` = ponta do `main` (nada entrou no `main` desde; sem rebase, `--ff-only` serve). **6 commits** (os 5 abaixo + o deste doc), 36 ficheiros (`git diff --stat bc4a6ce930..HEAD`: +1137 −539 antes deste doc). Nada integrado, nenhum push. **Suíte completa e `nextest-impacted` NÃO correram** (regra do dono 23/10): são do integrador. O handoff anterior ([Align e Pivot, 09/11](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-09_ALIGN_E_PIVOT.md)) JÁ está no `main`; os itens ABERTOS dele são o que esta linha fecha.

## 1. Commits

| Commit | O que |
|---|---|
| `a271b50367` | o executor da cena 5 sai da shell para `ph2d-sprite-anim-scene` (`mesa.rs`) |
| `9413724e40` | `FolhasAVista` + `commit_anim_edit` (a folha à vista atravessa o Ctrl+Z; o «+» do cartão escolhe a nova) + fotos |
| `cbbed33ef9` | handoff de continuação ([base do «Feito» abaixo](HANDOFF_CONTINUACAO_2026-11-10_ABERTOS_DO_PIVOT.md)) + `fotos_30` |
| `b3e8a9368e` | a animação nova sai do Align (pedido do dono no smoke de 10/11) |
| `b0244c195d` | curas da auditoria (2 lentes) + a catraca da família |

## 2. Feito (os ABERTOS do handoff de 09/11)

| ABERTO (09/11) | Cura | Prova |
|---|---|---|
| 1 · drivers de smoke na shell | o executor da cena 5 mudou para a folha `ph2d-sprite-anim-scene` (`mesa.rs`: trait `Mesa`, 18 portas-valor; a shell só a implementa e diz se a cena está armada). `janela_do_jogo_animado.rs` 356 → 145. Família → família é proibido (lei 1 das camadas) e a `ph2d-app-sprite-anim` estava no tecto (5 209), por isso a folha. Os passos das fotos do editor saíram para `fotos.rs` (tecto de 700 do `lib.rs`) | `no_dependency_climbs_a_layer`, `the_shell_only_shrinks*`, órfãos, LOC, tabela de strings da shell (3 isenções saíram com os literais) |
| 2 · `ShownSheet` volta a 0 depois do Ctrl+Z | `ph2d_ecs::FolhasAVista` (antes/depois pela `StableId`), levada pelo `AntesDoRestauro` (`shells/presos`) e devolvida por `repoe_a_sessao` logo a seguir ao `restore` em `apply_project_with` (+1 linha na shell, `undo_app.rs`). Vale para Ctrl+Z, Ctrl+Y e Stop do jogo (ver §4 para o que NÃO leva) | `a_folha_a_vista_atravessa_o_respawn_pela_identidade` · `o_editor_continua_na_folha_a_vista_depois_do_restauro` · `the_only_restore_door_of_the_shell_calls_the_two_halves_in_order` · `a_porta_do_restauro_leva_a_folha_a_vista_so_num_passo` · foto `fotos_30/animado_folhas_desfaz` |
| 4 · o «+» do cartão Animation não escolhe a nova | `inspector::commit_anim_edit` = a porta ÚNICA do cartão (shell `inspector_commits.rs`) e do editor: nota o gesto, aplica, commit, e depois de um Add o `SetCurrent` da nova (o `start` fica na de antes, a lei do `SetCurrent`). O laço do editor perdeu a cópia própria | `o_mais_escolhe_a_animacao_nova` |
| 7 · Follow depois da escala pela quina | medido na janela: foto `animado_pivot_segue_escala_desfaz` — `pivô 1 Ctrl+Z` == `pivô no ponto`. Sem mudança de código | `fotos_30/*.log` |
| 3 · «INSERT HERE» cobre o último quadro | **não reproduzido**: `a_ranhura_nunca_cobre_um_quadro` (5 larguras × marca no fim/meio × rolada 0/fim) verde; provável cura anterior (marca «presa aos quadros que a tira mostra», 09/11). Se o dono ainda o vir: rolar a tira ao fim pela barra e fotografar | teste |

Regressão: `animado_pivot_segue_desfaz` igual a `fotos_29` em 10 de 11 linhas; a 11.ª difere só em `visivel=0 → 1` (a cura 2). `fotos_29/animado_pivot_segue_desfaz.log` está desatualizado.

## 3. Align ao criar (pedido do dono, smoke 10/11)

«Se está no Align e cria animação nova, sai do Align.» `AnimEditorAlignRequests` era `Vec<u64>` (só ligar) e passou a `Vec<(u64, bool)>` = (aba, ligado). `commit_anim_edit` empurra `(aba, false)` depois de um Add (caminho do cartão, drenado no quadro seguinte); o laço do editor chama `AnimDoc::align_off()` já no mesmo quadro. `align_off` (`ph2d-anim-editor`, `align.rs`) = on/drag/picking como o botão; o botão Align passou a usá-lo (cura também o conta-gotas que ficava armado). Prova: `a_animacao_nova_sai_do_align` (mutação M7 vista vermelha). Smoke do dono: OK.

## 4. Curas da auditoria (2 lentes, `b0244c195d`)

1. **MÉDIA, confirmada:** `FolhasAVista` herdava a folha à vista ao ABRIR outro projeto (`StableId` repete-se entre projetos) → `repoe_a_sessao(world, passo)`; a shell passa `mark.is_some()`. `Some` = Ctrl+Z/Ctrl+Y (`undo_app_macros.rs:95`) e Stop (`undo_jogo.rs:73`); `None` = abrir projeto (`project_load.rs:289`) e Cancel de prefab (`prefab_exit.rs:46` — este deixa de levar a folha, aceite).
2. **MÉDIA:** o Align-off não desarmava o conta-gotas → `align_off`.
3. Não confirmado: «o `SetCurrent` do cartão muda o `start` da cena 5» — falso: a lei do `SetCurrent` (`inspector.rs`, `p.start.is_none() && p.current != name`) já gravava `start=idle` no passo 9 da cena; mesmo resultado.
4. Um quadro de atraso no editor → curado (`align_off` direto).
5. O Add sem tocador criaria `SpriteAnimator` → `nova` só com `SpriteAnimator` presente.
6. Comentário das fases do `alinha` em `mesa.rs` corrigido.

**Catraca da família:** `ph2d-app-sprite-anim` tinha crescido para 5 285 > tecto 5 209 (`the_shell_only_shrinks_and_every_app_crate_too`). Cura: `tiled.rs` (78) e `previa.rs` (49) desceram para `ph2d-anim-sheets` (re-export `pub use ph2d_anim_sheets::{previa, tiled, use_frame}`; `follow_auto_collider` de `pub(crate)` a `pub`; +dep `ph2d-physics-ecs` em `ph2d-anim-sheets`, que a família já tinha). Medido: **5 173** linhas, tecto 5 209 inalterado, sem catraca obsoleta.

## 5. Foundational, contratos, schema, registo

- `ph2d-ecs`: `FolhasAVista` (aditivo; nenhum tipo existente mudou; export em `lib.rs`). `ph2d-host-presos`: `AntesDoRestauro` virou struct com campos privados + `repoe_a_sessao`; dev-dep `ph2d-asset`. `ph2d-anim-editor`: `align_off`.
- **§6 intocado:** `git diff --name-only bc4a6ce930..HEAD | grep -E "nodegraph|tools|ph2d-vector|ph2d-imageio"` → vazio; `node.rs`/`tool.rs` «intocado» no `collision-surface`.
- **Schema e registo intocados:** `PROJECT_SCHEMA` 231 (= base), tripla `(231, 13, 29)`, VEC 29, FLIP 13, timeline 25; registo `ph2d-ecs` 121 = base (DELTA 0; `ShownSheet` continua não registado, nada gravado mudou). Sem ADR. Reconte pelo `main` do dia: `python3 scripts/schema-recount.py`.

## 6. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional/scripts/collision-surface.sh`, 10/11)

```
merge-base bc4a6ce930 · 5 commit(s) · 36 arquivo(s)
PROJECT_SCHEMA 231 (base 231) · tripla (231, 13, 29) = base · VEC 29 · FLIP 13 · DOC 25
REGISTRO ph2d-ecs 121 (base 121) · espelhos render/script: SONDA CEGA (✗, como no handoff de 09/11)
CONTRATO node.rs / tool.rs: intocado · ADR: nenhum (próximo livre 0201)
Cargo.lock: nenhum '+name' novo · marcadores de conflito: nenhum
LOC: nenhum arquivo da linha passa do teto
```
Arquivos quentes: `shells/desktop/src/undo_app.rs` (1 linha) · `render_loop/inspector_commits.rs` · `janela_do_jogo_animado.rs` · `every_word…string_table.rs` (−3 isenções) · `o_desfazer_nao_pula.rs` · `Cargo.lock` · `crates/ph2d-ecs/src/lib.rs` (export) · `CLAUDE.md` §5 (Game Engine).

## 7. O que um merge pode partir

- Outra linha que construa `AntesDoRestauro` por literal ou leia campos dele (agora privados): `git grep -n AntesDoRestauro`.
- Outra linha que escreva em `AnimEditorAlignRequests` com `u64` (agora `(u64, bool)`): `git grep -n AnimEditorAlignRequests`.
- Outra linha que chame `tiled`/`previa` por `ph2d_app_sprite_anim::...` continua a compilar (re-export); por caminho de ficheiro, não.
- Tabela de strings da shell: −3 isenções em `every_word…string_table.rs` (conflito textual provável se outra linha mexer nas isenções).

## 8. Prova de fecho (verificador, 2 rodadas)

- fmt e `clippy --all-targets -D warnings` das 9 crates tocadas (`ph2d-ecs`, `ph2d-anim-sheets`, `ph2d-anim-editor`, `ph2d-app-sprite-anim`, `ph2d-sprite-anim-scene`, `ph2d-host-presos`, `ph2d-host-desktop`, `ph2d-panel-registry-init`, `ph2d-host-game-scenes`): limpos.
- Gates: `the_shell_only_shrinks` 2/2 · `no_dependency_climbs_a_layer` 3/3 · `workspace_src_files_under_loc_cap` · `file_loc_caps` 4/4 · `architecture_no_orphan_source_file` 4/4 · `hr15_no_hardcoded_ui_strings` · `no_literal_color` · `arch_safe_clamp_only` · `every_word_the_shell_shows_comes_from_the_string_table` 3/3 · `the_only_restore_door_of_the_shell_calls_the_two_halves_in_order` — verdes.
- Testes: `ph2d-host-presos` 3/3 · `ph2d-anim-sheets` 61/61 · `ph2d-sprite-anim-scene` 3/3 · `editor_animacoes` 88/88 · `sprite_tiled` 4/4 · `plataforma_alargada` 1/1 · `colisao_segue_a_largura` 5/5 · `previa_tests` (desktop) 3/3 · os de item.
- **Mutações** (verdes antes, vermelhas com a mutação, restauradas): M1 `FolhasAVista::depois` vazio (2 testes) · M2 sem o `SetCurrent` da nova · M3 sem a linha `repoe_a_sessao` na shell (gate textual) · M4 pedido `(b, true)` · M5 `align.on=true` no laço e M5b `on=false` (`o_campo_pivot_abre_o_editor_com_o_align_ligado`) · **M6** `folhas: Default::default()` em `antes_do_restauro` **sobreviveu na 1.ª rodada**, fechada por `a_porta_do_restauro_leva_a_folha_a_vista_so_num_passo` (vermelho: passo=true left 0 right 2) · M7 `align_off` sem o reset do picking → `a_animacao_nova_sai_do_align` vermelho.
- O gate de `machete` só existe no `suite-completa`/`ship`: não correu.

## 9. ABERTO (baixa)

- As 3 strings de diagnóstico de consola que saíram da shell para `mesa.rs:166,174,280` ficam fora do censo da tabela de strings da shell (não são UI pintada; nenhum censo varre `ph2d-sprite-anim-scene`).
- `ph2d-sprite-anim-scene` depende de `ph2d-anim-editor` só pelo re-export `AnimDoc` (dep transitiva em `ph2d-app-game`).
- Se o `SetCurrent` recursivo fosse recusado, `commit_anim_edit` devolveria Recusado após um Add aplicado (improvável).
- Não feito, de propósito: «Cmd+Z» nos textos (rótulo por sistema operativo, linha de UI); `ph2d-app-macros tutorial_painter::outra_tela_diz_o_tamanho_gravado` (alheio); `janela_do_jogo_smoke.rs` (214 linhas) continua na shell — pode ser da GEF02, a `Mesa` serve-lhe de molde; «INSERT HERE» não reproduzido (§2).

## 10. Smoke do dono (binário `--profile smoke`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. «Two Sheets»: no cartão Image mostre a folha `run`; Edit in Animations; mude o tamanho da folha; Ctrl+Z → continua na `run`. Errado: volta à `idle`.
3. Cartão Animation, «+ Add» → a nova fica destacada. Errado: o destaque fica na de antes.
4. Align + Follow ligados, escale pela quina, Ctrl+Z → folha e Pivot voltam juntos.
5. Align ligado (conta-gotas armado ou não), «+» → sai do Align e a folha volta. Errado: continua em Align.

Fotos: [`fotos_30/`](../fotos_30/). Smoke do dono de 10/11: OK nos passos 2–4 e, depois do pedido do Align ao criar, no passo 5.
