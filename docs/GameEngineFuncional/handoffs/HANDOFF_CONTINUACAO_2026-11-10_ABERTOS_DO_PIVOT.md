# HANDOFF de continuação — `line/GameEngineFuncional`, 10/11 (os ABERTOS do handoff Align + Pivot)

Leitor: a próxima janela desta linha. Base `bc4a6ce930` (= `main` de 10/11); commits `a271b50367`, `9413724e40` e o
deste handoff. Nada integrado, nenhum push. Origem dos itens: [ALIGN_E_PIVOT §8](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-09_ALIGN_E_PIVOT.md).

## Feito

| ABERTO (09/11) | Cura | Prova |
|---|---|---|
| 1 · drivers de smoke na shell | o executor da cena 5 mudou para a folha `ph2d-sprite-anim-scene` (`mesa.rs`: trait `Mesa`, 18 portas-valor; a shell só a implementa e diz se a cena está armada). `janela_do_jogo_animado.rs` 356 → 145. Família → família é proibido (lei 1 das camadas) e a `ph2d-app-sprite-anim` estava no tecto (5 209), por isso a folha. Os passos das fotos do editor saíram para `fotos.rs` (tecto de 700 do `lib.rs`) | `no_dependency_climbs_a_layer`, `the_shell_only_shrinks*`, órfãos, LOC, tabela de strings da shell (3 isenções saíram com os literais) |
| 2 · `ShownSheet` volta a 0 depois do Ctrl+Z | `ph2d_ecs::FolhasAVista` (antes/depois pela `StableId`), levada pelo `AntesDoRestauro` (`shells/presos`) e devolvida por `repoe_a_sessao` logo a seguir ao `restore` em `apply_project_with` (+1 linha na shell). Vale para Ctrl+Z, Ctrl+Y, Stop do jogo e abrir, como a aba do editor | `a_folha_a_vista_atravessa_o_respawn_pela_identidade` · `o_editor_continua_na_folha_a_vista_depois_do_restauro` · a ordem da porta em `the_only_restore_door_of_the_shell_calls_the_two_halves_in_order` · foto `fotos_30/animado_folhas_desfaz` (`folha 1 Ctrl+Z` com `visivel=1` e a `run` de volta a 0,48) |
| 4 · o «+» do cartão Animation não escolhe a nova | `inspector::commit_anim_edit` = a porta ÚNICA do cartão (shell `inspector_commits.rs`) e do editor: nota o gesto, aplica, commit, e depois de um Add o `SetCurrent` da nova (o `start` fica na de antes, a lei do `SetCurrent`). O laço do editor perdeu a cópia própria | `o_mais_escolhe_a_animacao_nova` |
| 7 · Follow depois da escala pela quina | medido na janela: foto `animado_pivot_segue_escala_desfaz` — `pivô 1 Ctrl+Z` == `pivô no ponto` (escala e Pivot voltam num passo). Sem mudança de código | `fotos_30/*.log` |
| 3 · «INSERT HERE» cobre o último quadro | **não reproduzido**: `a_ranhura_nunca_cobre_um_quadro` (5 larguras × marca no fim/meio × rolada 0/fim) verde; a foto `animado_tira_rolada` mostra a tira no começo (o `SetStep` não rola). Provável cura anterior (a marca «presa aos quadros que a tira mostra», 09/11). Se o dono ainda o vir: rolar a tira ao fim pela barra e fotografar | teste |

Regressão: `animado_pivot_segue_desfaz` igual a `fotos_29` em 10 de 11 linhas; a 11.ª difere só em `visivel=0 → 1` (a cura 2). `fotos_29/animado_pivot_segue_desfaz.log` está, portanto, desatualizado.

## Não feito, de propósito

- «Cmd+Z» nos textos (8): o app inteiro diz «Cmd+Z» (menus, editor de código, avisos de várias famílias) — é um rótulo por sistema operativo da linha de UI, não desta.
- `ph2d-app-macros tutorial_painter::outra_tela_diz_o_tamanho_gravado` (5): alheio; não conferido.
- `janela_do_jogo_smoke.rs` (214 linhas, mesma família) continua na shell: pode ser da `GameEngineFuncional02`; a `Mesa` serve-lhe de molde.

## Para o fecho da linha

Não correram: fmt/clippy das crates tocadas, mutação, auditoria, suíte. Mutações sugeridas: `FolhasAVista::depois` vazio; `commit_anim_edit` sem o `SetCurrent`; tirar `antes.repoe_a_sessao(...)` de `undo_app.rs`. Crates tocadas: `ph2d-ecs` (foundational, aditivo: `FolhasAVista`), `ph2d-sprite-anim-scene` (+3 deps folha), `ph2d-app-sprite-anim`, `ph2d-host-presos` (`AntesDoRestauro` virou struct com campos), `ph2d-host-desktop`, testes em `ph2d-panel-registry-init` e `ph2d-host-game-scenes`. Schema e registo intocados.

## Smoke do dono (binário `--profile smoke` já compilado nesta worktree)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=5 cargo run -p ph2d-host-desktop --profile smoke`
2. «Two Sheets»: no cartão Image mostre a folha `run`; abra Edit Animations; mude algo (Scale sheet) e Ctrl+Z → o editor continua na `run`. Errado: volta à `idle`.
3. No cartão **Animation**, «+ Add» → a nova fica destacada. Errado: o destaque fica na de antes.
4. Align ligado, Follow ligado, escale pela quina, Ctrl+Z → folha e Pivot voltam juntos.
