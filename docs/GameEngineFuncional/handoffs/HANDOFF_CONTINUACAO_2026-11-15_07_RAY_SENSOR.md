# Handoff de continuação — GEF 07 Ray Sensor (15/11)

Linha `line/GameEngineFuncional`, reaberta do `main` `7b3dd983f7`. Ficha: [07_ray_sensor.md](../componentes/07_ray_sensor.md).
**Não integrado, não pushado. Smoke do dono: PENDENTE.**

## Decisão do dono (15/11)

Versão **B** da crítica, e: *«assim como o collision shape deixou de ser componente e passou a ser objeto que aparece
na hierarquia, também deve ser o Ray»*. A alça da C não foi pedida (não construída). Memória:
`project-memory/project_ray_sensor_is_a_child_object_2026_11_15.md`.

## Feito (commits `7d6a96ef15` ficha · `0ff6b66e20` obra · `78eed6f0af` portões · o do fecho)

- **Motor** (`ph2d-physics-ecs`): `RaySensor { angle (graus locais), reach, layer }`, `dir_local()`; `raio_em_mundo`
  sobe ao corpo dono (`owner_body`), exclui-o e nasce da pose viva + desvio do filho. Aceita ainda o raio na própria
  entidade do corpo (testes antigos). `pendura_raio`. `PROJECT_SCHEMA` 242 → 243 (sem leitor velho).
- **Catálogo:** RaySensor `il` (intrinsic): fora da paleta (`PORTAS` 7) e do Add Component (`ENTRADAS` 62); campos
  `angle` (K::Angle) · `reach` · `layer`; i18n `component.field.ray_sensor.0..2` = Angle/Reach/Layer.
- **Add ▸ Ray Sensor** (`ph2d-game-dicas::add::RAY_SENSOR`, `object_add.rs`): filho do selecionado na origem dele;
  sem seleção, na raiz. Dica e palavras de busca.
- **Hierarquia:** `ObjectType::RaySensor`, `IconId::Probe`, `obj-ray-sensor` = oklch(0,76/0,43 0,15 140) — matiz
  escolhido por varredura (40/70/90/175/250/310/335 reprovam o ΔE ≥ 0,05 em algum tema; 110 e 140 passam).
- **Cartão B** (`sections/ray.rs`): queixa · «Sees X, at Y» · relógio · Angle (dica) · Reach | Layer · botões ·
  Sees · grupo Signals (`INSP_RAY_FOLD_SIGNALS`, aberto). Queixa nova `CamadaVazia` (`ray_inspector::camada_vazia`,
  só `&Collider` na consulta — com `Option<&CollisionOverride>` o `try_query` devolve `None` sem exceções na cena e o
  aviso mentia; a foto apanhou). `SemDireccao` saiu.
- **O objeto raio não mostra a face «Add Physics Body»** (`paint_frame.rs`, `raio_solto`).
- **Cena `PH2D_GAME_SMOKE=26`** (`ph2d-physics-scene/src/cena_ray.rs`, `HORARIO_26`), fotos `componentes/fotos_07/`.
- **Catraca `ph2d-app-game`:** os dados dos níveis 1–2 mudaram-se para `ph2d-physics-scene::niveis` (reexportados
  em `smoke`): 2902 → abaixo de 2869. `color.rs` 701 → 700 (um comentário encurtado).

## Gates (todos verdes por nome; lote das 14 crates verde fora os 6 curados)

`um_raio_filho_exclui_o_corpo_dono_e_segue_a_pose_viva` (+20 `ray_` do physics-ecs), `a_camada_vazia_so_fala_quando_e_certo`
(+15 `ray_` do app-components), `seam_ray` (4: foco sob o dedo × cartões on/off, digitar 45 → `Angle(45)`, Signals abre e
fecha, dica do Angle), `o_angle_e_o_reach_do_raio_animam` (canal do catálogo, K::Angle), `o_raio_nasce_filho_do_seleccionado_a_olhar_para_baixo`,
`cena_ray` (2), `cenas_do_smoke`, `jogo_object_add`, `cada_tipo_tem_cor_propria_e_legivel`, `the_shell_only_shrinks…`,
`workspace_src_files_under_loc_cap`, `project_schema`, `file_loc_caps`. fmt + clippy `-D warnings` das crates tocadas: limpo.

**Mutação:** 10/10 sangram (a 10, `K::Angle`→`K::Scalar`, sobrevivia e ganhou o assert do tipo do campo).

## Segunda onda (15/11, pedidos do dono)

- **«Turns with facing»** (opção, desligada; ligada na cena 26), commit `1320f0fa5c`: campo 4 do `RaySensor`; o raio
  espelha ângulo, desvio e rotação relativa pelo `PlayerState::facing` do corpo dono (`player_state`, escrito antes
  do `cast_ray_sensors` no mesmo tique). Gates `com_a_opcao_o_raio_vira_com_o_lado_do_personagem`,
  `a_caixa_turns_with_facing_escreve_o_contrario_do_ecra`.
- **Play no editor dá o teclado ao jogo**, commit `7606874292`: a sessão Animating da Timeline tapava a cena no quadro
  da subida do relógio (a corrida só a fecha depois, no mesmo quadro). Cura: `Tapa::corrida`
  (`ph2d_transport::corrida`) e `toma_o_teclado` (o Play solta o campo focado). Gates em
  `shells/game-scenes/tests/it/jogo_play_da_o_teclado.rs` (4), mutação 3/3. Doc 06 §9.

## Terceira onda: o filtro (versão C)

Commits `8fa2181ffc` (motor e vocabulário) · `c6a6d7f98e` (painel) · `1d4bd82003` (canvas).

