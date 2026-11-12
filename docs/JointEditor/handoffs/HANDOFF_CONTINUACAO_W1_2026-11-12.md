# HANDOFF de continuação — line/JointEditor, W1 feita (12/11/2026)

Leitor: a próxima LLM da linha. Ramo `line/JointEditor`, worktree `Worktrees/line-JointEditor`. Commit da W1: `e3cc56d9eb`.
Plano: [01_plano.md](../01_plano.md) · números: [02_gabaritos_medidos.md](../02_gabaritos_medidos.md) ·
[ADR-0201](../../architecture/decisions/0201-a-montagem-de-juntas-e-um-objeto-e-o-gabarito-e-dado-puro.md) (número por recontar na integração) ·
memória `project-memory/project_joint_editor_version_b_2026_11_12.md`. Não é handoff de integração (a linha não fechou).

## 1. O que a W1 entregou (sem tela)

- Crate nova `crates/ph2d-joint-templates` (dados puros). `lib.rs` = modelo: `Template`, `Role`, `Guide`, `Many`, `JointRule`,
  `Anchor` (`At` | `Ends` | `Seam`), `Spot` (frações da caixa), `Link` (`Pair` | `Chain` | `ToFirst` | `ToLast`), `Tune`, `Motor`,
  `Length::Gap`. `JointRule::joint(gap)` = `PhysicsJoint::of_kind` + as diferenças. `Template::part_name(role, n)`;
  buscas `by_id` / `by_name` / `of_kind`.
- Catálogo em `src/catalog.rs` + `src/catalog/{wheels,hinges,rails,lines}.rs`: **18 gabaritos**, ids 1..18 **pinados**
  (gate `the_template_ids_are_pinned`, append-only); Custom não tem gabarito.
- `ph2d::physics::JointAssembly { template: u16, parts: Vec<AssemblyPart{role, body}>, linked: u32 }` em
  `crates/ph2d-physics-ecs/src/assembly.rs` (body 0 = cenário). `PhysicsJoint.assembly: u64` + `role: u16` **no FIM**
  (`role` = índice da REGRA, não do papel). Uma fonte só: a junta aponta para a montagem.
- `PROJECT_SCHEMA` 237 → **238** (registo de componentes +1). Remapeador `remap_assembly_refs` (e `remap_joint_refs` remapeia
  `assembly`), linha em `instance_refs.rs`. Catálogo: `JOINT_ASSEMBLY` r(1), campo 26 `assembly` da junta, `role` em hiding
  OtherEditor; rótulos i18n; porta Gravado em `ph2d-app-macros/src/mundo/portas.rs`; `with_properties_of` NÃO cola
  assembly/role (identidade); merge a três em `ph2d-preview-drive/src/facto.rs` leva os dois. `ColliderPose::reach` ficou `pub`.
- Porta de montagem `crates/ph2d-app-physics/src/joint_assembly.rs`: `new_assembly`; `place_part(sim, asm, role, Option<Entity>)
  -> Result<Vec<Entity>, PlaceError>` (`NotAnAssembly`/`UnknownRole`/`RoleTaken`/`WorldNotAllowed`/`AlreadyPlaced`/`NoSuchObject`);
  `sync_joints` (cria pares que faltam, apaga os que perderam o par, **não toca nos existentes**; a ponta `ToLast` segue a última
  tábua); `assemble` (desfaz tudo ao 1.º erro); `dissolve`; `joints_of`. Cria por `create_joint_at` / `create_world_pin_at`.
  Papel `may_be_world` sem peça = junta presa ao mundo; a polia mantém o `max_length` medido pela criação; eixo da regra =
  rotação da entidade-junta.
- Cena `PH2D_PHYSICS_SMOKE=123` (pausada; CENAS 122→123): `crates/ph2d-app-physics/src/physics_smoke_joint_templates.rs`.
  Door (batente = mundo) x=2,5 · Bridge x=7 com Crate de 10 kg · Elevator x=11,5 · Car x=14,5 · muro x=21.

## 2. Provas (verdes, por nome)

- `joint_assembly_tests.rs` (10 + 1 sonda ignorada `probe_factory_numbers`): `every_template_assembles_its_factory_parts_and_holds`,
  `the_car_drives_forward_on_its_suspension` (mutação `SPIN=0` sangrou), `the_car_joints_carry_their_role_and_factory_numbers`,
  `the_door_swings_and_stops_at_its_limit`, `the_bridge_holds_itself_and_a_load`, `the_far_end_follows_the_last_plank_whatever_the_order`,
  `the_elevator_rises_one_storey_and_stops`, `an_assembly_round_trips_through_the_snapshot`, `a_refused_part_is_named_and_leaves_nothing_behind`,
  `restoring_the_snapshot_before_the_gesture_removes_the_whole_assembly`.
- `ph2d-joint-templates` `tests/it/catalog.rs` (6); cena `the_template_scene_shows_four_named_machines_that_work`; `smoke_tests` (router) 4.
- fmt e clippy `-D warnings` limpos nas 8 crates tocadas. Verificador: shell compila; `every_registered_component_is_described`,
  `project_schema`, `project_file_round_trips_through_postcard`, gates do remapeador, `joint_persistence`, `ref_remap`, `grava*`,
  `canal_componente` verdes.
