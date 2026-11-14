# HANDOFF de integração — `line/GameEngineFuncional`, 13/11 (GEF 06 Projectile Motion · caixa em moldura · ícone da Hierarquia)

Leitor: o agente integrador. Branch `line/GameEngineFuncional`, base `4f46a3573c` (= tip do `main`, merge-base). **56 commits, 287 arquivos** (+11 544 / −2 144). Nada integrado, nenhum push. Suíte completa e `nextest-impacted` NÃO correram (regra do dono 23/10): são do integrador.

**Cobre TODO o intervalo não integrado.** A última integração da GEF no `main` é a de 11/11; o Top-Down (12/11) e os adendos NÃO foram integrados. Não repito o que eles dizem, linko:
- GEF 05 Top-Down Player (cena 24): [`…2026-11-12_TOPDOWN.md`](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-12_TOPDOWN.md) (§1–§7 e, no fim dele, o «Adendo — Rodada da física»).
- Adendos dos abertos / smoke / últimos abertos de 12/11: [`…_TOPDOWN_ADENDOS.md`](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-12_TOPDOWN_ADENDOS.md).
- Continuações desta onda (substituídas por este): [`06_PROJECTILE`](HANDOFF_CONTINUACAO_2026-11-13_06_PROJECTILE.md) · [`13b_CORRIGIR_TUDO`](HANDOFF_CONTINUACAO_2026-11-13b_CORRIGIR_TUDO.md). Ficha: [`06_projectile.md`](../componentes/06_projectile.md).

Smokes do dono, todos OK em 13/11: cena 25 (Projectile), «Checkbox ok», «smoke OK» do ícone da Hierarquia.

## 1. O que a linha entregou

1. **Top-Down Player** (cena 24) — ver handoff linkado.
2. **Física/pulo, rodada 12–13/11** — Bugs #11–#14 (wall jump, teto baixo, teto no tique do arranque, Snap, atrito na parede), `world_steps` do projeto, degrau único (`WalkConfig::step_limit`), tique dividido (`tick_gravity_scale`), índice de consultas incremental, curva do pulo no terreno, crate-folha nova `ph2d-physics-smoke-jump`. Mecanismo: [adendo da física](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-12_TOPDOWN.md) (commit `04e6b30234`) e [BUGS_physics](../../Physics/BUGS_physics.md).
3. **GEF 06 Projectile Motion** — cartão versão B (Speed/Gravity/Range/Faces path; caixas Bounce e Homing que gravam números; Target com conta-gotas; dobra «Speed change»); cena `PH2D_GAME_SMOKE=25`; Homing só vira (rapidez constante); arrastar objeto da Hierarquia para o Target da bala e da câmera; cena 25 com Floor/Back Wall sólidos, Missile sem Max Speed.
4. **CartoesUI — caixa de marcar do app inteiro numa moldura à Godot**, rótulo em cima, losango de keyframe acima do canto superior direito (dono 13/11).
5. **Hierarquia** — clique sem aproximar seleciona; o ícone de tipo da linha seleciona com um clique e arrasta como o nome; largar fora do painel solta a linha.

## 2. Commits por grupo (`git log --oneline 4f46a3573c..HEAD`)

- **G1 Top-Down:** `3a83b2bac6` `2929ec4948` `e0b66565dd` `1188f2196e` `486d1da055` `b6f0300a6c` `515c5498e4` `8571861b96` (+ `3ee0bd34f4` handoff; `c6c0848c00` horário da cena 24; `1979ca5c5d` numeração do roteiro CONTADA; `9c1f51706b` collision-surface sonda).
- **G2 Física/pulo:** `f3a92736ab` `ced6a4ac0a` `f591d5a3a2` `ef83e62d9b` `adcd392fc7` `7ac5a60521` `48afab6f40` `e175a655da` `b4d52331de` `092c7995a6` `e20eaea286` `3b1060f92d` `15429fc87f` `536b62470b` `b21a6dd3ec` `e2d1f616d4` `895cfb7883` `937f4ce952` `3173bc8a57` `eb83c15600` `50f6a8f6fa` `62c9ff8e9c` `a9430052cd` `e13112fe92` `a7ea182ff4` `eb729a2253` `73f18304eb` `f820043427` `1898193799` `8e2ae5e662` `47b3817e05` `ba4376222a` `2e0aa6840c` `0ec3202b6f` · adendo `04e6b30234`.
- **G3 Projectile:** `5142bb156b` (cartão B) · `96c68552ba` (cena 25, Target mostra o nome) · `d59d9edf5b` (ficha/plano/handoff) · `52aed2545e` + `0c7245ae80` (Homing só vira; fmt) · `353455239c` (arrastar para o Target; cena 25 Floor/Back Wall; Missile sem Max Speed) · `ff1189e46c` (handoff da onda b).
- **G4 Caixa em moldura:** `249611a86a` (central) · `fd82251aa1` (três sítios na conta antiga + cena VEC_CARTOES=10) · `89a6def873` (foto Pixel Art) · `ad4ce55542` (import sem uso).
- **G5 Hierarquia:** `534e826c69` · `839ca22c53` · `08fc2eeac2`.

