# Handoff — linha `TrelloUITeste`, blocos 3–7 e T54–T56 (02/11/2026)

Linha de TESTE (não integrada no main); o que sobrevive é `docs/CartoesUI`. Worktree `Worktrees/line-TrelloUITeste`, branch `line/TrelloUITeste`. Nunca houve push. Decisões: [01](../01_decisoes_do_dono.md) · plano/provas: [04](../04_plano_de_construcao.md) · código: [03](../03_mapa_do_codigo.md).

**Commits da janela:** `git log --oneline e0435f8831..HEAD` (11 commits, `7c3e264640` … `1cd71e99a7`, 75 ficheiros, +3614/−649).

**Smoke do dono: OK em 02/11 para tudo abaixo.**

## Feito

| item | commits | testes principais (por nome) | foto (`maquete/`) |
|---|---|---|---|
| T29, 1.ª metade — corpos dos componentes com a cara do cartão (rótulo em cima com unidade, campo Figma, segmentos pelo conteúdo) | `7c3e264640`, `f8f1553d8f` | `os_corpos_dos_componentes_em_cartoes`, `os_componentes_em_cartoes`, `card_field` | `t29_corpos_antes_depois.png` |
| Bloco 3 — estilos de cor (busca, nome inteiro, ligação viaja no colar/duplicar, cena 5 «Edit style»; acabamento: lista abaixo do cabeçalho, rótulo em 2 linhas, W·H) | `fee68cd4e1`, `f8f1553d8f` | `editing_a_style_recolours_every_linked_layer` e vizinhos (04, D28/D30) | `d30_estilos_ligados.png` |
| Bloco 5 T45–T49 — alfinete gravado por `StableId`+nome, animações de entrada/saída, alfinete+corda no Inspector (Transform), corda que contornava (T48, revogada pela T50), T49 iPad só registado | `201d788a10`, `3a66f477cc` | `a_pin_and_its_rope_survive_reopening_by_stable_id_and_name`, `a_section_that_leaves_or_enters_with_the_selection_never_jumps`, `a_loose_card_hidden_by_the_selection_fades_out_and_back_in`, `a_released_rope_reels_into_its_object`, `the_inspector_transform_card_pins_to_its_object_and_edits_it`, `the_rope_goes_around_another_loose_card` | `b5_alfinete_gravado_e_inspector.png` |
| Bloco 6 T50–T53 — corda por cima dos soltos, altura do solto com a aba atrás, relógio da corda por painel, corda só rompe ao encaixar | `479a31b14a` | `a_rope_segment_over_another_loose_card_gets_its_own_tone`, `a_loose_card_stays_on_the_canvas_when_its_panel_tab_is_behind`, `every_panel_rope_sags_under_the_real_clock`, `a_rope_survives_the_column_until_the_card_docks`, `a_rope_reels_in_when_the_card_docks_on_mouse_up` | `b6_smoke_curas.png` |
| Gates da casa (rótulo do catálogo numa linha, `IK`→`CARTAO_IK` no teste do solto) | `70f5d48825` | `architecture_every_editable_field_is_a_channel`, `node_id_collisions::a_node_id_name_has_one_value` | — |
| T54 — cópias soltas são instâncias: a 2.ª cópia de uma secção presa prende-se a outro objecto sem reescrever a 1.ª nem a corda | `cb453165a3` | `o_bloco7_duas_copias::{a_second_transform_card_pins_to_another_object_and_the_first_keeps_its_rope, two_ik_cards_pin_to_two_bones}`, `card_loose_ops_tests::two_loose_copies_of_a_section_survive_reopening_with_their_own_pins` | — |
| T55 — todo solto anda com a mão (outra classe escolhida, aba atrás) | `50106b8d7b` | `o_bloco5_abertos::every_pinned_loose_card_moves_with_another_class_and_its_tab_behind` | — |
| T56 — Physics Body, dois campos por linha | `b224a064a0`, `1cd71e99a7` | `the_physics_body_card_puts_two_fields_per_row`, `rows_par::tests::outside_a_card_a_pair_paints_the_two_rows_of_today`, `cards_corpo::tests::a_single_field_takes_the_letter_of_its_last_word` | `t56_fisica_dois_por_linha.png` |

Cenas de smoke: `PH2D_VEC_CARTOES_SMOKE=1..4`, `PH2D_VEC_BONE_SMOKE=8`, `PH2D_CARTAO_CORDA_SMOKE=1` (comandos e `cd` completos em 04).

## Contraprovas

Bloco 5: 6/6. Blocos 6, T54, T55, T56: uma por cura (a cura tirada ⇒ VERMELHO, reposta + `touch` ⇒ verde) — tabela por item em [04](../04_plano_de_construcao.md) (blocos 5, 6, 7, T55, T56).

## Medições

