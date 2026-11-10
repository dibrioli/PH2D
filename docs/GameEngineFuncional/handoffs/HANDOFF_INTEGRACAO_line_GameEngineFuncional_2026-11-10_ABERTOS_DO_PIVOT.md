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
| `c93ce28b8c` | smoke do dono 10/11: com o botão premido na folha, o quadro já aparecia no lugar da tira antes de o cartão descer (o `Drag::Cells` mostrava a lista emendada com o vão aberto; o voo só partia ao soltar, de vão fechado). `AnimDoc::arriving` (ph2d-anim-editor) junta o voo e os escolhidos do `Drag::Cells` (vão `0`); `flying` e a disposição da tira leem-no. Teste `com_o_botao_premido_o_quadro_ainda_nao_esta_na_tira` (mutação: intervalo vazio ⇒ vermelho); `editor_animacoes` 89/89; clippy da crate limpo |

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

## 9. ABERTO — fechado na onda `401d4d5e3c` (§11)

Os quatro itens desta secção e os dois de «não feito» que não são da GEF02 foram fechados num bloco (§11). Fica de fora, de propósito: `janela_do_jogo_smoke.rs` (214 linhas na shell) — da GEF02; a `Mesa` serve-lhe de molde.

## 10. Smoke do dono (binário `--profile smoke`)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. «Two Sheets»: no cartão Image mostre a folha `run`; Edit in Animations; mude o tamanho da folha; Ctrl+Z → continua na `run`. Errado: volta à `idle`.
3. Cartão Animation, «+ Add» → a nova fica destacada. Errado: o destaque fica na de antes.
4. Align + Follow ligados, escale pela quina, Ctrl+Z → folha e Pivot voltam juntos.
5. Align ligado (conta-gotas armado ou não), «+» → sai do Align e a folha volta. Errado: continua em Align.
6. Clique devagar (segure o botão) numa célula da folha → nada aparece na tira enquanto segura; ao soltar, o cartão desce e o lugar abre. Errado: o quadro aparece no lugar antes de descer.

Fotos: [`fotos_30/`](../fotos_30/). Smoke do dono de 10/11: OK nos passos 2–4 e, depois do pedido do Align ao criar, no passo 5.

## 11. Onda dos abertos (`401d4d5e3c`, 10/11)

| Item | Mecanismo / cura | Prova |
|---|---|---|
| «Cmd+Z» no Linux/Windows | a tabela escreve `Ctrl+`; `ph2d_i18n::atalho` (`Sistema::DESTE`, `atalho_em`, `da_tabela` com cache leak-once) diz `Cmd+` só no macOS — o despacho aceita Ctrl OU ⌘ em todo acorde (`keyboard.rs:36`, `input_handlers.rs:68`). «Hold Ctrl» sem `+` fica literal. 33 entradas (`chrome_menus`, `chrome_panes`, `code_editor`, `shell_media`) + 7 literais (showcase, fixtures de widget, `main.rs`, testes do app-code) | `a_tecla_dos_atalhos_vem_da_porta` (tabela COM subpastas — o leitor de `a_tabela_inglesa_fala_ingles` só lê o 1.º nível — e todo `.rs` de `crates/`+`shells/`), `o_acorde_diz_cmd_so_no_mac`, `a_tabela_passa_pela_porta`. No Linux tirar a porta do `tr` não se vê (mesma palavra): o teste exerce `da_tabela(Sistema::Mac, …)` |
| `tutorial_painter::outra_tela_diz_o_tamanho_gravado` | NÃO era flake: estouro de pilha do fio `macro-referencias` (2 MiB). `TocadorDoPainter` levava o `PainterTool` (~270 KB) por valor; `Trabalho` passa pelo `sync_channel` da medição e o debug copia-o 4–5× (gdb, `$sp` por quadro: 2 083 272 B em 28 quadros, 4 de ~270 KB + `corre` 900 KB). Cura: `tool: Box<PainterTool>` (`ph2d-tool-painter/macro_track/tocar.rs`). Passa com `RUST_MIN_STACK=1 MiB`; 512 KiB cai no fio do próprio teste | `o_tocador_e_leve_para_viajar_por_valor` (≤ 4 KiB) + o teste das macros |
| strings da cena 5 fora do censo | a crate inteira (39 literais, desde GEF 24, já no `main`) não tinha censo: `ph2d-sprite-anim-scene/tests/it/every_word_this_scene_shows…` (`gate::intrusos_fora_de`); `lib.rs`/`fotos.rs` isentos por ficheiro (guião = nomes de objectos + terminal), as 3 de `mesa.rs` literal a literal | os 2 testes + `isentos_mortos` |
| Add aplicado e `SetCurrent` recusado | `Commit::FeitoComAviso(Toast)` (aviso `anim_editor.new_not_selected`); shell e editor tratam-no como feito (o editor sai do Align). Só um defeito de registo o provoca (um nome não registado a fila salta em silêncio) | `ph2d-panel-registry-init` `o_add_que_pegou_e_feito_mesmo_sem_escolher_a_nova` (lá porque a catraca de `ph2d-app-sprite-anim` conta `src/` E `tests/`) |
| dep `ph2d-anim-editor` da cena | cortada: a `Mesa` pede `AlvosDoEditor` (valores: botões do Align, pivô, caixa, pega, barra, ranhura, último quadro); a shell monta-o. `ph2d-app-game` deixa de chegar ao editor | `cargo tree -i ph2d-anim-editor -p ph2d-app-game` vazio; compila |
| «INSERT HERE» | foto com a tira rolada ao fim PELA BARRA (`Passo::RolaATira`: clique no fim do trilho, arrasto, soltar, relato) a 1930×1040 e 1366×768: `cobre=false` nas duas. Curado na foto estreita: o rótulo saía cortado em «INSERT» (quadro de 58 px) → `colunas_da_ranhura` parte por palavra e centra cada coluna por `folga_do_texto` (a âncora do texto rodado é o TOPO da linha; a coluna única também ficava à direita do meio) | `o_rotulo_da_ranhura_parte_se_quando_nao_cabe`; [`fotos_31/`](../fotos_31/) |

