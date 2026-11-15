# Handoff de continuação — GEF 07 Ray Sensor (15/11)

Linha `line/GameEngineFuncional`, reaberta do `main` `7b3dd983f7`. Ficha: [07_ray_sensor.md](../componentes/07_ray_sensor.md).
**Não integrado, não pushado. Smoke do dono de 15/11: OK (antes da auditoria). Smoke das curas: PENDENTE.**

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

## Quarta onda: a auditoria curada (15/11)

Commits `412699ec58` (curas C1–C9, U1, U2, U6, U7) · `63372c90b3` (U4, U5, caixa de texto em repouso) · `8c06704f51`
(fecho: fmt, clippy, LOC). Lista e mecanismos: [07_auditoria_2026-11-15.md](../componentes/07_auditoria_2026-11-15.md) — as 16, num bloco.

- **C8, decisão do dono (15/11): «Ignore up to»** = campo de texto + conta-gotas que nomeia o objeto mais alto da cadeia que o
  raio ignora; vazio = automático (pára no personagem — PlatformPlayer/TopDownPlayer —, senão no corpo mais alto). Lei da
  família do raio. Memória: `project-memory/project_ray_family_ignore_up_to_field_2026_11_15.md`. Porta do conta-gotas: shell LOC líquido 0.
- **C6 real, curado na família:** 13 campos de referência do catálogo (Named Tag/NameHash/Stable, incl. `Ref` de junta/polia)
  passam pelo NOME em `canais_componente` (ler/escrever/publicar/aplicar levam `RefNames`/`TagTree`).
- **C9:** o doc 06 estava errado (Space é o preview da Timeline, não o Play). `a_posse_do_teclado_segue_a_ordem_do_quadro`
  lê a ordem real do quadro no código da shell e corre as peças nessa ordem; o gate por texto fica só para `posse_do_jogo.rs` pôr `corrida` em Tapa.