- Corda: 7,28 µs/quadro para contornar (T48, sonda `mede_o_custo_de_contornar`); T48 foi REVOGADA pela T50, o número só vale como custo histórico.
- Quadro do hero em depuração dura até 0,1 s (tecto do `dt`): a mola fecha ~46 % num quadro de teste (214 de 467 px). O «um quadro» dos testes do hero é até 0,1 s.
- Travamento com Scale X enorme: NÃO reproduzido com a placa livre (até 1e9 ≤ 30 ms/quadro; ≥1e10 os quadros esperam a placa; congela a 1e13). Tabela no bloco 6 de 04.

## ABERTO

1. **T29, 2.ª metade** (contagem em `sections/`): caixas de seleção 22 (`paint_check_row`) + 40 (`Checkbox`), listas suspensas ~109, `property_label_row` 11, avisos 123 (frases, já quebram), texto direto 76, número direto 8, grupos de cor 4.
2. O alfinete do Inspector existe só para o Transform; os outros cartões pedem um retrato por família.
3. A cópia 2 de um solto não herda tema/notas/dobra (chaveados pelo id da secção/cartão).
4. iPad, pressão longa na borda do solto (T49): sem código; o que falta está na T49 de 01.
5. Travamento com Scale X: repetir sem contenção de placa (sem outra linha na GPU).
6. A corda só se pinta por cima dos soltos do SEU painel; solto de painel pintado depois ainda a cobre.
7. Vermelhos pré-existentes dos gates da casa, a tratar na mudança definitiva: 16 no `ph2d-editor-core` (a lista NÃO ficou escrita no commit `70f5d48825` nem em 04 — reproduzir com os gates `architecture_*_loc_cap`/editor-core e registar) + 16 antigos de `ph2d-panel-skeleton/tests/seam.rs`.
8. Abertos antigos ainda válidos: dobra do cartão Tool não reproduzida; Skin não aparece com osso no Transform; as listas de ações do Smart Bone e os pop-ups de Tip abertos de um cartão preso leem o osso em foco; atalho de teclado logo após clicar num cartão preso vai ao osso preso.
9. Ferramenta fora da linha: `.claude/hooks/prazo-do-agente.sh:17` monta a pasta com `git rev-parse --git-common-dir` sem `--path-format=absolute`; agente em subpasta cria `./.git/ph2d-agentes`. Há um perdido em `docs/CartoesUI/.git/ph2d-agentes/` (o git ignora pastas `.git` aninhadas, por isso `git status` não o mostra; NÃO apagado — havia agentes a correr). Apagar com nenhum agente ativo.

## Regras aprendidas

- A shell não cresce (catraca `the_shell_only_shrinks`); `panel/cards.rs` nunca cresce.
- Crate do esqueleto no teto ⇒ código novo em `ph2d-card-rope`.
- «Um quadro» nos testes do hero = até 0,1 s de `dt`; meça o estado, não presuma um passo de 16 ms.

### Os vermelhos ANTIGOS de `ph2d-editor-core` (`--tests`, já vermelhos na base `e0435f8831`, medido 02/11 numa worktree à parte)

`node_id_collisions::a_node_id_name_has_one_value` foi curado em `70f5d48825`; os restantes 16 continuam:

- `a_caixa_de_numero_nao_corta_o_numero::um_decimal_que_nao_cabe_sai_arredondado_e_inteiro`
- `architecture_claude_md_cabe_no_orcamento::o_claude_md_cabe_no_orcamento`
- `architecture_panel_loc_cap::panel_files_under_loc_cap`
- `architecture_panel_loc_cap::panel_functions_under_loc_cap`
- `architecture_the_foundation_modules_form_a_dag::the_foundation_modules_form_a_dag`
- `architecture_widget_loc_cap::widget_primitives_under_loc_cap`
- `architecture_widget_mod_in_sync::widget_mod_block_in_sync_with_folder`
- `architecture_widget_showcase_coverage::every_widget_is_shown_or_explicitly_opted_out`
- `architecture_workspace_file_loc_cap::workspace_src_files_under_loc_cap`
- `every_frame_goes_through_the_theme_door::every_frame_goes_through_the_theme_door`
- `every_frame_goes_through_the_theme_door::no_stale_frame_markers`
- `hr12_widgets_a11y::every_widget_file_wires_a11y`
- `node_id_collisions::every_non_literal_hash_is_named`
- `no_label_of_this_crate_is_written_in_the_painter::every_label_this_crate_paints_comes_from_the_string_table`
- `no_magic_numeric::no_magic_numeric_in_widget_or_screens`
- `no_row_label_carries_its_own_unit::no_row_label_carries_its_own_unit`

Mais o censo `toast_reading` (lib) em `crates/ph2d-app-kanban/src/bg/scene.rs:39` (TrelloClone D13, `fa4165be1c`).