- Números medidos (sonda, 5 s, [doc 02](../02_gabaritos_medidos.md)): carro +9,092 m (chassi −0,077 m); moto +9,776 m; elevador
  +3,000 m; pistão +0,800 m; ponte flecha 8,3 cm; trampolim −3 mm; soft platform −8 mm; flipper vai a −0,5 rad.

## 3. Contadores como DELTA (para o integrador)

- `PROJECT_SCHEMA` **+1** (237 → 238 contado a partir do main de 12/11; reconte com `python3 scripts/schema-recount.py`).
- Registo de componentes **+1** (`JointAssembly`). ADR **+1** (escrito 0201; reconte). CENAS de física **+1** (122 → 123).
- Crate nova **+1** (`ph2d-joint-templates`; `Cargo.lock` +8 linhas).
- Superfície de colisão provável: `physics-ecs/{joint,lib,ref_remap}.rs`, `component-desc` (catálogo de física), `i18n/component_catalog.rs`,
  `project_schema{,_tests}.rs`, `smoke.rs` (lista de cenas), `decisions/README.md`. Um merge pode partir: a escada/tripla do schema, o
  número do ADR e o contador de cenas — são os três que somam entre linhas.

## 4. ABERTO (declarado)

1. ⚠️ **Gate VERMELHO de propósito:** `ph2d-panel-inspector every_registered_physics_component_has_a_ui_writer` reprova para
   `ph2d::physics::JointAssembly` — nenhum caminho de UI o escreve até a W3 (a aba). **NÃO** pôr `joint_assembly.rs` na lista
   `WRITERS` antes de a aba existir (calaria o alarme: hoje só a cena de smoke chega lá). Fechar na W3 acrescentando o arquivo da
   aba à lista. Mesmo intervalo que o gate já viveu no `PlatformPlayer`. Sem isto fechado a linha **não integra**.
2. A montagem aparece na Hierarchy como objeto sem tipo (ícone vazio, `ObjectType::Empty`). Tipo/ícone próprio é W2/W3:
   `hier_types.rs` `ObjectType` + `IconId` + `ColorToken` + gates de ícone/cor.
3. Undo de shell provado só no retrato do mundo; o teste dirigindo Ctrl+Z real vem com o gesto da W3.
4. W4 «Make the parts» tem de criar corpos pela regra do [ADR-0199](../../architecture/decisions/0199-a-forma-de-colisao-e-um-objeto-filho-do-corpo.md)
   (forma de colisão = filha, `nasce_com_forma` / `torna_corpo`), **não** com Collider no corpo como os testes e a cena fazem;
   reusar `Template::part_name`.
5. `linked` / grupo: o carro tem as duas rodas no grupo 0; a edição em grupo é W2.
6. Os números não medem carga (força do elevador/polia com peso, carro em rampa) — doc 02 §«O que esta tabela NÃO mede».

## 5. PRÓXIMO: W2 do plano (Inspector melhor)

Título do cartão pelo papel («Front wheel · Car 1»); rótulos por tipo com unidade; some Stiffness/Damping na Rod (controle morto);
caixa «Same as the rear wheel» (edita o grupo); cartão **Joints** no Inspector do corpo com «Open in Joint Editor»; Body A/B aceitam
arrastar (`DropTarget::JointBody`). Prova: testes de costura dirigindo o clique/arraste real → o `PhysicsJoint` muda. Caixas, nunca
toggles (dono 10/11). O Inspector não perde nenhum gesto de hoje. Mapa de arquivos: tabela do plano §«O que já existe».
Antes de editar, confira o código: o mapa é de 12/11.

## 6. Smoke do dono (W1)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && PH2D_PHYSICS_SMOKE=123 ./target/smoke/ph2d-host-desktop`
2. Na lista «Hierarchy» à esquerda aparecem «Door 1», «Bridge 1», «Elevator 1», «Car 1» — cada uma é uma máquina montada por um
   gabarito; as linhas com «:» são as juntas (ex.: «Plank 1 : Plank 2»).
3. Aperte o Play (o triângulo na barra da Timeline embaixo). Tem de acontecer: a porta da esquerda abre e para; a caixa azul cai na
   ponte, que afunda e segura; a plataforma do elevador sobe e para no alto; o carro anda para a direita e para no muro.
4. Deu errado se: alguma peça cai pelo chão, a ponte se solta, o elevador não sobe, o carro não anda ou atravessa o muro.

Fotos: [parada](Worktrees/line-JointEditor/docs/JointEditor/fotos/w1_cena123_parada.png) e
[tocando](Worktrees/line-JointEditor/docs/JointEditor/fotos/w1_cena123_tocando.png).

## 7. Prompt de continuação (cole na janela nova)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-JointEditor && pwd && git branch --show-current
```
Confirme `line/JointEditor`. Leia, nesta ordem: `docs/JointEditor/01_plano.md`, a memória
`project-memory/project_joint_editor_version_b_2026_11_12.md` e `docs/JointEditor/handoffs/HANDOFF_CONTINUACAO_W1_2026-11-12.md`.
A W1 está feita (commit e3cc56d9eb). Comece a **W2** (Inspector melhor) — §5 do handoff. Não feche o gate
`every_registered_physics_component_has_a_ui_writer` antes da W3. Inner loop = `cargo-check-narrow.sh`; testes por nome; sem push.