- **Painel:** `sections/ray_filtro.rs` (grupo `INSP_RAY_FOLD_FILTER`, fechado de fábrica, `resumo()`); ids em
  `ids/inspector_ray.rs` (`INSP_RAY_IGNORE_BODY` … `INSP_RAY_ALL_HITS`, `INSP_RAY_ONLY_SEE[3]`, `INSP_RAY_SEE_TAG` +
  `INSP_RAY_SEE_TAG_OPT[64]`, gerados por `inspector_physics_body::tabela::<N>`); popover diferido
  `PENDING_RAY_TAG_DD` (`popovers_tags.rs`); a árvore de tags pela porta `nav_tag_row::opcoes_da_arvore` (a mesma do
  Nav Agent). Eventos em `event_ray::clique_do_filtro` (caixa = contrário do snapshot). «Only see» é `paint_choice_row`
  (segmentos) e não lista suspensa: a porta da casa para escolhas.
- **Canvas:** `ProbeMark::more_hits` (`MaisAcertos`, `MAX_TIQUES_DO_RAIO = 8`, tecto do desenho); `ray_sensors.rs`
  desenha all_hits até ao bloqueio.
- **Gates verdes por nome:** `seam_ray` (9, +4 novos), `o_resumo_do_filter_muda_com_as_opcoes`,
  `o_cartao_pinta_a_chave_do_catalogo` (era o vermelho), `architecture_every_editable_field_is_a_channel` (+2 entradas
  Canal em POR_FUNCAO), `o_inspector_armado`, `versao_c_*` (9), `com_all_hits_ha_um_tique_em_cada_acerto`.
- **Mutação 9/9 sangram:** IgnoreBody sem `!` · Filter nasce aberto · tag/Walls block sem o `if` · pintor sem
  `more_hits` · ponte sem `com_mais_acertos` · fim sem o bloqueio · resumo sempre «Ignores me» · opções da tag pelo
  array do Nav · AllHits sem `!`.
- **Cena 26 com o filtro** (`7f318581fa`): `Crate` (−2,1 ; 1,1) e `Enemy` (−0,37 ; 2,1) no alto, fora da linha do
  Wall Ray; o `Enemy` leva a tag «Enemy» criada na árvore do projecto pela porta nova `smoke::Portas::marca`
  (`ph2d_app_components::tags_doc::marca`; a shell liga-a sobre `cx.tags` em `janela_do_jogo.rs`, o espelho
  `shells/game-scenes/tests/it/comp.rs` devolve a árvore por `montar_com_tags`). «Eye Ray» = terceiro raio de
  `RAIOS` (30°, 7 m, Only tag, Walls block off); «Eye Light» na fila das luzes (x = 0). `HORARIO_26` passou a andar
  ao 360 (era 120): o herói fica de pé, a luz acende e os avisos somem antes de andar (com 120 o olho perdia o
  inimigo aos ~0,8 m e a foto apanhava `enemy_gone`). O selecionado continua o Wall Ray.
- **Foto `ray_filter`** (`cena_ray::{Foto, NOMES, foto, gavetas}`): seleciona o Eye Ray e abre o Filter em vez de
  Signals; registada em `ph2d_game_smoke_fotos::nomes::LEITORES` (6) e nos dois censos de `jogo_nomes_do_smoke`.
- **Fotos** (`docs/GameEngineFuncional/componentes/fotos_07/`): `26_cartao.png` (Wall Ray, Filter fechado com
  «Ignores me»), `26_filter.png` (`PH2D_GAME_SMOKE_FOTO=ray_filter`), `26_play_eye.png` (`PH2D_GAME_SMOKE_DRIVE=editor`,
  espera 5: de pé, Eye Light acesa, raio a acabar no Enemy através da Crate). Olhadas: sem rótulo cortado nem
  sobreposição; caixas «On» em moldura como Turns with facing.
- **Fecho** (`0ec52ac519`): `cargo fmt` das crates; `clippy --all-targets -D warnings` verde em ph2d-physics,
  -physics-ecs, editor-core, panel-inspector, app-components, app-physics, physics-scene, app-game, component-desc,
  i18n, host-desktop, game-smoke-fotos, host-game-scenes — curados: `PointQuery` sem uso (`cast_query.rs`), doc e
  `#[must_use]` trocados entre `com_mais_acertos`/`na_camada` (`player_view.rs`), dois `if` colapsados
  (`ray_sensors.rs`). LOC: `workspace_src_files_under_loc_cap` reprovava `inspector_game.rs` 709 e
  `component_catalog.rs` 704 — o Filter foi para a irmã `ph2d-i18n/src/inspector_ray.rs` (+ 11.ª tabela do
  `every_word_this_panel_shows…`) e os canais `ray_sensor.4..11` para `component_catalog_forma.rs` (670 e 696).
- **Gates verdes por nome:** `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`, `file_loc_caps`,
  `architecture_every_editable_field_is_a_channel`, `o_cartao_pinta_a_chave_do_catalogo`, `seam_ray` (9),
  `versao_c` (9), `ray_` (physics-ecs 30, app-components 15), `project_schema`, `jogo_ray`, `jogo_nomes_do_smoke` (4),
  `cena_ray` (4), `every_word_this_panel_shows`, `every_word_this_catalog`, `o_tutorial` (6), `smoke` (app-game 5).
- **Binário smoke** (2.ª corrida): ``Finished `smoke` profile [optimized] target(s) in 0.23s``.

## Aberto

- Smoke do dono da cena 26 (passos na conversa de 15/11).
- `PH2D_RAY_SMOKE=1` (cena antiga) segue com olhos na raiz; não foi refeita como filhos.
- Próximo item da fila: **Motion Input** (`23_refundacao_plano.md`, «Corpo e movimento»).
- Fecho da linha (§1.5.9) e a entrada do §5 do CLAUDE.md: na integração.