- **U5 curado (`89a8c277fd`):** as 4 listas de tags (Nav, Ray, Action, Physics) oferecem TODAS as tags, sem tecto; ids por índice,
  o popover regista as linhas preguiçosamente ao abrir; «…and N more» saiu. (Antes, `63372c90b3`: 64 linhas + «…and N more», #65+ não escolhível.)
- **Cura achada nas fotos:** a caixa de texto do Inspector clássico em repouso pintava o valor com largura infinita (cortado seco sob a dica «1 listens»);
  agora `paint_text_elided` até à reserva da direita (`paint_text_input_with_buffer`, `text_input/mod.rs`); gate `a_dica_de_quem_ouve_nao_cobre_o_nome`.
  Notado aqui: no clássico (cards off) as checkboxes desligadas pareciam quadrados escuros de moldura fraca — curado em `05742878d4` (ver Aberto).
- **Gates novos:** `ray_sensor_auditoria` (5: `walls_block_com_a_origem_dentro_de_uma_parede_tapa`, `a_marca_bloqueada_nao_e_um_acerto`,
  `trocar_all_hits_no_meio_nao_inventa_arestas`, `o_raio_debaixo_da_espada_vira_com_o_heroi`, `a_familia_para_no_personagem_e_no_topo_nomeado`) ·
  `seam_ray_auditoria` (3: `o_ignore_up_to_escreve_pelo_nome_e_pelo_conta_gotas`, `cada_campo_do_catalogo_pintado_tem_o_rotulo`, `toda_linha_do_cartao_ray_tem_dica`) ·
  `jogo_ray_topo` (3) · `um_canal_de_referencia_le_e_escreve_pelo_nome` · `a_posse_do_teclado_segue_a_ordem_do_quadro` ·
  `a_lista_de_tags_nao_corta_calada` · `a_dica_de_quem_ouve_nao_cobre_o_nome` · censo U4: 8 696 → 9 336 literais (640 gerados em 13 tabelas).
- **Mutações:** motor 7/7 · C6 2/2 · C9 4/4 · UI/app 8/8 · U4 1/1 · dica 2/2 — todas sangraram.
- **Fecho:** fmt aplicado; clippy `--all-targets -D warnings` exit 0 nas 10 crates tocadas; gates por nome verdes: `ray_` (physics-ecs, app-components),
  `canais_`, `seam_ray`, `o_sufixo_do_sinal`, `every_word_this_panel_shows`, `every_word_this_catalog`, `a_seccao_ray_sensor_esta_viva`,
  `a_camada_pelo_nome_fora_do_cartao`, `os_quadros_da_colisao`, `nav_tag_row`, `o_resumo_do_filter`, `o_inspector_armado`,
  `architecture_every_editable_field_is_a_channel`, `node_id_collisions`, `the_shell_only_shrinks`, `workspace_src_files_under_loc_cap`,
  `text_input`, `ray_edits`, `o_cartao_pinta_a_chave_do_catalogo`, `architecture_tool_contract_surface`, `architecture_panel_loc_cap`, `jogo_ray`,
  `jogo_play_da_o_teclado`, `jogo_projectile`, `cenas_do_smoke`, `jogo_nomes_do_smoke`, `project_schema`, `autokey`, `cena_ray`.
- **Fotos:** `fotos_07/26_filter.png` (cartões) e `26_filter_cards_off.png` (`PH2D_CARDS=0`), por
  `bash docs/Components/ferramentas/fotografa_cena.sh "PH2D_GAME_SMOKE=26 PH2D_GAME_SMOKE_FOTO=ray_filter [PH2D_CARDS=0]" 1930 1040 <png>`.
- **Binário smoke** (2.ª corrida, `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`): ``Finished `smoke` profile [optimized] target(s) in 0.22s``.

## Sexta onda: as melhorias (RS-1..RS-12, dono 15/11)

Commits: `5757ddd28f` (motor, cartão, script, oráculo) · `dca2de0759` (cena 26 Guard+Turret, gates, cápsula, `ray_rumo.rs`) ·
`c980df2f3c` (canal da Timeline, desenho do feixe, conta-gotas no canvas, tecto da crate nova) · `0d77572e92` (fotos; tiro da Turret
0,5 m) · `5954d0bcc4` (Aim at em saltos de 20 tiques) · `2d99aaa773` (fecho fmt/clippy: `RayPath`, `DiscCandidate`).

**Motor (`crates/ph2d-physics-ecs`).** `RaySensor` deixa de ser `Copy`; campos novos NO FIM: `enabled`, `thickness` (m, diâmetro),
`cone` (graus, abertura total), `aim_at: u64` (StableId), `also_ignore: Vec<u64>`. `RayHit` ganha `shape` (a forma acertada).
UMA porta para o que um raio vê: `bridge/ray_lance.rs::lanca` (linha / círculo varrido / leque), usada por `cast_ray_sensors` E por
`PhysicsBridge::raycast` (script). Portas de física em `crates/ph2d-physics/src/world/cast_query.rs`: `cast_circle_query`,
`cast_circle_query_all` (uma consulta de cápsula + `cast_shapes` do parry por candidato), `shapes_in_disc` (`DiscCandidate`), `RayPath`.
Os sensores iteram em ordem de StableId (RS-10).

- **Cone** = linha de vista à Construct: candidatos no disco do alcance, DUAS linhas por candidato (ponto mais próximo e centro,
  presos ao cone). A 1.ª versão só com o ponto mais próximo lia um objeto meio escondido como escondido (o gate do leque apanhou).
- **Desligado** (`enabled` falso): sem lançar, sem marca, e o que via sai com On Exit nesse tique. Decisão (regra do Unreal): desligar
  fecha a aresta; senão uma luz acesa por On Enter ficava acesa.
- **RS-3 medido:** o constraint Look At no objeto do raio roda-o, mas corre DEPOIS da física (`fase_skeleton_drives` após
  `fase_physics_step`) → um quadro atrasado, e o `angle` soma por cima. Cura: campo `aim_at` lê a pose VIVA do corpo do alvo no mesmo tique
  (Transform se não há corpo). O cartão esconde Angle e Turns com facing enquanto há mira.
- **RS-4, decisão do dono:** sem tipo de material novo — o acerto leva a forma (filha Collision Shape); as tags dela são o material,
  mais fricção/restituição. Cartão «Sees Ground (Ice), at …»; script `RayHit.tags/friction/bounce`.
- **RS-5:** Signal Actions/máquina de estados NÃO têm condições (transição = nome de sinal) ⇒ regras leem «a ver» por On Enter/On Exit
  (já existia); script lê `ph2d.ray(id).seeing`. Planilha de Eventos abandonada, intocada.
- **RS-12, ordem do dono:** «Also ignore» = lista de objetos por conta-gotas (a crítica recomendava caixa de tags; a ordem do dono vence).
  `Target::StableSet` novo (`property_ref` `name_of`/`id_of`); canal = nomes unidos por «, » (`canais_componente` `LISTA_SEP`). Listar um objeto
  ignora-o e a família abaixo; escolher uma Collision Shape ignora o corpo dela.
- `ProbeMark.feixe` (`Feixe`: largura, meio_cone, pontos) → `ph2d-app-physics` `overlay/probes.rs::feixe_path` (fatia do leque; cápsula +
  círculo inteiro no acerto). `PROJECT_SCHEMA` 243 → 244 (sem leitor velho).

**Script (`crates/ph2d-script`).** `ph2d.ray(entity) -> RayView?` · `ph2d.raycast(from: Vec2, to: Vec2, filter: RayFilter?) -> {RayHit}`;
tipos `RayHit/RayView/RayFilter` em `api_table` + `api_help_props`; `lane_ray.rs` (HealthRequest/NoDoors mudaram para lá; `lane.rs` 695 linhas);
`SceneDoors::ray_view/raycast`; o filtro passa por `property::set_in` (novo) sobre um `RaySensor` por omissão — mesmos nomes do cartão,
ângulos em radianos. `ScriptDoors` ganha `physics: Option<&PhysicsBridge>`; a shell passa `Some(&*physics)` em `fase_signal_outbox.rs`.
`runtime/luau/ph2d.d.luau` regenerado por `cargo run -p ph2d-bindgen -- --write`.

**Crate NOVA `crates/ph2d-ray-sensor`** (`inspector.rs`, `topo.rs`, `gotas.rs`, `script.rs`): a família do raio saiu de `ph2d-app-components`
(reexportada como `ray_inspector`, `ray_topo`, `ray_gotas`). Entrada 989 (889 medido) em `TETOS_DAS_CRATES_DA_FAMILIA`.
Contadores: `ph2d-app-components` 80 396 / 80 869 · `ph2d-app-game` 2 865 / 2 869 · shell sob o tecto.

**Cartão.** `sections/ray.rs` (455) + `ray_rumo.rs` (novo, 220: Angle/Turns/Aim at, Reach|Layer, Thickness|Cone) + `ray_filtro.rs` (chips de
Also ignore com ×, conta-gotas; resumo «Also ignores N»). Enabled é a 1.ª linha (em cartões partilha a linha com Angle). Catálogo
`ray_sensor.13..17` = Enabled, Thickness, Cone, Aim at, Also ignore; todos canais (Timeline); dica em cada linha nova.

**Medido RS-9** (`mede_o_custo_de_see_all_hits`, 200 peças, `--profile smoke`, mínimo de 7×50 tiques, tique inteiro): linha primeiro 101,8 µs ·
linha todos 133,4 · círculo primeiro 99,9 · círculo todos 183,0 (1.ª versão a varrer de novo por acerto: 1 196,3 — recusada) · cone 60° 209,7
⇒ sem tecto de acertos (o `max_results=32` do Godot seria tecto sem recurso). Tabela na doc de `cast_ray_query_all`.

**Oráculo Godot** (MIT, 4.7.2, headless) CORRIDO sobre a NOSSA cena: `docs/GameEngineFuncional/componentes/oraculo_raio_rs/` (`oraculo.gd`,
`corre.sh`, `godot_rs.txt`); gate `oraculo_godot_das_melhorias` (`crates/ph2d-physics-ecs/tests/it/ray_sensor_oraculo.rs`) por passo: linha ·
desligado · `add_exception` = also_ignore · todos os acertos · ShapeCast círculo 0,6 (nosso 0,2264 exato, dentro do safe 0,2227 / unsafe 0,2402
do Godot, mesmo ponto de contacto) · `intersect_ray` = `raycast` · alvo apagado → o seguinte no mesmo tique (RS-11).

**Cena 26** (`crates/ph2d-physics-scene/src/cena_ray.rs`): «Guard» (alto, à direita) com «Guard Eye» (Cone 50°, ângulo 211°, alcance 6,5 m,
Only tag Player; o herói tem a tag Player) → «Alarm Light» (spotted/lost); «Turret» no ar em (0,6; 0,8) com `turret.luau` (`ph2d.raycast` 3 m para
baixo, 0,5 m de largura, Only tag Player) → «Shot Light» (turret_hit/turret_miss). Foto `ray_cone` (Guard Eye + Filter aberto). Fotos em
`componentes/fotos_07/`: `26_rs_cartao`, `26_rs_cone`, `26_rs_play`, `26_rs_play_turret` (+ `_cards_off` das duas primeiras). Vistas: campos
presentes, nenhum rótulo cortado salvo o clássico «Ignore my othe…» (regra da elipse da casa).

**Gates novos.** `ph2d-physics-ecs` `ray_sensor_melhorias` (`desligar_a_ver_emite_on_exit_e_cala`, `o_circulo_ve_todos_os_acertos_por_distancia`,
`o_leque_ve_quem_esta_dentro_e_nao_tapado`, `aim_at_acerta_o_alvo_na_pose_viva_no_mesmo_tique`, `also_ignore_tira_o_objecto_a_familia_e_o_corpo_da_forma`,
`o_acerto_diz_a_forma_que_tocou`, `dois_raios_em_ordem_trocada_dao_as_mesmas_arestas`, `o_tiro_ve_o_mesmo_que_o_raio_com_o_mesmo_filtro`) ·
`ray_sensor_oraculo` (`oraculo_godot_das_melhorias`; `mede_o_custo_de_see_all_hits` `#[ignore]`) · unit `o_rumo_fica_dentro_do_leque` ·
`ph2d-panel-inspector` `seam_ray_melhorias` (5, cliques reais nos dois modos) + tabelas de `seam_ray_auditoria` alargadas ·
`ph2d-app-components` `as_melhorias_do_raio_animam_pelo_canal` · `shells/game-scenes` `jogo_ray_melhorias`
(`na_cena_26_o_guarda_ve_o_heroi_pelo_leque`, `na_cena_26_a_torreta_atira_por_script` — o script passa pela física do quadro; o controlo sem
física falha com o motivo) · `jogo_ray_feixe` (`o_leque_e_a_fatia_e_o_raio_grosso_e_a_capsula`) ·
`jogo_conta_gotas_canvas::o_clique_no_canvas_escolhe_a_mira_e_a_lista_do_raio`.
**Mutação 20/20 sangrou** (um sobrevivente de início: Aim at a ler o Transform — curado despachando em saltos de 20 tiques).

**Fecho.** fmt limpo; clippy `--all-targets -D warnings` exit 0 nas 15 crates tocadas; 2.ª corrida do binário smoke:
``Finished `smoke` profile [optimized] target(s) in 0.24s``.

## Aberto

- **Smoke do dono das melhorias** (RS-1..RS-12) — pendente.
- **`oraculo_lsp` G5b** (os exemplos de `ph2d.ray`/`ph2d.raycast` no luau-lsp) NÃO correu: `luau-lsp` não está instalado nesta máquina.
- Próximo item da fila: **Motion Input** ([`23_refundacao_plano.md`](../23_refundacao_plano.md)), começando pela TABELA DE CAPACIDADES antes das maquetes.
- Integração: só por ordem do dono; fecho da linha (§1.5.9) e a entrada do §5 do CLAUDE.md nela. (Smokes anteriores — auditoria curada e a quinta onda — já fechados/registados acima.)