**Mutações** (verdes antes, vermelhas com a mutação, restauradas): M1 sem o `Box` (2 testes; o das macros aborta) · M2 `atalho_em` sem o braço Mac · M3 `da_tabela` com `Sistema::Outro` · M4 `Cmd+Z` de volta numa entrada · M5 `"Cmd+O"` no showcase · M6 o `SetCurrent` cru no fim de `commit_anim_edit` (refeita no sítio novo do teste) · M7 isenção apagada / M7b literal mudado · M8 `colunas_da_ranhura` sempre inteira.

**Fecho:** fmt e `clippy --all-targets -D warnings` limpos em ph2d-i18n, ph2d-tool-painter, ph2d-app-code, ph2d-editor-core, ph2d-host-desktop, ph2d-sprite-anim-scene, ph2d-app-sprite-anim, ph2d-anim-editor, ph2d-panel-registry-init. Verdes por nome: `the_shell_only_shrinks*`, `no_dependency_climbs_a_layer`, `workspace_src_files_under_loc_cap`, `architecture_no_orphan_source_file`, `hr15_no_hardcoded_ui_strings`, `no_literal_color`, `no_tofu_glyphs`, os 3 da tabela da shell (+1 isenção `main.rs` «Ctrl+Q»), os do i18n, `paleta`/`onda1` do app-code, `inspector` (22), `strip_bar`, `tutorial_painter`, `smoke` do app-game. Schema/registo/§6 intocados. **Suíte completa: do integrador.**

**O que um merge pode partir:** quem escrever `"Cmd+"` num texto (o censo reprova: escreva `Ctrl+`); quem fizer `match` exaustivo em `inspector::Commit` (variante nova); quem implementar `Mesa` (`editor` → `alvos`).

### Smoke do dono (onda)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=5 ./target/smoke/ph2d-host-desktop`
2. Menu **Edit**: tem de dizer «Undo · Ctrl+Z» e «Redo · Ctrl+Shift+Z». Errado: «Cmd».
3. Selecione «Two Sheets», cartão Animation ▸ «Edit animations…», escolha `run`, arraste a barra azul debaixo da tira até o fim: «INSERT HERE» fica DEPOIS do quadro 12. Errado: por cima dele.
4. Estreite a janela até os quadros ficarem pequenos: o rótulo vira duas colunas «INSERT» / «HERE», inteiras. Errado: só «INSERT».
5. «+ Add» no cartão Animation: a nova aparece e fica escolhida, sem aviso vermelho.
