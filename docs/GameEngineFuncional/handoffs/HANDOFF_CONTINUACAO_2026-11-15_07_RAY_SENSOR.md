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

## Aberto

- Smoke do dono da cena 26 (passos na conversa de 15/11).
- `PH2D_RAY_SMOKE=1` (cena antiga) segue com olhos na raiz; não foi refeita como filhos.
- Próximo item da fila: **Motion Input** (`23_refundacao_plano.md`, «Corpo e movimento»).
- Fecho da linha (§1.5.9) e a entrada do §5 do CLAUDE.md: na integração.