## 3. Superfície de colisão

- **Schema, DELTA 0:** `PROJECT_SCHEMA` = 237 (`shells/desktop/src/project_schema.rs:459`); `git diff 4f46a3573c..HEAD -- '*schema*'` vazio. Nenhum campo gravado novo (`KinematicState`, `AlvoArmado` e `ObjectRef{field}` são estado de execução/sessão). Reconte do `main` do dia: `python3 scripts/schema-recount.py`.
- **Cargo.lock:** `+name` só `ph2d-game-smoke-fotos` e `ph2d-physics-smoke-jump` (internas, sem dep externa nova).
- **Contratos §6:** nodes/tools intocados. ADR: nenhum criado.
- **Ponte da física** (`ph2d-physics-ecs/src/bridge*`, `physics_apply.rs`, `ph2d-platformer`): editada com autorização do dono (GEF02 fechada). Rode `collision-surface.sh` (caminho absoluto) contra o `main` do dia.
- **Cross-module, ATENÇÃO — caixa de marcar:** `ph2d-editor-core` `panel/cards_corpo_marca.rs` (NOVO: `check_em`, `pede`, `check_solto`), `cards_rotulos::tem_rotulo` (Check abre a banda do rótulo), `cards_linhas`, `cards_compacta`, `cards_linhas_extra`, `property_row/marca.rs` (`paint_check_rows` mede por `pede_marca`); i18n `chrome.checkbox.on`; painter-layers `cards_linhas::par_com_inerte`; inspector `cards_sprite_pivot::caixa_do_mirror`, `ordering_cartao`; `corpo::caixa_ao_lado_do_rotulo` REMOVIDA (única chamadora era o Y-Sort).
- **Hierarquia:** `WidgetStore::hierarchy_row_grip` (`ph2d-editor-core` `store_hierarchy.rs`), `pointer_down.rs` (+4), `pointer_up.rs` (early return virou else-if), `ph2d-panel-hierarchy` `event.rs`/`lib.rs`, shell `asset_drag_wire.rs` (1 por 1 linha).

## 4. Contadores que SOMAM entre linhas — o integrador RECONTA (DELTA da linha)

| Contador | Estado na linha | Nota |
|---|---|---|
| Próxima cena `PH2D_GAME_SMOKE` | **26** (6..23 Input Map, 24 Top-Down, 25 Projectile) | numeração CONTADA: `input_map::ULTIMA`, `topdown::CENA == ULTIMA+1` (const assert); a 25 é a última (`smoke_tests`). Outra linha com cena 24/25 reconta |
| Carga de comandos do inspector | 104 → **105** | conta-gotas do Target = capacidade nova |
| Balão `INSP_RAY_ORIGIN_X` | controlo-balão da cena 25 | colide se outra linha usar o mesmo id |
| `ph2d-app-components` LOC | 80 848 ≤ tecto **80 869** | cortando comentário histórico (`asset_drop`) |
| `ph2d-game-smoke-fotos` | 1 075 / tecto **1 077** | crate NOVA (12/11) |
| `ph2d-physics-smoke-jump` | 1 158 / tecto **1 258** | crate NOVA (13/11); em `TETOS_DAS_CRATES_DA_FAMILIA` |
| `ph2d-physics-smoke-scenes` | 18 061 / tecto **18 422** (era 19 008) | desceu; outra linha que some cenas ali reconta |
| `ph2d-app-game` | tecto 2 093 (medido 1 993) | 12/11 |
| Shell | `main.rs` 997 → 984 (`FILE_OVERAGE_OK` 996 → 984); janela do jogo 214 → 77; −2 líquidas na onda 06 | **tecto da shell NÃO mexido: reconte pelo `main`** |
| `FACTORY_ART` | 9 → 10 | chave `u32::MAX-11` em `hero_topdown.rs` |
| Cena `PH2D_VEC_CARTOES_SMOKE=10` | Material solto em (620, **310**) | cobria o Ordering (defeito anterior) |
| `ph2d-ecs` componentes | DELTA 0 | o script de colisão pode marcar SONDA CEGA em render/script: defeito do script, curado em `9c1f51706b` |

