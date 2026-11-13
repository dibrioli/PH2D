# HANDOFF de continuação — GEF «corrigir tudo» (13/11/2026, onda b)

Linha `line/GameEngineFuncional`, worktree `Worktrees/line-GameEngineFuncional`. Fecha os 5 ABERTOS do [handoff anterior](HANDOFF_CONTINUACAO_2026-11-13_06_PROJECTILE.md) (ordem do dono, 13/11). Ficha: [06_projectile.md](../componentes/06_projectile.md). Plano: [23_refundacao_plano.md](../23_refundacao_plano.md). Não integrada (a integração é do integrador, por ordem do Enio).

Commits: `52aed2545e`, `0c7245ae80` (fmt), `249611a86a`, `353455239c`, `fd82251aa1`, `89a6def873` (foto).

## 1. Checkbox do app numa MOLDURA à Godot (`249611a86a` + `fd82251aa1`)

- A linha de marcar do cartão é: rótulo em cima (o losango de keyframe na linha do rótulo, acima do canto superior direito da moldura) + moldura de campo (`paint_field_frame`) com ☑ e «On» (i18n `chrome.checkbox.on`). A linha ficou mais alta (ganhou a banda do rótulo).
- Central em `ph2d-editor-core`: `panel/cards_corpo_marca.rs` (`check_em`, `pede`, `check_solto` para item de lista), `cards_rotulos::tem_rotulo` (Check abre a banda do rótulo), `cards_linhas` / `cards_compacta` / `cards_linhas_extra`; `property_row/marca.rs` (`paint_check_rows` mede por `pede_marca`).
- **Ficam a caixa simples** (o item É a caixa): `audio_set` `check_in`, chips de barra `cards_banda`/`_tons` (Auto Key etc., já com moldura própria), `rows_par::caixa_de_cabecalho`, `rich_editor` checklist, `command_palette` header, `date_picker`, `board_private_paint`, `options_card` do `ph2d-panel-code`. `PH2D_CARDS=0` sem mudança.
- **Três sítios punham a caixa na conta antiga** (`fd82251aa1`), corrigidos posicionando pela linha normal:
  - `cards_sprite_pivot::caixa_do_mirror`: Keep in place começa em `fim_flip` e o fundo conta no cartão.
  - painter-layers `cards_linhas::par_com_inerte`: Check conta como rotulada (Same as Paper descia 17 px ao lado do Grain).
  - `ordering_cartao`: Y-Sort pela linha normal em y, fundo em yy; `corpo::caixa_ao_lado_do_rotulo` apagada (só esse chamador).
- Cena `PH2D_VEC_CARTOES_SMOKE=10`: Material solto em (620, 310) (cobria o Ordering, que cresceu com Y-Sort/Sorting Layer; defeito anterior a esta onda).
- Teste novo: `a_check_row_is_a_frame_under_its_name_with_the_key_mark_above_its_corner` (`cards_compacta_tests.rs`, 3 mutações mortas).
- Verdes por nome: editor-core, panel-inspector (16 filtros), panel-vector, pixel-slices, host-desktop, painter-layers (h6 19, `a_caixa_do_ponto_fixo`).
- Fotos: `docs/GameEngineFuncional/componentes/fotos_checkbox/` — `antes_`/`depois_`/`lado_` de projectile, physics, sprite, topdown; `depois_`/`corte_` de sprite_keep_ao_lado, sprite_keep_largura, painter_wet, painter_digital, painter_grain, forma_look, vector_camera, vector_resize_box, vector_ordering; `depois_pixel_slices`.
- **NÃO verificado** (nenhuma cena os abre): tela Try it/Controls do Input Map (`ph2d-input-map-editor` `view_try`/`controls.rs` usa `Widget::Check` pelo caminho central; é da line-InputMap), cartões Vector Effects/States/Pattern, Painter adjustments/Color ramp. Cartões com caixa cresceram em altura.

