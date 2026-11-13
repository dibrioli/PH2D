# HANDOFF de continuação — line/JointEditor, W2 feita (12/11/2026)

Leitor: a próxima LLM da linha. Ramo `line/JointEditor`, worktree `Worktrees/line-JointEditor`. Commits da W2: `ca75ff30cf` (ponte) ·
`031edd57bb` + `0da4f8d570` (Inspector) · `6955fa7e3e` (cena 123 + fotos). Não é handoff de integração (a linha não fechou).
Plano: [01_plano.md](../01_plano.md) · números: [02_gabaritos_medidos.md](../02_gabaritos_medidos.md) ·
W1: [HANDOFF_CONTINUACAO_W1_2026-11-12.md](HANDOFF_CONTINUACAO_W1_2026-11-12.md) ·
memória `project-memory/project_joint_editor_version_b_2026_11_12.md`.

## 1. O que a W2 entregou

### 1.1 A ponte sob carga (cura medida)

- `ph2d-physics` `world/chains.rs`: Pins com `JointDesc::one_chain` (habilitado, inquebrável, sem motor: `takes_one_chain`) ganham um
  gémeo multibody. Árvores crescem em largura a partir de CADA corpo fixo (a ponte = duas árvores que se encontram por UM Pin de
  impulso). O `ImpulseJoint` continua sendo o registro (handle, âncoras, desenho) e fica desabilitado enquanto o gémeo existe; `retune`
  copia dados; spawn/remove/remove_body marcam sujo, reconstrói no início do passo (`refresh_chains`) semeado das poses reais; o
  checkpoint leva `Chains`; `joint_is_enabled` diz true para gémeos; novo `joint_is_in_one_chain`.
- `PhysicsJoint.one_chain` no FIM. `PROJECT_SCHEMA` continua **238** (mesmo degrau da W1; descrição estendida em `project_schema.rs`).
  Catálogo campo 28 «Solve as one chain»; `properties`/`merge` carregam.
- Gabarito Bridge: Pins `one_chain` no meio de cada vão (`Anchor::At(Spot ±1,04)`).
- Tabela medida: [02_gabaritos_medidos.md §«A cura (W2)»](../02_gabaritos_medidos.md) (não copiada). Resumo: tábua de fábrica 200:1 —
  Rope 0,944 (mínimo +0,013), Pin de impulso 1,017 (caixa escorrega), uma corrente 1,283 (+0,740); a 2000:1 o impulso esmaga e a
  corrente segura; árvore única explode. Sonda `sonda_da_ponte` (ignorada) em `ph2d-physics/src/world/bridge_chain_probe.rs`.

### 1.2 Inspector

- Título do cartão pelo papel («Front wheel · Car 1»): `ph2d_app_physics::joint_assembly_card::card_context`.
- Rótulos da Wheel: Travel down/up, Suspension stiffness/damping, Drive, Spin speed (`panel-inspector sections/joint_assembly_rows.rs`).
- Caixa «Same as <papéis>» (`INSP_JOINT_LINKED`) escreve `JointAssembly.linked`; fan-out de grupo `with_linked` em
  `shells/desktop/src/render_loop/fase_physics_edits.rs` (partilha = edições de afinação; Kind, Drive/MotorEnabled, Active e gestos de
  objeto NÃO se espalham).
- Caixa «Solve as one chain» (`INSP_JOINT_ONE_CHAIN`) só em Pin sem motor e inquebrável.
- `DropTarget::JointBody`: arrastar um objeto para Body A/B religa por `set_joint_body` (`joint_drop::{is_bindable_body, drop_body,
  drop_disarming}`, compartilhado com o conta-gotas); a linha inteira é alvo (`INSP_JOINT_BODY_ROW_A/B`).
- Bloco «Joints» no cartão de física do corpo (ambos os painters, `paint_body_joints`, snapshot `set_current_inspector_body_joints`,
  `joint_assembly_card::body_joints`, ids dinâmicos `insp_body_joint_row(i)` registrados por quadro); clicar seleciona a junta.