## 5. Gates/testes novos ou alterados (por nome)

- **Projectile:** `seam_projectile` (8, `ph2d-panel-inspector`); testes de `projectile_inspector` (10); testes de `cena_projectile`; `jogo_projectile` (`shells/game-scenes`: Missile a 0,55 m do Target, Arc > 2 m, `o_target_da_bala_recebe_um_objeto_arrastado`, as cinco balas dentro da arena e o Arc no Floor); `ph2d-projectile` `lib_tests.rs`: `a_perseguicao_vira_sem_mudar_a_rapidez`, `a_perseguicao_nao_mexe_na_rapidez_do_avanco`, `o_missil_lancado_para_baixo_da_a_volta_e_acerta_sem_tecto`. `architecture_every_editable_field_is_a_channel` (`projectile::numero`/`voo` = Canal; `ricochete`/`perseguicao` = NaoAnimavel); `every_word…` lista `projectile.rs`. `game-scenes` ganhou dev-dep `ph2d-panel-inspector`.
- **Caixa:** `a_check_row_is_a_frame_under_its_name_with_the_key_mark_above_its_corner` (`cards_compacta_tests.rs`).
- **Hierarquia** (`crates/ph2d-panel-hierarchy/tests/it/one_click_on_an_animated_sprite_row.rs`, 8): `um_clique_no_nome_de_um_animated_sprite_no_fundo_da_lista`, `uma_pressao_de_450_ms_ainda_seleciona`, `um_clique_com_6_px_de_tremor_ainda_seleciona`, `um_clique_sem_aproximar_ainda_seleciona`, `um_clique_no_icone_seleciona`, `arrastar_pelo_icone_arrasta_o_objeto`, `duplo_clique_no_icone_ainda_enquadra`, `largar_fora_do_painel_solta_a_linha`.
- **Física (12–13/11), principais:** `os_passos_do_projeto_sao_os_da_lei`, `the_leg_refuses_an_outer_foot_on_a_step_steeper_than_the_limit`, `a_curva_recusa_o_degrau_que_a_perna_recusa`, `every_ceiling_in_reach_is_bonked_once_at_every_jump_height`, `a_ceiling_inside_the_takeoff_tick_is_bonked_with_the_launch_speed`, `a_alca_poe_o_apice_onde_foi_pedido`, `a_curva_desce_ao_chao_de_baixo` / `_pousa_na_plataforma_mais_alta` / `_para_na_parede`, `o_numero_da_alca_e_o_do_campo`, `os_passos_do_mundo_de_fabrica_sao_os_da_lei`; os censos da shell (régua, alça, every_word) agora REPROVAM braço do roteador sem corpo.
- **Numeração:** `as_cenas_do_input_map_fecham_o_roteiro_pela_ordem` (a aba fecha em `ULTIMA`, o Top-Down fecha o roteiro).

## 6. Provas de mutação (vistas vermelhas)

Projectile: cartão + sync do Target + teto de velocidade do Missile 13/13 · Homing só vira 3/3 · arrastar para o Target 3/3 · `projectile_target_slot_pick` só com Homing 3/3. Caixa em moldura: 3/3. Hierarquia (ícone/clique/largar fora): feitas por commit. Top-Down: cartão 10/10, cena 2/2, foco 5/5, arte 4/4, acabamento 6/6. Numeração: `CENAS = CENA + 1` ⇒ vermelho; 19.º código na aba ⇒ não compila. Pasta nova dos censos tirada ⇒ vermelho. Física: lei sem passos, arco sem iterações, readout `{:.0}`, fábrica 4 ⇒ vermelho (detalhe no adendo e no BUGS_physics).