## 2. Homing só VIRA (`52aed2545e`)

`ph2d-projectile` `advance()`: integra avanço + gravidade; depois o Pull roda a velocidade mantendo a rapidez; bala parada fica parada. Gates (`lib_tests.rs`): `a_perseguicao_vira_sem_mudar_a_rapidez`, `a_perseguicao_nao_mexe_na_rapidez_do_avanco`, `o_missil_lancado_para_baixo_da_a_volta_e_acerta_sem_tecto` (3/3 mutações mortas: o Pull voltar a somar).

## 3. Arrastar da Hierarquia para o Target (`353455239c`)

O Target da câmera já aceitava (GEF 20 C2). Agora: `DropTarget::CameraTarget` → `ObjectRef{entity_bits, field: RefField::{CameraFollow, Homing}}`; `DropAction::SetCameraTarget` → `SetObjectRef{…, field, …}`; escrita numa tabela, `asset_drop::edicao_do_campo` (a mesma edição do conta-gotas, um passo de undo); `ph2d_panel_inspector::projectile_target_slot_pick` só com `homing_accel > 0`. Teste `shells/game-scenes` `jogo_projectile::o_target_da_bala_recebe_um_objeto_arrastado` (3/3 mutações mortas); `game-scenes` ganhou dev-dep `ph2d-panel-inspector`. `ph2d-app-components` fica sob o tecto 80 869, cortando comentário histórico do `asset_drop`.

## 4. Cena 25 (`353455239c`)

Fundo «Background» (só imagem); «Floor» (Static Body, faixa (0, -4,35), meia (7; 0,15)); «Back Wall» ((-6,85; 0), meia (0,15; 4,2)); Missile sem Max Speed (resumo None). Números do Play (180 tiques): Missile acaba a 0,55 m do Target; Straight para em x 5,05; Bouncy volta da parede de trás (x -4,88 em 3 s); Arc pousa em (1,50; -4,05) e acaba; Short Range para em x -0,90. O gate `jogo_projectile` prova as cinco dentro da arena e o Arc no Floor.

## 5. Comentários (`52aed2545e`)

`bridge/projectile.rs`, `components/health.rs`, `bridge/mortes.rs`, `bridge/health.rs` dizem a porta `removable_in_run`: no Play (corrida sobre cópia) a bala de documento sai e o Stop devolve.

## O que um merge pode partir

- **A altura da linha de marcar mudou em TODOS os cartões:** gates de altura/posição de caixa de outras linhas reprovam; a cura é posicionar pela linha normal (ver os três sítios).
- `DropTarget`/`DropAction` renomeados: quem casar `CameraTarget`/`SetCameraTarget` quebra.
- Tecto de LOC de `ph2d-app-components` (80 869, soma entre linhas).
- Cena `PH2D_VEC_CARTOES_SMOKE=10` mexida (Material em 310).

## Smoke do dono (a fazer)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop && PH2D_GAME_SMOKE=25 ./target/smoke/ph2d-host-desktop`
2. No cartão da bala: Bounce e Homing são caixas em moldura, com o nome em cima. Arraste «Target» da Hierarquia para o campo Target: ele vira o nome do objeto; Ctrl+Z volta.
3. Play: Missile (sem Max Speed) acerta o Target; Arc pousa no Floor; Bouncy volta da parede de trás.
4. Deu errado se: o Missile faz espiral/erra, uma bala sai da arena, ou a caixa de marcar de algum cartão cobre o campo ao lado.

## ABERTO

1. Smoke do dono desta onda.
2. **Ray Sensor** (fila `23_refundacao_plano.md`, «Corpo e movimento»): protocolo `estudo_secao.py` + ficha 07 + versões A/B/C ao dono antes de construir.
3. Fecho da linha ainda não feito (fmt/clippy 1×, gate batched).
4. Conferir as telas listadas como NÃO verificadas no item 1.