- Rod: Stiffness/Damping já não eram pintados (`kind_has_spring` exclui Rod) — não há controle morto.
- Cena 123 respeita `PH2D_PHYSICS_SMOKE_SELECT=<nome>` (então não empurra o painel Physics: o Inspector fica à frente).

## 2. Provas (verdes, por nome)

- `ph2d-physics` `one_chain::` (5); `the_bridge_holds_itself_and_a_load` (tábua do meio > 0,9; a caixa agora é forma filha, ADR-0199);
  gate da cena (caixa > 1,2); teste do catálogo atualizado.
- `seam_joint_w2.rs` (5, em `ph2d-panel-inspector/tests/it`); `the_car_wheel_card_titles_links_and_lists_its_joints`
  (`joint_assembly_tests.rs`); `as_linhas_body_da_junta_recebem_um_objecto` (`asset_drop_field_tests.rs`);
  `largar_um_corpo_em_body_a_religa_o_lado_a_e_recusa_o_resto` (`joint_drop.rs`); testes unitários das funções de rótulo.
- Mutações: sangraram `edit_is_shared`, `offers_one_chain`, o evento Linked. **Uma sobreviveu:** inverter só o estado DESENHADO da
  caixa Linked (o clique lê o valor guardado, não o desenhado).
- Encolhimento da shell na onda: −5 linhas (`the_shell_only_shrinks` verde).
- Fotos (cena 123): [ponte com peso](Worktrees/line-JointEditor/docs/JointEditor/fotos/w2_ponte_com_peso.png) (com
  `PH2D_PHYSICS_SMOKE_TOCA=1`, 3 s: caixa em repouso, ponte plana) ·
  [roda](Worktrees/line-JointEditor/docs/JointEditor/fotos/w2_cartao_roda.png) («Car Body : Car Front Wheel», título certo mas cartão DOBRADO) ·
  [lista de juntas](Worktrees/line-JointEditor/docs/JointEditor/fotos/w2_lista_juntas.png) («Car Body»; bloco Joints abaixo da dobra, exige rolar) ·
  [junta da ponte](Worktrees/line-JointEditor/docs/JointEditor/fotos/w2_ponte_junta.png) («Plank 4 : Plank 5», «Plank · Bridge 1», dobrado).
  Cena de smoke não consegue desdobrar cartão (`SceneSetup` sem campo; acrescentar um cresce a shell).

## 3. Contadores como DELTA (para o integrador)

- `PROJECT_SCHEMA`: a W2 não soma degrau; o total da linha segue **+1** (238). Registro de componentes **+1** (W1) · ADR **+1** (W1) ·
  cenas **+1** (W1) · crates novas na W2: **0**.
- i18n novas: `panel.inspector.joint.{role_title,same_as,travel_down,travel_up,suspension_stiffness,suspension_damping,drive,spin_speed}`,
  `panel.inspector.joints.title`, `component.field.joint.28`.
- Superfície de colisão extra da W2: `ph2d-physics/src/world{,/step,/joints,/joint_desc,/checkpoint,/joint_break}.rs`,
  `physics-ecs/{joint,joint/properties,bridge/joint_desc}.rs`, `ph2d-preview-drive/src/facto.rs`, `project_schema.rs`,
  `panel-inspector` (`lib.rs`, `paint.rs`, `state_components.rs`, `ids/inspector_joint.rs`, `sections/*`), `editor-core` (`lib.rs`,
  `screens/mod.rs`), `asset_drag_wire.rs`, `fase_physics_edits.rs`, `snapshots_inspector.rs`, `asset_drop.rs`. Um merge pode partir:
  a escada/tripla do schema (descrição estendida), o número do ADR, o contador de cenas.

## 4. ABERTO (declarado)

1. ⚠️ **Gate VERMELHO de propósito:** `every_registered_physics_component_has_a_ui_writer` (`ph2d::physics::JointAssembly`). A W3 fecha
   acrescentando o arquivo da aba à lista E registrando que `joint_assembly_card.rs` (caixa Linked) também escreve `JointAssembly` pelo
   Inspector. Sem isto a linha não integra.