## 7. O que um merge pode partir

- **A altura da linha de marcar mudou em TODOS os cartões com caixa** (ganhou a banda do rótulo). Gate de altura/posição de caixa de outra linha reprova; cartão NOVO de outra linha com checkbox tem de usar a geometria nova (posicionar pela linha normal; ver `cards_sprite_pivot`, `ordering_cartao`, `par_com_inerte`). `caixa_ao_lado_do_rotulo` não existe mais: chamador novo não compila. Ficam caixa simples (o item É a caixa): `audio_set check_in`, chips de barra, `caixa_de_cabecalho`, `rich_editor` checklist, `command_palette`, `date_picker`, `board_private_paint`, `options_card` do `ph2d-panel-code`. `PH2D_CARDS=0` sem mudança.
- `DropTarget::CameraTarget` → `ObjectRef{entity_bits, field: RefField::{CameraFollow, Homing}}` e `DropAction::SetCameraTarget` → `SetObjectRef{…, field, …}`; escrita em `asset_drop::edicao_do_campo`. Quem casa os nomes antigos quebra.
- Rótulos de catálogo renomeados: (12/11) Acceleration→Speed up, Deceleration→Slow down, Turn Speed→Turn, Push Recovery→Hit recovery; (13/11) Face Velocity→Faces path, Homing→Pull.
- No Play a bala de documento sai (RunOnCopy, `removable_in_run`) e o Stop devolve: regra da casa, não defeito.
- Hierarquia: `pointer_up.rs` e `store_hierarchy.rs` do `ph2d-editor-core`; outra linha que toque o arrasto de linha conflita.
- Física: `bridge/player*.rs`, `ceiling_fact.rs`, `player_seat.rs`, `ph2d-platformer` (`kinematic.rs`, `walk_brink.rs`, `jump_config.rs`); fixturas que dependiam do atrito do player na parede mudam. Cenas 110/111/água/ledge re-derivadas.
- Shell: `janela_do_jogo_smoke.rs`, `asset_drag_wire.rs`, `main.rs`. `ph2d-i18n`: `projectile.rs` (novo), `topdown.rs`, `cartoes.rs` (`chrome.checkbox.on`), `component_catalog.rs`.

## 8. Fecho (bateria)

PENDENTE — preenchido pela janela principal

## 9. Smokes (comando inteiro)

1. Projectile: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop && PH2D_GAME_SMOKE=25 ./target/smoke/ph2d-host-desktop` — bala selecionada; Bounce/Homing em caixas com moldura e nome em cima; arraste «Target» da Hierarquia para o campo Target; Play: Missile acerta, Arc pousa no Floor, Bouncy volta da parede. Errado: Missile em espiral, bala fora da arena, caixa cobrindo o campo ao lado.
2. Top-Down: o mesmo comando com `PH2D_GAME_SMOKE=24`.
3. Caixa em outros cartões: `PH2D_VEC_CARTOES_SMOKE=10` (Material não cobre o Ordering).
4. Hierarquia: numa cena com Animated Sprite, clique no ÍCONE da linha (seleciona), arraste (arrasta o objeto), largue fora do painel (a linha solta).
5. Gates: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && bash scripts/cargo-test-narrow.sh ph2d-panel-hierarchy um_clique` e `bash scripts/cargo-test-narrow.sh ph2d-host-game-scenes jogo_projectile`.

## 10. ABERTO

- **Ray Sensor** (próxima): fila `docs/GameEngineFuncional/23_refundacao_plano.md`, «Corpo e movimento»; protocolo `estudo_secao.py`, ficha 07, versões A/B/C mostradas ao dono ANTES de construir.
- **Não fotografado após a mudança da caixa** (nenhuma cena os abre): tela Try it/Controls do Input Map (`ph2d-input-map-editor` `view_try`/`controls.rs`, caminho central; é da line-InputMap, que confere ao integrar), cartões Vector Effects/States/Pattern, Painter adjustments/Color ramp.
- **Caneta Wacom no Wayland nativo** para a Hierarquia: não provável em Xvfb; só o dono.
- Tecto da shell e `main.rs` 984/600: do integrador (§4).
- Fecho da linha (§8): a janela principal preenche.