2. ⚠️ **Gate de LOC `the_shell_only_shrinks_and_every_app_crate_too` VERMELHO:** `ph2d-app-physics` 39.521 vs 37.798; `ph2d-app-components`
   80.932 vs 80.869 (quase tudo da W1; W2 ≈ +75 / +12). Cura = cortar/mover código no fechamento da linha; nunca subir o número.
3. Botão «Open in Joint Editor» → W3.
4. Duas caixas de aceitar aparecem numa linha Body durante o arraste (linha + ícone); `field_marks` intocado.
5. O rótulo de carga «0 N» sobrepõe o anel da junta selecionada (pré-existente).
6. O gémeo one_chain não tem reação medida (carga lê 0/None); reconstruir em pleno Play zera a velocidade do elo; Pins com motor ou
   inquebráveis continuam de impulso (não medido); o gabarito Chain continua Rope (não medido sob carga).
7. Título e rótulos provados por teste unitário das funções de rótulo, não pelo texto pintado (o kit só vê ids).
8. fmt reformatou duas chamadas em `sections/joint.rs`; gates de paridade de fiação do painel e de LOC do painel NÃO foram rodados.
9. As 63 cenas de smoke antigas com `Collider` no corpo (do handoff W1) seguem decisão do dono.
10. Ícone da montagem na Hierarchy ainda vazio (W3).

## 5. PRÓXIMO: W3 (a aba Joint Editor)

Plano §W3. Antes: fechar §4.1 (gate do escritor de UI), levar em conta §4.2 (corte de LOC) e §4.3/§4.10 (botão e ícone). Confira o
código antes de editar.

## 6. Smoke do dono (W2)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 ./target/smoke/ph2d-host-desktop`
2. Aperte Play (o triângulo na barra da Timeline). A caixa azul cai na ponte e ela quase não afunda. Aperte Stop.
3. Na Hierarchy clique «Car Body : Car Front Wheel» (role a lista). No Inspector o cartão se chama «Front wheel · Car 1»; clique no
   título para abrir: «Same as Rear wheel» marcado, Travel down/up, Suspension stiffness, Drive, Spin speed. Mude Suspension
   stiffness; clique «Car Body : Car Rear Wheel»: o número é o mesmo.
4. Clique «Car Body». No cartão de física há o bloco «Joints» com as duas rodas; clique uma e o cartão da junta abre.
5. Clique «Plank 4 : Plank 5»: «Plank · Bridge 1», «Solve as one chain» marcado. Desmarque e aperte Play: a ponte afunda muito mais.
   Marque de novo.
6. Arraste «Crate» da Hierarchy para a linha «Body B» de uma junta: a linha acende e, ao soltar, Body B passa a ser Crate (Ctrl+Z desfaz).

Deu errado se: a ponte afunda até o chão com a caixa em cima, o título diz só «Joint», mudar uma roda não muda a outra, a lista Joints
não aparece, ou soltar não troca o corpo.

## 7. Prompt de continuação (cole na janela nova)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && pwd && git branch --show-current
```
Confirme `line/JointEditor`. Leia, nesta ordem: `docs/JointEditor/01_plano.md`, a memória
`project-memory/project_joint_editor_version_b_2026_11_12.md`, `docs/JointEditor/handoffs/HANDOFF_CONTINUACAO_W1_2026-11-12.md` e
`docs/JointEditor/handoffs/HANDOFF_CONTINUACAO_W2_2026-11-12.md`. A W1 e a W2 estão feitas (a ponte sob carga está curada e medida; o
Inspector fala pelo papel). Comece a **W3** (a aba Joint Editor, plano §W3) — §5 do handoff W2. Feche o gate
`every_registered_physics_component_has_a_ui_writer` acrescentando o arquivo da aba E `joint_assembly_card.rs` à lista, e planeje o corte
de LOC de `ph2d-app-physics` (§4.2). Inner loop = `cargo-check-narrow.sh`; testes por nome; sem push.
```
